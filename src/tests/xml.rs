// Copyright 2025 Cameron Swords
// SPDX-License-Identifier: Apache-2.0

use crate::*;

// -------------------------------------------------------------------------------------------------
// XML AST a la Wadler
// -------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum XML {
    Element {
        name: String,
        attrs: Vec<Attribute>,
        body: Vec<XML>,
    },
    Text(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

impl Attribute {
    pub fn new<N: Into<String>, V: Into<String>>(name: N, value: V) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl XML {
    pub fn text<S: Into<String>>(s: S) -> Self {
        XML::Text(s.into())
    }

    pub fn element<N: Into<String>>(name: N, attrs: Vec<Attribute>, body: Vec<XML>) -> Self {
        XML::Element {
            name: name.into(),
            attrs,
            body,
        }
    }

    /// Convenience: `XML::elem("a", [("href","/")], [XML::text("home")])`
    pub fn elem(name: &str, attrs: Vec<(&str, &str)>, children: Vec<XML>) -> Self {
        let name = name.into();
        let attrs = attrs
            .into_iter()
            .map(|p| {
                let (n, v) = p.into();
                Attribute::new(n, v)
            })
            .collect();
        let body = children.into_iter().collect();
        XML::Element { name, attrs, body }
    }
}

// -------------------------------------------------------------------------------------------------
// XML to Doc
// -------------------------------------------------------------------------------------------------

pub fn xml_doc_pretty(x: &XML) -> Doc {
    match x {
        XML::Text(s) => Doc::text(escape_text(s)),

        XML::Element { name, attrs, body } => {
            let open_head = Doc::langle()
                .concat(Doc::text(name.clone()))
                .concat(attrs_doc(attrs));

            if body.is_empty() {
                // <tag .../>
                return open_head.concat_space(Doc::text("/>"));
            }

            let open = open_head.clone().concat(Doc::rangle());
            let close = Doc::text("</".to_string())
                .concat(Doc::text(name.clone()))
                .concat(Doc::rangle());

            if body.iter().any(|entry| matches!(entry, XML::Text(_))) {
                return open.concat(Doc::hsep(body.iter().map(xml_doc_pretty))).concat(close);
            }

            // Soft separator between children: space when flat, newline when broken
            let kids_soft = Doc::sep(body.iter().map(xml_doc_pretty));

            // Inline: no leading/trailing softlines → no stray spaces
            let inline =
                open.clone()
                    .concat(kids_soft.clone().flatten()) // children separated by spaces
                    .concat(close.clone());

            // Block: one child per line, indented. No .group() on the kids.
            let kids_vertical = body.iter()
                .map(xml_doc_pretty)
                .reduce(|a, b| a.concat(Doc::line()).concat(b))
                .unwrap_or_else(Doc::nil);

            let block =
                open
                    .concat(Doc::line())
                    .concat(kids_vertical.indent(4))
                    .concat(Doc::line())
                    .concat(close);

            // Choose: first if it fits, otherwise the vertical one.
            Doc::alt(inline, block)
        }
    }
}

pub fn xml_doc_compact(x: &XML) -> Doc {
    xml_doc_pretty(x).flatten()
}

// -------------------------------------------------------------------------------------------------
// Attributes
// -------------------------------------------------------------------------------------------------

fn attrs_doc(attrs: &[Attribute]) -> Doc {
    if attrs.is_empty() {
        return Doc::nil();
    }
    let parts = attrs.iter().map(|a| {
        Doc::text(&a.name)
            .concat(Doc::text("=\""))
            .concat(Doc::text(escape_attr(&a.value)))
            .concat(Doc::text("\""))
    });
    // Leading space before first attribute, then space-separated list.
    Doc::text(" ").concat(Doc::hsep(parts))
}

fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

// -------------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    // Adjust this helper if your API differs (e.g., `render_to_string(width)`).
    fn render(d: Doc, width: i16) -> String {
        d.render(width)
    }

    #[test]
    fn t1_simple_text() {
        let xml = XML::elem("p", vec![], vec![XML::text("Hello")]);
        assert_snapshot!("t1_compact", render(xml_doc_compact(&xml), 120));
        assert_snapshot!("t1_pretty", render(xml_doc_pretty(&xml), 20));
    }

    #[test]
    fn t2_attrs_and_nested() {
        let xml = XML::elem(
            "a",
            vec![("href", "/docs")],
            vec![XML::elem("b", vec![], vec![XML::text("click")])],
        );
        assert_snapshot!("t2_compact", render(xml_doc_compact(&xml), 120));
        assert_snapshot!("t2_pretty", render(xml_doc_pretty(&xml), 12));
    }

    #[test]
    fn t3_self_closing() {
        let xml = XML::element(
            "img".to_string(),
            vec![Attribute::new("src", "logo.png")],
            vec![],
        );
        assert_snapshot!("t3_compact", render(xml_doc_compact(&xml), 120));
        assert_snapshot!("t3_pretty", render(xml_doc_pretty(&xml), 10));
    }

    #[test]
    fn t4_escaping() {
        let xml = XML::elem(
            "msg",
            vec![("title", r#"He said "hi" & left <quickly>"#)],
            vec![XML::text("5 < 7 & 9 > 3")],
        );
        assert_snapshot!("t4_compact", render(xml_doc_compact(&xml), 120));
        assert_snapshot!("t4_pretty", render(xml_doc_pretty(&xml), 40));
    }

    #[test]
    fn t5_mixed_inline() {
        let xml = XML::elem(
            "p",
            vec![],
            vec![
                XML::text("Hello "),
                XML::elem("em", vec![], vec![XML::text("world")]),
                XML::text("!"),
            ],
        );
        assert_snapshot!("t5_compact", render(xml_doc_compact(&xml), 120));
        assert_snapshot!("t5_pretty", render(xml_doc_pretty(&xml), 8));
    }

    #[test]
    fn t6_block_children_break() {
        let li = |s| XML::elem("li", vec![], vec![XML::text(s)]);
        let xml = XML::elem("ul", vec![], vec![li("one"), li("two"), li("three")]);
        assert_snapshot!("t6_compact", render(xml_doc_compact(&xml), 120));
        assert_snapshot!("t6_pretty_narrow", render(xml_doc_pretty(&xml), 6));
        assert_snapshot!("t6_pretty_wide", render(xml_doc_pretty(&xml), 30));
    }

    #[test]
    fn t7_block_children_break() {
        let li = |s| XML::elem("li", vec![], vec![XML::text(s)]);
        let xml = XML::elem(
            "ul",
            vec![],
            vec![li("one"), li("two"), li("three"), li("four"), li("five")],
        );
        assert_snapshot!("t7_compact", render(xml_doc_compact(&xml), 120));
        assert_snapshot!("t7_pretty_narrow", render(xml_doc_pretty(&xml), 6));
        assert_snapshot!("t7_pretty_wide", render(xml_doc_pretty(&xml), 30));
    }
}
