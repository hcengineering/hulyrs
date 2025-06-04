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

use std::{sync::LazyLock, time::Duration};

use crate::Result;
use crate::services::ForceHttpScheme;
use crate::services::jwt::Claims;
use crate::services::types::WorkspaceUuid;
use reqwest::{StatusCode, header::HeaderValue};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{
    RetryTransientMiddleware, Retryable, RetryableStrategy, default_on_request_failure,
    policies::ExponentialBackoff,
};
use secrecy::{ExposeSecret, SecretString};
use tracing::*;
use url::Url;

pub mod document;
pub mod event;
pub mod person;

pub type HttpClient = ClientWithMiddleware;

#[derive(Clone)]
pub struct TransactorClient {
    pub workspace: WorkspaceUuid,
    pub base: Url,
    token: SecretString,
    http: HttpClient,
}

impl PartialEq for TransactorClient {
    fn eq(&self, other: &Self) -> bool {
        self.workspace == other.workspace
            && self.token.expose_secret() == other.token.expose_secret()
            && self.base == other.base
    }
}

static CLIENT: LazyLock<HttpClient> = LazyLock::new(|| {
    let policy =
        ExponentialBackoff::builder().build_with_total_retry_duration(Duration::from_secs(30));

    struct TransactorStrategy;

    impl RetryableStrategy for TransactorStrategy {
        #[tracing::instrument(level = "trace", skip_all)]
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
                        let limit_remaining = hstr(success.headers().get("X-RateLimit-Remaining"));
                        let limit_reset = hstr(success.headers().get("X-RateLimit-Reset"));

                        trace!(
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

    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy_and_strategy(
            policy,
            TransactorStrategy,
        ))
        .build()
});

impl super::TokenProvider for &TransactorClient {
    fn provide_token(&self) -> Option<&str> {
        Some(self.token.expose_secret())
    }
}

impl TransactorClient {
    pub fn new(base: Url, claims: &Claims) -> Result<Self> {
        let base = base.force_http_scheme();
        let workspace = claims.workspace()?;
        let token = claims.encode()?;

        let http = CLIENT.clone();
        Ok(Self {
            http,
            workspace,
            token,
            base,
        })
    }

    pub fn from_token(
        base: Url,
        workspace: WorkspaceUuid,
        token: impl Into<SecretString>,
    ) -> Result<Self> {
        let base = base.force_http_scheme();
        let http = CLIENT.clone();
        Ok(Self {
            workspace,
            http,
            base,
            token: token.into(),
        })
    }
}
