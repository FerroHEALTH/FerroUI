// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Your preferences: one page behind the operator's name, the same three
//! tabs in every console (T3, family rule 9).
//!
//! The tabs are Appearance (theme and density as radio preview cards,
//! reduced motion), Language and region (the language as a select button
//! with each language in its own name, the first day of the week; FerroEHR
//! calls the tab Reading) and Keyboard (the shortcut hints and the legend).
//! Each tab is a route of the console's, and Keyboard, marked
//! `data-tab="keyboard"`, leaves the row below 40 rem (R3). A console never
//! adds a tab: its own preferences go inside these three, after the
//! library's rows ([`PreferencesPage::own`]), built with [`preference_row`]
//! and [`select_preference`].
//!
//! Each preference is its own cookie, set by a plain form post to the
//! console's route, so the page works with no script; once the
//! `preference` module runs, a change posts at once and the Save button
//! leaves. The console keeps the cookie and draws the page
//! again with the row it kept marked Saved, in a fixed slot above the
//! control that never moves it (R28). The page says it is only for you, in
//! this browser, and Reset to defaults asks first, then posts to the
//! console's route, which clears every preference cookie. The library
//! writes no cookie. No specification governs the page: our own design;
//! the preview cards are a radio group (WAI-ARIA 1.2, `radiogroup`, by a
//! `fieldset` of native radios) and the language select is R29.

use askama::Template;

use crate::calendar::Weekday;
use crate::html::{Html, Render as _};
use crate::i18n::{Arg, Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::theme::{Density, Hints, Language, Motion, Preference, Preferences, Theme, WeekStart};
use crate::view::action::ActionGroup;
use crate::view::action_menu::{Choice, Listbox, Pick, listbox};
use crate::view::button::{Variant, label};
use crate::view::dialog::{Confirmation, confirm_dialog};
use crate::view::surface::{Header, page_heading};
use crate::view::tabs::{Tab, TabRow, joined};

/// The id of the reset dialog, which Reset to defaults opens.
pub const RESET_DIALOG: &str = "preferences-reset";

/// The id of the language select's listbox.
pub const LANGUAGE_LIST: &str = "preferences-language";

/// One of the three tabs of Your preferences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    /// The theme, the density and reduced motion.
    #[default]
    Appearance,
    /// The language and the first day of the week.
    Language,
    /// The shortcut hints and the legend.
    Keyboard,
}

impl Section {
    /// The three tabs, in their order.
    pub const ALL: [Self; 3] = [Self::Appearance, Self::Language, Self::Keyboard];

    /// The tab's key, as its `data-tab` carries it and a route may name it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Appearance => "appearance",
            Self::Language => "language",
            Self::Keyboard => "keyboard",
        }
    }

    /// The tab `key` names, or `None` for a key that names none.
    #[must_use]
    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|section| section.key() == key)
    }
}

/// The console's route of each tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionRoutes {
    /// The route of Appearance.
    pub appearance: String,
    /// The route of Language and region.
    pub language: String,
    /// The route of Keyboard.
    pub keyboard: String,
}

impl SectionRoutes {
    /// The route of `section`.
    #[must_use]
    pub fn of(&self, section: Section) -> &str {
        match section {
            Section::Appearance => &self.appearance,
            Section::Language => &self.language,
            Section::Keyboard => &self.keyboard,
        }
    }
}

/// Where a preference's form posts: the console's route and the page to
/// return to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    /// The console's route that keeps the preference in its cookie.
    pub action: String,
    /// The page to return to, which the console checks is one of its own.
    pub back: String,
}

/// What Your preferences shows.
#[derive(Debug)]
pub struct PreferencesPage {
    /// The tab the page is.
    pub section: Section,
    /// The route of each tab.
    pub routes: SectionRoutes,
    /// Whether the Language and region tab is called Reading, as in
    /// FerroEHR.
    pub reading: bool,
    /// The preferences in force, read from their cookies.
    pub preferences: Preferences,
    /// The console's route every preference posts to, its value under its
    /// cookie's name ([`Preference::COOKIE`]) beside `back`.
    pub action: String,
    /// The cookie name of the preference just kept, whose row says Saved.
    pub saved: Option<String>,
    /// The name of the brand's accent, such as "Indigo", which the theme
    /// cards name.
    pub accent: String,
    /// The languages the console offers, each named in its own.
    pub languages: Vec<Locale>,
    /// The language the browser asks for, which Follow the browser names.
    pub browser: Locale,
    /// The first day of the week the page's language uses.
    pub week_from_language: Weekday,
    /// The route of the shortcut legend, if the console has one.
    pub shortcuts: Option<String>,
    /// The console's route Reset to defaults posts to, which clears every
    /// preference cookie and returns to `back`.
    pub reset: String,
    /// The console's own rows of this tab, after the library's, built with
    /// [`preference_row`].
    pub own: Vec<Html>,
}

/// One row of a preference: its label and line, and its control.
#[derive(Debug)]
pub struct PreferenceRow {
    /// The preference's name.
    pub label: Word,
    /// One line on what it governs.
    pub description: Word,
    /// The id of the control the label names, when the control is one
    /// field; a group names itself.
    pub control_id: Option<String>,
    /// The control.
    pub control: Html,
    /// Whether the preference was just kept, which marks the row Saved.
    pub saved: bool,
}

/// Returns one preference's row (R27): the label and its line, then the
/// control, with Saved in its fixed slot above the control when the
/// preference was just kept (R28).
#[must_use]
pub fn preference_row(locale: &Locale, row: PreferenceRow) -> Html {
    let PreferenceRow {
        label,
        description,
        control_id,
        control,
        saved,
    } = row;
    RowView {
        control_id,
        label,
        description,
        saved: saved.then(|| (icon(Glyph::CHECK, "14"), tr(locale, Text::PreferencesSaved))),
        control,
    }
    .html()
}

#[derive(Template)]
#[template(path = "preferences/preferences.html", block = "row")]
struct RowView {
    control_id: Option<String>,
    label: Word,
    description: Word,
    saved: Option<(Html, Word)>,
    control: Html,
}

/// A preference chosen from a native select.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectPreference {
    /// The select's id and the name it posts, the preference's cookie name.
    pub name: String,
    /// Each choice: the value it posts and its words.
    pub choices: Vec<(String, Word)>,
    /// The value in force.
    pub current: String,
    /// Where it posts.
    pub post: Post,
    /// Whether it was just kept, which gives its select the focus back.
    pub saved: bool,
}

/// Returns a preference's select in its form: it posts on every change
/// once the `preference` module runs, and through its Save button before.
#[must_use]
pub fn select_preference(locale: &Locale, select: SelectPreference) -> Html {
    let SelectPreference {
        name,
        choices,
        current,
        post,
        saved,
    } = select;
    let control = SelectView {
        choices: choices
            .into_iter()
            .map(|(value, words)| {
                let chosen = value == current;
                (value, words, chosen)
            })
            .collect(),
        name,
    }
    .html();
    form(locale, post, saved, control)
}

#[derive(Template)]
#[template(path = "preferences/preferences.html", block = "select")]
struct SelectView {
    name: String,
    choices: Vec<(String, Word, bool)>,
}

/// A preference that is on or off, drawn as a switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchPreference {
    /// The switch's id and the name it posts, the preference's cookie name.
    pub name: String,
    /// The switch's accessible name: what it turns on.
    pub label: String,
    /// Whether it is on.
    pub on: bool,
    /// Where it posts.
    pub post: Post,
    /// Whether it was just kept, which gives the switch the focus back.
    pub saved: bool,
}

/// Returns a preference's switch in its form: it posts `on`, or `off` when
/// turned off, on every change once the `preference` module runs, and
/// through its Save button before.
#[must_use]
pub fn switch_preference(locale: &Locale, switch: SwitchPreference) -> Html {
    let SwitchPreference {
        name,
        label,
        on,
        post,
        saved,
    } = switch;
    // NOTE: HTML, "Constructing the entry list": an unchecked checkbox posts nothing,
    // so a hidden `off` goes first and a checked box's `on` after it wins.
    let control = SwitchView { name, label, on }.html();
    form(locale, post, saved, control)
}

#[derive(Template)]
#[template(path = "preferences/preferences.html", block = "switch")]
struct SwitchView {
    name: String,
    label: String,
    on: bool,
}

/// Returns `control` in the form of one preference, posting to `post`.
fn form(locale: &Locale, post: Post, saved: bool, control: Html) -> Html {
    let Post { action, back } = post;
    FormView {
        action,
        saved,
        back,
        control,
        small: Variant::Secondary.small(),
        save: tr(locale, Text::PreferencesSave),
    }
    .html()
}

#[derive(Template)]
#[template(path = "preferences/preferences.html", block = "form")]
struct FormView {
    action: String,
    saved: bool,
    back: String,
    control: Html,
    small: &'static str,
    save: Word,
}

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

/// Whether the preference kept in the cookie `cookie` is the one `saved`
/// names.
fn is_saved(saved: Option<&str>, cookie: &str) -> bool {
    saved == Some(cookie)
}

/// Returns the rows of Appearance: the theme and the density as preview
/// cards, and reduced motion.
fn appearance(locale: &Locale, page: &PreferencesPage, post: &Post) -> Vec<Html> {
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

/// Returns the language choice as a select button and its listbox (R29).
///
/// Follow the browser comes first, naming the language `browser`, then
/// each language in `languages` named in its own (WCAG 2.2 SC 3.1.2), the
/// one `current` names chosen, its listbox `id`; each is a plain form post
/// of `language` to `post`.
#[must_use]
pub fn language_select(
    locale: &Locale,
    id: &str,
    current: &Language,
    languages: &[Locale],
    browser: &Locale,
    post: &Post,
) -> Html {
    let pick = |value: String| Pick::Post {
        action: post.action.clone(),
        fields: vec![
            (String::from(Language::COOKIE), value),
            (String::from("back"), post.back.clone()),
        ],
    };
    let follow = tr(locale, Text::PreferencesLanguageBrowser);
    let mut value = follow.clone();
    let mut choices = vec![Choice {
        label: follow,
        note: Some(tr_with(
            locale,
            Text::PreferencesLanguageBrowserNote,
            &[("language", Arg::from(browser.text(Text::LanguageName)))],
        )),
        lang: None,
        selected: *current == Language::Browser,
        pick: pick(String::from(Language::BROWSER)),
    }];
    for language in languages {
        let tag = language.tag();
        let selected = current.tag() == Some(tag.as_str());
        let name = language.text(Text::LanguageName);
        if selected {
            value = Word::new(name.clone());
        }
        choices.push(Choice {
            label: Word::new(name),
            note: None,
            lang: Some(tag.clone()),
            selected,
            pick: pick(tag),
        });
    }
    listbox(
        locale,
        &Listbox {
            id: id.to_owned(),
            label: tr(locale, Text::LanguageSwitch).into(),
            value,
            choices,
        },
    )
}

/// Returns the rows of Language and region: the language and the first
/// day of the week.
fn language(locale: &Locale, page: &PreferencesPage, post: &Post) -> Vec<Html> {
    let saved = page.saved.as_deref();
    let week_saved = is_saved(saved, WeekStart::COOKIE);
    let from_language = tr_with(
        locale,
        Text::WeekStartLanguage,
        &[(
            "weekday",
            Arg::from(tr(locale, weekday_text(page.week_from_language))),
        )],
    );
    let week = select_preference(
        locale,
        SelectPreference {
            name: String::from(WeekStart::COOKIE),
            choices: WeekStart::ALL
                .into_iter()
                .map(|start| {
                    let words = match start {
                        WeekStart::Language => from_language.clone(),
                        WeekStart::Monday => tr(locale, Text::WeekdayMonday),
                        WeekStart::Sunday => tr(locale, Text::WeekdaySunday),
                    };
                    (String::from(start.key()), words)
                })
                .collect(),
            current: String::from(page.preferences.week_start.key()),
            post: post.clone(),
            saved: week_saved,
        },
    );
    vec![
        preference_row(
            locale,
            PreferenceRow {
                label: tr(locale, Text::LanguageSwitch),
                description: tr(locale, Text::PreferencesLanguageDesc),
                control_id: None,
                control: language_select(
                    locale,
                    LANGUAGE_LIST,
                    &page.preferences.language,
                    &page.languages,
                    &page.browser,
                    post,
                ),
                saved: is_saved(saved, Language::COOKIE),
            },
        ),
        preference_row(
            locale,
            PreferenceRow {
                label: tr(locale, Text::PreferencesWeek),
                description: tr(locale, Text::PreferencesWeekDesc),
                control_id: Some(String::from(WeekStart::COOKIE)),
                control: week,
                saved: week_saved,
            },
        ),
    ]
}

/// The message of `day`'s name.
const fn weekday_text(day: Weekday) -> Text {
    match day {
        Weekday::Monday => Text::WeekdayMonday,
        Weekday::Tuesday => Text::WeekdayTuesday,
        Weekday::Wednesday => Text::WeekdayWednesday,
        Weekday::Thursday => Text::WeekdayThursday,
        Weekday::Friday => Text::WeekdayFriday,
        Weekday::Saturday => Text::WeekdaySaturday,
        Weekday::Sunday => Text::WeekdaySunday,
    }
}

/// Returns the rows of Keyboard: the shortcut hints, and the link to the
/// legend where the console has one.
fn keyboard(locale: &Locale, page: &PreferencesPage, post: &Post) -> Vec<Html> {
    let hints_saved = is_saved(page.saved.as_deref(), Hints::COOKIE);
    let hints = select_preference(
        locale,
        SelectPreference {
            name: String::from(Hints::COOKIE),
            choices: Hints::ALL
                .into_iter()
                .map(|hints| (String::from(hints.key()), hints.label(locale)))
                .collect(),
            current: String::from(page.preferences.hints.key()),
            post: post.clone(),
            saved: hints_saved,
        },
    );
    let mut rows = vec![preference_row(
        locale,
        PreferenceRow {
            label: tr(locale, Text::PreferencesHints),
            description: tr(locale, Text::PreferencesHintsDesc),
            control_id: Some(String::from(Hints::COOKIE)),
            control: hints,
            saved: hints_saved,
        },
    )];
    if let Some(href) = page.shortcuts.clone() {
        let link = Html::raw(format!(
            "<a href=\"{}\" aria-keyshortcuts=\"?\" class=\"{}\">{}<kbd class=\"fui-kbd\">?</kbd></a>",
            Html::text(href),
            Variant::Secondary.small(),
            label(
                Some(Glyph::KEYBOARD),
                tr(locale, Text::PreferencesShortcutsOpen)
            )
        ));
        rows.push(preference_row(
            locale,
            PreferenceRow {
                label: tr(locale, Text::PreferencesShortcuts),
                description: tr(locale, Text::PreferencesShortcutsDesc),
                control_id: None,
                control: link,
                saved: false,
            },
        ));
    }
    rows
}

/// Returns Your preferences: the title, the three tabs joined to the panel
/// of the tab `page.section`, the line that says where the preferences are
/// kept, and Reset to defaults with its confirmation.
#[must_use]
pub fn preferences_page(locale: &Locale, page: PreferencesPage) -> Html {
    let post = Post {
        action: page.action.clone(),
        back: page.routes.of(page.section).to_owned(),
    };
    let mut rows = match page.section {
        Section::Appearance => appearance(locale, &page, &post),
        Section::Language => language(locale, &page, &post),
        Section::Keyboard => keyboard(locale, &page, &post),
    };
    let PreferencesPage {
        section,
        routes,
        reading,
        reset,
        own,
        ..
    } = page;
    rows.extend(own);
    let language_tab = if reading {
        Text::PreferencesReading
    } else {
        Text::PreferencesLanguageRegion
    };
    let tabs = vec![
        Tab::new(
            Section::Appearance.key(),
            routes.appearance.clone(),
            tr(locale, Text::PreferencesAppearance),
        ),
        Tab::new(
            Section::Language.key(),
            routes.language.clone(),
            tr(locale, language_tab),
        ),
        Tab::new(
            Section::Keyboard.key(),
            routes.keyboard.clone(),
            tr(locale, Text::PreferencesKeyboard),
        ),
    ];
    let panel = Html::raw(format!(
        "<div class=\"fui-settings-panel\"><div class=\"fui-setting-rows\">{}</div></div>",
        Html::join(rows)
    ));
    let sheet = joined(
        locale,
        TabRow {
            label: tr(locale, Text::PreferencesSections).into(),
            tabs,
            current: String::from(section.key()),
        },
        panel,
        Some("fui-settings-tabbed"),
    );
    let heading = page_heading(
        locale,
        Header {
            title: tr(locale, Text::YourPreferences),
            subtitle: Some(tr(locale, Text::PreferencesIntro)),
            ..Header::default()
        },
        ActionGroup::none(),
    );
    let Post { back, .. } = post;
    PageView {
        heading,
        sheet,
        foot: tr(locale, Text::PreferencesFoot),
        reset_dialog_id: RESET_DIALOG,
        ghost: Variant::Ghost.small(),
        reset_label: label(Some(Glyph::ROTATE_CCW), tr(locale, Text::PreferencesReset)),
        reset_dialog: reset_dialog(locale, &reset, &back),
    }
    .html()
}

#[derive(Template)]
#[template(path = "preferences/preferences.html", block = "page")]
struct PageView {
    heading: Html,
    sheet: Html,
    foot: Word,
    reset_dialog_id: &'static str,
    ghost: &'static str,
    reset_label: Html,
    reset_dialog: Html,
}

/// Returns the reset dialog: what a reset does and leaves alone, Cancel,
/// and Reset to defaults posting `back` to the console's route `reset`.
fn reset_dialog(locale: &Locale, reset: &str, back: &str) -> Html {
    let confirm = Html::raw(format!(
        "<form method=\"post\" action=\"{}\"><input type=\"hidden\" name=\"back\" value=\"{}\"><button type=\"submit\" class=\"{}\">{}</button></form>",
        Html::text(reset),
        Html::text(back),
        Variant::Primary.class(),
        label(
            Some(Glyph::ROTATE_CCW),
            tr(locale, Text::PreferencesResetConfirm)
        )
    ));
    confirm_dialog(
        locale,
        Confirmation {
            id: RESET_DIALOG.into(),
            title: tr(locale, Text::PreferencesResetTitle),
            message: tr(locale, Text::PreferencesResetBody),
        },
        confirm,
    )
}

#[cfg(test)]
mod tests {
    use super::Section;

    #[test]
    fn every_section_reads_back_from_its_key() {
        for section in Section::ALL {
            assert_eq!(Some(section), Section::parse(section.key()), "{section:?}");
        }
        assert_eq!(None, Section::parse("account"), "no fourth tab");
        assert_eq!(
            "keyboard",
            Section::Keyboard.key(),
            "the family hook phones hide (R3)"
        );
    }
}
