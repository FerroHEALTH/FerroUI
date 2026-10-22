// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Your preferences: exactly three tabs, each preference a plain form post
//! that works with no WebAssembly, Saved in its slot, and Reset to defaults
//! behind its confirmation.

use ferrohealth_ui::calendar::Weekday;
use ferrohealth_ui::i18n::Locale;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::theme::{Density, Hints, Language, Motion, Preferences, Theme, WeekStart};
use ferrohealth_ui::view::preferences::{
    Post, PreferenceRow, PreferencesPage, RESET_DIALOG, Section, SectionRoutes, SelectPreference,
    SwitchPreference, preference_row, preferences_page, select_preference, switch_preference,
};
use leptos::prelude::*;

use crate::markup::{has, tags};
use crate::render::page_html;

/// The routes of the three tabs.
fn routes() -> SectionRoutes {
    SectionRoutes {
        appearance: String::from("/preferences/appearance"),
        language: String::from("/preferences/language"),
        keyboard: String::from("/preferences/keyboard"),
    }
}

/// The page of `section` with `preferences` in force and `saved` just
/// kept.
fn page(section: Section, preferences: Preferences, saved: Option<&str>) -> PreferencesPage {
    PreferencesPage {
        section,
        routes: routes(),
        reading: false,
        preferences,
        action: String::from("/preference"),
        saved: saved.map(str::to_owned),
        accent: String::from("Indigo"),
        languages: vec![Locale::english()],
        browser: Locale::english(),
        week_from_language: Weekday::Monday,
        shortcuts: Some(String::from("/shortcuts")),
        reset: String::from("/preferences/reset"),
        own: Vec::new(),
    }
}

/// The HTML of `page`.
fn drawn(page: PreferencesPage) -> String {
    page_html(|| preferences_page(page))
}

#[test]
fn the_page_has_exactly_three_tabs_and_keyboard_carries_the_phone_hook() {
    let shown = drawn(page(Section::Language, Preferences::default(), None));
    let tabs = tags(&shown, "a")
        .into_iter()
        .filter(|tab| tab.get("class").is_some_and(|class| class == "fui-tab"))
        .collect::<Vec<_>>();
    let keys = tabs
        .iter()
        .filter_map(|tab| tab.get("data-tab").map(String::as_str))
        .collect::<Vec<_>>();
    assert_eq!(vec!["appearance", "language", "keyboard"], keys, "{shown}");
    assert!(
        has(
            &shown,
            "a",
            &[
                ("data-tab", "language"),
                ("href", "/preferences/language"),
                ("aria-current", "page"),
            ],
        ),
        "the tab the page is: {shown}"
    );
    assert!(shown.contains(">Language and region<"), "{shown}");
    assert!(shown.contains(">Your preferences<"), "{shown}");
    assert!(shown.contains("Only for you, in this browser."), "{shown}");
}

#[test]
fn ferroehr_calls_the_language_tab_reading() {
    let shown = drawn(PreferencesPage {
        reading: true,
        ..page(Section::Appearance, Preferences::default(), None)
    });
    assert!(shown.contains(">Reading<"), "{shown}");
    assert!(!shown.contains(">Language and region<"), "{shown}");
}

#[test]
fn appearance_draws_the_theme_and_density_as_radio_cards_marked_by_more_than_colour() {
    let preferences = Preferences {
        theme: Theme::Dark,
        density: Density::Compact,
        ..Preferences::default()
    };
    let shown = drawn(page(Section::Appearance, preferences, None));
    let radios = tags(&shown, "input")
        .into_iter()
        .filter(|input| input.get("type").is_some_and(|kind| kind == "radio"))
        .collect::<Vec<_>>();
    assert_eq!(5, radios.len(), "three themes and two densities: {shown}");
    assert!(
        has(
            &shown,
            "input",
            &[("name", "theme"), ("value", "dark"), ("checked", "")]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "input",
            &[("name", "density"), ("value", "compact"), ("checked", "")]
        ),
        "{shown}"
    );
    let checked = radios
        .iter()
        .filter(|radio| radio.contains_key("checked"))
        .count();
    assert_eq!(2, checked, "one choice of each: {shown}");
    assert_eq!(
        2,
        shown
            .matches(r#"<fieldset class="fui-choice-cards">"#)
            .count(),
        "each set of cards is a radio group: {shown}"
    );
    assert!(
        shown.contains(Glyph::CHECK.markup()),
        "a check marks the card in force: {shown}"
    );
    assert!(shown.contains("Indigo on white"), "{shown}");
    assert!(shown.contains("Indigo on ink"), "{shown}");
    assert!(shown.contains("fui-preview-split"), "{shown}");
    assert!(
        has(&shown, "select", &[("id", "motion"), ("name", "motion")]),
        "{shown}"
    );
    assert!(
        has(&shown, "label", &[("for", "motion")]),
        "the label names the select: {shown}"
    );
}

#[test]
fn every_preference_is_a_plain_form_post_with_a_save_button_before_hydration() {
    let shown = drawn(page(Section::Appearance, Preferences::default(), None));
    let forms = tags(&shown, "form")
        .into_iter()
        .filter(|form| {
            form.get("class")
                .is_some_and(|class| class == "fui-preference-form")
        })
        .collect::<Vec<_>>();
    assert_eq!(3, forms.len(), "theme, density and motion: {shown}");
    for form in &forms {
        assert_eq!(Some("post"), form.get("method").map(String::as_str));
        assert_eq!(Some("/preference"), form.get("action").map(String::as_str));
    }
    assert_eq!(
        4,
        tags(&shown, "input")
            .iter()
            .filter(|input| {
                input.get("name").is_some_and(|name| name == "back")
                    && input
                        .get("value")
                        .is_some_and(|value| value == "/preferences/appearance")
            })
            .count(),
        "the three preferences and the reset return to the tab: {shown}"
    );
    let saves = tags(&shown, "button")
        .into_iter()
        .filter(|button| button.get("type").is_some_and(|kind| kind == "submit"))
        .filter(|button| !button.contains_key("hidden"))
        .count();
    assert!(
        saves >= 3,
        "a Save button posts each before the page hydrates: {shown}"
    );
    assert!(!shown.contains("style="), "{shown}");
}

#[test]
fn saved_marks_only_the_row_just_kept_in_the_slot_before_its_control() {
    let shown = drawn(page(
        Section::Appearance,
        Preferences::default(),
        Some("density"),
    ));
    assert_eq!(1, shown.matches(r#"role="status""#).count(), "{shown}");
    let density = shown.find("Compact fits more rows").unwrap();
    let mark = shown.find("fui-setting-saved").unwrap();
    let cards = shown[density..].find("fui-choice-cards").unwrap() + density;
    assert!(
        density < mark && mark < cards,
        "Saved sits in the density row, before its control: {shown}"
    );
    let quiet = drawn(page(Section::Appearance, Preferences::default(), None));
    assert!(!quiet.contains("fui-setting-saved"), "{quiet}");
}

#[test]
fn language_and_region_offers_the_language_and_the_first_day_of_the_week() {
    let preferences = Preferences {
        language: Language::Chosen(String::from("en")),
        week_start: WeekStart::Sunday,
        ..Preferences::default()
    };
    let shown = drawn(page(Section::Language, preferences, None));
    assert!(
        has(
            &shown,
            "input",
            &[("name", "language"), ("value", "browser")]
        ),
        "{shown}"
    );
    assert!(
        has(&shown, "input", &[("name", "language"), ("value", "en")]),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "button",
            &[("lang", "en"), ("aria-selected", "true")]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "select",
            &[("id", "week-start"), ("name", "week-start")]
        ),
        "{shown}"
    );
    assert!(
        shown.contains("Monday, from the language (default)"),
        "{shown}"
    );
    assert!(
        has(&shown, "option", &[("value", "sunday"), ("selected", "")]),
        "{shown}"
    );
}

#[test]
fn keyboard_offers_the_hints_and_the_legend() {
    let preferences = Preferences {
        hints: Hints::Never,
        motion: Motion::Reduce,
        ..Preferences::default()
    };
    let shown = drawn(page(Section::Keyboard, preferences, None));
    assert!(
        has(&shown, "option", &[("value", "never"), ("selected", "")]),
        "{shown}"
    );
    assert_eq!(
        3,
        tags(&shown, "option").len(),
        "wide, always and never: {shown}"
    );
    assert!(
        has(
            &shown,
            "a",
            &[("href", "/shortcuts"), ("aria-keyshortcuts", "?")]
        ),
        "{shown}"
    );
    let none = drawn(PreferencesPage {
        shortcuts: None,
        ..page(Section::Keyboard, Preferences::default(), None)
    });
    assert!(!none.contains("/shortcuts"), "{none}");
}

#[test]
fn a_console_s_own_rows_follow_the_library_s_in_their_tab() {
    let post = Post {
        action: String::from("/preference"),
        back: String::from("/preferences/appearance"),
    };
    let shown = page_html(|| {
        let reveal = select_preference(SelectPreference {
            name: String::from("reveal"),
            choices: vec![
                (String::from("10"), String::from("10 seconds")),
                (String::from("15"), String::from("15 seconds")),
            ],
            current: String::from("15"),
            post,
            saved: false,
        });
        let own = preference_row(PreferenceRow {
            label: String::from("Revealed identifiers stay for <b>"),
            description: String::from("The server caps every reveal."),
            control_id: Some(String::from("reveal")),
            control: reveal,
            saved: false,
        });
        preferences_page(PreferencesPage {
            own: vec![own],
            ..page(Section::Appearance, Preferences::default(), None)
        })
    });
    let motion = shown.find("Reduce motion").unwrap();
    let reveal = shown.find("Revealed identifiers stay for").unwrap();
    assert!(motion < reveal, "{shown}");
    assert!(shown.contains("stay for &lt;b&gt;"), "escaped: {shown}");
    assert!(
        has(&shown, "option", &[("value", "15"), ("selected", "")]),
        "{shown}"
    );
}

#[test]
fn reset_to_defaults_asks_first_and_says_what_it_leaves_alone() {
    let shown = drawn(page(Section::Keyboard, Preferences::default(), None));
    assert!(
        has(&shown, "button", &[("popovertarget", RESET_DIALOG)]),
        "{shown}"
    );
    assert!(shown.contains("Reset to defaults…"), "{shown}");
    assert!(
        has(
            &shown,
            "div",
            &[
                ("id", RESET_DIALOG),
                ("popover", "auto"),
                ("role", "dialog")
            ]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "form",
            &[("method", "post"), ("action", "/preferences/reset")]
        ),
        "{shown}"
    );
    assert!(
        has(
            &shown,
            "input",
            &[("name", "back"), ("value", "/preferences/keyboard")]
        ),
        "{shown}"
    );
    assert!(shown.contains("are not touched"), "{shown}");
    assert!(
        shown.contains("Kept in this browser as cookies"),
        "the foot says where they are kept: {shown}"
    );
}

#[test]
fn the_accent_the_cards_name_is_escaped() {
    let shown = drawn(PreferencesPage {
        accent: String::from("<script>"),
        ..page(Section::Appearance, Preferences::default(), None)
    });
    assert!(shown.contains("&lt;script&gt; on white"), "{shown}");
    assert!(!shown.contains("<script>"), "{shown}");
}

#[test]
fn a_select_preference_alone_posts_its_name_and_value() {
    let shown = page_html(|| {
        select_preference(SelectPreference {
            name: String::from("hide"),
            choices: vec![(String::from("on"), String::from("On"))],
            current: String::from("on"),
            post: Post {
                action: String::from("/preference"),
                back: String::from("/preferences/appearance"),
            },
            saved: true,
        })
        .into_any()
    });
    assert!(
        has(
            &shown,
            "form",
            &[("method", "post"), ("action", "/preference")]
        ),
        "{shown}"
    );
    assert!(
        has(&shown, "select", &[("name", "hide"), ("id", "hide")]),
        "{shown}"
    );
}

#[test]
fn a_switch_preference_posts_off_unless_it_is_on() {
    let shown = page_html(|| {
        switch_preference(SwitchPreference {
            name: String::from("hide"),
            label: String::from("Hide revealed identifiers when I leave the page"),
            on: true,
            post: Post {
                action: String::from("/preference"),
                back: String::from("/preferences/appearance"),
            },
            saved: false,
        })
    });
    let hide = tags(&shown, "input")
        .into_iter()
        .filter(|input| input.get("name").is_some_and(|name| name == "hide"))
        .collect::<Vec<_>>();
    assert_eq!(2, hide.len(), "{shown}");
    assert_eq!(Some("hidden"), hide[0].get("type").map(String::as_str));
    assert_eq!(Some("off"), hide[0].get("value").map(String::as_str));
    assert_eq!(Some("checkbox"), hide[1].get("type").map(String::as_str));
    assert!(hide[1].contains_key("checked"), "{shown}");
    assert!(
        has(
            &shown,
            "input",
            &[(
                "aria-label",
                "Hide revealed identifiers when I leave the page"
            )]
        ),
        "{shown}"
    );
}
