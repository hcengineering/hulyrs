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

use crate::Result;
use crate::services::ForceScheme;
use crate::services::core::WorkspaceUuid;
use crate::services::core::classes::OperationDomain;
use crate::services::core::storage::DomainResult;
use crate::services::event::{Class, DocT};
use crate::services::transactor::backend::Backend;
use crate::services::transactor::backend::http::{HttpBackend, HttpClient};
use crate::services::transactor::backend::ws::{WsBackend, WsBackendOpts};
use crate::services::transactor::document::{FindOptions, RemoveDocument};
use crate::services::transactor::methods::Method;
use crate::services::transactor::subscription::LiveQueryEvent;
use futures::Stream;
use secrecy::{ExposeSecret, SecretString};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use subscription::SubscribedQuery;
use url::Url;

pub mod backend;
pub mod comm;
pub mod document;
pub mod methods;
pub mod person;
pub mod subscription;
pub mod tx;

pub trait Transaction {
    fn to_value(self) -> crate::Result<Value>;
}

pub trait TransactionValue {
    fn matches(&self, class: Option<&str>, domain: Option<&str>) -> bool;
}
impl TransactionValue for Value {
    fn matches(&self, class: Option<&str>, domain: Option<&str>) -> bool {
        let class = class.is_none() || self["_class"].as_str() == class;
        let domain = domain.is_none() || self["domain"].as_str() == domain;

        class && domain
    }
}

#[derive(Clone)]
pub struct TransactorClient<B> {
    backend: B,
}

impl<B: Backend> PartialEq for TransactorClient<B> {
    fn eq(&self, other: &Self) -> bool {
        self.backend.workspace() == other.backend.workspace()
            && self.backend.provide_token() == other.backend.provide_token()
            && self.base() == other.base()
    }
}

impl<B: Backend> super::TokenProvider for &TransactorClient<B> {
    fn provide_token(&self) -> Option<&str> {
        self.backend.provide_token()
    }
}

impl<B: Backend> TransactorClient<B> {
    pub fn base(&self) -> &Url {
        self.backend.base()
    }

    pub fn workspace(&self) -> WorkspaceUuid {
        self.backend.workspace()
    }

    pub async fn get<T: DeserializeOwned + Send>(
        &self,
        method: Method,
        params: impl IntoIterator<Item = (String, Value)> + Send,
    ) -> Result<T> {
        self.backend.get(method, params).await
    }

    pub async fn post<T: DeserializeOwned + Send, Q: Serialize>(
        &self,
        method: Method,
        body: &Q,
    ) -> Result<T> {
        self.backend.post(method, body).await
    }

    pub async fn domain_request<T: DeserializeOwned + Send, Q: Serialize>(
        &self,
        domain: OperationDomain,
        operation: &str,
        params: &Q,
    ) -> Result<DomainResult<T>> {
        self.backend.domain_request(domain, operation, params).await
    }

    pub async fn tx_raw<T: Serialize, R: DeserializeOwned + Send>(&self, tx: T) -> Result<R> {
        self.backend.tx_raw(tx).await
    }

    pub async fn tx<T: Transaction, R: DeserializeOwned + Send>(&self, tx: T) -> Result<R> {
        self.backend.tx(tx).await
    }

    pub(in crate::services::transactor) fn backend(&self) -> &B {
        &self.backend
    }

    pub async fn remove<T: DocT + Clone>(&self, doc: &T) -> Result<()> {
        let tx = RemoveDocument::builder()
            .object_class(&doc.doc().obj.class)
            .object_id(doc.id())
            .object_space(&doc.doc().space)
            .build()
            .expect("fields filled");

        self.backend.tx(tx).await
    }
}

impl TransactorClient<HttpBackend> {
    pub fn new(
        http: HttpClient,
        base: Url,
        workspace: WorkspaceUuid,
        token: impl Into<SecretString>,
    ) -> Result<Self> {
        let base = base.force_http_scheme();
        Ok(Self {
            backend: HttpBackend::new(http, base, workspace, token),
        })
    }
}

impl TransactorClient<WsBackend> {
    pub async fn new_ws(
        base: Url,
        workspace: WorkspaceUuid,
        token: impl Into<SecretString>,
        opts: WsBackendOpts,
    ) -> Result<Self> {
        let base = base.force_ws_scheme();
        let token = token.into();
        let backend = WsBackend::connect(base, workspace, token.expose_secret(), opts).await?;

        Ok(Self { backend })
    }

    pub async fn subscribe<T: crate::services::event::Event + DeserializeOwned>(
        &self,
    ) -> SubscribedQuery<T> {
        SubscribedQuery::new(self.clone())
    }

    /// Fetches all documents of the specified [`Class`], and subscribes to future events
    pub fn live_query<C: Class + DeserializeOwned + Send + Unpin + 'static, Q: Serialize + Send>(
        &self,
        query: Q,
        options: FindOptions,
    ) -> impl Stream<Item = Result<LiveQueryEvent<C>>> + Send + use<C, Q> {
        subscription::live_query(self.clone(), query, options)
    }
}

#[cfg(feature = "kafka")]
pub mod kafka {
    use super::*;
    use crate::{Config, Error, services::core::WorkspaceUuid};
    use rdkafka::{
        ClientConfig, Message,
        consumer::{ConsumerContext, StreamConsumer},
        message::{Header, Headers, OwnedHeaders},
        producer::FutureProducer,
    };
    use serde_json::{self as json, Value};
    use std::time::Duration;
    use tracing::{debug, warn};
    use uuid::Uuid;

    pub struct KafkaProducer {
        producer: FutureProducer,
        topic: String,
    }

    impl KafkaProducer {
        pub fn new(config: &Config, topic: &str) -> Result<Self> {
            let producer = ClientConfig::new()
                .set(
                    "bootstrap.servers",
                    config.kafka_bootstrap_servers.join(","),
                )
                .set("message.timeout.ms", "5000")
                .create()?;

            Ok(Self {
                producer,
                topic: topic.to_owned(),
            })
        }

        pub async fn tx_raw<T: Serialize>(
            &self,
            workspace: WorkspaceUuid,
            transaction: T,
            partition_key: Option<&str>,
        ) -> Result<()> {
            let payload = json::to_vec(&transaction)?;

            let headers = OwnedHeaders::new()
                .insert(Header {
                    key: "WorkspaceUuid",
                    value: Some(&workspace.to_string()),
                })
                .insert(Header {
                    key: "workspace",
                    value: Some(&workspace.to_string()),
                })
                .insert(Header {
                    key: "Mode",
                    value: Some("transaction"),
                });

            let mut message = rdkafka::producer::FutureRecord::to(&self.topic)
                .payload(&payload)
                .headers(headers);

            if let Some(partition_key) = partition_key {
                message = message.key(partition_key)
            }

            self.producer
                .send(message, Duration::from_secs(10))
                .await
                .map_err(|e| e.0)?;

            Ok(())
        }

        pub async fn tx<T: Transaction>(
            &self,
            workspace: WorkspaceUuid,
            tx: T,
            partition_key: Option<&str>,
        ) -> Result<()> {
            self.tx_raw(workspace, tx.to_value()?, partition_key).await
        }
    }

    pub trait TransactionsConsumer {
        fn tx_recv(&self) -> impl Future<Output = (WorkspaceUuid, Value)>;
    }

    impl<C: ConsumerContext + 'static, R> TransactionsConsumer for StreamConsumer<C, R> {
        async fn tx_recv(&self) -> (WorkspaceUuid, Value) {
            fn inner(message: Option<impl Message>) -> Result<(WorkspaceUuid, Value)> {
                let message = message.ok_or_else(|| Error::Other("KafkaError"))?;

                let payload = message
                    .payload()
                    .ok_or_else(|| Error::Other("MissingPayload"))?;
                let payload = json::from_slice::<Value>(payload)?;

                let workspace_id = message
                    .headers()
                    .and_then(|headers| {
                        headers
                            .iter()
                            .find(|h| h.key == "workspace" || h.key == "WorkspaceUuid")
                    })
                    .and_then(|header| header.value)
                    .map(String::from_utf8_lossy);

                debug!(?workspace_id, "workspace id");

                let workspace_id = workspace_id
                    .map(|s| Uuid::from_slice(s.as_bytes()))
                    .ok_or_else(|| Error::Other("NoWorkspaceId"))?
                    .map_err(|_| Error::Other("InvalidWorkspaceId"))?;

                Ok((workspace_id, payload))
            }

            loop {
                let message = self.recv().await.ok();

                println!("{message:?}");

                match inner(message) {
                    Ok(transaction) => break transaction,
                    Err(error) => {
                        warn!(%error, "transaction error");
                    }
                }
            }
        }
    }

    pub fn parse_message(message: &impl Message) -> Result<(WorkspaceUuid, Value)> {
        let payload = message
            .payload()
            .ok_or_else(|| Error::Other("MissingPayload"))?;
        let payload = json::from_slice::<Value>(payload)?;

        let workspace_id = message
            .headers()
            .and_then(|headers| headers.iter().find(|h| h.key == "WorkspaceUuid"))
            .and_then(|header| header.value)
            .or(message.key())
            .map(String::from_utf8_lossy);

        let workspace_id = workspace_id
            .map(|s| Uuid::parse_str(&s))
            .ok_or_else(|| Error::Other("NoWorkspaceId"))?
            .map_err(|_| Error::Other("InvalidWorkspaceId"))?;

        Ok((workspace_id, payload))
    }
}
