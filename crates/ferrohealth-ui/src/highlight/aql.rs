// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The AQL tokenizer: reserved words, identifiers, paths with their
//! predicates, strings, parameters, numbers and comments (openEHR AQL).

use std::iter::Peekable;
use std::str::Chars;

use crate::highlight::json::number;
use crate::highlight::{Kind, Token, one, push, quoted, take_while};

/// The AQL keywords, in upper case (openEHR AQL, the grammar's reserved
/// words), matched in any case.
const KEYWORDS: &[&str] = &[
    "AND",
    "AS",
    "ASC",
    "ASCENDING",
    "BY",
    "CONTAINS",
    "DESC",
    "DESCENDING",
    "DISTINCT",
    "EXISTS",
    "FALSE",
    "FETCH",
    "FROM",
    "LIKE",
    "LIMIT",
    "MATCHES",
    "NOT",
    "NULL",
    "OFFSET",
    "OR",
    "ORDER",
    "SELECT",
    "TOP",
    "TRUE",
    "WHERE",
    "XOR",
];

/// The tokens of the AQL text `aql`.
pub(super) fn tokens(aql: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut chars = aql.chars().peekable();
    while let Some(&next) = chars.peek() {
        let (kind, text) = match next {
            '\'' | '"' => (Kind::String, quoted(&mut chars)),
            '$' => (Kind::Parameter, parameter(&mut chars)),
            '[' => (Kind::Path, predicate(&mut chars)),
            '/' => path(&mut chars),
            '-' => comment_or_dash(&mut chars),
            c if c.is_ascii_digit() => (Kind::Number, number(&mut chars)),
            c if starts_word(c) => {
                let word = take_while(&mut chars, continues_word);
                let kind = if is_keyword(&word) {
                    Kind::Keyword
                } else {
                    Kind::Identifier
                };
                (kind, word)
            }
            c if c.is_whitespace() => (Kind::Plain, take_while(&mut chars, char::is_whitespace)),
            c if "(),;=<>!*.{}:+".contains(c) => (Kind::Punctuation, one(&mut chars)),
            _ => (Kind::Plain, one(&mut chars)),
        };
        push(&mut out, kind, text);
    }
    out
}

/// Whether `c` starts a word: a letter in any script, or `_`.
fn starts_word(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

/// Whether `c` continues a word.
fn continues_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Whether `word` is an AQL keyword, in any case.
fn is_keyword(word: &str) -> bool {
    KEYWORDS
        .iter()
        .any(|keyword| keyword.eq_ignore_ascii_case(word))
}

/// A parameter: `$` and the word after it.
fn parameter(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut text = one(chars);
    text.push_str(&take_while(chars, continues_word));
    text
}

/// A predicate from `[` to the `]` that closes it, strings inside it taken
/// whole, so a `]` in a quoted value does not end it.
fn predicate(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut text = one(chars);
    let mut depth = 1_u32;
    while let Some(&c) = chars.peek() {
        match c {
            '\'' | '"' => text.push_str(&quoted(chars)),
            _ => {
                text.push_str(&one(chars));
                if c == '[' {
                    depth = depth.saturating_add(1);
                } else if c == ']' {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
            }
        }
    }
    text
}

/// A path from its `/`: each step's name and its predicates, or the `/`
/// alone as punctuation when no name follows it.
fn path(chars: &mut Peekable<Chars<'_>>) -> (Kind, String) {
    let mut text = String::new();
    while chars.peek() == Some(&'/') {
        text.push_str(&one(chars));
        text.push_str(&take_while(chars, continues_word));
        while chars.peek() == Some(&'[') {
            text.push_str(&predicate(chars));
        }
    }
    if text == "/" {
        (Kind::Punctuation, text)
    } else {
        (Kind::Path, text)
    }
}

/// A comment from `--` to the end of the line, or a `-` alone as
/// punctuation.
fn comment_or_dash(chars: &mut Peekable<Chars<'_>>) -> (Kind, String) {
    let dash = one(chars);
    if chars.peek() != Some(&'-') {
        return (Kind::Punctuation, dash);
    }
    let mut text = dash;
    text.push_str(&take_while(chars, |c| c != '\n' && c != '\r'));
    (Kind::Comment, text)
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
    fn every_kind_of_aql_text_has_its_token() {
        let query = "SELECT c/uid/value, o/data[at0001]/events[at0006 and name/value='Any \
                     event']/data AS weight\nFROM EHR e[ehr_id/value=$ehr_id] CONTAINS \
                     COMPOSITION c[openEHR-EHR-COMPOSITION.encounter.v1]\n-- the newest first\n\
                     WHERE o/time > '2020-01-01' and 1.5e3 >= 10 ORDER BY c/name DESC LIMIT 10";
        let tokens = tokens(query);
        assert_eq!(
            query,
            tokens.iter().map(|t| t.text.as_str()).collect::<String>()
        );
        let shown = shown(&tokens);
        for expected in [
            (Kind::Keyword, "SELECT"),
            (Kind::Identifier, "c"),
            (Kind::Path, "/uid/value"),
            (
                Kind::Path,
                "/data[at0001]/events[at0006 and name/value='Any event']/data",
            ),
            (Kind::Path, "[ehr_id/value=$ehr_id]"),
            (Kind::Path, "[openEHR-EHR-COMPOSITION.encounter.v1]"),
            (Kind::Comment, "-- the newest first"),
            (Kind::String, "'2020-01-01'"),
            (Kind::Keyword, "and"),
            (Kind::Number, "1.5e3"),
            (Kind::Punctuation, ">="),
            (Kind::Keyword, "DESC"),
        ] {
            assert!(shown.contains(&expected), "{expected:?} in {shown:?}");
        }
    }

    #[test]
    fn a_parameter_is_its_dollar_and_its_name() {
        let tokens = tokens("WHERE e/ehr_id/value = $ehr_id AND c/name/value = $name_2");
        let shown = shown(&tokens);
        assert!(shown.contains(&(Kind::Parameter, "$ehr_id")), "{shown:?}");
        assert!(shown.contains(&(Kind::Parameter, "$name_2")), "{shown:?}");
    }
}
