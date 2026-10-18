// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Every view rendered to HTML on the host, as a console's server draws it
//! and as a read-only screen hands it to the browser: what each must say,
//! what each must never carry, and the escaping of what it is given.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::model::{Paging, Sort, Window};
use ferrohealth_ui::table::view::{Body, Column, DataTable, data_table, link_pager};
use ferrohealth_ui::theme::Theme;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::{Variant, link_button};
use ferrohealth_ui::view::code::{code_lines, code_panel};
use ferrohealth_ui::view::dialog::{Confirmation, confirm_dialog, confirm_trigger};
use ferrohealth_ui::view::empty::{empty_state, skeleton};
use ferrohealth_ui::view::refusal::{Issue, Refusal, Severity, refusal};
use ferrohealth_ui::view::shell::{Footer, footer, lockup, mark};
use ferrohealth_ui::view::sign_in::{Basic, Notice, Provider, SignIn, sign_in};
use ferrohealth_ui::view::status::{badge, banner, pill, status};
use ferrohealth_ui::view::surface::{Stat, definitions, stat_card};
use ferrohealth_ui::view::tabs::{Tab, link_tabs};
use ferrohealth_ui::view::theme_switch::theme_switch;
use leptos::prelude::*;

/// The HTML `view` renders to, under a reactive owner as a server render
/// runs.
fn html(view: impl FnOnce() -> AnyView) -> String {
    let owner = Owner::new();
    owner.with(|| view().to_html())
}

/// A sign-in card with the methods `basic` and `provider`.
fn card(basic: bool, provider: bool, refusal: Option<&str>) -> String {
    html(|| {
        sign_in(SignIn {
            lockup: lockup("/", mark(Glyph::SERVER), "FerroBRIDGE", "Console").into_any(),
            notices: vec![
                Notice::new(Tone::Warn, "Your session ended. Sign in again to continue.")
                    .id("session-expired"),
                Notice::new(Tone::Accent, "A sandbox.\nUse synthetic / synthetic."),
            ],
            refusal: refusal.map(str::to_owned),
            basic: basic.then(|| Basic {
                action: "/login",
                next: String::from("/files?x=\"<y>\""),
            }),
            provider: provider.then(|| Provider {
                href: String::from("/auth/oidc/login"),
                label: String::from("Sign in with OIDC"),
            }),
            links: vec![(
                String::from("API reference"),
                String::from("https://example.org/docs"),
            )],
        })
        .into_any()
    })
}

#[test]
fn the_basic_sign_in_is_a_plain_post_form_a_password_manager_reads() {
    let form = card(true, false, Some("Wrong username or password."));
    assert!(form.contains(r#"method="post""#), "{form}");
    assert!(form.contains(r#"action="/login""#), "{form}");
    assert!(form.contains(r#"name="username""#), "{form}");
    assert!(form.contains(r#"autocomplete="username""#), "{form}");
    assert!(
        form.contains(r#"autocomplete="current-password""#),
        "{form}"
    );
    assert!(form.contains(r#"type="password""#), "{form}");
    assert!(form.contains(r#"name="next""#), "{form}");
    assert!(
        form.contains("&lt;y&gt;"),
        "the destination is escaped: {form}"
    );
    assert!(form.contains("Wrong username or password."), "{form}");
    assert!(form.contains(r#"id="sign-in-refused""#), "{form}");
    assert!(form.contains(r#"id="session-expired""#), "{form}");
    assert!(form.contains("A sandbox."), "both notices show: {form}");
    assert!(form.contains("API reference"), "{form}");
    assert!(form.contains("Your session ended."), "{form}");
    assert!(
        !form.contains("rel=\"external\" class=\"fui-button"),
        "no provider: {form}"
    );
    assert!(
        !form.contains("fui-divider"),
        "one method, no divider: {form}"
    );
}

#[test]
fn the_oauth_sign_in_is_one_external_link_and_both_methods_split_by_or() {
    let button = card(false, true, None);
    assert!(button.contains(r#"href="/auth/oidc/login""#), "{button}");
    assert!(button.contains(r#"rel="external""#), "{button}");
    assert!(
        button.contains("fui-button-primary"),
        "the one method stands out: {button}"
    );
    assert!(!button.contains("<form"), "{button}");
    let both = card(true, true, None);
    assert!(both.contains("fui-divider"), "{both}");
    assert!(
        both.contains("fui-button-secondary"),
        "the provider steps back: {both}"
    );
    let none = card(false, false, None);
    assert!(none.contains("No sign-in method is available"), "{none}");
}

#[test]
fn a_status_is_a_word_as_well_as_a_tint() {
    let marks = html(|| {
        view! {
            {pill(Tone::Ok, "Bridge UP")}
            {badge(Tone::Bad, "down")}
            {status(Tone::Info, Glyph::INFO, "Serving", Some(String::from("R4")))}
            {banner(Tone::Warn, Glyph::TRIANGLE_ALERT, "Sandbox", "Holds no real patient data.")}
        }
        .into_any()
    });
    for word in [
        "Bridge UP",
        "down",
        "Serving",
        "R4",
        "Sandbox",
        "fui-tone-warn",
    ] {
        assert!(marks.contains(word), "{word}: {marks}");
    }
}

#[test]
fn the_theme_switch_posts_and_marks_the_theme_in_force_by_more_than_colour() {
    let switch = html(|| {
        theme_switch(Theme::Dark, String::from("/theme"), String::from("/files")).into_any()
    });
    assert!(switch.contains(r#"action="/theme""#), "{switch}");
    assert_eq!(
        1,
        switch.matches(r#"aria-pressed="true""#).count(),
        "{switch}"
    );
    assert_eq!(3, switch.matches(r#"name="theme""#).count(), "{switch}");
    assert!(
        switch.contains(Glyph::CHECK.markup()),
        "a check on the pressed one: {switch}"
    );
}

#[test]
fn a_refusal_lists_its_issues_with_their_text_and_counts_the_silent_ones() {
    let shown = html(|| {
        refusal(&Refusal {
            status: Some(422),
            summary: String::from("the bridge refused the request."),
            issues: vec![
                Issue::new(Severity::Information, "informational"),
                Issue::new(Severity::Error, "processing")
                    .text("no program maps <the> template")
                    .place("Bundle.entry[0]")
                    .details(vec![String::from("<b>MSG</b>")]),
            ],
            request: Some(String::from("GET /fhir?q=<script>alert(1)</script>")),
            evidence: Some(String::from("<img src=x onerror=alert(1)>")),
        })
        .into_any()
    });
    assert!(
        shown.contains("422: the bridge refused the request."),
        "{shown}"
    );
    assert!(
        shown.contains("no program maps &lt;the&gt; template"),
        "{shown}"
    );
    assert!(
        shown.contains("1 information issue with no text left out."),
        "{shown}"
    );
    assert!(!shown.contains("informational"), "{shown}");
    assert!(
        shown.contains("&lt;b&gt;MSG&lt;/b&gt;"),
        "a detail code is text: {shown}"
    );
    assert!(
        shown.contains("GET /fhir?q=&lt;script&gt;alert(1)&lt;/script&gt;"),
        "the request is text: {shown}"
    );
    assert!(
        shown.contains("&lt;img src=x onerror=alert(1)&gt;"),
        "the evidence is text: {shown}"
    );
    assert!(
        !shown.contains("<script") && !shown.contains("<img"),
        "{shown}"
    );
}

#[test]
fn a_code_panel_escapes_its_text_and_numbers_its_lines_for_a_link() {
    let panel = html(|| code_panel("The answer", Language::Json, "{\"a\": \"</pre>\"}").into_any());
    assert!(panel.contains("&lt;/pre&gt;"), "{panel}");
    assert!(panel.contains(r#"role="region""#), "{panel}");
    let lines = html(|| code_lines("The file", Language::Yaml, "a: 1\nb: 2", "L").into_any());
    assert!(lines.contains(r#"id="L2""#), "{lines}");
}

#[test]
fn the_data_table_marks_its_sort_and_the_pager_names_an_unavailable_step() {
    let columns = [
        Column::text("name", "Name").sortable(),
        Column::text("rows", "Rows").numeric(),
    ];
    let sort = Sort::read(Some("name"), Some("asc"), &["name"]);
    let table = html(|| {
        data_table(DataTable {
            caption: "Mapping files",
            columns: &columns,
            sort: sort.as_ref(),
            sort_href: &|sort: &Sort| format!("/files?sort={}", sort.column),
            body: Body::Rows(
                view! { <tr><td>"a"</td><td class="fui-num">"1"</td></tr> }.into_any(),
            ),
        })
        .into_any()
    });
    assert!(table.contains(r#"aria-sort="ascending""#), "{table}");
    assert!(table.contains("<tbody>"), "{table}");
    assert!(
        table.contains("<caption>Mapping files</caption>"),
        "{table}"
    );
    let window = Window::over(30, Paging::default());
    let pager = html(|| {
        link_pager("Pages", window, &|paging: Paging| {
            format!("/files?page={}", paging.page)
        })
        .into_any()
    });
    assert!(pager.contains("1-25 of 30"), "{pager}");
    assert_eq!(2, pager.matches("(unavailable)").count(), "{pager}");
    assert!(pager.contains(r#"href="/files?page=2""#), "{pager}");
    assert!(pager.contains("Page 1 of 2"), "{pager}");
    let loading = html(|| {
        data_table(DataTable {
            caption: "Loading",
            columns: &columns,
            sort: None,
            sort_href: &|_| String::new(),
            body: Body::Loading(2),
        })
        .into_any()
    });
    assert!(loading.contains(r#"aria-busy="true""#), "{loading}");
}

#[test]
fn the_surfaces_render_and_none_writes_a_style_attribute() {
    let page = html(|| {
        view! {
            {lockup("/", mark(Glyph::LAYERS), "FerroUI", "Gallery")}
            {stat_card(Stat { glyph: Glyph::USERS, value: String::from("12"), label: "Members".into(), href: Some(String::from("/members")) })}
            {definitions(vec![("Version", "0.1.0".into_any())])}
            {link_tabs("Views", &[Tab { href: "/a", label: "A" }, Tab { href: "/b", label: "B" }], "/b")}
            {empty_state(Glyph::INBOX, "Nothing here", "No file yet.", ())}
            {skeleton(3, "Loading the files.")}
            {link_button(String::from("/new"), Variant::Primary, Some(Glyph::PLUS), "New")}
            {confirm_trigger("delete", Variant::Danger, Glyph::TRASH, "Delete")}
            {confirm_dialog(Confirmation { id: "delete", title: "Delete it?", message: "It goes for good." }, ())}
            {footer(Footer { product: "FerroUI Gallery", version: "0.1.0", about: "/about" })}
        }
        .into_any()
    });
    for expected in [
        "fui-lockup-accent",
        "fui-stat-value",
        "fui-facts",
        r#"aria-current="page""#,
        "fui-empty",
        r#"aria-busy="true""#,
        r#"popovertarget="delete""#,
        r#"popover="auto""#,
        "Cadasto B.V.",
    ] {
        assert!(page.contains(expected), "{expected}: {page}");
    }
    assert!(!page.contains("style="), "{page}");
}
