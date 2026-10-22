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
use ferrohealth_ui::theme::{Density, Theme};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::segmented::segmented_links;
use ferrohealth_ui::view::shell::{Footer, Shell, TopBar, centred, lockup, mark, shell, topbar};
use ferrohealth_ui::view::status::{Reach, StatusPill, banner, status_pill};
use ferrohealth_ui::widget::menu::{Choice, Choices, ThemeChoice, UserMenu, user_menu};
use ferrohealth_ui::widget::nav::{Count, Entry, Group, Rail, Sidebar, sidebar};
use ferrohealth_ui::widget::toast::RegionWords;
use leptos::prelude::*;
use leptos_meta::{Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::hooks::{use_location, use_params_map, use_query_map};
use leptos_router::path;

use crate::locale::gallery_locale;
use crate::locale::{Msg, tr, tr_with};

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

/// The path of the route a theme choice posts to.
pub const THEME_ROUTE: &str = "/theme";

/// The name of the cookie the theme choice is kept in.
pub const THEME_COOKIE: &str = "gallery-theme";

/// The path of the route a density choice posts to.
pub const DENSITY_ROUTE: &str = "/density";

/// The name of the cookie the density choice is kept in.
pub const DENSITY_COOKIE: &str = "gallery-density";

/// The path of the route a language choice posts to.
pub const LOCALE_ROUTE: &str = "/locale";

/// The name of the cookie the language choice is kept in.
pub const LOCALE_COOKIE: &str = "gallery-locale";

/// The path of the route a session listing's pager posts its page and size
/// to.
pub const PAGING_ROUTE: &str = "/paging";

/// The listings that keep their position in the gallery's session, each
/// under the cookie its key names after [`PAGING_COOKIE`].
pub const PAGED_LISTINGS: [&str; 2] = ["files", "ehrs"];

/// The prefix of the cookie a session listing's position is kept in, for
/// the browser's session alone and never in a URL.
pub const PAGING_COOKIE: &str = "gallery-paging-";

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

/// The theme the `Cookie` header value `cookies` names, the system theme for
/// none.
#[must_use]
pub fn theme_of(cookies: &str) -> Theme {
    cookie(cookies, THEME_COOKIE)
        .and_then(Theme::parse)
        .unwrap_or_default()
}

/// The density the `Cookie` header value `cookies` names, the comfortable
/// density for none.
#[must_use]
pub fn density_of(cookies: &str) -> Density {
    cookie(cookies, DENSITY_COOKIE)
        .and_then(Density::parse)
        .unwrap_or_default()
}

/// The locale the `Cookie` header value `cookies` names, English for none.
#[must_use]
pub fn locale_of(cookies: &str) -> Locale {
    gallery_locale(cookie(cookies, LOCALE_COOKIE).unwrap_or_default())
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
/// The tables page is server HTML with no island around its listings, so
/// only the server draws it and reads the cookie.
#[must_use]
pub fn session_paging(listing: &'static str) -> Paging {
    #[cfg(not(target_arch = "wasm32"))]
    {
        paging_of(&request_cookies(), listing)
    }
    // NOTE: no specification governs this: our own design; the browser never draws
    // a server-only page, so its half names the first page.
    #[cfg(target_arch = "wasm32")]
    {
        let _: &str = listing;
        Paging::default()
    }
}

/// The value of the cookie `name` in the `Cookie` header value `cookies`.
fn cookie<'a>(cookies: &'a str, name: &str) -> Option<&'a str> {
    // NOTE: RFC 6265 §5.4: cookies arrive as `name=value` pairs joined by `; `, and
    // a value that names no choice is the choice not made.
    cookies
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
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
    let theme = theme_of(&cookies).key();
    let density = density_of(&cookies).key();
    let lang = locale_of(&cookies).tag();
    view! {
        <!DOCTYPE html>
        <html lang=lang data-theme=theme data-density=density>
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
                <link rel="stylesheet" href=STYLESHEET />
                <HydrationScripts options islands=true />
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
    #[cfg(not(target_arch = "wasm32"))]
    {
        theme_of(&request_cookies())
    }
    // NOTE: the browser reads the value the server decided; this runs only if
    // that value is missing, and then marks the theme that follows the system.
    #[cfg(target_arch = "wasm32")]
    {
        Theme::default()
    }
}

/// Whether the `Cookie` header value `cookies` asks for the sidebar's rail.
#[must_use]
pub fn rail_of(cookies: &str) -> bool {
    cookie(cookies, SIDEBAR_COOKIE) == Some("rail")
}

/// Whether the request asks for the sidebar's rail, decided as the theme is.
fn rail_requested() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        rail_of(&request_cookies())
    }
    #[cfg(target_arch = "wasm32")]
    {
        false
    }
}

/// The density the request names, decided as the theme is.
fn density_requested() -> Density {
    #[cfg(not(target_arch = "wasm32"))]
    {
        density_of(&request_cookies())
    }
    #[cfg(target_arch = "wasm32")]
    {
        Density::default()
    }
}

/// The tag of the locale the request names, decided as the theme is.
fn locale_requested() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        locale_of(&request_cookies()).tag()
    }
    #[cfg(target_arch = "wasm32")]
    {
        Locale::english().tag()
    }
}

/// The request's `Cookie` header, empty for none.
#[cfg(not(target_arch = "wasm32"))]
fn request_cookies() -> String {
    use_context::<http::request::Parts>()
        .and_then(|parts| {
            parts
                .headers
                .get(http::header::COOKIE)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned)
        })
        .unwrap_or_default()
}

/// The theme in force, shared with the pages.
#[derive(Debug, Clone, Copy)]
pub struct InForce(pub Theme);

/// The density in force, shared with the pages.
#[derive(Debug, Clone, Copy)]
pub struct DensityInForce(pub Density);

/// The root component: the routes, the toast region and the title.
///
/// It runs on the server alone: the gallery renders in islands mode, so
/// the browser hydrates the islands of a page and nothing else.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    // NOTE: the Leptos book, `ssr/22_life_cycle`: a value the server decides reaches
    // the hydrating page serialized, so both halves draw the same page.
    let theme = *SharedValue::new_str(theme_requested);
    provide_context(InForce(theme));
    let density = *SharedValue::new_str(density_requested);
    provide_context(DensityInForce(density));
    let tag = SharedValue::new_str(locale_requested);
    provide_locale(gallery_locale(&tag));
    view! {
        <Title formatter=crate::pages::title_format(product()) text=String::new() />
        <Router>
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/") view=Index />
                <Route path=path!("/sign-in/:brand/:variant") view=crate::pages::sign_in::SignInPage />
                <Route path=path!("/fault/:brand") view=crate::pages::templates::FaultPage />
                <ParentRoute path=path!("/:brand") view=BrandShell>
                    <Route path=path!("") view=crate::pages::overview::OverviewPage />
                    <Route path=path!("forms") view=crate::pages::forms::FormsPage />
                    <Route path=path!("tables") view=crate::pages::tables::TablesPage />
                    <Route path=path!("code") view=crate::pages::code::CodePage />
                    <Route path=path!("overlays") view=crate::pages::overlays::OverlaysPage />
                    <Route path=path!("charts") view=crate::pages::charts::ChartsPage />
                    <Route path=path!("layouts") view=crate::pages::layouts::LayoutsPage />
                    <Route
                        path=path!("templates/:variant")
                        view=crate::pages::templates::TemplatePage
                    />
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
        version: VERSION,
        about: "/",
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
                href: concat!("/", $key),
                label: tr(Msg::Overview).into(),
                glyph: Glyph::LAYOUT_DASHBOARD,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/forms"),
                label: tr(Msg::FormsEditor).into(),
                glyph: Glyph::PENCIL,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/tables"),
                label: tr(Msg::TablesPaging).into(),
                glyph: Glyph::DATABASE,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/code"),
                label: tr(Msg::Documents).into(),
                glyph: Glyph::SQUARE_TERMINAL,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/overlays"),
                label: tr(Msg::Overlays).into(),
                glyph: Glyph::LAYERS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/charts"),
                label: tr(Msg::Charts).into(),
                glyph: Glyph::CHART_LINE,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/layouts"),
                label: tr(Msg::Layouts).into(),
                glyph: Glyph::PANEL_RIGHT,
                exact: true,
                count: None,
            },
        ]
    };
}

/// The page template entries of the brand `key`.
macro_rules! templates {
    ($key:literal) => {
        vec![
            Entry {
                href: concat!("/", $key, "/templates/list"),
                label: tr(Msg::NavList).into(),
                glyph: Glyph::LIST_CHECKS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/list-many"),
                label: tr(Msg::NavListMany).into(),
                glyph: Glyph::LIST_CHECKS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/list-empty"),
                label: tr(Msg::NavListEmpty).into(),
                glyph: Glyph::INBOX,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/list-tabs"),
                label: tr(Msg::NavListTabs).into(),
                glyph: Glyph::LIST_CHECKS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/detail"),
                label: tr(Msg::NavDetail).into(),
                glyph: Glyph::FILE_TEXT,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/settings"),
                label: tr(Msg::NavSettings).into(),
                glyph: Glyph::SETTINGS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/settings-managed"),
                label: tr(Msg::NavSettingsManaged).into(),
                glyph: Glyph::SLIDERS_HORIZONTAL,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/about"),
                label: tr(Msg::NavAbout).into(),
                glyph: Glyph::INFO,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/forbidden"),
                label: tr(Msg::NavForbidden).into(),
                glyph: Glyph::SHIELD_OFF,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/", $key, "/templates/not-found"),
                label: tr(Msg::NavNotFound).into(),
                glyph: Glyph::COMPASS,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/fault/", $key),
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
                href: concat!("/sign-in/", $key, "/basic"),
                label: tr(Msg::BasicSign).into(),
                glyph: Glyph::LOG_IN,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/sign-in/", $key, "/provider"),
                label: tr(Msg::ProviderSign).into(),
                glyph: Glyph::SHIELD_CHECK,
                exact: true,
                count: None,
            },
            Entry {
                href: concat!("/sign-in/", $key, "/expired"),
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
        (brand, location.pathname.get(), search)
    });
    move || match at.get() {
        (Some(brand), back, search) => brand_shell(brand, back, search.as_deref()),
        (None, _, _) => view! { <NotFound /> }.into_any(),
    }
}

/// The shell of `brand`'s pages, whose theme switch returns to `back`, its
/// search in the state `search` names.
fn brand_shell(brand: Brand, back: String, search: Option<&str>) -> AnyView {
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
    });
    let theme = use_context::<InForce>().map_or(Theme::System, |in_force| in_force.0);
    // A name as long as a FHIR user reference gets, so the top bar shows how
    // it gives way; the synthetic operator the gallery is signed in as.
    let operator = String::from("Practitioner/synthetic-terminologist-on-call");
    let choices = brand_choices(brand, &back);
    let menu = user_menu(UserMenu {
        id: "user-menu",
        label: operator.clone(),
        name: Some(operator.clone()),
        who: operator,
        line: tr(Msg::GalleryOperatorLine),
        preferences: format!("/{}", brand.key()),
        theme: ThemeChoice {
            current: theme,
            action: String::from(THEME_ROUTE),
            back,
        },
        choices: Some(choices),
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
        {pill}
        {menu}
    }
    .into_any();
    let search = crate::search::gallery_search(brand, search);
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
            <p class="fui-muted">
                {tr(Msg::EveryComponentLibraryEveryProduct)}
            </p>
            <div class="fui-stack">{brands}</div>
        </div>
    }
    .into_any();
    centred(content, footer())
}

/// The page for a path the gallery does not serve: `404`, and the way back.
#[component]
fn NotFound() -> impl IntoView {
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
