// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The stylesheet's own rules: every class a component writes is defined in
//! it, so a console needs no scan of this crate; every size of type names a
//! step of the scale; and the layer order is declared before anything else.

use std::collections::BTreeSet;
use std::path::Path;

use ferrohealth_ui::style::STYLESHEET;

/// Every `fui-` class name in `text`.
fn classes(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut rest = text;
    while let Some(at) = rest.find("fui-") {
        let tail = &rest[at..];
        let end = tail
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(tail.len());
        found.insert(tail[..end].trim_end_matches('-').to_owned());
        rest = &tail[end..];
    }
    found
}

/// The text of every Rust source file under `dir`.
fn sources(dir: &Path) -> String {
    let mut text = String::new();
    for entry in std::fs::read_dir(dir).expect("a source folder") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            text.push_str(&sources(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            text.push_str(&std::fs::read_to_string(&path).expect("a source file"));
        }
    }
    text
}

#[test]
fn every_class_a_component_writes_is_defined_in_the_stylesheet() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let written = classes(&sources(&src));
    let defined: BTreeSet<String> = classes(STYLESHEET)
        .into_iter()
        .filter(|class| STYLESHEET.contains(&format!(".{class}")))
        .collect();
    let missing: Vec<&String> = written
        .iter()
        .filter(|class| !defined.contains(*class) && class.as_str() != "fui")
        .collect();
    assert!(
        missing.is_empty(),
        "classes written but not defined: {missing:?}"
    );
}

#[test]
fn every_size_of_type_names_a_step_of_the_scale() {
    let off_scale: Vec<&str> = STYLESHEET
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("font-size:"))
        .filter(|line| !line.contains("var(--text-") && !line.contains("inherit"))
        .collect();
    assert!(off_scale.is_empty(), "sizes off the scale: {off_scale:?}");
}

#[test]
fn the_layer_order_comes_first_and_no_style_is_inline() {
    let first_rule = STYLESHEET
        .lines()
        .find(|line| line.starts_with('@'))
        .expect("an at-rule");
    assert_eq!("@layer theme, base, components, utilities;", first_rule);
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let code: String = sources(&src)
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code.contains(" style="),
        "a component writes a style attribute"
    );
    assert!(
        !code.contains("<style"),
        "a component writes a style element"
    );
}
