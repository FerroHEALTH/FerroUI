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

use ferrohealth_ui::i18n::{Locale, provide_locale};
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::theme::{Density, Theme};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::menu::{OperatorMenu, operator_menu};
use ferrohealth_ui::view::search::{SearchBox, search_box};
use ferrohealth_ui::view::segmented::segmented_links;
use ferrohealth_ui::view::shell::{
    Footer, Shell, centred, drawer_button, lockup, mark, shell, topbar_with,
};
use ferrohealth_ui::view::status::{banner, pill};
use ferrohealth_ui::widget::nav::{Entry, Group, sidebar};
use ferrohealth_ui::widget::toast::{provide_toasts, toast_region};
use leptos::prelude::*;
use leptos_meta::{Stylesheet, Title, provide_meta_context};
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::hooks::{use_location, use_params_map};
use leptos_router::path;

use crate::locale::gallery_locale;

/// The gallery's name, as its footer and titles name it.
pub const PRODUCT: &str = "FerroHEALTH UI Gallery";

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
    let toasts = provide_toasts();
    view! {
        <Title text=PRODUCT />
        <Router>
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/") view=Index />
                <Route path=path!("/sign-in/:brand/:variant") view=crate::pages::sign_in::SignInPage />
                <ParentRoute path=path!("/:brand") view=BrandShell>
                    <Route path=path!("") view=crate::pages::overview::OverviewPage />
                    <Route path=path!("forms") view=crate::pages::forms::FormsPage />
                    <Route path=path!("tables") view=crate::pages::tables::TablesPage />
                    <Route path=path!("code") view=crate::pages::code::CodePage />
                    <Route path=path!("overlays") view=crate::pages::overlays::OverlaysPage />
                    <Route path=path!("charts") view=crate::pages::charts::ChartsPage />
                    <Route path=path!("layouts") view=crate::pages::layouts::LayoutsPage />
                </ParentRoute>
            </Routes>
        </Router>
        {toast_region(toasts)}
    }
}

/// The footer every gallery page carries.
pub const FOOTER: Footer = Footer {
    product: PRODUCT,
    version: VERSION,
    about: "/",
};

/// The brand the route's `:brand` names, when it names one.
#[must_use]
pub fn route_brand() -> Option<Brand> {
    let params = use_params_map();
    params.with_untracked(|params| params.get_str("brand").and_then(Brand::parse))
}

/// The entries of the navigation of the brand `key`.
macro_rules! entries {
    ($key:literal) => {
        &[
            Entry {
                href: concat!("/", $key),
                label: "Overview",
                glyph: Glyph::LAYOUT_DASHBOARD,
                exact: true,
            },
            Entry {
                href: concat!("/", $key, "/forms"),
                label: "Forms and editor",
                glyph: Glyph::PENCIL,
                exact: true,
            },
            Entry {
                href: concat!("/", $key, "/tables"),
                label: "Tables and paging",
                glyph: Glyph::DATABASE,
                exact: true,
            },
            Entry {
                href: concat!("/", $key, "/code"),
                label: "Documents",
                glyph: Glyph::SQUARE_TERMINAL,
                exact: true,
            },
            Entry {
                href: concat!("/", $key, "/overlays"),
                label: "Overlays",
                glyph: Glyph::LAYERS,
                exact: true,
            },
            Entry {
                href: concat!("/", $key, "/charts"),
                label: "Charts",
                glyph: Glyph::CHART_LINE,
                exact: true,
            },
            Entry {
                href: concat!("/", $key, "/layouts"),
                label: "Layouts",
                glyph: Glyph::PANEL_RIGHT,
                exact: true,
            },
        ]
    };
}

/// The sign-in entries of the brand `key`.
macro_rules! sign_ins {
    ($key:literal) => {
        &[
            Entry {
                href: concat!("/sign-in/", $key, "/basic"),
                label: "Basic sign-in",
                glyph: Glyph::LOG_IN,
                exact: true,
            },
            Entry {
                href: concat!("/sign-in/", $key, "/provider"),
                label: "Provider sign-in",
                glyph: Glyph::SHIELD_CHECK,
                exact: true,
            },
            Entry {
                href: concat!("/sign-in/", $key, "/both"),
                label: "Both methods",
                glyph: Glyph::USERS,
                exact: true,
            },
        ]
    };
}

/// The navigation groups of `brand`: its pages and its sign-in cards. The
/// brand itself is chosen in the top bar, so the navigation marks one page.
fn navigation(brand: Brand) -> [Group; 2] {
    let (pages, signs): (&'static [Entry], &'static [Entry]) = match brand {
        Brand::Bridge => (entries!("ferrobridge"), sign_ins!("ferrobridge")),
        Brand::Fed => (entries!("ferrofed"), sign_ins!("ferrofed")),
        Brand::Ehr => (entries!("ferroehr"), sign_ins!("ferroehr")),
        Brand::Term => (entries!("ferroterm"), sign_ins!("ferroterm")),
    };
    [
        Group {
            id: "nav-components",
            title: "Components",
            entries: pages,
        },
        Group {
            id: "nav-sign-in",
            title: "Sign-in",
            entries: signs,
        },
    ]
}

/// The brand switcher: the same page in each brand, the one in force marked
/// current.
fn brand_switch(brand: Brand, back: &str) -> impl IntoView + use<> {
    let page = back
        .strip_prefix('/')
        .and_then(|rest| rest.split_once('/').map(|(_, page)| format!("/{page}")))
        .unwrap_or_default();
    let choices = Brand::ALL
        .into_iter()
        .map(|each| {
            let words = match each {
                Brand::Bridge => "BRIDGE",
                Brand::Fed => "FED",
                Brand::Ehr => "EHR",
                Brand::Term => "TERM",
            };
            (format!("/{}{page}", each.key()), words)
        })
        .collect();
    let current = Brand::ALL
        .into_iter()
        .position(|each| each == brand)
        .unwrap_or(0);
    segmented_links("Brand", choices, current)
}

/// The glyph a brand's mark draws in the gallery.
#[must_use]
pub const fn brand_glyph(brand: Brand) -> Glyph {
    match brand {
        Brand::Bridge => Glyph::LAYERS,
        Brand::Fed => Glyph::USERS,
        Brand::Ehr => Glyph::HEART_PULSE,
        Brand::Term => Glyph::LIST_CHECKS,
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
    let at = Memo::new(move |_| {
        let brand = params.with(|params| params.get_str("brand").and_then(Brand::parse));
        (brand, location.pathname.get())
    });
    move || match at.get() {
        (Some(brand), back) => brand_shell(brand, back),
        (None, _) => view! { <NotFound /> }.into_any(),
    }
}

/// Keeps `theme`, which the toggle has set in place, in the gallery's
/// cookie by posting it to the theme route, as the toggle's form would.
fn keep_theme(theme: Theme) {
    #[cfg(target_arch = "wasm32")]
    leptos::task::spawn_local(async move {
        use wasm_bindgen::JsValue;
        let init = web_sys::RequestInit::new();
        init.set_method("POST");
        init.set_body(&JsValue::from_str(&format!(
            "theme={}&back=%2F",
            theme.key()
        )));
        let Ok(headers) = web_sys::Headers::new() else {
            leptos::logging::warn!("the browser refused the headers of the theme post");
            return;
        };
        if headers
            .set("content-type", "application/x-www-form-urlencoded")
            .is_err()
        {
            leptos::logging::warn!("the browser refused the content type of the theme post");
            return;
        }
        init.set_headers(&headers);
        let Some(window) = web_sys::window() else {
            return;
        };
        let sent = window.fetch_with_str_and_init(THEME_ROUTE, &init);
        if wasm_bindgen_futures::JsFuture::from(sent).await.is_err() {
            leptos::logging::warn!("the theme choice was not kept: the post failed");
        }
    });
    #[cfg(not(target_arch = "wasm32"))]
    let _: Theme = theme;
}

/// The synthetic operator the gallery is signed in as.
const OPERATOR: &str = "Practitioner/synthetic-terminologist-on-call";

/// The shell of `brand`'s pages, whose theme switch returns to `back`.
fn brand_shell(brand: Brand, back: String) -> AnyView {
    let start = view! {
        {drawer_button(SIDEBAR_ID)}
        {lockup("/", mark(brand_glyph(brand)), brand.product(), "Gallery")}
    };
    let switch = brand_switch(brand, &back).into_any();
    let end = view! {
        <span class="fui-wide-only">{switch}</span>
        <span class="fui-roomy-only">{pill(Tone::Ok, "Library v0.1.7")}</span>
        {operator_menu(
            OperatorMenu {
                id: "operator-menu",
                // A name as long as a FHIR user reference gets, so the top
                // bar shows how it gives way.
                name: String::from(OPERATOR),
                subtitle: "Signed in to the gallery",
                sign_out: None,
            },
            (),
        )}
        {ferrohealth_ui::widget::theme::theme_toggle(
            String::from(THEME_ROUTE),
            back,
            Callback::new(keep_theme),
        )}
    };
    let search = search_box(SearchBox {
        id: "gallery-search",
        name: "q",
        label: "Search the mapping files",
        placeholder: "A file, a template or a path",
        action: format!("/{}/tables", brand.key()),
        value: String::new(),
    });
    let groups = navigation(brand);
    let page = shell(Shell {
        topbar: topbar_with(start, search, end).into_any(),
        banner: Some(
            banner(
                Tone::Warn,
                Glyph::TRIANGLE_ALERT,
                "Gallery",
                "every row and text on these pages is invented for the gallery; none is patient data.",
            )
            .into_any(),
        ),
        sidebar: sidebar(SIDEBAR_ID, "Pages", &groups),
        content: view! { <Outlet /> }.into_any(),
        footer: FOOTER,
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
                {lockup("/", mark(Glyph::LAYERS), "FerroHEALTH UI", "Gallery")}
            </div>
            <p class="fui-muted">
                "Every component of the library in every product theme. Choose a brand."
            </p>
            <div class="fui-stack">{brands}</div>
        </div>
    }
    .into_any();
    view! {
        <Title text=PRODUCT />
        {centred(content, FOOTER)}
    }
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
        "Not found",
        "404: the gallery has no page at this address.",
        view! {
            <a href="/" class="fui-button fui-button-primary">
                {ferrohealth_ui::icon::icon(Glyph::ARROW_LEFT, "16")}
                "Back to the gallery"
            </a>
        },
    );
    view! {
        <Title text=format!("Not found · {PRODUCT}") />
        {centred(view! { <div class="fui-card">{content}</div> }.into_any(), FOOTER)}
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
