use crate::services::event::{Class, HasId};
use crate::services::platform::Asset;
use crate::services::transactor::tx::{Doc, Obj};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub type Ref = String;
pub type Timestamp = chrono::DateTime<chrono::Utc>;
pub type Markup = String;
pub type Hyperlink = String;
pub type Rank = String;
pub type MarkupBlobRef = Ref;
pub type AccountUuid = Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct UXObject {
    #[serde(flatten)]
    pub obj: Obj,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Asset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(rename = "readonly")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttachedDoc {
    #[serde(flatten)]
    pub doc: Doc,
    pub attached_to: Ref,
    pub attached_to_class: Ref,
}

pub type OperationDomain = String;

pub type BlobMetadata = HashMap<String, serde_json::Value>;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Blob {
    #[serde(flatten)]
    pub doc: Doc,
    pub provider: String,
    pub content_type: String,
    pub etag: String,
    pub version: Option<String>,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BlobType {
    pub file: Ref,
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BlobMetadata>,
}

pub type Blobs = HashMap<String, BlobType>;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Space {
    #[serde(flatten)]
    pub doc: Doc,

    pub name: String,
    pub description: String,
    pub private: bool,
    pub archived: bool,
    pub members: Vec<AccountUuid>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owners: Vec<AccountUuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_join: Option<bool>,
}

impl Class for Space {
    const CLASS: &'static str = "core:class:Space";
}

impl HasId for Space {
    fn id(&self) -> &str {
        &self.doc.id
    }
}
