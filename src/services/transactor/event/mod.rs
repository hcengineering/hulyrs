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

use std::time::Duration;

use rdkafka::{
    ClientConfig,
    message::{Header, OwnedHeaders},
    producer::FutureProducer,
};
use serde::{Deserialize, Serialize};
use serde_json as json;

use crate::{
    CONFIG,
    services::{HttpClient, JsonClient, Result, types::WorkspaceUuid},
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

pub trait EventClient<T: Serialize> {
    fn request_raw(&self, envelope: &Envelope<T>) -> impl Future<Output = Result<()>>;

    fn request(&self, r#type: MessageRequestType, request: T) -> impl Future<Output = Result<()>> {
        async { Ok(self.request_raw(&Envelope::new(r#type, request)).await?) }
    }
}

impl<T: Serialize> EventClient<T> for super::TransactorClient {
    async fn request_raw(&self, envelope: &Envelope<T>) -> Result<()> {
        let path = format!("/api/v1/event/{}", self.workspace);
        let url = self.base.join(&path)?;

        let _: serde_json::Value =
            <HttpClient as JsonClient>::post(&self.http, self, url, envelope).await?;

        Ok(())
    }
}

pub struct KafkaEventPublisher {
    producer: FutureProducer,
    topic: String,
}

impl KafkaEventPublisher {
    pub fn new(topic: &str) -> Result<Self> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", CONFIG.kafka_bootstrap_servers())
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
