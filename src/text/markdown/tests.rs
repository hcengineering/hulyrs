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

#[cfg(test)]
mod tests {
    use crate::text::markdown::markup_to_markdown;
    use crate::text::{AttrValue, MarkupMark, MarkupMarkType, MarkupNode, MarkupNodeType};
    use regex::Regex;
    use std::collections::HashMap;
    use test_case::test_case;

    const IMAGE_URL: &str = "http://lo";
    const REF_URL: &str = "ref://";

    fn render(markup: &MarkupNode) -> String {
        markup_to_markdown(markup, IMAGE_URL.to_string(), REF_URL.to_string())
    }

    fn assert_renders_to(markup: MarkupNode, expected: &str) {
        assert_eq!(
            normalize_markdown(&render(&markup)),
            normalize_markdown(expected)
        );
    }

    fn normalize_markdown(source: &str) -> String {
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

                let attr_regex =
                    Regex::new(r#"(\w+)(?:=(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?"#).unwrap();
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

    fn text(s: &str) -> MarkupNode {
        text_with_marks(s, vec![])
    }

    fn text_with_marks(s: &str, marks: Vec<MarkupMark>) -> MarkupNode {
        MarkupNode {
            node_type: MarkupNodeType::Text,
            content: Vec::new(),
            marks,
            attrs: HashMap::new(),
            text: s.to_string(),
        }
    }

    fn para(content: Vec<MarkupNode>) -> MarkupNode {
        MarkupNode {
            node_type: MarkupNodeType::Paragraph,
            content,
            marks: Vec::new(),
            attrs: HashMap::new(),
            text: String::new(),
        }
    }

    fn doc(content: Vec<MarkupNode>) -> MarkupNode {
        MarkupNode {
            node_type: MarkupNodeType::Doc,
            content,
            marks: Vec::new(),
            attrs: HashMap::new(),
            text: String::new(),
        }
    }

    fn mark(mark_type: MarkupMarkType, pairs: Vec<(&str, AttrValue)>) -> MarkupMark {
        MarkupMark {
            mark_type,
            attrs: attrs(pairs),
        }
    }

    fn bold_mark() -> MarkupMark {
        mark(MarkupMarkType::Bold, vec![])
    }

    fn italic_mark() -> MarkupMark {
        mark(MarkupMarkType::Italic, vec![])
    }

    fn strike_mark() -> MarkupMark {
        mark(MarkupMarkType::Strike, vec![])
    }

    fn underline_mark() -> MarkupMark {
        mark(MarkupMarkType::Underline, vec![])
    }

    fn link_mark(href: &str) -> MarkupMark {
        mark(
            MarkupMarkType::Link,
            vec![("href", AttrValue::Str(href.to_string()))],
        )
    }

    fn bold(s: &str) -> MarkupNode {
        text_with_marks(s, vec![bold_mark()])
    }

    fn code(s: &str) -> MarkupNode {
        text_with_marks(s, vec![mark(MarkupMarkType::Code, vec![])])
    }

    fn underline(s: &str) -> MarkupNode {
        text_with_marks(s, vec![underline_mark()])
    }

    fn link(href: &str, label: &str) -> MarkupNode {
        text_with_marks(label, vec![link_mark(href)])
    }

    fn heading(level: i32, content: &str) -> MarkupNode {
        node(
            MarkupNodeType::Heading,
            vec![text(content)],
            attrs(vec![
                ("level", AttrValue::Num(level)),
                ("marker", AttrValue::Str("#".to_string())),
            ]),
        )
    }

    fn list_item(content: &str) -> MarkupNode {
        node(
            MarkupNodeType::ListItem,
            vec![para(vec![text(content)])],
            HashMap::new(),
        )
    }

    fn list_item_with(content: Vec<MarkupNode>) -> MarkupNode {
        node(MarkupNodeType::ListItem, content, HashMap::new())
    }

    fn bullet_list(items: Vec<MarkupNode>) -> MarkupNode {
        node(
            MarkupNodeType::BulletList,
            items,
            attrs(vec![("bullet", AttrValue::Str("-".to_string()))]),
        )
    }

    fn todo(content: &str, checked: bool) -> MarkupNode {
        node(
            MarkupNodeType::TodoItem,
            vec![para(vec![text(content)])],
            attrs(vec![("checked", AttrValue::Bool(checked))]),
        )
    }

    fn todo_with_ids(content: &str, checked: bool, todoid: &str, userid: &str) -> MarkupNode {
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

    fn todo_with(content: Vec<MarkupNode>, checked: bool) -> MarkupNode {
        node(
            MarkupNodeType::TodoItem,
            content,
            attrs(vec![("checked", AttrValue::Bool(checked))]),
        )
    }

    fn todo_list(items: Vec<MarkupNode>) -> MarkupNode {
        node(
            MarkupNodeType::TodoList,
            items,
            attrs(vec![("bullet", AttrValue::Str("-".to_string()))]),
        )
    }

    fn ordered_list(items: Vec<MarkupNode>) -> MarkupNode {
        node(MarkupNodeType::OrderedList, items, HashMap::new())
    }

    fn ordered_list_from(start: i32, items: Vec<MarkupNode>) -> MarkupNode {
        node(
            MarkupNodeType::OrderedList,
            items,
            attrs(vec![("order", AttrValue::Num(start))]),
        )
    }

    fn hard_break() -> MarkupNode {
        node(MarkupNodeType::HardBreak, vec![], HashMap::new())
    }

    fn hard_break_with_marks(marks: Vec<MarkupMark>) -> MarkupNode {
        MarkupNode {
            node_type: MarkupNodeType::HardBreak,
            content: vec![],
            marks,
            attrs: HashMap::new(),
            text: String::new(),
        }
    }

    fn node(
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

    fn attrs(pairs: Vec<(&str, AttrValue)>) -> HashMap<String, AttrValue> {
        pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
    }

    fn embed(src: &str) -> MarkupNode {
        node(
            MarkupNodeType::Embed,
            vec![],
            attrs(vec![("src", AttrValue::Str(src.to_string()))]),
        )
    }

    fn image(src: &str, alt: &str) -> MarkupNode {
        node(
            MarkupNodeType::Image,
            vec![],
            attrs(vec![
                ("src", AttrValue::Str(src.to_string())),
                ("alt", AttrValue::Str(alt.to_string())),
            ]),
        )
    }

    fn image_with_attrs(attrs: HashMap<String, AttrValue>) -> MarkupNode {
        node(MarkupNodeType::Image, vec![], attrs)
    }

    fn mermaid(code: &str) -> MarkupNode {
        node(
            MarkupNodeType::Mermaid,
            vec![text(code)],
            attrs(vec![("language", AttrValue::Str("mermaid".to_string()))]),
        )
    }

    fn code_block(lang: &str, code: &str) -> MarkupNode {
        let attrs_map = if !lang.is_empty() {
            attrs(vec![("language", AttrValue::Str(lang.to_string()))])
        } else {
            HashMap::new()
        };
        node(MarkupNodeType::CodeBlock, vec![text(code)], attrs_map)
    }

    fn blockquote(content: Vec<MarkupNode>) -> MarkupNode {
        node(MarkupNodeType::Blockquote, content, HashMap::new())
    }

    fn hr() -> MarkupNode {
        node(MarkupNodeType::HorizontalRule, vec![], HashMap::new())
    }

    fn hr_custom(markup: &str) -> MarkupNode {
        node(
            MarkupNodeType::HorizontalRule,
            vec![],
            attrs(vec![("markup", AttrValue::Str(markup.to_string()))]),
        )
    }

    fn emoji(emoji_char: &str) -> MarkupNode {
        node(
            MarkupNodeType::Emoji,
            vec![],
            attrs(vec![("emoji", AttrValue::Str(emoji_char.to_string()))]),
        )
    }

    fn comment(content: &str) -> MarkupNode {
        node(MarkupNodeType::Comment, vec![text(content)], HashMap::new())
    }

    fn markdown_node(content: &str) -> MarkupNode {
        node(
            MarkupNodeType::Markdown,
            vec![text(content)],
            HashMap::new(),
        )
    }

    fn task_item(content: &str) -> MarkupNode {
        node(
            MarkupNodeType::TaskItem,
            vec![para(vec![text(content)])],
            HashMap::new(),
        )
    }

    fn task_list(items: Vec<MarkupNode>) -> MarkupNode {
        node(MarkupNodeType::TaskList, items, HashMap::new())
    }

    fn table_header(content: &str) -> MarkupNode {
        node(
            MarkupNodeType::TableHeader,
            vec![text(content)],
            HashMap::new(),
        )
    }

    fn table_cell(content: &str) -> MarkupNode {
        node(
            MarkupNodeType::TableCell,
            vec![text(content)],
            HashMap::new(),
        )
    }

    fn table_cell_with(content: Vec<MarkupNode>) -> MarkupNode {
        node(MarkupNodeType::TableCell, content, HashMap::new())
    }

    fn table_row(cells: Vec<MarkupNode>) -> MarkupNode {
        node(MarkupNodeType::TableRow, cells, HashMap::new())
    }

    fn table(rows: Vec<MarkupNode>) -> MarkupNode {
        node(MarkupNodeType::Table, rows, HashMap::new())
    }

    fn sublink(content: Vec<MarkupNode>) -> MarkupNode {
        node(MarkupNodeType::SubLink, content, HashMap::new())
    }

    fn reference(pairs: Vec<(&str, &str)>) -> MarkupNode {
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

    #[test]
    fn test_simple_text() {
        assert_renders_to(
            doc(vec![para(vec![text("Lorem ipsum dolor sit amet.")])]),
            "Lorem ipsum dolor sit amet.",
        );
    }

    #[test]
    fn test_text_with_heading() {
        assert_renders_to(
            doc(vec![
                heading(1, "Lorem ipsum"),
                para(vec![text("Lorem ipsum dolor sit amet.")]),
            ]),
            "# Lorem ipsum\n\nLorem ipsum dolor sit amet.\n",
        );
    }

    #[test]
    fn test_bullet_list() {
        assert_renders_to(
            doc(vec![
                heading(1, "bullet list"),
                bullet_list(vec![list_item("list item 1"), list_item("list item 2")]),
            ]),
            "# bullet list\n- list item 1\n- list item 2\n",
        );
    }

    #[test]
    fn test_todos() {
        assert_renders_to(
            doc(vec![
                heading(1, "TODO"),
                todo_list(vec![
                    todo_with_ids("todo 1", false, "todo123", "user456"),
                    todo("todo 2", true),
                ]),
            ]),
            "# TODO\n- [ ] <!-- todoid=todo123,userid=user456 -->todo 1\n- [x] todo 2\n",
        );
    }

    #[test]
    fn test_todos_followed_by_list_items() {
        assert_renders_to(
            doc(vec![
                heading(1, "todo and list"),
                todo_list(vec![todo("todo 1", false), todo("todo 2", true)]),
                bullet_list(vec![list_item("list item 1"), list_item("list item 2")]),
            ]),
            "# todo and list\n- [ ] todo 1\n- [x] todo 2\n- list item 1\n- list item 2\n",
        );
    }

    #[test]
    fn test_mixed_lists() {
        assert_renders_to(
            doc(vec![
                heading(1, "mixed lists"),
                todo_list(vec![todo("todo 1", false)]),
                bullet_list(vec![list_item("list item 1")]),
                todo_list(vec![todo("todo 2", true)]),
                bullet_list(vec![list_item("list item 2")]),
            ]),
            "# mixed lists\n- [ ] todo 1\n- list item 1\n- [x] todo 2\n- list item 2\n",
        );
    }

    #[test]
    fn test_nested_todos() {
        assert_renders_to(
            doc(vec![
                heading(1, "nested todos"),
                todo_list(vec![todo_with(
                    vec![
                        para(vec![text("todo")]),
                        todo_list(vec![todo("sub todo", true)]),
                    ],
                    false,
                )]),
            ]),
            "# nested todos\n- [ ] todo\n  - [x] sub todo\n",
        );
    }

    #[test]
    fn test_nested_lists() {
        assert_renders_to(
            doc(vec![
                heading(1, "nested lists"),
                todo_list(vec![todo_with(
                    vec![
                        para(vec![text("todo")]),
                        bullet_list(vec![list_item("sub list item")]),
                        todo_list(vec![todo("sub todo", true)]),
                    ],
                    false,
                )]),
                bullet_list(vec![list_item_with(vec![
                    para(vec![text("list item")]),
                    todo_list(vec![todo("sub todo", true)]),
                    bullet_list(vec![list_item("sub list item")]),
                ])]),
            ]),
            "# nested lists\n- [ ] todo\n  - sub list item\n  - [x] sub todo\n- list item\n  - [x] sub todo\n  - sub list item\n",
        );
    }

    #[test]
    fn test_mermaid_diagram() {
        assert_renders_to(
            doc(vec![mermaid(
                "graph TD;\n\tA-->B;\n\tA-->C;\n\tB-->D;\n\tC-->D;",
            )]),
            "```mermaid\ngraph TD;\n\tA-->B;\n\tA-->C;\n\tB-->D;\n\tC-->D;\n```",
        );
    }

    #[test_case("http://lo/embed", "<a href=\"http://lo/embed\" data-type=\"embed\">http:&#x2F;&#x2F;lo&#x2F;embed</a>" ; "basic")]
    #[test_case("http://lo/embed spaces", "<a href=\"http://lo/embed%20spaces\" data-type=\"embed\">http:&#x2F;&#x2F;lo&#x2F;embed spaces</a>" ; "uri escape")]
    #[test_case("http://lo/embed<html>", "<a href=\"http://lo/embed%3Chtml%3E\" data-type=\"embed\">http:&#x2F;&#x2F;lo&#x2F;embed&lt;html&gt;</a>" ; "html escape")]
    fn test_embed(url: &str, expected: &str) {
        assert_renders_to(doc(vec![para(vec![embed(url)])]), expected);
    }

    #[test]
    fn test_multiline_image_alt() {
        assert_renders_to(
            doc(vec![para(vec![image(
                "http://example.com/image.png",
                "line0\n\nline1",
            )])]),
            "![line0\\\n\\\nline1](http://example.com/image.png)",
        );
    }

    #[test]
    fn test_image_with_file_id_and_dimensions() {
        let attrs = attrs(vec![
            ("file-id", AttrValue::Str("abc123".to_string())),
            ("width", AttrValue::Num(800)),
            ("height", AttrValue::Num(600)),
            ("alt", AttrValue::Str("test image".to_string())),
        ]);

        assert_renders_to(
            doc(vec![para(vec![image_with_attrs(attrs)])]),
            "![test image](http://lo/abc123?width=800&height=600)",
        );
    }

    #[test]
    fn test_text_color() {
        let mark = mark(
            MarkupMarkType::TextColor,
            vec![("color", AttrValue::Str("#abcdef".to_string()))],
        );

        let text = text_with_marks("colored", vec![mark]);
        let markup = doc(vec![para(vec![text])]);
        let expected = "<span style=\"color: #abcdef\" data-color=\"#abcdef\">colored</span>";
        assert_renders_to(markup, expected);
    }

    #[test]
    fn test_links() {
        assert_renders_to(
            doc(vec![
                para(vec![link("https://example.com", "Link")]),
                para(vec![link(
                    "https://example.com/with spaces",
                    "Link with spaces",
                )]),
                para(vec![link(
                    "https://example.com/<with spaces>",
                    "Link with spaces and braces",
                )]),
            ]),
            "[Link](https://example.com)\n\n[Link with spaces](<https://example.com/with spaces>)\n\n[Link with spaces and braces](<https://example.com/\\<with spaces\\>>)",
        );
    }

    #[test]
    fn test_plain_url_autolink() {
        assert_renders_to(
            doc(vec![para(vec![link(
                "https://example.com",
                "https://example.com",
            )])]),
            "<https://example.com>",
        );
    }

    #[test]
    fn test_plain_url_not_autolink_with_title() {
        let mark = mark(
            MarkupMarkType::Link,
            vec![
                ("href", AttrValue::Str("https://example.com".to_string())),
                ("title", AttrValue::Str("Example".to_string())),
            ],
        );
        let text = text_with_marks("https://example.com", vec![mark]);

        assert_renders_to(
            doc(vec![para(vec![text])]),
            "[https://example.com](https://example.com \"Example\")",
        );
    }

    #[test]
    fn test_plain_url_not_autolink_different_text() {
        assert_renders_to(
            doc(vec![para(vec![link("https://example.com", "Click here")])]),
            "[Click here](https://example.com)",
        );
    }

    #[test]
    fn test_underline() {
        assert_renders_to(
            doc(vec![para(vec![underline("underlined text")])]),
            "<ins>underlined text</ins>\n",
        );
    }

    #[test_case(" bold text ", bold_mark(), " **bold text** "; "bold")]
    #[test_case("  italic  ", italic_mark(), "  *italic*  "; "italic")]
    #[test_case("\tstrike\t", strike_mark(), "\t~~strike~~\t"; "strike")]
    fn test_whitespace_expulsion(input: &str, mark: MarkupMark, expected: &str) {
        assert_renders_to(
            doc(vec![para(vec![text_with_marks(input, vec![mark])])]),
            expected,
        );
    }

    #[test]
    fn test_whitespace_no_expulsion_code() {
        assert_renders_to(doc(vec![para(vec![code(" code ")])]), "` code `");
    }

    #[test]
    fn test_whitespace_expulsion_combined() {
        assert_renders_to(
            doc(vec![para(vec![
                text("plain "),
                bold(" bold "),
                text(" plain"),
            ])]),
            "plain  **bold**  plain",
        );
    }

    #[test]
    fn test_text_color_with_italic() {
        let mark1 = mark(
            MarkupMarkType::TextColor,
            vec![("color", AttrValue::Str("#abcdef".to_string()))],
        );
        let mark2 = mark(MarkupMarkType::Italic, vec![]);

        assert_renders_to(
            doc(vec![para(vec![text_with_marks(
                "styled",
                vec![mark1, mark2],
            )])]),
            "<span style=\"color: #abcdef\" data-color=\"#abcdef\">*styled*</span>",
        );
    }

    #[test]
    fn test_mark_reordering_bold_italic() {
        assert_renders_to(
            doc(vec![para(vec![text_with_marks(
                "mixed",
                vec![bold_mark(), italic_mark()],
            )])]),
            "***mixed***",
        );
    }

    #[test]
    fn test_mark_reordering_with_strike() {
        assert_renders_to(
            doc(vec![para(vec![text_with_marks(
                "text",
                vec![bold_mark(), strike_mark()],
            )])]),
            "**~~text~~**",
        );
    }

    #[test]
    fn test_text_style() {
        let mark = mark(
            MarkupMarkType::TextStyle,
            vec![
                ("fontFamily", AttrValue::Str("Arial".to_string())),
                ("fontSize", AttrValue::Str("16px".to_string())),
                ("fontWeight", AttrValue::Str("bold".to_string())),
            ],
        );

        let text = text_with_marks("styled", vec![mark]);
        let markup = doc(vec![para(vec![text])]);
        let expected =
            "<span style=\"font-family: Arial; font-size: 16px; font-weight: bold\">styled</span>";

        assert_renders_to(markup, expected);
    }

    #[test]
    fn test_text_style_with_color_no_data_color() {
        let mut style_attrs = HashMap::new();
        style_attrs.insert("color".to_string(), AttrValue::Str("red".to_string()));

        let mark = MarkupMark {
            mark_type: MarkupMarkType::TextStyle,
            attrs: style_attrs,
        };

        let text = text_with_marks("red", vec![mark]);
        let markup = doc(vec![para(vec![text])]);

        let result = render(&markup);
        assert_eq!(
            normalize_markdown(&result),
            normalize_markdown("<span style=\"color: red\">red</span>")
        );
    }

    #[test]
    fn test_horizontal_rule() {
        assert_renders_to(doc(vec![hr(), hr_custom("***")]), "---\n***");
    }

    #[test_case(vec![para(vec![text("This is a quote")])], "> This is a quote" ; "single line")]
    #[test_case(vec![para(vec![text("First line")]), para(vec![text("Second line")])], "> First line\n>\n> Second line" ; "multiline")]
    fn test_blockquote(content: Vec<MarkupNode>, expected: &str) {
        assert_renders_to(doc(vec![blockquote(content)]), expected);
    }

    #[test_case("rust", "fn main() {\n    println!(\"Hello\");\n}", "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```" ; "with language")]
    #[test_case("", "some code", "```\nsome code\n```" ; "no language")]
    fn test_code_block(lang: &str, code: &str, expected: &str) {
        assert_renders_to(doc(vec![code_block(lang, code)]), expected);
    }

    #[test]
    fn test_emoji() {
        assert_renders_to(doc(vec![para(vec![emoji("😀")])]), "😀");
    }

    #[test_case("This is a comment", "<!--This is a comment-->" ; "single line")]
    #[test_case("Line 1\nLine 2\nLine 3", "<!--Line 1\nLine 2\nLine 3-->" ; "multiline")]
    fn test_comment(content: &str, expected: &str) {
        assert_renders_to(doc(vec![comment(content)]), expected);
    }

    #[test]
    fn test_markdown_passthrough() {
        assert_renders_to(
            doc(vec![markdown_node("**bold** and *italic*")]),
            "\\*\\*bold\\*\\* and \\*italic\\*",
        );
    }

    #[test_case(vec![text("Line 1"), hard_break(), text("Line 2")], "Line 1\\\nLine 2" ; "single")]
    #[test_case(vec![text("Line 1"), hard_break(), hard_break(), text("Line 2")], "Line 1\\\n\\\nLine 2" ; "multiple")]
    #[test_case(vec![text("Line 1"), hard_break()], "Line 1" ; "trailing")]
    fn test_hard_break(nodes: Vec<MarkupNode>, expected: &str) {
        assert_renders_to(doc(vec![para(nodes)]), expected);
    }

    #[test]
    fn test_hard_break_with_bold_mark() {
        let bm = bold_mark();
        assert_renders_to(
            doc(vec![para(vec![
                text_with_marks("Line 1", vec![bm.clone()]),
                hard_break_with_marks(vec![bm.clone()]),
                text_with_marks("Line 2", vec![bm]),
            ])]),
            "**Line 1\\\nLine 2**",
        );
    }

    #[test]
    fn test_hard_break_mark_not_continued() {
        let bm = bold_mark();
        assert_renders_to(
            doc(vec![para(vec![
                text_with_marks("Line 1", vec![bm.clone()]),
                hard_break_with_marks(vec![bm]),
                text("Line 2"),
            ])]),
            "**Line 1**\\\nLine 2",
        );
    }

    #[test]
    fn test_ordered_list() {
        assert_renders_to(
            doc(vec![
                heading(1, "ordered list"),
                ordered_list(vec![
                    list_item("First item"),
                    list_item("Second item"),
                    list_item("Third item"),
                ]),
            ]),
            "# ordered list\n1. First item\n2. Second item\n3. Third item\n",
        );
    }

    #[test]
    fn test_ordered_list_custom_start() {
        assert_renders_to(
            doc(vec![ordered_list_from(
                5,
                vec![list_item("Item five"), list_item("Item six")],
            )]),
            "5. Item five\n6. Item six\n",
        );
    }

    #[test]
    fn test_task_list() {
        assert_renders_to(
            doc(vec![
                heading(1, "task list"),
                task_list(vec![task_item("Task 1"), task_item("Task 2")]),
            ]),
            "# task list\n* [ ] Task 1\n* [ ] Task 2\n",
        );
    }

    #[test]
    fn test_reference_basic() {
        let markup = doc(vec![para(vec![reference(vec![
            ("label", "Issue-123"),
            ("objectclass", "tracker:class:Issue"),
            ("id", "issue123"),
        ])])]);

        let result = render(&markup);

        assert!(result.contains("[Issue-123]"));
        assert!(result.contains("ref://?"));
        assert!(result.contains("%5Fclass=") || result.contains("_class="));
        assert!(result.contains("%5Fid=") || result.contains("_id="));
        assert!(result.contains("label="));
    }

    #[test]
    fn test_reference_with_title() {
        let markup = doc(vec![para(vec![reference(vec![
            ("label", "Task-456"),
            ("objectclass", "task:class:Task"),
            ("id", "task456"),
            ("title", "Important Task"),
        ])])]);

        let result = render(&markup);

        assert!(result.contains("[Task-456]"));
        assert!(result.contains("\"Important Task\"") || result.contains("'Important Task'"));
    }

    #[test]
    fn test_sublink() {
        assert_renders_to(
            doc(vec![para(vec![sublink(vec![link(
                "https://example.com",
                "footer note",
            )])])]),
            "<sub><a href=\"https://example.com\">footer note</a></sub>\n",
        );
    }

    #[test]
    fn test_table_basic() {
        let markup = doc(vec![table(vec![
            table_row(vec![table_header("Name"), table_header("Age")]),
            table_row(vec![table_cell("Alice"), table_cell("30")]),
        ])]);

        let result = render(&markup);

        assert!(result.contains("<table>"));
        assert!(result.contains("</table>"));
        assert!(result.contains("<th>Name</th>"));
        assert!(result.contains("<th>Age</th>"));
        assert!(result.contains("<td>Alice</td>"));
        assert!(result.contains("<td>30</td>"));
    }

    #[test]
    fn test_table_with_paragraph_content() {
        let markup = doc(vec![table(vec![table_row(vec![table_cell_with(vec![
            para(vec![text("Cell content")]),
        ])])])]);

        let result = render(&markup);

        assert!(result.contains("<td>Cell content</td>"));
    }

    #[test]
    fn test_table_html_escaping() {
        let markup = doc(vec![table(vec![table_row(vec![table_cell(
            "<script>alert('xss')</script>",
        )])])]);
        let result = render(&markup);
        assert!(result.contains("&lt;script&gt;"));
        assert!(result.contains("&lt;/script&gt;"));
        assert!(!result.contains("<script>alert"));
    }
}
