// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The theme switch: light, dark and system, the one in force pressed; and
//! the density and language switches of the same shape.
//!
//! Each button posts its theme as a plain form to the console's own route,
//! before the page has hydrated and after, so it needs no script; the
//! console keeps the choice and sends the browser back to `back`, which it
//! checks is one of its own pages.

use leptos::prelude::*;

use crate::i18n::{Locale, Text, locale, tr};
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
/// shows on the first paint with no script and no cookie. The console's
/// route keeps the choice and answers with the page in the new theme.
#[must_use]
pub fn theme_toggle(action: String, back: String) -> AnyView {
    let to_dark = tr(Text::ThemeToDark);
    let to_light = tr(Text::ThemeToLight);
    view! {
        <form method="post" action=action class="fui-theme-toggle">
            <input type="hidden" name="back" value=back />
            <button
                type="submit"
                id=TO_DARK
                name="theme"
                value=Theme::Dark.key()
                class="fui-icon-button fui-theme-to-dark"
                aria-label=to_dark.clone()
                title=to_dark
            >
                {icon(Glyph::MOON, "16")}
            </button>
            <button
                type="submit"
                id=TO_LIGHT
                name="theme"
                value=Theme::Light.key()
                class="fui-icon-button fui-theme-to-light"
                aria-label=to_light.clone()
                title=to_light
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
            <div role="group" aria-label=tr(Text::ThemeSwitch) class="fui-segmented">
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
            <div role="group" aria-label=tr(Text::DensitySwitch) class="fui-segmented">
                {buttons}
            </div>
        </form>
    }
    .into_any()
}

/// Returns the language switch over `choices`, the page's locale pressed,
/// the shape of [`theme_switch`].
///
/// Each choice is a submit button of one plain form posting `locale`, the
/// choice's BCP 47 tag, to `action` with the page `back` to return to. Each
/// is named in its own language, from its own catalog's `language-name`,
/// and marked with that language in `lang` (WCAG 2.2 SC 3.1.2). The console
/// keeps the choice, provides the locale and writes `<html lang>` as it
/// renders (`crate::i18n`).
#[must_use]
pub fn locale_switch(choices: &[Locale], action: String, back: String) -> AnyView {
    let current = locale();
    let buttons = choices
        .iter()
        .map(|choice| {
            let pressed = *choice == current;
            view! {
                <button
                    type="submit"
                    name="locale"
                    value=choice.tag()
                    lang=choice.tag()
                    class="fui-segment"
                    aria-pressed=pressed.to_string()
                >
                    {choice.text(Text::LanguageName)}
                    {pressed.then(|| icon(Glyph::CHECK, "12"))}
                </button>
            }
        })
        .collect_view();
    view! {
        <form method="post" action=action>
            <input type="hidden" name="back" value=back />
            <div role="group" aria-label=tr(Text::LanguageSwitch) class="fui-segmented">
                {buttons}
            </div>
        </form>
    }
    .into_any()
}
