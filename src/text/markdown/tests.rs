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

    use crate::text::{AttrValue, MarkupMarkType, MarkupNode};

    use crate::text::{
        bullet_list, doc, embed, heading, image, list_item, list_item_with, mark,
        markdown_to_markup, markup_to_markdown, mermaid, normalize_markdown, para, table,
        table_cell_with, table_row, text, text_with_marks, todo, todo_list, todo_with,
    };

    pub const IMAGE_URL: &str = "http://lo";
    pub const REF_URL: &str = "ref://";

    fn markdowns_equal(a: &str, b: &str) -> bool {
        normalize_markdown(a) == normalize_markdown(b)
    }

    pub fn assert_renders_to(markup: MarkupNode, expected: &str) {
        assert!(markdowns_equal(
            &markup_to_markdown(&markup, IMAGE_URL.to_string(), REF_URL.to_string()),
            expected
        ));
    }

    pub fn assert_parses_to(markdown: &str, expected: MarkupNode) {
        let parsed = markdown_to_markup(markdown);
        assert_eq!(parsed, expected, "Failed to parse markdown correctly");
    }

    pub fn assert_roundtrip(markdown: &str) {
        let parsed = markdown_to_markup(markdown);
        let serialized = markup_to_markdown(&parsed, IMAGE_URL.to_string(), REF_URL.to_string());
        if !markdowns_equal(&serialized, markdown) {
            eprintln!("Original:   {:?}", markdown);
            eprintln!("Serialized: {:?}", serialized);
            eprintln!("Parsed: {:?}", parsed);
        }
        assert!(markdowns_equal(&serialized, markdown), "Round-trip failed");
    }

    pub fn assert_roundtrip_with_alternate(markdown: &str, alternate: &str) {
        let parsed = markdown_to_markup(markdown);
        let serialized = markup_to_markdown(&parsed, IMAGE_URL.to_string(), REF_URL.to_string());
        assert!(
            markdowns_equal(&serialized, markdown) || markdowns_equal(&serialized, alternate),
            "Round-trip failed"
        );
    }

    #[test]
    fn test_simple_text() {
        let markdown = "Lorem ipsum dolor sit amet.";
        let markup = doc(vec![para(vec![text("Lorem ipsum dolor sit amet.")])]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_text_with_heading() {
        let markdown = "# Lorem ipsum\n\nLorem ipsum dolor sit amet.";
        let markup = doc(vec![
            heading(1, "Lorem ipsum"),
            para(vec![text("Lorem ipsum dolor sit amet.")]),
        ]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_bullet_list() {
        let markdown = "# bullet list\n\n- list item 1\n- list item 2";
        let markup = doc(vec![
            heading(1, "bullet list"),
            bullet_list(vec![list_item("list item 1"), list_item("list item 2")]),
        ]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_todos() {
        let markdown = "# TODO\n\n- [ ] todo 1\n- [x] todo 2";
        let markup = doc(vec![
            heading(1, "TODO"),
            todo_list(vec![todo("todo 1", false), todo("todo 2", true)]),
        ]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_todos_followed_by_list_items() {
        let markdown =
            "# todo and list\n\n- [ ] todo 1\n- [x] todo 2\n\n- list item 1\n- list item 2";
        let markup = doc(vec![
            heading(1, "todo and list"),
            todo_list(vec![todo("todo 1", false), todo("todo 2", true)]),
            bullet_list(vec![list_item("list item 1"), list_item("list item 2")]),
        ]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_mixed_lists() {
        let markdown =
            "# mixed lists\n\n- [ ] todo 1\n\n- list item 1\n\n- [x] todo 2\n\n- list item 2";
        let markup = doc(vec![
            heading(1, "mixed lists"),
            todo_list(vec![todo("todo 1", false)]),
            bullet_list(vec![list_item("list item 1")]),
            todo_list(vec![todo("todo 2", true)]),
            bullet_list(vec![list_item("list item 2")]),
        ]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_nested_todos() {
        let markdown = "# nested todos\n\n- [ ] todo\n  - [x] sub todo";
        let markup = doc(vec![
            heading(1, "nested todos"),
            todo_list(vec![todo_with(
                vec![
                    para(vec![text("todo")]),
                    todo_list(vec![todo("sub todo", true)]),
                ],
                false,
            )]),
        ]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_nested_lists() {
        let markdown = "# nested lists\n\n- [ ] todo\n  - sub list item\n  - [x] sub todo\n\n- list item\n  - [x] sub todo\n  - sub list item";
        let markup = doc(vec![
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
        ]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_mermaid_diagram() {
        let markdown = "```mermaid\ngraph TD;\n\tA-->B;\n\tA-->C;\n\tB-->D;\n\tC-->D;\n```";
        let markup = doc(vec![mermaid(
            "graph TD;\n\tA-->B;\n\tA-->C;\n\tB-->D;\n\tC-->D;",
        )]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_embed() {
        let markdown = "<a href=\"http://localhost/embed\" data-type=\"embed\">http:&#x2F;&#x2F;localhost&#x2F;embed</a>";
        let markup = doc(vec![para(vec![embed("http://localhost/embed")])]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_embed_uri_escape() {
        let markdown = "<a href=\"http://localhost/embed%20spaces\" data-type=\"embed\">http:&#x2F;&#x2F;localhost&#x2F;embed spaces</a>";
        let markup = doc(vec![para(vec![embed("http://localhost/embed spaces")])]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_embed_html_escape() {
        let markdown = "<a href=\"http://localhost/embed%3Chtml%3E\" data-type=\"embed\">http:&#x2F;&#x2F;localhost&#x2F;embed&lt;html&gt;</a>";
        let markup = doc(vec![para(vec![embed("http://localhost/embed<html>")])]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_multiline_image_alt() {
        let markdown = "![line0\\\n\\\nline1](http://example.com/image.png)";
        let markup = doc(vec![para(vec![image(
            "http://example.com/image.png",
            "line0\n\nline1",
        )])]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_image_in_table_cell() {
        let markdown = "<table><tbody><tr><td><p>Some text</p><p> <img src=\"files/image_1.png\" alt=\"image-alt\"/></p></td></tr></tbody></table>";
        let markup = doc(vec![table(vec![table_row(vec![table_cell_with(vec![
            para(vec![text("Some text")]),
            para(vec![text(" "), image("files/image_1.png", "image-alt")]),
        ])])])]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_text_color() {
        let markdown = "<span style=\"color: #abcdef\" data-color=\"#abcdef\">colored</span>";
        let markup = doc(vec![para(vec![text_with_marks(
            "colored",
            vec![mark(
                MarkupMarkType::TextColor,
                vec![("color", AttrValue::Str("#abcdef".to_string()))],
            )],
        )])]);

        assert_parses_to(markdown, markup.clone());
        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_links() {
        let markdown = "[Link](https://example.com)\n\n[Link with spaces](<https://example.com/with spaces>)\n\n[Link with spaces and braces](<https://example.com/\\<with spaces\\>>)";
        let markup = doc(vec![
            para(vec![text_with_marks(
                "Link",
                vec![mark(
                    MarkupMarkType::Link,
                    vec![("href", AttrValue::Str("https://example.com".to_string()))],
                )],
            )]),
            para(vec![text_with_marks(
                "Link with spaces",
                vec![mark(
                    MarkupMarkType::Link,
                    vec![(
                        "href",
                        AttrValue::Str("https://example.com/with spaces".to_string()),
                    )],
                )],
            )]),
            para(vec![text_with_marks(
                "Link with spaces and braces",
                vec![mark(
                    MarkupMarkType::Link,
                    vec![(
                        "href",
                        AttrValue::Str("https://example.com/<with spaces>".to_string()),
                    )],
                )],
            )]),
        ]);

        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_text_color_serialization() {
        let markdown = "<span style=\"color: #abcdef\" data-color=\"#abcdef\">colored</span>";
        let markup = doc(vec![para(vec![text_with_marks(
            "colored",
            vec![mark(
                MarkupMarkType::TextColor,
                vec![("color", AttrValue::Str("#abcdef".to_string()))],
            )],
        )])]);

        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_text_color_italic() {
        let markdown = "<span style=\"color: #abcdef\" data-color=\"#abcdef\">*styled*</span>";
        let markup = doc(vec![para(vec![text_with_marks(
            "styled",
            vec![
                mark(
                    MarkupMarkType::TextColor,
                    vec![("color", AttrValue::Str("#abcdef".to_string()))],
                ),
                mark(MarkupMarkType::Italic, vec![]),
            ],
        )])]);

        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_text_style_font_family() {
        let markdown = "<span style=\"font-family: Arial\">arial</span>";
        let markup = doc(vec![para(vec![text_with_marks(
            "arial",
            vec![mark(
                MarkupMarkType::TextStyle,
                vec![("fontFamily", AttrValue::Str("Arial".to_string()))],
            )],
        )])]);

        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_text_style_multiple_properties() {
        let markdown =
            "<span style=\"font-family: Arial; font-size: 16px; font-weight: bold\">styled</span>";
        let markup = doc(vec![para(vec![text_with_marks(
            "styled",
            vec![mark(
                MarkupMarkType::TextStyle,
                vec![
                    ("fontFamily", AttrValue::Str("Arial".to_string())),
                    ("fontSize", AttrValue::Str("16px".to_string())),
                    ("fontWeight", AttrValue::Str("bold".to_string())),
                ],
            )],
        )])]);

        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_text_style_color_no_data_color() {
        let markdown = "<span style=\"color: red\">red</span>";
        let markup = doc(vec![para(vec![text_with_marks(
            "red",
            vec![mark(
                MarkupMarkType::TextStyle,
                vec![("color", AttrValue::Str("red".to_string()))],
            )],
        )])]);

        assert_renders_to(markup, markdown);
    }

    #[test]
    fn test_roundtrip_italic() {
        let markdown = "*Asteriscs* and _Underscores_";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_bold() {
        let markdown = "**Asteriscs** and __Underscores__";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_bullet_list_asterisks() {
        let markdown = "Asterisks :\n\n* Firstly\n* Secondly";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_bullet_list_dashes() {
        let markdown = "Dashes :\n\n- Firstly\n- Secondly";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_todo_list_asterisks() {
        let markdown = "* [ ] Take\n* [ ] Do";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_todo_list_dashes() {
        let markdown = "- [x] Take\n- [ ] Do";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_different_markers() {
        let markdown = "Asterisks bulleted list:\n\n* Asterisks: *Italic* and  **Bold**\n* Underscores: _Italic_ and __Bold__\n\nDash bulleted list:\n\n- Asterisks: *Italic* and  **Bold**\n- Underscores: _Italic_ and __Bold__\n-";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_single_line_comment() {
        let markdown = "<!-- Do not erase me -->";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_multiline_comment() {
        let markdown = "<!--\n\nPlease title your PR as follows: `module: description` (e.g. `time: fix date format`).\nAlways start with the thing you are fixing, then describe the fix.\nDon't use past tense (e.g. \"fixed foo bar\").\n\nExplain what your PR does and why.\n\nIf you are adding a new function, please document it and add tests:\n\n```\n// foo does foo and bar\nfn foo() {\n\n// file_test.v\nfn test_foo() {\n    assert foo() == ...\n    ...\n}\n```\n\nIf you are fixing a bug, please add a test that covers it.\n\nBefore submitting a PR, please run `v test-all` .\nSee also `TESTS.md`.\n\nI try to process PRs as soon as possible. They should be handled within 24 hours.\n\nApplying labels to PRs is not needed.\n\nThanks a lot for your contribution!\n\n-->\n\nThis PR fix issue #22424";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_link() {
        let markdown = "See [link](https://example.com)";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_link_with_spaces() {
        let markdown = "See [link](<https://example.com/with spaces>)";
        let alternate = "See [link](https://example.com/with%20spaces)";
        assert_roundtrip_with_alternate(markdown, alternate);
    }

    #[test]
    fn test_roundtrip_link_with_spaces_and_braces() {
        let markdown = "See [link](<https://example.com/\\<with spaces\\>>)";
        let alternate = "See [link](https://example.com/%3Cwith%20spaces%3E)";
        assert_roundtrip_with_alternate(markdown, alternate);
    }

    #[test]
    fn test_roundtrip_codeblock() {
        let markdown = "```typescript\nconst x: number = 42;\n```";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_image() {
        let markdown =
            "<img width=\"320\" height=\"160\" src=\"http://example.com/image\" alt=\"image\">";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_images() {
        let markdown = "<img width=\"250\" height=\"330\" src=\"https://github.com/user-attachments/assets/f348e016-3f7d-45b1-b8a0-9098e9961885\" alt=\"Screenshot 2025-09-11 at 15 42 40\" />\n\n<img width=\"250\" height=\"230\" alt=\"Screenshot 2025-09-11 at 15 43 42\" src=\"https://github.com/user-attachments/assets/4502eba1-1f55-44df-b691-c4d3d3d3d67d\" >\n\n<img src=\"https://github.com/user-attachments/assets/e21431a3-2062-4b0b-9c8f-d06c92ede741\" alt=\"Screenshot 2025-09-11 at 15 43 50\" width=\"250\" height=\"210\" >";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_image_with_multiline_alt() {
        let markdown = "![link0\\\n\\\nline1](http://example.com/image.png)";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_table() {
        let markdown = "<table><tbody><tr><th><p>Header 1</p></th><th><p>Header 2</p></th></tr><tr><td><p>Cell 1</p></td><td><p>Cell 2</p></td></tr><tr><td><p>Cell 3</p></td><td><p>Cell 4</p></td></tr></tbody></table>";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_complex_table() {
        let markdown = "<table><tbody><tr><td colspan=\"2\" colwidth=\"320\"><p>Header</p></td></tr><tr><td rowspan=\"2\"><p>Cell 1</p></td><td><p>Cell 2</p></td></tr><tr><td><p>Cell 3</p></td></tr></tbody></table>";
        assert_roundtrip(markdown);
    }

    #[test]
    fn test_roundtrip_sub() {
        let markdown = "<sub>View in Huly <a href=\"http://localhost:8080/guest/github?token=token\">TSK-50</a></sub>";
        assert_roundtrip(markdown);
    }
}
