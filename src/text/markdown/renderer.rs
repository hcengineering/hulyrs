use std::collections::HashMap;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use regex::Regex;

use crate::text::{
    AttrValue, MarkupMark, MarkupMarkType, MarkupNode, MarkupNodeType, get_bool_attr, get_num_attr,
    get_str_attr, markup_to_html,
};

const URI_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'[')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

pub fn markup_to_markdown(markup: &MarkupNode, image_url: String, ref_url: String) -> String {
    let mut renderer = MarkdownRenderer::new(image_url, ref_url);
    renderer.render_content(markup);
    renderer.out
}

fn is_in_set(mark: &MarkupMark, marks: &[MarkupMark]) -> bool {
    marks.iter().any(|m| mark == m)
}

fn is_plain_url(link: &MarkupMark, parent: &MarkupNode, index: usize) -> bool {
    let href = get_str_attr(&link.attrs, "href");
    let protocol_regex = Regex::new(r"^\w+:").unwrap();
    if link.attrs.contains_key("title") || !protocol_regex.is_match(&href) {
        return false;
    }

    let Some(content) = parent.content.get(index) else {
        return false;
    };

    if content.node_type != MarkupNodeType::Text || content.text != href {
        return false;
    }

    let marks = &content.marks;
    if marks.is_empty() || marks.last() != Some(link) {
        return false;
    }

    parent
        .content
        .get(index + 1)
        .map_or(true, |next| !is_in_set(link, &next.marks))
}

struct InlineState<'a> {
    active: Vec<MarkupMark>,
    trailing: String,
    parent: &'a MarkupNode,
    node: Option<&'a MarkupNode>,
    marks: Vec<MarkupMark>,
    modified_text: Option<String>,
}

fn format_todo_item(
    _index: usize,
    child_attrs: HashMap<String, AttrValue>,
    parent_attrs: HashMap<String, AttrValue>,
) -> String {
    let bullet = get_str_attr(&parent_attrs, "bullet");
    let bullet = if bullet.is_empty() { "*" } else { &bullet };

    let checked = get_bool_attr(&child_attrs, "checked").unwrap_or(false);
    let checkbox = if checked { "x" } else { " " };

    let todoid = get_str_attr(&child_attrs, "todoid");
    let userid = get_str_attr(&child_attrs, "userid");
    let meta = if !todoid.is_empty() && !userid.is_empty() {
        format!("<!-- todoid={},userid={} -->", todoid, userid)
    } else {
        String::new()
    };

    format!("{} [{}] {}", bullet, checkbox, meta)
}

struct MarkdownRenderer<'a> {
    image_url: String,
    ref_url: String,

    out: String,
    delim: String,
    closed: bool,
    close_node: Option<&'a MarkupNode>,
    in_tight_list: bool,
    render_a_href: bool,
    in_autolink: bool,
}

impl<'a> MarkdownRenderer<'a> {
    fn new(image_url: String, ref_url: String) -> Self {
        Self {
            image_url,
            ref_url,
            out: String::new(),
            delim: String::new(),
            closed: false,
            close_node: None,
            in_tight_list: false,
            render_a_href: false,
            in_autolink: false,
        }
    }

    fn render_content(&mut self, parent: &'a MarkupNode) {
        for (i, node) in parent.content.iter().enumerate() {
            self.render_node_with_parent(node, Some(parent), i);
        }
    }

    fn render_node(&mut self, node: &'a MarkupNode) {
        self.render_node_with_parent(node, None, 0);
    }

    fn render_node_with_parent(
        &mut self,
        node: &'a MarkupNode,
        parent: Option<&'a MarkupNode>,
        index: usize,
    ) {
        match node.node_type {
            MarkupNodeType::Doc => self.render_content(node),
            MarkupNodeType::Paragraph => {
                self.render_inline(node);
                self.close_block(node);
            }
            MarkupNodeType::Blockquote => {
                self.wrap_block("> ", None, node, |renderer| {
                    renderer.render_content(node);
                });
            }
            MarkupNodeType::HorizontalRule => {
                let markup = get_str_attr(&node.attrs, "markup");
                let markup = if markup.is_empty() { "---" } else { &markup };
                self.write(markup);
                self.close_block(node);
            }
            MarkupNodeType::Heading => {
                self.heading(node);
            }
            MarkupNodeType::CodeBlock => {
                let language = get_str_attr(&node.attrs, "language");
                self.write(&format!("```{}\n", language));
                self.render_inline(node);
                self.ensure_new_line();
                self.write("```");
                self.close_block(node);
            }
            MarkupNodeType::Mermaid => {
                self.write("```mermaid\n");
                self.render_inline(node);
                self.ensure_new_line();
                self.write("```");
                self.close_block(node);
            }
            MarkupNodeType::Text => self.text(&node.text, false),
            MarkupNodeType::Image => self.image(node),
            MarkupNodeType::File => todo!(),
            MarkupNodeType::Reference => {
                let label = get_str_attr(&node.attrs, "label");

                let mut url = self.ref_url.clone();
                if !url.contains('?') {
                    url.push('?');
                } else {
                    url.push('&');
                }

                let mut query_params = HashMap::new();
                let obj_class = get_str_attr(&node.attrs, "objectclass");
                if !obj_class.is_empty() {
                    query_params.insert("_class".to_string(), AttrValue::Str(obj_class));
                }
                let id = get_str_attr(&node.attrs, "id");
                if !id.is_empty() {
                    query_params.insert("_id".to_string(), AttrValue::Str(id));
                }
                if !label.is_empty() {
                    query_params.insert("label".to_string(), AttrValue::Str(label.clone()));
                }

                url.push_str(&make_query(&query_params));

                let title = get_str_attr(&node.attrs, "title");
                let title_part = if !title.is_empty() {
                    format!(" {}", self.quote(&title))
                } else {
                    String::new()
                };

                self.write(&format!(
                    "[{}]({}{})",
                    self.esc(&label, false),
                    url,
                    title_part
                ));
            }
            MarkupNodeType::Emoji => {
                let emoji = get_str_attr(&node.attrs, "emoji");
                self.text(&emoji, false);
            }
            MarkupNodeType::HardBreak => {
                if let Some(parent_node) = parent {
                    let content = &parent_node.content;
                    for i in (index + 1)..content.len() {
                        if content[i].node_type != node.node_type {
                            self.write("\\\n");
                            return;
                        }
                    }
                }
            }
            MarkupNodeType::OrderedList => {
                let start = get_num_attr(&node.attrs, "order").unwrap_or(1);

                let max_w = (start + node.content.len() as i32 - 1).to_string().len();
                let space = " ".repeat(max_w + 2);

                let first_delim = move |i: usize, _, _| {
                    let n_str = (start + i as i32).to_string();
                    let padding = " ".repeat(max_w - n_str.len());
                    format!("{}{}. ", padding, n_str)
                };

                self.render_list(node, &space, first_delim);
            }
            MarkupNodeType::BulletList => {
                let first_delim = |_, _, _| {
                    let bullet = get_str_attr(&node.attrs, "bullet");
                    if bullet.is_empty() {
                        "* ".to_string()
                    } else {
                        format!("{} ", bullet)
                    }
                };
                self.render_list(node, "  ", first_delim);
            }
            MarkupNodeType::ListItem => self.render_content(node),
            MarkupNodeType::TaskList => {
                let first_delim = |_, _, _| "* [ ] ".to_string();
                self.render_list(node, "  ", first_delim);
            }
            MarkupNodeType::TaskItem => self.render_content(node),
            MarkupNodeType::TodoList => {
                self.render_list(node, "  ", format_todo_item);
            }
            MarkupNodeType::TodoItem => self.render_content(node),
            MarkupNodeType::SubLink => {
                self.write("<sub>");
                self.render_a_href = true;
                self.render_inline(node);
                self.render_a_href = false;
                self.write("</sub>");
            }
            MarkupNodeType::Table => {
                self.write(&markup_to_html(node));
                self.close_block(node);
            }
            MarkupNodeType::TableRow => {}
            MarkupNodeType::TableCell => {}
            MarkupNodeType::TableHeader => {}
            MarkupNodeType::Comment => {
                self.write("<!--");
                self.render_inline(node);
                self.write("-->");
                self.close_block(node);
            }
            MarkupNodeType::Markdown => {
                self.render_inline(node);
                self.close_block(node);
            }
            MarkupNodeType::Embed => {
                let embed_url = get_str_attr(&node.attrs, "src");
                self.write(&format!(
                    "<a href=\"{}\" data-type=\"embed\">{}</a>",
                    self.encode_uri(&embed_url),
                    self.html_esc(&embed_url).replace('/', "&#x2F;")
                ));
            }
        }
    }

    fn render_inline(&mut self, parent: &'a MarkupNode) {
        let mut state = InlineState {
            active: Vec::new(),
            trailing: String::new(),
            parent,
            node: None,
            marks: Vec::new(),
            modified_text: None,
        };
        for (i, node) in parent.content.iter().enumerate() {
            state.node = Some(node);
            self.render_node_inline(&mut state, i);
        }

        state.node = None;
        self.render_node_inline(&mut state, 0)
    }

    fn render_node_inline(&mut self, state: &mut InlineState<'a>, index: usize) {
        state.marks = state
            .node
            .as_ref()
            .and_then(|n| Some(n.marks.clone()))
            .unwrap_or_default();

        self.update_hardbreak_marks(state, index);

        let leading = self.adjust_leading(state);

        let len = state.marks.len();

        self.reorder_mixable_marks(state, len);

        self.check_close_marks(state, len, index);

        if !leading.is_empty() {
            self.text(&leading, true);
        }

        self.check_open_marks(state, len, index);
    }

    fn check_close_marks(&mut self, state: &mut InlineState<'a>, len: usize, index: usize) {
        let mut keep = 0;
        while keep < state.active.len().min(len) && state.marks[keep] == state.active[keep] {
            keep += 1;
        }

        while keep < state.active.len() {
            if let Some(mark) = state.active.pop() {
                let mark_str = self.mark_string(&mark, false, state.parent, index);
                self.write(&mark_str);
            }
        }
    }

    fn check_open_marks(&mut self, state: &mut InlineState<'a>, len: usize, index: usize) {
        if state.node.is_some() {
            self.update_active_marks(state, len, index);

            if let Some(node) = &state.node {
                if let Some(ref modified_text) = state.modified_text {
                    self.text(modified_text, true);
                } else {
                    self.render_node_with_parent(node, Some(state.parent), index);
                }
            }
            state.modified_text = None;
        }
    }

    fn update_active_marks(&mut self, state: &mut InlineState<'a>, len: usize, index: usize) {
        while state.active.len() < len {
            let mark = state.marks[state.active.len()].clone();
            state.active.push(mark.clone());
            let mark_str = self.mark_string(&mark, true, state.parent, index);
            self.text(&mark_str, false);
        }
    }

    fn render_list<F>(&mut self, node: &'a MarkupNode, delim: &str, first_delim: F)
    where
        F: Fn(usize, HashMap<String, AttrValue>, HashMap<String, AttrValue>) -> String,
    {
        self.flush_list_close(node);

        let is_tight = get_bool_attr(&node.attrs, "tight").unwrap_or(false);

        for (i, child) in node.content.iter().enumerate() {
            self.render_list_item(node, child, i, is_tight, delim, &first_delim);
        }
    }

    fn render_list_item<F>(
        &mut self,
        node: &'a MarkupNode,
        child: &'a MarkupNode,
        index: usize,
        is_tight: bool,
        delim: &str,
        first_delim: &F,
    ) where
        F: Fn(usize, HashMap<String, AttrValue>, HashMap<String, AttrValue>) -> String,
    {
        if (index > 0) && is_tight {
            self.flush_close(1);
        }

        let first_delim = first_delim(index, child.attrs.clone(), node.attrs.clone());
        self.wrap_block(delim, Some(&first_delim), node, |renderer| {
            renderer.render_node(child);
        });
    }

    fn wrap_block<F: FnOnce(&mut Self)>(
        &mut self,
        delim: &str,
        first_delim: Option<&str>,
        node: &'a MarkupNode,
        render_fn: F,
    ) {
        let old = self.delim.clone();
        if let Some(fd) = first_delim {
            self.write(fd);
        } else {
            self.write(delim);
        }

        self.delim.push_str(delim);
        render_fn(self);
        self.delim = old;
        self.close_block(node);
    }

    fn at_blank(&self) -> bool {
        self.out.is_empty() || self.out.ends_with('\n')
    }

    fn ensure_new_line(&mut self) {
        if !self.at_blank() {
            self.out.push('\n');
        }
    }

    fn flush_close(&mut self, size: usize) {
        if self.closed {
            if !self.at_blank() {
                self.out.push('\n');
            }
            if size > 1 {
                self.add_delim(size);
            }
            self.closed = false;
        }
    }

    fn flush_list_close(&mut self, node: &MarkupNode) {
        if let Some(close_node) = &self.close_node {
            if self.closed && close_node.node_type == node.node_type {
                self.flush_close(3);
            }
        } else if self.in_tight_list {
            self.flush_close(1);
        }
    }

    fn add_delim(&mut self, size: usize) {
        let delim_min = self.delim.trim_end();
        for _ in 1..size {
            self.out.push_str(delim_min);
            self.out.push('\n');
        }
    }

    fn write(&mut self, content: &str) {
        self.flush_close(2);
        if !self.delim.is_empty() && self.at_blank() {
            self.out.push_str(&self.delim);
        }
        if !content.is_empty() {
            self.out.push_str(content);
        }
    }

    fn close_block(&mut self, node: &'a MarkupNode) {
        self.close_node = Some(node);
        self.closed = true;
    }

    fn mark_string(
        &mut self,
        mark: &MarkupMark,
        open: bool,
        parent: &MarkupNode,
        index: usize,
    ) -> String {
        let marker = get_str_attr(&mark.attrs, "marker");
        if !marker.is_empty() {
            return marker;
        }

        match mark.mark_type {
            MarkupMarkType::Bold => "**".to_string(),
            MarkupMarkType::Italic => "*".to_string(),
            MarkupMarkType::Code => "`".to_string(),
            MarkupMarkType::Strike => "~~".to_string(),
            MarkupMarkType::Underline => if open { "<ins>" } else { "</ins>" }.to_string(),
            MarkupMarkType::Link => {
                if open {
                    if self.render_a_href {
                        let href = get_str_attr(&mark.attrs, "href");
                        return format!("<a href=\"{}\">", self.encode_uri(&href));
                    }
                    self.in_autolink = is_plain_url(mark, parent, index);
                    if self.in_autolink { "<" } else { "[" }.to_string()
                } else {
                    if self.render_a_href {
                        return "</a>".to_string();
                    }

                    let was_autolink = self.in_autolink;
                    self.in_autolink = false;

                    if was_autolink {
                        return ">".to_string();
                    }

                    let href = get_str_attr(&mark.attrs, "href");
                    let url = href
                        .replace('\\', "\\\\")
                        .replace('(', "\\(")
                        .replace(')', "\\)")
                        .replace('"', "\\\"")
                        .replace('<', "\\<")
                        .replace('>', "\\>");

                    let url_part = if url.contains(' ') {
                        format!("<{}>", url)
                    } else {
                        url
                    };

                    let title = get_str_attr(&mark.attrs, "title");
                    let title_part = if !title.is_empty() {
                        format!(" \"{}\"", title.replace('"', "\\\""))
                    } else {
                        String::new()
                    };
                    format!("]({}{})", url_part, title_part)
                }
            }
            MarkupMarkType::TextColor => {
                let color = get_str_attr(&mark.attrs, "color");
                if color.is_empty() {
                    String::new()
                } else if open {
                    format!("<span style=\"color: {}\" data-color=\"{}\">", color, color)
                } else {
                    "</span>".to_string()
                }
            }
            MarkupMarkType::TextStyle => {
                if mark.attrs.is_empty() {
                    return String::new();
                }

                if open {
                    let mut sorted_keys: Vec<_> = mark.attrs.keys().collect();
                    sorted_keys.sort();

                    let style_attrs: Vec<String> = sorted_keys
                        .iter()
                        .map(|key| {
                            let kebab_key = key
                                .chars()
                                .flat_map(|c| {
                                    if c.is_uppercase() {
                                        vec!['-', c.to_lowercase().next().unwrap()]
                                    } else {
                                        vec![c]
                                    }
                                })
                                .collect::<String>();

                            let value_str = match &mark.attrs[*key] {
                                AttrValue::Str(s) => s.clone(),
                                AttrValue::Num(n) => n.to_string(),
                                _ => String::new(),
                            };

                            format!("{}: {}", kebab_key, value_str)
                        })
                        .collect();

                    format!("<span style=\"{}\">", style_attrs.join("; "))
                } else {
                    "</span>".to_string()
                }
            }
        }
    }

    fn is_text(&self, node: Option<&MarkupNode>) -> bool {
        matches!(node, Some(n) if n.node_type == MarkupNodeType::Text && !n.text.is_empty())
    }

    fn is_hardbreak_text(&self, next: Option<&MarkupNode>) -> bool {
        match next {
            None => false,
            Some(node) => {
                node.node_type != MarkupNodeType::HardBreak
                    && (node.node_type != MarkupNodeType::Text
                        || !node.text.is_empty() && {
                            let regex = regex::Regex::new(r"\S").unwrap();
                            regex.is_match(&node.text)
                        })
            }
        }
    }

    fn is_mixable(&self, mark_type: &MarkupMarkType) -> bool {
        matches!(
            mark_type,
            MarkupMarkType::Bold | MarkupMarkType::Italic | MarkupMarkType::Strike
        )
    }

    fn is_marks_has_expel_enclosing_whitespace(&self, state: &InlineState) -> bool {
        state.marks.iter().any(|mark| {
            matches!(
                mark.mark_type,
                MarkupMarkType::Bold
                    | MarkupMarkType::Italic
                    | MarkupMarkType::Strike
                    | MarkupMarkType::Underline
            )
        })
    }

    fn adjust_leading_text_node(
        &self,
        lead: &str,
        trail: &str,
        state: &mut InlineState<'a>,
        inner: &str,
        _node: &'a MarkupNode,
    ) {
        if !lead.is_empty() || !trail.is_empty() {
            if !inner.is_empty() {
                state.modified_text = Some(inner.to_string());
            } else {
                state.node = None;
                state.marks = state.active.clone();
                state.modified_text = None;
            }
        }
    }

    fn adjust_leading(&mut self, state: &mut InlineState<'a>) -> String {
        let mut leading = state.trailing.clone();
        state.trailing.clear();

        if self.is_text(state.node) && self.is_marks_has_expel_enclosing_whitespace(state) {
            if let Some(node) = state.node {
                let text_str = node.text.as_str();
                let lead_end = text_str.len() - text_str.trim_start().len();
                let trail_start = text_str.trim_end().len();

                let lead_match = &text_str[..lead_end];
                let inner_match = &text_str[lead_end..trail_start];
                let trail_match = &text_str[trail_start..];

                leading.push_str(lead_match);
                state.trailing = trail_match.to_string();
                self.adjust_leading_text_node(lead_match, trail_match, state, inner_match, node);
            }
        }
        leading
    }

    fn filter_hardbreak_marks(
        &self,
        marks: &[MarkupMark],
        index: usize,
        state: &InlineState<'a>,
    ) -> Vec<MarkupMark> {
        let next = state.parent.content.get(index + 1);
        if !self.is_hardbreak_text(next) {
            return Vec::new();
        }
        let empty_marks = Vec::new();
        let next_marks = next.and_then(|n| Some(&n.marks)).unwrap_or(&empty_marks);
        marks
            .iter()
            .filter(|m| is_in_set(m, next_marks))
            .cloned()
            .collect()
    }

    fn update_hardbreak_marks(&mut self, state: &mut InlineState<'a>, index: usize) {
        if let Some(node) = state.node {
            if node.node_type == MarkupNodeType::HardBreak {
                state.marks = self.filter_hardbreak_marks(&state.marks, index, state);
            }
        }
    }

    fn reorder_mixable_mark(
        &self,
        state: &mut InlineState<'a>,
        mark: &MarkupMark,
        i: usize,
        len: usize,
    ) {
        for j in 0..state.active.len() {
            let other = state.active[j].clone();
            if !self.is_mixable(&other.mark_type)
                || self.check_switch_marks(i, j, state, mark, &other, len)
            {
                break;
            }
        }
    }

    fn reorder_mixable_marks(&mut self, state: &mut InlineState<'a>, len: usize) {
        for i in 0..len {
            let mark = state.marks[i].clone();
            if !self.is_mixable(&mark.mark_type) {
                break;
            }
            self.reorder_mixable_mark(state, &mark, i, len);
        }
    }

    fn check_switch_marks(
        &self,
        i: usize,
        j: usize,
        state: &mut InlineState<'a>,
        mark: &MarkupMark,
        other: &MarkupMark,
        len: usize,
    ) -> bool {
        if mark != other || i == j {
            return false;
        }
        self.switch_marks(i, j, state, mark, len);
        true
    }

    fn switch_marks(
        &self,
        i: usize,
        j: usize,
        state: &mut InlineState<'a>,
        mark: &MarkupMark,
        len: usize,
    ) {
        let (start, end) = if i > j {
            (j, i + 1)
        } else if j > i {
            (i, j)
        } else {
            return;
        };

        let mut new_marks = Vec::new();
        new_marks.extend_from_slice(&state.marks[0..start]);
        if i > j {
            new_marks.push(mark.clone());
            new_marks.extend_from_slice(&state.marks[j..i]);
        } else {
            new_marks.extend_from_slice(&state.marks[i + 1..j]);
            new_marks.push(mark.clone());
        }
        new_marks.extend_from_slice(&state.marks[end..len]);
        state.marks = new_marks;
    }

    fn text(&mut self, text: &str, escape: bool) {
        let lines: Vec<&str> = text.split('\n').collect();

        for (i, line) in lines.iter().enumerate() {
            let start_of_line = self.at_blank() || self.closed;
            self.write("");
            if escape {
                self.out.push_str(&self.esc(line, start_of_line));
            } else {
                self.out.push_str(line);
            }
            if i != lines.len() - 1 {
                self.out.push('\n');
            }
        }
    }

    fn heading(&mut self, node: &'a MarkupNode) {
        let marker = get_str_attr(&node.attrs, "marker");
        let level = get_num_attr(&node.attrs, "level").unwrap_or(1);

        match (marker.as_str(), level) {
            ("=", 1) => {
                self.render_inline(node);
                self.ensure_new_line();
                self.write("===\n");
            }
            ("-", 2) => {
                self.render_inline(node);
                self.ensure_new_line();
                self.write("---\n");
            }
            _ => {
                let hashes = "#".repeat(level as usize);
                self.write(&format!("{} ", hashes));
                self.render_inline(node);
            }
        }

        self.close_block(node);
    }

    fn image(&mut self, node: &'a MarkupNode) {
        let attrs = &node.attrs;

        let alt = get_str_attr(attrs, "alt");
        let escaped_alt = self.esc(&alt, false);
        let file_id = get_str_attr(attrs, "file-id");
        let token = get_str_attr(attrs, "token");
        let title = get_str_attr(attrs, "title");
        let width = get_num_attr(attrs, "width");
        let height = get_num_attr(attrs, "height");

        let url = if !file_id.is_empty() {
            let mut url = if !token.is_empty() {
                format!("{}{}?file={}", self.image_url, file_id, file_id)
            } else {
                format!("{}/{}", self.image_url, file_id)
            };

            let sep = if url.contains('?') { "&" } else { "?" };
            if let Some(w) = width {
                url.push_str(&format!(
                    "{}width={}",
                    if url.ends_with('?') || url.ends_with('&') {
                        ""
                    } else {
                        sep
                    },
                    w
                ));
            }
            if let Some(h) = height {
                let sep = if url.contains('?') { "&" } else { "?" };
                url.push_str(&format!("{}height={}", sep, h));
            }
            if !token.is_empty() {
                url.push_str(&format!("&token={}", self.esc(&token, false)));
            }
            Some(url)
        } else {
            None
        };

        if url.is_some() {
            let src = url.as_ref().unwrap();
            let title_part = if !title.is_empty() {
                format!(" {}", self.quote(&title))
            } else {
                String::new()
            };

            self.write(&format!(
                "![{}]({}{})",
                escaped_alt,
                self.esc(src, false),
                title_part
            ));
        } else if width.is_some() || height.is_some() {
            let mut parts = vec!["<img".to_string()];

            if let Some(w) = width {
                parts.push(format!(" width=\"{}\"", w));
            }
            if let Some(h) = height {
                parts.push(format!(" height=\"{}\"", h));
            }

            let src = get_str_attr(attrs, "src");
            if !src.is_empty() {
                parts.push(format!(" src=\"{}\"", self.esc(&src, false)));
            }

            if !alt.is_empty() {
                parts.push(format!(" alt=\"{}\"", escaped_alt));
            }

            if !title.is_empty() {
                parts.push(format!(">{}</img>", self.quote(&title)));
            } else {
                parts.push(">".to_string());
            }

            self.write(&parts.join(""));
        } else {
            let src = get_str_attr(attrs, "src");
            let title_part = if !title.is_empty() {
                format!(" {}", self.quote(&title))
            } else {
                String::new()
            };

            self.write(&format!(
                "![{}]({}{})",
                escaped_alt,
                self.esc(&src, false),
                title_part
            ));
        }
    }

    fn esc(&self, text: &str, start_of_line: bool) -> String {
        let regex = regex::Regex::new(r"[`*\\~\[\]]").unwrap();
        let mut result = regex.replace_all(text, "\\$0").to_string();

        if start_of_line {
            let start_regex = regex::Regex::new(r"^[:#\-*+]").unwrap();
            result = start_regex.replace(&result, "\\$0").to_string();

            let num_regex = regex::Regex::new(r"^(\d+)\.").unwrap();
            result = num_regex.replace(&result, "$1\\.").to_string();
        }

        let newline_regex = regex::Regex::new(r"\r?\n").unwrap();
        result = newline_regex.replace_all(&result, "\\\n").to_string();

        result
    }

    fn html_esc(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#039;")
    }

    fn encode_uri(&self, uri: &str) -> String {
        utf8_percent_encode(uri, URI_ENCODE_SET).to_string()
    }

    fn quote(&self, text: &str) -> String {
        let wrap = if !text.contains('"') {
            "\"\""
        } else if !text.contains('\'') {
            "''"
        } else {
            "()"
        };
        format!("{}{}{}", &wrap[0..1], text, &wrap[1..2])
    }
}

fn make_query(params: &HashMap<String, AttrValue>) -> String {
    params
        .iter()
        .filter_map(|(k, v)| {
            let value_str = match v {
                AttrValue::Str(s) => Some(s.clone()),
                AttrValue::Num(n) => Some(n.to_string()),
                AttrValue::Bool(b) => Some(b.to_string()),
                _ => None,
            };
            value_str.map(|val| {
                format!(
                    "{}={}",
                    utf8_percent_encode(k, percent_encoding::NON_ALPHANUMERIC),
                    utf8_percent_encode(&val, percent_encoding::NON_ALPHANUMERIC)
                )
            })
        })
        .collect::<Vec<_>>()
        .join("&")
}
