//
// Copyright © 2025 Hardcore Engineering Inc.
//
// Licensed under the Eclipse Public License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License. You may
// obtain a copy of the License at https://www.eclipse.org/legal/epl-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::collections::HashMap;
use std::time::Duration;

use rdkafka::consumer::StreamConsumer;
use reqwest::{self, Response, Url};
use reqwest::{StatusCode, header::HeaderValue};
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::{ClientWithMiddleware as HttpClient, RequestBuilder};
use reqwest_retry::{
    RetryTransientMiddleware, Retryable, RetryableStrategy, default_on_request_failure,
    policies::ExponentialBackoff,
};
use secrecy::SecretString;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{self as json, Value};
use tracing::*;

use super::{
    account::AccountClient,
    jwt::Claims,
    kvs::KvsClient,
    transactor::TransactorClient,
    types::{AccountUuid, WorkspaceUuid},
};
use crate::{Error, Result, config::Config};

#[cfg(feature = "kafka")]
use crate::services::transactor::kafka;

pub trait RequestBuilderExt {
    fn send_ext(self) -> impl Future<Output = Result<Response>>;
}

pub trait TokenProvider {
    fn provide_token(&self) -> Option<&str>;
}

pub trait BasePathProvider {
    fn provide_base_path(&self) -> &Url;
}

pub trait ForceHttpScheme {
    fn force_http_scheme(self) -> Url;
}

impl ForceHttpScheme for Url {
    fn force_http_scheme(mut self) -> Url {
        match self.scheme() {
            "ws" => {
                self.set_scheme("http").unwrap();
            }

            "wss" => {
                self.set_scheme("https").unwrap();
            }

            _ => panic!(),
        };

        self
    }
}

impl RequestBuilderExt for RequestBuilder {
    async fn send_ext(self) -> Result<Response> {
        let response = self.send().await?;

        if response.status().is_success() {
            Ok(response)
        } else {
            let status = response.status();
            let body = response.text().await?;

            Err(Error::HttpError(status, body))
        }
    }
}

pub trait ResponseExt {
    fn json_body<T: DeserializeOwned>(self) -> impl Future<Output = Result<T>>;
}

impl ResponseExt for reqwest::Response {
    async fn json_body<T: DeserializeOwned>(self) -> Result<T> {
        let body = self.text().await?;

        serde_json::from_str::<T>(&body).map_err(|error| {
            error!(%body, %error);
            Error::Serde(error)
        })
    }
}

fn from_value<T: DeserializeOwned>(value: Value) -> Result<T> {
    json::from_value(value).map_err(|error| {
        error!(%error, "Cannot deserialize response");
        Error::Serde(error)
    })
}

pub trait JsonClient {
    fn get<U: TokenProvider, R: DeserializeOwned>(
        &self,
        user: U,
        url: Url,
    ) -> impl Future<Output = Result<R>>;

    fn post<U: TokenProvider, Q: Serialize, R: DeserializeOwned>(
        &self,
        user: U,
        url: Url,
        body: &Q,
    ) -> impl Future<Output = Result<R>>;
}

impl JsonClient for HttpClient {
    #[tracing::instrument(
        level = "trace",
        skip(self, user, url),
        fields(%url, method = "get", type = "json")
    )]
    async fn get<U: TokenProvider, R: DeserializeOwned>(&self, user: U, url: Url) -> Result<R> {
        trace!("request");

        let mut request = self.get(url.clone());

        if let Some(token) = user.provide_token() {
            request = request.bearer_auth(token);
        }

        request.send_ext().await?.json_body::<R>().await
    }

    async fn post<U: TokenProvider, Q: Serialize, R: DeserializeOwned>(
        &self,
        user: U,
        url: Url,
        body: &Q,
    ) -> Result<R> {
        let body = json::to_value(body)?;

        trace!(type="json", %url, method="post", %body, "http request");

        let mut request = self.post(url.clone()).json(&body);

        if let Some(token) = user.provide_token() {
            request = request.bearer_auth(token);
        }

        let response = request.send_ext().await?.json::<Value>().await?;

        trace!(type="json", %url, method="post", %response, "http response");

        Ok(from_value(response)?)
    }
}

#[derive(Deserialize, Debug, Clone, strum::Display)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Ok,
    Info,
    Warning,
    Error,
}

#[derive(Deserialize, Debug, Clone, thiserror::Error)]
pub struct Status {
    pub severity: Severity,
    pub code: String,
    pub params: HashMap<String, String>,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.severity, self.code)
    }
}

pub trait ServiceClient {
    fn service<U: TokenProvider + BasePathProvider, R: serde::de::DeserializeOwned>(
        &self,
        user: U,
        method: &str,
        params: impl Serialize,
    ) -> impl Future<Output = Result<R>>;
}

impl ServiceClient for HttpClient {
    async fn service<U: TokenProvider + BasePathProvider, R: DeserializeOwned>(
        &self,
        user: U,
        method: &str,
        params: impl Serialize,
    ) -> Result<R> {
        let url = user.provide_base_path();

        let params = json::to_value(&params)?;

        trace!(type="service", %url, %method, %params, "http request");

        #[derive(Serialize, Debug)]
        struct Request<'a> {
            method: &'a str,
            params: json::Value,
        }

        #[derive(Deserialize, Debug)]
        struct Response {
            result: Option<json::Value>,
            error: Option<json::Value>,
        }

        let mut req = self.post(url.clone()).json(&Request { method, params });

        if let Some(token) = user.provide_token() {
            req = req.bearer_auth(token);
        }

        let response = req.send_ext().await?.json::<Value>().await?;

        trace!(type="service", %url,  %response, "http response");

        let response = from_value(response)?;

        match json::from_value(response)? {
            Response {
                result: Some(result),
                error: None,
            } => Ok(from_value::<R>(result)?),

            Response {
                result: None,
                error: Some(error),
            } => Err(Error::ServiceError(from_value::<Status>(error)?)),

            Response {
                result: None,
                error: None,
            } => Ok(json::from_value(json::Value::Null)?),

            _ => Err(Error::Other("Unexpected service response")),
        }
    }
}

#[derive(Clone)]
pub struct ServiceFactory {
    config: Config,
    account_http: HttpClient,
    kvs_http: HttpClient,
    transactor_http: HttpClient,
}

impl ServiceFactory {
    pub fn new(config: Config) -> Self {
        #[cfg(feature = "reqwest_middleware")]
        let account_http = {
            let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

            ClientBuilder::new(reqwest::Client::new())
                .with(RetryTransientMiddleware::new_with_policy(retry_policy))
                .build()
        };

        #[cfg(not(feature = "reqwest_middleware"))]
        let account_http = { ClientBuilder::new(reqwest::Client::new()).build() };

        #[cfg(feature = "reqwest_middleware")]
        let kvs_http = {
            let policy = ExponentialBackoff::builder()
                .build_with_total_retry_duration(Duration::from_secs(10));

            ClientBuilder::new(reqwest::Client::new())
                .with(RetryTransientMiddleware::new_with_policy(policy))
                .build()
        };

        #[cfg(not(feature = "reqwest_middleware"))]
        let kvs_http = { ClientBuilder::new(reqwest::Client::new()).build() };

        #[cfg(feature = "reqwest_middleware")]
        let transactor_http = {
            let policy = ExponentialBackoff::builder()
                .build_with_total_retry_duration(Duration::from_secs(120));

            struct TransactorStrategy;

            impl RetryableStrategy for TransactorStrategy {
                #[tracing::instrument(level = "debug", skip_all)]
                fn handle(
                    &self,
                    res: &std::result::Result<reqwest::Response, reqwest_middleware::Error>,
                ) -> Option<Retryable> {
                    match res {
                        Ok(success) => match success.status() {
                            StatusCode::REQUEST_TIMEOUT | StatusCode::TOO_MANY_REQUESTS => {
                                fn hstr(h: Option<&HeaderValue>) -> &str {
                                    h.map(|h| h.to_str().unwrap()).unwrap_or("")
                                }

                                let retry_after = hstr(success.headers().get("Retry-After"));
                                let limit = hstr(success.headers().get("X-RateLimit-Limit"));
                                let limit_remaining =
                                    hstr(success.headers().get("X-RateLimit-Remaining"));
                                let limit_reset = hstr(success.headers().get("X-RateLimit-Reset"));

                                warn!(
                                    code = %success.status(),
                                    retry_after, limit, limit_remaining, limit_reset,
                                    "Transient error"
                                );

                                Some(Retryable::Transient)
                            }

                            other => {
                                if other.is_success() {
                                    None
                                } else {
                                    Some(Retryable::Fatal)
                                }
                            }
                        },
                        Err(error) => default_on_request_failure(error),
                    }
                }
            }

            let retry =
                RetryTransientMiddleware::new_with_policy_and_strategy(policy, TransactorStrategy);

            let rate_limiter = {
                use governor::{
                    Quota, RateLimiter,
                    clock::{Clock, MonotonicClock},
                    middleware::NoOpMiddleware,
                    state::{InMemoryState, NotKeyed},
                };
                use std::num::NonZeroU32;

                pub type DirectRateLimiter = RateLimiter<
                    NotKeyed,
                    InMemoryState,
                    MonotonicClock,
                    NoOpMiddleware<<MonotonicClock as Clock>::Instant>,
                >;

                struct Limiter(DirectRateLimiter);

                impl Limiter {
                    fn new(limit: NonZeroU32) -> Self {
                        let limiter = RateLimiter::direct_with_clock(
                            Quota::per_second(limit).allow_burst(1.try_into().unwrap()),
                            MonotonicClock,
                        );

                        Self(limiter)
                    }
                }

                impl reqwest_ratelimit::RateLimiter for Limiter {
                    async fn acquire_permit(&self) {
                        self.0.until_ready().await;
                    }
                }

                reqwest_ratelimit::all(Limiter::new(config.account_service_rate_limit))
            };

            ClientBuilder::new(reqwest::Client::new())
                .with(rate_limiter)
                .with(retry)
                .build()
        };

        #[cfg(not(feature = "reqwest_middleware"))]
        let transactor_http = { ClientBuilder::new(reqwest::Client::new()).build() };

        Self {
            config,
            account_http,
            kvs_http,
            transactor_http,
        }
    }

    pub fn new_account_client(&self, claims: &Claims) -> Result<AccountClient> {
        AccountClient::new(
            &self.config,
            self.account_http.clone(),
            claims.account,
            claims.encode(
                self.config
                    .token_secret
                    .as_ref()
                    .ok_or(Error::Other("NoSecret"))?,
            )?,
        )
    }

    pub fn new_account_client_from_token(
        &self,
        account: AccountUuid,
        token: impl Into<SecretString>,
    ) -> Result<AccountClient> {
        AccountClient::new(&self.config, self.account_http.clone(), account, token)
    }

    pub fn new_kvs_client(&self, namespace: &str, claims: &Claims) -> Result<KvsClient> {
        KvsClient::new(
            &self.config,
            self.kvs_http.clone(),
            namespace.to_owned(),
            claims,
        )
    }

    pub fn new_transactor_client(&self, base: Url, claims: &Claims) -> Result<TransactorClient> {
        TransactorClient::new(
            self.transactor_http.clone(),
            base,
            claims.workspace()?,
            claims.encode(
                self.config
                    .token_secret
                    .as_ref()
                    .ok_or(Error::Other("NoSecret"))?,
            )?,
        )
    }

    pub fn new_transactor_client_from_token(
        &self,
        base: Url,
        workspace: WorkspaceUuid,
        token: impl Into<SecretString>,
    ) -> Result<TransactorClient> {
        TransactorClient::new(self.transactor_http.clone(), base, workspace, token)
    }

    #[cfg(feature = "kafka")]
    pub fn new_kafka_publisher(&self, topic: &str) -> Result<kafka::KafkaProducer> {
        kafka::KafkaProducer::new(&self.config, topic)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
