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

use std::sync::LazyLock;

use reqwest::Client as HttpClient;
use secrecy::{ExposeSecret, SecretString};
use url::Url;

use super::{ForceHttpScheme, Result, jwt::Claims, types::WorkspaceUuid};

pub mod document;
pub mod event;
pub mod person;

pub struct TransactorClient {
    pub workspace: WorkspaceUuid,
    token: SecretString,
    http: HttpClient,
    pub base: Url,
}

static CLIENT: LazyLock<HttpClient> =
    LazyLock::new(|| reqwest::ClientBuilder::default().build().unwrap());

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
}
