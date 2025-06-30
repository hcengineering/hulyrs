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

use crate::Result;
use crate::services::ForceHttpScheme;
use crate::services::types::WorkspaceUuid;
use reqwest_middleware::ClientWithMiddleware;
use secrecy::{ExposeSecret, SecretString};
use url::Url;

pub mod document;
pub mod event;
pub mod person;
pub mod tx;

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

impl super::TokenProvider for &TransactorClient {
    fn provide_token(&self) -> Option<&str> {
        Some(self.token.expose_secret())
    }
}

impl TransactorClient {
    pub fn new(
        http: HttpClient,
        base: Url,
        workspace: WorkspaceUuid,
        token: impl Into<SecretString>,
    ) -> Result<Self> {
        let base = base.force_http_scheme();
        Ok(Self {
            workspace,
            http,
            base,
            token: token.into(),
        })
    }
}
