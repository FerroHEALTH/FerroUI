// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The JSON tokenizer: member names, strings, numbers and the three
//! literals (RFC 8259).

use std::iter::Peekable;
use std::str::Chars;

use crate::highlight::{Kind, Token, one, push, quoted, take_while};

/// The tokens of the JSON text `json`.
pub(super) fn tokens(json: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut chars = json.chars().peekable();
    while let Some(&next) = chars.peek() {
        let (kind, text) = match next {
            '"' => {
                let text = quoted(&mut chars);
                // NOTE: RFC 8259 §4: a member name is the string a `:` follows,
                // whitespace allowed between them.
                let kind = if names_a_member(&chars) {
                    Kind::Key
                } else {
                    Kind::String
                };
                (kind, text)
            }
            '-' | '0'..='9' => (Kind::Number, number(&mut chars)),
            c if c.is_ascii_alphabetic() => {
                let word = take_while(&mut chars, |c| c.is_ascii_alphabetic());
                let kind = if matches!(word.as_str(), "true" | "false" | "null") {
                    Kind::Keyword
                } else {
                    Kind::Plain
                };
                (kind, word)
            }
            c if c.is_whitespace() => (Kind::Plain, take_while(&mut chars, char::is_whitespace)),
            '{' | '}' | '[' | ']' | ',' | ':' => (Kind::Punctuation, one(&mut chars)),
            _ => (Kind::Plain, one(&mut chars)),
        };
        push(&mut out, kind, text);
    }
    out
}

/// Whether the next character that is not whitespace is `:`.
fn names_a_member(chars: &Peekable<Chars<'_>>) -> bool {
    chars.clone().find(|c| !c.is_whitespace()) == Some(':')
}

/// A number: a leading minus, digits, a fraction and an exponent.
pub(super) fn number(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut text = String::new();
    if let Some(minus) = chars.next_if_eq(&'-') {
        text.push(minus);
    }
    let mut previous = None;
    while let Some(c) = chars.next_if(|&c| {
        c.is_ascii_digit()
            || matches!(c, '.' | 'e' | 'E')
            || (matches!(c, '+' | '-') && matches!(previous, Some('e' | 'E')))
    }) {
        text.push(c);
        previous = Some(c);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::tokens;
    use crate::highlight::{Kind, Token};

    /// The tokens that are not whitespace, as kind and trimmed text.
    fn shown(tokens: &[Token]) -> Vec<(Kind, &str)> {
        tokens
            .iter()
            .filter(|token| !token.text.trim().is_empty())
            .map(|token| (token.kind, token.text.trim()))
            .collect()
    }

    #[test]
    fn json_names_its_keys_strings_numbers_and_literals() {
        let text = "{\n  \"id\": \"example\",\n  \"ms\": -1.5E+3,\n  \"on\" : true,\n  \
                    \"auth\": null,\n  \"modes\": [\"a:b\", false],\n  \"name\": \"Zoë\"\n}";
        let tokens = tokens(text);
        assert_eq!(
            text,
            tokens.iter().map(|t| t.text.as_str()).collect::<String>()
        );
        let shown = shown(&tokens);
        for expected in [
            (Kind::Key, "\"id\""),
            (Kind::String, "\"example\""),
            (Kind::Number, "-1.5E+3"),
            (Kind::Key, "\"on\""),
            (Kind::Keyword, "true"),
            (Kind::Keyword, "null"),
            (Kind::String, "\"a:b\""),
            (Kind::Keyword, "false"),
            (Kind::String, "\"Zoë\""),
        ] {
            assert!(shown.contains(&expected), "{expected:?} in {shown:?}");
        }
    }

    #[test]
    fn an_escaped_quote_does_not_end_a_string() {
        let tokens = tokens(r#"{"q": "say \"hi\"", "n": 1}"#);
        assert!(
            shown(&tokens).contains(&(Kind::String, r#""say \"hi\"""#)),
            "{tokens:?}"
        );
    }
}
