use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkupNodeType {
    Doc,
    Paragraph,
    Blockquote,
    HorizontalRule,
    Heading,
    CodeBlock,
    Text,
    Image,
    File,
    Reference,
    Emoji,
    HardBreak,
    OrderedList,
    BulletList,
    ListItem,
    TaskList,
    TaskItem,
    TodoList,
    TodoItem,
    SubLink,
    Table,
    TableRow,
    TableCell,
    TableHeader,
    Mermaid,
    Comment,
    Markdown,
    Embed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MarkupMarkType {
    Link,
    Italic,
    Bold,
    Code,
    Strike,
    Underline,
    TextColor,
    TextStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttrValue {
    Str(String),
    Num(i32),
    Bool(bool),
    Null,
    Undefined,
}

pub type Attrs = HashMap<String, AttrValue>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkupMark {
    #[serde(rename = "type")]
    pub mark_type: MarkupMarkType,
    pub attrs: Attrs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkupNode {
    #[serde(rename = "type")]
    pub node_type: MarkupNodeType,
    #[serde(default)]
    pub content: Vec<MarkupNode>,
    #[serde(default)]
    pub marks: Vec<MarkupMark>,
    #[serde(default)]
    pub attrs: Attrs,
    #[serde(default)]
    pub text: String,
}

pub fn get_str_attr(attrs: &HashMap<String, AttrValue>, key: &str) -> String {
    attrs
        .get(key)
        .and_then(|v| match v {
            AttrValue::Str(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default()
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

pub fn text(s: &str) -> MarkupNode {
    text_with_marks(s, vec![])
}

pub fn text_with_marks(s: &str, marks: Vec<MarkupMark>) -> MarkupNode {
    MarkupNode {
        node_type: MarkupNodeType::Text,
        content: Vec::new(),
        marks,
        attrs: HashMap::new(),
        text: s.to_string(),
    }
}

pub fn para(content: Vec<MarkupNode>) -> MarkupNode {
    MarkupNode {
        node_type: MarkupNodeType::Paragraph,
        content,
        marks: Vec::new(),
        attrs: HashMap::new(),
        text: String::new(),
    }
}

pub fn doc(content: Vec<MarkupNode>) -> MarkupNode {
    MarkupNode {
        node_type: MarkupNodeType::Doc,
        content,
        marks: Vec::new(),
        attrs: HashMap::new(),
        text: String::new(),
    }
}

pub fn mark(mark_type: MarkupMarkType, pairs: Vec<(&str, AttrValue)>) -> MarkupMark {
    MarkupMark {
        mark_type,
        attrs: attrs(pairs),
    }
}

pub fn bold_mark() -> MarkupMark {
    mark(MarkupMarkType::Bold, vec![])
}

pub fn italic_mark() -> MarkupMark {
    mark(MarkupMarkType::Italic, vec![])
}

pub fn strike_mark() -> MarkupMark {
    mark(MarkupMarkType::Strike, vec![])
}

pub fn underline_mark() -> MarkupMark {
    mark(MarkupMarkType::Underline, vec![])
}

pub fn link_mark(href: &str) -> MarkupMark {
    mark(
        MarkupMarkType::Link,
        vec![("href", AttrValue::Str(href.to_string()))],
    )
}

pub fn bold(s: &str) -> MarkupNode {
    text_with_marks(s, vec![bold_mark()])
}

pub fn code(s: &str) -> MarkupNode {
    text_with_marks(s, vec![mark(MarkupMarkType::Code, vec![])])
}

pub fn underline(s: &str) -> MarkupNode {
    text_with_marks(s, vec![underline_mark()])
}

pub fn link(href: &str, label: &str) -> MarkupNode {
    text_with_marks(label, vec![link_mark(href)])
}

pub fn heading(level: i32, content: &str) -> MarkupNode {
    node(
        MarkupNodeType::Heading,
        vec![text(content)],
        attrs(vec![
            ("level", AttrValue::Num(level)),
            ("marker", AttrValue::Str("#".to_string())),
        ]),
    )
}

pub fn heading_without_marker(level: i32, content: &str) -> MarkupNode {
    node(
        MarkupNodeType::Heading,
        vec![text(content)],
        attrs(vec![("level", AttrValue::Num(level))]),
    )
}

pub fn list_item(content: &str) -> MarkupNode {
    node(
        MarkupNodeType::ListItem,
        vec![para(vec![text(content)])],
        HashMap::new(),
    )
}

pub fn list_item_with(content: Vec<MarkupNode>) -> MarkupNode {
    node(MarkupNodeType::ListItem, content, HashMap::new())
}

pub fn bullet_list(items: Vec<MarkupNode>) -> MarkupNode {
    node(
        MarkupNodeType::BulletList,
        items,
        attrs(vec![("bullet", AttrValue::Str("-".to_string()))]),
    )
}

pub fn todo(content: &str, checked: bool) -> MarkupNode {
    node(
        MarkupNodeType::TodoItem,
        vec![para(vec![text(content)])],
        attrs(vec![("checked", AttrValue::Bool(checked))]),
    )
}

pub fn todo_with_ids(content: &str, checked: bool, todoid: &str, userid: &str) -> MarkupNode {
    node(
        MarkupNodeType::TodoItem,
        vec![para(vec![text(content)])],
        attrs(vec![
            ("checked", AttrValue::Bool(checked)),
            ("todoid", AttrValue::Str(todoid.to_string())),
            ("userid", AttrValue::Str(userid.to_string())),
        ]),
    )
}

pub fn todo_with(content: Vec<MarkupNode>, checked: bool) -> MarkupNode {
    node(
        MarkupNodeType::TodoItem,
        content,
        attrs(vec![("checked", AttrValue::Bool(checked))]),
    )
}

pub fn todo_list(items: Vec<MarkupNode>) -> MarkupNode {
    node(
        MarkupNodeType::TodoList,
        items,
        attrs(vec![("bullet", AttrValue::Str("-".to_string()))]),
    )
}

pub fn ordered_list(items: Vec<MarkupNode>) -> MarkupNode {
    node(MarkupNodeType::OrderedList, items, HashMap::new())
}

pub fn ordered_list_from(start: i32, items: Vec<MarkupNode>) -> MarkupNode {
    node(
        MarkupNodeType::OrderedList,
        items,
        attrs(vec![("order", AttrValue::Num(start))]),
    )
}

pub fn hard_break() -> MarkupNode {
    node(MarkupNodeType::HardBreak, vec![], HashMap::new())
}

pub fn hard_break_with_marks(marks: Vec<MarkupMark>) -> MarkupNode {
    MarkupNode {
        node_type: MarkupNodeType::HardBreak,
        content: vec![],
        marks,
        attrs: HashMap::new(),
        text: String::new(),
    }
}

pub fn node(
    node_type: MarkupNodeType,
    content: Vec<MarkupNode>,
    attrs: HashMap<String, AttrValue>,
) -> MarkupNode {
    MarkupNode {
        node_type,
        content,
        marks: Vec::new(),
        attrs,
        text: String::new(),
    }
}

pub fn attrs(pairs: Vec<(&str, AttrValue)>) -> HashMap<String, AttrValue> {
    pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

pub fn embed(src: &str) -> MarkupNode {
    node(
        MarkupNodeType::Embed,
        vec![],
        attrs(vec![("src", AttrValue::Str(src.to_string()))]),
    )
}

pub fn image(src: &str, alt: &str) -> MarkupNode {
    node(
        MarkupNodeType::Image,
        vec![],
        attrs(vec![
            ("src", AttrValue::Str(src.to_string())),
            ("alt", AttrValue::Str(alt.to_string())),
        ]),
    )
}

pub fn image_with_attrs(attrs: HashMap<String, AttrValue>) -> MarkupNode {
    node(MarkupNodeType::Image, vec![], attrs)
}

pub fn mermaid(code: &str) -> MarkupNode {
    node(
        MarkupNodeType::Mermaid,
        vec![text(code)],
        attrs(vec![("language", AttrValue::Str("mermaid".to_string()))]),
    )
}

pub fn code_block(lang: &str, code: &str) -> MarkupNode {
    let attrs_map = if !lang.is_empty() {
        attrs(vec![("language", AttrValue::Str(lang.to_string()))])
    } else {
        HashMap::new()
    };
    node(MarkupNodeType::CodeBlock, vec![text(code)], attrs_map)
}

pub fn blockquote(content: Vec<MarkupNode>) -> MarkupNode {
    node(MarkupNodeType::Blockquote, content, HashMap::new())
}

pub fn hr() -> MarkupNode {
    node(MarkupNodeType::HorizontalRule, vec![], HashMap::new())
}

pub fn hr_custom(markup: &str) -> MarkupNode {
    node(
        MarkupNodeType::HorizontalRule,
        vec![],
        attrs(vec![("markup", AttrValue::Str(markup.to_string()))]),
    )
}

pub fn emoji(emoji_char: &str) -> MarkupNode {
    node(
        MarkupNodeType::Emoji,
        vec![],
        attrs(vec![("emoji", AttrValue::Str(emoji_char.to_string()))]),
    )
}

pub fn comment(content: &str) -> MarkupNode {
    node(MarkupNodeType::Comment, vec![text(content)], HashMap::new())
}

pub fn markdown_node(content: &str) -> MarkupNode {
    node(
        MarkupNodeType::Markdown,
        vec![text(content)],
        HashMap::new(),
    )
}

pub fn task_item(content: &str) -> MarkupNode {
    node(
        MarkupNodeType::TaskItem,
        vec![para(vec![text(content)])],
        HashMap::new(),
    )
}

pub fn task_list(items: Vec<MarkupNode>) -> MarkupNode {
    node(MarkupNodeType::TaskList, items, HashMap::new())
}

pub fn table_header(content: &str) -> MarkupNode {
    node(
        MarkupNodeType::TableHeader,
        vec![text(content)],
        HashMap::new(),
    )
}

pub fn table_cell(content: &str) -> MarkupNode {
    node(
        MarkupNodeType::TableCell,
        vec![text(content)],
        HashMap::new(),
    )
}

pub fn table_cell_with(content: Vec<MarkupNode>) -> MarkupNode {
    node(MarkupNodeType::TableCell, content, HashMap::new())
}

pub fn table_row(cells: Vec<MarkupNode>) -> MarkupNode {
    node(MarkupNodeType::TableRow, cells, HashMap::new())
}

pub fn table(rows: Vec<MarkupNode>) -> MarkupNode {
    node(MarkupNodeType::Table, rows, HashMap::new())
}

pub fn sublink(content: Vec<MarkupNode>) -> MarkupNode {
    node(MarkupNodeType::SubLink, content, HashMap::new())
}

pub fn reference(pairs: Vec<(&str, &str)>) -> MarkupNode {
    let attrs_map = pairs
        .into_iter()
        .map(|(k, v)| (k.to_string(), AttrValue::Str(v.to_string())))
        .collect();
    MarkupNode {
        node_type: MarkupNodeType::Reference,
        content: vec![],
        marks: Vec::new(),
        attrs: attrs_map,
        text: String::new(),
    }
}
