use crate::services::transactor::document::Doc;
use crate::services::types::AccountUuid;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Space {
    #[serde(flatten)]
    pub doc: Doc,
    pub name: String,
    pub private: bool,
    pub members: Vec<AccountUuid>,
    pub archived: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owners: Option<Vec<AccountUuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_join: Option<bool>,
}
