use std::collections::HashMap;

use regex::Regex;

#[allow(dead_code)]
pub fn normalize_markdown(source: &str) -> String {
    if source.is_empty() {
        return String::new();
    }

    let mut result = source.to_string();

    result = result.replace("\r\n", "\n").replace('\r', "\n");

    result = result
        .split('\n')
        .map(|line| line.trim_end())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    let tag_regex = Regex::new(r"<(\w+)([^>]*?)(\/?)>").unwrap();
    result = tag_regex
        .replace_all(&result, |caps: &regex::Captures| {
            let tag_name = &caps[1];
            let attributes = &caps[2];

            let mut attrs: HashMap<String, String> = HashMap::new();

            let attr_regex = Regex::new(r#"(\w+)(?:=(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?"#).unwrap();
            for attr_match in attr_regex.captures_iter(attributes) {
                let attr_name = attr_match[1].to_string();
                let attr_value = attr_match
                    .get(2)
                    .or(attr_match.get(3))
                    .or(attr_match.get(4))
                    .map(|m| m.as_str())
                    .unwrap_or("");
                attrs.insert(attr_name, attr_value.to_string());
            }

            let mut sorted_keys: Vec<_> = attrs.keys().collect();
            sorted_keys.sort();

            let sorted_attrs = sorted_keys
                .iter()
                .map(|key| {
                    let value = &attrs[*key];
                    if !value.is_empty() {
                        format!("{}=\"{}\"", key, value)
                    } else {
                        (*key).to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            let void_elements = [
                "img", "br", "hr", "input", "meta", "area", "base", "col", "embed", "link",
                "param", "source", "track", "wbr",
            ];
            let is_void_element = void_elements.contains(&tag_name.to_lowercase().as_str());

            if !sorted_attrs.is_empty() {
                if is_void_element {
                    format!("<{} {} />", tag_name, sorted_attrs)
                } else {
                    format!("<{} {}>", tag_name, sorted_attrs)
                }
            } else if is_void_element {
                format!("<{} />", tag_name)
            } else {
                format!("<{}>", tag_name)
            }
        })
        .to_string();

    result
}
