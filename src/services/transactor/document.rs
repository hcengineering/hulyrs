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
use chrono::Utc;
use derive_builder::Builder;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{self as json, Value};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::atomic::AtomicUsize;

use super::{
    Transaction,
    tx::{Doc, Obj},
};

use crate::services::core::classes::{Ref, Timestamp};
use crate::services::core::ser::Data;
use crate::services::core::tx::{Tx, TxCUD, TxCreateDoc, TxRemoveDoc};
use crate::services::core::{Account, FindResult, PersonId};
use crate::services::event::Class;
use crate::services::transactor::backend::Backend;
use crate::services::transactor::methods::Method;
use crate::{Error, Result};

static COUNT: AtomicUsize = AtomicUsize::new(0);
static RANDOM: LazyLock<String> = LazyLock::new(|| {
    format!(
        "{:6X}{:4X}",
        rand::random::<u32>().wrapping_mul(1 << 24),
        rand::random::<u32>().wrapping_mul(1 << 16)
    )
});

pub(crate) fn generate_object_id() -> Ref {
    let count = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut timestamp = Utc::now().timestamp() / 1000;
    if timestamp < 0 {
        timestamp = 0;
    }

    format!("{timestamp:8X}{}{count}", &*RANDOM)
}

#[derive(Default, Debug, derive_builder::Builder, Clone)]
pub struct CreateDocument<C: Serialize> {
    #[builder(setter(into), default = generate_object_id())]
    object_id: Ref,

    #[builder(setter(into))]
    object_class: String,

    #[builder(setter(into), default = Utc::now())]
    modified_on: Timestamp,

    #[builder(setter(into, strip_option), default)]
    modified_by: Option<PersonId>,

    #[builder(setter(into, strip_option), default)]
    created_on: Option<Timestamp>,

    #[builder(setter(into, strip_option), default)]
    created_by: Option<PersonId>,

    #[builder(setter(into))]
    object_space: String,

    attributes: C,
}

impl<C: Clone + Serialize> CreateDocument<C> {
    pub fn builder() -> CreateDocumentBuilder<C> {
        CreateDocumentBuilder::default()
    }
}

impl<C: Class + Serialize> Transaction for CreateDocument<C> {
    fn to_value(self) -> Result<Value> {
        let doc = TxCreateDoc {
            txcud: TxCUD {
                tx: Tx {
                    doc: Doc {
                        obj: Obj {
                            class: Ref::from(crate::services::core::class::TxCreateDoc),
                        },

                        id: generate_object_id(),
                        modified_on: Some(self.modified_on),
                        modified_by: self.modified_by,
                        created_on: self.created_on,
                        created_by: self.created_by,
                        space: Ref::from(crate::services::core::space::Tx),
                    },
                    object_space: self.object_space,
                },
                object_id: self.object_id,
                object_class: self.object_class,
                attached_to: None,
                attached_to_class: None,
                collection: None,
            },

            attributes: Data::new(self.attributes),
        };

        Ok(json::to_value(&doc)?)
    }
}

#[derive(Default, Debug, derive_builder::Builder, Clone, Serialize, Deserialize)]
pub struct RemoveDocument {
    #[builder(setter(into))]
    object_id: Ref,

    #[builder(setter(into))]
    object_class: String,

    #[builder(setter(into), default)]
    modified_on: Option<Timestamp>,

    #[builder(setter(into), default)]
    modified_by: Option<PersonId>,

    #[builder(setter(into), default)]
    created_on: Option<Timestamp>,

    #[builder(setter(into), default)]
    created_by: Option<PersonId>,

    #[builder(setter(into))]
    object_space: String,
}

impl RemoveDocument {
    pub fn builder() -> RemoveDocumentBuilder {
        RemoveDocumentBuilder::default()
    }
}

impl Transaction for RemoveDocument {
    fn to_value(self) -> Result<Value> {
        let doc = TxRemoveDoc {
            txcud: TxCUD {
                tx: Tx {
                    doc: Doc {
                        obj: Obj {
                            class: Ref::from(crate::services::core::class::TxRemoveDoc),
                        },

                        id: generate_object_id(),
                        modified_on: self.modified_on,
                        modified_by: self.modified_by,
                        created_on: self.created_on,
                        created_by: self.created_by,
                        space: Ref::from(crate::services::core::space::Tx),
                    },
                    object_space: self.object_space,
                },
                object_id: self.object_id,
                object_class: self.object_class,
                attached_to: None,
                attached_to_class: None,
                collection: None,
            },
        };

        Ok(json::to_value(&doc)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum LookupValue {
    Simple(String),
    Nested(String, Box<Lookup>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ReverseLookupValue {
    Simple(String),
    WithAttribute(String, String),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Lookup {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub reverse_lookups: Option<HashMap<String, ReverseLookupValue>>,

    #[serde(flatten)]
    pub forward_lookups: HashMap<String, LookupValue>,
}

#[macro_export]
macro_rules! lookup_reverse {
    (@build $map:expr,) => {};

    (@build $map:expr, $key:ident: $value:expr, $($rest:tt)*) => {
        $map.insert(
            stringify!($key).to_string(),
            $crate::services::transactor::document::ReverseLookupValue::Simple($value.into()),
        );
        $crate::lookup_reverse!(@build $map, $($rest)*);
    };

    (@build $map:expr, $key:ident: [$class:expr, $attr:expr], $($rest:tt)*) => {
        $map.insert(
            stringify!($key).to_string(),
            $crate::services::transactor::document::ReverseLookupValue::WithAttribute($class.into(), $attr.into()),
        );
        $crate::lookup_reverse!(@build $map, $($rest)*);
    };
}

#[macro_export]
macro_rules! lookup {
    // Base case
    (@build $map:expr, $(,)*) => {};

    (@build $lookup:expr, _id: { $($reverse_tts:tt)* }, $($rest:tt)*) => {
        {
            let mut reverse_map = ::std::collections::HashMap::new();
            $crate::lookup_reverse!(@build &mut reverse_map, $($reverse_tts)*,);
            $lookup.reverse_lookups = Some(reverse_map);
        }
        $crate::lookup!(@build $lookup, $($rest)*);
    };

    (@build $lookup:expr, $key:ident: $value:expr, $($rest:tt)*) => {
        $lookup.forward_lookups.insert(
            stringify!($key).to_string(),
            $crate::services::transactor::document::LookupValue::Simple($value.into()),
        );
        $crate::lookup!(@build $lookup, $($rest)*);
    };

    (@build $lookup:expr, $key:ident: [$value:expr, { $($nested_tts:tt)* }], $($rest:tt)*) => {
        $lookup.forward_lookups.insert(
            stringify!($key).to_string(),
            $crate::services::transactor::document::LookupValue::Nested(
                $value.into(),
                Box::new($crate::lookup!{ $($nested_tts)* }),
            ),
        );
        $crate::lookup!(@build $lookup, $($rest)*);
    };

    ( $($tts:tt)* ) => {
        {
            let mut lookup = $crate::services::transactor::document::Lookup::default();
            $crate::lookup!(@build &mut lookup, $($tts)*,);
            lookup
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Builder)]
#[builder(build_fn(private, name = "fallible_build"))]
#[serde(rename_all = "camelCase")]
pub struct FindOptions {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,

    // sort?: SortingQuery<T>
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lookup: Option<Lookup>,
    // projection?: Projection<T>
    // associations?: AssociationQuery[]
    #[builder(setter(custom), default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    projection: HashMap<String, u16>,

    #[builder(default)]
    total: bool,

    #[builder(default)]
    show_archived: bool,
}

impl FindOptions {
    pub fn builder() -> FindOptionsBuilder {
        FindOptionsBuilder::default()
    }
}

impl FindOptionsBuilder {
    pub fn project(&mut self, field: &str) -> &mut Self {
        if self.projection.is_none() {
            self.projection = Some(HashMap::new());
        }

        self.projection
            .as_mut()
            .unwrap()
            .insert(field.to_owned(), 1);

        self
    }

    pub fn build(&mut self) -> FindOptions {
        self.fallible_build()
            .expect("All required fields set at initialization")
    }
}

pub trait DocumentClient {
    fn get_account(&self) -> impl Future<Output = Result<Account>>;

    fn find_all<Q: Serialize, C: DeserializeOwned>(
        &self,
        class: &str,
        query: Q,
        options: &FindOptions,
    ) -> impl Future<Output = Result<FindResult<C>>>;

    fn find_one<Q: Serialize, C: DeserializeOwned>(
        &self,
        class: &str,
        query: Q,
        options: &FindOptions,
    ) -> impl Future<Output = Result<Option<C>>>;
}

impl<B: Backend> DocumentClient for super::TransactorClient<B> {
    async fn get_account(&self) -> Result<Account> {
        self.get(Method::Account, []).await
    }

    async fn find_all<Q: Serialize, C: DeserializeOwned>(
        &self,
        class: &str,
        query: Q,
        options: &FindOptions,
    ) -> Result<FindResult<C>> {
        let query = json::to_value(query)?;

        if !query.is_object() {
            return Err(Error::Other("QueryIsNotObject"));
        }

        let query = query.as_object().unwrap();

        let mut result: FindResult<Value> = self
            .get(
                Method::FindAll,
                [
                    (String::from("class"), class.into()),
                    (String::from("query"), json::to_value(query)?),
                    (String::from("options"), json::to_value(options)?),
                ],
            )
            .await?;

        // as in api-client/src/rest.ts
        if let Some(lookup_map) = &result.lookup_map {
            for entry in result.value.iter_mut() {
                let Some(obj_lookup) = entry.get_mut("$lookup").and_then(Value::as_object_mut)
                else {
                    continue;
                };

                for value in obj_lookup.values_mut() {
                    if let Some(array) = value.as_array_mut() {
                        for item in array {
                            if let Some(lookup_key) = item.as_str() {
                                *item = lookup_map.get(lookup_key).cloned().unwrap_or(Value::Null)
                            }
                        }
                    } else if let Some(lookup_key) = value.as_str() {
                        *value = lookup_map.get(lookup_key).cloned().unwrap_or(Value::Null)
                    }
                }
            }
        }

        // as in api-client/src/rest.ts
        for entry in result.value.iter_mut() {
            let object = entry.as_object_mut().unwrap();
            if !object.contains_key("_class") {
                object.insert("_class".into(), Value::String(class.into()));
            }

            for (k, v) in query.iter() {
                if !object.contains_key(k) && (v.is_string() || v.is_boolean() || v.is_number()) {
                    object.insert(k.to_owned(), v.clone());
                }
            }
        }

        let result = FindResult {
            total: result.total,
            value: {
                let mut value = Vec::new();

                for v in result.value.into_iter() {
                    value.push(json::from_value(v)?);
                }

                value
            },
            lookup_map: match result.lookup_map {
                Some(lookup_map) => {
                    let new_map = lookup_map
                        .into_iter()
                        .map(|(k, v)| match json::from_value(v) {
                            Ok(val) => Ok((k, val)),
                            Err(e) => Err(e.into()),
                        })
                        .collect::<Result<_>>()?;

                    Some(new_map)
                }
                None => None,
            },
        };

        Ok(result)
    }

    async fn find_one<Q: Serialize, C: DeserializeOwned>(
        &self,
        class: &str,
        query: Q,
        options: &FindOptions,
    ) -> Result<Option<C>> {
        Ok(self
            .find_all::<_, C>(
                class,
                query,
                &FindOptions {
                    limit: Some(1),
                    ..options.clone()
                },
            )
            .await?
            .value
            .into_iter()
            .next())
    }
}
