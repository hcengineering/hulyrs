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
use serde_json::{self as json, Value};

use crate::{
    Result,
    services::{HttpClient, JsonClient},
};

mod message;
use super::tx::{Doc, Obj, Tx, TxDomainEvent};
use crate::services::core::Ref;
pub use message::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum MessageRequestType {
    // Message
    CreateMessage,
    UpdatePatch,
    RemovePatch,
    ReactionPatch,
    BlobPatch,
    LinkPreviewPatch,
    ThreadPatch,

    // Label
    CreateLabel,
    RemoveLabel,

    // Notification
    AddCollaborators,
    RemoveCollaborators,

    UpdateNotification,
    CreateNotification,
    RemoveNotifications,

    CreateNotificationContext,
    RemoveNotificationContext,
    UpdateNotificationContext,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Envelope<T: Serialize> {
    pub r#type: MessageRequestType,

    #[serde(flatten)]
    pub request: T,
}

impl<T: Serialize + DeserializeOwned> Envelope<T> {
    pub fn new(r#type: MessageRequestType, body: T) -> Self {
        Self {
            r#type,
            request: body,
        }
    }
}

impl<T: Serialize> super::Transaction for Envelope<T> {
    fn to_value(self) -> Result<Value> {
        let event = TxDomainEvent {
            tx: Tx {
                doc: Doc {
                    obj: Obj {
                        class: Ref::from(crate::services::core::class::TxDomainEvent),
                    },

                    id: ksuid::Ksuid::generate().to_hex(),
                    space: "core:space:Tx".to_string(),

                    modified_on: None,
                    modified_by: None,
                    created_on: None,
                    created_by: None,
                },
                object_space: "core:space:Domain".to_string(),
            },

            domain: "communication".to_string(),
            event: self,
        };
        Ok(json::to_value(&event)?)
    }
}

pub trait EventClient {
    #[deprecated = "use transactor directly"]
    fn request_raw<T: Serialize + DeserializeOwned, R: DeserializeOwned>(
        &self,
        body: &T,
    ) -> impl Future<Output = Result<R>>;

    #[deprecated = "use transactor directly"]
    #[allow(deprecated)]
    fn request_for_result<T: Serialize + DeserializeOwned, R: DeserializeOwned>(
        &self,
        r#type: MessageRequestType,
        request: T,
    ) -> impl Future<Output = Result<R>> {
        async { self.request_raw(&Envelope::new(r#type, request)).await }
    }

    #[deprecated = "use transactor directly"]
    #[allow(deprecated)]
    fn request<T: Serialize + DeserializeOwned>(
        &self,
        r#type: MessageRequestType,
        request: T,
    ) -> impl Future<Output = Result<()>> {
        async {
            self.request_raw::<_, json::Value>(&Envelope::new(r#type, request))
                .await
                .map(|_| ())
        }
    }
}

impl EventClient for super::TransactorClient {
    async fn request_raw<T: Serialize, R: DeserializeOwned>(&self, envelope: &T) -> Result<R> {
        let path = format!("/api/v1/event/{}", self.workspace);
        let url = self.base.join(&path)?;

        <HttpClient as JsonClient>::post(&self.http, self, url, envelope).await
    }
}
