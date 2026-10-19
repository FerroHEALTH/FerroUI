// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The start tags of rendered HTML read by attribute name, so a render test
//! asserts what an element carries and never the order Leptos writes it in
//! (Leptos writes `class` last). Enough of the HTML Living Standard's
//! tokenizer for the library's own output: quoted values and bare names.

use std::collections::BTreeMap;

/// The attributes of one start tag, by name; a bare attribute holds `""`.
pub(crate) type Attributes = BTreeMap<String, String>;

/// The attributes of every `<tag>` start tag in `html`, in document order.
pub(crate) fn tags(html: &str, tag: &str) -> Vec<Attributes> {
    let open = format!("<{tag}");
    let mut found = Vec::new();
    let mut rest = html;
    while let Some(at) = rest.find(&open) {
        rest = &rest[at + open.len()..];
        if !rest.starts_with([' ', '>', '/']) {
            continue;
        }
        let (attributes, after) = attributes(rest);
        found.push(attributes);
        rest = after;
    }
    found
}

/// The attributes at the head of `text` up to the end of the start tag, and
/// the text after it.
fn attributes(mut text: &str) -> (Attributes, &str) {
    let mut read = Attributes::new();
    loop {
        text = text.trim_start_matches([' ', '/']);
        if let Some(after) = text.strip_prefix('>') {
            return (read, after);
        }
        if text.is_empty() {
            return (read, text);
        }
        let end = text.find(['=', ' ', '>', '/']).unwrap_or(text.len());
        let name = text[..end].to_owned();
        text = &text[end..];
        if let Some(quoted) = text.strip_prefix("=\"") {
            let close = quoted.find('"').unwrap_or(quoted.len());
            read.insert(name, quoted[..close].to_owned());
            text = quoted.get(close + 1..).unwrap_or("");
        } else {
            read.insert(name, String::new());
        }
    }
}

/// Whether some `<tag>` in `html` carries every attribute of `wanted` with
/// its value, whatever else it carries and in whatever order.
pub(crate) fn has(html: &str, tag: &str, wanted: &[(&str, &str)]) -> bool {
    tags(html, tag).iter().any(|attributes| {
        wanted
            .iter()
            .all(|(name, value)| attributes.get(*name).map(String::as_str) == Some(*value))
    })
}

#[test]
fn a_start_tag_reads_its_attributes_by_name_whatever_their_order() {
    let html = r#"<p><th class="a b" scope="col" hidden>x</th><thead></thead><th aria-sort="none" scope="col"></th></p>"#;
    let read = tags(html, "th");
    assert_eq!(2, read.len(), "a <thead> is not a <th>: {read:?}");
    assert_eq!(Some("a b"), read[0].get("class").map(String::as_str));
    assert_eq!(Some(""), read[0].get("hidden").map(String::as_str));
    assert!(has(html, "th", &[("scope", "col"), ("class", "a b")]));
    assert!(has(html, "th", &[("aria-sort", "none"), ("scope", "col")]));
    assert!(!has(html, "th", &[("aria-sort", "none"), ("class", "a b")]));
}
