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

// platform/packages/core/src/classes.ts

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type PersonUuid = Uuid;
pub type PersonId = String;
pub type WorkspaceDataId = String;
pub type WorkspaceUuid = Uuid;
pub type AccountUuid = Uuid;
pub type Ref = String;
pub type SocialIdId = String;

pub type Timestamp = chrono::DateTime<chrono::Utc>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
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

#[derive(Deserialize, Serialize, Debug, Clone)]
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
