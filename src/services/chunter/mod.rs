use crate::services::core::Space;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChunterSpace {
    #[serde(flatten)]
    pub space: Space,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    #[serde(flatten)]
    pub space: ChunterSpace,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
}

pub type DirectMessage = ChunterSpace;
