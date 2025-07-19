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

use secrecy::{ExposeSecret, SecretString};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use tracing::*;
use url::Url;

pub mod comm;
pub mod document;
pub mod person;
pub mod tx;

use crate::{
    Error, Result,
    services::{ForceHttpScheme, HttpClient, JsonClient, types::WorkspaceUuid},
};

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
pub struct TransactorClient {
    pub workspace: WorkspaceUuid,
    pub base: Url,
    token: SecretString,
    http: HttpClient,
}

impl PartialEq for TransactorClient {
    fn eq(&self, other: &Self) -> bool {
        self.workspace == other.workspace
            && self.token.expose_secret() == other.token.expose_secret()
            && self.base == other.base
    }
}

impl super::TokenProvider for &TransactorClient {
    fn provide_token(&self) -> Option<&str> {
        Some(self.token.expose_secret())
    }
}

impl TransactorClient {
    pub fn new(
        http: HttpClient,
        base: Url,
        workspace: WorkspaceUuid,
        token: impl Into<SecretString>,
    ) -> Result<Self> {
        let base = base.force_http_scheme();
        Ok(Self {
            workspace,
            http,
            base,
            token: token.into(),
        })
    }

    pub async fn tx_raw<T: Serialize, R: DeserializeOwned>(&self, tx: T) -> Result<R> {
        let path = format!("/api/v1/tx/{}", self.workspace);
        let url = self.base.join(&path)?;

        <HttpClient as JsonClient>::post(&self.http, self, url, &tx).await
    }

    pub async fn tx<T: Transaction, R: DeserializeOwned>(&self, tx: T) -> Result<R> {
        self.tx_raw(tx.to_value()?).await
    }
}

#[cfg(feature = "kafka")]
pub mod kafka {
    use super::*;
    use crate::{Config, services::types::WorkspaceUuid};
    use rdkafka::{
        ClientConfig, Message,
        consumer::{ConsumerContext, StreamConsumer},
        message::{Header, Headers, OwnedHeaders},
        producer::FutureProducer,
    };
    use serde_json::{self as json, Value};
    use std::time::Duration;
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
