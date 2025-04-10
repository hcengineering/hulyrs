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

use serde::{Deserialize, Serialize};

use crate::services::{HttpClient, JsonClient, Result};

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
struct Envelope<T: serde::Serialize> {
    r#type: MessageRequestType,

    #[serde(flatten)]
    body: T,
}

impl<T: serde::Serialize> Envelope<T> {
    pub fn new(r#type: MessageRequestType, body: T) -> Self {
        Self { r#type, body }
    }
}

pub trait EventClient {
    fn request<T: Serialize>(
        &self,
        r#type: MessageRequestType,
        event: T,
    ) -> impl Future<Output = Result<()>>;
}

impl EventClient for super::TransactorClient {
    async fn request<T: Serialize>(&self, r#type: MessageRequestType, event: T) -> Result<()> {
        let path = format!("/api/v1/event/{}", self.workspace);
        let url = self.base.join(&path)?;

        let _: serde_json::Value =
            <HttpClient as JsonClient>::post(&self.http, self, url, &Envelope::new(r#type, event))
                .await?;

        Ok(())
    }
}
