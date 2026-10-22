// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The shell and the sign-in page rendered to HTML on the host: the top
//! bar with its status pill and no theme control, the navigation with its
//! rail and counts, the user menu, and the sign-in card in every state.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::theme::Theme;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::flow::FlowState;
use ferrohealth_ui::view::shell::{TopBar, lockup, lockup_plain, mark, topbar};
use ferrohealth_ui::view::sign_in::{
    Basic, NOTICE_ID, Notice, Provider, Retry, SignIn, State, sign_in,
};
use ferrohealth_ui::view::status::{Reach, StatusPill, badge, banner, pill, status, status_pill};
use ferrohealth_ui::view::tabs::{Tab, binder};
use ferrohealth_ui::widget::menu::{Choice, Choices, ThemeChoice, UserMenu, user_menu};
use ferrohealth_ui::widget::nav::{Count, Entry, Group, Nest, NestItem, Rail, Sidebar, sidebar};
use leptos::prelude::*;

use crate::markup::{has, tags};
use crate::render::{html, page_html};

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

/// The status pill of `reach` saying `words`, with `detail` in its tooltip.
fn status_html(reach: Reach, words: &str, detail: Option<&str>) -> String {
    html(|| {
        status_pill(StatusPill {
            reach,
            words: words.to_owned(),
            detail: detail.map(str::to_owned),
        })
        .into_any()
    })
}

#[test]
fn the_status_pill_keeps_its_words_as_its_name_and_its_version_in_the_tooltip() {
    // WCAG 2.2 SC 1.4.1 and family rule 12: a shape, a colour and a word together.
    let up = status_html(Reach::Up, "Bridge up", Some("FerroBRIDGE 0.0.12"));
    assert!(
        has(
            &up,
            "span",
            &[
                ("class", "fui-pill fui-tone-ok fui-topbar-status"),
                ("title", "Bridge up · FerroBRIDGE 0.0.12"),
            ]
        ),
        "the tooltip holds the words and the version: {up}"
    );
    assert!(
        up.contains(r#"<span class="fui-roomy-text">Bridge up</span>"#),
        "the words stay in the page as the pill's name: {up}"
    );
    assert!(
        !up.contains(">FerroBRIDGE 0.0.12<"),
        "the version is never drawn in the pill: {up}"
    );
    let down = status_html(Reach::Unreachable, "Bridge <down>", None);
    assert!(
        has(&down, "span", &[("title", "Bridge &lt;down&gt;")]),
        "no detail, the words alone, escaped: {down}"
    );
}

#[test]
fn every_reach_draws_the_shape_of_its_tone() {
    for (reach, tone) in [
        (Reach::Up, Tone::Ok),
        (Reach::Degraded, Tone::Warn),
        (Reach::Unknown, Tone::Warn),
        (Reach::Unreachable, Tone::Bad),
        (Reach::Refused, Tone::Bad),
        (Reach::NotProbed, Tone::Neutral),
    ] {
        assert_eq!(tone, reach.tone(), "{reach:?}");
        let drawn = status_html(reach, "Server up", None);
        assert!(drawn.contains(tone.class()), "{reach:?}: {drawn}");
    }
    assert_eq!(6, Reach::ALL.len());
}

#[test]
fn the_top_bar_holds_its_parts_and_no_theme_control() {
    // Family rule 5: the theme is set only in the user menu and Your preferences.
    let bar = html(|| {
        topbar(TopBar {
            sidebar: "sidebar",
            lockup: lockup("/", mark(Glyph::SERVER), "FerroBRIDGE", "Console").into_any(),
            search: Some(view! { <form role="search"></form> }.into_any()),
            end: view! { <span class="fui-pill">"Bridge up"</span> }.into_any(),
        })
    });
    assert!(
        has(
            &bar,
            "button",
            &[
                ("command", "show-modal"),
                ("commandfor", "sidebar"),
                ("aria-haspopup", "dialog"),
                ("aria-label", "Menu")
            ]
        ),
        "the drawer button opens the navigation as a modal dialog: {bar}"
    );
    assert!(bar.contains(r#"<div class="fui-topbar-middle">"#), "{bar}");
    assert!(bar.contains(r#"<div class="fui-topbar-end">"#), "{bar}");
    assert!(!bar.contains("theme"), "no theme control: {bar}");
    let bare = html(|| {
        topbar(TopBar {
            sidebar: "sidebar",
            lockup: lockup("/", mark(Glyph::SERVER), "FerroBRIDGE", "Console").into_any(),
            search: None,
            end: ().into_any(),
        })
    });
    assert!(
        !bare.contains("fui-topbar-middle"),
        "no search, no middle: {bare}"
    );
}

#[test]
fn a_plain_lockup_leads_nowhere() {
    let plain = html(|| lockup_plain(mark(Glyph::SERVER), "FerroBRIDGE", "Console").into_any());
    assert!(plain.starts_with(r#"<span class="fui-lockup">"#), "{plain}");
    assert!(!plain.contains("<a"), "{plain}");
    assert!(plain.contains("FerroBRIDGE"), "{plain}");
}

/// The navigation with one group of three entries, the page at
/// `/files/synthetic-007`, and the rail `rail`.
fn navigation(rail: Option<Rail>) -> String {
    page_html(|| {
        sidebar(Sidebar {
            id: "sidebar",
            label: String::from("Pages"),
            groups: vec![Group {
                id: "nav-pages",
                title: "Pages".into(),
                entries: vec![
                    Entry {
                        href: "/files".into(),
                        label: "Files".into(),
                        glyph: Glyph::DATABASE,
                        exact: false,
                        count: Some(Count::Known {
                            count: 2,
                            words: String::from("2 errors in the running load"),
                        }),
                    },
                    Entry {
                        href: "/routes".into(),
                        label: "Routes".into(),
                        glyph: Glyph::ROUTE,
                        exact: false,
                        count: Some(Count::Unknown {
                            why: String::from("Not known: the bridge did not answer"),
                        }),
                    },
                    Entry {
                        href: "/about".into(),
                        label: "About".into(),
                        glyph: Glyph::INFO,
                        exact: true,
                        count: Some(Count::Known {
                            count: 0,
                            words: String::from("0 errors"),
                        }),
                    },
                ],
            }],
            current: String::from("/files/synthetic-007"),
            rail,
            nest: None,
        })
    })
}

#[test]
fn the_navigation_is_plain_links_with_the_page_marked_inside_its_drawer_island() {
    let page = navigation(None);
    assert!(
        page.contains("<leptos-island"),
        "the drawer is an island: {page}"
    );
    assert!(
        has(
            &page,
            "a",
            &[
                ("href", "/files"),
                ("aria-current", "page"),
                ("class", "fui-nav-entry")
            ]
        ),
        "the section of the page is marked: {page}"
    );
    assert!(
        page.contains(r#"<a href="/about" class="fui-nav-entry">"#),
        "another page is a plain link, unmarked, with no tooltip: {page}"
    );
    assert!(
        !page.contains("fui-sidebar-foot"),
        "a navigation that never folds has no foot: {page}"
    );
    // WAI-ARIA Authoring Practices, the dialog (modal) pattern: the drawer is a
    // native modal dialog, so the page behind is inert with no WebAssembly.
    assert!(
        has(
            &page,
            "dialog",
            &[("closedby", "any"), ("class", "fui-sidebar")]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "button",
            &[("command", "close"), ("commandfor", "sidebar")]
        ),
        "the drawer's Close closes its dialog: {page}"
    );
    assert!(
        has(&page, "nav", &[("class", "fui-sidebar-nav")]),
        "the entries are a navigation landmark: {page}"
    );
    assert!(!page.contains("popover"), "{page}");
}

#[test]
fn a_count_is_a_number_on_screen_and_a_phrase_to_assistive_technology() {
    // R12, the honest count; WCAG 2.2 SC 1.3.1.
    let page = navigation(None);
    assert!(
        page.contains(r#"<span aria-hidden="true" class="fui-nav-count-whole">2</span><!><span class="fui-sr-only">2 errors in the running load</span>"#),
        "{page}"
    );
    assert!(
        has(
            &page,
            "span",
            &[
                ("class", "fui-nav-count fui-nav-count-unknown"),
                ("title", "Not known: the bridge did not answer")
            ]
        ),
        "an unknown count is a muted dash with the reason: {page}"
    );
    assert!(
        !page.contains("0 errors"),
        "a count of zero draws nothing: {page}"
    );
}

#[test]
fn a_count_past_99_shows_whole_and_short_for_the_rail() {
    // Shell edge audit, row 12: 123456 on the rail's badge covered its glyph.
    let page = page_html(|| {
        sidebar(Sidebar {
            id: "sidebar",
            label: String::from("Pages"),
            groups: vec![Group {
                id: "nav-pages",
                title: "Pages".into(),
                entries: vec![Entry {
                    href: "/files".into(),
                    label: "Files".into(),
                    glyph: Glyph::DATABASE,
                    exact: false,
                    count: Some(Count::Known {
                        count: 123_456,
                        words: String::from("123456 errors in the running load"),
                    }),
                }],
            }],
            current: String::from("/files"),
            rail: None,
            nest: None,
        })
    });
    assert!(
        page.contains(r#"<span aria-hidden="true" class="fui-nav-count-whole">123456</span><span aria-hidden="true" class="fui-nav-count-short">99+</span><span class="fui-sr-only">123456 errors in the running load</span>"#),
        "{page}"
    );
    assert!(
        has(&page, "span", &[("class", "fui-nav-count is-long")]),
        "{page}"
    );
}

#[test]
fn an_entry_and_its_nested_section_take_a_base_path_learnt_at_run_time() {
    // Shell edge audit, row 29: a console mounted under a base path it reads from its configuration.
    let base = String::from("/console");
    let files = format!("{base}/files");
    let page = page_html(|| {
        sidebar(Sidebar {
            id: "sidebar",
            label: String::from("Pages"),
            groups: vec![Group {
                id: "nav-pages",
                title: "Pages".into(),
                entries: vec![Entry {
                    href: files.clone().into(),
                    label: "Files".into(),
                    glyph: Glyph::DATABASE,
                    exact: false,
                    count: None,
                }],
            }],
            current: format!("{files}/synthetic-007"),
            rail: None,
            nest: Some(Nest {
                under: files.clone().into(),
                label: String::from("Synthetic set and its files"),
                parent: NestItem {
                    label: String::from("Synthetic set"),
                    href: format!("{files}/set"),
                    state: FlowState::Up,
                    state_words: String::from("Reachable"),
                    mono: false,
                },
                count_words: String::from("0 files"),
                children: Vec::new(),
                all: String::from("All files"),
            }),
        })
    });
    assert!(
        page.contains(r#"<a href="/console/files" aria-current="location" class="fui-nav-entry">"#),
        "the entry leads under the base and opens its nested section: {page}"
    );
    assert!(page.contains("fui-nav-tree"), "{page}");
}

#[test]
fn the_rail_names_every_entry_by_tooltip_and_posts_its_expand() {
    let page = navigation(Some(Rail {
        collapsed: true,
        action: String::from("/sidebar"),
        back: String::from("/files?page=2"),
    }));
    assert!(
        has(
            &page,
            "dialog",
            &[("class", "fui-sidebar fui-sidebar-rail")]
        ),
        "{page}"
    );
    for word in ["Files", "Routes", "About"] {
        // WCAG 2.2 SC 1.4.13: a drawn tooltip shows on keyboard focus too.
        assert!(
            page.contains(&format!(
                r#"<span aria-hidden="true" class="fui-tooltip">{word}</span>"#
            )),
            "{word} is named by its tooltip: {page}"
        );
        assert!(
            page.contains(&format!(r#"<span class="fui-nav-word">{word}</span>"#)),
            "{word} keeps its accessible name: {page}"
        );
    }
    assert!(
        has(
            &page,
            "form",
            &[
                ("method", "post"),
                ("action", "/sidebar"),
                ("class", "fui-sidebar-foot fui-wide-only")
            ]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "input",
            &[("name", "back"), ("value", "/files?page=2")]
        ),
        "{page}"
    );
    assert!(
        has(
            &page,
            "button",
            &[
                ("name", "sidebar"),
                ("value", "full"),
                ("aria-expanded", "false"),
                ("aria-controls", "sidebar")
            ]
        ),
        "the rail posts its expand: {page}"
    );
    assert!(page.contains("Expand"), "{page}");
    let full = navigation(Some(Rail {
        collapsed: false,
        action: String::from("/sidebar"),
        back: String::from("/files"),
    }));
    assert!(
        has(
            &full,
            "button",
            &[
                ("name", "sidebar"),
                ("value", "rail"),
                ("aria-expanded", "true")
            ]
        ),
        "the full sidebar posts its collapse: {full}"
    );
    assert!(full.contains("Collapse"), "{full}");
    assert!(
        !has(&full, "a", &[("title", "Files")]),
        "no tooltip outside the rail: {full}"
    );
}

/// The user menu of `name`, signed in with a sign-out route when `sign_out`.
fn menu_html(name: Option<&str>, sign_out: bool) -> String {
    page_html(|| {
        user_menu(UserMenu {
            id: "user-menu",
            label: name.unwrap_or("Your menu").to_owned(),
            name: name.map(str::to_owned),
            who: name.unwrap_or("No sign-in").to_owned(),
            line: String::from("Bridge operator · FerroHEALTH ID"),
            preferences: String::from("/preferences"),
            theme: ThemeChoice {
                current: Theme::Dark,
                action: String::from("/theme"),
                back: String::from("/files"),
            },
            choices: vec![Choices {
                label: String::from("FHIR version"),
                choices: vec![
                    Choice {
                        words: String::from("R4"),
                        href: String::from("?fhir=R4"),
                        chosen: true,
                    },
                    Choice {
                        words: String::from("R5"),
                        href: String::from("?fhir=R5"),
                        chosen: false,
                    },
                ],
            }],
            shortcuts: Some(String::from("/shortcuts")),
            sign_out: sign_out.then(|| String::from("/sign-out")),
        })
    })
}

#[test]
fn the_user_menu_is_a_menu_button_over_a_popover() {
    // WAI-ARIA Authoring Practices 1.2, the menu button pattern.
    let menu = menu_html(Some("Demo <Operator>"), true);
    assert!(menu.contains("<leptos-island"), "an island: {menu}");
    assert!(
        has(
            &menu,
            "button",
            &[
                ("popovertarget", "user-menu"),
                ("aria-haspopup", "menu"),
                ("aria-controls", "user-menu-menu"),
                ("title", "Demo &lt;Operator&gt;")
            ]
        ),
        "{menu}"
    );
    assert!(
        !menu.contains("aria-expanded"),
        "the button says it is open only once its island runs, never a fixed false: {menu}"
    );
    assert!(
        menu.contains(r#"<span class="fui-operator-name">Demo &lt;Operator&gt;</span>"#),
        "the name is escaped: {menu}"
    );
    assert!(
        has(
            &menu,
            "div",
            &[
                ("id", "user-menu"),
                ("popover", "auto"),
                (
                    "class",
                    "fui-menu fui-menu-compact fui-menu-anchored fui-menu-sheet fui-user-menu"
                )
            ]
        ),
        "{menu}"
    );
    assert!(
        has(
            &menu,
            "div",
            &[
                ("id", "user-menu-menu"),
                ("role", "menu"),
                ("aria-label", "Demo &lt;Operator&gt;")
            ]
        ),
        "{menu}"
    );
    assert!(
        has(
            &menu,
            "button",
            &[
                ("class", "fui-menu-cancel"),
                ("popovertarget", "user-menu"),
                ("popovertargetaction", "hide")
            ]
        ),
        "the sheet's Cancel closes it with no script: {menu}"
    );
    assert!(
        has(
            &menu,
            "a",
            &[("role", "menuitem"), ("href", "/preferences")]
        ),
        "{menu}"
    );
    assert!(menu.contains("Your preferences"), "{menu}");
    assert!(
        has(
            &menu,
            "a",
            &[
                ("role", "menuitem"),
                ("href", "/shortcuts"),
                ("class", "fui-menu-item fui-wide-only")
            ]
        ),
        "no key hints on phones (family rule 6): {menu}"
    );
    assert!(menu.contains(r#"<kbd class="fui-kbd">?</kbd>"#), "{menu}");
}

#[test]
fn who_is_signed_in_heads_the_popup_outside_the_aria_menu() {
    // WAI-ARIA 1.2, the menu role: its owned elements are menu items and their groups, so plain
    // text inside it is skipped by menu navigation.
    let menu = menu_html(Some("Demo Operator"), true);
    let head = menu.find("fui-menu-header").expect("the head");
    let list = menu.find(r#"role="menu""#).expect("the menu");
    assert!(head < list, "the head comes before the menu: {menu}");
    let inside = menu.get(list..).unwrap_or_default();
    assert!(
        !inside.contains("fui-menu-header") && !inside.contains("<strong>"),
        "the identity is outside the menu role: {menu}"
    );
    assert!(
        inside.contains(r#"href="/preferences""#),
        "the items are inside it: {menu}"
    );
}

#[test]
fn the_theme_is_three_menuitemradio_choices_of_one_plain_form() {
    let menu = menu_html(Some("Demo Operator"), true);
    let radios: Vec<_> = tags(&menu, "button")
        .into_iter()
        .filter(|button| button.get("role").map(String::as_str) == Some("menuitemradio"))
        .collect();
    assert_eq!(3, radios.len(), "{menu}");
    assert!(
        radios.iter().all(
            |radio| radio.get("type").map(String::as_str) == Some("submit")
                && radio.get("name").map(String::as_str) == Some("theme")
        ),
        "each choice posts the theme: {menu}"
    );
    let checked: Vec<_> = radios
        .iter()
        .filter(|radio| radio.get("aria-checked").map(String::as_str) == Some("true"))
        .collect();
    assert_eq!(1, checked.len(), "{menu}");
    assert_eq!(
        Some("dark"),
        checked[0].get("value").map(String::as_str),
        "the theme in force is checked: {menu}"
    );
    assert!(
        has(&menu, "form", &[("method", "post"), ("action", "/theme")]),
        "{menu}"
    );
    assert!(
        has(&menu, "input", &[("name", "back"), ("value", "/files")]),
        "{menu}"
    );
}

#[test]
fn the_narrow_choices_are_links_and_sign_out_comes_last_after_a_separator() {
    // Family rule 7: destructive actions come last in menus, after a separator.
    let menu = menu_html(Some("Demo Operator"), true);
    assert!(
        has(
            &menu,
            "a",
            &[
                ("role", "menuitemradio"),
                ("href", "?fhir=R4"),
                ("aria-checked", "true")
            ]
        ),
        "{menu}"
    );
    assert!(menu.contains(r#"<div class="fui-menu-version">"#), "{menu}");
    let sign_out = menu.find("/sign-out").expect("a sign-out form");
    let last_separator = menu.rfind(r#"role="separator""#).expect("a separator");
    assert!(last_separator < sign_out, "{menu}");
    assert!(menu[sign_out..].find("menuitem").is_some(), "{menu}");
    assert!(
        has(
            &menu,
            "form",
            &[("method", "post"), ("action", "/sign-out")]
        ),
        "Sign out posts: {menu}"
    );
    assert!(
        menu.find("Keyboard shortcuts") < Some(sign_out),
        "nothing follows Sign out: {menu}"
    );
}

#[test]
fn a_menu_with_nobody_signed_in_has_an_icon_button_and_no_sign_out() {
    let menu = menu_html(None, false);
    assert!(
        has(
            &menu,
            "button",
            &[("popovertarget", "user-menu"), ("aria-label", "Your menu")]
        ),
        "the icon button is named: {menu}"
    );
    assert!(!menu.contains("fui-operator-name"), "{menu}");
    assert!(!menu.contains("Sign out"), "{menu}");
}

/// The sign-in card in `state`, with the provider's action, and the basic
/// form when `basic`.
fn sign_in_html(state: State, basic: Option<Basic>) -> String {
    html(|| {
        sign_in(SignIn {
            lockup: lockup_plain(mark(Glyph::SERVER), "FerroBRIDGE", "Console").into_any(),
            title: Some(String::from("Sign in to the console")),
            state,
            provider: Some(Provider {
                href: String::from("/auth/login?next=%2Ffiles"),
                label: String::from("Sign in with FerroHEALTH ID"),
                busy: String::from("Taking you to FerroHEALTH ID…"),
                glyph: Glyph::LOG_IN,
            }),
            basic,
            line: None,
            note: Some(String::from(
                "You sign in at your organisation's identity provider.",
            )),
            links: vec![
                (
                    String::from("Documentation"),
                    String::from("https://example.org/docs"),
                ),
                (
                    String::from("Privacy notice"),
                    String::from("https://example.org/privacy"),
                ),
            ],
            caption: Some(String::from(
                "ferrobridge-console. It holds no clinical data.",
            )),
        })
        .into_any()
    })
}

/// A basic form after a wrong password.
fn basic() -> Basic {
    Basic {
        action: String::from("/login"),
        next: String::from("/files?x=\"<y>\""),
        divider: None,
        submit: None,
        username: String::from("demo.steward"),
        error: Some(String::from("4 tries are left.")),
    }
}

#[test]
fn the_card_offers_the_provider_as_one_external_link_with_its_links_and_caption() {
    let card = sign_in_html(State::Ready, None);
    assert!(
        has(
            &card,
            "a",
            &[
                ("href", "/auth/login?next=%2Ffiles"),
                ("rel", "external"),
                ("class", "fui-button fui-button-primary")
            ]
        ),
        "{card}"
    );
    assert!(
        card.contains(r#"<h1 class="fui-sign-in-title">Sign in to the console</h1>"#),
        "{card}"
    );
    assert!(
        has(
            &card,
            "a",
            &[
                ("href", "https://example.org/privacy"),
                ("rel", "external noopener")
            ]
        ),
        "the links come from the console: {card}"
    );
    assert!(
        card.contains(r#"<p class="fui-sign-in-caption">"#),
        "{card}"
    );
    assert!(!card.contains(NOTICE_ID), "ready says nothing: {card}");
    assert!(
        !card.contains("type=\"password\""),
        "no password field unless the console passes a form: {card}"
    );
    assert!(
        !card.contains("<a href=\"/\" class=\"fui-lockup\""),
        "{card}"
    );
}

#[test]
fn every_state_names_its_cause_in_its_tone_and_role() {
    // WAI-ARIA 1.2, `alert` and `status`; WCAG 2.2 SC 4.1.3.
    let words = || Notice::new("The cause <and> the way on.");
    for (state, tone, role) in [
        (State::Redirecting(Some(words())), Tone::Info, "status"),
        (State::SignedOut(words()), Tone::Ok, "status"),
        (State::Expired(words()), Tone::Warn, "status"),
        (State::Full(words()), Tone::Warn, "alert"),
        (State::Unavailable(words()), Tone::Warn, "status"),
        (State::Refused(words()), Tone::Bad, "alert"),
        (State::Rejected(words()), Tone::Bad, "alert"),
        (State::ProviderUnreachable(words()), Tone::Bad, "alert"),
        (State::UpstreamUnreachable(words()), Tone::Warn, "status"),
        (State::Locked(words()), Tone::Warn, "alert"),
    ] {
        let name = format!("{state:?}");
        let card = sign_in_html(state, None);
        let class = format!("fui-sign-in-notice {}", tone.class());
        assert!(
            has(
                &card,
                "div",
                &[("role", role), ("id", NOTICE_ID), ("class", class.as_str())]
            ),
            "{name}: {card}"
        );
        assert!(
            card.contains("The cause &lt;and&gt; the way on."),
            "{name}: the words are escaped: {card}"
        );
    }
}

#[test]
fn a_notice_carries_its_detail_and_a_way_to_try_again() {
    let card = sign_in_html(
        State::UpstreamUnreachable(Notice {
            text: String::from("The CDR did not answer."),
            detail: Some(String::from("(502)")),
            retry: Some(Retry {
                label: String::from("Try again"),
                href: String::from("/sign-in"),
            }),
        }),
        None,
    );
    assert!(
        card.contains(r#"<span class="fui-small fui-muted">(502)</span>"#),
        "{card}"
    );
    assert!(
        has(
            &card,
            "a",
            &[
                ("href", "/sign-in"),
                ("class", "fui-button fui-button-secondary fui-button-small")
            ]
        ),
        "{card}"
    );
}

#[test]
fn redirecting_is_a_busy_disabled_button_and_unavailable_a_disabled_one() {
    let busy = sign_in_html(State::Redirecting(None), Some(basic()));
    assert!(
        has(&busy, "button", &[("aria-busy", "true"), ("disabled", "")]),
        "{busy}"
    );
    assert!(busy.contains("Taking you to FerroHEALTH ID…"), "{busy}");
    assert!(
        !busy.contains("/auth/login"),
        "no second request while on the way: {busy}"
    );
    assert!(
        has(&busy, "input", &[("name", "username"), ("disabled", "")]),
        "the form waits too: {busy}"
    );
    let off = sign_in_html(State::Unavailable(Notice::new("Not set up.")), None);
    assert!(
        has(
            &off,
            "button",
            &[("class", "fui-button fui-button-primary"), ("disabled", "")]
        ),
        "{off}"
    );
    assert!(!off.contains("/auth/login"), "{off}");
}

#[test]
fn the_basic_form_is_a_plain_post_a_password_manager_reads_with_no_placeholder() {
    let card = sign_in_html(State::Refused(Notice::new("Not signed in.")), Some(basic()));
    assert!(
        has(&card, "form", &[("method", "post"), ("action", "/login")]),
        "{card}"
    );
    assert!(
        has(
            &card,
            "input",
            &[
                ("name", "username"),
                ("autocomplete", "username"),
                ("value", "demo.steward")
            ]
        ),
        "{card}"
    );
    assert!(
        has(
            &card,
            "input",
            &[
                ("type", "password"),
                ("autocomplete", "current-password"),
                ("aria-invalid", "true"),
                ("aria-describedby", "sign-in-password-error")
            ]
        ),
        "{card}"
    );
    assert!(card.contains(r#"id="sign-in-password-error""#), "{card}");
    assert!(
        !card.contains("placeholder"),
        "no placeholder repeats a label: {card}"
    );
    assert!(
        has(
            &card,
            "input",
            &[
                ("name", "next"),
                ("value", "/files?x=&quot;&lt;y&gt;&quot;")
            ]
        ),
        "the destination is escaped: {card}"
    );
    assert!(
        card.contains(r#"<div class="fui-divider">or</div>"#),
        "the form follows the provider after the divider: {card}"
    );
    let locked = sign_in_html(State::Locked(Notice::new("Paused.")), Some(basic()));
    assert!(
        has(&locked, "input", &[("type", "password"), ("disabled", "")]),
        "a locked username's form is disabled: {locked}"
    );
}

#[test]
fn a_card_with_no_method_says_so() {
    let card = html(|| {
        sign_in(SignIn {
            lockup: lockup_plain(mark(Glyph::SERVER), "FerroBRIDGE", "Console").into_any(),
            title: None,
            state: State::Ready,
            provider: None,
            basic: None,
            line: None,
            note: None,
            links: Vec::new(),
            caption: None,
        })
        .into_any()
    });
    assert!(card.contains("No sign-in method is available"), "{card}");
    assert!(!card.contains("<h1"), "{card}");
}

#[test]
fn a_navigation_or_tab_count_groups_its_digits() {
    let page = page_html(|| {
        sidebar(Sidebar {
            id: "sidebar",
            label: String::from("Pages"),
            groups: vec![Group {
                id: "nav-pages",
                title: "Pages".into(),
                entries: vec![Entry {
                    href: "/findings",
                    label: "Findings".into(),
                    glyph: Glyph::DATABASE,
                    exact: false,
                    count: Some(Count::Known {
                        count: 312_000,
                        words: String::from("312,000 findings"),
                    }),
                }],
            }],
            current: String::from("/files"),
            rail: None,
            nest: None,
        })
    });
    assert!(
        page.contains(r#"<span aria-hidden="true">312,000</span>"#),
        "{page}"
    );
    let tabs = html(|| {
        binder(
            "Sections",
            vec![Tab::new("nodes", "/nodes", "Nodes").count(12_345, "12,345 nodes")],
            "nodes",
            ().into_any(),
        )
    });
    assert!(
        tabs.contains(r#"<span aria-hidden="true">12,345</span>"#),
        "{tabs}"
    );
}
