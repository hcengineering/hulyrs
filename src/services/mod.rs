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

use reqwest::{self, Response, Url};
use reqwest_middleware::{ClientWithMiddleware as HttpClient, RequestBuilder};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{self as json, Value};
use tracing::*;

pub mod account;
pub mod jwt;
pub mod kvs;
pub mod transactor;
pub mod types;

use super::{Error, Result};

trait RequestBuilderExt {
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

trait JsonClient {
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
