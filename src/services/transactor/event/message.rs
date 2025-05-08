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
use serde_json as json;

use derive_builder::Builder;

use crate::services::types::{PersonId, Timestamp};

type Date = chrono::DateTime<chrono::Utc>;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    #[default]
    Message,
    Activity,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RepliesUpdate {
    Increment,
    Decrement,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessagesGroup {
    pub card: CardId,
    pub blob_id: BlobId,
    pub from_sec: Date,
    pub to_sec: Date,
    pub count: u32,
    pub patches: Option<Vec<Patch>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub message: MessageId,
    pub r#type: PatchType,
    pub content: String,
    pub creator: PersonId,
    pub created: Date,
}

type MessageId = String;
type CardId = String;
type CardType = String;
type RichText = String;
type MessageData = json::Value;
type BlobId = String;

pub trait PartitionKeyProvider {
    fn partition_key(&self) -> &str;
}

macro_rules! message_event {
    ($name:ident, $field:ident) => {
        impl PartitionKeyProvider for $name {
            fn partition_key(&self) -> &str {
                &self.$field
            }
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageEvent {
    #[builder(default)]
    pub message_type: MessageType,

    #[builder(setter(into))]
    pub card: CardId,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_type: Option<CardType>,

    #[builder(setter(into))]
    pub content: RichText,

    #[builder(setter(into))]
    pub creator: PersonId,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<MessageData>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<Timestamp>,
}
message_event!(CreateMessageEvent, card);

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMessagesEvent {
    // pub r#type: MessageRequestEventType,
    pub card: CardId,
    pub messages: Vec<MessageId>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PatchType {
    Update,
    AddReaction,
    RemoveReaction,
    AddReply,
    RemoveReply,
    AddFile,
    RemoveFile,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatePatchEvent {
    //  pub r#type: MessageRequestEventType,
    pub patch_type: PatchType,
    pub card: CardId,
    pub message: MessageId,
    pub content: RichText,
    pub creator: PersonId,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReactionEvent {
    //  pub r#type: MessageRequestEventType,
    pub card: CardId,
    pub message: MessageId,
    pub reaction: String,
    pub creator: PersonId,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveReactionEvent {
    //  pub r#type: MessageRequestEventType,
    pub card: CardId,
    pub message: MessageId,
    pub reaction: String,
    pub creator: PersonId,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileEvent {
    //  pub r#type: MessageRequestEventType,
    pub card: CardId,
    pub message: MessageId,
    pub blob_id: BlobId,
    pub size: u32,
    pub file_type: String,
    pub filename: String,
    pub creator: PersonId,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveFileEvent {
    //  pub r#type: MessageRequestEventType,
    pub card: CardId,
    pub message: MessageId,
    pub blob_id: BlobId,
    pub creator: PersonId,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateThreadEvent {
    //  pub r#type: MessageRequestEventType,
    pub card: CardId,
    pub message: MessageId,
    pub thread: CardId,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateThreadEvent {
    //  pub r#type: MessageRequestEventType,
    pub thread: CardId,
    pub replies: RepliesUpdate,
    pub last_reply: Option<Date>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessagesGroupEvent {
    //  pub r#type: MessageRequestEventType,
    pub group: MessagesGroup,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMessagesGroupEvent {
    // pub r#type: MessageRequestEventType,
    pub card: CardId,
    pub blob_id: BlobId,
}
