use serde_json::Value;

pub trait Class {
    const CLASS: &'static str;
}

pub trait Event: Class {
    fn matches(value: &Value) -> bool {
        value.get("_class").and_then(|v| v.as_str()) == Some(Self::CLASS)
    }
}
