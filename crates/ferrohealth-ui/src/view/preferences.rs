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
//! console's route ([`crate::island::PreferenceForm`]), so the page works
//! with no WebAssembly; the console keeps the cookie and draws the page
//! again with the row it kept marked Saved, in a fixed slot above the
//! control that never moves it (R28). The page says it is only for you, in
//! this browser, and Reset to defaults asks first, then posts to the
//! console's route, which clears every preference cookie. The library
//! writes no cookie. No specification governs the page: our own design;
//! the preview cards are a radio group (WAI-ARIA 1.2, `radiogroup`, by a
//! `fieldset` of native radios) and the language select is R29.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::calendar::Weekday;
use crate::i18n::{Arg, Locale, Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::theme::{Density, Hints, Language, Motion, Preference, Preferences, Theme, WeekStart};
use crate::view::action::ActionGroup;
use crate::view::button::{Variant, label};
use crate::view::dialog::{Confirmation, confirm_dialog};
use crate::view::surface::{Header, page_heading};
use crate::view::tabs::{Tab, TabRow, joined};
use crate::widget::action_menu::{Choice, Listbox, Pick, listbox};

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
    pub own: Vec<AnyView>,
}

/// One row of a preference: its label and line, and its control.
#[derive(Debug)]
pub struct PreferenceRow {
    /// The preference's name.
    pub label: String,
    /// One line on what it governs.
    pub description: String,
    /// The id of the control the label names, when the control is one
    /// field; a group names itself.
    pub control_id: Option<String>,
    /// The control.
    pub control: AnyView,
    /// Whether the preference was just kept, which marks the row Saved.
    pub saved: bool,
}

/// Returns one preference's row (R27): the label and its line, then the
/// control, with Saved in its fixed slot above the control when the
/// preference was just kept (R28).
#[must_use]
pub fn preference_row(row: PreferenceRow) -> AnyView {
    let PreferenceRow {
        label,
        description,
        control_id,
        control,
        saved,
    } = row;
    let label = match control_id {
        Some(id) => view! { <label for=id class="fui-setting-label">{label}</label> }.into_any(),
        None => view! { <span class="fui-setting-label">{label}</span> }.into_any(),
    };
    let mark = saved.then(|| {
        view! {
            <span class="fui-setting-saved" role="status">
                {icon(Glyph::CHECK, "14")}
                {tr(Text::PreferencesSaved)}
            </span>
        }
    });
    view! {
        <div class="fui-setting-row">
            <div class="fui-setting-text">
                {label} <p class="fui-setting-desc">{description}</p>
            </div>
            <div class="fui-setting-control">{mark} {control}</div>
        </div>
    }
    .into_any()
}

/// A preference chosen from a native select.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectPreference {
    /// The select's id and the name it posts, the preference's cookie name.
    pub name: String,
    /// Each choice: the value it posts and its words.
    pub choices: Vec<(String, String)>,
    /// The value in force.
    pub current: String,
    /// Where it posts.
    pub post: Post,
    /// Whether it was just kept, which gives its select the focus back.
    pub saved: bool,
}

/// Returns a preference's select in its form: it posts on every change
/// once the page has hydrated, and through its Save button before.
#[must_use]
pub fn select_preference(select: SelectPreference) -> AnyView {
    let SelectPreference {
        name,
        choices,
        current,
        post,
        saved,
    } = select;
    let options = choices
        .into_iter()
        .map(|(value, words)| {
            let chosen = value == current;
            view! {
                <option value=value selected=chosen>
                    {words}
                </option>
            }
        })
        .collect_view();
    let control = view! {
        <select id=name.clone() name=name class="fui-select fui-setting-select">
            {options}
        </select>
    };
    form(post, saved, control.into_any())
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
/// turned off, on every change once the page has hydrated, and through its
/// Save button before.
#[must_use]
pub fn switch_preference(switch: SwitchPreference) -> AnyView {
    let SwitchPreference {
        name,
        label,
        on,
        post,
        saved,
    } = switch;
    // NOTE: HTML, "Constructing the entry list": an unchecked checkbox posts nothing,
    // so a hidden `off` goes first and a checked box's `on` after it wins.
    let control = view! {
        <input type="hidden" name=name.clone() value="off" />
        <input
            type="checkbox"
            class="fui-switch"
            id=name.clone()
            name=name
            value="on"
            checked=on
            aria-label=label
        />
    };
    form(post, saved, control.into_any())
}

/// Returns `control` in the form of one preference, posting to `post`.
fn form(post: Post, saved: bool, control: AnyView) -> AnyView {
    let Post { action, back } = post;
    let save = tr(Text::PreferencesSave);
    view! {
        <crate::island::PreferenceForm action back save saved>
            {control}
        </crate::island::PreferenceForm>
    }
    .into_any()
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
fn page_preview(class: &'static str) -> impl IntoView {
    view! {
        <span class=class aria-hidden="true">
            <span class="pv-top"></span>
            <span class="pv-body">
                <span class="pv-side">
                    <i></i>
                    <i></i>
                    <i></i>
                </span>
                <span class="pv-main">
                    <i class="pv-title"></i>
                    <i></i>
                    <i></i>
                    <i class="pv-short"></i>
                    <b></b>
                </span>
            </span>
        </span>
    }
}

/// Returns `preview` drawn.
fn preview(preview: Preview) -> AnyView {
    match preview {
        Preview::Page(class) => page_preview(class).into_any(),
        Preview::Split => view! {
            <span class="fui-preview fui-preview-split" aria-hidden="true">
                {page_preview("fui-preview fui-preview-light")}
                {page_preview("fui-preview fui-preview-dark")}
            </span>
        }
        .into_any(),
        Preview::Table(class, rows) => {
            let rows = (0..rows).map(|_| view! { <i></i> }).collect_view();
            view! {
                <span class=class aria-hidden="true">
                    <span class="pv-head"></span>
                    {rows}
                </span>
            }
            .into_any()
        }
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
    title: String,
    /// The note under the title.
    note: String,
}

/// Returns the radio preview cards (R28) named `legend`, posting `name`,
/// the card whose value is `current` checked.
fn choice_cards(name: &'static str, legend: String, current: &str, cards: Vec<Card>) -> AnyView {
    let cards = cards
        .into_iter()
        .map(|card| {
            let checked = card.value == current;
            view! {
                <label class="fui-choice-card">
                    <input
                        type="radio"
                        class="fui-sr-only"
                        name=name
                        value=card.value
                        checked=checked
                    />
                    {preview(card.preview)}
                    <span class="fui-choice-card-text">
                        <span class="fui-choice-card-title">{card.title}</span>
                        <span class="fui-choice-card-sub">{card.note}</span>
                    </span>
                    <span class="fui-choice-card-check" aria-hidden="true">
                        {icon(Glyph::CHECK, "12")}
                    </span>
                </label>
            }
        })
        .collect_view();
    view! {
        <fieldset class="fui-choice-cards">
            <legend class="fui-sr-only">{legend}</legend>
            {cards}
        </fieldset>
    }
    .into_any()
}

/// Whether the preference kept in the cookie `cookie` is the one `saved`
/// names.
fn is_saved(saved: Option<&str>, cookie: &str) -> bool {
    saved == Some(cookie)
}

/// Returns the rows of Appearance: the theme and the density as preview
/// cards, and reduced motion.
fn appearance(page: &PreferencesPage, post: &Post) -> Vec<AnyView> {
    let saved = page.saved.as_deref();
    let accent = || Arg::from(page.accent.clone());
    let theme_cards = vec![
        Card {
            value: Theme::Light.key(),
            preview: Preview::Page("fui-preview fui-preview-light"),
            title: Theme::Light.label(),
            note: tr_with(Text::PreferencesThemeLightNote, &[("accent", accent())]),
        },
        Card {
            value: Theme::Dark.key(),
            preview: Preview::Page("fui-preview fui-preview-dark"),
            title: Theme::Dark.label(),
            note: tr_with(Text::PreferencesThemeDarkNote, &[("accent", accent())]),
        },
        Card {
            value: Theme::System.key(),
            preview: Preview::Split,
            title: Theme::System.label(),
            note: tr(Text::PreferencesThemeSystemNote),
        },
    ];
    let theme_saved = is_saved(saved, Theme::COOKIE);
    let theme = choice_cards(
        Theme::COOKIE,
        tr(Text::ThemeSwitch),
        page.preferences.theme.key(),
        theme_cards,
    );
    let density_cards = vec![
        Card {
            value: Density::Comfortable.key(),
            preview: Preview::Table("fui-preview fui-preview-table fui-preview-comfortable", 3),
            title: Density::Comfortable.label(),
            note: tr(Text::PreferencesDensityComfortableNote),
        },
        Card {
            value: Density::Compact.key(),
            preview: Preview::Table("fui-preview fui-preview-table fui-preview-compact", 5),
            title: Density::Compact.label(),
            note: tr(Text::PreferencesDensityCompactNote),
        },
    ];
    let density_saved = is_saved(saved, Density::COOKIE);
    let density = choice_cards(
        Density::COOKIE,
        tr(Text::DensitySwitch),
        page.preferences.density.key(),
        density_cards,
    );
    let motion_saved = is_saved(saved, Motion::COOKIE);
    let motion = select_preference(SelectPreference {
        name: String::from(Motion::COOKIE),
        choices: Motion::ALL
            .into_iter()
            .map(|motion| (String::from(motion.key()), motion.label()))
            .collect(),
        current: String::from(page.preferences.motion.key()),
        post: post.clone(),
        saved: motion_saved,
    });
    vec![
        preference_row(PreferenceRow {
            label: tr(Text::ThemeSwitch),
            description: tr(Text::PreferencesThemeDesc),
            control_id: None,
            control: form(post.clone(), theme_saved, theme),
            saved: theme_saved,
        }),
        preference_row(PreferenceRow {
            label: tr(Text::DensitySwitch),
            description: tr(Text::PreferencesDensityDesc),
            control_id: None,
            control: form(post.clone(), density_saved, density),
            saved: density_saved,
        }),
        preference_row(PreferenceRow {
            label: tr(Text::PreferencesMotion),
            description: tr(Text::PreferencesMotionDesc),
            control_id: Some(String::from(Motion::COOKIE)),
            control: motion,
            saved: motion_saved,
        }),
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
    id: &str,
    current: &Language,
    languages: &[Locale],
    browser: &Locale,
    post: &Post,
) -> AnyView {
    let pick = |value: String| Pick::Post {
        action: post.action.clone(),
        fields: vec![
            (String::from(Language::COOKIE), value),
            (String::from("back"), post.back.clone()),
        ],
    };
    let follow = tr(Text::PreferencesLanguageBrowser);
    let mut value = follow.clone();
    let mut choices = vec![Choice {
        label: follow,
        note: Some(tr_with(
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
            value.clone_from(&name);
        }
        choices.push(Choice {
            label: name,
            note: None,
            lang: Some(tag.clone()),
            selected,
            pick: pick(tag),
        });
    }
    listbox(Listbox {
        id: id.to_owned(),
        label: tr(Text::LanguageSwitch),
        value,
        choices,
    })
}

/// Returns the rows of Language and region: the language and the first
/// day of the week.
fn language(page: &PreferencesPage, post: &Post) -> Vec<AnyView> {
    let saved = page.saved.as_deref();
    let week_saved = is_saved(saved, WeekStart::COOKIE);
    let from_language = tr_with(
        Text::WeekStartLanguage,
        &[(
            "weekday",
            Arg::from(tr(weekday_text(page.week_from_language))),
        )],
    );
    let week = select_preference(SelectPreference {
        name: String::from(WeekStart::COOKIE),
        choices: WeekStart::ALL
            .into_iter()
            .map(|start| {
                let words = match start {
                    WeekStart::Language => from_language.clone(),
                    WeekStart::Monday => tr(Text::WeekdayMonday),
                    WeekStart::Sunday => tr(Text::WeekdaySunday),
                };
                (String::from(start.key()), words)
            })
            .collect(),
        current: String::from(page.preferences.week_start.key()),
        post: post.clone(),
        saved: week_saved,
    });
    vec![
        preference_row(PreferenceRow {
            label: tr(Text::LanguageSwitch),
            description: tr(Text::PreferencesLanguageDesc),
            control_id: None,
            control: language_select(
                LANGUAGE_LIST,
                &page.preferences.language,
                &page.languages,
                &page.browser,
                post,
            ),
            saved: is_saved(saved, Language::COOKIE),
        }),
        preference_row(PreferenceRow {
            label: tr(Text::PreferencesWeek),
            description: tr(Text::PreferencesWeekDesc),
            control_id: Some(String::from(WeekStart::COOKIE)),
            control: week,
            saved: week_saved,
        }),
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
fn keyboard(page: &PreferencesPage, post: &Post) -> Vec<AnyView> {
    let hints_saved = is_saved(page.saved.as_deref(), Hints::COOKIE);
    let hints = select_preference(SelectPreference {
        name: String::from(Hints::COOKIE),
        choices: Hints::ALL
            .into_iter()
            .map(|hints| (String::from(hints.key()), hints.label()))
            .collect(),
        current: String::from(page.preferences.hints.key()),
        post: post.clone(),
        saved: hints_saved,
    });
    let mut rows = vec![preference_row(PreferenceRow {
        label: tr(Text::PreferencesHints),
        description: tr(Text::PreferencesHintsDesc),
        control_id: Some(String::from(Hints::COOKIE)),
        control: hints,
        saved: hints_saved,
    })];
    if let Some(href) = page.shortcuts.clone() {
        let link = view! {
            <a href=href class=Variant::Secondary.small() aria-keyshortcuts="?">
                {label(Some(Glyph::KEYBOARD), tr(Text::PreferencesShortcutsOpen))}
                <kbd class="fui-kbd">"?"</kbd>
            </a>
        };
        rows.push(preference_row(PreferenceRow {
            label: tr(Text::PreferencesShortcuts),
            description: tr(Text::PreferencesShortcutsDesc),
            control_id: None,
            control: link.into_any(),
            saved: false,
        }));
    }
    rows
}

/// Returns Your preferences: the title, the three tabs joined to the panel
/// of the tab `page.section`, the line that says where the preferences are
/// kept, and Reset to defaults with its confirmation.
#[must_use]
pub fn preferences_page(page: PreferencesPage) -> AnyView {
    let post = Post {
        action: page.action.clone(),
        back: page.routes.of(page.section).to_owned(),
    };
    let mut rows = match page.section {
        Section::Appearance => appearance(&page, &post),
        Section::Language => language(&page, &post),
        Section::Keyboard => keyboard(&page, &post),
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
            tr(Text::PreferencesAppearance),
        ),
        Tab::new(
            Section::Language.key(),
            routes.language.clone(),
            tr(language_tab),
        ),
        Tab::new(
            Section::Keyboard.key(),
            routes.keyboard.clone(),
            tr(Text::PreferencesKeyboard),
        ),
    ];
    let panel = view! {
        <div class="fui-settings-panel">
            <div class="fui-setting-rows">{rows}</div>
        </div>
    }
    .into_any();
    let sheet = joined(
        TabRow {
            label: tr(Text::PreferencesSections),
            tabs,
            current: String::from(section.key()),
        },
        panel,
        Some("fui-settings-tabbed"),
    );
    let heading = page_heading(
        Header {
            title: tr(Text::YourPreferences),
            subtitle: Some(tr(Text::PreferencesIntro)),
            ..Header::default()
        },
        ActionGroup::none(),
    )
    .into_any();
    let Post { back, .. } = post;
    view! {
        {heading}
        <div class="fui-stack">
            {sheet}
            <div class="fui-settings-foot">
                <p class="fui-small fui-muted">{tr(Text::PreferencesFoot)}</p>
                <button
                    type="button"
                    class=Variant::Ghost.small()
                    popovertarget=RESET_DIALOG
                    aria-haspopup="dialog"
                >
                    {label(Some(Glyph::ROTATE_CCW), tr(Text::PreferencesReset))}
                </button>
            </div>
        </div>
        {reset_dialog(reset, back)}
    }
    .into_any()
}

/// Returns the reset dialog: what a reset does and leaves alone, Cancel,
/// and Reset to defaults posting `back` to the console's route `reset`.
fn reset_dialog(reset: String, back: String) -> impl IntoView {
    let confirm = view! {
        <form method="post" action=reset>
            <input type="hidden" name="back" value=back />
            <button type="submit" class=Variant::Primary.class()>
                {label(Some(Glyph::ROTATE_CCW), tr(Text::PreferencesResetConfirm))}
            </button>
        </form>
    };
    confirm_dialog(
        Confirmation {
            id: RESET_DIALOG,
            title: Cow::Owned(tr(Text::PreferencesResetTitle)),
            message: Cow::Owned(tr(Text::PreferencesResetBody)),
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
