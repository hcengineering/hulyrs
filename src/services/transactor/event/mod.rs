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

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json as json;

use crate::{
    Result,
    services::{HttpClient, JsonClient},
};

mod message;
pub use message::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum MessageRequestType {
    // Message
    CreateMessage,
    RemoveMessages,

    CreatePatch,

    CreateReaction,
    RemoveReaction,

    CreateFile,
    RemoveFile,

    CreateThread,
    UpdateThread,

    CreateMessagesGroup,
    RemoveMessagesGroup,

    // Label
    CreateLabel,
    RemoveLabel,

    // Notification
    AddCollaborators,
    RemoveCollaborators,

    CreateNotification,
    RemoveNotifications,

    CreateNotificationContext,
    RemoveNotificationContext,
    UpdateNotificationContext,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Envelope<T: serde::Serialize> {
    r#type: MessageRequestType,

    #[serde(flatten)]
    request: T,
}

impl<T: serde::Serialize> Envelope<T> {
    pub fn new(r#type: MessageRequestType, body: T) -> Self {
        Self {
            r#type,
            request: body,
        }
    }
}

pub trait EventClient {
    fn request_raw<T: Serialize, R: DeserializeOwned>(
        &self,
        envelope: &Envelope<T>,
    ) -> impl Future<Output = Result<R>>;

    fn request_for_result<T: Serialize, R: DeserializeOwned>(
        &self,
        r#type: MessageRequestType,
        request: T,
    ) -> impl Future<Output = Result<R>> {
        async { Ok(self.request_raw(&Envelope::new(r#type, request)).await?) }
    }

    fn request<T: Serialize>(
        &self,
        r#type: MessageRequestType,
        request: T,
    ) -> impl Future<Output = Result<()>> {
        async {
            self.request_raw::<T, json::Value>(&Envelope::new(r#type, request))
                .await
                .map(|_| ())
        }
    }
}

impl EventClient for super::TransactorClient {
    async fn request_raw<T: Serialize, R: DeserializeOwned>(
        &self,
        envelope: &Envelope<T>,
    ) -> Result<R> {
        let path = format!("/api/v1/event/{}", self.workspace);
        let url = self.base.join(&path)?;

        Ok(<HttpClient as JsonClient>::post(&self.http, self, url, envelope).await?)
    }
}

#[cfg(feature = "kafka")]
pub mod kafka {
    use super::*;
    use crate::{Config, services::types::WorkspaceUuid};
    use rdkafka::{
        ClientConfig,
        message::{Header, OwnedHeaders},
        producer::FutureProducer,
    };
    use serde_json as json;
    use std::time::Duration;

    pub struct KafkaEventPublisher {
        producer: FutureProducer,
        topic: String,
    }

    impl KafkaEventPublisher {
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

        pub async fn request<T: Serialize + PartitionKeyProvider>(
            &self,
            workspace: WorkspaceUuid,
            r#type: MessageRequestType,
            event: T,
        ) -> Result<()> {
            let envelope = Envelope::new(r#type, event);
            let payload = json::to_vec(&envelope)?;

            let message = rdkafka::producer::FutureRecord::to(&self.topic)
                .payload(&payload)
                .headers(OwnedHeaders::new().insert(Header {
                    key: "WorkspaceUuid",
                    value: Some(&workspace.to_string()),
                }))
                .key(envelope.request.partition_key());

            self.producer
                .send(message, Duration::from_secs(10))
                .await
                .map_err(|e| e.0)?;

            Ok(())
        }
    }
}
