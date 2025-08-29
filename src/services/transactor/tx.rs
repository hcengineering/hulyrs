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

use crate::services::core::PersonId;
use crate::services::core::classes::{Ref, Timestamp};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Obj {
    #[serde(rename = "_class")]
    pub class: Ref,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Doc {
    #[serde(flatten)]
    pub obj: Obj,

    #[serde(rename = "_id")]
    pub id: Ref,
    pub space: Ref,

    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_on: Option<Timestamp>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_by: Option<PersonId>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub created_by: Option<PersonId>,

    #[serde(with = "chrono::serde::ts_milliseconds_option", default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_on: Option<Timestamp>,
}
