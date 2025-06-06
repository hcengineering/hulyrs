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

use super::{RequestBuilderExt, jwt::Claims};
use crate::Result;
use reqwest::{Method, header};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware as HttpClient, RequestBuilder};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use secrecy::{ExposeSecret, SecretString};
use url::Url;

pub struct KvsClient {
    token: SecretString,
    namespace: String,
    http: HttpClient,
    base: Url,
}

static CLIENT: LazyLock<HttpClient> = LazyLock::new(|| {
    let policy =
        ExponentialBackoff::builder().build_with_total_retry_duration(Duration::from_secs(10));

    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(policy))
        .build()
});

impl KvsClient {
    pub fn new(base: &str, namespace: String, claims: Claims) -> Result<Self> {
        let base = base.try_into()?;
        let http = CLIENT.clone();
        let token = claims.encode()?;

        Ok(Self {
            http,
            base,
            namespace,
            token,
        })
    }

    fn request(&self, method: Method, url: Url) -> RequestBuilder {
        self.http
            .request(method, url)
            .bearer_auth(self.token.expose_secret())
    }

    pub async fn upsert(&self, key: &str, value: &[u8]) -> Result<()> {
        let path = format!("api/{}/{}", self.namespace, key);
        let url = self.base.join(&path)?;

        self.request(Method::POST, url)
            .body(value.to_vec())
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .send_ext()
            .await?;

        tracing::trace!(namespace=self.namespace, %key, bytes=value.len(), "upsert");

        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = format!("api/{}/{}", self.namespace, key);
        let url = self.base.join(&path)?;

        let response = self.request(Method::GET, url).send().await?;

        if response.status().is_success() {
            let bytes = response.bytes().await?.to_vec();

            tracing::trace!(namespace=self.namespace, %key, bytes=bytes.len(), "get");

            Ok(Some(bytes))
        } else {
            match response.status() {
                reqwest::StatusCode::NOT_FOUND => Ok(None),

                _ => Err(super::Error::HttpError(
                    response.status(),
                    response.text().await?,
                )),
            }
        }
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let path = format!("api/{}/{}", self.namespace, key);
        let url = self.base.join(&path)?;

        self.request(Method::DELETE, url).send_ext().await?;

        tracing::trace!(namespace=self.namespace, %key, "delete");

        Ok(())
    }
}
