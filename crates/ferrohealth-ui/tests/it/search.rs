// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The top bar's search as the server draws it (#144): one joined `GET`
//! form, its states, the `/` key cap and its shortcut, and the answer under
//! it, grouped by kind with its first hit selected.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::STYLESHEET;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::search::{
    Hit, HitGroup, Mark, SearchAnswer, SearchBox, Way, search_answer, search_box,
};
use leptos::prelude::*;

use crate::markup::{has, tags};

/// `view` rendered to HTML in a page tree with the server's shared
/// context, as a page draws an island that takes children.
fn page_html(view: impl FnOnce() -> AnyView) -> String {
    let owner = Owner::new_root(Some(std::sync::Arc::new(
        hydration_context::SsrSharedContext::new(),
    )));
    owner.with(|| view().to_html())
}

/// A search box of the console's route `/find`, changed by `change`.
fn search(change: impl FnOnce(&mut SearchBox)) -> SearchBox {
    let mut search = SearchBox {
        id: String::from("find"),
        label: String::from("Search the files"),
        placeholder: String::from("A file or a path"),
        action: String::from("/find"),
        ..SearchBox::default()
    };
    change(&mut search);
    search
}

/// The HTML of `search` with `answer` under it.
fn drawn(search: SearchBox, answer: Option<SearchAnswer>) -> String {
    page_html(move || search_box(search, answer))
}

/// A hit named `name` that leads to `href`.
fn hit(name: &str, href: &str) -> Hit {
    Hit {
        href: String::from(href),
        name: String::from(name),
        line: String::from("model · 40 lines"),
        mark: None,
        tag: None,
    }
}

#[test]
fn the_search_is_one_get_form_a_landmark_with_a_hidden_label_and_a_named_submit() {
    let html = drawn(search(|_| {}), None);
    assert!(
        has(
            &html,
            "form",
            &[
                ("role", "search"),
                ("method", "get"),
                ("action", "/find"),
                ("class", "fui-search")
            ]
        ),
        "{html}"
    );
    assert!(
        has(&html, "label", &[("for", "find"), ("class", "fui-sr-only")]),
        "{html}"
    );
    assert!(
        has(
            &html,
            "input",
            &[
                ("id", "find"),
                ("name", "q"),
                ("type", "search"),
                ("placeholder", "A file or a path"),
                ("aria-keyshortcuts", "/")
            ]
        ),
        "{html}"
    );
    assert!(
        has(
            &html,
            "button",
            &[
                ("type", "submit"),
                ("class", "fui-search-submit"),
                ("aria-label", "Find")
            ]
        ),
        "{html}"
    );
    assert!(html.contains(r#"<span class="fui-search-word">Find</span>"#));
    assert!(!html.contains("style="), "{html}");
}

#[test]
fn the_slash_cap_is_hidden_from_a_reader_and_drawn_only_with_a_shortcut() {
    let html = drawn(search(|_| {}), None);
    assert!(
        has(
            &html,
            "kbd",
            &[
                ("class", "fui-kbd fui-search-hint"),
                ("aria-hidden", "true")
            ]
        ),
        "{html}"
    );
    let none = drawn(search(|search| search.shortcut = None), None);
    assert!(tags(&none, "kbd").is_empty(), "no shortcut, no cap: {none}");
    assert!(
        tags(&none, "input")
            .iter()
            .all(|input| !input.contains_key("aria-keyshortcuts")),
        "no shortcut is announced: {none}"
    );
}

#[test]
fn the_busy_search_draws_the_spinner_in_the_glyphs_place_and_the_submit_waits() {
    let html = drawn(search(|search| search.busy = true), None);
    assert!(has(&html, "form", &[("aria-busy", "true")]), "{html}");
    assert!(
        has(
            &html,
            "span",
            &[("class", "fui-spinner-ring"), ("aria-hidden", "true")]
        ),
        "{html}"
    );
    assert!(
        tags(&html, "button")
            .iter()
            .any(|button| button.contains_key("disabled")),
        "{html}"
    );
    let idle = drawn(search(|_| {}), None);
    assert!(!idle.contains("fui-spinner-ring"), "{idle}");
    assert!(!idle.contains("aria-busy"), "{idle}");
}

#[test]
fn the_disabled_search_shows_its_reason_as_text_and_drops_its_key_cap() {
    let reason = "Search waits while the index is rebuilding";
    let html = drawn(
        search(|search| search.disabled = Some(String::from(reason))),
        None,
    );
    let input = tags(&html, "input").remove(0);
    assert!(input.contains_key("disabled"), "{html}");
    assert_eq!(Some(reason), input.get("placeholder").map(String::as_str));
    assert_eq!(
        Some("find-why"),
        input.get("aria-describedby").map(String::as_str)
    );
    assert!(!input.contains_key("aria-keyshortcuts"), "{html}");
    assert!(
        html.contains(&format!(
            r#"<span id="find-why" class="fui-sr-only">{reason}</span>"#
        )),
        "{html}"
    );
    assert!(tags(&html, "kbd").is_empty(), "{html}");
    assert!(
        tags(&html, "button")
            .iter()
            .all(|button| button.contains_key("disabled")),
        "{html}"
    );
}

#[test]
fn the_answer_groups_its_hits_by_kind_and_selects_the_first() {
    let answer = SearchAnswer {
        head: Some(String::from("3 answers")),
        groups: vec![
            HitGroup {
                kind: String::from("Files"),
                hits: vec![hit("alpha.yml", "/a"), hit("beta.yml", "/b")],
            },
            HitGroup {
                kind: String::from("Templates"),
                hits: vec![Hit {
                    mark: Some(Mark::Glyph(Glyph::FILE_TEXT)),
                    tag: Some(String::from("v1.0.2")),
                    ..hit("vitals", "/t")
                }],
            },
        ],
        none: None,
        more: Some(Way {
            href: String::from("/find?q=a"),
            words: String::from("All results"),
        }),
    };
    let html = drawn(search(|_| {}), Some(answer));
    assert!(
        has(
            &html,
            "div",
            &[
                ("id", "find-results"),
                ("class", "fui-search-results"),
                ("role", "region"),
                ("aria-label", "Search answer")
            ]
        ),
        "{html}"
    );
    assert!(
        has(
            &html,
            "a",
            &[
                ("id", "find-hit-1"),
                ("href", "/a"),
                ("aria-current", "true")
            ]
        ),
        "{html}"
    );
    let current: Vec<_> = tags(&html, "a")
        .into_iter()
        .filter(|link| link.contains_key("aria-current"))
        .collect();
    assert_eq!(1, current.len(), "one hit is selected: {html}");
    assert!(
        has(&html, "a", &[("id", "find-hit-3"), ("href", "/t")]),
        "{html}"
    );
    assert!(
        has(
            &html,
            "p",
            &[("id", "find-kind-2"), ("class", "fui-search-results-group")]
        ),
        "{html}"
    );
    assert!(
        has(&html, "ul", &[("aria-labelledby", "find-kind-2")]),
        "{html}"
    );
    assert!(
        html.contains(r#"<span class="fui-tag">v1.0.2</span>"#),
        "{html}"
    );
    assert!(
        has(
            &html,
            "a",
            &[("href", "/find?q=a"), ("class", "fui-search-results-more")]
        ),
        "{html}"
    );
    assert!(html.contains("3 answers"), "{html}");
}

#[test]
fn an_answer_of_one_kind_names_its_list_and_draws_no_heading() {
    let answer = SearchAnswer {
        groups: vec![HitGroup {
            kind: String::from("Files"),
            hits: vec![Hit {
                mark: Some(Mark::Dot(Tone::Ok)),
                ..hit("alpha.yml", "/a")
            }],
        }],
        ..SearchAnswer::default()
    };
    let html = page_html(move || search_answer("find", answer));
    assert!(has(&html, "ul", &[("aria-label", "Files")]), "{html}");
    assert!(!html.contains("fui-search-results-group"), "{html}");
    assert!(
        has(
            &html,
            "span",
            &[("class", "fui-dot fui-tone-ok"), ("aria-hidden", "true")]
        ),
        "{html}"
    );
}

#[test]
fn an_answer_that_found_nothing_says_so_and_selects_nothing() {
    let answer = SearchAnswer {
        none: Some((
            String::from("No file found for <zz>"),
            String::from("Check the spelling."),
        )),
        more: Some(Way {
            href: String::from("/files"),
            words: String::from("Browse every file"),
        }),
        ..SearchAnswer::default()
    };
    let html = drawn(search(|_| {}), Some(answer));
    assert!(
        html.contains(r#"<div class="fui-search-results-empty">"#),
        "{html}"
    );
    assert!(
        html.contains("No file found for &lt;zz&gt;"),
        "escaped: {html}"
    );
    assert!(!html.contains("aria-current"), "{html}");
}

#[test]
fn a_hit_escapes_its_name_and_line() {
    let answer = SearchAnswer {
        groups: vec![HitGroup {
            kind: String::from("Files"),
            hits: vec![Hit {
                line: String::from("a & b"),
                ..hit("<script>x</script>", "/a")
            }],
        }],
        ..SearchAnswer::default()
    };
    let html = page_html(move || search_answer("find", answer));
    assert!(!html.contains("<script>"), "{html}");
    assert!(html.contains("&lt;script&gt;x&lt;/script&gt;"), "{html}");
    assert!(html.contains("a &amp; b"), "{html}");
}

#[test]
fn the_answer_is_one_overlay_surface_and_a_hits_line_keeps_its_row() {
    let surface = "  .fui-search-results {\n    position: absolute;\n    inset: calc(100% + 0.375rem) 0 auto 0;\n    z-index: 35;\n    border: 1px solid var(--edge-strong);";
    assert!(
        STYLESHEET.contains(surface),
        "the answer is no overlay surface"
    );
    assert!(
        STYLESHEET.contains("  .fui-search-results li > a > .fui-small {\n    grid-row: 2;"),
        "a hit's line shares a row"
    );
    assert!(
        STYLESHEET.contains(
            "  .fui-search-results li > a > :is(.fui-identifier, .fui-tag) {\n    grid-row: 1 / 3;"
        ),
        "a tag can clip the hit's line"
    );
}

#[test]
fn a_search_that_can_carry_clinical_content_posts_its_query() {
    let html = drawn(search(|search| search.post = true), None);
    assert!(
        has(&html, "form", &[("role", "search"), ("method", "post")]),
        "{html}"
    );
}
