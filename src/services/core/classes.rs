use crate::services::transactor::tx::Doc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Ref = String;
pub type Timestamp = chrono::DateTime<chrono::Utc>;
pub type Markup = String;
pub type Hyperlink = String;
pub type Rank = String;
pub type MarkupBlobRef = Ref;

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
