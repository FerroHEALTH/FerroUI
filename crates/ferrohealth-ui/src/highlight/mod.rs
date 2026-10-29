// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The syntax highlighting of YAML, JSON, XML, AQL and HL7 v2, compiled
//! into both halves, so the server and the browser draw the same text the
//! same way.
//!
//! Each tokenizer is one linear pass over the characters of its input, so a
//! multi-byte character is never split, and the texts of its tokens joined
//! together are the input exactly: text the grammar does not classify is a
//! [`Kind::Plain`] token, never dropped. It is a pure function of its input,
//! so the server pass and hydration agree. The classes are decoration over
//! text that reads the same without them. [`html`] writes a text as escaped,
//! classed lines, which the code panel and the editor draw. The YAML
//! tokenizer is FerroBRIDGE's, the AQL and JSON ones FerroFED's, the XML one
//! FerroEHR's, the HL7 v2 one the library's own for FerroBRIDGE's message
//! viewer. No specification governs the highlighting: our own design.

mod aql;
mod hl7v2;
mod json;
mod xml;
mod yaml;

use std::iter::Peekable;
use std::str::Chars;

/// The longest text the tokenizers classify.
///
/// A longer one is one [`Kind::Plain`] token, so a page never carries a span
/// per word of a very large document; server and browser make the same
/// decision from the length.
pub const MAX_HIGHLIGHT_BYTES: usize = 512 * 1024;

/// A grammar a text is highlighted as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// YAML 1.2, such as a mapping file.
    Yaml,
    /// JSON (RFC 8259).
    Json,
    /// XML 1.0.
    Xml,
    /// openEHR AQL.
    Aql,
    /// An HL7 v2 message, one segment per line.
    Hl7v2,
    /// Text drawn as it stands.
    Plain,
}

impl Language {
    /// Every language, in the order the gallery shows them.
    pub const ALL: [Self; 6] = [
        Self::Yaml,
        Self::Json,
        Self::Xml,
        Self::Aql,
        Self::Hl7v2,
        Self::Plain,
    ];

    /// The language's name, as a label shows it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Yaml => "YAML",
            Self::Json => "JSON",
            Self::Xml => "XML",
            Self::Aql => "AQL",
            Self::Hl7v2 => "HL7 v2",
            Self::Plain => "text",
        }
    }

    /// The grammar of a document `body`, from its first character that is
    /// not whitespace: JSON for `{` or `[`, XML for `<`, text otherwise.
    #[must_use]
    pub fn detect(body: &str) -> Self {
        match body.trim_start().chars().next() {
            Some('{' | '[') => Self::Json,
            Some('<') => Self::Xml,
            _ => Self::Plain,
        }
    }
}

/// The class of one token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Whitespace, and text the grammar does not classify.
    Plain,
    /// A YAML mapping key or a JSON member name.
    Key,
    /// A quoted string, or an XML attribute value.
    String,
    /// A number.
    Number,
    /// A keyword or a literal: `true`, `false`, `null`, an AQL reserved
    /// word, a YAML anchor, alias or tag.
    Keyword,
    /// A comment, an XML declaration or a CDATA section.
    Comment,
    /// Punctuation: brackets, separators, operators.
    Punctuation,
    /// An XML element name.
    Tag,
    /// An XML attribute name.
    Attribute,
    /// An AQL path or predicate.
    Path,
    /// An AQL parameter, `$name`.
    Parameter,
    /// An AQL identifier: a variable, an RM type, a function.
    Identifier,
    /// An HL7 v2 segment's id, such as `PID`.
    Segment,
    /// An HL7 v2 separator: the field, component, repetition, escape or
    /// subcomponent separator.
    Separator,
}

impl Kind {
    /// The class the stylesheet paints a token of this kind with, or `None`
    /// for text drawn as it stands.
    #[must_use]
    pub const fn class(self) -> Option<&'static str> {
        match self {
            Self::Plain | Self::Punctuation | Self::Identifier => None,
            Self::Key => Some("fui-tok-key"),
            Self::String => Some("fui-tok-string"),
            Self::Number => Some("fui-tok-number"),
            Self::Keyword => Some("fui-tok-keyword"),
            Self::Comment => Some("fui-tok-comment"),
            Self::Tag => Some("fui-tok-tag"),
            Self::Attribute => Some("fui-tok-attribute"),
            Self::Path => Some("fui-tok-path"),
            Self::Parameter => Some("fui-tok-parameter"),
            Self::Segment => Some("fui-hl7-segid"),
            Self::Separator => Some("fui-hl7-sep"),
        }
    }
}

/// One token: its kind and the exact text of the input it covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// What the text is.
    pub kind: Kind,
    /// The text, as the input has it.
    pub text: String,
}

/// The tokens of `text` in `language`; their texts joined are `text`.
#[must_use]
pub fn tokens(language: Language, text: &str) -> Vec<Token> {
    if text.len() > MAX_HIGHLIGHT_BYTES || language == Language::Plain {
        return plain(text);
    }
    match language {
        Language::Yaml => yaml::tokens(text),
        Language::Json => json::tokens(text),
        Language::Xml => xml::tokens(text),
        Language::Aql => aql::tokens(text),
        Language::Hl7v2 => hl7v2::tokens(text),
        Language::Plain => plain(text),
    }
}

/// The tokens of `text` in `language`, cut into lines at each line break,
/// which no line carries. A final line break opens one more, empty, line,
/// as a text area shows it.
#[must_use]
pub fn lines(language: Language, text: &str) -> Vec<Vec<Token>> {
    let mut lines = vec![Vec::new()];
    for token in tokens(language, text) {
        let mut parts = token.text.split('\n');
        if let Some(first) = parts.next() {
            push_onto(&mut lines, token.kind, first);
        }
        for part in parts {
            lines.push(Vec::new());
            push_onto(&mut lines, token.kind, part);
        }
    }
    lines
}

/// Adds `text` as a token of `kind` to the last line of `lines`.
fn push_onto(lines: &mut [Vec<Token>], kind: Kind, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(line) = lines.last_mut() {
        line.push(Token {
            kind,
            text: text.to_owned(),
        });
    }
}

/// How [`html`] writes its lines.
#[derive(Debug, Clone, Copy, Default)]
pub struct Lines<'a> {
    /// The id prefix each line takes, its 1-based number after it, or `None`
    /// for lines with no id.
    pub ids: Option<&'a str>,
}

/// Returns `text` in `language` as HTML lines.
///
/// Each line is a `<span class="fui-code-line">`, with the class `marked`
/// names for its 1-based number beside, and with an id when `options` asks
/// for one; each classed token is a `<span>` of its class, and every
/// character is escaped. A final line break opens one more, empty, line.
#[must_use]
pub fn html(
    language: Language,
    text: &str,
    options: Lines<'_>,
    marked: impl Fn(u32) -> Option<&'static str>,
) -> String {
    let mut out = String::with_capacity(text.len().saturating_mul(2));
    for (number, line) in (1_u32..).zip(lines(language, text)) {
        let mut body = String::new();
        push_tokens(&line, &mut body);
        push_line(&mut out, number, marked(number), options.ids, &body);
    }
    out
}

/// Appends line `number` to `out` as [`html`] writes it: a
/// `<span class="fui-code-line">` with `class` beside, the id `prefix` and
/// the number when there is a prefix, around the line's HTML `body`.
pub(crate) fn push_line(
    out: &mut String,
    number: u32,
    class: Option<&str>,
    prefix: Option<&str>,
    body: &str,
) {
    out.push_str("<span class=\"fui-code-line");
    if let Some(class) = class {
        out.push(' ');
        out.push_str(class);
    }
    out.push('"');
    if let Some(prefix) = prefix {
        out.push_str(" id=\"");
        escape(prefix, out);
        out.push_str(&number.to_string());
        out.push('"');
    }
    out.push('>');
    out.push_str(body);
    out.push_str("</span>");
}

/// Returns the tokens of one line, as [`lines`] cut them, as HTML: each
/// classed token a `<span>` of its class, every character escaped.
#[must_use]
pub fn line_html(line: &[Token]) -> String {
    let mut out = String::new();
    push_tokens(line, &mut out);
    out
}

/// Returns `text` in `language` as one string of HTML per line, each as
/// [`line_html`] writes the line [`lines`] cuts, in one pass over the
/// tokens.
// TODO(#510): the code viewer reads it again in the documents pull request.
#[expect(dead_code, reason = "the code viewer returns in a later pull request of #510")]
pub(crate) fn lines_html(language: Language, text: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    for token in tokens(language, text) {
        let mut parts = token.text.split('\n');
        if let (Some(first), Some(line)) = (parts.next(), out.last_mut()) {
            push_token(token.kind, first, line);
        }
        for part in parts {
            let mut line = String::new();
            push_token(token.kind, part, &mut line);
            out.push(line);
        }
    }
    out
}

/// Appends the tokens `line` to `out` as escaped, classed HTML.
fn push_tokens(line: &[Token], out: &mut String) {
    for token in line {
        push_token(token.kind, &token.text, out);
    }
}

/// Appends `text` as a token of `kind` to `out`, escaped and classed; an
/// empty text adds nothing.
fn push_token(kind: Kind, text: &str, out: &mut String) {
    if text.is_empty() {
        return;
    }
    match kind.class() {
        Some(class) => {
            out.push_str("<span class=\"");
            out.push_str(class);
            out.push_str("\">");
            escape(text, out);
            out.push_str("</span>");
        }
        None => escape(text, out),
    }
}

/// Appends `text` to `out` with the five characters HTML gives meaning to
/// escaped (HTML Living Standard, §13.1.2, text and attribute values).
pub fn escape(text: &str, out: &mut String) {
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
}

/// The whole of `text` as one plain token, none for empty text.
fn plain(text: &str) -> Vec<Token> {
    if text.is_empty() {
        return Vec::new();
    }
    vec![Token {
        kind: Kind::Plain,
        text: text.to_owned(),
    }]
}

/// Adds `text` as a token of `kind`, joining it to the last token when that
/// is of the same kind, so a run costs one span.
fn push(out: &mut Vec<Token>, kind: Kind, text: String) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = out.last_mut()
        && last.kind == kind
    {
        last.text.push_str(&text);
        return;
    }
    out.push(Token { kind, text });
}

/// The next character alone.
fn one(chars: &mut Peekable<Chars<'_>>) -> String {
    chars.next().map(String::from).unwrap_or_default()
}

/// The run of characters from here that `accept` takes.
fn take_while(chars: &mut Peekable<Chars<'_>>, accept: impl Fn(char) -> bool) -> String {
    let mut text = String::new();
    while let Some(c) = chars.next_if(|c| accept(*c)) {
        text.push(c);
    }
    text
}

/// A string from its opening quote to the same quote closing it, a
/// backslash escaping the character after it; an unclosed string runs to
/// the end.
fn quoted(chars: &mut Peekable<Chars<'_>>) -> String {
    let mut text = String::new();
    let Some(quote) = chars.next() else {
        return text;
    };
    text.push(quote);
    let mut escaped = false;
    for c in chars.by_ref() {
        text.push(c);
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == quote {
            break;
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::{Kind, Language, Lines, html, line_html, lines, lines_html, tokens};

    #[test]
    fn the_lines_as_html_are_the_cut_lines_each_written() {
        for text in [
            "",
            "\n",
            "a: |\n  b\n  c\n",
            "{\"a\": [1,\n \"x\\\"y\"]}\n",
            "<a b='1'><!-- c\n d --></a>",
            "SELECT c\nFROM EHR e",
            "MSH|^~\\&|A\rPID|1",
            "naïve ✓ 患者\n<b>&",
        ] {
            for language in Language::ALL {
                let cut: Vec<String> = lines(language, text)
                    .iter()
                    .map(|line| line_html(line))
                    .collect();
                assert_eq!(cut, lines_html(language, text), "{language:?} {text:?}");
            }
        }
    }

    #[test]
    fn every_language_reads_back_as_its_input() {
        for text in [
            "",
            "a: 1\n- b\n",
            "{\"a\": [1, \"x\\\"y\"]}",
            "<a b='1'><!-- c --></a>",
            "SELECT c FROM EHR e",
            "naïve ✓ 患者\n",
            "\u{1F600}\u{200D}\u{1F4BB}",
        ] {
            for language in Language::ALL {
                let joined: String = tokens(language, text)
                    .into_iter()
                    .map(|token| token.text)
                    .collect();
                assert_eq!(text, joined, "{language:?} {text:?}");
            }
        }
    }

    #[test]
    fn a_multi_line_token_is_cut_at_each_line_break() {
        let cut = lines(Language::Xml, "<!-- one\ntwo -->\n<a/>");
        assert_eq!(3, cut.len(), "{cut:?}");
        assert_eq!(Some(Kind::Comment), cut[1].first().map(|token| token.kind));
        assert_eq!(
            Some("two -->"),
            cut[1].first().map(|token| token.text.as_str())
        );
    }

    #[test]
    fn the_html_escapes_every_character_marks_and_numbers_every_line() {
        assert_eq!(
            concat!(
                "<span class=\"fui-code-line\" id=\"e-L1\"><span class=\"fui-tok-key\">a</span>: ",
                "<span class=\"fui-tok-string\">&quot;&lt;b&gt;&quot;</span></span>",
                "<span class=\"fui-code-line fui-mark-error\" id=\"e-L2\">&lt;/span&gt;</span>",
                "<span class=\"fui-code-line\" id=\"e-L3\"></span>"
            ),
            html(
                Language::Yaml,
                "a: \"<b>\"\n</span>\n",
                Lines { ids: Some("e-L") },
                |line| (line == 2).then_some("fui-mark-error")
            )
        );
        assert_eq!(
            "<span class=\"fui-code-line\">x &amp; &#39;y&#39;</span>",
            html(Language::Plain, "x & 'y'", Lines::default(), |_| None)
        );
    }

    #[test]
    fn a_line_writes_its_tokens_escaped_and_classed() {
        let cut = lines(Language::Yaml, "a: \"<b>\"");
        assert_eq!(
            "<span class=\"fui-tok-key\">a</span>: <span class=\"fui-tok-string\">&quot;&lt;b&gt;&quot;</span>",
            line_html(cut.first().map_or(&[][..], Vec::as_slice))
        );
    }

    #[test]
    fn a_document_names_its_grammar_by_its_first_character() {
        assert_eq!(Language::Json, Language::detect("  {\"a\": 1}"));
        assert_eq!(Language::Json, Language::detect("[1]"));
        assert_eq!(Language::Xml, Language::detect("\n<?xml version=\"1.0\"?>"));
        assert_eq!(Language::Plain, Language::detect("SELECT c"));
    }

    #[test]
    fn a_text_over_the_limit_is_one_plain_token() {
        let long = "x".repeat(super::MAX_HIGHLIGHT_BYTES + 1);
        let tokens = tokens(Language::Json, &long);
        assert_eq!(1, tokens.len(), "one token");
        assert_eq!(Some(Kind::Plain), tokens.first().map(|token| token.kind));
    }
}
