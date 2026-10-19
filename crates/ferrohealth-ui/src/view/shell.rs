// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The chrome around every page: the skip link, the top bar with the
//! lockup, the body with the navigation beside the page, and the footer
//! that names the product, its version and its maker.
//!
//! The navigation itself, which marks the current page and behaves as a
//! drawer below the first breakpoint, is [`crate::widget::nav`].

use leptos::prelude::*;

use crate::i18n::{Text, tr, tr_with};
use crate::icon::{Glyph, icon};

/// The maker every FerroHEALTH console's footer names.
pub const MAKER: &str = "Cadasto B.V.";

/// The id of the page's main region, which the skip link leads to.
pub const MAIN_ID: &str = "main";

/// What the footer names.
#[derive(Debug, Clone, Copy)]
pub struct Footer {
    /// The product, such as "FerroBRIDGE Console".
    pub product: &'static str,
    /// Its version.
    pub version: &'static str,
    /// The About page every footer links to.
    pub about: &'static str,
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
    /// What the footer names.
    pub footer: Footer,
}

/// Returns the link that skips the chrome to the page, shown once it has
/// the focus (WCAG 2.2 SC 2.4.1).
#[must_use]
pub fn skip_link() -> impl IntoView {
    view! {
        <a href=format!("#{MAIN_ID}") class="fui-skip">
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

/// Returns the lockup that leads to `href`: the decorative `mark` beside the
/// product's `name` and its accented `word`, such as "FerroBRIDGE" and
/// "Console".
#[must_use]
pub fn lockup(
    href: &'static str,
    mark: impl IntoView,
    name: &'static str,
    word: &'static str,
) -> impl IntoView {
    view! {
        <a href=href class="fui-lockup">
            {mark}
            <span class="fui-lockup-name">
                {name} " " <span class="fui-lockup-accent">{word}</span>
            </span>
        </a>
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
            popovertarget=sidebar_id
            aria-controls=sidebar_id
            aria-label=tr(Text::OpenMenu)
        >
            {icon(Glyph::MENU, "18")}
        </button>
    }
}

/// Returns the top bar: `start` (the drawer button and the lockup) and, at
/// the far end, `end` (the status, the theme switch, the operator).
#[must_use]
pub fn topbar(start: impl IntoView + 'static, end: impl IntoView + 'static) -> AnyView {
    let start = start.into_any();
    let end = end.into_any();
    view! {
        <header class="fui-topbar">
            <div class="fui-topbar-row">
                {start} <div class="fui-topbar-end">{end}</div>
            </div>
        </header>
    }
    .into_any()
}

/// Returns the top bar as [`topbar`] draws it, with `middle` (a search box,
/// from [`crate::view::search::search_box`]) in the free width between the
/// two ends, centred.
///
/// Below 48 rem the middle wraps onto its own row under the lockup, so a
/// 320 px screen never scrolls sideways (WCAG 2.2 SC 1.4.10).
#[must_use]
pub fn topbar_with(
    start: impl IntoView + 'static,
    middle: impl IntoView + 'static,
    end: impl IntoView + 'static,
) -> AnyView {
    let start = start.into_any();
    let middle = middle.into_any();
    let end = end.into_any();
    view! {
        <header class="fui-topbar">
            <div class="fui-topbar-row">
                {start} <div class="fui-topbar-middle">{middle}</div>
                <div class="fui-topbar-end">{end}</div>
            </div>
        </header>
    }
    .into_any()
}

/// How the footer's About link is followed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum About {
    /// By the console's router, as every other link.
    #[default]
    Routed,
    /// By a full page load (`rel="external"`), for an About page the server
    /// renders once and the router could not draw.
    FullLoad,
}

/// Returns the one line at the foot of every page: the product, its
/// version, its maker and the About page.
#[must_use]
pub fn footer(footer: Footer) -> impl IntoView {
    footer_about(footer, About::Routed)
}

/// Returns the footer [`footer`] draws, its About link followed as `about`
/// says.
#[must_use]
pub fn footer_about(footer: Footer, about: About) -> impl IntoView {
    let rel = (about == About::FullLoad).then_some("external");
    view! {
        <footer class="fui-footer">
            <p>
                {format!(
                    "{} {} · {MAKER} · ",
                    footer.product,
                    tr_with(Text::FooterVersion, &[("version", footer.version.into())]),
                )} <a href=footer.about rel=rel>
                    {tr(Text::FooterAbout)}
                </a>
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
    shell_about(shell, About::Routed)
}

/// Returns the page [`shell`] draws, its footer's About link followed as
/// `about` says.
#[must_use]
pub fn shell_about(shell: Shell, about: About) -> AnyView {
    let skip = skip_link().into_any();
    let foot = footer_about(shell.footer, about).into_any();
    view! {
        <div class="fui-page">
            {skip} {shell.topbar} {shell.banner} <div class="fui-body">
                {shell.sidebar} <main id=MAIN_ID class="fui-main">
                    <div class="fui-column">{shell.content}</div>
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
    centred_about(content, footer_text, About::Routed)
}

/// Returns the page [`centred`] draws, its footer's About link followed as
/// `about` says.
#[must_use]
pub fn centred_about(content: AnyView, footer_text: Footer, about: About) -> AnyView {
    let foot = footer_about(footer_text, about).into_any();
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
