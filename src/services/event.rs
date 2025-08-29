use crate::services::transactor::tx::Doc;
use serde_json::Value;

pub trait DocT: Class + HasId {
    fn doc(&self) -> &Doc;
}

pub trait Class {
    const CLASS: &'static str;
}

pub trait HasId {
    fn id(&self) -> &str;
}

pub trait Event: Class {
    fn matches(value: &Value) -> bool {
        value.get("_class").and_then(|v| v.as_str()) == Some(Self::CLASS)
    }
}
