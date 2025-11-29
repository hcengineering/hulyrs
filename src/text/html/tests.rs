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
    use text::AttrValue;

    use crate::text::markdown::normalize_markdown;
    use crate::text::{
        self, attrs, blockquote, code_block, doc, embed, hard_break, heading_without_marker, hr,
        list_item, node, ordered_list, para, reference, text, text_with_marks,
    };
    use crate::text::{html_to_markup, markup_to_html};

    fn assert_renders_to_html(markup: text::MarkupNode, expected: &str) {
        let result = markup_to_html(&markup);
        assert_eq!(normalize_markdown(&result), normalize_markdown(expected));
    }

    fn assert_parses_from_html(html: &str, expected: text::MarkupNode) {
        let result = html_to_markup(html);
        assert_eq!(result, expected, "Failed to parse HTML correctly");
    }

    fn assert_round_trip(html: &str) {
        let parsed = html_to_markup(html);
        let serialized = markup_to_html(&parsed);
        assert_eq!(normalize_markdown(&serialized), normalize_markdown(html));
    }

    #[test]
    fn test_paragraph() {
        let markup = doc(vec![para(vec![text("paragraph 1")])]);
        let html = "<p>paragraph 1</p>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_text_alignment() {
        let markup = doc(vec![
            node(
                text::MarkupNodeType::Heading,
                vec![text("heading 1")],
                attrs(vec![
                    ("level", AttrValue::Num(1)),
                    ("textAlign", AttrValue::Str("left".to_string())),
                ]),
            ),
            node(
                text::MarkupNodeType::Paragraph,
                vec![text("paragraph 1")],
                attrs(vec![("textAlign", AttrValue::Str("right".to_string()))]),
            ),
        ]);
        let html = "<h1 style=\"text-align: left\">heading 1</h1><p style=\"text-align: right\">paragraph 1</p>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_headings() {
        let markup = doc(vec![
            heading_without_marker(1, "heading 1"),
            heading_without_marker(2, "heading 2"),
            heading_without_marker(3, "heading 3"),
            heading_without_marker(4, "heading 4"),
            heading_without_marker(5, "heading 5"),
            heading_without_marker(6, "heading 6"),
        ]);
        let html = "<h1>heading 1</h1><h2>heading 2</h2><h3>heading 3</h3><h4>heading 4</h4><h5>heading 5</h5><h6>heading 6</h6>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_blockquote() {
        let markup = doc(vec![blockquote(vec![para(vec![text(
            "Lorem ipsum dolor sit amet.",
        )])])]);
        let html = "<blockquote><p>Lorem ipsum dolor sit amet.</p></blockquote>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_codeblock() {
        let markup = doc(vec![code_block("typescript", "const x: number = 42;")]);
        let html = "<pre><code class=\"language-typescript\">const x: number = 42;</code></pre>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_hr() {
        let markup = doc(vec![
            para(vec![text("paragraph 1")]),
            hr(),
            para(vec![text("paragraph 2")]),
        ]);
        let html = "<p>paragraph 1</p><hr/><p>paragraph 2</p>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_br() {
        let markup = doc(vec![para(vec![
            text("line 1"),
            hard_break(),
            text("line 2"),
        ])]);
        let html = "<p>line 1<br/>line 2</p>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_ordered_list() {
        let markup = doc(vec![ordered_list(vec![
            list_item("item 1"),
            list_item("item 2"),
            list_item("item 3"),
        ])]);
        let html = "<ol><li><p>item 1</p></li><li><p>item 2</p></li><li><p>item 3</p></li></ol>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_bullet_list() {
        let markup = doc(vec![node(
            text::MarkupNodeType::BulletList,
            vec![
                list_item("item 1"),
                list_item("item 2"),
                list_item("item 3"),
            ],
            std::collections::HashMap::new(),
        )]);
        let html = "<ul><li><p>item 1</p></li><li><p>item 2</p></li><li><p>item 3</p></li></ul>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_ref() {
        let markup = doc(vec![para(vec![
            text("hello "),
            reference(vec![
                ("id", "64708c79c8f2613474dea38b"),
                ("objectclass", "contact:class:Person"),
                ("label", "John Doe"),
            ]),
        ])]);
        let html = "<p>hello <span data-type=\"reference\" data-id=\"64708c79c8f2613474dea38b\" data-objectclass=\"contact:class:Person\" data-label=\"John Doe\">@John Doe</span></p>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_embed() {
        let markup = doc(vec![para(vec![
            text("hello "),
            embed("http://localhost/embed"),
        ])]);
        let html = "<p>hello <a href=\"http://localhost/embed\" data-type=\"embed\">http://localhost/embed</a></p>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_embed_uri_escape() {
        let markup = doc(vec![para(vec![
            text("hello "),
            embed("http://localhost/embed spaces"),
        ])]);
        let html = "<p>hello <a href=\"http://localhost/embed%20spaces\" data-type=\"embed\">http://localhost/embed spaces</a></p>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_embed_html_escape() {
        let markup = doc(vec![para(vec![
            text("hello "),
            embed("http://localhost/embed<html>"),
        ])]);
        let html = "<p>hello <a href=\"http://localhost/embed%3Chtml%3E\" data-type=\"embed\">http://localhost/embed&lt;html&gt;</a></p>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }

    #[test]
    fn test_link_with_class() {
        let markup = doc(vec![para(vec![text_with_marks(
            "https://example.com",
            vec![crate::text::mark(
                text::MarkupMarkType::Link,
                vec![
                    ("href", AttrValue::Str("https://example.com".to_string())),
                    ("target", AttrValue::Str("_blank".to_string())),
                    ("rel", AttrValue::Str("noopener noreferrer".to_string())),
                    ("class", AttrValue::Str("cursor-pointer".to_string())),
                ],
            )],
        )])]);
        let html = "<p><a href=\"https://example.com\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"cursor-pointer\">https://example.com</a></p>";

        assert_renders_to_html(markup.clone(), html);
        assert_parses_from_html(html, markup.clone());
        assert_round_trip(html);
    }
}
