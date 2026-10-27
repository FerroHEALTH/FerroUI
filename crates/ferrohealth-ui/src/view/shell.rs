// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The chrome around every page.
//!
//! It holds the skip link, the top bar with the drawer button, the lockup,
//! the search box, the status pill and the user menu, the body with the
//! navigation beside the page and the dock at the foot of the main area,
//! and the footer that names the product, its version and its maker.
//!
//! The navigation itself, which marks the current page, folds to a rail
//! and behaves as a drawer below the first breakpoint, is
//! [`crate::widget::nav`]; the user menu is [`crate::widget::menu`].

use std::borrow::Cow;

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};

/// The maker every FerroHEALTH console's footer names.
pub const MAKER: &str = "Cadasto B.V.";

/// The id of the page's main region, which the skip link leads to.
pub const MAIN_ID: &str = "main";

/// What the footer names.
#[derive(Debug, Clone)]
pub struct Footer {
    /// The product, such as "FerroBRIDGE Console", a word of which the
    /// console may look up at run time.
    pub product: Cow<'static, str>,
    /// Its version, which the console may read at run time.
    pub version: Cow<'static, str>,
    /// The About page every footer links to, under the console's base path,
    /// which may be known only at run time.
    pub about: Cow<'static, str>,
}

impl Footer {
    /// The product and its version in the one form the footer and the
    /// About page both write, such as "FerroBRIDGE Console 0.0.12".
    #[must_use]
    pub fn release(&self) -> String {
        // NOTE: no specification governs this: our own design; the product's
        // name and its version are data, drawn as they stand in every locale.
        format!("{} {}", self.product, self.version)
    }
}

/// The parts of a signed-in page.
#[derive(Debug)]
pub struct Shell {
    /// The top bar, from [`topbar`].
    pub topbar: AnyView,
    /// A banner under the top bar, such as the sandbox notice, if any.
    pub banner: Option<AnyView>,
    /// The navigation, from [`crate::widget::nav::sidebar`].
    pub sidebar: AnyView,
    /// The page.
    pub content: AnyView,
    /// The docked bar at the foot of the main area (R4), such as an
    /// editor's reason and its Review changes, if the page has one.
    pub dock: Option<AnyView>,
    /// What the footer names.
    pub footer: Footer,
}

/// Returns the link that skips the chrome to the page, shown once it has
/// the focus (WCAG 2.2 SC 2.4.1).
///
/// The browser follows it on a hydrated page too, so its fragment
/// navigation moves the sequential focus starting point to the main area
/// and the next Tab reaches the page's first control.
#[must_use]
pub fn skip_link() -> impl IntoView {
    // NOTE: `leptos_router` 0.8 `handle_anchor_click` leaves a link with a `target` to the browser,
    // and `_self` is the link's own navigable (HTML Living Standard, "the rules for choosing a navigable").
    view! {
        <a href=format!("#{MAIN_ID}") target="_self" class="fui-skip">
            {tr(Text::SkipToPage)}
        </a>
    }
}

/// Returns a mark for a product that has no drawn one: `glyph` in the
/// accent. A console with a drawn mark passes [`brand_mark`] to [`lockup`].
#[must_use]
pub fn mark(glyph: Glyph) -> impl IntoView {
    view! { <span class="fui-mark" aria-hidden="true">{icon(glyph, "22")}</span> }
}

/// Returns a console's own drawn mark, light and dark.
///
/// The image at `light`, and the one at `dark` under the dark theme, both served from the console's own origin
/// and chosen by the stylesheet before any script runs. It is decoration
/// beside the product's name, so it carries no text.
#[must_use]
pub fn brand_mark(light: &str, dark: &str) -> impl IntoView + use<> {
    view! {
        <span class="fui-brand-mark" aria-hidden="true">
            <img class="fui-brand-light" src=light.to_owned() alt="" width="24" height="24" />
            <img class="fui-brand-dark" src=dark.to_owned() alt="" width="24" height="24" />
        </span>
    }
}

/// Returns the lockup that leads to `href`, a route under the console's
/// base path: the decorative `mark` beside the product's `name` and its
/// accented `word`, such as "FerroBRIDGE" and "Console".
#[must_use]
pub fn lockup(
    href: impl Into<Cow<'static, str>>,
    mark: impl IntoView,
    name: impl Into<String>,
    word: impl Into<String>,
) -> impl IntoView {
    let (href, name, word) = (href.into(), name.into(), word.into());
    view! {
        <a href=href class="fui-lockup">
            {mark}
            <span class="fui-lockup-name">
                {name} " " <span class="fui-lockup-accent">{word}</span>
            </span>
        </a>
    }
}

/// Returns the lockup [`lockup`] draws as plain text that leads nowhere,
/// for a page shown before sign-in.
#[must_use]
pub fn lockup_plain(
    mark: impl IntoView,
    name: impl Into<String>,
    word: impl Into<String>,
) -> impl IntoView {
    let (name, word) = (name.into(), word.into());
    view! {
        <span class="fui-lockup">
            {mark}
            <span class="fui-lockup-name">
                {name} " " <span class="fui-lockup-accent">{word}</span>
            </span>
        </span>
    }
}

/// Returns the button that opens the navigation drawer `sidebar_id`, shown
/// below the first breakpoint alone.
#[must_use]
pub fn drawer_button(sidebar_id: &'static str) -> impl IntoView {
    view! {
        <button
            type="button"
            class="fui-icon-button fui-drawer-only"
            command="show-modal"
            commandfor=sidebar_id
            aria-controls=sidebar_id
            aria-haspopup="dialog"
            aria-label=tr(Text::OpenMenu)
        >
            {icon(Glyph::MENU, "18")}
        </button>
    }
}

/// What the top bar holds, start to end.
#[derive(Debug)]
pub struct TopBar {
    /// The id of the navigation the drawer button opens below 48 rem.
    pub sidebar: &'static str,
    /// The lockup, from [`lockup`].
    pub lockup: AnyView,
    /// The search box in the free width between the two ends, from
    /// [`crate::view::search::search_box`], if the console has one.
    pub search: Option<AnyView>,
    /// The end: the status pill from [`crate::view::status::status_pill`],
    /// any control of the console's own, and the user menu from
    /// [`crate::widget::menu::user_menu`], last.
    pub end: AnyView,
}

/// Returns the top bar: the drawer button, the lockup, the search box and,
/// at the far end, the status pill and the user menu.
///
/// It carries no theme control: the theme is set in the user menu and in
/// Your preferences (family rule 5). Below 48 rem the search wraps onto its
/// own row under the lockup, so a 320 px screen never scrolls sideways
/// (WCAG 2.2 SC 1.4.10).
#[must_use]
pub fn topbar(bar: TopBar) -> AnyView {
    let drawer = drawer_button(bar.sidebar).into_any();
    let search = bar
        .search
        .map(|search| view! { <div class="fui-topbar-middle">{search}</div> });
    view! {
        <header class="fui-topbar">
            <div class="fui-topbar-row">
                {drawer} {bar.lockup} {search} <div class="fui-topbar-end">{bar.end}</div>
            </div>
        </header>
    }
    .into_any()
}

/// Returns the one line at the foot of every page: the product and its
/// version ([`Footer::release`]), its maker and the About page.
#[must_use]
pub fn footer(footer: Footer) -> impl IntoView {
    let line = format!("{} · {MAKER} · ", footer.release());
    let about = footer.about;
    view! {
        <footer class="fui-footer">
            <p>
                {line} <a href=about>{tr(Text::FooterAbout)}</a>
            </p>
        </footer>
    }
}

/// Returns a signed-in page: the skip link, the top bar, the banner, the
/// navigation beside the page in its column, and the footer.
///
/// The parts arrive erased and the footer is erased here, so the page's
/// view type stays shallow (rustc's layout recursion limit).
#[must_use]
pub fn shell(shell: Shell) -> AnyView {
    let skip = skip_link().into_any();
    let foot = footer(shell.footer).into_any();
    view! {
        <div class="fui-page">
            {skip} {shell.topbar} {shell.banner} <div class="fui-body">
                {shell.sidebar} <main id=MAIN_ID class="fui-main">
                    <div class="fui-column">{shell.content}</div>
                    {shell.dock}
                </main>
            </div> {foot}
        </div>
    }
    .into_any()
}

/// Returns a page that centres `content` (the sign-in, a page the console
/// does not serve, a fault) above the footer.
#[must_use]
pub fn centred(content: AnyView, footer_text: Footer) -> AnyView {
    let foot = footer(footer_text).into_any();
    view! {
        <div class="fui-page">
            <main id=MAIN_ID class="fui-centred">
                {content}
            </main>
            {foot}
        </div>
    }
    .into_any()
}
