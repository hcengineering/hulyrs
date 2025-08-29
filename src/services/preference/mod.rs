use crate::services::transactor::tx::Doc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Preference {
    pub doc: Doc,
    pub attached_to: String,
}
