// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The page templates T1 to T3, the About page and the error page rendered
//! to HTML on the host: each optional part appears only where
//! `design/COMPONENTS.md` says it does, and what a template is given is
//! escaped.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::i18n::Word;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::error_page::{Detail, ErrorPage, Status, Way, error_page};
use ferrohealth_ui::view::shell::{Footer, footer};
use ferrohealth_ui::view::status::pill;
use ferrohealth_ui::view::surface::{Crumb, Header, page_heading};
use ferrohealth_ui::view::tabs::{Tab, TabRow};
use ferrohealth_ui::view::template::about::{
    AboutPage, Link, Manufacturer, SelfDescription, about_page,
};
use ferrohealth_ui::view::template::detail::{DetailPage, Stepper, detail_page, stepper};
use ferrohealth_ui::view::template::list::{
    ActiveFilters, FilterChip, HeadingCount, ListCard, ListPage, list_empty, list_page,
};
use ferrohealth_ui::view::template::settings::{
    How, Managed, Setting, SettingsPage, Source, ValueSource, revision_line, setting_row,
    settings_page,
};
use ferrohealth_ui::widget::action_menu::{Act, Action};
use leptos::prelude::*;

use crate::markup::{has, tags};

/// The HTML `view` renders to, under a reactive owner as a server render
/// runs, without the hydration markers Leptos writes between parts.
fn html(view: impl FnOnce() -> AnyView) -> String {
    let owner = Owner::new();
    owner.with(|| view().to_html()).replace("<!>", "")
}

/// A header titled `title` with a one-sentence intro.
fn header(title: &str) -> Header {
    Header {
        title: Word::from(title),
        subtitle: Some(Word::from("Example content: the template's parts.")),
        ..Header::default()
    }
}

/// A row of one binder tab, named "Sections".
fn row() -> TabRow {
    TabRow {
        label: String::from("Sections"),
        tabs: vec![Tab::new("all", "/all", "tabs")],
        current: String::from("all"),
    }
}

/// A list page of `total` items with every optional part offered.
fn list(total: Option<u64>, filters: ActiveFilters, tabs: bool) -> String {
    html(move || {
        list_page(ListPage {
            header: header("Locations"),
            actions: ActionGroup::primary(Action::new(
                None,
                "New location",
                Act::Link(String::from("/new")),
            )),
            total,
            summary: Some(view! { <div class="fui-summary-chips">"summary"</div> }.into_any()),
            toolbar: Some(view! { <div class="fui-list-toolbar">"toolbar"</div> }.into_any()),
            filters,
            card: ListCard {
                title: String::from("Locations"),
                count: total.map(HeadingCount::All),
                order: None,
                tabs: tabs.then(row),
                actions: None,
                body: view! { <table class="fui-table"></table> }.into_any(),
                pager: total
                    .filter(|count| *count > 25)
                    .map(|_| view! { <nav class="fui-pager">"pager"</nav> }.into_any()),
            },
        })
    })
}

#[test]
fn a_list_of_ten_items_or_fewer_has_no_summary_strip_and_no_toolbar() {
    let page = list(Some(5), ActiveFilters::default(), false);
    assert!(!page.contains("fui-summary-chips"), "{page}");
    assert!(!page.contains("fui-list-toolbar"), "{page}");
    assert!(
        !page.contains("fui-filter-chips"),
        "no filter is set: {page}"
    );
    assert!(
        !page.contains("fui-pager"),
        "one page holds every row: {page}"
    );
    assert!(page.contains(r#"<div class="fui-list-page">"#), "{page}");
}

#[test]
fn a_long_list_draws_its_summary_strip_toolbar_and_pager() {
    let page = list(Some(30), ActiveFilters::default(), false);
    assert!(page.contains("fui-summary-chips"), "{page}");
    assert!(page.contains("fui-list-toolbar"), "{page}");
    assert!(
        page.contains(r#"<div class="fui-list-foot"><nav class="fui-pager">"#),
        "{page}"
    );
}

#[test]
fn a_list_with_no_stated_end_has_a_toolbar_and_no_summary_strip() {
    let page = list(None, ActiveFilters::default(), false);
    assert!(page.contains("fui-list-toolbar"), "{page}");
    assert!(!page.contains("fui-summary-chips"), "{page}");
    assert!(
        !page.contains("fui-list-count"),
        "no count is stated: {page}"
    );
}

#[test]
fn the_card_opens_with_its_header_row_title_and_count() {
    let page = list(Some(5), ActiveFilters::default(), false);
    assert!(
        has(
            &page,
            "section",
            &[
                ("class", "fui-card fui-panel fui-worklist fui-list-card"),
                ("aria-label", "Locations")
            ]
        ),
        "{page}"
    );
    assert!(
        page.contains(r#"<div class="fui-panel-head fui-list-head">"#),
        "{page}"
    );
    assert!(
        page.contains(r#"Locations <span class="fui-list-count">· 5</span>"#),
        "the count muted after a middle dot: {page}"
    );
    assert!(
        page.contains("New location"),
        "the title row's primary: {page}"
    );
}

#[test]
fn binder_tabs_carry_the_title_and_the_card_is_their_panel() {
    let page = list(Some(5), ActiveFilters::default(), true);
    assert!(
        !page.contains("fui-list-head"),
        "no header row under tabs: {page}"
    );
    assert!(
        page.contains(
            r#"<div class="fui-tabbed"><nav aria-label="Sections" data-ferroui-tab-keys="" class="fui-tabs">"#
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "section",
            &[
                ("class", "fui-worklist fui-list-panel"),
                ("aria-label", "Locations")
            ]
        ),
        "{page}"
    );
}

#[test]
fn the_active_filters_say_what_they_keep_and_each_is_removed_by_a_link() {
    let filters = ActiveFilters {
        chips: vec![FilterChip {
            label: String::from("State: <degraded>"),
            remove: String::from("/things?state="),
        }],
        clear: String::from("/things"),
        shown: Some(7),
    };
    let page = list(Some(21), filters, false);
    assert!(page.contains("Showing 7 of 21"), "R14 with a count: {page}");
    assert!(page.contains("State: &lt;degraded&gt;"), "escaped: {page}");
    assert!(
        has(
            &page,
            "a",
            &[
                ("href", "/things?state="),
                ("aria-label", "Remove the filter State: &lt;degraded&gt;")
            ]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "a",
            &[
                ("href", "/things"),
                ("class", "fui-button fui-button-ghost fui-button-small")
            ]
        ),
        "Clear filters is a link: {page}"
    );
    assert!(page.contains("Clear filters"), "{page}");
    assert!(
        tags(&page, "button").is_empty(),
        "no button that leads nowhere: {page}"
    );
}

#[test]
fn an_empty_list_offers_the_primary_action_as_one_quiet_text_link() {
    let line = html(|| {
        list_empty(
            String::from("No location yet."),
            Some((String::from("Add the first location"), String::from("/new"))),
        )
    });
    assert!(
        line.contains(r#"<p class="fui-transform-empty">No location yet. <a href="/new">Add the first location</a></p>"#),
        "{line}"
    );
    assert!(
        !line.contains("fui-button"),
        "never a second primary button: {line}"
    );
}

#[test]
fn the_trail_turns_into_a_back_link_and_the_badge_sits_beside_the_title() {
    let page = html(|| {
        page_heading(
            Header {
                title: Word::from("Locatie <Noord>"),
                crumbs: vec![Crumb::new("Locations", "/things")],
                badge: Some(pill(Tone::Ok, "Reachable").into_any()),
                ..Header::default()
            },
            ActionGroup::none(),
        )
        .into_any()
    });
    assert!(
        page.contains(r#"<li><a href="/things"><span"#)
            && has(
                &page,
                "span",
                &[("class", "fui-crumb-back"), ("aria-hidden", "true")]
            ),
        "the arrow a phone shows before the parent: {page}"
    );
    assert!(
        page.contains(r#"<li aria-current="page">Locatie &lt;Noord&gt;</li>"#),
        "{page}"
    );
    assert!(
        page.contains(r#"<div class="fui-page-title-line"><h1>Locatie &lt;Noord&gt;</h1>"#),
        "{page}"
    );
    assert!(page.contains("Reachable"), "{page}");
}

#[test]
fn a_detail_page_puts_its_one_section_under_binder_tabs() {
    let page = html(|| {
        detail_page(DetailPage {
            header: header("Locatie Noord"),
            actions: ActionGroup::none(),
            stepper: None,
            tabs: Some(row()),
            body: view! { <p>"Overview"</p> }.into_any(),
        })
    });
    assert!(
        page.contains(r#"<div class="fui-stack"><div class="fui-tabbed"><nav aria-label="Sections" data-ferroui-tab-keys="" class="fui-tabs"><a href="/all" aria-current="page" data-tab="all" class="fui-tab">tabs</a></nav><!--<() />--><div class="fui-tab-panel"><p>Overview</p>"#),
        "{page}"
    );
    let plain = html(|| {
        detail_page(DetailPage {
            header: header("Locatie Noord"),
            actions: ActionGroup::none(),
            stepper: None,
            tabs: None,
            body: view! { <p>"Overview"</p> }.into_any(),
        })
    });
    assert!(
        !plain.contains("fui-tabbed"),
        "one section has no tabs: {plain}"
    );
}

#[test]
fn the_stepper_leads_by_links_and_marks_an_end_unavailable() {
    let page = html(|| {
        stepper(Stepper {
            label: String::from("Candidates"),
            position: String::from("Candidate 1 of 12"),
            what: String::from("candidate"),
            previous: None,
            next: Some(String::from("/queue/2")),
        })
    });
    assert!(
        has(
            &page,
            "nav",
            &[("class", "fui-stepper"), ("aria-label", "Candidates")]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "a",
            &[("href", "/queue/2"), ("aria-label", "Next candidate")]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "a",
            &[
                ("role", "link"),
                ("aria-disabled", "true"),
                ("aria-label", "Previous candidate")
            ]
        ),
        "{page}"
    );
    assert!(
        tags(&page, "a")
            .iter()
            .filter(|a| a.contains_key("href"))
            .count()
            == 1,
        "the unavailable step leads nowhere: {page}"
    );
    assert!(page.contains("Candidate 1 of 12"), "{page}");
}

/// A setting with the key `key`, at the next start when `next_start`.
fn setting(key: &str, next_start: bool) -> Setting {
    Setting {
        label: String::from("Format"),
        description: String::from("auto, json or pretty."),
        value: view! { <code>"json"</code> }.into_any(),
        keys: vec![key.to_owned()],
        next_start,
        source: None,
    }
}

/// A file-only settings page of `settings`.
fn file_page(settings: Vec<Setting>) -> String {
    html(|| {
        settings_page(SettingsPage {
            header: header("Configuration"),
            actions: ActionGroup::none(),
            source: Source::File {
                file: String::from("ferrobridge.toml"),
                how: How::Page(String::from("#")),
            },
            tabs: None,
            settings,
            replica: None,
        })
    })
}

#[test]
fn a_value_from_the_environment_says_so_after_the_value() {
    let page = file_page(vec![Setting {
        source: Some(ValueSource::Environment),
        ..setting("telemetry.format", false)
    }]);
    let value = page
        .find(r#"<span class="fui-setting-value"><code>json</code></span>"#)
        .unwrap_or_else(|| panic!("the value: {page}"));
    let source = page
        .find(r#"<span class="fui-setting-source">"#)
        .unwrap_or_else(|| panic!("the source marker: {page}"));
    assert!(source > value, "the marker follows the value: {page}");
    let marker = &page[source..];
    let marker = &marker[..marker.find("</span>").unwrap_or(marker.len())];
    assert!(
        marker.contains(r#"aria-hidden="true""#) && marker.ends_with("From the environment"),
        "the glyph is hidden and the words name the source: {marker}"
    );
}

#[test]
fn a_default_value_says_so_after_the_value() {
    let page = file_page(vec![Setting {
        source: Some(ValueSource::Default),
        ..setting("telemetry.format", false)
    }]);
    assert!(
        page.contains("Default</span></div>"),
        "the marker ends the value's column: {page}"
    );
    assert!(!page.contains("From the environment"), "{page}");
}

#[test]
fn a_value_from_the_file_carries_no_source_marker() {
    let page = file_page(vec![setting("telemetry.format", false)]);
    assert!(
        !page.contains("fui-setting-source"),
        "the file is the unmarked case: {page}"
    );
}

#[test]
fn a_managed_value_from_the_file_says_so() {
    let row = html(|| {
        setting_row(
            Setting {
                source: Some(ValueSource::File),
                ..setting("limits.mode", false)
            },
            true,
        )
    });
    assert!(
        row.contains("From the file</span></div>"),
        "a file-first section's value in a managed configuration: {row}"
    );
}

#[test]
fn a_managed_setting_shows_next_start_before_its_source() {
    let row = html(|| {
        setting_row(
            Setting {
                source: Some(ValueSource::Environment),
                ..setting("limits.mode", true)
            },
            true,
        )
    });
    let next = row.find("Next start").unwrap_or_else(|| panic!("{row}"));
    let source = row
        .find("From the environment")
        .unwrap_or_else(|| panic!("{row}"));
    assert!(next < source, "{row}");
}

#[test]
fn a_file_only_configuration_says_where_it_is_set_and_has_no_badge_or_marker() {
    let page = html(|| {
        settings_page(SettingsPage {
            header: Header {
                badge: Some(view! { <span class="fui-mode-badge">"Managed"</span> }.into_any()),
                ..header("Configuration")
            },
            actions: ActionGroup::none(),
            source: Source::File {
                file: String::from("ferrobridge.toml"),
                how: How::Page(String::from("/configuration/how")),
            },
            tabs: Some(row()),
            settings: vec![setting("telemetry.format", true)],
            replica: None,
        })
    });
    assert!(
        page.contains(r#"Set in <code>ferrobridge.toml</code> · read at start · <a href="/configuration/how">How configuration works</a>"#),
        "{page}"
    );
    assert!(
        !page.contains("fui-mode-badge"),
        "no mode badge in file mode: {page}"
    );
    assert!(
        !page.contains("Next start"),
        "no per-row Next start marker: {page}"
    );
    assert!(
        page.contains(r#"<div class="fui-tabbed fui-settings-tabbed">"#),
        "{page}"
    );
}

#[test]
fn a_file_line_opens_the_informational_dialog_with_no_script() {
    // NOTE: HTML Living Standard, the button element's command attribute: show-modal opens a dialog.
    let page = html(|| {
        settings_page(SettingsPage {
            header: header("Configuration"),
            actions: ActionGroup::none(),
            source: Source::File {
                file: String::from("ferrobridge.toml"),
                how: How::Dialog("how-configuration".into()),
            },
            tabs: None,
            settings: vec![setting("telemetry.format", false)],
            replica: None,
        })
    });
    assert!(
        has(
            &page,
            "button",
            &[
                ("type", "button"),
                ("class", "fui-link-button"),
                ("command", "show-modal"),
                ("commandfor", "how-configuration"),
                ("aria-haspopup", "dialog"),
            ]
        ),
        "{page}"
    );
    assert!(
        page.contains(r#"class="fui-link-button">How configuration works</button></p>"#),
        "the button ends the line with the words the link would say: {page}"
    );
    assert!(
        !page.contains(">How configuration works</a>"),
        "no link in the file line: {page}"
    );
}

#[test]
fn a_setting_names_its_key_with_a_labelled_copy_button() {
    let page = html(|| {
        settings_page(SettingsPage {
            header: header("Configuration"),
            actions: ActionGroup::none(),
            source: Source::File {
                file: String::from("ferrobridge.toml"),
                how: How::Page(String::from("#")),
            },
            tabs: None,
            settings: vec![setting("telemetry.<format>", false)],
            replica: None,
        })
    });
    assert!(
        has(
            &page,
            "button",
            &[
                ("type", "button"),
                ("class", "fui-key-hint"),
                ("aria-label", "Copy the key telemetry.&lt;format&gt;")
            ]
        ),
        "R15: a labelled button the keyboard reaches once hydrated: {page}"
    );
    assert!(
        has(
            &page,
            "button",
            &[("class", "fui-key-hint"), ("hidden", "")]
        ),
        "§15.8: the button waits for the WebAssembly: {page}"
    );
    assert!(
        page.contains(
            r#"<span class="fui-key-hint is-text"><code>telemetry.&lt;format&gt;</code></span>"#
        ),
        "until then the key is selectable text: {page}"
    );
    assert!(
        page.contains(r#"<span class="fui-setting-label">Format"#),
        "the hint sits on the label's line: {page}"
    );
}

#[test]
fn a_managed_configuration_shows_its_badge_revision_intro_and_marker() {
    let page = html(|| {
        settings_page(SettingsPage {
            header: header("Settings"),
            actions: ActionGroup::none(),
            source: Source::Managed(Box::new(Managed {
                badge: view! { <span class="fui-mode-badge">"Managed"</span> }.into_any(),
                revision: Some(revision_line(
                    view! { <strong>"Revision 7"</strong> }.into_any(),
                )),
                intro: String::from("How calls behave."),
                actions: Some(view! { <a href="/edit">"Edit"</a> }.into_any()),
            })),
            tabs: None,
            settings: vec![setting("mode", true)],
            replica: None,
        })
    });
    assert!(
        page.contains(
            r#"<div class="fui-page-title-line"><h1>Settings</h1><span class="fui-mode-badge">"#
        ),
        "{page}"
    );
    assert!(
        page.contains(r#"<div class="fui-revision fui-revision-compact">"#),
        "{page}"
    );
    assert!(
        page.contains(r#"<p class="fui-settings-panel-intro">How calls behave.</p>"#),
        "{page}"
    );
    assert!(page.contains(r#"<a href="/edit">Edit</a>"#), "{page}");
    assert!(page.contains("Next start"), "{page}");
    assert!(!page.contains("Set in"), "{page}");
}

#[test]
fn the_about_page_writes_the_console_version_in_the_footer_form() {
    let release = Footer {
        product: "Example Console".into(),
        version: "0.1.0".into(),
        about: "/about".into(),
    };
    let page = html(|| {
        about_page(AboutPage {
            header: header("About"),
            release: release.clone().release(),
            versions: vec![(String::from("Server"), "Example 0.1.0".into_any())],
            manufacturer: Manufacturer {
                licence: String::from("Business Source License 1.1"),
                support: Link {
                    label: String::from("info@cadasto.com"),
                    href: String::from("mailto:info@cadasto.com"),
                },
                documentation: None,
            },
            cards: Vec::new(),
            self_description: Some(SelfDescription {
                title: String::from("The self-description"),
                line: String::from("What the server declares."),
                language: Language::Json,
                text: String::from("{\"name\": \"<x>\"}"),
                copy_id: String::from("about-copy"),
                download: String::from("/about/document.json"),
            }),
            replica: None,
        })
    });
    assert!(
        page.contains("<dt>Console</dt><dd>Example Console 0.1.0</dd>"),
        "{page}"
    );
    assert!(
        page.contains("<dt>Made by</dt><dd>Cadasto B.V.</dd>"),
        "{page}"
    );
    assert!(
        !page.contains("Documentation"),
        "no documentation, no row: {page}"
    );
    assert!(
        has(
            &page,
            "div",
            &[
                (
                    "class",
                    "fui-code-panel fui-self-description fui-scroll-cue"
                ),
                ("role", "region"),
                ("tabindex", "0")
            ]
        ),
        "{page}"
    );
    assert!(
        page.contains("&lt;x&gt;"),
        "the document is escaped: {page}"
    );
    assert!(
        has(
            &page,
            "a",
            &[("href", "/about/document.json"), ("download", "")]
        ),
        "Download is a link that works with no script: {page}"
    );
    assert!(
        has(&page, "button", &[("id", "about-copy"), ("hidden", "")]),
        "Copy is the copy island, hidden until it works: {page}"
    );
    let foot = html(|| footer(release).into_any());
    assert!(
        foot.contains("Example Console 0.1.0 · Cadasto B.V. · "),
        "{foot}"
    );
}

/// The error page of `status`, with its detail when `detail`.
fn error(status: Status, detail: bool) -> String {
    html(move || {
        error_page(ErrorPage {
            status,
            title: String::from("No mapping file at <this> path"),
            body: view! { "The bridge reads no such file." }.into_any(),
            hint: Some(view! { <code>"__ferrobridge.operator"</code> }.into_any()),
            ways: vec![
                Way {
                    label: String::from("Every mapping file"),
                    href: String::from("/mappings/files"),
                    glyph: Some(Glyph::FILES),
                },
                Way {
                    label: String::from("Dashboard"),
                    href: String::from("/"),
                    glyph: None,
                },
            ],
            detail: detail.then(|| Detail {
                time: String::from("2026-10-21 09:42:11 UTC"),
                request_id: String::from("0199c2a4-77e1"),
            }),
        })
    })
}

#[test]
fn the_error_page_says_the_status_in_words_before_its_number() {
    for (status, line) in [
        (Status::Forbidden, "Not allowed · 403"),
        (Status::NotFound, "Not found · 404"),
        (Status::Fault, "Fault · 500"),
    ] {
        let page = error(status, false);
        assert!(
            page.contains(&format!(r#"<p class="fui-status-page-code">{line}</p>"#)),
            "{page}"
        );
    }
}

#[test]
fn the_error_page_names_the_error_and_leads_on_by_links() {
    let page = error(Status::NotFound, false);
    assert!(
        page.contains("<h1>No mapping file at &lt;this&gt; path</h1>"),
        "{page}"
    );
    assert!(
        has(
            &page,
            "a",
            &[
                ("href", "/mappings/files"),
                ("class", "fui-button fui-button-primary")
            ]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "a",
            &[("href", "/"), ("class", "fui-button fui-button-secondary")]
        ),
        "{page}"
    );
    assert!(
        tags(&page, "button").is_empty(),
        "the way on is a link: {page}"
    );
    assert!(
        !page.contains("<details"),
        "no detail, no disclosure: {page}"
    );
    assert!(
        page.contains(r#"<p class="fui-small fui-muted"><code>__ferrobridge.operator</code></p>"#),
        "{page}"
    );
}

#[test]
fn the_error_page_folds_the_time_and_request_id_away() {
    let page = error(Status::Fault, true);
    assert!(
        page.contains(r#"<details class="fui-disclosure"><summary>Technical detail</summary>"#),
        "closed by default: {page}"
    );
    assert!(
        page.contains("<dt>Request id</dt><dd><code>0199c2a4-77e1</code></dd>"),
        "{page}"
    );
    assert!(page.contains("2026-10-21 09:42:11 UTC"), "{page}");
    assert!(!page.contains("style="), "{page}");
}

#[test]
fn a_detail_page_of_a_queue_holds_its_stepper_in_the_title_row() {
    let page = html(|| {
        detail_page(DetailPage {
            header: header("Candidate 3"),
            actions: ActionGroup::primary(Action::new(
                None,
                "Link",
                Act::Link(String::from("/link")),
            )),
            stepper: Some(Stepper {
                label: String::from("Candidates"),
                position: String::from("Candidate 3 of 12"),
                what: String::from("candidate"),
                previous: Some(String::from("/queue/2")),
                next: Some(String::from("/queue/4")),
            }),
            tabs: None,
            body: view! { <p>"Overview"</p> }.into_any(),
        })
    });
    assert!(
        page.contains(
            r#"<div class="fui-actions"><nav aria-label="Candidates" class="fui-stepper">"#
        ),
        "{page}"
    );
}
