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

pub mod classes;
pub(crate) mod ser;
pub mod storage;
pub mod tx;

use crate::services::transactor::tx::Doc;
use classes::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::event::{Class, HasId};

pub type PersonUuid = Uuid;
pub type PersonId = String;
pub type WorkspaceDataId = String;
pub type WorkspaceUuid = Uuid;
pub type AccountUuid = Uuid;
pub type SocialIdId = String;

#[allow(non_upper_case_globals)]
pub mod space {
    pub const Workspace: &str = "core.space.Workspace";
    pub const Space: &str = "core.space.Space";
    pub const Tx: &str = "core:space:Tx";
}

#[allow(non_upper_case_globals)]
pub mod class {
    pub const TxCreateDoc: &str = "core:class:TxCreateDoc";
    pub const TxUpdateDoc: &str = "core:class:TxUpdateDoc";
    pub const TxRemoveDoc: &str = "core:class:TxRemoveDoc";
    pub const TxDomainEvent: &str = "core:class:TxDomainEvent";
    pub const TxWorkspaceEvent: &str = "core:class:TxWorkspaceEvent";
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Space {
    #[serde(flatten)]
    pub doc: Doc,
    pub name: String,
    pub private: bool,
    pub members: Vec<AccountUuid>,
    pub archived: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owners: Option<Vec<AccountUuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_join: Option<bool>,
}

impl Class for Space {
    const CLASS: &'static str = "core:class:Space";
}

impl HasId for Space {
    fn id(&self) -> &str {
        &self.doc.id
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BasePerson {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person_uuid: Option<PersonUuid>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SocialIdType {
    Email,
    GitHub,
    Google,
    Phone,
    OIDC,
    Huly,
    Telegram,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SocialId {
    #[serde(rename = "_id")]
    pub id: SocialIdId,

    pub r#type: SocialIdType,
    pub value: String,
    pub key: String,
    pub display_value: Option<String>,

    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub created_on: Option<Timestamp>,

    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    pub verified_on: Option<Timestamp>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum AccountRole {
    DocGuest,
    GUEST,
    USER,
    MAINTAINER,
    OWNER,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub uuid: AccountUuid,
    pub role: AccountRole,
    pub primary_social_id: PersonId,
    pub social_ids: Vec<PersonId>,
    pub full_social_ids: Vec<SocialId>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindResult<T> {
    pub total: i64,
    pub value: Vec<T>,
    pub lookup_map: Option<HashMap<String, T>>,
}
