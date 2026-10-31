// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The Appearance tab of Your preferences: the theme and the density as
//! radio preview cards (R28), and reduced motion.

use askama::Template;

use super::{
    Post, PreferenceRow, PreferencesPage, SelectPreference, form, is_saved, preference_row,
    select_preference,
};
use crate::html::{Html, Render as _};
use crate::i18n::{Arg, Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::theme::{Density, Motion, Preference as _, Theme};

/// The preview a choice card draws, hidden from assistive technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Preview {
    /// A page in the theme named by the class.
    Page(&'static str),
    /// A page half light, half dark: the system theme.
    Split,
    /// A table in the density named by the class.
    Table(&'static str, usize),
}

/// Returns the small page a theme card draws: a top bar, a sidebar and a
/// main column in the theme's preview accent.
fn page_preview(class: &'static str) -> Html {
    PagePreview { class }.html()
}

#[derive(Template)]
#[template(path = "preferences/preferences.html", block = "page_preview")]
struct PagePreview {
    class: &'static str,
}

/// Returns `preview` drawn.
fn preview(preview: Preview) -> Html {
    match preview {
        Preview::Page(class) => page_preview(class),
        Preview::Split => Html::raw(format!(
            "<span aria-hidden=\"true\" class=\"fui-preview fui-preview-split\">{}{}</span>",
            page_preview("fui-preview fui-preview-light"),
            page_preview("fui-preview fui-preview-dark")
        )),
        Preview::Table(class, rows) => Html::raw(format!(
            "<span aria-hidden=\"true\" class=\"{class}\"><span class=\"pv-head\"></span>{}</span>",
            "<i></i>".repeat(rows)
        )),
    }
}

/// One radio preview card: the value it posts, its preview, its title and
/// its note.
struct Card {
    /// The value it posts.
    value: &'static str,
    /// Its preview.
    preview: Preview,
    /// Its title.
    title: Word,
    /// The note under the title.
    note: Word,
}

/// Returns the radio preview cards (R28) named `legend`, posting `name`,
/// the card whose value is `current` checked.
fn choice_cards(name: &'static str, legend: Word, current: &str, cards: Vec<Card>) -> Html {
    CardsView {
        legend,
        name,
        cards: cards
            .into_iter()
            .map(|card| CardView {
                checked: card.value == current,
                value: card.value,
                preview: preview(card.preview),
                title: card.title,
                note: card.note,
            })
            .collect(),
        check: icon(Glyph::CHECK, "12"),
    }
    .html()
}

/// One card as the template writes it.
struct CardView {
    value: &'static str,
    checked: bool,
    preview: Html,
    title: Word,
    note: Word,
}

#[derive(Template)]
#[template(path = "preferences/preferences.html", block = "cards")]
struct CardsView {
    legend: Word,
    name: &'static str,
    cards: Vec<CardView>,
    check: Html,
}

/// Returns the rows of Appearance: the theme and the density as preview
/// cards, and reduced motion.
#[expect(
    clippy::too_many_lines,
    reason = "the three rows and their five preview cards read as one list, as on main"
)]
pub(super) fn appearance(locale: &Locale, page: &PreferencesPage, post: &Post) -> Vec<Html> {
    let saved = page.saved.as_deref();
    let accent = || Arg::from(page.accent.clone());
    let theme_cards = vec![
        Card {
            value: Theme::Light.key(),
            preview: Preview::Page("fui-preview fui-preview-light"),
            title: Theme::Light.label(locale),
            note: tr_with(
                locale,
                Text::PreferencesThemeLightNote,
                &[("accent", accent())],
            ),
        },
        Card {
            value: Theme::Dark.key(),
            preview: Preview::Page("fui-preview fui-preview-dark"),
            title: Theme::Dark.label(locale),
            note: tr_with(
                locale,
                Text::PreferencesThemeDarkNote,
                &[("accent", accent())],
            ),
        },
        Card {
            value: Theme::System.key(),
            preview: Preview::Split,
            title: Theme::System.label(locale),
            note: tr(locale, Text::PreferencesThemeSystemNote),
        },
    ];
    let theme_saved = is_saved(saved, Theme::COOKIE);
    let theme = choice_cards(
        Theme::COOKIE,
        tr(locale, Text::ThemeSwitch),
        page.preferences.theme.key(),
        theme_cards,
    );
    let density_cards = vec![
        Card {
            value: Density::Comfortable.key(),
            preview: Preview::Table("fui-preview fui-preview-table fui-preview-comfortable", 3),
            title: Density::Comfortable.label(locale),
            note: tr(locale, Text::PreferencesDensityComfortableNote),
        },
        Card {
            value: Density::Compact.key(),
            preview: Preview::Table("fui-preview fui-preview-table fui-preview-compact", 5),
            title: Density::Compact.label(locale),
            note: tr(locale, Text::PreferencesDensityCompactNote),
        },
    ];
    let density_saved = is_saved(saved, Density::COOKIE);
    let density = choice_cards(
        Density::COOKIE,
        tr(locale, Text::DensitySwitch),
        page.preferences.density.key(),
        density_cards,
    );
    let motion_saved = is_saved(saved, Motion::COOKIE);
    let motion = select_preference(
        locale,
        SelectPreference {
            name: String::from(Motion::COOKIE),
            choices: Motion::ALL
                .into_iter()
                .map(|motion| (String::from(motion.key()), motion.label(locale)))
                .collect(),
            current: String::from(page.preferences.motion.key()),
            post: post.clone(),
            saved: motion_saved,
        },
    );
    vec![
        preference_row(
            locale,
            PreferenceRow {
                label: tr(locale, Text::ThemeSwitch),
                description: tr(locale, Text::PreferencesThemeDesc),
                control_id: None,
                control: form(locale, post.clone(), theme_saved, theme),
                saved: theme_saved,
            },
        ),
        preference_row(
            locale,
            PreferenceRow {
                label: tr(locale, Text::DensitySwitch),
                description: tr(locale, Text::PreferencesDensityDesc),
                control_id: None,
                control: form(locale, post.clone(), density_saved, density),
                saved: density_saved,
            },
        ),
        preference_row(
            locale,
            PreferenceRow {
                label: tr(locale, Text::PreferencesMotion),
                description: tr(locale, Text::PreferencesMotionDesc),
                control_id: Some(String::from(Motion::COOKIE)),
                control: motion,
                saved: motion_saved,
            },
        ),
    ]
}
