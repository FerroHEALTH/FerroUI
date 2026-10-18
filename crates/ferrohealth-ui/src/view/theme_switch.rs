// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The theme switch: light, dark and system, the one in force pressed.
//!
//! Each button posts its theme as a plain form to the console's own route,
//! before the page has hydrated and after, so it needs no script; the
//! console keeps the choice and sends the browser back to `back`, which it
//! checks is one of its own pages. A console that switches in place adds an
//! action of its own over the same form.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::theme::{Density, Theme};

/// The glyph a theme is drawn with.
const fn glyph(theme: Theme) -> Glyph {
    match theme {
        Theme::Light => Glyph::SUN,
        Theme::Dark => Glyph::MOON,
        Theme::System => Glyph::MONITOR,
    }
}

/// Returns the compact theme control of a top bar.
///
/// It is a square button showing the theme in force, which opens a menu of the three themes as an HTML
/// popover (opened, closed on Escape and the focus returned with no
/// script); each is a button of one form posting to `action` with the page
/// `back` to return to, the one in force pressed and checked.
#[must_use]
pub fn theme_menu(current: Theme, action: String, back: String) -> AnyView {
    const ID: &str = "theme-menu";
    let items = Theme::ALL
        .into_iter()
        .map(|choice| {
            let pressed = current == choice;
            view! {
                <button
                    type="submit"
                    name="theme"
                    value=choice.key()
                    class="fui-menu-item"
                    aria-pressed=pressed.to_string()
                >
                    {icon(glyph(choice), "16")}
                    <span>{choice.label()}</span>
                    {pressed.then(|| icon(Glyph::CHECK, "14"))}
                </button>
            }
        })
        .collect_view();
    let label = format!("Theme: {}", current.label());
    view! {
        <button
            type="button"
            class="fui-icon-button"
            popovertarget=ID
            aria-controls=ID
            aria-label=label
        >
            {icon(glyph(current), "16")}
        </button>
        <div id=ID popover="auto" class="fui-menu fui-menu-compact">
            <form method="post" action=action>
                <input type="hidden" name="back" value=back />
                <div role="group" aria-label="Theme" class="fui-menu-items">
                    {items}
                </div>
            </form>
        </div>
    }
    .into_any()
}

/// Returns the theme switch for the theme `current`, posting to `action`
/// with the page `back` to return to.
#[must_use]
pub fn theme_switch(current: Theme, action: String, back: String) -> impl IntoView + use<> {
    let buttons = Theme::ALL
        .into_iter()
        .map(|choice| {
            let pressed = current == choice;
            view! {
                <button
                    type="submit"
                    name="theme"
                    value=choice.key()
                    class="fui-segment"
                    aria-pressed=pressed.to_string()
                >
                    {icon(glyph(choice), "14")}
                    {choice.label()}
                    {pressed.then(|| icon(Glyph::CHECK, "12"))}
                </button>
            }
        })
        .collect_view();
    view! {
        <form method="post" action=action>
            <input type="hidden" name="back" value=back />
            <div role="group" aria-label="Theme" class="fui-segmented">
                {buttons}
            </div>
        </form>
    }
}

/// Returns the density switch for the density `current`, the shape of
/// [`theme_switch`].
///
/// Comfortable and compact are submit buttons of one plain form posting
/// `density` to `action` with the page `back` to return to, the one in
/// force pressed and checked.
///
/// It needs no script; the console keeps the choice and writes
/// `data-density` on `<html>` as it renders, so the first paint is right.
#[must_use]
pub fn density_switch(current: Density, action: String, back: String) -> AnyView {
    let buttons = Density::ALL
        .into_iter()
        .map(|choice| {
            let pressed = current == choice;
            view! {
                <button
                    type="submit"
                    name="density"
                    value=choice.key()
                    class="fui-segment"
                    aria-pressed=pressed.to_string()
                >
                    {choice.label()}
                    {pressed.then(|| icon(Glyph::CHECK, "12"))}
                </button>
            }
        })
        .collect_view();
    view! {
        <form method="post" action=action>
            <input type="hidden" name="back" value=back />
            <div role="group" aria-label="Density" class="fui-segmented">
                {buttons}
            </div>
        </form>
    }
    .into_any()
}
