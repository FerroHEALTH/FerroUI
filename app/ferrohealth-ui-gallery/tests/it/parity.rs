// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The logic written twice, in the library's Rust and again in a script
//! module, held to one answer (#548). Each test feeds the same inputs to
//! both copies and fails on any difference: the Rust copy runs here, and
//! the module's copy runs in the gallery's browser, imported from the page
//! as the loader imports it. `docs/consoles.md` lists every pair, and each
//! test's name carries the words of both copies' files, so a change to
//! either runs it (`scripts/checks/affected.sh`). This module holds the
//! harness, the highlighters and the editor's arithmetic; the dates, the
//! keys and the words follow in `parity_dates`, `parity_keys` and
//! `parity_words`.

use std::error::Error;

use ferrohealth_ui::highlight::{Kind, Language, line_html, lines, tokens};
use ferrohealth_ui::i18n::Locale;
use ferrohealth_ui::view::code::line_count;
use ferrohealth_ui::view::diagnostics::Position;
use ferrohealth_ui::view::editor::{
    Editor, EditorWords, LineMark, MarkKind, Opened, Panel, caret_at, editor_panel, mark_kind,
    offset_of,
};
use serde_json::{Value, json};

use crate::browser::with_browser;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// A page that loads the modules' loader.
const PAGE: &str = "/ferropix/forms";

/// Imports the module `module` (its file name without `.js`) and returns
/// what the function `call`, a script arrow function whose first argument
/// is the module, answers for each argument list of `cases`, read back as
/// JSON (`undefined` as `null`).
async fn answers(
    browser: &crate::browser::Browser,
    module: &str,
    call: &str,
    cases: &[Value],
) -> Result<Vec<Value>, Box<dyn Error>> {
    let script = format!(
        "const call = {call};\n{}",
        r"const done = arguments[arguments.length - 1];
        const [module, cases] = arguments;
        (async () => {
          const loader = document.querySelector('script[src*=\'fui.js\']');
          const found = await import(new URL(module + '.js', loader.src).href);
          const out = cases.map((args) => {
            const answer = call(found, ...args);
            return answer === undefined ? null : JSON.parse(JSON.stringify(answer));
          });
          done({ out });
        })().catch((error) => done({ error: String(error && error.stack || error) }));"
    );
    let answer = browser
        .driver()
        .execute_async(
            &script,
            vec![serde_json::to_value(module)?, Value::Array(cases.to_vec())],
        )
        .await?;
    let answer = answer.json().clone();
    if let Some(error) = answer.get("error") {
        return Err(format!("the module {module} threw: {error}").into());
    }
    match answer.get("out") {
        Some(Value::Array(out)) if out.len() == cases.len() => Ok(out.clone()),
        _ => Err(format!("the module {module} answered {answer}").into()),
    }
}

/// Runs `call` of the module `module` over the argument list of each case
/// in the browser and fails naming every case whose answer differs from
/// the Rust copy's, the second of the case.
pub(crate) async fn held(
    module: &str,
    call: &str,
    cases: Vec<(Value, Value)>,
) -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    assert!(
        !cases.is_empty(),
        "a parity test feeds both copies something"
    );
    let gallery = Gallery::start().await?;
    let (inputs, rust): (Vec<Value>, Vec<Value>) = cases.into_iter().unzip();
    let script = with_answers(&gallery, module, call, &inputs).await?;
    let differ: Vec<String> = inputs
        .iter()
        .zip(rust.iter().zip(&script))
        .filter(|(_, (rust, script))| normal(rust) != normal(script))
        .map(|(input, (rust, script))| {
            format!(
                "{module} {}:\n  the Rust copy  {rust}\n  the module     {script}",
                cut(&input.to_string())
            )
        })
        .collect();
    assert!(
        differ.is_empty(),
        "{} of {} cases answer differently in Rust and in {module}.js:\n{}",
        differ.len(),
        inputs.len(),
        differ.join("\n")
    );
    Ok(())
}

/// `text` cut to its first 400 characters, so a case that carries a
/// page of words still reads in a failure.
fn cut(text: &str) -> String {
    match text.char_indices().nth(400) {
        Some((at, _)) => format!("{}…", text.get(..at).unwrap_or(text)),
        None => text.to_owned(),
    }
}

/// `value` with every number as a float, so the integer a script writes
/// for `1.0` reads as the float Rust writes.
fn normal(value: &Value) -> Value {
    match value {
        Value::Number(number) => number
            .as_f64()
            .and_then(serde_json::Number::from_f64)
            .map_or_else(|| value.clone(), Value::Number),
        Value::Array(items) => Value::Array(items.iter().map(normal).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(name, field)| (name.clone(), normal(field)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// The answers of `call` over `inputs`, in a fresh browser at [`PAGE`].
async fn with_answers(
    gallery: &Gallery,
    module: &str,
    call: &str,
    inputs: &[Value],
) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut out = Vec::new();
    with_browser(async |browser| {
        browser.goto(&gallery.url(PAGE)).await?;
        out = answers(browser, module, call, inputs).await?;
        Ok(())
    })
    .await?;
    Ok(out)
}

/// The name the `highlight` module gives a token of `kind`.
const fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Plain => "plain",
        Kind::Key => "key",
        Kind::String => "string",
        Kind::Number => "number",
        Kind::Keyword => "keyword",
        Kind::Comment => "comment",
        Kind::Punctuation => "punctuation",
        Kind::Tag => "tag",
        Kind::Attribute => "attribute",
        Kind::Path => "path",
        Kind::Parameter => "parameter",
        Kind::Identifier => "identifier",
        Kind::Segment => "segment",
        Kind::Separator => "separator",
    }
}

/// The key the `highlight` module names `language` by.
const fn language_key(language: Language) -> &'static str {
    match language {
        Language::Yaml => "yaml",
        Language::Json => "json",
        Language::Xml => "xml",
        Language::Aql => "aql",
        Language::Hl7v2 => "hl7v2",
        Language::Plain => "plain",
    }
}

/// The documents every highlighter reads, each grammar's own and every
/// other's: multi-line strings and comments, tags, CDATA, unclosed and
/// malformed input, characters outside the BMP, a byte order mark, a next
/// line (U+0085) and the white space the two languages trim differently.
const CORPUS: &[&str] = &[
    "",
    " ",
    "\n",
    "\r\n",
    "\u{feff}{\"a\": 1}",
    "\u{85}key: value\u{85}",
    "\u{2028}a\u{2029}b\u{a0}c\u{3000}",
    "# a comment\nkey: value # trailing\n- item\n- 'single ''quoted'''\n- \"double \\\"escaped\\\"\"\n",
    "mapping:\n  nested: true\n  number: -12.5e+3\n  none: ~\n  anchor: &base\n  alias: *base\n  tag: !!str 42\n",
    "flow: [a, b, c]\nmap: {a: 1, b: 2}\nurl: http://example.org/a#b\ntime: 12:30\n",
    "multi: \"a string\n  that runs on\"\nnext: 'one\ntwo'\n",
    "- - nested\n  - list\n-\n- : odd\n\"quoted key\": v\n'k': v\nTRUE: False\nnull: NULL\n",
    "{\"name\": \"Ana\", \"age\": 42, \"ok\": true, \"none\": null, \"list\": [1, -2.5, 3e10, 1E-3]}",
    "{\n  \"text\": \"a string\nthat runs past its line\",\n  \"escaped\": \"q\\\"uote \\\\ \\n\"\n}",
    "{\"unclosed\": \"string",
    "{\"key\" : \"spaced key\", bare: word, 'single': 1, \"emoji\": \"\u{1f600}\u{1f468}\u{200d}\u{1f469}\"}",
    "[tru, nul, falsey, -, --1, 1.2.3, +4, .5]",
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE note>\n<!-- a comment\nthat runs over two lines -->\n<composition xmlns:x=\"urn:x\" archetype_node_id='openEHR-EHR-COMPOSITION.encounter.v1'>\n  <name>Ana &amp; Bo</name>\n  <![CDATA[raw <text> & more\nover lines ]]>\n  <x:empty/>\n</composition>\n",
    "<open attr=\"unclosed\n<broken <<>> </ > <a b=c d='e' f>text</a>",
    "<!-- unclosed comment\n<![CDATA[ unclosed cdata\n",
    "<a>\u{e9}\u{1f600}<\u{e9}l \u{e9}=\"\u{1f600}\">t</\u{e9}l></a>",
    "SELECT c/name/value AS name, o/data[at0001]/events[at0002]/data[at0003]/items[at0004]/value/magnitude\nFROM EHR e CONTAINS COMPOSITION c[openEHR-EHR-COMPOSITION.encounter.v1]\n  CONTAINS OBSERVATION o[openEHR-EHR-OBSERVATION.blood_pressure.v2]\nWHERE e/ehr_id/value = $ehr_id AND o/data/value > 140 -- a comment\nORDER BY name DESC LIMIT 10 OFFSET 5",
    "select distinct top 5 x from y where z matches {'a', \"b\"} and not exists q/r[name/value='a ]b'] or w like 'c%' xor 1.5e3",
    "SELECT [unclosed predicate and 'unclosed string\n/ // /a[ $ $1 _under \u{e9}l\u{e9}ment SELECTé",
    "MSH|^~\\&|SENDER|FAC|RECV|FAC|20261031120000||ADT^A01^ADT_A01|MSG0001|P|2.5.1\rEVN|A01|20261031120000\rPID|1||12345^^^HOSP^MR~67890^^^NAT^NI||Doe^Ana^^^Dr.||19850314|F\nPV1|1|I|W^389^1^UABH^^^^3||||12345^Doe^Ana\r\n",
    "FHS|#~\\&|A\rBHS^~\\&|B\rMSH|\rmsh|lower\rPI|short\rTOOLONG|x\r|empty id\rMSH\u{1f600}~\\&|emoji sep",
    "MSH|\t|tab\rMSH|^~\\&#|five seps\r",
];

#[tokio::test(flavor = "multi_thread")]
async fn the_highlight_tokens_of_the_editor_code_are_the_rust_tokens() -> Result<(), Box<dyn Error>>
{
    let mut cases = Vec::new();
    for language in Language::ALL {
        for document in CORPUS {
            let rust: Vec<Value> = tokens(language, document)
                .into_iter()
                .map(|token| json!({"kind": kind_name(token.kind), "text": token.text}))
                .collect();
            cases.push((json!([language_key(language), document]), json!(rust)));
        }
    }
    held(
        "highlight",
        "(m, language, text) => m.tokens(language, text)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_highlight_markup_of_the_editor_code_is_the_rust_markup() -> Result<(), Box<dyn Error>>
{
    let mut cases = Vec::new();
    for language in Language::ALL {
        for document in CORPUS {
            let rust: Vec<String> = lines(language, document)
                .iter()
                .map(|line| line_html(line))
                .collect();
            cases.push((json!([language_key(language), document]), json!(rust)));
        }
    }
    held(
        "highlight",
        "(m, language, text) => m.linesHtml(language, text)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_highlight_text_past_the_limit_is_plain_in_the_editor_code_too()
-> Result<(), Box<dyn Error>> {
    let limit = ferrohealth_ui::highlight::MAX_HIGHLIGHT_BYTES;
    // Each document reads as several tokens when it is highlighted, so a
    // limit one byte off draws it differently; the multi-byte ones are under
    // the limit in UTF-16 units and at it or past it in UTF-8 bytes.
    let number = |bytes: usize| format!("[1, {}]", "2".repeat(bytes - 5));
    let accented = |bytes: usize| format!("[\"{}\"]", "\u{e9}".repeat((bytes - 4) / 2));
    let documents = [
        number(limit),
        number(limit + 1),
        accented(limit),
        format!("{}a", accented(limit)),
        format!("[\"{}\"]", "\u{1f600}".repeat(limit / 4)),
    ];
    let mut cases = Vec::new();
    for document in &documents {
        for language in [Language::Json, Language::Yaml] {
            let rust: Vec<Value> = tokens(language, document)
                .into_iter()
                .map(|token| json!([kind_name(token.kind), token.text.len()]))
                .collect();
            cases.push((json!([language_key(language), document]), json!(rust)));
        }
    }
    held(
        "highlight",
        "(m, language, text) => m.tokens(language, text).map((t) => [t.kind, new TextEncoder().encode(t.text).length])",
        cases,
    )
    .await
}

/// The texts the editor's arithmetic reads: line breaks at either end,
/// characters outside the BMP, a combining mark, white space the two
/// languages trim differently.
const TEXTS: &[&str] = &[
    "",
    "\n",
    "\n\n",
    "a",
    "a\n",
    "ab\ncd\n\nef",
    "\u{1f600}x\n\u{e9}\u{301}y\n",
    "line one\r\nline two\r\n",
    "  {\"a\": 1}",
    "\n\t<a/>",
    "\u{feff}{}",
    "\u{feff}<a/>",
    "\u{85}[1]",
    "\u{a0}\u{2028}<x/>",
    "\u{3000}{",
    "plain [",
    "key: value",
];

/// The number of UTF-16 code units of `text`.
fn units(text: &str) -> u32 {
    u32::try_from(text.encode_utf16().count()).unwrap_or(u32::MAX)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_editor_caret_is_read_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for text in TEXTS {
        for offset in 0..=units(text).saturating_add(2) {
            let at = caret_at(text, offset);
            cases.push((
                json!([text, offset]),
                json!({"line": at.line, "column": at.column}),
            ));
        }
    }
    held(
        "editor",
        "(m, text, offset) => m.caretAt(text, offset)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_editor_offset_of_a_line_and_column_is_read_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for text in TEXTS {
        let lines = u32::try_from(text.split('\n').count()).unwrap_or(u32::MAX);
        let widest = text
            .split('\n')
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        let widest = u32::try_from(widest).unwrap_or(u32::MAX);
        for line in 0..=lines.saturating_add(1) {
            for column in 0..=widest.saturating_add(2) {
                let units = offset_of(text, Position { line, column });
                cases.push((
                    json!([text, {"line": line, "column": column}]),
                    json!(units),
                ));
            }
        }
    }
    held("editor", "(m, text, at) => m.offsetOf(text, at)", cases).await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_editor_grammar_and_the_code_line_count_are_read_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for text in TEXTS.iter().chain(CORPUS) {
        cases.push((
            json!([text]),
            json!([language_key(Language::detect(text)), line_count(text)]),
        ));
    }
    held(
        "editor",
        "(m, text) => [m.detect(text), m.lineCount(text)]",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_editor_marks_a_line_alike() -> Result<(), Box<dyn Error>> {
    let error = |line| LineMark {
        line,
        kind: MarkKind::Error,
    };
    let warning = |line| LineMark {
        line,
        kind: MarkKind::Warning,
    };
    let sets = [
        vec![],
        vec![error(1)],
        vec![warning(2)],
        vec![warning(1), error(1)],
        vec![error(1), warning(1)],
        vec![warning(3), warning(3), error(2)],
    ];
    let mut cases = Vec::new();
    for marks in &sets {
        for line in 0..=4 {
            let kind = mark_kind(marks, line);
            cases.push((
                json!([marks, line]),
                json!([kind, kind.map_or("", MarkKind::class)]),
            ));
        }
    }
    held(
        "editor",
        "(m, marks, line) => { const kind = m.kinds(marks).get(line) ?? null; return [kind, m.lineClass(kind, line, null)]; }",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_editor_panel_is_empty_alike() -> Result<(), Box<dyn Error>> {
    let english = Locale::english();
    let mut cases = Vec::new();
    for text in TEXTS
        .iter()
        .chain(&["\u{85}", "\u{feff}", "\u{a0}\u{2028}\u{3000}", " \t\r\n"])
    {
        let panel = editor_panel(Panel {
            editor: Editor {
                id: "parity".into(),
                language: Language::Yaml,
                text: (*text).to_owned(),
                marks: Vec::new(),
                editable: true,
            },
            label: String::from("Parity"),
            lead: String::from("YAML"),
            facts: Vec::new(),
            opened: Opened::New,
            caret: Position::default(),
            tools: None,
            end: None,
            words: EditorWords::of(&english),
        })
        .to_string();
        let empty = panel
            .split_once("fui-mapping-editor")
            .is_some_and(|(_, after)| after.starts_with(" is-empty"));
        cases.push((json!([text]), json!(empty)));
    }
    held("editor", "(m, text) => m.blank(text)", cases).await
}
