// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The chrome around every page: the skip link, the top bar with the
//! lockup, the body with the navigation beside the page, and the footer
//! that names the product, its version and its maker.
//!
//! The navigation itself, which marks the current page and behaves as a
//! drawer below the first breakpoint, is [`crate::widget::nav`].

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

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
            "Skip to the page"
        </a>
    }
}

/// Returns a mark for a product that has no drawn one: `glyph` on the
/// accent. A console with a brand mark passes its own to [`lockup`].
#[must_use]
pub fn mark(glyph: Glyph) -> impl IntoView {
    view! { <span class="fui-mark" aria-hidden="true">{icon(glyph, "16")}</span> }
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
    let class = format!("{} fui-drawer-only", Variant::Secondary.class());
    view! {
        <button type="button" class=class popovertarget=sidebar_id aria-controls=sidebar_id>
            {icon(Glyph::MENU, "16")}
            "Menu"
        </button>
    }
}

/// Returns the top bar: `start` (the drawer button and the lockup) and, at
/// the far end, `end` (the status, the theme switch, the operator).
#[must_use]
pub fn topbar(start: impl IntoView, end: impl IntoView) -> impl IntoView {
    view! {
        <header class="fui-topbar">
            <div class="fui-topbar-row">
                {start} <div class="fui-topbar-end">{end}</div>
            </div>
        </header>
    }
}

/// Returns the one line at the foot of every page: the product, its
/// version, its maker and the About page.
#[must_use]
pub fn footer(footer: Footer) -> impl IntoView {
    view! {
        <footer class="fui-footer">
            <p>
                {footer.product} " v" {footer.version} " · " {MAKER} " · "
                <a href=footer.about>"About"</a>
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
