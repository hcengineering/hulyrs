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

use reqwest_middleware::ClientWithMiddleware as HttpClient;
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{self as json, Value, from_value};
use std::collections::HashMap;
use url::Url;

use crate::config::Config;
use crate::services::Status;
use crate::services::core::WorkspaceUuid;
use crate::services::core::classes::{Markup, Ref};
use crate::{Error, Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetContentRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Ref>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetContentResponse {
    pub content: HashMap<String, Markup>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollaborativeDoc {
    pub object_id: Ref,
    pub object_class: Ref,
    pub object_attr: String,
}

#[derive(Clone)]
pub struct CollaboratorClient {
    workspace: WorkspaceUuid,
    token: SecretString,
    base: Url,
    http: HttpClient,
}

impl PartialEq for CollaboratorClient {
    fn eq(&self, other: &Self) -> bool {
        self.workspace == other.workspace
            && self.token.expose_secret() == other.token.expose_secret()
            && self.base == other.base
    }
}

impl CollaboratorClient {
    pub fn new(
        config: &Config,
        http: HttpClient,
        workspace: WorkspaceUuid,
        token: impl Into<SecretString>,
    ) -> Result<Self> {
        let base = config
            .collaborator_service
            .clone()
            .ok_or(Error::Other("NoCollaboratorService"))?;

        let base = Self::force_http_scheme(base);

        Ok(Self {
            http,
            base,
            workspace,
            token: token.into(),
        })
    }

    pub async fn get_content(
        &self,
        document: &CollaborativeDoc,
        source: Option<Ref>,
    ) -> Result<Markup> {
        let payload = GetContentRequest { source };

        let response = self
            .rpc::<GetContentResponse>(document, "getContent", payload.clone())
            .await?;

        Ok(response
            .content
            .get(&document.object_attr)
            .cloned()
            .unwrap_or_default())
    }

    fn force_http_scheme(mut url: Url) -> Url {
        match url.scheme() {
            "ws" => {
                url.set_scheme("http").unwrap();
            }
            "wss" => {
                url.set_scheme("https").unwrap();
            }
            _ => {}
        };
        url
    }

    fn encode_document_id(&self, document: &CollaborativeDoc) -> String {
        format!(
            "{}.{}.{}.{}",
            self.workspace, document.object_id, document.object_class, document.object_attr
        )
    }

    async fn rpc<R: DeserializeOwned>(
        &self,
        document: &CollaborativeDoc,
        method: &str,
        payload: impl Serialize,
    ) -> Result<R> {
        use crate::services::RequestBuilderExt;

        let document_id = self.encode_document_id(document);
        let payload = json::to_value(&payload)?;

        let url = self
            .base
            .join(&format!("/rpc/{}", document_id))
            .map_err(|_| Error::Other("InvalidUrl"))?;

        #[derive(Serialize, Debug)]
        struct Request<'a> {
            method: &'a str,
            payload: json::Value,
        }

        #[derive(Deserialize, Debug)]
        struct Response {
            result: Option<json::Value>,
            error: Option<json::Value>,
        }

        let response = self
            .http
            .post(url)
            .bearer_auth(self.token.expose_secret())
            .header("Content-Type", "application/json")
            .json(&Request { method, payload })
            .send_ext()
            .await?;

        let response = response.json::<Value>().await?;
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
