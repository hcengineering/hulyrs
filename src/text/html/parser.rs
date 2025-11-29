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
use html5ever::tendril::TendrilSink;
use html5ever::{ParseOpts, QualName, local_name, ns, parse_fragment};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::collections::HashMap;

pub fn html_to_markup(html: &str) -> MarkupNode {
    let parser = HtmlParser::new();
    parser.parse(html)
}

struct HtmlParseState {
    stack: Vec<MarkupNode>,
    marks: Vec<MarkupMark>,
}

impl HtmlParseState {
    fn new(root: MarkupNode) -> Self {
        Self {
            stack: vec![root],
            marks: Vec::new(),
        }
    }

    fn top(&self) -> Option<&MarkupNode> {
        self.stack.last()
    }

    fn top_mut(&mut self) -> Option<&mut MarkupNode> {
        self.stack.last_mut()
    }

    fn add_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        if self.top().map(|n| &n.node_type) == Some(&MarkupNodeType::Doc) {
            self.open_node(MarkupNodeType::Paragraph, None);
            self.push(self.create_text_node(text));
            self.close_node(MarkupNodeType::Paragraph);
        } else {
            self.push(self.create_text_node(text));
        }
    }

    fn create_text_node(&self, text: &str) -> MarkupNode {
        MarkupNode {
            node_type: MarkupNodeType::Text,
            text: text.to_string(),
            marks: self.marks.clone(),
            content: Vec::new(),
            attrs: HashMap::new(),
        }
    }

    fn create_node(&self, node_type: MarkupNodeType) -> MarkupNode {
        MarkupNode {
            node_type,
            content: Vec::new(),
            attrs: HashMap::new(),
            marks: Vec::new(),
            text: String::new(),
        }
    }

    fn open_mark(&mut self, mark_type: MarkupMarkType, attrs: Option<HashMap<String, AttrValue>>) {
        self.marks.push(MarkupMark {
            mark_type,
            attrs: attrs.unwrap_or_default(),
        });
    }

    fn close_mark(&mut self, mark: MarkupMarkType) {
        if self.marks.last().map(|m| &m.mark_type) == Some(&mark) {
            self.marks.pop();
        }
    }

    fn open_node(&mut self, node_type: MarkupNodeType, attrs: Option<HashMap<String, AttrValue>>) {
        self.stack.push(MarkupNode {
            node_type,
            attrs: attrs.unwrap_or_default(),
            content: Vec::new(),
            marks: Vec::new(),
            text: String::new(),
        });
    }

    fn close_node(&mut self, _node_type: MarkupNodeType) {
        self.marks.clear();
        if let Some(info) = self.stack.pop() {
            self.push(info);
        }
    }

    fn push(&mut self, node: MarkupNode) {
        if let Some(parent) = self.top_mut() {
            parent.content.push(node);
        }
    }
}

pub struct HtmlParser;

impl HtmlParser {
    fn new() -> Self {
        Self
    }
    pub fn parse(&self, html: &str) -> MarkupNode {
        let mut state = HtmlParseState::new(Self::create_doc());

        let dom = parse_fragment(
            RcDom::default(),
            ParseOpts::default(),
            QualName::new(None, ns!(html), local_name!("body")),
            vec![],
            false,
        )
        .one(html);

        for child in dom.document.children.borrow().iter() {
            self.process_node(child, &mut state);
        }

        state
            .stack
            .into_iter()
            .next()
            .unwrap_or_else(Self::create_doc)
    }

    fn create_doc() -> MarkupNode {
        MarkupNode {
            node_type: MarkupNodeType::Doc,
            content: Vec::new(),
            attrs: HashMap::new(),
            marks: Vec::new(),
            text: String::new(),
        }
    }

    fn process_node(&self, handle: &Handle, state: &mut HtmlParseState) {
        match handle.data {
            NodeData::Document => {
                for child in handle.children.borrow().iter() {
                    self.process_node(child, state);
                }
            }
            NodeData::Text { ref contents } => {
                let text = contents.borrow().to_string();
                state.add_text(&text);
            }
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let tag_name = name.local.as_ref();
                let attributes = self.extract_attributes(attrs);

                self.handle_open_tag(state, tag_name, &attributes);

                for child in handle.children.borrow().iter() {
                    self.process_node(child, state);
                }

                self.handle_close_tag(state, tag_name, &attributes);
            }
            _ => {}
        }
    }

    fn extract_attributes(
        &self,
        attrs: &std::cell::RefCell<Vec<html5ever::Attribute>>,
    ) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for attr in attrs.borrow().iter() {
            result.insert(attr.name.local.as_ref().to_string(), attr.value.to_string());
        }
        result
    }

    fn handle_open_tag(
        &self,
        state: &mut HtmlParseState,
        tag: &str,
        attributes: &HashMap<String, String>,
    ) {
        let mut attrs_with_style = attributes.clone();
        if let Some(style) = attributes.get("style") {
            self.extract_style_attrs(&mut attrs_with_style, style);
        }

        match tag {
            "p" => {
                let top_type = state.top().map(|n| n.node_type.clone());
                if top_type != Some(MarkupNodeType::Paragraph) {
                    let mut attrs = HashMap::new();
                    if let Some(text_align) = attrs_with_style.get("textAlign") {
                        attrs.insert("textAlign".to_string(), AttrValue::Str(text_align.clone()));
                    }
                    state.open_node(MarkupNodeType::Paragraph, Some(attrs));
                }
            }

            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                let level = tag.chars().nth(1).unwrap().to_digit(10).unwrap() as i32;
                let mut attrs = HashMap::new();
                attrs.insert("level".to_string(), AttrValue::Num(level));
                if let Some(text_align) = attrs_with_style.get("textAlign") {
                    attrs.insert("textAlign".to_string(), AttrValue::Str(text_align.clone()));
                }
                state.open_node(MarkupNodeType::Heading, Some(attrs));
            }

            "blockquote" => state.open_node(MarkupNodeType::Blockquote, None),
            "pre" => state.open_node(MarkupNodeType::CodeBlock, None),
            "hr" => state.push(state.create_node(MarkupNodeType::HorizontalRule)),
            "br" => state.push(state.create_node(MarkupNodeType::HardBreak)),

            "ol" => {
                let mut attrs = HashMap::new();
                if let Some(start) = attributes.get("start") {
                    if let Ok(num) = start.parse::<i32>() {
                        attrs.insert("start".to_string(), AttrValue::Num(num));
                    }
                }
                state.open_node(MarkupNodeType::OrderedList, Some(attrs));
            }
            "ul" => {
                let data_type = attributes.get("data-type").map(|s| s.as_str());
                match data_type {
                    Some("todoList") => state.open_node(MarkupNodeType::TodoList, None),
                    Some("taskList") => state.open_node(MarkupNodeType::TaskList, None),
                    _ => state.open_node(MarkupNodeType::BulletList, None),
                }
            }
            "li" => {
                let data_type = attributes.get("data-type").map(|s| s.as_str());
                match data_type {
                    Some("todoItem") => {
                        let mut attrs = HashMap::new();
                        if let Some(checked) = attributes.get("data-checked") {
                            attrs.insert("checked".to_string(), AttrValue::Bool(checked == "true"));
                        }
                        if let Some(todoid) = attributes.get("data-todoid") {
                            attrs.insert("todoid".to_string(), AttrValue::Str(todoid.clone()));
                        }
                        if let Some(userid) = attributes.get("data-userid") {
                            attrs.insert("userid".to_string(), AttrValue::Str(userid.clone()));
                        }
                        state.open_node(MarkupNodeType::TodoItem, Some(attrs));
                    }
                    Some("taskItem") => {
                        let mut attrs = HashMap::new();
                        if let Some(checked) = attributes.get("data-checked") {
                            attrs.insert("checked".to_string(), AttrValue::Bool(checked == "true"));
                        }
                        state.open_node(MarkupNodeType::TaskItem, Some(attrs));
                    }
                    _ => state.open_node(MarkupNodeType::ListItem, None),
                }
            }

            "img" => {
                let mut attrs = HashMap::new();
                self.add_str_attr(&mut attrs, attributes, "src", "src");
                self.add_str_attr(&mut attrs, attributes, "alt", "alt");
                self.add_str_attr(&mut attrs, attributes, "title", "title");
                self.add_str_attr(&mut attrs, attributes, "file-id", "file-id");
                self.add_num_attr(&mut attrs, attributes, "width", "width");
                self.add_num_attr(&mut attrs, attributes, "height", "height");

                let should_wrap =
                    state.top().map(|n| &n.node_type) != Some(&MarkupNodeType::Paragraph);
                if should_wrap {
                    state.open_node(MarkupNodeType::Paragraph, None);
                }

                state.push(MarkupNode {
                    node_type: MarkupNodeType::Image,
                    content: Vec::new(),
                    attrs,
                    marks: Vec::new(),
                    text: String::new(),
                });

                if should_wrap {
                    state.close_node(MarkupNodeType::Paragraph);
                }
            }

            "table" => state.open_node(MarkupNodeType::Table, None),
            "tr" => state.open_node(MarkupNodeType::TableRow, None),
            "td" => {
                let attrs = self.extract_table_cell_attrs(attributes);
                state.open_node(MarkupNodeType::TableCell, Some(attrs));
                state.open_node(MarkupNodeType::Paragraph, None);
            }
            "th" => {
                let attrs = self.extract_table_cell_attrs(attributes);
                state.open_node(MarkupNodeType::TableHeader, Some(attrs));
                state.open_node(MarkupNodeType::Paragraph, None);
            }

            "sub" => state.open_node(MarkupNodeType::SubLink, None),

            "b" | "strong" => state.open_mark(MarkupMarkType::Bold, None),
            "em" | "i" => state.open_mark(MarkupMarkType::Italic, None),
            "s" => state.open_mark(MarkupMarkType::Strike, None),
            "u" => state.open_mark(MarkupMarkType::Underline, None),

            "span" => {
                let data_type = attributes.get("data-type").map(|s| s.as_str());
                let data_color = attributes.get("data-color");
                let style = attributes.get("style");

                if data_type == Some("reference") {
                    let mut attrs = HashMap::new();
                    if let Some(id) = attributes.get("data-id") {
                        attrs.insert("id".to_string(), AttrValue::Str(id.clone()));
                    }
                    if let Some(objectclass) = attributes.get("data-objectclass") {
                        attrs.insert(
                            "objectclass".to_string(),
                            AttrValue::Str(objectclass.clone()),
                        );
                    }
                    if let Some(label) = attributes.get("data-label") {
                        attrs.insert("label".to_string(), AttrValue::Str(label.clone()));
                    }
                    state.open_node(MarkupNodeType::Reference, Some(attrs));
                } else if let Some(color) = data_color {
                    let mut attrs = HashMap::new();
                    attrs.insert("color".to_string(), AttrValue::Str(color.clone()));
                    state.open_mark(MarkupMarkType::TextColor, Some(attrs));
                } else if let Some(style_str) = style {
                    let style_attrs = self.parse_style_to_attrs(style_str);
                    if !style_attrs.is_empty() {
                        state.open_mark(MarkupMarkType::TextStyle, Some(style_attrs));
                    }
                }
            }

            "code" => {
                let top_type = state.top().map(|n| n.node_type.clone());
                if top_type == Some(MarkupNodeType::CodeBlock) {
                    if let Some(class) = attributes.get("class") {
                        for cls in class.split_whitespace() {
                            if let Some(lang) = cls.strip_prefix("language-") {
                                if let Some(top) = state.stack.last_mut() {
                                    top.attrs.insert(
                                        "language".to_string(),
                                        AttrValue::Str(lang.to_string()),
                                    );
                                }
                            }
                        }
                    }
                } else {
                    state.open_mark(MarkupMarkType::Code, None);
                }
            }

            "a" => {
                let data_type = attributes.get("data-type").map(|s| s.as_str());
                if data_type == Some("embed") {
                    let mut attrs = HashMap::new();
                    if let Some(href) = attributes.get("href") {
                        let decoded =
                            urlencoding::decode(href).unwrap_or(std::borrow::Cow::Borrowed(href));
                        attrs.insert("src".to_string(), AttrValue::Str(decoded.to_string()));
                    }

                    let should_wrap =
                        state.top().map(|n| n.node_type.clone()) != Some(MarkupNodeType::Paragraph);
                    if should_wrap {
                        state.open_node(MarkupNodeType::Paragraph, None);
                    }

                    state.open_node(MarkupNodeType::Embed, Some(attrs));
                } else {
                    let mut attrs = HashMap::new();
                    if let Some(href) = attributes.get("href") {
                        attrs.insert("href".to_string(), AttrValue::Str(href.clone()));
                    }
                    if let Some(title) = attributes.get("title") {
                        attrs.insert("title".to_string(), AttrValue::Str(title.clone()));
                    }
                    if let Some(target) = attributes.get("target") {
                        attrs.insert("target".to_string(), AttrValue::Str(target.clone()));
                    }
                    if let Some(rel) = attributes.get("rel") {
                        attrs.insert("rel".to_string(), AttrValue::Str(rel.clone()));
                    }
                    if let Some(class) = attributes.get("class") {
                        attrs.insert("class".to_string(), AttrValue::Str(class.clone()));
                    }
                    state.open_mark(MarkupMarkType::Link, Some(attrs));
                }
            }

            "html" | "head" | "body" | "thead" | "tbody" => {}

            _ => {}
        }
    }

    fn handle_close_tag(
        &self,
        state: &mut HtmlParseState,
        tag: &str,
        attributes: &HashMap<String, String>,
    ) {
        match tag {
            "p" => {
                if state.top().map(|n| n.node_type.clone()) == Some(MarkupNodeType::Paragraph) {
                    state.close_node(MarkupNodeType::Paragraph);
                }
            }

            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                state.close_node(MarkupNodeType::Heading);
            }

            "blockquote" => state.close_node(MarkupNodeType::Blockquote),
            "pre" => state.close_node(MarkupNodeType::CodeBlock),
            "ol" => state.close_node(MarkupNodeType::OrderedList),
            "ul" => {
                let top_type = state.top().map(|n| n.node_type.clone());
                match top_type {
                    Some(MarkupNodeType::TodoList) => state.close_node(MarkupNodeType::TodoList),
                    Some(MarkupNodeType::TaskList) => state.close_node(MarkupNodeType::TaskList),
                    _ => state.close_node(MarkupNodeType::BulletList),
                }
            }
            "li" => {
                let top_type = state.top().map(|n| n.node_type.clone());
                match top_type {
                    Some(MarkupNodeType::TodoItem) => state.close_node(MarkupNodeType::TodoItem),
                    Some(MarkupNodeType::TaskItem) => state.close_node(MarkupNodeType::TaskItem),
                    _ => state.close_node(MarkupNodeType::ListItem),
                }
            }

            "table" => state.close_node(MarkupNodeType::Table),
            "tr" => state.close_node(MarkupNodeType::TableRow),
            "td" => {
                if state.top().map(|n| n.node_type.clone()) == Some(MarkupNodeType::Paragraph) {
                    state.close_node(MarkupNodeType::Paragraph);
                }
                state.close_node(MarkupNodeType::TableCell);
            }
            "th" => {
                if state.top().map(|n| n.node_type.clone()) == Some(MarkupNodeType::Paragraph) {
                    state.close_node(MarkupNodeType::Paragraph);
                }
                state.close_node(MarkupNodeType::TableHeader);
            }

            "sub" => state.close_node(MarkupNodeType::SubLink),

            "b" | "strong" => state.close_mark(MarkupMarkType::Bold),
            "em" | "i" => state.close_mark(MarkupMarkType::Italic),
            "s" => state.close_mark(MarkupMarkType::Strike),
            "u" => state.close_mark(MarkupMarkType::Underline),

            "span" => {
                let top_type = state.top().map(|n| n.node_type.clone());
                if top_type == Some(MarkupNodeType::Reference) {
                    if let Some(top) = state.stack.last_mut() {
                        top.content.clear();
                    }
                    state.close_node(MarkupNodeType::Reference);
                } else {
                    state.close_mark(MarkupMarkType::TextColor);
                    state.close_mark(MarkupMarkType::TextStyle);
                }
            }

            "code" => {
                let top_type = state.top().map(|n| n.node_type.clone());
                if top_type != Some(MarkupNodeType::CodeBlock) {
                    state.close_mark(MarkupMarkType::Code);
                }
            }

            "a" => {
                let top_type = state.top().map(|n| n.node_type.clone());
                let data_type = attributes.get("data-type").map(|s| s.as_str());

                if top_type == Some(MarkupNodeType::Embed) {
                    if let Some(top) = state.stack.last_mut() {
                        top.content.clear();
                    }
                    state.close_node(MarkupNodeType::Embed);

                    if data_type == Some("embed") {
                        let parent_type = state.top().map(|n| n.node_type.clone());
                        if parent_type == Some(MarkupNodeType::Paragraph) {
                            state.close_node(MarkupNodeType::Paragraph);
                        }
                    }
                } else {
                    state.close_mark(MarkupMarkType::Link);
                }
            }

            "html" | "head" | "body" | "thead" | "tbody" | "hr" | "br" | "img" => {}

            _ => {}
        }
    }

    fn extract_style_attrs(&self, attrs: &mut HashMap<String, String>, style: &str) {
        for part in style.split(';') {
            if let Some((key, value)) = part.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                if key == "text-align" && !value.is_empty() {
                    attrs.insert("textAlign".to_string(), value.to_string());
                }
            }
        }
    }

    fn parse_style_to_attrs(&self, style: &str) -> HashMap<String, AttrValue> {
        let mut attrs = HashMap::new();
        for part in style.split(';') {
            if let Some((key, value)) = part.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                if !key.is_empty() && !value.is_empty() {
                    let camel_key = self.to_camel_case(key);
                    attrs.insert(camel_key, AttrValue::Str(value.to_string()));
                }
            }
        }
        attrs
    }

    fn to_camel_case(&self, s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;
        for c in s.chars() {
            if c == '-' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }
        result
    }
}

impl HtmlParser {
    fn add_str_attr(
        &self,
        attrs: &mut HashMap<String, AttrValue>,
        src: &HashMap<String, String>,
        key: &str,
        dest_key: &str,
    ) {
        if let Some(val) = src.get(key) {
            attrs.insert(dest_key.to_string(), AttrValue::Str(val.clone()));
        }
    }

    fn add_num_attr(
        &self,
        attrs: &mut HashMap<String, AttrValue>,
        src: &HashMap<String, String>,
        key: &str,
        dest_key: &str,
    ) {
        if let Some(val) = src.get(key) {
            if let Ok(n) = val.parse::<i32>() {
                attrs.insert(dest_key.to_string(), AttrValue::Num(n));
            }
        }
    }

    fn extract_table_cell_attrs(
        &self,
        attributes: &HashMap<String, String>,
    ) -> HashMap<String, AttrValue> {
        let mut attrs = HashMap::new();
        self.add_num_attr(&mut attrs, attributes, "colspan", "colspan");
        self.add_num_attr(&mut attrs, attributes, "rowspan", "rowspan");
        self.add_num_attr(&mut attrs, attributes, "colwidth", "colwidth");
        attrs
    }
}
