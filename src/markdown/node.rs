use std::collections::HashMap;

use crate::text::AttrValue;

pub fn get_str_attr(attrs: &HashMap<String, AttrValue>, key: &str) -> String {
    attrs.get(key).and_then(|v| match v {
        AttrValue::Str(s) => Some(s.clone()),
        _ => None,
    }).unwrap_or_default()
}

pub fn get_num_attr(attrs: &HashMap<String, AttrValue>, key: &str) -> Option<i32> {
    attrs.get(key).and_then(|v| match v {
        AttrValue::Num(n) => Some(*n),
        _ => None,
    })
}

pub fn get_bool_attr(attrs: &HashMap<String, AttrValue>, key: &str) -> Option<bool> {
    attrs.get(key).and_then(|v| match v {
        AttrValue::Bool(b) => Some(*b),
        _ => None,
    })
}
