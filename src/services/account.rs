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

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware as HttpClient};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;
use uuid::Uuid;

use super::{Result, ServiceClient, jwt::Claims, types::*};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfo {
    pub account: PersonUuid,
    pub name: Option<String>,
    pub social_id: Option<PersonId>,
    pub token: Option<String>,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountSocialId {
    #[serde(flatten)]
    pub base: SocialId,

    pub person_uuid: PersonUuid,
    pub is_deleted: bool,
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddSocialIdToPersonParams {
    pub person: PersonUuid,
    pub r#type: SocialIdType,
    pub value: String,
    pub confirmed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationKey {
    pub social_id: PersonId,
    pub kind: String,
    pub workspace_uuid: Option<WorkspaceUuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PartialIntegrationKey {
    pub social_id: Option<PersonId>,
    pub kind: Option<String>,

    /// top level None - ignore, second level no ws/specific ws
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_uuid: Option<Option<WorkspaceUuid>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Integration {
    pub social_id: PersonId,
    pub kind: String,
    pub workspace_uuid: Option<WorkspaceUuid>,
    pub data: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceKind {
    #[default]
    Internal,
    External,
    ByRegion,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Location {
    KV,
    WEUR,
    EEUR,
    WNAM,
    ENAM,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub uuid: WorkspaceUuid,
    pub name: String,
    pub url: String,
    pub data_id: Option<WorkspaceDataId>,
    pub branding: Option<String>,
    pub location: Option<Location>,
    pub region: Option<String>,
    pub created_by: Option<PersonUuid>,
    pub billing_account: Option<PersonUuid>,

    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub created_on: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SelectWorkspaceParams {
    pub workspace_url: String,
    pub kind: WorkspaceKind,
    pub external_regions: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceLoginInfo {
    #[serde(flatten)]
    pub base: LoginInfo,

    pub workspace: WorkspaceUuid,
    pub workspace_url: Option<String>,
    pub workspace_data_id: Option<WorkspaceDataId>,
    pub endpoint: Url,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, derive_builder::Builder)]
#[serde(rename_all = "camelCase")]
pub struct EnsurePersonParams {
    pub social_type: SocialIdType,
    pub social_value: String,

    #[builder(setter(into))]
    pub first_name: String,

    #[builder(setter(into, strip_option), default)]
    pub last_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnsurePersonResult {
    pub uuid: PersonUuid,
    pub social_id: PersonId,
}

pub struct AccountClient {
    pub account: AccountUuid,
    token: SecretString,
    base: Url,
    http: HttpClient,
}

impl super::TokenProvider for &AccountClient {
    fn provide_token(&self) -> Option<&str> {
        Some(self.token.expose_secret())
    }
}

impl super::BasePathProvider for &AccountClient {
    fn provide_base_path(&self) -> &Url {
        &self.base
    }
}

static CLIENT: LazyLock<HttpClient> = LazyLock::new(|| {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
});

impl AccountClient {
    pub fn new(claims: &Claims) -> Result<Self> {
        let account = claims.account;
        let base = crate::CONFIG.account_service.clone();
        let http = CLIENT.clone();
        let token = claims.encode()?;

        Ok(Self {
            http,
            base,
            account,
            token,
        })
    }

    pub fn assume_claims(&self, claims: &Claims) -> Result<Self> {
        let account = claims.account;
        let base = self.base.clone();
        let http = self.http.clone();
        let token = claims.encode()?;

        Ok(Self {
            http,
            base,
            account,
            token,
        })
    }

    pub async fn select_workspace(
        &self,
        params: &SelectWorkspaceParams,
    ) -> Result<WorkspaceLoginInfo> {
        self.http.service(self, "selectWorkspace", params).await
    }

    pub async fn ensure_person(&self, params: &EnsurePersonParams) -> Result<EnsurePersonResult> {
        self.http.service(self, "ensurePerson", params).await
    }

    pub async fn get_login_info_by_token(&self) -> Result<LoginInfo> {
        self.http.service(self, "getLoginInfoByToken", ()).await
    }

    pub async fn get_workspace_login_info_by_token(&self) -> Result<WorkspaceLoginInfo> {
        self.http.service(self, "getLoginInfoByToken", ()).await
    }

    pub async fn get_social_ids(&self, confirmed: bool) -> Result<Vec<AccountSocialId>> {
        self.http
            .service(self, "getSocialIds", json!({"confirmed": confirmed}))
            .await
    }

    pub async fn add_social_id_to_person(
        &self,
        params: &AddSocialIdToPersonParams,
    ) -> Result<PersonId> {
        self.http.service(self, "addSocialIdToPerson", params).await
    }

    pub async fn list_integrations(
        &self,
        params: &PartialIntegrationKey,
    ) -> Result<Vec<Integration>> {
        self.http.service(self, "listIntegrations", params).await
    }

    pub async fn get_integration(&self, params: &IntegrationKey) -> Result<Option<Integration>> {
        self.http.service(self, "getIntegration", params).await
    }

    pub async fn create_integration(&self, params: &Integration) -> Result<()> {
        self.http.service(self, "createIntegration", params).await
    }

    pub async fn update_integration(&self, params: &Integration) -> Result<()> {
        self.http.service(self, "updateIntegration", params).await
    }

    pub async fn delete_integration(&self, params: &IntegrationKey) -> Result<()> {
        self.http.service(self, "deleteIntegration", params).await
    }

    pub async fn find_person_by_social_key(
        &self,
        key: &str,
        require_account: bool,
    ) -> Result<Option<Uuid>> {
        let params = json!({"socialString": key, "requireAccount": require_account});
        self.http
            .service(self, "findPersonBySocialKey", params)
            .await
    }

    pub async fn find_social_id_by_social_key(
        &self,
        key: &str,
        require_account: bool,
    ) -> Result<Option<String>> {
        let params = json!({"socialKey": key, "requireAccount": require_account});
        self.http
            .service(self, "findSocialIdBySocialKey", params)
            .await
    }

    pub async fn get_user_workspaces(&self) -> Result<Vec<Workspace>> {
        self.http.service(self, "getUserWorkspaces", ()).await
    }
}
