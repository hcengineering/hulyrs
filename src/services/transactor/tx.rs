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

use crate::services::core::ser::Data;
use crate::services::types::{PersonId, Ref, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Obj {
    #[serde(rename = "_class")]
    pub class: Ref,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tx {
    #[serde(flatten)]
    pub doc: Doc,
    pub object_space: Ref,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxCUD {
    #[serde(flatten)]
    pub tx: Tx,
    pub object_id: Ref,
    pub object_class: Ref,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attached_to: Option<Ref>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attached_to_class: Option<Ref>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxCreateDoc<T> {
    #[serde(flatten)]
    pub txcud: TxCUD,
    pub attributes: Data<T>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxRemoveDoc {
    #[serde(flatten)]
    pub txcud: TxCUD,
}

pub type OperationDomain = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct TxDomainEvent<T: Serialize> {
    #[serde(flatten)]
    pub tx: Tx,

    pub domain: OperationDomain,
    pub event: T,
}
