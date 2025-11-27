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
    pub content: Vec<MarkupNode>,
    pub marks: Option<Vec<MarkupMark>>,
    pub attrs: Attrs,
    pub text: Option<String>,
}
