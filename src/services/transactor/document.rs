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
pub struct CreateDocument<T: Serialize> {
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

    attributes: T,
}

impl<T: Serialize> Transaction for CreateDocument<T> {
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
struct RemoveDocument {
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

#[derive(Serialize, Deserialize, Debug, Default, Clone, Builder)]
#[serde(rename_all = "camelCase")]
pub struct FindOptions {
    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,

    // sort?: SortingQuery<T>
    // lookup?: Lookup<T>
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
}

pub trait DocumentClient {
    fn get_account(&self) -> impl Future<Output = Result<Account>>;

    fn find_all<Q: Serialize, T: DeserializeOwned>(
        &self,
        class: &str,
        query: Q,
        options: &FindOptions,
    ) -> impl Future<Output = Result<FindResult<T>>>;

    fn find_one<Q: Serialize, T: DeserializeOwned>(
        &self,
        class: &str,
        query: Q,
        options: &FindOptions,
    ) -> impl Future<Output = Result<Option<T>>>;
}

impl<B: Backend> DocumentClient for super::TransactorClient<B> {
    async fn get_account(&self) -> Result<Account> {
        self.get(Method::Account, []).await
    }

    async fn find_all<Q: Serialize, T: DeserializeOwned>(
        &self,
        class: &str,
        query: Q,
        options: &FindOptions,
    ) -> Result<FindResult<T>> {
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

        // TODO?
        /* api-client/src/rest.ts
        if (result.lookupMap !== undefined) {
            // We need to extract lookup map to document lookups
            for (const d of result) {
              if (d.$lookup !== undefined) {
                for (const [k, v] of Object.entries(d.$lookup)) {
                  if (!Array.isArray(v)) {
                    d.$lookup[k] = result.lookupMap[v as any]
                  } else {
                    d.$lookup[k] = v.map((it) => result.lookupMap?.[it])
                  }
                }
              }
            }
            delete result.lookupMap
          }*/

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

    async fn find_one<Q: Serialize, T: DeserializeOwned>(
        &self,
        class: &str,
        query: Q,
        options: &FindOptions,
    ) -> Result<Option<T>> {
        Ok(self
            .find_all(
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
