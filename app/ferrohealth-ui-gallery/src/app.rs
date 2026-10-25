// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The page tree the server renders and the browser hydrates.
//!
//! Each brand has its pages under its key, `/ferrobridge/…`, drawn in the
//! shell a console draws, and the sign-in cards under `/sign-in/…`. The
//! document links the library's stylesheet and the brand's theme file, which
//! the server serves from the library's constants, and carries the theme the
//! request's cookie names as `data-theme` and the density as
//! `data-density`, so a page arrives in its theme before any script runs,
//! and its locale as `lang`, the locale every component draws its words in.

use ferrohealth_ui::i18n::{Arg, Locale, provide_locale};
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::island::ToastRegion;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::table::model::Paging;
use ferrohealth_ui::theme::{Preferences, Theme, cookie};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::segmented::segmented_links;
use ferrohealth_ui::view::shell::{Footer, Shell, TopBar, centred, lockup, mark, shell, topbar};
use ferrohealth_ui::view::status::{Reach, StatusPill, banner, status_pill};
use ferrohealth_ui::widget::menu::{Choice, Choices, ThemeChoice, UserLink, UserMenu, user_menu};
use ferrohealth_ui::widget::nav::{Count, Entry, Group, Rail, Sidebar, sidebar};
use ferrohealth_ui::widget::toast::RegionWords;
use ferrohealth_ui::widget::version::{VersionSelect, version_select};
use leptos::prelude::*;
use leptos_meta::{Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::hooks::{use_location, use_params_map, use_query_map};
use leptos_router::{Lazy, path};

use crate::cookies::cookies_requested;
use crate::locale::gallery_locale;
use crate::locale::{Msg, tr, tr_with};
use crate::routes::{LAside, LCore, LFederation, LRecords, LVisual};

/// The gallery's name, as its footer and titles name it, in the page's
/// locale.
#[must_use]
pub fn product() -> String {
    tr(Msg::Product)
}

/// The gallery's version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The path of the library's stylesheet the server serves.
pub const STYLESHEET: &str = "/ferrohealth-ui.css";

/// The path of the route every preference posts to, its value under its
/// cookie's name: Your preferences, the user menu's theme and the
/// overview's density.
pub const PREFERENCE_ROUTE: &str = "/preference";

/// The path of the route Reset to defaults posts to.
pub const RESET_ROUTE: &str = "/preferences/reset";

/// What the gallery puts before each preference's cookie name
/// ([`ferrohealth_ui::theme::Preference::COOKIE`]), so `gallery-theme` keeps
/// the theme.
pub const COOKIE_PREFIX: &str = "gallery-";

/// The gallery's own preferences, shown in FerroPIX's Appearance as a
/// console's own rows: how long a revealed identifier stays, and whether
/// leaving the page hides it.
pub const OWN_PREFERENCES: [&str; 2] = ["reveal", "hide"];

/// The path of the route a session listing's pager posts its page and size
/// to.
pub const PAGING_ROUTE: &str = "/paging";

/// The listings that keep their position in the gallery's session, each
/// under the cookie its key names after [`PAGING_COOKIE`].
pub const PAGED_LISTINGS: [&str; 2] = ["files", "ehrs"];

/// The prefix of the cookie a session listing's position is kept in, for
/// the browser's session alone and never in a URL.
pub const PAGING_COOKIE: &str = "gallery-paging-";

/// The path of the route the code editor's form posts its draft to, which
/// answers with the draft it received.
pub const DRAFT_ROUTE: &str = "/draft";

/// The path under which an answer riding its query's form is exported, as
/// `/export/csv` or `/export/json`.
pub const EXPORT_ROUTE: &str = "/export";

/// The path of the route the topology's Probe now posts to, which sends
/// the browser back where it came from.
pub const PROBE_ROUTE: &str = "/probe";

/// The path of the route a member's removal posts to, which returns to the
/// page naming the member removed.
pub const REMOVE_ROUTE: &str = "/remove-member";

/// The path of the route the sidebar's Collapse and Expand post to.
pub const SIDEBAR_ROUTE: &str = "/sidebar";

/// The name of the cookie the sidebar's state is kept in.
pub const SIDEBAR_COOKIE: &str = "gallery-sidebar";

/// The path under which a brand's Sign out posts.
pub const SIGN_OUT_ROUTE: &str = "/sign-out";

/// The id of the navigation, which the drawer button opens.
const SIDEBAR_ID: &str = "sidebar";

/// The path of the theme file of `brand`.
#[must_use]
pub fn theme_path(brand: Brand) -> String {
    format!("/themes/{}.css", brand.key())
}

/// The brand a request path names: the first segment that is a brand's key,
/// FerroBRIDGE for a path that names none.
#[must_use]
pub fn brand_of(path: &str) -> Brand {
    path.split('/')
        .find_map(Brand::parse)
        .unwrap_or(Brand::Bridge)
}

/// The preferences the `Cookie` header value `cookies` keeps, each under
/// its cookie's name after [`COOKIE_PREFIX`].
#[must_use]
pub fn preferences_of(cookies: &str) -> Preferences {
    Preferences::read(cookies, COOKIE_PREFIX)
}

/// The theme the `Cookie` header value `cookies` names, the system theme for
/// none.
#[must_use]
pub fn theme_of(cookies: &str) -> Theme {
    preferences_of(cookies).theme
}

/// The locale the language preference names: the pseudo-locale when it is
/// chosen, and English, the language the gallery serves every browser, for
/// anything else.
#[must_use]
pub fn locale_of(cookies: &str) -> Locale {
    gallery_locale(preferences_of(cookies).language.tag().unwrap_or_default())
}

/// The value of the gallery's own preference `name`, one of
/// [`OWN_PREFERENCES`], that the `Cookie` header value `cookies` keeps.
#[must_use]
pub fn own_of<'a>(cookies: &'a str, name: &str) -> Option<&'a str> {
    cookie(cookies, &format!("{COOKIE_PREFIX}{name}"))
}

/// The paging of the session listing `listing` that the `Cookie` header
/// value `cookies` keeps, as `<page>.<size>`, the first page for none.
#[must_use]
pub fn paging_of(cookies: &str, listing: &str) -> Paging {
    let kept = cookie(cookies, &format!("{PAGING_COOKIE}{listing}"))
        .and_then(|value| value.split_once('.'));
    Paging::read(kept.map(|(page, _)| page), kept.map(|(_, size)| size))
}

/// The paging of the session listing `listing` the request keeps.
///
/// Both halves read it from the cookies the page knows
/// ([`crate::cookies::Cookies`]).
#[must_use]
pub fn session_paging(listing: &'static str) -> Paging {
    paging_of(&cookies_requested(), listing)
}

/// The whole HTML document the server renders around [`App`], in the theme
/// the request's cookie names and with the theme file of the brand its path
/// names.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn document(options: LeptosOptions) -> impl IntoView {
    let parts = use_context::<http::request::Parts>();
    let cookies = parts
        .as_ref()
        .and_then(|parts| parts.headers.get(http::header::COOKIE))
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let preferences = preferences_of(&cookies);
    let theme = preferences.theme.key();
    let density = preferences.density.key();
    let motion = preferences.motion.key();
    let hints = preferences.hints.key();
    let lang = locale_of(&cookies).tag();
    view! {
        <!DOCTYPE html>
        <html lang=lang data-theme=theme data-density=density data-motion=motion data-hints=hints>
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
                <link rel="stylesheet" href=STYLESHEET />
                <HydrationScripts options />
                <leptos_meta::MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

/// The theme the request names, which the server decided and the browser
/// reads serialized, so both halves draw the same switch.
fn theme_requested() -> Theme {
    theme_of(&cookies_requested())
}

/// Whether the `Cookie` header value `cookies` asks for the sidebar's rail.
#[must_use]
pub fn rail_of(cookies: &str) -> bool {
    cookie(cookies, SIDEBAR_COOKIE) == Some("rail")
}

/// Whether the request asks for the sidebar's rail, decided as the theme is.
fn rail_requested() -> bool {
    rail_of(&cookies_requested())
}

/// The tag of the locale the request names, decided as the theme is.
fn locale_requested() -> String {
    locale_of(&cookies_requested()).tag()
}

/// Marks the root element `data-hydrated` once the page has hydrated.
fn mark_hydrated() {
    #[cfg(target_arch = "wasm32")]
    if let Some(root) = document().document_element()
        && let Err(error) = root.set_attribute("data-hydrated", "")
    {
        leptos::logging::warn!("the browser refused the hydrated mark: {error:?}");
    }
}

/// The theme in force, shared with the pages.
#[derive(Debug, Clone, Copy)]
pub struct InForce(pub Theme);

/// The root component: the routes, the toast region and the title.
///
/// The server draws it and the browser hydrates it whole; each page is a
/// lazy route ([`crate::routes`]), and the browser marks the root element
/// `data-hydrated` once the page has hydrated, which the screenshot capture
/// waits for.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    // NOTE: the Leptos book, `ssr/22_life_cycle`: the cookies the server read reach the
    // hydrating page serialized, so both halves draw the same page.
    let cookies = RwSignal::new((*SharedValue::new_str(crate::cookies::server_cookies)).clone());
    provide_context(crate::cookies::Cookies(cookies));
    ferrohealth_ui::widget::prefetch::routes(crate::routes::ROUTES);
    Effect::new(move |_| mark_hydrated());
    // NOTE: the Leptos book, `ssr/22_life_cycle`: a value the server decides reaches
    // the hydrating page serialized, so both halves draw the same page.
    let theme = *SharedValue::new_str(theme_requested);
    provide_context(InForce(theme));
    let tag = SharedValue::new_str(locale_requested);
    provide_locale(gallery_locale(&tag));
    view! {
        <Title formatter=crate::pages::title_format(product()) text=String::new() />
        <Router>
            <crate::cookies::FollowCookies />
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/") view=Index />
                <Route path=path!("/sign-in/:brand/:variant") view={Lazy::<LAside>::new()} />
                <Route path=path!("/fault/:brand") view={Lazy::<LAside>::new()} />
                <ParentRoute path=path!("/:brand") view=BrandShell>
                    <Route path=path!("") view={Lazy::<LCore>::new()} />
                    <Route path=path!("forms") view={Lazy::<LCore>::new()} />
                    <Route path=path!("tables") view={Lazy::<LCore>::new()} />
                    <Route path=path!("code") view={Lazy::<LCore>::new()} />
                    <Route path=path!("configuration") view={Lazy::<LCore>::new()} />
                    <Route path=path!("configuration/provisioned") view={Lazy::<LCore>::new()} />
                    <Route path=path!("overlays") view={Lazy::<LFederation>::new()} />
                    <Route path=path!("overlays/access") view={Lazy::<LFederation>::new()} />
                    <Route path=path!("charts") view={Lazy::<LVisual>::new()} />
                    <Route path=path!("layouts") view={Lazy::<LVisual>::new()} />
                    <Route path=path!("operations") view={Lazy::<LRecords>::new()} />
                    <Route path=path!("clinical") view={Lazy::<LRecords>::new()} />
                    <Route path=path!("terminology") view={Lazy::<LRecords>::new()} />
                    <Route path=path!("federation") view={Lazy::<LFederation>::new()} />
                    <Route path=path!("find") view={Lazy::<LCore>::new()} />
                    <Route path=path!("federation/:part") view={Lazy::<LFederation>::new()} />
                    <Route path=path!("preferences") view={Lazy::<LCore>::new()} />
                    <Route path=path!("preferences/:section") view={Lazy::<LCore>::new()} />
                    <Route path=path!("templates/:variant") view={Lazy::<LAside>::new()} />
                    <Route path=path!("trees") view={Lazy::<LVisual>::new()} />
                </ParentRoute>
            </Routes>
        </Router>
        <ToastRegion words=RegionWords::of_page() flash=Vec::new() />
    }
}

/// The footer every gallery page carries.
#[must_use]
pub fn footer() -> Footer {
    Footer {
        product: product().into(),
        version: VERSION.into(),
        about: "/".into(),
    }
}

/// The brand the route's `:brand` names, when it names one.
#[must_use]
pub fn route_brand() -> Option<Brand> {
    let params = use_params_map();
    params.with_untracked(|params| params.get_str("brand").and_then(Brand::parse))
}

/// The entries of the navigation of the brand `key`.
macro_rules! entries {
    ($key:literal) => {
        vec![
            Entry {
                href: concat!("/", $key).into(),
                label: tr(Msg::Overview).into(),
                glyph: Glyph::LAYOUT_DASHBOARD,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/forms").into(),
                label: tr(Msg::FormsEditor).into(),
                glyph: Glyph::PENCIL,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/tables").into(),
                label: tr(Msg::TablesPaging).into(),
                glyph: Glyph::DATABASE,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/code").into(),
                label: tr(Msg::Documents).into(),
                glyph: Glyph::SQUARE_TERMINAL,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/configuration").into(),
                label: tr(Msg::Configuration).into(),
                glyph: Glyph::SETTINGS,
                exact: false,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/overlays").into(),
                label: tr(Msg::Overlays).into(),
                glyph: Glyph::LAYERS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/charts").into(),
                label: tr(Msg::Charts).into(),
                glyph: Glyph::CHART_LINE,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/layouts").into(),
                label: tr(Msg::Layouts).into(),
                glyph: Glyph::PANEL_RIGHT,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/operations").into(),
                label: tr(Msg::Operations).into(),
                glyph: Glyph::FLASK_CONICAL,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/terminology").into(),
                label: tr(Msg::Terminology).into(),
                glyph: Glyph::BOOK_A,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/trees").into(),
                label: tr(Msg::Trees).into(),
                glyph: Glyph::FOLDER_OPEN,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/find").into(),
                label: tr(Msg::FindTitle).into(),
                glyph: Glyph::SEARCH,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/clinical").into(),
                label: tr(Msg::ClinicalRecords).into(),
                glyph: Glyph::HEART_PULSE,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/federation").into(),
                label: tr(Msg::Federation).into(),
                glyph: Glyph::NETWORK,
                exact: false,
                count: None,
            },
        ]
    };
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

/// The page template entries of the brand `key`.
macro_rules! templates {
    ($key:literal) => {
        vec![
            Entry {
                href: concat!("/", $key, "/templates/list").into(),
                label: tr(Msg::NavList).into(),
                glyph: Glyph::LIST_CHECKS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/list-many").into(),
                label: tr(Msg::NavListMany).into(),
                glyph: Glyph::LIST_CHECKS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/list-empty").into(),
                label: tr(Msg::NavListEmpty).into(),
                glyph: Glyph::INBOX,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/list-tabs").into(),
                label: tr(Msg::NavListTabs).into(),
                glyph: Glyph::LIST_CHECKS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/detail").into(),
                label: tr(Msg::NavDetail).into(),
                glyph: Glyph::FILE_TEXT,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/settings").into(),
                label: tr(Msg::NavSettings).into(),
                glyph: Glyph::SETTINGS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/settings-managed").into(),
                label: tr(Msg::NavSettingsManaged).into(),
                glyph: Glyph::SLIDERS_HORIZONTAL,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/preferences").into(),
                label: tr(Msg::NavPreferences).into(),
                glyph: Glyph::CIRCLE_USER,
                exact: false,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/about").into(),
                label: tr(Msg::NavAbout).into(),
                glyph: Glyph::INFO,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/forbidden").into(),
                label: tr(Msg::NavForbidden).into(),
                glyph: Glyph::SHIELD_OFF,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/not-found").into(),
                label: tr(Msg::NavNotFound).into(),
                glyph: Glyph::COMPASS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/fault/", $key).into(),
                label: tr(Msg::NavFault).into(),
                glyph: Glyph::SERVER_CRASH,
                exact: true,
                count: None,
            },
        ]
    };
}

/// The sign-in entries of the brand `key`.
macro_rules! sign_ins {
    ($key:literal) => {
        vec![
            Entry {
                href: concat!("/sign-in/", $key, "/basic").into(),
                label: tr(Msg::BasicSign).into(),
                glyph: Glyph::LOG_IN,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/sign-in/", $key, "/provider").into(),
                label: tr(Msg::ProviderSign).into(),
                glyph: Glyph::SHIELD_CHECK,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/sign-in/", $key, "/expired").into(),
                label: tr(Msg::SignInStates).into(),
                glyph: Glyph::CLOCK,
                exact: true,
                count: None,
            },
        ]
    };
}

/// The navigation groups of `brand`: its pages, its page templates and
/// its sign-in cards. The brand itself is chosen in the top bar, so the
/// navigation marks one page.
fn navigation(brand: Brand) -> [Group; 3] {
    let (mut pages, layouts, signs): (Vec<Entry>, Vec<Entry>, Vec<Entry>) = match brand {
        Brand::Bridge => (
            entries!("ferrobridge"),
            templates!("ferrobridge"),
            sign_ins!("ferrobridge"),
        ),
        Brand::Fed => (
            entries!("ferrofed"),
            templates!("ferrofed"),
            sign_ins!("ferrofed"),
        ),
        Brand::Ehr => (
            entries!("ferroehr"),
            templates!("ferroehr"),
            sign_ins!("ferroehr"),
        ),
        Brand::Term => (
            entries!("ferroterm"),
            templates!("ferroterm"),
            sign_ins!("ferroterm"),
        ),
        Brand::Pix => (
            entries!("ferropix"),
            templates!("ferropix"),
            sign_ins!("ferropix"),
        ),
    };
    // The honest count (R12): a number on the overlays, and none known for the charts.
    if let Some(overlays) = pages.get_mut(4) {
        overlays.count = Some(Count::Known {
            count: 2,
            words: tr_with(Msg::NavIssues, &[("count", Arg::Number(2.0))]),
        });
    }
    if let Some(charts) = pages.get_mut(5) {
        charts.count = Some(Count::Unknown {
            why: tr(Msg::NavNotProbed),
        });
    }
    [
        Group {
            id: "nav-components",
            title: tr(Msg::Components).into(),
            entries: pages,
        },
        Group {
            id: "nav-templates",
            title: tr(Msg::Templates).into(),
            entries: layouts,
        },
        Group {
            id: "nav-sign-in",
            title: tr(Msg::SignInGroup).into(),
            entries: signs,
        },
    ]
}

/// The same page as `back` in each brand, as a link and its words.
fn brand_links(back: &str) -> Vec<(Brand, String, String)> {
    let page = back
        .strip_prefix('/')
        .and_then(|rest| rest.split_once('/').map(|(_, page)| format!("/{page}")))
        .unwrap_or_default();
    Brand::ALL
        .into_iter()
        .map(|each| {
            let words = match each {
                Brand::Bridge => tr(Msg::Bridge),
                Brand::Fed => tr(Msg::Fed),
                Brand::Ehr => tr(Msg::Ehr),
                Brand::Term => tr(Msg::Term),
                Brand::Pix => tr(Msg::Pix),
            };
            (each, format!("/{}{page}", each.key()), words)
        })
        .collect()
}

/// The brand switcher of the top bar from 40 rem up: the same page in each
/// brand, the one in force marked current.
fn brand_switch(brand: Brand, back: &str) -> impl IntoView + use<> {
    let links = brand_links(back);
    let current = links
        .iter()
        .position(|(each, _, _)| *each == brand)
        .unwrap_or(0);
    let choices = links
        .into_iter()
        .map(|(_, href, words)| (href, words))
        .collect();
    segmented_links(tr(Msg::Brand), choices, current)
}

/// The brand switcher as the user menu holds it below 40 rem.
fn brand_choices(brand: Brand, back: &str) -> Choices {
    Choices {
        label: tr(Msg::Brand),
        choices: brand_links(back)
            .into_iter()
            .map(|(each, href, words)| Choice {
                words,
                href,
                chosen: each == brand,
            })
            .collect(),
    }
}

/// The glyph a brand's mark draws in the gallery.
#[must_use]
pub const fn brand_glyph(brand: Brand) -> Glyph {
    match brand {
        Brand::Bridge => Glyph::LAYERS,
        Brand::Fed => Glyph::USERS,
        Brand::Ehr => Glyph::HEART_PULSE,
        Brand::Term => Glyph::LIST_CHECKS,
        Brand::Pix => Glyph::LINK,
    }
}

/// The shell of a brand's pages: the top bar with the theme switch and the
/// operator, the sandbox banner, the navigation and the page.
///
/// The shell is drawn again when the brand or the page changes, so the
/// brand's theme file and the page the theme switch returns to follow a
/// navigation the router makes in the browser.
#[component]
fn BrandShell() -> impl IntoView {
    let params = use_params_map();
    let location = use_location();
    let query = use_query_map();
    let at = Memo::new(move |_| {
        let brand = params.with(|params| params.get_str("brand").and_then(Brand::parse));
        let search = query.with(|query| query.get("search"));
        let fhir = query.with(|query| query.get(crate::pages::terminology::version_key()));
        let typed = query.with(|query| query.get("q"));
        (brand, location.pathname.get(), search, fhir, typed)
    });
    move || match at.get() {
        (Some(brand), back, search, fhir, typed) => {
            let typed = typed.filter(|_| back.ends_with("/find"));
            brand_shell(
                brand,
                back,
                Searched {
                    state: search.as_deref(),
                    typed: typed.as_deref(),
                },
                fhir.as_deref(),
            )
        }
        (None, _, _, _, _) => view! { <NotFound /> }.into_any(),
    }
}

/// The FHIR version select (M3) the top bar of the page at `back` carries,
/// `chosen` in view: the terminology page's alone.
fn fhir_version(back: &str, chosen: Option<&str>) -> Option<VersionSelect> {
    back.ends_with("/terminology")
        .then(|| crate::pages::terminology::version_select(back, chosen))
}

/// What the top bar's search shows: the state the address names and the
/// words the find page answers.
#[derive(Debug, Clone, Copy)]
struct Searched<'a> {
    /// The state, `busy` or `disabled`.
    state: Option<&'a str>,
    /// The words the find page answers.
    typed: Option<&'a str>,
}

/// The shell of `brand`'s pages, whose theme switch returns to `back`, its
/// search `search` and its FHIR version `fhir`.
fn brand_shell(brand: Brand, back: String, search: Searched<'_>, fhir: Option<&str>) -> AnyView {
    let lockup = lockup(
        "/",
        mark(brand_glyph(brand)),
        brand.product(),
        tr(Msg::Gallery),
    )
    .into_any();
    let switch = brand_switch(brand, &back).into_any();
    let collapsed = rail_requested();
    let nav = sidebar(Sidebar {
        id: SIDEBAR_ID,
        label: tr(Msg::Pages),
        groups: navigation(brand).into(),
        current: back.clone(),
        rail: Some(Rail {
            collapsed,
            action: String::from(SIDEBAR_ROUTE),
            back: back.clone(),
        }),
        nest: crate::pages::federation::nest(brand, &back, federation_entry(brand)),
    });
    let theme = use_context::<InForce>().map_or(Theme::System, |in_force| in_force.0);
    // A name as long as a FHIR user reference gets, so the top bar shows how
    // it gives way; the synthetic operator the gallery is signed in as.
    let operator = String::from("Practitioner/synthetic-terminologist-on-call");
    let version = fhir_version(&back, fhir);
    let mut choices = vec![brand_choices(brand, &back)];
    choices.extend(version.as_ref().map(VersionSelect::choices));
    let menu = user_menu(UserMenu {
        id: "user-menu",
        label: operator.clone(),
        name: Some(operator.clone()),
        who: operator,
        line: tr(Msg::GalleryOperatorLine),
        preferences: format!("/{}/preferences", brand.key()),
        links: vec![UserLink {
            words: tr(Msg::Access),
            href: format!("/{}/overlays/access", brand.key()),
            glyph: Glyph::KEY_ROUND,
        }],
        theme: ThemeChoice {
            current: theme,
            action: String::from(PREFERENCE_ROUTE),
            back,
        },
        choices,
        shortcuts: Some(format!("/{}/overlays", brand.key())),
        sign_out: Some(format!("{SIGN_OUT_ROUTE}/{}", brand.key())),
    });
    let pill = status_pill(StatusPill {
        reach: Reach::Up,
        words: tr(Msg::LibraryUp),
        detail: Some(tr_with(
            Msg::LibraryVersion,
            &[("version", Arg::from(VERSION))],
        )),
    });
    let end = view! {
        <span class="fui-wide-only">{switch}</span>
        {version.map(version_select)}
        {pill}
        {menu}
    }
    .into_any();
    let search = crate::search::gallery_search(brand, search.state, search.typed);
    let page = shell(Shell {
        topbar: topbar(TopBar {
            sidebar: SIDEBAR_ID,
            lockup,
            search: Some(search),
            end,
        }),
        banner: Some(
            banner(
                Tone::Warn,
                Glyph::TRIANGLE_ALERT,
                tr(Msg::Gallery),
                &tr(Msg::EveryRowTextThesePages),
                None,
            )
            .into_any(),
        ),
        sidebar: nav,
        content: view! { <Outlet /> }.into_any(),
        dock: None,
        footer: footer(),
    });
    view! {
        <Stylesheet id="brand-theme" href=theme_path(brand) />
        {page}
    }
    .into_any()
}

/// The gallery's front page: the brands, each a link to its pages.
#[component]
fn Index() -> impl IntoView {
    let brands = Brand::ALL
        .into_iter()
        .map(|brand| {
            view! {
                <p>
                    <a href=format!("/{}", brand.key()) class="fui-button fui-button-secondary">
                        {ferrohealth_ui::icon::icon(brand_glyph(brand), "16")}
                        {brand.product()}
                    </a>
                </p>
            }
        })
        .collect_view()
        .into_any();
    let content = view! {
        <div class="fui-card fui-sign-in">
            <div class="fui-sign-in-lockup">
                {lockup("/", mark(Glyph::LAYERS), tr(Msg::FerrohealthUi), tr(Msg::Gallery))}
            </div>
            <p class="fui-muted">{tr(Msg::EveryComponentLibraryEveryProduct)}</p>
            <div class="fui-stack">{brands}</div>
        </div>
    }
    .into_any();
    centred(content, footer())
}

/// The page for a path the gallery does not serve: `404`, and the way back.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub(crate) fn NotFound() -> impl IntoView {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
        response.set_status(http::StatusCode::NOT_FOUND);
    }
    let content = ferrohealth_ui::view::empty::empty_state(
        Glyph::COMPASS,
        tr(Msg::NotFound),
        tr(Msg::NotFoundBody),
        view! {
            <a href="/" class="fui-button fui-button-primary">
                {ferrohealth_ui::icon::icon(Glyph::ARROW_LEFT, "16")}
                {tr(Msg::BackGallery)}
            </a>
        },
    );
    view! {
        <Title text=tr(Msg::NotFound) />
        {centred(view! { <div class="fui-card">{content}</div> }.into_any(), footer())}
    }
}

#[cfg(test)]
mod tests {
    use ferrohealth_ui::style::Brand;
    use ferrohealth_ui::theme::Theme;

    use super::{brand_of, theme_of};

    #[test]
    fn the_brand_and_the_theme_are_read_from_the_request() {
        assert_eq!(Brand::Ehr, brand_of("/ferroehr/tables"));
        assert_eq!(Brand::Term, brand_of("/sign-in/ferroterm/basic"));
        assert_eq!(Brand::Bridge, brand_of("/"));
        assert_eq!(Theme::Dark, theme_of("a=1; gallery-theme=dark"));
        assert_eq!(Theme::System, theme_of("gallery-theme=sepia"));
        assert_eq!(Theme::System, theme_of(""));
    }
}
