use crate::text::{
    AttrValue, MarkupMark, MarkupMarkType, MarkupNode, MarkupNodeType, html_to_markup,
};
use markdown::mdast::List;
use markdown::unist::Position;
use markdown::{ParseOptions, mdast::Node, to_mdast};
use std::collections::HashMap;

pub fn markdown_to_markup(input: &str) -> MarkupNode {
    let options = ParseOptions::gfm();
    let ast = to_mdast(input, &options).expect("Failed to parse markdown");

    let state = ParserState::new(input);
    let nodes = state.convert_nodes(&ast);
    assert_eq!(nodes.len(), 1, "Root should produce exactly one node");
    nodes.into_iter().next().unwrap()
}

struct ParserState<'a> {
    source: &'a str,
}

impl<'a> ParserState<'a> {
    fn new(source: &'a str) -> Self {
        Self { source }
    }

    fn node(node_type: MarkupNodeType, content: Vec<MarkupNode>) -> MarkupNode {
        MarkupNode {
            node_type,
            content,
            marks: vec![],
            attrs: HashMap::new(),
            text: String::new(),
        }
    }

    fn node_with_attrs(
        node_type: MarkupNodeType,
        content: Vec<MarkupNode>,
        attrs: HashMap<String, AttrValue>,
    ) -> MarkupNode {
        MarkupNode {
            node_type,
            content,
            marks: vec![],
            attrs,
            text: String::new(),
        }
    }

    fn text_node(text: String) -> MarkupNode {
        MarkupNode {
            node_type: MarkupNodeType::Text,
            content: vec![],
            marks: vec![],
            attrs: HashMap::new(),
            text,
        }
    }

    fn extract_image_alt(&self, start_offset: usize, end_offset: usize) -> String {
        let slice = &self.source[start_offset..end_offset];

        if let Some(alt_start) = slice.find("![") {
            if let Some(alt_end) = slice[alt_start + 2..].find(']') {
                let alt_text = &slice[alt_start + 2..alt_start + 2 + alt_end];
                let result = alt_text.replace("\\\n", "\n");
                return result;
            }
        }

        String::new()
    }

    fn create_list_node(&self, items: Vec<MarkupNode>, is_todo: bool) -> MarkupNode {
        let mut attrs = HashMap::new();
        attrs.insert("bullet".to_string(), AttrValue::Str("-".to_string()));
        let node_type = if is_todo {
            MarkupNodeType::TodoList
        } else {
            MarkupNodeType::BulletList
        };
        Self::node_with_attrs(node_type, items, attrs)
    }

    fn split_todo_and_bullet_lists(&self, items: &[Node]) -> Vec<MarkupNode> {
        let mut result_lists = Vec::new();
        let mut current_group: Vec<MarkupNode> = Vec::new();
        let mut current_is_todo: Option<bool> = None;

        for item in items {
            if let Node::ListItem(list_item) = item {
                let is_todo = list_item.checked.is_some();

                if current_is_todo.is_none() || current_is_todo != Some(is_todo) {
                    if !current_group.is_empty() {
                        let list_node =
                            self.create_list_node(current_group, current_is_todo.unwrap());
                        result_lists.push(list_node);
                        current_group = Vec::new();
                    }
                    current_is_todo = Some(is_todo);
                }

                let nodes = self.convert_nodes(item);
                assert_eq!(nodes.len(), 1, "ListItem should produce exactly one node");
                current_group.push(nodes.into_iter().next().unwrap());
            }
        }

        if !current_group.is_empty() {
            let list_node = self.create_list_node(current_group, current_is_todo.unwrap());
            result_lists.push(list_node);
        }

        result_lists
    }

    fn process_paragraph_children(&self, children: &[Node]) -> Vec<MarkupNode> {
        let mut content: Vec<MarkupNode> = Vec::new();
        let mut i = 0;

        while i < children.len() {
            if let Node::Html(html_open) = &children[i] {
                let tag_content = html_open.value.trim();

                if tag_content == "<sub>" || tag_content.starts_with("<span") {
                    let closing_tag = if tag_content == "<sub>" {
                        "</sub>"
                    } else {
                        "</span>"
                    };
                    let (combined_html, next_i) = self.merge_html_tag(&children[i..], closing_tag);
                    self.add_parsed_html(&combined_html, &mut content);
                    i += next_i;
                    continue;
                }

                if tag_content.starts_with('<')
                    && !tag_content.starts_with("</")
                    && !tag_content.ends_with("/>")
                {
                    let mut combined_html = html_open.value.clone();
                    let mut j = 1;

                    while i + j < children.len() {
                        combined_html.push_str(&self.node_to_string(&children[i + j]));
                        if let Node::Html(html_node) = &children[i + j] {
                            j += 1;
                            if html_node.value.trim().starts_with("</") {
                                break;
                            }
                        } else {
                            j += 1;
                        }
                    }

                    self.add_parsed_html(&combined_html, &mut content);
                    i += j;
                    continue;
                }
            }

            content.extend(self.convert_nodes(&children[i]));
            i += 1;
        }

        content
    }

    fn merge_html_tag(&self, nodes: &[Node], closing_tag: &str) -> (String, usize) {
        let mut combined_html = String::new();
        let mut j = 0;

        while j < nodes.len() {
            combined_html.push_str(&self.node_to_string(&nodes[j]));

            if let Node::Html(html_node) = &nodes[j] {
                if html_node.value.trim() == closing_tag {
                    return (combined_html, j + 1);
                }
            }
            j += 1;
        }

        (combined_html, j)
    }

    fn add_parsed_html(&self, html: &str, content: &mut Vec<MarkupNode>) {
        let markup = html_to_markup(html);
        match markup.node_type {
            MarkupNodeType::Doc => {
                for child in markup.content {
                    if child.node_type == MarkupNodeType::Paragraph {
                        content.extend(child.content);
                    } else {
                        content.push(child);
                    }
                }
            }
            MarkupNodeType::Paragraph => content.extend(markup.content),
            _ => content.push(markup),
        }
    }

    fn node_to_string(&self, node: &Node) -> String {
        match node {
            Node::Text(text) => text.value.clone(),
            Node::Html(html) => html.value.clone(),
            Node::Link(link) => {
                let mut result = format!("<a href=\"{}\"", link.url);
                if let Some(title) = &link.title {
                    result.push_str(&format!(" title=\"{}\"", title));
                }
                result.push('>');
                for child in &link.children {
                    result.push_str(&self.node_to_string(child));
                }
                result.push_str("</a>");
                result
            }
            _ => String::new(),
        }
    }

    fn convert_list_node(&self, list: &List) -> Vec<MarkupNode> {
        if list.ordered {
            let content: Vec<MarkupNode> = list
                .children
                .iter()
                .flat_map(|n| self.convert_nodes(n))
                .collect();
            let mut attrs = HashMap::new();

            if let Some(start) = list.start {
                if start != 1 {
                    attrs.insert("order".to_string(), AttrValue::Num(start as i32));
                }
            }
            vec![Self::node_with_attrs(
                MarkupNodeType::OrderedList,
                content,
                attrs,
            )]
        } else {
            let (has_todo, has_regular) =
                list.children
                    .iter()
                    .fold((false, false), |(todo, reg), item| {
                        if let Node::ListItem(li) = item {
                            (todo || li.checked.is_some(), reg || li.checked.is_none())
                        } else {
                            (todo, reg)
                        }
                    });

            if has_todo && has_regular {
                return self.split_todo_and_bullet_lists(&list.children);
            }

            let content: Vec<MarkupNode> = list
                .children
                .iter()
                .flat_map(|n| self.convert_nodes(n))
                .collect();

            let bullet = self.extract_list_bullet_marker(&list.position);
            let mut attrs = HashMap::new();
            attrs.insert("bullet".to_string(), AttrValue::Str(bullet));
            let node_type = if has_todo && !has_regular {
                MarkupNodeType::TodoList
            } else {
                MarkupNodeType::BulletList
            };
            vec![Self::node_with_attrs(node_type, content, attrs)]
        }
    }

    fn convert_list_item_node(&self, item: &markdown::mdast::ListItem) -> Vec<MarkupNode> {
        let content: Vec<MarkupNode> = item
            .children
            .iter()
            .flat_map(|n| self.convert_nodes(n))
            .collect();

        if let Some(checked) = item.checked {
            let mut attrs = HashMap::new();
            attrs.insert("checked".to_string(), AttrValue::Bool(checked));
            vec![Self::node_with_attrs(
                MarkupNodeType::TodoItem,
                content,
                attrs,
            )]
        } else {
            vec![Self::node(MarkupNodeType::ListItem, content)]
        }
    }

    fn convert_nodes(&self, node: &Node) -> Vec<MarkupNode> {
        match node {
            Node::Root(root) => {
                let mut content = Vec::new();
                for child in &root.children {
                    content.extend(self.convert_nodes(child));
                }
                vec![Self::node(MarkupNodeType::Doc, content)]
            }
            Node::Paragraph(paragraph) => {
                let content = self.process_paragraph_children(&paragraph.children);
                vec![Self::node(MarkupNodeType::Paragraph, content)]
            }
            Node::Text(text_node) => vec![Self::text_node(text_node.value.clone())],
            Node::Heading(heading) => {
                let content: Vec<MarkupNode> = heading
                    .children
                    .iter()
                    .flat_map(|n| self.convert_nodes(n))
                    .collect();
                let mut attrs = HashMap::new();
                attrs.insert("level".to_string(), AttrValue::Num(heading.depth as i32));
                attrs.insert("marker".to_string(), AttrValue::Str("#".to_string()));
                vec![Self::node_with_attrs(
                    MarkupNodeType::Heading,
                    content,
                    attrs,
                )]
            }
            Node::List(list) => self.convert_list_node(list),
            Node::ListItem(item) => self.convert_list_item_node(item),
            Node::Image(image) => {
                let mut attrs = HashMap::new();
                attrs.insert("src".to_string(), AttrValue::Str(image.url.clone()));

                let alt = if let Some(pos) = &image.position {
                    self.extract_image_alt(pos.start.offset, pos.end.offset)
                } else {
                    image.alt.clone()
                };
                attrs.insert("alt".to_string(), AttrValue::Str(alt));
                if let Some(title) = &image.title {
                    attrs.insert("title".to_string(), AttrValue::Str(title.clone()));
                }

                vec![Self::node_with_attrs(MarkupNodeType::Image, vec![], attrs)]
            }
            Node::Code(code) => {
                let lang = code.lang.as_deref().unwrap_or("");
                let mut attrs = HashMap::new();
                attrs.insert("language".to_string(), AttrValue::Str(lang.to_string()));

                let node_type = if lang == "mermaid" {
                    MarkupNodeType::Mermaid
                } else {
                    MarkupNodeType::CodeBlock
                };
                let content = vec![Self::text_node(code.value.clone())];
                vec![Self::node_with_attrs(node_type, content, attrs)]
            }
            Node::Html(html) => {
                let trimmed = html.value.trim();
                if trimmed.starts_with("<!--") && trimmed.ends_with("-->") {
                    let text_content = trimmed
                        .strip_prefix("<!--")
                        .and_then(|s| s.strip_suffix("-->"))
                        .unwrap_or("")
                        .to_string();

                    vec![Self::node(
                        MarkupNodeType::Comment,
                        vec![Self::text_node(text_content)],
                    )]
                } else {
                    let markup = html_to_markup(&html.value);
                    if markup.node_type == MarkupNodeType::Doc {
                        markup.content
                    } else {
                        vec![markup]
                    }
                }
            }
            Node::Emphasis(emphasis) => {
                let content: Vec<MarkupNode> = emphasis
                    .children
                    .iter()
                    .flat_map(|n| self.convert_nodes(n))
                    .collect();

                let marker = self.extract_emphasis_marker(&emphasis.position, "_", "*");
                self.apply_mark_with_marker(content, MarkupMarkType::Italic, marker)
            }
            Node::Strong(strong) => {
                let content: Vec<MarkupNode> = strong
                    .children
                    .iter()
                    .flat_map(|n| self.convert_nodes(n))
                    .collect();

                let marker = self.extract_emphasis_marker(&strong.position, "__", "**");
                self.apply_mark_with_marker(content, MarkupMarkType::Bold, marker)
            }
            Node::Link(link) => {
                let content: Vec<MarkupNode> = link
                    .children
                    .iter()
                    .flat_map(|n| self.convert_nodes(n))
                    .collect();

                let mut attrs = HashMap::new();
                attrs.insert("href".to_string(), AttrValue::Str(link.url.clone()));
                if let Some(title) = &link.title {
                    attrs.insert("title".to_string(), AttrValue::Str(title.clone()));
                }

                self.apply_mark_to_nodes(content, MarkupMarkType::Link, attrs)
            }
            _ => vec![Self::text_node(String::new())],
        }
    }

    fn extract_emphasis_marker(
        &self,
        position: &Option<Position>,
        alt_marker: &str,
        default_marker: &str,
    ) -> String {
        position
            .as_ref()
            .filter(|pos| {
                let start = pos.start.offset;
                start + alt_marker.len() <= self.source.len()
                    && &self.source[start..start + alt_marker.len()] == alt_marker
            })
            .map(|_| alt_marker.to_string())
            .unwrap_or_else(|| default_marker.to_string())
    }

    fn extract_list_bullet_marker(&self, position: &Option<Position>) -> String {
        if let Some(pos) = position {
            let start = pos.start.offset;
            if start < self.source.len() {
                let remaining = &self.source[start..];
                for ch in remaining.chars() {
                    if ch == '*' || ch == '-' || ch == '+' {
                        return ch.to_string();
                    }
                    if !ch.is_whitespace() && ch != '>' {
                        break;
                    }
                }
            }
        }
        "-".to_string()
    }

    fn apply_mark_with_marker(
        &self,
        nodes: Vec<MarkupNode>,
        mark_type: MarkupMarkType,
        marker: String,
    ) -> Vec<MarkupNode> {
        let mut attrs = HashMap::new();
        attrs.insert("marker".to_string(), AttrValue::Str(marker));
        self.apply_mark_to_nodes(nodes, mark_type, attrs)
    }

    fn apply_mark_to_nodes(
        &self,
        nodes: Vec<MarkupNode>,
        mark_type: MarkupMarkType,
        attrs: HashMap<String, AttrValue>,
    ) -> Vec<MarkupNode> {
        nodes
            .into_iter()
            .map(|mut node| {
                node.marks.push(MarkupMark {
                    mark_type: mark_type.clone(),
                    attrs: attrs.clone(),
                });
                node
            })
            .collect()
    }
}
