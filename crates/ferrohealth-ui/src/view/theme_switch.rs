// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The theme switch: light, dark and system, the one in force pressed; and
//! the density and language switches of the same shape, and the language
//! choice as a select button.
//!
//! Each button posts its theme as a plain form to the console's own route,
//! before the page has hydrated and after, so it needs no script; the
//! console keeps the choice and sends the browser back to `back`, which it
//! checks is one of its own pages.

use leptos::prelude::*;

use crate::i18n::{Locale, Text, locale, tr};
use crate::icon::{Glyph, icon};
use crate::theme::{Density, Theme};
use crate::widget::action_menu::{Choice, Listbox, Pick, listbox};

/// The glyph a theme is drawn with.
pub(crate) const fn glyph(theme: Theme) -> Glyph {
    match theme {
        Theme::Light => Glyph::SUN,
        Theme::Dark => Glyph::MOON,
        Theme::System => Glyph::MONITOR,
    }
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

/// Returns the language choice over `choices` as a select button and its
/// listbox (R29), the page's locale chosen, its listbox `id`.
///
/// Each choice is named in its own language and marked with it in `lang`
/// (WCAG 2.2 SC 3.1.2), and posts `locale` and `back` to `action` as a
/// plain form, the same post as [`locale_switch`], so it works with no
/// WebAssembly.
#[must_use]
pub fn locale_select(id: &str, choices: &[Locale], action: &str, back: &str) -> AnyView {
    let current = locale();
    let choices = choices
        .iter()
        .map(|choice| Choice {
            label: choice.text(Text::LanguageName),
            note: None,
            lang: Some(choice.tag()),
            selected: *choice == current,
            pick: Pick::Post {
                action: action.to_owned(),
                fields: vec![
                    (String::from("locale"), choice.tag()),
                    (String::from("back"), back.to_owned()),
                ],
            },
        })
        .collect();
    listbox(Listbox {
        id: id.to_owned(),
        label: tr(Text::LanguageSwitch),
        value: current.text(Text::LanguageName),
        choices,
    })
}
