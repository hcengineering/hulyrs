use crate::services::core::classes::Ref;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Icon {
    ColorOrCodepoint(u32),
    Codepoints(Vec<u32>),
    BlobRef(Ref),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum IconSize {
    #[serde(rename = "inline")]
    Inline,
    #[serde(rename = "tiny")]
    Tiny,
    #[serde(rename = "card")]
    Card,
    #[serde(rename = "xx-small")]
    XxSmall,
    #[serde(rename = "x-small")]
    XSmall,
    #[serde(rename = "smaller")]
    Smaller,
    #[serde(rename = "small")]
    Small,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "large")]
    Large,
    #[serde(rename = "x-large")]
    XLarge,
    #[serde(rename = "2x-large")]
    DoubleXLarge,
    #[serde(rename = "full")]
    Full,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct IconProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Icon>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<IconSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filled: Option<bool>,
}
