// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The stylesheet's own rules: every class a component writes is defined in
//! it, so a console needs no scan of this crate; every size of type names a
//! step of the scale; and the layer order is declared before anything else.

use std::collections::BTreeSet;
use std::path::Path;

use ferrohealth_ui::style::{STYLESHEET, TAILWIND_THEME};

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
    // NOTE: no specification governs this: our own design; a size of 0 sets no type, it takes
    // the words of a pill cut to its shape off the line and leaves them its accessible name.
    let off_scale: Vec<&str> = STYLESHEET
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("font-size:"))
        .filter(|line| !line.contains("var(--text-") && !line.contains("inherit"))
        .filter(|line| *line != "font-size: 0;")
        .collect();
    assert!(off_scale.is_empty(), "sizes off the scale: {off_scale:?}");
}

#[test]
fn every_step_of_type_is_written_with_its_leading() {
    let mut unpaired = Vec::new();
    for rule in STYLESHEET.split('}') {
        let body = rule.rsplit('{').next().unwrap_or_default();
        if body.contains("font-size: var(--text-") && !body.contains("line-height:") {
            unpaired.push(body.trim().lines().next().unwrap_or_default().to_owned());
        }
    }
    assert!(unpaired.is_empty(), "a step with no leading: {unpaired:?}");
    for step in ["display", "title", "heading", "body", "small", "micro"] {
        assert!(
            STYLESHEET.contains(&format!("--leading-{step}:")),
            "the step {step} has no leading token"
        );
    }
}

#[test]
fn the_tailwind_theme_writes_no_variable_that_names_itself() {
    // NOTE: CSS Custom Properties Level 1, section 2.3; a reference block is
    // inlined into each utility, so a same-named entry is never written out.
    let reference = TAILWIND_THEME.contains("@theme inline reference {");
    let cycles: Vec<&str> = TAILWIND_THEME
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.split_once(':')
                .is_some_and(|(name, value)| value.trim() == format!("var({});", name.trim()))
        })
        .collect();
    assert!(
        reference || cycles.is_empty(),
        "entries that would unset themselves: {cycles:?}"
    );
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

#[test]
fn the_sticky_top_bar_pads_the_scroll_so_a_focused_control_clears_it() {
    let rule = "@media (width >= 48rem) {\n    html {\n      scroll-padding-top: calc(var(--bar)";
    assert!(
        STYLESHEET.contains(rule),
        "no scroll padding beside the sticky top bar"
    );
}

#[test]
fn a_fieldset_well_keeps_its_frame_after_the_fieldset_reset() {
    let reset = STYLESHEET
        .find("  .fui-fieldset {")
        .expect("the fieldset reset");
    let well = STYLESHEET
        .find("  .fui-fieldset.fui-well {")
        .expect("the fieldset well");
    assert!(well > reset, "the well rule must follow the reset to win");
}

#[test]
fn a_disabled_control_keeps_the_control_edge_on_a_sunken_ground() {
    // WCAG 2.2 SC 1.4.11 exempts a disabled control, but the field must still read as one on a
    // well, whose ground is the disabled control's own.
    let rule = "  :is(.fui-input, .fui-select, .fui-textarea):disabled {\n    border-color: var(--edge-control);";
    assert!(
        STYLESHEET.contains(rule),
        "a disabled control loses its edge"
    );
    let hover = "  :is(.fui-input, .fui-select, .fui-textarea):disabled:hover {\n    border-color: var(--edge-control);";
    assert!(
        STYLESHEET.contains(hover),
        "a disabled control loses its edge under the pointer"
    );
}

#[test]
fn a_timestamp_never_breaks_and_the_compact_stat_cuts_its_label() {
    let time = "  .fui-time {\n    font-variant-numeric: tabular-nums;\n    overflow-wrap: normal;\n    white-space: nowrap;";
    assert!(STYLESHEET.contains(time), "a timestamp can break");
    let compact = STYLESHEET
        .find("  @media (width < 30rem) {\n    .fui-stat {")
        .expect("the compact stat tile");
    let label = STYLESHEET
        .get(compact..)
        .and_then(|rest| rest.find("text-overflow: ellipsis;"))
        .expect("the compact label's ellipsis");
    assert!(label < 1200, "the ellipsis belongs to the compact tile");
}

#[test]
fn a_table_of_several_bodies_keeps_the_edges_and_spacing_between_them() {
    assert!(
        STYLESHEET.contains("  .fui-table tbody:last-of-type tr:last-child > * {"),
        "a body's last row loses its edge before the next body"
    );
    assert!(
        STYLESHEET.contains(
            "    .fui-table-stack .fui-table tbody tr + tr,\n    .fui-table-stack .fui-table tbody + tbody {"
        ),
        "the stacked bodies touch"
    );
}

#[test]
fn a_field_and_its_button_take_the_whole_row_below_30rem() {
    let rule = "  @media (width < 30rem) {\n    .fui-field-flow > :is(.fui-field, button, .fui-button) {\n      flex: 1 1 100%;";
    assert!(
        STYLESHEET.contains(rule),
        "a field flow keeps its row on a phone"
    );
}

#[test]
fn a_button_inside_a_field_keeps_its_own_width_at_the_start() {
    assert!(
        STYLESHEET.contains("  .fui-field > :is(button, .fui-button) {\n    justify-self: start;"),
        "a button in a field's grid would stretch across it"
    );
}

#[test]
fn an_island_wrapper_draws_no_box_of_its_own() {
    assert!(
        STYLESHEET.contains("  leptos-island,\n  leptos-children {\n    display: contents;"),
        "an island's wrapper would break the flex or grid it sits in"
    );
}

#[test]
fn a_cards_rhythm_reaches_through_an_islands_wrapper() {
    assert!(
        STYLESHEET.contains(
            "  .fui-card > * + *,\n  .fui-card > :where(leptos-island, leptos-children) > * + *,\n  .fui-card > * + :where(leptos-island, leptos-children) > :where(:first-child) {"
        ),
        "the parts an island draws in a card would lose the space between them"
    );
}
