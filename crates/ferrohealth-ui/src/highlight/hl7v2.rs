// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The HL7 v2 tokenizer: each segment's id and the five separators.
//!
//! The separators are the ones the message declares in its header (HL7
//! v2.5.1, chapter 2, §2.5.4 "Message delimiters": MSH-1 is the field
//! separator, MSH-2 the component, repetition, escape and subcomponent
//! separators, in that order), read from a text that opens with `MSH`,
//! `BHS` or `FHS`; a text that declares none takes the defaults `|^~\&`.
//! A segment ends at a carriage return, a line feed or both.

use crate::highlight::{Kind, Token, push};

/// The five separators: the field separator, then the four encoding
/// characters.
type Separators = [char; 5];

/// The separators HL7 v2 recommends, which a text with no header takes.
const DEFAULT: Separators = ['|', '^', '~', '\\', '&'];

/// The tokens of the HL7 v2 message `message`.
pub(super) fn tokens(message: &str) -> Vec<Token> {
    let separators = declared(message);
    let mut out = Vec::new();
    let mut at_start = true;
    let mut chars = message.chars().peekable();
    while let Some(&next) = chars.peek() {
        if at_start {
            at_start = false;
            let mut id = String::new();
            while let Some(c) = chars.next_if(|c| !is_break(*c) && *c != separators[0]) {
                id.push(c);
            }
            let kind = if is_segment_id(&id) {
                Kind::Segment
            } else {
                Kind::Plain
            };
            push(&mut out, kind, id);
            continue;
        }
        let kind = if is_break(next) {
            at_start = true;
            Kind::Plain
        } else if separators.contains(&next) {
            Kind::Separator
        } else {
            Kind::Plain
        };
        if let Some(c) = chars.next() {
            push(&mut out, kind, String::from(c));
        }
    }
    out
}

/// Whether `c` ends a segment.
const fn is_break(c: char) -> bool {
    matches!(c, '\r' | '\n')
}

/// Whether `id` is a segment id: a three-character code (HL7 v2.5.1,
/// chapter 2, §2.5.2 "Segments and segment groups"), read here as three
/// upper-case letters or digits, the first a letter.
fn is_segment_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase())
        && id.len() == 3
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

/// The separators the header of `message` declares, or [`DEFAULT`].
fn declared(message: &str) -> Separators {
    let mut chars = message.chars();
    let header: String = chars.by_ref().take(3).collect();
    if !matches!(header.as_str(), "MSH" | "BHS" | "FHS") {
        return DEFAULT;
    }
    let Some(field) = chars.next().filter(|c| is_separator_char(*c)) else {
        return DEFAULT;
    };
    let mut separators = DEFAULT;
    separators[0] = field;
    for (slot, c) in separators
        .iter_mut()
        .skip(1)
        .zip(chars.take_while(|c| *c != field && is_separator_char(*c)))
    {
        *slot = c;
    }
    separators
}

/// Whether `c` can be a separator: printable, and neither a letter, a
/// digit, a space nor a segment's end.
fn is_separator_char(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace() && !c.is_control()
}

#[cfg(test)]
mod tests {
    use super::{declared, tokens};
    use crate::highlight::{Kind, Token};

    /// The tokens that are not plain, as kind and text.
    fn tinted(tokens: &[Token]) -> Vec<(Kind, &str)> {
        tokens
            .iter()
            .filter(|token| token.kind != Kind::Plain)
            .map(|token| (token.kind, token.text.as_str()))
            .collect()
    }

    #[test]
    fn a_message_tints_each_segment_id_and_every_separator() {
        let text = "MSH|^~\\&|GAL|SYN\rPID|1||SYN-0042^^^GAL~X&Y\nOBX|1";
        let tokens = tokens(text);
        assert_eq!(
            text,
            tokens.iter().map(|t| t.text.as_str()).collect::<String>()
        );
        let tinted = tinted(&tokens);
        assert_eq!(Some(&(Kind::Segment, "MSH")), tinted.first());
        assert!(tinted.contains(&(Kind::Segment, "PID")), "{tinted:?}");
        assert!(tinted.contains(&(Kind::Segment, "OBX")), "{tinted:?}");
        assert!(tinted.contains(&(Kind::Separator, "^^^")), "{tinted:?}");
        for separator in ["|", "~", "&"] {
            assert!(
                tinted.contains(&(Kind::Separator, separator)),
                "{separator} in {tinted:?}"
            );
        }
    }

    #[test]
    fn the_header_declares_the_separators_and_a_text_without_one_takes_the_defaults() {
        assert_eq!(['#', '*', '~', '\\', '&'], declared("MSH#*~\\&#GAL"));
        assert_eq!(['|', '^', '~', '\\', '&'], declared("PID|1"));
        assert_eq!(['|', '^', '~', '\\', '&'], declared("MSH"));
        let tokens = tokens("MSH#*~\\&#A|B\rPID#1*2");
        let tinted = tinted(&tokens);
        assert!(tinted.contains(&(Kind::Separator, "#")), "{tinted:?}");
        assert!(tinted.contains(&(Kind::Separator, "*")), "{tinted:?}");
        assert!(!tinted.contains(&(Kind::Separator, "|")), "{tinted:?}");
    }

    #[test]
    fn a_line_that_opens_with_no_segment_id_is_plain() {
        let tinted_ids: Vec<(Kind, String)> = tokens("pid|1\r\n\rZ1|x\rZZ1|y")
            .into_iter()
            .filter(|token| token.kind == Kind::Segment)
            .map(|token| (token.kind, token.text))
            .collect();
        assert_eq!(vec![(Kind::Segment, String::from("ZZ1"))], tinted_ids);
    }

    #[test]
    fn the_tokens_read_back_as_any_input() {
        for text in [
            "",
            "MSH",
            "MSH|",
            "\r\r",
            "naïve|患者^✓",
            "MSH|^~\\&|\u{1F600}",
        ] {
            let joined: String = tokens(text).into_iter().map(|token| token.text).collect();
            assert_eq!(text, joined, "{text:?}");
        }
    }
}
