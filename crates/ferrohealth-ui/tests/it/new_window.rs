// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! An action whose answer opens in a new window (#685): a plain form post
//! with `target="_blank"` and `rel="noopener"` (HTML Living Standard, form
//! submission and the rules for choosing a navigable), as a menu item and
//! as an action button, saying so after its words (WCAG 2.2 technique
//! G201).

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::view::action::action_button;
use ferrohealth_ui::view::action_menu::{Act, Action, Item, Menu, more_menu};
use ferrohealth_ui::view::button::Variant;

use crate::markup::{has, tags};
use crate::render::{en, html};

/// The launch of an app, posting the record it opens on.
fn launch() -> Action {
    Action::new(
        Some(Glyph::PLAY),
        "Launch Growth chart",
        Act::PostToNewWindow {
            action: String::from("/launch"),
            fields: vec![(String::from("app"), String::from("growth-chart"))],
        },
    )
}

/// The `<form>` of `page` that posts to `/launch`.
fn launch_form(page: &str) -> Vec<(String, String)> {
    let forms: Vec<_> = tags(page, "form")
        .into_iter()
        .filter(|form| form.get("action").map(String::as_str) == Some("/launch"))
        .collect();
    assert_eq!(1, forms.len(), "one form posts the launch: {page}");
    forms
        .into_iter()
        .flatten()
        .collect::<Vec<(String, String)>>()
}

#[test]
fn a_menu_item_posts_its_form_into_a_new_window_with_no_opener() {
    let page = html(more_menu(
        &en(),
        &Menu::new(
            "record-more",
            "More actions for the record",
            vec![Item::new(launch())],
        ),
    ));
    let form = launch_form(&page);
    for wanted in [
        ("method", "post"),
        ("target", "_blank"),
        ("rel", "noopener"),
    ] {
        assert!(
            form.contains(&(wanted.0.to_owned(), wanted.1.to_owned())),
            "the form carries {wanted:?}: {page}"
        );
    }
    assert!(
        has(
            &page,
            "input",
            &[
                ("type", "hidden"),
                ("name", "app"),
                ("value", "growth-chart")
            ]
        ),
        "the item's fields are hidden inputs of its form: {page}"
    );
    assert!(
        has(&page, "button", &[("type", "submit"), ("role", "menuitem")]),
        "the item is the form's submit button, a menu item: {page}"
    );
    assert!(
        tags(&page, "button")
            .iter()
            .filter(|button| button.get("role").map(String::as_str) == Some("menuitem"))
            .all(|item| !item.contains_key("hidden") && !item.contains_key("disabled")),
        "the item works with no script, so it is never held back for one: {page}"
    );
}

#[test]
fn the_item_says_it_opens_a_new_window_in_words_and_by_its_icon() {
    let page = html(more_menu(
        &en(),
        &Menu::new(
            "record-more",
            "More actions for the record",
            vec![Item::new(launch())],
        ),
    ));
    let button = page
        .split("role=\"menuitem\"")
        .nth(1)
        .and_then(|rest| rest.split("</button>").next())
        .unwrap_or_default();
    assert!(
        button.contains("<span class=\"fui-sr-only\"> (opens in a new window)</span>"),
        "assistive technology hears that the item opens a new window, inside its name: {page}"
    );
    let words = button.find("Launch Growth chart").unwrap_or(usize::MAX);
    let said = button.find("(opens in a new window)").unwrap_or(0);
    assert!(words < said, "the words come first: {button}");
    assert!(
        has(
            button,
            "svg",
            &[
                ("class", "fui-icon fui-new-window"),
                ("aria-hidden", "true")
            ]
        ),
        "the eye sees the external-link icon, hidden from assistive technology: {button}"
    );
    assert!(
        button.contains(Glyph::EXTERNAL_LINK.markup()),
        "the icon is Lucide's external-link: {button}"
    );
}

#[test]
fn an_action_button_posts_into_a_new_window_too() {
    let page = html(action_button(&en(), launch(), Variant::Secondary));
    let form = launch_form(&page);
    for wanted in [
        ("method", "post"),
        ("target", "_blank"),
        ("rel", "noopener"),
    ] {
        assert!(
            form.contains(&(wanted.0.to_owned(), wanted.1.to_owned())),
            "the form carries {wanted:?}: {page}"
        );
    }
    assert!(
        page.contains("(opens in a new window)</span>")
            && page.contains("fui-new-window")
            && page.contains("Launch Growth chart"),
        "the button says it opens a new window: {page}"
    );
}

#[test]
fn a_plain_post_opens_its_answer_in_the_same_window() {
    let page = html(more_menu(
        &en(),
        &Menu::new(
            "record-more",
            "More actions for the record",
            vec![Item::new(Action::new(
                None,
                "Check",
                Act::Post {
                    action: String::from("/check"),
                    fields: Vec::new(),
                },
            ))],
        ),
    ));
    assert!(
        !page.contains("target=") && !page.contains("fui-new-window"),
        "only the new-window action names a target: {page}"
    );
}
