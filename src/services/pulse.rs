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

use crate::{
    Config, Error, Result,
    services::{HttpClient, RequestBuilderExt},
};
use chrono::Utc;
use reqwest::{
    Method, StatusCode,
    header::{self, HeaderName},
};
use reqwest_middleware::RequestBuilder;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use url::Url;

use super::{ForceScheme, core::WorkspaceUuid};

pub struct PulseClient {
    token: SecretString,
    http: HttpClient,
    base: Url,
}

const PULSE_TTL_HEADER: HeaderName = HeaderName::from_static("huly-ttl");
const PULSE_EXPIRE_AT_HEADER: HeaderName = HeaderName::from_static("huly-expire-at");

#[derive(Deserialize)]
struct ObjectResponse {
    key: String,
    data: String,
    expires_at: u64,
    etag: String,
}

#[derive(Debug, Default, Clone)]
pub enum PutMode {
    #[default]
    Upsert, // default: set or overwrite
    Insert,        // only if not exists (NX)
    Update,        // only if exists (XX)
    Equal(String), // only if md5 matches provided
}

#[derive(Debug, Clone)]
pub struct FullObject {
    pub key: String,
    pub data: String,
    pub expires_at: Expiration,
    pub etag: String,
}

impl From<ObjectResponse> for FullObject {
    fn from(
        ObjectResponse {
            key,
            data,
            expires_at,
            etag,
        }: ObjectResponse,
    ) -> Self {
        FullObject {
            key,
            data,
            expires_at: Expiration::InSeconds(expires_at),
            etag,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Expiration {
    InSeconds(u64),
    AtTime(chrono::DateTime<Utc>),
}

fn make_rest_api_endpoint(url: Url) -> Result<Url> {
    let url = if matches!(url.scheme(), "ws" | "wss") && url.path().ends_with("/ws") {
        let mut url = url.force_http_scheme();
        url.path_segments_mut()
            .map_err(|_| Error::Other("InvalidPulseUrl"))?
            .pop()
            .push("api")
            .push("");
        url
    } else {
        url
    };
    if !matches!(url.scheme(), "http" | "https") || !url.path().ends_with("/api/") {
        return Err(Error::Other("InvalidPulseUrl"));
    }
    Ok(url)
}

impl PulseClient {
    pub fn new(
        config: &Config,
        http: HttpClient,
        workspace: WorkspaceUuid,
        token: SecretString,
    ) -> Result<Self> {
        let base = config
            .pulse_service
            .as_ref()
            .ok_or(Error::Other("NoPulse"))?
            .clone();
        let base = make_rest_api_endpoint(base)?;

        Ok(Self {
            http,
            base: base.join(&format!("{workspace}/"))?,
            token,
        })
    }

    fn request(&self, method: Method, url: Url) -> RequestBuilder {
        self.http
            .request(method, url)
            .bearer_auth(self.token.expose_secret())
    }

    pub async fn list(&self, key_prefix: &str) -> Result<Vec<FullObject>> {
        let request = self.request(Method::GET, self.base.join(&format!("{key_prefix}/"))?);
        let response = request.send_ext().await?;
        let objects: Vec<ObjectResponse> = response.json().await?;

        Ok(objects.into_iter().map(Into::into).collect())
    }

    pub async fn get(&self, key: &str) -> Result<Option<FullObject>> {
        let request = self.request(Method::GET, self.base.join(key)?);
        let response = request.send().await?;
        if response.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else if response.status().is_success() {
            let object: ObjectResponse = response.json().await?;
            Ok(Some(object.into()))
        } else {
            let status = response.status();
            let body = response.text().await?;

            Err(Error::HttpError(status, body))
        }
    }

    pub async fn put(
        &self,
        key: &str,
        data: String,
        expiration: Option<Expiration>,
        mode: PutMode,
    ) -> Result<()> {
        let request = self.request(Method::PUT, self.base.join(key)?);

        let request = match expiration {
            Some(Expiration::InSeconds(secs)) => request.header(PULSE_TTL_HEADER, secs),
            Some(Expiration::AtTime(time)) => {
                request.header(PULSE_EXPIRE_AT_HEADER, time.timestamp() as u64)
            }
            None => request,
        };
        let request = match mode {
            PutMode::Upsert => request,
            PutMode::Insert => request.header(header::IF_NONE_MATCH, "*"),
            PutMode::Update => request.header(header::IF_MATCH, "*"),
            PutMode::Equal(etag) => request.header(header::IF_MATCH, etag),
        };
        request.body(data).send_ext().await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str, mode: PutMode) -> Result<()> {
        let request = self.request(Method::DELETE, self.base.join(key)?);
        let request = match mode {
            PutMode::Upsert => request,
            PutMode::Insert => request.header(header::IF_NONE_MATCH, "*"),
            PutMode::Update => request.header(header::IF_MATCH, "*"),
            PutMode::Equal(etag) => request.header(header::IF_MATCH, etag),
        };
        request.send_ext().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::make_rest_api_endpoint;

    #[test]
    fn test_make_rest_api_endpoint() {
        let check_url = |original: &str, expected: &str| {
            let original_url = Url::parse(original).unwrap();
            let expected_url = Url::parse(expected).unwrap();
            let processed = make_rest_api_endpoint(original_url)
                .unwrap()
                .join("workspace")
                .unwrap();
            assert_eq!(processed, expected_url);
        };
        let check_invalid_rejected = |original: &str| {
            let original_url = Url::parse(original).unwrap();
            let processed = make_rest_api_endpoint(original_url);
            assert!(processed.is_err());
        };
        check_url(
            "ws://pulse.on.some.host/ws",
            "http://pulse.on.some.host/api/workspace",
        );
        check_url(
            "wss://pulse.on.some.host/ws",
            "https://pulse.on.some.host/api/workspace",
        );
        check_url(
            "ws://pulse.on.some.host/path/ws",
            "http://pulse.on.some.host/path/api/workspace",
        );
        check_url(
            "wss://pulse.on.some.host/path/ws",
            "https://pulse.on.some.host/path/api/workspace",
        );
        check_url(
            "http://pulse.on.some.host/api/",
            "http://pulse.on.some.host/api/workspace",
        );
        check_url(
            "https://pulse.on.some.host/api/",
            "https://pulse.on.some.host/api/workspace",
        );
        check_url(
            "http://pulse.on.some.host/path/api/",
            "http://pulse.on.some.host/path/api/workspace",
        );
        check_url(
            "https://pulse.on.some.host/path/api/",
            "https://pulse.on.some.host/path/api/workspace",
        );
        check_url(
            "wss://pulse.hc.engineering/ws",
            "https://pulse.hc.engineering/api/workspace",
        );

        check_invalid_rejected("ws://pulse.on.some.host/");
        check_invalid_rejected("ws://pulse.on.some.host/some/path");
        check_invalid_rejected("wss://pulse.on.some.host/api/");
        check_invalid_rejected("http://pulse.on.some.host/");
        check_invalid_rejected("http://pulse.on.some.host/api");
        check_invalid_rejected("https://pulse.on.some.host/some/path");
    }
}
