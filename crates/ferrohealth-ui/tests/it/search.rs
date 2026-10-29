// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The top bar's search as the server draws it (#144): one joined `GET`
//! form, its states, the `/` key cap and its shortcut, the typeahead panel
//! it holds closed until the page hydrates, the answer's JSON wire, and the
//! find page's results in the page, grouped by kind.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::STYLESHEET;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::search::{
    Hit, HitGroup, Mark, SearchAnswer, SearchBox, Way, find_results, search_box,
};

use crate::markup::{has, tags};
use crate::render::{en, html};

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

/// The HTML of `search`.
fn drawn(search: &SearchBox) -> String {
    html(search_box(&en(), search))
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
    let html = drawn(&search(|_| {}));
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
    let html = drawn(&search(|_| {}));
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
    let none = drawn(&search(|search| search.shortcut = None));
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
    let html = drawn(&search(|search| search.busy = true));
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
    let idle = drawn(&search(|_| {}));
    assert!(!idle.contains("fui-spinner-ring"), "{idle}");
    assert!(!idle.contains("aria-busy"), "{idle}");
}

#[test]
fn the_disabled_search_shows_its_reason_as_text_and_drops_its_key_cap() {
    let reason = "Search waits while the index is rebuilding";
    let html = drawn(&search(|search| {
        search.disabled = Some(String::from(reason));
    }));
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

/// An answer to "alp" of seven files and a template.
fn answer() -> SearchAnswer {
    SearchAnswer {
        query: String::from("alp"),
        head: Some(String::from("8 answers")),
        groups: vec![
            HitGroup {
                kind: String::from("Files"),
                hits: (1..=7)
                    .map(|index| hit(&format!("alpha-{index}.yml"), &format!("/f/{index}")))
                    .collect(),
            },
            HitGroup {
                kind: String::from("Templates"),
                hits: vec![Hit {
                    mark: Some(Mark::Glyph(Glyph::FILE_TEXT)),
                    tag: Some(String::from("v1.0.2")),
                    ..hit("Alpine vitals", "/t")
                }],
            },
        ],
        ..SearchAnswer::default()
    }
}

#[test]
fn a_search_with_an_answer_route_holds_its_panel_closed_and_claims_no_combobox_before_it_hydrates()
{
    let html = drawn(&search(|search| {
        search.suggest = Some(String::from("/find/suggest"));
    }));
    assert!(
        has(
            &html,
            "div",
            &[("id", "find-panel"), ("class", "fui-search-results")]
        ),
        "{html}"
    );
    let panel = tags(&html, "div")
        .into_iter()
        .find(|div| div.get("id").map(String::as_str) == Some("find-panel"))
        .unwrap_or_default();
    assert!(
        panel.contains_key("hidden"),
        "closed until typed into: {html}"
    );
    assert!(
        has(
            &html,
            "div",
            &[
                ("role", "listbox"),
                ("id", "find-results"),
                ("aria-label", "Search answer")
            ]
        ),
        "{html}"
    );
    assert!(
        has(
            &html,
            "span",
            &[("role", "status"), ("class", "fui-sr-only")]
        ),
        "the count is announced from outside the panel: {html}"
    );
    // WAI-ARIA 1.2: with no WebAssembly the field never expands, so it is no combobox.
    let input = tags(&html, "input").remove(0);
    assert!(!input.contains_key("role"), "{html}");
    assert!(!input.contains_key("aria-expanded"), "{html}");
    let plain = drawn(&search(|_| {}));
    assert!(
        !plain.contains("fui-search-results"),
        "no route, no panel: {plain}"
    );
    assert!(!plain.contains("listbox"), "{plain}");
}

#[test]
fn the_answer_crosses_the_wire_as_json_and_reads_back() -> Result<(), Box<dyn std::error::Error>> {
    let mut answer = answer();
    answer.groups.truncate(1);
    answer.groups[0].hits.truncate(1);
    answer.groups[0].hits[0].mark = Some(Mark::Dot(Tone::Ok));
    answer.unfinished = Some(String::from("Templates did not answer"));
    let json = serde_json::to_value(&answer)?;
    assert_eq!(
        serde_json::json!({
            "query": "alp",
            "head": "8 answers",
            "groups": [{
                "kind": "Files",
                "hits": [{
                    "href": "/f/1",
                    "name": "alpha-1.yml",
                    "line": "model · 40 lines",
                    "mark": { "Dot": "Ok" },
                    "tag": null
                }]
            }],
            "none": null,
            "unfinished": "Templates did not answer",
            "more": null
        }),
        json,
        "the wire docs/consoles.md documents"
    );
    assert_eq!(answer, serde_json::from_value(json)?);
    let sparse: SearchAnswer = serde_json::from_str(r#"{"query":"x"}"#)?;
    assert_eq!(
        String::from("x"),
        sparse.query,
        "every other field may be left out"
    );
    let glyph: Mark = serde_json::from_str(r#"{"Glyph":"file-text"}"#)?;
    assert_eq!(
        Mark::Glyph(Glyph::FILE_TEXT),
        glyph,
        "a glyph goes by its Lucide name"
    );
    Ok(())
}

#[test]
fn the_find_page_draws_every_hit_of_each_kind_in_a_list_card_of_its_own() {
    let html = html(find_results(&en(), &answer()));
    assert!(html.contains(r#"<div class="fui-find-results">"#), "{html}");
    assert!(
        has(&html, "section", &[("aria-label", "Files")]),
        "a card per kind: {html}"
    );
    assert!(
        has(&html, "section", &[("aria-label", "Templates")]),
        "{html}"
    );
    for index in 1..=7 {
        assert!(
            has(
                &html,
                "a",
                &[("class", "fui-hit"), ("href", &format!("/f/{index}"))]
            ),
            "the find page cuts no kind short: {index} in {html}"
        );
    }
    assert!(html.contains("· 7"), "the kind counts its hits: {html}");
    assert!(
        html.contains(r#"<span class="fui-tag">v1.0.2</span>"#),
        "{html}"
    );
    assert!(html.contains("8 answers"), "{html}");
    assert!(
        !html.contains("fui-search-results"),
        "never the overlay: {html}"
    );
    assert!(!html.contains("style="), "{html}");
}

#[test]
fn a_hit_emphasises_the_words_searched_for() {
    let html = html(find_results(&en(), &answer()));
    assert!(
        html.contains(r#"<mark class="fui-search-match">alp</mark>ha-1.yml"#),
        "{html}"
    );
    assert!(
        html.contains(r#"<mark class="fui-search-match">Alp</mark>ine vitals"#),
        "without regard to case: {html}"
    );
}

#[test]
fn a_hit_on_the_find_page_draws_its_mark() {
    let answer = SearchAnswer {
        query: String::from("a"),
        groups: vec![HitGroup {
            kind: String::from("Files"),
            hits: vec![Hit {
                mark: Some(Mark::Dot(Tone::Ok)),
                ..hit("alpha.yml", "/a")
            }],
        }],
        ..SearchAnswer::default()
    };
    let html = html(find_results(&en(), &answer));
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
fn a_find_page_that_found_nothing_says_so_in_the_page() {
    let answer = SearchAnswer {
        query: String::from("zz"),
        more: Some(Way {
            href: String::from("/files"),
            words: String::from("Browse every file"),
        }),
        ..SearchAnswer::default()
    };
    let html = html(find_results(&en(), &answer));
    assert!(html.contains("fui-find-empty"), "{html}");
    assert!(html.contains("No matches for “zz”"), "{html}");
    assert!(
        has(
            &html,
            "a",
            &[("href", "/files"), ("class", "fui-find-more")]
        ),
        "{html}"
    );
    let said = SearchAnswer {
        query: String::from("zz"),
        none: Some((
            String::from("No file found for <zz>"),
            String::from("Check the spelling."),
        )),
        ..SearchAnswer::default()
    };
    let html = crate::render::html(find_results(&en(), &said));
    assert!(
        html.contains("No file found for &#60;zz&#62;"),
        "escaped: {html}"
    );
    assert!(html.contains("Check the spelling."), "{html}");
}

#[test]
fn an_answer_that_may_not_be_whole_says_so_over_its_hits() {
    let mut answer = answer();
    answer.unfinished = Some(String::from("Templates did not answer"));
    let html = html(find_results(&en(), &answer));
    assert!(
        has(
            &html,
            "div",
            &[("role", "status"), ("class", "fui-notice fui-tone-warn")]
        ),
        "{html}"
    );
    assert!(html.contains("Templates did not answer"), "{html}");
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
    let html = html(find_results(&en(), &answer));
    assert!(!html.contains("<script>"), "{html}");
    assert!(
        html.contains("&#60;script&#62;x&#60;/script&#62;"),
        "{html}"
    );
    assert!(html.contains("a &#38; b"), "{html}");
}

#[test]
fn the_panel_is_an_overlay_anchored_under_the_field_and_a_hits_line_keeps_its_row() {
    let surface = "  .fui-search-results {\n    position: absolute;\n    top: calc(100% + 0.375rem);\n    inset-inline: -1px;\n    z-index: 35;\n    border: 1px solid var(--edge-strong);";
    assert!(
        STYLESHEET.contains(surface),
        "the panel is no overlay surface under the field"
    );
    assert!(
        STYLESHEET.contains("      width: max(calc(100% + 2px), min(40rem, 100vw - 2rem));"),
        "the panel is not at least the field's width and at most 40rem"
    );
    assert!(
        STYLESHEET.contains("  .fui-search {\n    position: relative;\n  }"),
        "the panel is not anchored to the search box"
    );
    assert!(
        STYLESHEET.contains("  .fui-hit > .fui-hit-line {\n    grid-row: 2;"),
        "a hit's line shares a row"
    );
    assert!(
        STYLESHEET.contains("  .fui-hit > :is(.fui-identifier, .fui-tag) {\n    grid-row: 1 / 3;"),
        "a tag can clip the hit's line"
    );
}

#[test]
fn a_search_that_can_carry_clinical_content_posts_its_query() {
    let html = drawn(&search(|search| search.post = true));
    assert!(
        has(&html, "form", &[("role", "search"), ("method", "post")]),
        "{html}"
    );
}
