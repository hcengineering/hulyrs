//
// Copyright © 2025 Hardcore Engineering Inc.
//
// Licensed under the Eclipse Public License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License. You may
// obtain a copy of the License at https://www.eclipse.org/legal/epl-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//
// See the License for the specific language governing permissions and
// limitations under the License.
//

use crate::text::{AttrValue, MarkupMark, MarkupMarkType, MarkupNode, MarkupNodeType};
use std::collections::HashMap;

pub fn markup_to_html(markup: &MarkupNode) -> String {
    let serializer = HtmlSerializer::new();
    serializer.serialize(markup)
}

struct HtmlSerializer;

impl HtmlSerializer {
    fn new() -> Self {
        Self
    }

    fn serialize(&self, markup: &MarkupNode) -> String {
        let mut builder = NodeBuilder::new(true);
        add_node(&mut builder, markup);
        builder.to_text()
    }
}

struct NodeBuilder {
    text_parts: Vec<String>,
    add_tags: bool,
}

impl NodeBuilder {
    fn new(add_tags: bool) -> Self {
        Self {
            text_parts: Vec::new(),
            add_tags,
        }
    }

    fn add_text(&mut self, text: &str) {
        self.text_parts.push(text.to_string());
    }

    fn open_tag(
        &mut self,
        tag: &str,
        attributes: &HashMap<&str, Option<String>>,
        self_closing: bool,
    ) {
        if self.add_tags {
            self.text_parts.push("<".to_string());
            self.text_parts.push(tag.to_string());

            for (key, value) in attributes {
                if let Some(val) = value {
                    self.text_parts
                        .push(format!(" {}=\"{}\"", key, escape_html(val)));
                }
            }

            self.text_parts
                .push((if self_closing { "/>" } else { ">" }).to_string());
        }
    }

    fn close_tag(&mut self, tag: &str) {
        if self.add_tags {
            self.text_parts.push(format!("</{}>", tag));
        }
    }

    fn to_text(&self) -> String {
        self.text_parts.join("")
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#039;")
        .replace('\r', "&#13;")
        .replace('\n', "&#10;")
}

fn add_mark<F>(builder: &mut NodeBuilder, mark: &MarkupMark, next: F)
where
    F: FnOnce(&mut NodeBuilder),
{
    let attrs = &mark.attrs;

    match mark.mark_type {
        MarkupMarkType::Bold => {
            builder.open_tag("strong", &HashMap::new(), false);
            next(builder);
            builder.close_tag("strong");
        }
        MarkupMarkType::Code => {
            builder.open_tag("code", &HashMap::new(), false);
            next(builder);
            builder.close_tag("code");
        }
        MarkupMarkType::Italic => {
            builder.open_tag("em", &HashMap::new(), false);
            next(builder);
            builder.close_tag("em");
        }
        MarkupMarkType::Link => {
            let mut link_attrs = HashMap::new();
            if let Some(href) = attrs.get("href") {
                link_attrs.insert("href", Some(attr_to_string(href)));
            }
            if let Some(target) = attrs.get("target") {
                link_attrs.insert("target", Some(attr_to_string(target)));
            }
            if let Some(rel) = attrs.get("rel") {
                link_attrs.insert("rel", Some(attr_to_string(rel)));
            }
            if let Some(class) = attrs.get("class") {
                link_attrs.insert("class", Some(attr_to_string(class)));
            }
            if let Some(title) = attrs.get("title") {
                link_attrs.insert("title", Some(attr_to_string(title)));
            }
            builder.open_tag("a", &link_attrs, false);
            next(builder);
            builder.close_tag("a");
        }
        MarkupMarkType::Strike => {
            builder.open_tag("s", &HashMap::new(), false);
            next(builder);
            builder.close_tag("s");
        }
        MarkupMarkType::Underline => {
            builder.open_tag("u", &HashMap::new(), false);
            next(builder);
            builder.close_tag("u");
        }
        MarkupMarkType::TextColor => {
            let mut span_attrs = HashMap::new();
            if let Some(color) = attrs.get("color") {
                let color_str = attr_to_string(color);
                span_attrs.insert("style", Some(format!("color: {}", color_str)));
                span_attrs.insert("data-color", Some(color_str));
            }
            builder.open_tag("span", &span_attrs, false);
            next(builder);
            builder.close_tag("span");
        }
        _ => {
            let mut span_attrs = HashMap::new();
            span_attrs.insert(
                "data-mark-type",
                Some(format!("{:?}", mark.mark_type).to_lowercase()),
            );
            builder.open_tag("span", &span_attrs, false);
            next(builder);
            builder.close_tag("span");
        }
    }
}

fn add_marks<F>(builder: &mut NodeBuilder, marks: &[MarkupMark], next: F)
where
    F: FnOnce(&mut NodeBuilder),
{
    if marks.is_empty() {
        next(builder);
    } else {
        let mark = &marks[0];
        let remaining = &marks[1..];

        if !remaining.is_empty() {
            add_mark(builder, mark, |builder| {
                add_marks(builder, remaining, next);
            });
        } else {
            add_mark(builder, mark, next);
        }
    }
}

fn add_nodes(builder: &mut NodeBuilder, nodes: &[MarkupNode]) {
    for node in nodes {
        add_node(builder, node);
    }
}

fn add_node_content(builder: &mut NodeBuilder, node: &MarkupNode) {
    let attrs = &node.attrs;
    let nodes = &node.content;

    match node.node_type {
        MarkupNodeType::Doc => {
            add_nodes(builder, nodes);
        }
        MarkupNodeType::Paragraph => {
            let mut p_attrs = HashMap::new();
            if let Some(style) = to_style_attr(attrs) {
                p_attrs.insert("style", Some(style));
            }
            builder.open_tag("p", &p_attrs, false);
            add_nodes(builder, nodes);
            builder.close_tag("p");
        }
        MarkupNodeType::Blockquote => {
            builder.open_tag("blockquote", &HashMap::new(), false);
            add_nodes(builder, nodes);
            builder.close_tag("blockquote");
        }
        MarkupNodeType::HorizontalRule => {
            builder.open_tag("hr", &HashMap::new(), true);
        }
        MarkupNodeType::Heading => {
            let level = attr_to_number(attrs.get("level")).unwrap_or(1);
            let tag = format!("h{}", level);
            let mut h_attrs = HashMap::new();
            if let Some(style) = to_style_attr(attrs) {
                h_attrs.insert("style", Some(style));
            }
            builder.open_tag(&tag, &h_attrs, false);
            add_nodes(builder, nodes);
            builder.close_tag(&tag);
        }
        MarkupNodeType::CodeBlock => {
            builder.open_tag("pre", &HashMap::new(), false);
            let mut code_attrs = HashMap::new();
            if let Some(lang) = attrs.get("language") {
                code_attrs.insert("class", Some(format!("language-{}", attr_to_string(lang))));
            }
            builder.open_tag("code", &code_attrs, false);
            add_nodes(builder, nodes);
            builder.close_tag("code");
            builder.close_tag("pre");
        }
        MarkupNodeType::Text => {
            builder.add_text(&node.text);
        }
        MarkupNodeType::Image => {
            let mut img_attrs = HashMap::new();
            if let Some(src) = attrs.get("src") {
                img_attrs.insert("src", Some(attr_to_string(src)));
            }
            if let Some(alt) = attrs.get("alt") {
                img_attrs.insert("alt", Some(attr_to_string(alt)));
            }
            if let Some(width) = attrs.get("width") {
                img_attrs.insert("width", Some(attr_to_string(width)));
            }
            if let Some(height) = attrs.get("height") {
                img_attrs.insert("height", Some(attr_to_string(height)));
            }
            builder.open_tag("img", &img_attrs, true);
        }
        MarkupNodeType::Reference => {
            let mut ref_attrs = HashMap::new();
            ref_attrs.insert("data-type", Some("reference".to_string()));
            if let Some(id) = attrs.get("id") {
                ref_attrs.insert("data-id", Some(attr_to_string(id)));
            }
            if let Some(objectclass) = attrs.get("objectclass") {
                ref_attrs.insert("data-objectclass", Some(attr_to_string(objectclass)));
            }
            if let Some(label) = attrs.get("label") {
                ref_attrs.insert("data-label", Some(attr_to_string(label)));
            }
            builder.open_tag("span", &ref_attrs, false);
            if let Some(label) = attrs.get("label") {
                builder.add_text(&format!("@{}", attr_to_string(label)));
            }
            builder.close_tag("span");
        }
        MarkupNodeType::HardBreak => {
            builder.open_tag("br", &HashMap::new(), true);
        }
        MarkupNodeType::OrderedList => {
            builder.open_tag("ol", &HashMap::new(), false);
            add_nodes(builder, nodes);
            builder.close_tag("ol");
        }
        MarkupNodeType::BulletList => {
            builder.open_tag("ul", &HashMap::new(), false);
            add_nodes(builder, nodes);
            builder.close_tag("ul");
        }
        MarkupNodeType::ListItem => {
            builder.open_tag("li", &HashMap::new(), false);
            add_nodes(builder, nodes);
            builder.close_tag("li");
        }
        MarkupNodeType::TodoList => {
            let mut ul_attrs = HashMap::new();
            ul_attrs.insert("data-type", Some("todoList".to_string()));
            builder.open_tag("ul", &ul_attrs, false);
            add_nodes(builder, nodes);
            builder.close_tag("ul");
        }
        MarkupNodeType::TodoItem => {
            let checked = attr_to_bool(attrs.get("checked"));
            let disabled = attr_to_bool(attrs.get("disabled"));

            let mut li_attrs = HashMap::new();
            li_attrs.insert("data-type", Some("todoItem".to_string()));
            if let Some(todoid) = attrs.get("todoid") {
                li_attrs.insert("data-todoid", Some(attr_to_string(todoid)));
            }
            if let Some(userid) = attrs.get("userid") {
                li_attrs.insert("data-userid", Some(attr_to_string(userid)));
            }
            if checked {
                li_attrs.insert("data-checked", Some("true".to_string()));
            }

            builder.open_tag("li", &li_attrs, false);

            let mut input_attrs = HashMap::new();
            input_attrs.insert("type", Some("checkbox".to_string()));
            if checked {
                input_attrs.insert("checked", Some("checked".to_string()));
            }
            if disabled {
                input_attrs.insert("disabled", Some("disabled".to_string()));
            }
            builder.open_tag("input", &input_attrs, true);

            add_nodes(builder, nodes);
            builder.close_tag("li");
        }
        MarkupNodeType::TaskList => {
            let mut ul_attrs = HashMap::new();
            ul_attrs.insert("data-type", Some("taskList".to_string()));
            builder.open_tag("ul", &ul_attrs, false);
            add_nodes(builder, nodes);
            builder.close_tag("ul");
        }
        MarkupNodeType::TaskItem => {
            let checked = attr_to_bool(attrs.get("checked"));
            let disabled = attr_to_bool(attrs.get("disabled"));

            let mut li_attrs = HashMap::new();
            li_attrs.insert("data-type", Some("taskItem".to_string()));
            if checked {
                li_attrs.insert("data-checked", Some("true".to_string()));
            }

            builder.open_tag("li", &li_attrs, false);

            let mut input_attrs = HashMap::new();
            input_attrs.insert("type", Some("checkbox".to_string()));
            if checked {
                input_attrs.insert("checked", Some("checked".to_string()));
            }
            if disabled {
                input_attrs.insert("disabled", Some("disabled".to_string()));
            }
            builder.open_tag("input", &input_attrs, true);

            add_nodes(builder, nodes);
            builder.close_tag("li");
        }
        MarkupNodeType::SubLink => {
            builder.open_tag("sub", &HashMap::new(), false);
            add_nodes(builder, nodes);
            builder.close_tag("sub");
        }
        MarkupNodeType::Table => {
            builder.open_tag("table", &HashMap::new(), false);
            builder.open_tag("tbody", &HashMap::new(), false);
            add_nodes(builder, nodes);
            builder.close_tag("tbody");
            builder.close_tag("table");
        }
        MarkupNodeType::TableRow => {
            builder.open_tag("tr", &HashMap::new(), false);
            add_nodes(builder, nodes);
            builder.close_tag("tr");
        }
        MarkupNodeType::TableCell => {
            let colspan = attr_to_number(attrs.get("colspan")).unwrap_or(1);
            let rowspan = attr_to_number(attrs.get("rowspan")).unwrap_or(1);
            let colwidth = attr_to_number(attrs.get("colwidth"));

            let mut td_attrs = HashMap::new();
            if colspan != 1 {
                td_attrs.insert("colspan", Some(colspan.to_string()));
            }
            if rowspan != 1 {
                td_attrs.insert("rowspan", Some(rowspan.to_string()));
            }
            if let Some(width) = colwidth {
                if width > 0 {
                    td_attrs.insert("colwidth", Some(width.to_string()));
                }
            }

            builder.open_tag("td", &td_attrs, false);
            add_nodes(builder, nodes);
            builder.close_tag("td");
        }
        MarkupNodeType::TableHeader => {
            let colspan = attr_to_number(attrs.get("colspan")).unwrap_or(1);
            let rowspan = attr_to_number(attrs.get("rowspan")).unwrap_or(1);
            let colwidth = attr_to_number(attrs.get("colwidth"));

            let mut th_attrs = HashMap::new();
            if colspan != 1 {
                th_attrs.insert("colspan", Some(colspan.to_string()));
            }
            if rowspan != 1 {
                th_attrs.insert("rowspan", Some(rowspan.to_string()));
            }
            if let Some(width) = colwidth {
                if width > 0 {
                    th_attrs.insert("colwidth", Some(width.to_string()));
                }
            }

            builder.open_tag("th", &th_attrs, false);
            add_nodes(builder, nodes);
            builder.close_tag("th");
        }
        MarkupNodeType::Comment => {
            builder.add_text("<!-- ");
            add_nodes(builder, nodes);
            builder.add_text(" -->");
        }
        MarkupNodeType::Embed => {
            let src = attrs
                .get("src")
                .map(|s| attr_to_string(s))
                .unwrap_or_default();
            let encoded_src = encode_uri(&src);

            let mut a_attrs = HashMap::new();
            a_attrs.insert("href", Some(encoded_src.clone()));
            a_attrs.insert("data-type", Some("embed".to_string()));

            builder.open_tag("a", &a_attrs, false);
            builder.add_text(&escape_html(&src));
            builder.close_tag("a");
        }
        _ => {
            let mut div_attrs = HashMap::new();
            div_attrs.insert(
                "data-node-type",
                Some(format!("{:?}", node.node_type).to_lowercase()),
            );
            builder.open_tag("div", &div_attrs, false);
            add_nodes(builder, nodes);
            builder.close_tag("div");
        }
    }
}

fn add_node(builder: &mut NodeBuilder, node: &MarkupNode) {
    let marks = &node.marks;

    if !marks.is_empty() {
        add_marks(builder, marks, |builder| {
            add_node_content(builder, node);
        });
    } else {
        add_node_content(builder, node);
    }
}

fn attr_to_string(value: &AttrValue) -> String {
    match value {
        AttrValue::Str(s) => s.clone(),
        AttrValue::Num(n) => n.to_string(),
        AttrValue::Bool(b) => b.to_string(),
        AttrValue::Null | AttrValue::Undefined => String::new(),
    }
}

fn attr_to_number(value: Option<&AttrValue>) -> Option<i32> {
    value.and_then(|v| match v {
        AttrValue::Num(n) => Some(*n),
        AttrValue::Str(s) => s.parse().ok(),
        AttrValue::Bool(true) => Some(1),
        AttrValue::Bool(false) => Some(0),
        AttrValue::Null | AttrValue::Undefined => None,
    })
}

fn attr_to_bool(value: Option<&AttrValue>) -> bool {
    match value {
        Some(AttrValue::Bool(b)) => *b,
        Some(AttrValue::Str(s)) => s == "true",
        _ => false,
    }
}

fn to_style_attr(attrs: &HashMap<String, AttrValue>) -> Option<String> {
    let mut styles = Vec::new();

    if let Some(text_align) = attrs.get("textAlign") {
        styles.push(format!("text-align: {}", attr_to_string(text_align)));
    }

    if styles.is_empty() {
        None
    } else {
        Some(styles.join("; "))
    }
}

fn encode_uri(uri: &str) -> String {
    uri.chars()
        .map(|c| match c {
            ' ' => "%20".to_string(),
            '<' => "%3C".to_string(),
            '>' => "%3E".to_string(),
            '"' => "%22".to_string(),
            '{' => "%7B".to_string(),
            '}' => "%7D".to_string(),
            '|' => "%7C".to_string(),
            '\\' => "%5C".to_string(),
            '^' => "%5E".to_string(),
            '`' => "%60".to_string(),
            _ => c.to_string(),
        })
        .collect()
}
