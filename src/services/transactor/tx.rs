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
use crate::services::core::{PersonId, Ref, Timestamp};
use crate::services::event::{Class, Event};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::fmt::Debug;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Obj {
    #[serde(rename = "_class")]
    pub class: Ref,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tx {
    #[serde(flatten)]
    pub doc: Doc,
    pub object_space: Ref,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

impl<T> Class for TxCreateDoc<T> {
    const CLASS: &'static str = crate::services::core::class::TxCreateDoc;
}

impl<T: Class> Event for TxCreateDoc<T> {
    fn matches(value: &Value) -> bool {
        if value.get("_class").and_then(|v| v.as_str()) != Some(Self::CLASS) {
            return false;
        }
        value.get("objectClass").and_then(|v| v.as_str()) == Some(T::CLASS)
    }
}

impl<'de, T> Deserialize<'de> for TxCreateDoc<T>
where
    T: Serialize + DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        let txcud = serde_json::from_value(value.clone()).map_err(serde::de::Error::custom)?;

        let attributes = serde_json::from_value(value).map_err(serde::de::Error::custom)?;

        Ok(TxCreateDoc { txcud, attributes })
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxRemoveDoc {
    #[serde(flatten)]
    pub txcud: TxCUD,
}

impl Class for TxRemoveDoc {
    const CLASS: &'static str = crate::services::core::class::TxRemoveDoc;
}

impl Event for TxRemoveDoc {}

pub type OperationDomain = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct TxDomainEvent<T> {
    #[serde(flatten)]
    pub tx: Tx,

    pub domain: OperationDomain,
    pub event: T,
}

impl<T> Class for TxDomainEvent<T> {
    const CLASS: &'static str = crate::services::core::class::TxDomainEvent;
}

impl<T: Class> Event for TxDomainEvent<T> {
    fn matches(value: &Value) -> bool {
        if value.get("_class").and_then(|v| v.as_str()) != Some(Self::CLASS) {
            return false;
        }
        value.get("objectClass").and_then(|v| v.as_str()) == Some(T::CLASS)
    }
}
