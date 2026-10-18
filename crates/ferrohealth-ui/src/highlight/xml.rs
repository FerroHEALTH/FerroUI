// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The XML tokenizer: character data outside markup stays plain; inside a
//! tag the element name, attribute names and quoted values are classed, and
//! comments, declarations and CDATA sections are one comment token each.

use std::iter::Peekable;
use std::str::Chars;

use crate::highlight::{Kind, Token, one, push, quoted, take_while};

/// The tokens of the XML text `body`.
pub(super) fn tokens(body: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut chars = body.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c != '<' {
            let text = take_while(&mut chars, |c| c != '<');
            push(&mut out, Kind::Plain, text);
            continue;
        }
        if starts_with(&chars, "<!--") {
            push(&mut out, Kind::Comment, take_through(&mut chars, "-->"));
            continue;
        }
        if starts_with(&chars, "<![CDATA[") {
            push(&mut out, Kind::Comment, take_through(&mut chars, "]]>"));
            continue;
        }
        if starts_with(&chars, "<!") {
            push(&mut out, Kind::Comment, take_through(&mut chars, ">"));
            continue;
        }
        let opener = take_while(&mut chars, |c| matches!(c, '<' | '/' | '?'));
        push(&mut out, Kind::Punctuation, opener);
        let name = take_while(&mut chars, is_name_char);
        push(&mut out, Kind::Tag, name);
        tag_body(&mut chars, &mut out);
    }
    out
}

/// The inside of one tag, up to and including its closing `>`.
fn tag_body(chars: &mut Peekable<Chars<'_>>, out: &mut Vec<Token>) {
    while let Some(&c) = chars.peek() {
        if c == '>' {
            push(out, Kind::Punctuation, one(chars));
            return;
        }
        if c.is_whitespace() {
            push(out, Kind::Plain, take_while(chars, char::is_whitespace));
        } else if c == '"' || c == '\'' {
            push(out, Kind::String, quoted(chars));
        } else if is_name_char(c) {
            push(out, Kind::Attribute, take_while(chars, is_name_char));
        } else {
            // `=`, `/`, `?` and anything a malformed tag carries, one at a
            // time, so the scan always advances.
            push(out, Kind::Punctuation, one(chars));
        }
    }
}

/// Up to and including `terminator`, or to the end of the input.
fn take_through(chars: &mut Peekable<Chars<'_>>, terminator: &str) -> String {
    let mut text = String::new();
    for c in chars.by_ref() {
        text.push(c);
        if text.ends_with(terminator) {
            break;
        }
    }
    text
}

/// Whether the rest of the input starts with `prefix`.
fn starts_with(chars: &Peekable<Chars<'_>>, prefix: &str) -> bool {
    chars
        .clone()
        .take(prefix.chars().count())
        .eq(prefix.chars())
}

/// Whether `c` may stand in an element or attribute name, a namespace
/// prefix included.
fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, ':' | '_' | '-' | '.')
}

#[cfg(test)]
mod tests {
    use super::tokens;
    use crate::highlight::{Kind, Token};

    /// Every token of `kind`, its texts joined in order.
    fn text_of(tokens: &[Token], kind: Kind) -> String {
        tokens
            .iter()
            .filter(|token| token.kind == kind)
            .map(|token| token.text.as_str())
            .collect()
    }

    #[test]
    fn xml_tags_attributes_values_and_comments_are_classified() {
        let body = "<!-- gen --><composition xmlns=\"http://schemas.openehr.org/v1\" \
                    archetype_node_id='at0000'>\n  <name>Pulse</name>\n</composition>";
        let tokens = tokens(body);
        assert_eq!(
            "compositionnamenamecomposition",
            text_of(&tokens, Kind::Tag)
        );
        assert_eq!("xmlnsarchetype_node_id", text_of(&tokens, Kind::Attribute));
        assert_eq!(
            "\"http://schemas.openehr.org/v1\"'at0000'",
            text_of(&tokens, Kind::String)
        );
        assert_eq!("<!-- gen -->", text_of(&tokens, Kind::Comment));
        assert!(
            text_of(&tokens, Kind::Plain).contains("Pulse"),
            "{tokens:?}"
        );
    }

    #[test]
    fn broken_markup_reads_back_whole() {
        for body in [
            "<broken attr=",
            "<x a=\"1\" b='2'><![CDATA[<raw>]]>text &amp; more</x>",
            "<?xml version=\"1.0\"?>",
        ] {
            let joined: String = tokens(body).into_iter().map(|token| token.text).collect();
            assert_eq!(body, joined, "{body:?}");
        }
    }
}
