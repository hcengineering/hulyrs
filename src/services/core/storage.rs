use crate::services::core::classes::OperationDomain;
use crate::services::event::{Class, HasId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Source {
    #[serde(rename = "$score")]
    pub score: f64,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WithLookup<T> {
    #[serde(flatten)]
    pub doc: T,

    #[serde(rename = "$lookup", skip_serializing_if = "Option::is_none")]
    pub lookup: Option<HashMap<String, Value>>,

    #[serde(rename = "$associations", skip_serializing_if = "Option::is_none")]
    pub associations: Option<HashMap<String, Vec<Value>>>,

    #[serde(rename = "$source", skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
}

impl<C: Class> Class for WithLookup<C> {
    const CLASS: &'static str = C::CLASS;
}

impl<C: HasId> HasId for WithLookup<C> {
    fn id(&self) -> &str {
        self.doc.id()
    }
}

impl<C: PartialEq> PartialEq for WithLookup<C> {
    fn eq(&self, other: &Self) -> bool {
        self.doc.eq(&other.doc)
    }
}

impl<T> WithLookup<T> {
    pub fn into_inner(self) -> T {
        self.doc
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DomainResult<T> {
    pub domain: OperationDomain,
    pub value: T,
}

impl<T: PartialEq> PartialEq for DomainResult<T> {
    fn eq(&self, other: &Self) -> bool {
        self.domain.eq(&other.domain) && self.value.eq(&other.value)
    }
}
