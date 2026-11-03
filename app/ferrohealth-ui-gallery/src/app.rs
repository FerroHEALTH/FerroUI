// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The document around every page and the shell around a brand's pages.
//!
//! Each brand has its pages under its key, `/ferrobridge/…`, drawn in the
//! shell a console draws, and the sign-in cards under `/sign-in/…`. The
//! document links the library's stylesheet and the brand's theme file from
//! the library's asset router, and carries the theme the request's cookie
//! names as `data-theme` and the density as `data-density`, so a page
//! arrives in its theme before any script runs, and its locale as `lang`,
//! the locale every component draws its words in.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::Arg;
use ferrohealth_ui::icon::{Glyph, icon};
use ferrohealth_ui::serve::Assets;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::menu::{Choice, Choices, ThemeChoice, UserLink, UserMenu, user_menu};
use ferrohealth_ui::view::nav::{Count, Entry, Group, Rail, Sidebar, sidebar};
use ferrohealth_ui::view::segmented::segmented_links;
use ferrohealth_ui::view::shell::{
    Document, Shell, TopBar, centred, document, lockup, mark, shell, topbar,
};
use ferrohealth_ui::view::status::{Reach, StatusPill, banner, status_pill};
use ferrohealth_ui::view::toast::toast_region;
use ferrohealth_ui::view::version::{VersionSelect, version_select};

use crate::locale::Msg;
use crate::page::{Frame, Page, VERSION, brand_glyph, footer, title};
use crate::request::{PREFERENCE_ROUTE, SIDEBAR_ROUTE, SIGN_OUT_ROUTE, rail_of};
use crate::visit::Visit;

/// The id of the navigation, which the drawer button opens.
const SIDEBAR_ID: &str = "sidebar";

/// The whole document of `page` for `visit`.
#[must_use]
pub fn render(visit: &Visit, page: Page) -> Html {
    let toasts = toast_region(&visit.locale, page.flash);
    let body = match page.frame {
        Frame::Shell => brand_shell(visit, page.content, toasts),
        Frame::Centred => centred(&visit.locale, page.content, toasts, footer(visit)),
    };
    document(Document {
        locale: visit.locale.clone(),
        preferences: visit.preferences.clone(),
        title: title(visit, &page.title),
        assets: Assets::of(visit.brand),
        nonce: None,
        head: Html::raw(String::from(
            "<link rel=\"icon\" type=\"image/svg+xml\" href=\"/favicon.svg\">",
        )),
        body,
    })
}

/// One entry of the navigation.
fn entry(href: String, label: String, glyph: Glyph, exact: bool) -> Entry {
    Entry {
        href: href.into(),
        label: label.into(),
        glyph,
        exact,
        count: None,
    }
}

/// The entries of the navigation of `brand`.
fn entries(visit: &Visit, brand: Brand) -> Vec<Entry> {
    let key = brand.key();
    [
        ("", Msg::Overview, Glyph::LAYOUT_DASHBOARD, true),
        ("/forms", Msg::FormsEditor, Glyph::PENCIL, true),
        ("/tables", Msg::TablesPaging, Glyph::DATABASE, true),
        ("/code", Msg::Documents, Glyph::SQUARE_TERMINAL, true),
        ("/configuration", Msg::Configuration, Glyph::SETTINGS, false),
        ("/overlays", Msg::Overlays, Glyph::LAYERS, true),
        ("/charts", Msg::Charts, Glyph::CHART_LINE, true),
        ("/layouts", Msg::Layouts, Glyph::PANEL_RIGHT, true),
        ("/operations", Msg::Operations, Glyph::FLASK_CONICAL, true),
        ("/terminology", Msg::Terminology, Glyph::BOOK_A, true),
        ("/trees", Msg::Trees, Glyph::FOLDER_OPEN, true),
        ("/find", Msg::FindTitle, Glyph::SEARCH, true),
        ("/clinical", Msg::ClinicalRecords, Glyph::HEART_PULSE, true),
        ("/federation", Msg::Federation, Glyph::NETWORK, false),
    ]
    .into_iter()
    .map(|(page, msg, glyph, exact)| entry(format!("/{key}{page}"), visit.tr(msg), glyph, exact))
    .collect()
}

/// The `href` of the federation entry of `brand`, the nested sidebar
/// section's place.
const fn federation_entry(brand: Brand) -> &'static str {
    match brand {
        Brand::Bridge => "/ferrobridge/federation",
        Brand::Fed => "/ferrofed/federation",
        Brand::Ehr => "/ferroehr/federation",
        Brand::Term => "/ferroterm/federation",
        Brand::Pix => "/ferropix/federation",
    }
}

/// The page template entries of `brand`.
fn templates(visit: &Visit, brand: Brand) -> Vec<Entry> {
    let key = brand.key();
    let mut entries: Vec<Entry> = [
        ("list", Msg::NavList, Glyph::LIST_CHECKS),
        ("list-many", Msg::NavListMany, Glyph::LIST_CHECKS),
        ("list-empty", Msg::NavListEmpty, Glyph::INBOX),
        ("list-tabs", Msg::NavListTabs, Glyph::LIST_CHECKS),
        ("detail", Msg::NavDetail, Glyph::FILE_TEXT),
        ("settings", Msg::NavSettings, Glyph::SETTINGS),
        (
            "settings-managed",
            Msg::NavSettingsManaged,
            Glyph::SLIDERS_HORIZONTAL,
        ),
    ]
    .into_iter()
    .map(|(page, msg, glyph)| {
        entry(
            format!("/{key}/templates/{page}"),
            visit.tr(msg),
            glyph,
            true,
        )
    })
    .collect();
    entries.push(entry(
        format!("/{key}/preferences"),
        visit.tr(Msg::NavPreferences),
        Glyph::CIRCLE_USER,
        false,
    ));
    entries.extend(
        [
            ("about", Msg::NavAbout, Glyph::INFO),
            ("forbidden", Msg::NavForbidden, Glyph::SHIELD_OFF),
            ("not-found", Msg::NavNotFound, Glyph::COMPASS),
        ]
        .into_iter()
        .map(|(page, msg, glyph)| {
            entry(
                format!("/{key}/templates/{page}"),
                visit.tr(msg),
                glyph,
                true,
            )
        }),
    );
    entries.push(entry(
        format!("/fault/{key}"),
        visit.tr(Msg::NavFault),
        Glyph::SERVER_CRASH,
        true,
    ));
    entries
}

/// The sign-in entries of `brand`.
fn sign_ins(visit: &Visit, brand: Brand) -> Vec<Entry> {
    let key = brand.key();
    [
        ("basic", Msg::BasicSign, Glyph::LOG_IN),
        ("provider", Msg::ProviderSign, Glyph::SHIELD_CHECK),
        ("expired", Msg::SignInStates, Glyph::CLOCK),
    ]
    .into_iter()
    .map(|(page, msg, glyph)| entry(format!("/sign-in/{key}/{page}"), visit.tr(msg), glyph, true))
    .collect()
}

/// The navigation groups of `brand`: its pages, its page templates and
/// its sign-in cards. The brand itself is chosen in the top bar, so the
/// navigation marks one page.
fn navigation(visit: &Visit, brand: Brand) -> Vec<Group> {
    let mut pages = entries(visit, brand);
    // The honest count (R12): a number on the overlays, and none known for the charts.
    if let Some(overlays) = pages.get_mut(5) {
        overlays.count = Some(Count::Known {
            count: 2,
            words: visit.tr_with(Msg::NavIssues, &[("count", Arg::Number(2.0))]),
        });
    }
    if let Some(charts) = pages.get_mut(6) {
        charts.count = Some(Count::Unknown {
            why: visit.tr(Msg::NavNotProbed),
        });
    }
    vec![
        Group {
            id: "nav-components",
            title: visit.tr(Msg::Components).into(),
            entries: pages,
        },
        Group {
            id: "nav-templates",
            title: visit.tr(Msg::Templates).into(),
            entries: templates(visit, brand),
        },
        Group {
            id: "nav-sign-in",
            title: visit.tr(Msg::SignInGroup).into(),
            entries: sign_ins(visit, brand),
        },
    ]
}

/// The same page as `back` in each brand, as a link and its words.
fn brand_links(visit: &Visit, back: &str) -> Vec<(Brand, String, String)> {
    let page = back
        .strip_prefix('/')
        .and_then(|rest| rest.split_once('/').map(|(_, page)| format!("/{page}")))
        .unwrap_or_default();
    Brand::ALL
        .into_iter()
        .map(|each| {
            let words = visit.tr(match each {
                Brand::Bridge => Msg::Bridge,
                Brand::Fed => Msg::Fed,
                Brand::Ehr => Msg::Ehr,
                Brand::Term => Msg::Term,
                Brand::Pix => Msg::Pix,
            });
            (each, format!("/{}{page}", each.key()), words)
        })
        .collect()
}

/// The brand switcher of the top bar from 40 rem up: the same page in each
/// brand, the one in force marked current.
fn brand_switch(visit: &Visit, back: &str) -> Html {
    let links = brand_links(visit, back);
    let current = links
        .iter()
        .position(|(each, _, _)| *each == visit.brand)
        .unwrap_or(0);
    let choices = links
        .into_iter()
        .map(|(_, href, words)| (href, words))
        .collect();
    segmented_links(visit.tr(Msg::Brand), choices, current)
}

/// The brand switcher as the user menu holds it below 40 rem.
fn brand_choices(visit: &Visit, back: &str) -> Choices {
    Choices {
        label: visit.tr(Msg::Brand),
        choices: brand_links(visit, back)
            .into_iter()
            .map(|(each, href, words)| Choice {
                words,
                href,
                chosen: each == visit.brand,
            })
            .collect(),
    }
}
/// The FHIR version select (M3) the top bar of the visit's page carries:
/// the terminology page's alone.
fn fhir_version(visit: &Visit) -> Option<VersionSelect> {
    visit.path.ends_with("/terminology").then(|| {
        crate::pages::terminology::version_select(
            visit,
            &visit.path,
            visit.param(crate::pages::terminology::version_key()),
        )
    })
}

/// The user menu of the visit's brand.
fn operator_menu(visit: &Visit, version: Option<&VersionSelect>) -> Html {
    let brand = visit.brand;
    let back = &visit.path;
    // A name as long as a FHIR user reference gets, so the top bar shows how
    // it gives way; the synthetic operator the gallery is signed in as.
    let operator = String::from("Practitioner/synthetic-terminologist-on-call");
    let mut choices = vec![brand_choices(visit, back)];
    choices.extend(version.map(VersionSelect::choices));
    user_menu(
        &visit.locale,
        UserMenu {
            id: "user-menu",
            label: operator.clone(),
            name: Some(operator.clone()),
            who: operator,
            line: visit.tr(Msg::GalleryOperatorLine),
            preferences: format!("/{}/preferences", brand.key()),
            links: vec![UserLink {
                words: visit.tr(Msg::Access),
                href: format!("/{}/overlays/access", brand.key()),
                glyph: Glyph::KEY_ROUND,
            }],
            theme: ThemeChoice {
                current: visit.preferences.theme,
                action: String::from(PREFERENCE_ROUTE),
                back: back.clone(),
            },
            choices,
            shortcuts: Some(format!("/{}/overlays", brand.key())),
            sign_out: Some(format!("{SIGN_OUT_ROUTE}/{}", brand.key())),
        },
    )
}

/// The shell of the visit's brand around `content`: the top bar with the
/// brand switch and the operator, the sandbox banner, the navigation, the
/// page, the session watch while a journey's session lasts, and the
/// toasts.
fn brand_shell(visit: &Visit, content: Html, toasts: Html) -> Html {
    let brand = visit.brand;
    let back = &visit.path;
    let locale = &visit.locale;
    let lockup = lockup(
        "/",
        mark(brand_glyph(brand)),
        brand.product(),
        visit.tr(Msg::Gallery),
    );
    let nav = sidebar(
        locale,
        Sidebar {
            id: SIDEBAR_ID,
            label: visit.tr(Msg::Pages),
            groups: navigation(visit, brand),
            current: back.clone(),
            rail: Some(Rail {
                collapsed: rail_of(&visit.cookies),
                action: String::from(SIDEBAR_ROUTE),
                back: back.clone(),
            }),
            nest: crate::pages::federation::nest(visit, brand, back, federation_entry(brand)),
        },
    );
    let pill = status_pill(StatusPill {
        reach: Reach::Up,
        words: visit.tr(Msg::LibraryUp),
        detail: Some(visit.tr_with(Msg::LibraryVersion, &[("version", Arg::from(VERSION))])),
    });
    let version = fhir_version(visit);
    let end = Html::join([
        Html::raw(format!(
            "<span class=\"fui-wide-only\">{}</span>",
            brand_switch(visit, back)
        )),
        version
            .clone()
            .map(|select| version_select(locale, select))
            .unwrap_or_default(),
        pill,
        operator_menu(visit, version.as_ref()),
    ]);
    let typed = visit.param("q").filter(|_| back.ends_with("/find"));
    let search = crate::search::gallery_search(visit, visit.param("search"), typed);
    let content = Html::join([content, crate::session::watch(visit).unwrap_or_default()]);
    shell(
        locale,
        Shell {
            topbar: topbar(
                locale,
                TopBar {
                    sidebar: SIDEBAR_ID,
                    lockup,
                    search: Some(search),
                    end,
                },
            ),
            banner: Some(banner(
                Tone::Warn,
                Glyph::TRIANGLE_ALERT,
                visit.tr(Msg::Gallery),
                &visit.tr(Msg::EveryRowTextThesePages),
                None,
            )),
            sidebar: nav,
            content,
            dock: None,
            toasts,
            footer: footer(visit),
        },
    )
}

/// The gallery's front page: the brands, each a link to its pages.
#[must_use]
pub fn index(visit: &Visit) -> Page {
    let brands = Html::join(Brand::ALL.into_iter().map(|brand| {
        Html::raw(format!(
            "<p><a href=\"/{}\" class=\"fui-button fui-button-secondary\">{}{}</a></p>",
            brand.key(),
            icon(brand_glyph(brand), "16"),
            Html::text(brand.product())
        ))
    }));
    let content = Html::raw(format!(
        "<div class=\"fui-card fui-sign-in\"><div class=\"fui-sign-in-lockup\">{}</div><p class=\"fui-muted\">{}</p><div class=\"fui-stack\">{brands}</div></div>",
        lockup(
            "/",
            mark(Glyph::LAYERS),
            visit.tr(Msg::FerrohealthUi),
            visit.tr(Msg::Gallery)
        ),
        Html::text(visit.tr(Msg::EveryComponentLibraryEveryProduct)),
    ));
    Page::centred(String::new(), content)
}
