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

/// The id of the toggle's button that switches to the dark theme.
pub const TO_DARK: &str = "theme-to-dark";

/// The id of the toggle's button that switches to the light theme.
pub const TO_LIGHT: &str = "theme-to-light";

/// Returns the theme toggle of a top bar: one 32 px icon button showing a
/// moon in the light theme and a sun in the dark one, which posts the other
/// theme to `action` with the page `back` to return to.
///
/// The form holds both buttons and the stylesheet shows the one that leads
/// away from the theme on screen, read from `data-theme` on `<html>` and,
/// for the system theme, from `prefers-color-scheme`, so the right glyph
/// shows on the first paint with no script and no cookie. A hydrated console
/// that swaps the theme in place uses [`crate::widget::theme::theme_toggle`]
/// over the same markup.
#[must_use]
pub fn theme_toggle(action: String, back: String) -> AnyView {
    view! {
        <form method="post" action=action class="fui-theme-toggle">
            <input type="hidden" name="back" value=back />
            <button
                type="submit"
                id=TO_DARK
                name="theme"
                value=Theme::Dark.key()
                class="fui-icon-button fui-theme-to-dark"
                aria-label="Switch to the dark theme"
                title="Switch to the dark theme"
            >
                {icon(Glyph::MOON, "16")}
            </button>
            <button
                type="submit"
                id=TO_LIGHT
                name="theme"
                value=Theme::Light.key()
                class="fui-icon-button fui-theme-to-light"
                aria-label="Switch to the light theme"
                title="Switch to the light theme"
            >
                {icon(Glyph::SUN, "16")}
            </button>
        </form>
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
