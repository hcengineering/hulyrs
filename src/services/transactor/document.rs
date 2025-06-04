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

use std::collections::HashMap;

use derive_builder::Builder;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{self as json, Value};

use crate::services::{
    Error, HttpClient, JsonClient, Result,
    types::{Account, PersonId, Ref, Timestamp},
};

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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<PersonId>,

    #[serde(with = "chrono::serde::ts_milliseconds_option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_on: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tx {
    #[serde(flatten)]
    pub parent: Doc,
    pub object_space: Ref,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxCUD {
    #[serde(flatten)]
    pub parent: Tx,
    pub object_id: Ref,
    pub object_class: Ref,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attached_to: Option<Ref>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attached_to_class: Option<Ref>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxCreateDoc<T> {
    #[serde(flatten)]
    pub parent: TxCUD,
    pub attributes: T,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TxRemoveDoc {
    #[serde(flatten)]
    pub parent: TxCUD,
}

#[derive(Default, Debug, derive_builder::Builder, Clone)]
pub struct CreateDocument<T: Serialize> {
    #[builder(setter(into))]
    object_id: Ref,

    #[builder(setter(into))]
    object_class: String,

    #[builder(setter(into, strip_option), default)]
    modified_on: Option<Timestamp>,

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
    fn transaction(self) -> impl Serialize {
        TxCreateDoc {
            parent: TxCUD {
                parent: Tx {
                    parent: Doc {
                        obj: Obj {
                            class: "core:class:TxCreateDoc".to_string(),
                        },

                        id: ksuid::Ksuid::generate().to_hex(),
                        modified_on: self.modified_on,
                        modified_by: self.modified_by,
                        created_on: self.created_on,
                        created_by: self.created_by,
                        space: "core:space:Tx".to_string(),
                    },
                    object_space: self.object_space,
                },
                object_id: self.object_id,
                object_class: self.object_class,
                attached_to: None,
                attached_to_class: None,
                collection: None,
            },

            attributes: self.attributes,
        }
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
    fn transaction(self) -> impl Serialize {
        TxRemoveDoc {
            parent: TxCUD {
                parent: Tx {
                    parent: Doc {
                        obj: Obj {
                            class: "core:class:TxRemoveDoc".to_string(),
                        },

                        id: ksuid::Ksuid::generate().to_hex(),
                        modified_on: self.modified_on,
                        modified_by: self.modified_by,
                        created_on: self.created_on,
                        created_by: self.created_by,
                        space: "core:space:Tx".to_string(),
                    },
                    object_space: self.object_space,
                },
                object_id: self.object_id,
                object_class: self.object_class,
                attached_to: None,
                attached_to_class: None,
                collection: None,
            },
        }
    }
}

pub trait Transaction {
    fn transaction(self) -> impl Serialize;
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindResult<T> {
    pub data_type: String,
    pub total: i64,
    pub value: Vec<T>,
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

    fn tx<R: DeserializeOwned, T>(&self, tx: T) -> impl Future<Output = Result<R>>
    where
        T: Transaction;
}

impl DocumentClient for super::TransactorClient {
    async fn get_account(&self) -> Result<Account> {
        let path = format!("/api/v1/account/{}", self.workspace);
        let url = self.base.join(&path)?;

        <HttpClient as JsonClient>::get(&self.http, self, url).await
    }

    async fn find_all<Q: Serialize, T: DeserializeOwned>(
        &self,
        class: &str,
        query: Q,
        options: &FindOptions,
    ) -> Result<FindResult<T>> {
        let path = format!("/api/v1/find-all/{}", self.workspace);
        let mut url = self.base.join(&path)?;

        let query = json::to_value(query)?;

        if !query.is_object() {
            return Err(Error::Other("QueryIsNotObject"));
        }

        let query = query.as_object().unwrap();

        url.query_pairs_mut()
            .append_pair("class", class)
            .append_pair("query", &json::to_string(&query)?)
            .append_pair("options", &json::to_string(&options)?);

        let mut result: FindResult<Value> =
            <HttpClient as JsonClient>::get(&self.http, self, url).await?;

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
            data_type: result.data_type,
            total: result.total,
            value: {
                let mut value = Vec::new();

                for v in result.value.into_iter() {
                    value.push(json::from_value(v)?);
                }

                value
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

    async fn tx<R: DeserializeOwned, T>(&self, tx: T) -> Result<R>
    where
        T: Transaction,
    {
        let path = format!("/api/v1/tx/{}", self.workspace);
        let url = self.base.join(&path)?;

        <HttpClient as JsonClient>::post(&self.http, self, url, &tx.transaction()).await
    }
}
