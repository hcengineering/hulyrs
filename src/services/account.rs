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
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;
use uuid::Uuid;

use crate::config::Config;
use crate::services::core::classes::Timestamp;
use crate::services::core::{AccountUuid, PersonId, PersonUuid, WorkspaceDataId, WorkspaceUuid};
use crate::{
    Error, Result,
    services::{
        ServiceClient,
        core::{SocialId, SocialIdType},
        jwt::Claims,
    },
};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfo {
    pub account: PersonUuid,
    pub name: Option<String>,
    pub social_id: Option<PersonId>,
    pub token: Option<String>,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationSecret {
    pub social_id: PersonId,
    pub kind: String,
    pub workspace_uuid: Option<WorkspaceUuid>,
    pub key: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationSecretKey {
    pub social_id: PersonId,
    pub kind: String,
    pub workspace_uuid: Option<WorkspaceUuid>,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceKind {
    #[default]
    Internal,
    External,
    ByRegion,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Location {
    KV,
    WEUR,
    EEUR,
    WNAM,
    ENAM,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfo {
    pub uuid: WorkspaceUuid,
    pub data_id: Option<WorkspaceDataId>,
    pub name: String,
    pub url: String,
    pub region: Option<String>,
    pub branding: Option<String>,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub created_on: Option<Timestamp>,
    pub created_by: Option<PersonUuid>,
    pub billing_account: Option<PersonUuid>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceMode {
    ManualCreation,
    PendingCreation,
    Creating,
    Upgrading,
    PendingDeletion,
    Deleting,
    Active,
    Deleted,
    ArchingPendingBackup,
    ArchivingBackup,
    ArchivingPendingClean,
    ArchivingClean,
    Archived,
    MigrationPendingBackup,
    MigrationBackup,
    MigrationPendingClean,
    MigrationClean,
    PendingRestore,
    Restoring,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupStatus {
    #[serde(deserialize_with = "crate::optional_rounded_float", default)]
    pub data_size: Option<u32>,

    #[serde(deserialize_with = "crate::optional_rounded_float", default)]
    pub blobs_size: Option<u32>,

    #[serde(deserialize_with = "crate::optional_rounded_float", default)]
    pub backup_size: Option<u32>,

    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub last_backup: Timestamp,

    pub backups: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInfoWithStatus {
    #[serde(flatten)]
    pub workspace: WorkspaceInfo,
    pub status: WorkspaceStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceVersion {
    pub version_major: i32,
    pub version_minor: i32,
    pub version_patch: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatus {
    #[serde(flatten)]
    pub version: WorkspaceVersion,
    pub mode: Option<WorkspaceMode>,

    pub processing_progress: Option<u32>,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub last_processing_time: Option<Timestamp>,
    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub last_visit: Option<Timestamp>,

    #[serde(default)]
    pub is_disabled: Option<bool>,

    pub processing_attempts: Option<u32>,
    pub processing_message: Option<String>,
    pub backup_info: Option<BackupStatus>,
    pub target_region: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegionInfo {
    pub region: String,
    pub name: String,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceParams {
    pub workspace_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignUpParams {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub locale: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListAccountsParams {
    pub search: Option<String>,
    pub skip: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Clone)]
pub struct AccountClient {
    pub account: Option<AccountUuid>,
    token: Option<SecretString>,
    base: Url,
    http: HttpClient,
}

impl PartialEq for AccountClient {
    fn eq(&self, other: &Self) -> bool {
        self.account == other.account
            && self.token.as_ref().map(SecretString::expose_secret)
                == other.token.as_ref().map(SecretString::expose_secret)
            && self.base == other.base
    }
}

impl super::TokenProvider for &AccountClient {
    fn provide_token(&self) -> Option<&str> {
        self.token.as_ref().map(SecretString::expose_secret)
    }
}

impl super::BasePathProvider for &AccountClient {
    fn provide_base_path(&self) -> &Url {
        &self.base
    }
}

impl AccountClient {
    pub fn new(
        config: &Config,
        http: HttpClient,
        account: AccountUuid,
        token: impl Into<SecretString>,
    ) -> Result<Self> {
        let base = config.account_service.clone();
        Ok(Self {
            http,
            base: base.ok_or(Error::Other("NoAccountService"))?,
            account: Some(account),
            token: Some(token.into()),
        })
    }

    pub fn without_user(config: &Config, http: HttpClient) -> Result<Self> {
        let base = config.account_service.clone();
        Ok(Self {
            http,
            base: base.ok_or(Error::Other("NoAccountService"))?,
            account: None,
            token: None,
        })
    }

    #[deprecated(note = "use ServiceFactory")]
    pub fn assume_claims(&self, claims: &Claims, secret: &SecretString) -> Result<Self> {
        let account = Some(claims.account);
        let base = self.base.clone();
        let http = self.http.clone();
        let token = Some(claims.encode(secret)?);

        Ok(Self {
            http,
            base,
            account,
            token,
        })
    }

    #[deprecated(note = "use ServiceFactory")]
    pub fn assume_token(&self, token: impl AsRef<str>) -> Self {
        let account = self.account;
        let base = self.base.clone();
        let http = self.http.clone();

        Self {
            http,
            base,
            account,
            token: Some(token.as_ref().into()),
        }
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

    pub async fn get_region_info(&self) -> Result<Vec<RegionInfo>> {
        self.http.service(self, "getRegionInfo", json!({})).await
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

    pub async fn create_workspace(
        &self,
        params: &CreateWorkspaceParams,
    ) -> Result<WorkspaceLoginInfo> {
        self.http.service(self, "createWorkspace", params).await
    }

    pub async fn sign_up(&self, params: &SignUpParams) -> Result<LoginInfo> {
        self.http.service(self, "signUp", params).await
    }

    pub async fn login(&self, params: &LoginParams) -> Result<LoginInfo> {
        self.http.service(self, "login", params).await
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

    pub async fn get_user_workspaces(&self) -> Result<Vec<WorkspaceInfoWithStatus>> {
        self.http.service(self, "getUserWorkspaces", ()).await
    }

    pub async fn get_account_info(&self, account_uuid: &AccountUuid) -> Result<AccountInfo> {
        let params = json!({"accountId": account_uuid});
        self.http.service(self, "getAccountInfo", params).await
    }

    pub async fn list_accounts(&self, params: &ListAccountsParams) -> Result<Vec<AccountInfo>> {
        self.http.service(self, "listAccounts", params).await
    }

    pub async fn add_integration_secret(&self, secret: &IntegrationSecret) -> Result<()> {
        self.http
            .service(self, "addIntegrationSecret", secret)
            .await
    }

    pub async fn get_integration_secret(
        &self,
        params: &IntegrationSecretKey,
    ) -> Result<Option<IntegrationSecret>> {
        self.http
            .service(self, "getIntegrationSecret", params)
            .await
    }

    pub async fn update_integration_secret(&self, params: &IntegrationSecret) -> Result<()> {
        self.http
            .service(self, "updateIntegrationSecret", params)
            .await
    }

    pub async fn delete_integration_secret(&self, params: &IntegrationSecretKey) -> Result<()> {
        self.http
            .service(self, "deleteIntegrationSecret", params)
            .await
    }
}
