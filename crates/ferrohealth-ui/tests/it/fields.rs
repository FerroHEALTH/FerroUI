// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tag (R2), the field with a part inside its edge (R10), and the date
//! field and range with their picker (R22, R23, R25), as the server draws
//! them: what each says, what each never carries, the escaping of what it
//! is given, and the stylesheet rules that hold their layout.

use ferrohealth_ui::calendar::{Bounds, Day, Preset, Weekday};
use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::Phrase;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::STYLESHEET;
use ferrohealth_ui::view::date::{
    DateField, DateRange, DateWords, RangeEnd, RangePreset, date_field, date_range,
};
use ferrohealth_ui::view::form::{
    Field, INNER_BUTTON, INPUT, Inner, described_by, described_with_unit, masked_input, unit_id,
};
use ferrohealth_ui::view::masked::HoldWords;
use ferrohealth_ui::view::tag::{Tag, tag};

use crate::markup::{has, tags};
use crate::render::{en, html};

fn day(text: &str) -> Day {
    Day::parse(text).expect("a test day")
}

/// An audit's From field, which forbids the future.
fn audit_from(value: &str, error: Option<&str>) -> DateField {
    DateField {
        id: String::from("audit-from"),
        name: String::from("from"),
        label: String::from("From <b>"),
        value: value.to_owned(),
        help: Some(String::from("The first day of actions to show.")),
        error: error.map(str::to_owned),
        today: day("2026-10-21"),
        bounds: Bounds {
            earliest: None,
            latest: Some(day("2026-10-21")),
        },
        first_day: Weekday::Monday,
        with_time: false,
        quick: Vec::new(),
        chosen: None,
        words: DateWords::of(&en(), false),
    }
}

#[test]
fn a_tag_is_its_words_on_one_line_with_a_quiet_code_and_a_hidden_glyph() {
    let plain = html(tag("FHIR R4"));
    assert!(has(&plain, "span", &[("class", "fui-tag")]), "{plain}");
    assert!(plain.contains(">FHIR R4</span>"), "{plain}");
    let coded = html(Tag::new("Read").code("r").view());
    assert!(
        coded.contains(r#"Read<span class="fui-tag-code">r</span>"#),
        "the raw code follows the word: {coded}"
    );
    let locked = html({
        Tag::new("Directory")
            .locked("From the care services directory")
            .view()
    });
    assert!(
        has(
            &locked,
            "span",
            &[
                ("class", "fui-tag fui-tag-locked"),
                ("title", "From the care services directory")
            ]
        ),
        "{locked}"
    );
    assert!(
        has(&locked, "svg", &[("aria-hidden", "true")]),
        "the lock is hidden from assistive technology: {locked}"
    );
    let mono = html(Tag::new("1.1.0").mono().glyph(Glyph::TAGS).view());
    assert!(
        has(&mono, "span", &[("class", "fui-tag fui-tag-mono")]),
        "{mono}"
    );
    let escaped = html(Tag::new("<script>").code("a&b").view());
    assert!(escaped.contains("&#60;script&#62;"), "{escaped}");
    assert!(escaped.contains("a&#38;b"), "{escaped}");
    assert!(!escaped.contains("style="), "{escaped}");
}

#[test]
fn a_tag_keeps_to_one_line_while_it_fits_and_never_widens_the_page() {
    // NOTE: WCAG 2.2 SC 1.4.10: a tag holding a canonical URL or an OID wraps inside its
    // box; in a row it keeps its width, as a word that never broke did.
    let rule = STYLESHEET
        .split("\n  .fui-tag {\n")
        .nth(1)
        .and_then(|rest| rest.split("\n  }").next())
        .expect("the tag's rule");
    for declaration in [
        "flex-shrink: 0;",
        "max-width: 100%;",
        "white-space: normal;",
        "overflow-wrap: anywhere;",
    ] {
        assert!(
            rule.contains(declaration),
            "the tag lacks {declaration}: {rule}"
        );
    }
    assert!(!rule.contains("nowrap"), "the tag may wrap: {rule}");
}

#[test]
fn a_field_holds_its_unit_inside_its_edge_and_names_it_in_its_description() {
    let page = html({
        let id = "timeout";
        let input = Html::raw(format!(
            r#"<input id="{id}" name="timeout" inputmode="numeric" aria-describedby="{}" class="{INPUT}">"#,
            described_with_unit(id, true, false)
        ));
        Field::new(id, "Timeout")
            .help("Below the overall budget.")
            .trailing(Inner::Unit(String::from("ms")))
            .view(input)
    });
    assert!(has(&page, "div", &[("class", "fui-input-group")]), "{page}");
    assert!(
        has(
            &page,
            "span",
            &[("id", &unit_id("timeout")), ("class", "fui-input-suffix")]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "input",
            &[("aria-describedby", "timeout-unit timeout-help")]
        ),
        "the unit is read with the value, before the help: {page}"
    );
    let before = page.find("<input").expect("an input");
    let suffix = page.find("fui-input-suffix").expect("a suffix");
    assert!(before < suffix, "the suffix follows the typed text");
}

#[test]
fn a_field_holds_a_prefix_and_a_named_button_inside_its_edge() {
    let page = html({
        let input = Html::raw(format!(
            r#"<input id="base" name="base" type="password" class="{INPUT}">"#
        ));
        let reveal = Html::raw(format!(
            r#"<button type="button" aria-label="Show the secret" aria-pressed="false" class="{INNER_BUTTON}">o</button>"#
        ));
        Field::new("base", "Secret")
            .leading(Inner::Text(String::from("https://")))
            .trailing(Inner::Button(reveal))
            .view(input)
    });
    assert!(
        has(&page, "span", &[("class", "fui-input-prefix")]),
        "{page}"
    );
    assert!(
        has(
            &page,
            "button",
            &[("class", INNER_BUTTON), ("aria-label", "Show the secret")]
        ),
        "an inner button keeps its name: {page}"
    );
    let plain = html(
        Field::new("plain", "Plain")
            .view(Html::raw(format!(r#"<input id="plain" class="{INPUT}">"#))),
    );
    assert!(
        !plain.contains("fui-input-group"),
        "no part, no group: {plain}"
    );
}

/// The masked identifier field with its hold-to-show eye, as a page draws
/// it.
fn masked_field() -> String {
    html(
        Field::new("identifier", "Identifier")
            .help("Nine digits.")
            .trailing(Inner::HoldToShow(
                HoldWords::of(&en()).named("Show the identifier while held"),
            ))
            .view(masked_input(
                "identifier",
                "identifier",
                described_by("identifier", true, false),
            )),
    )
}

#[test]
fn a_masked_field_draws_its_input_masked_with_no_autocomplete_capitals_or_spelling() {
    let page = masked_field();
    assert!(
        has(
            &page,
            "input",
            &[
                ("id", "identifier"),
                ("name", "identifier"),
                ("type", "password"),
                ("autocomplete", "off"),
                ("autocapitalize", "off"),
                ("spellcheck", "false"),
                ("aria-describedby", "identifier-help"),
            ]
        ),
        "{page}"
    );
    assert!(
        tags(&page, "input")
            .iter()
            .all(|input| !input.contains_key("value")),
        "the masked input carries no value: {page}"
    );
}

#[test]
fn the_eye_is_disabled_and_says_the_field_stays_masked_until_its_module_runs() {
    // WAI-ARIA 1.2 `button`: a toggle button says whether it is pressed.
    let page = masked_field();
    assert!(
        has(
            &page,
            "button",
            &[
                ("type", "button"),
                ("aria-pressed", "false"),
                ("disabled", ""),
                (
                    "aria-label",
                    "The text stays masked: showing it works once the page has loaded"
                ),
            ]
        ),
        "{page}"
    );
    assert!(
        page.contains("fui-input-hold"),
        "the eye is the hold button: {page}"
    );
}

#[test]
fn the_masked_field_writes_no_props_into_the_page() {
    // NOTE: no specification governs this: our own design; no typed value travels outside
    // its input, so the page carries no copy of it as data.
    let page = masked_field();
    assert!(!page.contains("data-props"), "{page}");
}

#[test]
fn an_inner_button_keeps_the_groups_target_size() {
    // WCAG 2.2 SC 2.5.8: the masked input's eye is the same 30 px target as any inner button.
    for rule in [
        "  .fui-input-group > .fui-input-toggle {\n    width: 1.875rem;\n    height: 1.875rem;",
        "  .fui-input-group > .fui-input:has(+ :is(.fui-input-suffix, .fui-input-toggle)) {",
        "  .fui-input-hold {\n    touch-action: none;",
    ] {
        assert!(STYLESHEET.contains(rule), "missing: {rule}");
    }
}

#[test]
fn the_group_draws_the_fields_one_edge_and_the_focus_ring_round_its_parts() {
    // WCAG 2.2 SC 2.4.7 and SC 2.4.13: the ring goes round the field the typed text is in.
    for rule in [
        "  .fui-input-group:focus-within {\n    border-color: var(--focus);\n    outline: 2px solid var(--focus);",
        "  .fui-input-group > .fui-input {\n    flex: 1 1 auto;\n    align-self: stretch;\n    min-height: calc(var(--control) - 2px);\n    border: 0;",
        "  .fui-input-group > .fui-input:focus-visible {\n    outline: 0;",
        "  .fui-input-group:has(> .fui-input[aria-invalid=\"true\"]) {\n    border-color: var(--bad);",
        "  .fui-input-group > .fui-input-toggle {\n    width: 1.875rem;\n    height: 1.875rem;",
    ] {
        assert!(STYLESHEET.contains(rule), "missing: {rule}");
    }
    assert!(
        !STYLESHEET.contains("  .fui-input-suffix {\n    position: absolute;"),
        "a suffix never lies over the typed text"
    );
}

#[test]
fn the_server_draws_the_date_field_closed_with_its_typed_value_posting() {
    let page = html(date_field(audit_from("2026-10-14", None)));
    let input = tags(&page, "input")
        .into_iter()
        .find(|input| input.get("id").map(String::as_str) == Some("audit-from"))
        .expect("the field's input");
    assert_eq!(
        Some("from"),
        input.get("name").map(String::as_str),
        "{page}"
    );
    assert_eq!(Some("2026-10-14"), input.get("value").map(String::as_str));
    assert_eq!(Some("text"), input.get("type").map(String::as_str));
    assert_eq!(
        Some("YYYY-MM-DD"),
        input.get("placeholder").map(String::as_str)
    );
    assert_eq!(
        Some("audit-from-help audit-from-error"),
        input.get("aria-describedby").map(String::as_str)
    );
    assert!(!input.contains_key("aria-invalid"), "{page}");
    assert!(
        page.contains(r#"<span class="fui-muted">(YYYY-MM-DD)</span>"#),
        "the format is stated after the label: {page}"
    );
    assert!(
        page.contains("From &#60;b&#62;"),
        "the label is escaped: {page}"
    );
    let button = tags(&page, "button")
        .into_iter()
        .find(|button| button.get("aria-haspopup").map(String::as_str) == Some("dialog"))
        .expect("the calendar button");
    assert!(
        button.contains_key("disabled"),
        "disabled until its module runs: {page}"
    );
    assert_eq!(
        Some("Type the date: the calendar opens once the page has loaded"),
        button.get("aria-label").map(String::as_str),
        "its name says to type the date"
    );
    assert_eq!(
        Some("false"),
        button.get("aria-expanded").map(String::as_str)
    );
    assert!(
        !page.contains("fui-datepicker"),
        "the picker is closed: {page}"
    );
    assert!(!page.contains("style="), "{page}");
}

#[test]
fn a_refused_value_marks_the_field_invalid_and_says_why() {
    let reason = "2026-10-41 is not a date: October has 31 days.";
    let page = html(date_field(audit_from("2026-10-41", Some(reason))));
    assert!(
        has(
            &page,
            "input",
            &[("id", "audit-from"), ("aria-invalid", "true")]
        ),
        "{page}"
    );
    assert!(page.contains(reason), "{page}");
    assert!(
        has(
            &page,
            "p",
            &[("id", "audit-from-error"), ("aria-live", "polite")]
        ),
        "{page}"
    );
}

#[test]
fn the_range_shows_its_from_and_to_fields_until_its_module_runs() {
    let range = DateRange {
        id: String::from("audit-range"),
        label: String::from("Actions between"),
        from: RangeEnd {
            id: String::from("audit-range-from"),
            name: String::from("from"),
            label: String::from("From"),
            value: String::from("2026-10-15"),
        },
        to: RangeEnd {
            id: String::from("audit-range-to"),
            name: String::from("to"),
            label: String::from("To"),
            value: String::from("2026-10-21"),
        },
        today: day("2026-10-21"),
        bounds: Bounds {
            earliest: None,
            latest: Some(day("2026-10-21")),
        },
        first_day: Weekday::Monday,
        presets: vec![RangePreset::of(&en(), Preset::LastDays(7))],
        words: DateWords::of(&en(), false),
    };
    let page = html(date_range(range));
    assert!(
        has(&page, "div", &[("class", "fui-date-range-fields")]),
        "{page}"
    );
    let fields = tags(&page, "div")
        .into_iter()
        .find(|div| div.get("class").map(String::as_str) == Some("fui-date-range-fields"))
        .expect("the fields");
    assert!(!fields.contains_key("hidden"), "the fields show: {page}");
    assert!(
        has(&page, "input", &[("name", "from"), ("value", "2026-10-15")]),
        "{page}"
    );
    assert!(
        has(&page, "input", &[("name", "to"), ("value", "2026-10-21")]),
        "{page}"
    );
    let button = tags(&page, "button")
        .into_iter()
        .find(|button| button.get("aria-haspopup").map(String::as_str) == Some("dialog"))
        .expect("the range button");
    assert!(
        button.contains_key("hidden"),
        "the button waits for its module: {page}"
    );
    assert!(
        page.contains("2026-10-15 to 2026-10-21"),
        "the button names the range: {page}"
    );
}

#[test]
fn a_preset_names_the_range_it_fills() {
    let words = RangePreset::of(&en(), Preset::LastDays(7)).words;
    assert_eq!("Last 7 days", words);
    let (from, to) = Preset::LastDays(7)
        .range(day("2026-10-21"))
        .expect("a range");
    assert_eq!(7, from.days_through(to), "Last 7 days holds 7 days");
}

#[test]
fn a_consoles_sentence_takes_the_chosen_days_name() {
    let sentence = Phrase::new(format!(
        "Back in the queue on {}.",
        Phrase::open_mark("day")
    ));
    assert_eq!(
        "Back in the queue on Monday 26 October 2026.",
        sentence.fill(&[("day", "Monday 26 October 2026")])
    );
}

#[test]
fn the_phone_sheet_shows_one_month_with_both_arrows_and_centred_buttons() {
    for rule in [
        "  .fui-datepicker-phone-prev {\n    visibility: hidden;",
        "    .fui-datepicker-pair .fui-datepicker-phone-prev {\n      visibility: visible;",
        "    .fui-datepicker-pair > .fui-datepicker-month:first-child {\n      display: none;",
        "    .fui-datepicker-foot .fui-button {\n      min-height: 2.75rem;\n      align-items: center;",
        "    .fui-date-field > .fui-datepicker,\n    .fui-date-field > .fui-datepicker.is-end,\n    .fui-date-field > .fui-datepicker.fui-datepicker-range,\n    .fui-date-field > .fui-datepicker.fui-datepicker-range.is-start {\n      position: fixed;\n      inset: auto 0 0 0;",
    ] {
        assert!(STYLESHEET.contains(rule), "missing: {rule}");
    }
}
