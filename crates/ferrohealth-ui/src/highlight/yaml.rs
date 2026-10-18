// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The YAML tokenizer: each line on its own, a block mapping key, the flow
//! scalars, comments, anchors, aliases and tags (YAML 1.2.2).

use crate::highlight::{Kind, Token, push};

/// The tokens of the YAML text `text`, line breaks kept as plain text.
pub(super) fn tokens(text: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut first = true;
    for line_text in text.split('\n') {
        if !first {
            push(&mut out, Kind::Plain, String::from("\n"));
        }
        first = false;
        for token in line(line_text) {
            push(&mut out, token.kind, token.text);
        }
    }
    out
}

/// The tokens of the YAML line `line`, which carries no line break.
fn line(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut rest = line;
    let indent = rest.len() - rest.trim_start_matches([' ', '\t']).len();
    rest = take(&mut tokens, Kind::Plain, rest, indent);
    while let Some(after) = rest.strip_prefix('-')
        && (after.is_empty() || after.starts_with([' ', '\t']))
    {
        rest = take(&mut tokens, Kind::Punctuation, rest, 1);
        let space = rest.len() - rest.trim_start_matches([' ', '\t']).len();
        rest = take(&mut tokens, Kind::Plain, rest, space);
    }
    if let Some(key) = key_length(rest) {
        rest = take(&mut tokens, Kind::Key, rest, key);
        rest = take(&mut tokens, Kind::Punctuation, rest, 1);
    }
    value(&mut tokens, rest);
    tokens
}

/// Pushes the first `bytes` of `text` as a token of `kind`, and returns what
/// follows; an empty token is not pushed.
fn take<'a>(tokens: &mut Vec<Token>, kind: Kind, text: &'a str, bytes: usize) -> &'a str {
    let (head, tail) = text.split_at_checked(bytes).unwrap_or((text, ""));
    if !head.is_empty() {
        tokens.push(Token {
            kind,
            text: head.to_owned(),
        });
    }
    tail
}

/// The length of the block mapping key `text` opens with, when it opens
/// with one: a quoted or plain scalar followed by `:` and a space or the
/// end of the line.
fn key_length(text: &str) -> Option<usize> {
    if text.starts_with('#') {
        return None;
    }
    let first = text.chars().next()?;
    let end = if first == '"' || first == '\'' {
        quoted_length(text, first)?
    } else {
        plain_key_length(text)?
    };
    let after = text.get(end..)?;
    let next = after.strip_prefix(':')?;
    (next.is_empty() || next.starts_with([' ', '\t'])).then_some(end)
}

/// The length of the plain key `text` opens with: up to the first `:` that
/// a space or the end of the line follows, unless a comment starts first.
fn plain_key_length(text: &str) -> Option<usize> {
    for (index, character) in text.char_indices() {
        if character == ':' {
            let next = text.get(index + 1..).unwrap_or_default();
            if next.is_empty() || next.starts_with([' ', '\t']) {
                return Some(index);
            }
        }
        if character == '#'
            && text
                .get(..index)
                .is_some_and(|before| before.ends_with(' '))
        {
            return None;
        }
    }
    None
}

/// The length of the quoted scalar `text` opens with, its closing quote
/// included, or `None` when the line ends inside it.
fn quoted_length(text: &str, quote: char) -> Option<usize> {
    let mut characters = text.char_indices().skip(1).peekable();
    while let Some((index, character)) = characters.next() {
        if quote == '"' && character == '\\' {
            characters.next();
            continue;
        }
        if character == quote {
            if quote == '\'' && characters.peek().is_some_and(|(_, next)| *next == '\'') {
                characters.next();
                continue;
            }
            return Some(index + character.len_utf8());
        }
    }
    None
}

/// Pushes the tokens of the value part of a line: scalars, anchors, aliases,
/// tags, a comment, and the whitespace between them.
fn value(tokens: &mut Vec<Token>, mut rest: &str) {
    while !rest.is_empty() {
        let space = rest.len() - rest.trim_start_matches([' ', '\t']).len();
        if space > 0 {
            rest = take(tokens, Kind::Plain, rest, space);
            continue;
        }
        let first = rest.chars().next().unwrap_or(' ');
        if first == '#' {
            rest = take(tokens, Kind::Comment, rest, rest.len());
            continue;
        }
        if first == '"' || first == '\'' {
            let length = quoted_length(rest, first).unwrap_or(rest.len());
            rest = take(tokens, Kind::String, rest, length);
            continue;
        }
        if matches!(first, '&' | '*' | '!') {
            let length = rest.find([' ', '\t']).unwrap_or(rest.len());
            rest = take(tokens, Kind::Keyword, rest, length);
            continue;
        }
        let length = scalar_length(rest);
        let (scalar, _) = rest.split_at_checked(length).unwrap_or((rest, ""));
        // NOTE: YAML 1.2.2 §7.4, flow collections: a `,` ends a flow entry, so the
        // scalar before it is classed without it.
        let entry = scalar.strip_suffix(',').unwrap_or(scalar);
        let kind = classify(entry.trim_end());
        rest = take(tokens, kind, rest, entry.len());
        if entry.len() < length {
            rest = take(tokens, Kind::Punctuation, rest, 1);
        }
    }
}

/// The length of the plain scalar `text` opens with: up to a comment, which
/// a `#` after whitespace starts, or the end of the line.
fn scalar_length(text: &str) -> usize {
    let mut previous = ' ';
    for (index, character) in text.char_indices() {
        if character == '#' && index > 0 && matches!(previous, ' ' | '\t') {
            let head = text.get(..index).unwrap_or(text);
            return head.trim_end_matches([' ', '\t']).len();
        }
        previous = character;
    }
    text.len()
}

/// The kind of the plain scalar `scalar`.
fn classify(scalar: &str) -> Kind {
    match scalar {
        "true" | "false" | "null" | "~" | "True" | "False" | "Null" | "TRUE" | "FALSE" | "NULL" => {
            Kind::Keyword
        }
        _ if is_number(scalar) => Kind::Number,
        _ => Kind::Plain,
    }
}

/// Whether `scalar` reads as a YAML 1.2 core schema integer or float.
fn is_number(scalar: &str) -> bool {
    let digits = scalar.strip_prefix(['-', '+']).unwrap_or(scalar);
    !digits.is_empty()
        && digits.chars().any(|character| character.is_ascii_digit())
        && digits.chars().all(|character| {
            character.is_ascii_digit() || matches!(character, '.' | 'e' | 'E' | '-' | '+')
        })
        && digits
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_digit() || first == '.')
}

#[cfg(test)]
mod tests {
    use super::{Kind, Token, line};

    /// The texts of `tokens`, joined.
    fn joined(tokens: &[Token]) -> String {
        tokens.iter().map(|token| token.text.as_str()).collect()
    }

    /// The kinds and texts of `tokens` that carry a class.
    fn classed(tokens: &[Token]) -> Vec<(Kind, &str)> {
        tokens
            .iter()
            .filter(|token| token.kind.class().is_some())
            .map(|token| (token.kind, token.text.as_str()))
            .collect()
    }

    #[test]
    fn every_line_reads_back_as_itself() {
        for text in [
            "",
            "   ",
            "grammar: FHIRConnect/v1.0.0",
            "  - name: \"a: b\" # a comment",
            "- - x",
            "key: 'it''s' # c",
            "url: http://example.org/a#b",
            "count: -12.5e3",
            "anchor: &base",
            "<<: *base",
            "tag: !!str 7",
            "naïve: ünïcode # ü",
            "\"quoted key\": value",
            "unterminated: \"open",
            "# a whole-line comment",
            "\ttabbed: true",
            "  \"count\": 12,",
        ] {
            assert_eq!(text, joined(&line(text)), "{text:?}");
        }
    }

    #[test]
    fn keys_scalars_comments_anchors_and_tags_are_classed() {
        assert_eq!(
            vec![
                (Kind::Key, "name"),
                (Kind::String, "\"a: b\""),
                (Kind::Comment, "# a comment")
            ],
            classed(&line("  - name: \"a: b\" # a comment"))
        );
        assert_eq!(
            vec![(Kind::Key, "count"), (Kind::Number, "-12.5e3")],
            classed(&line("count: -12.5e3"))
        );
        assert_eq!(
            vec![(Kind::Key, "enabled"), (Kind::Keyword, "true")],
            classed(&line("enabled: true"))
        );
        assert_eq!(
            vec![(Kind::Key, "url")],
            classed(&line("url: http://example.org/a#b")),
            "a # inside a scalar is no comment"
        );
        assert_eq!(
            vec![
                (Kind::Key, "tag"),
                (Kind::Keyword, "!!str"),
                (Kind::Number, "7")
            ],
            classed(&line("tag: !!str 7"))
        );
        assert_eq!(
            vec![(Kind::Key, "\"count\""), (Kind::Number, "12")],
            classed(&line("  \"count\": 12,")),
            "a flow entry's comma is not part of its scalar"
        );
    }
}
