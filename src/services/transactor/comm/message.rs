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
use serde::{Deserialize, Serialize};
use serde_json::{self as json};

use crate::services::core::{PersonId, Timestamp};

type Date = chrono::DateTime<chrono::Utc>;

type MessageId = String;
type CardId = String;
type CardType = String;
type Markdown = String;
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

pub type MessageExtra = HashMap<String, json::Value>;
pub type BlobMetadata = HashMap<String, json::Value>;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    #[default]
    Message,
    Activity,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageOptions {
    // Available for regular users (Not implemented yet)
    #[builder(default)]
    #[serde(default)]
    skip_link_previews: bool,

    // Available only for system
    #[builder(default)]
    #[serde(default)]
    no_notify: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageEvent {
    #[builder(setter(into))]
    pub card_id: CardId,

    #[builder(setter(into, strip_option))]
    pub card_type: CardType,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,

    #[builder(default)]
    pub message_type: MessageType,

    #[builder(setter(into))]
    pub content: Markdown,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<MessageExtra>,

    #[builder(setter(into))]
    pub social_id: PersonId,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Timestamp>,

    #[builder(setter(strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<CreateMessageOptions>,
}
message_event!(CreateMessageEvent, card_id);

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePatchOptions {
    #[builder(default)]
    #[serde(default)]
    skip_link_previews_update: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePatchEvent {
    #[builder(setter(into))]
    pub card_id: CardId,

    #[builder(setter(into))]
    pub message_id: String,

    #[builder(setter(into))]
    pub content: Markdown,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<MessageExtra>,

    #[builder(setter(into))]
    pub social_id: PersonId,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Timestamp>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<UpdatePatchOptions>,
}
message_event!(UpdatePatchEvent, card_id);

#[derive(Serialize, Deserialize, Debug, Builder)]
#[serde(rename_all = "camelCase")]
pub struct RemovePatchEvent {
    #[builder(setter(into))]
    pub card_id: CardId,

    #[builder(setter(into))]
    pub message_id: MessageId,

    #[builder(setter(into))]
    pub social_id: PersonId,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Timestamp>,
}
message_event!(RemovePatchEvent, card_id);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "opcode", rename_all = "lowercase")]
pub enum ReactionPatchOperation {
    Add { reaction: String },
    Remove { reaction: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
#[serde(rename_all = "camelCase")]
pub struct ReactionPatchEvent {
    #[builder(setter(into))]
    pub card_id: CardId,

    #[builder(setter(into))]
    pub message_id: MessageId,

    pub operation: ReactionPatchOperation,

    #[builder(setter(into))]
    pub social_id: PersonId,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Timestamp>,
}
message_event!(ReactionPatchEvent, card_id);

#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlobData {
    #[builder(setter(into))]
    pub blob_id: BlobId,

    #[builder(setter(into))]
    pub mime_type: String,

    #[builder(setter(into))]
    pub file_name: String,

    #[builder(setter(into), default)]
    pub size: u32,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BlobMetadata>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "opcode", rename_all = "lowercase")]
pub enum BlobPatchOperation {
    Attach {
        blobs: Vec<BlobData>,
    },
    Detach {
        #[serde(rename = "blobIds")]
        blob_ids: Vec<BlobId>,
    },
    Set {
        blobs: Vec<BlobData>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
#[serde(rename_all = "camelCase")]
pub struct BlobPatchEvent {
    #[builder(setter(into))]
    pub card_id: CardId,

    #[builder(setter(into))]
    pub message_id: MessageId,

    pub operations: Vec<BlobPatchOperation>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_id: Option<PersonId>,

    #[builder(setter(into, strip_option), default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Timestamp>,
}
message_event!(BlobPatchEvent, card_id);

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateMessageResult {
    pub message_id: MessageId,
    pub created: Date,
}
