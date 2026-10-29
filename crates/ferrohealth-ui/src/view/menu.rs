// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The user menu (F13), the one place in the top bar that sets the theme.
//!
//! A button with the operator's name opens who is signed in over an ARIA
//! menu of Your preferences, the console's own links, the theme as three
//! choices, Keyboard shortcuts with its key, and Sign out, last after a
//! separator (family rule 7). Below 40 rem the menu opens as a bottom sheet
//! with Cancel.
//!
//! The menu is an HTML popover its button opens, so the browser opens it,
//! closes it on Escape or a click outside and returns the focus to the
//! button, with no script. Every item works without it: the links lead on,
//! and the theme and Sign out are plain forms that post to the console's
//! routes. The `user_menu` module adds the menu button pattern of the
//! WAI-ARIA Authoring Practices 1.2: the button says whether the menu is
//! open, the focus moves to the first item as it opens, Up and Down move
//! between the items and wrap, Home and End go to the first and the last,
//! and Tab closes the menu.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::theme::Theme;

/// The glyph a theme is drawn with.
const fn glyph(theme: Theme) -> Glyph {
    match theme {
        Theme::Light => Glyph::SUN,
        Theme::Dark => Glyph::MOON,
        Theme::System => Glyph::MONITOR,
    }
}

/// The theme choice the menu posts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeChoice {
    /// The theme in force, checked in the menu.
    pub current: Theme,
    /// The console's route the choice posts `theme` to, which keeps it in
    /// the theme cookie (family rule 9).
    pub action: String,
    /// The page to return to, posted as `back`, which the console checks is
    /// one of its own.
    pub back: String,
}

/// One choice of a [`Choices`] group: a link that chooses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// What it says, such as "R4".
    pub words: String,
    /// The link that chooses it.
    pub href: String,
    /// Whether it is the choice in force.
    pub chosen: bool,
}

/// A group of choices the top bar shows from 40 rem up and the menu holds
/// below it, such as the FHIR version a console reads through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choices {
    /// The group's name, such as "FHIR version".
    pub label: String,
    /// Its choices, in order.
    pub choices: Vec<Choice>,
}

/// A link of the console's own in the user menu, such as Your access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserLink {
    /// Its words, such as "Your access".
    pub words: String,
    /// Where it leads.
    pub href: String,
    /// The glyph beside its words.
    pub glyph: Glyph,
}

/// What the user menu shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserMenu {
    /// The menu's id, which its button opens and its items' ids start with.
    pub id: &'static str,
    /// The menu's accessible name and the title of its sheet: the
    /// operator's name, or words such as "Your menu" when nobody is signed
    /// in.
    pub label: String,
    /// The name the button shows, or `None` for a button with its icon
    /// alone, named by `label`.
    pub name: Option<String>,
    /// Who is signed in, at the head of the menu.
    pub who: String,
    /// The line under it: the role and the identity provider.
    pub line: String,
    /// The link to Your preferences.
    pub preferences: String,
    /// The console's own links, in order, between Your preferences and the
    /// theme, such as Your access.
    pub links: Vec<UserLink>,
    /// The theme choice.
    pub theme: ThemeChoice,
    /// The groups the menu holds below 40 rem alone, one per control the
    /// top bar shows from 40 rem up, in order.
    pub choices: Vec<Choices>,
    /// The link to the keyboard shortcuts, shown from 40 rem up (family
    /// rule 6), if the console has them.
    pub shortcuts: Option<String>,
    /// The console's sign-out route, a plain form post it answers with a
    /// redirect, or `None` for a console with no sign-in.
    pub sign_out: Option<String>,
}

/// The item a key moves the focus to from `current` of `count` items: Down
/// and Up move and wrap.
///
/// Home and End go to the ends, and from no item Down
/// goes to the first and Up to the last; any other key is `None`.
#[must_use]
pub fn step(key: &str, current: Option<u32>, count: u32) -> Option<u32> {
    let last = count.checked_sub(1)?;
    match (key, current) {
        ("ArrowDown", Some(at)) if at < last => Some(at + 1),
        ("ArrowDown" | "Home", _) => Some(0),
        ("ArrowUp", Some(at)) if at > 0 && at <= last => Some(at - 1),
        ("ArrowUp" | "End", _) => Some(last),
        _ => None,
    }
}

/// A link item of the menu.
struct LinkItem {
    href: String,
    glyph: Html,
    words: Word,
    key: Option<&'static str>,
    wide_only: bool,
}

/// One choice of a group, as the template writes it.
struct ChoiceView {
    href: String,
    words: String,
    chosen: bool,
}

/// One group of choices, as the template writes it.
struct GroupView {
    label_id: String,
    label: String,
    choices: Vec<ChoiceView>,
}

/// One theme of the theme group, as the template writes it.
struct ThemeView {
    key: &'static str,
    checked: bool,
    glyph: Html,
    label: Word,
}

#[derive(Template)]
#[template(path = "menu/user_menu.html")]
struct UserMenuView {
    id: &'static str,
    list_id: String,
    label: String,
    named: Option<String>,
    name: Option<String>,
    user_glyph: Html,
    chevron: Html,
    who: String,
    line: String,
    links: Vec<LinkItem>,
    groups: Vec<GroupView>,
    theme_action: String,
    theme_back: String,
    theme_label: Word,
    themes: Vec<ThemeView>,
    shortcuts: Option<LinkItem>,
    sign_out: Option<String>,
    sign_out_glyph: Html,
    sign_out_words: Word,
    cancel: Word,
}

/// Returns the user menu `menu`: its button and, behind it, the menu the
/// `user_menu` module opens with the keys. Who is signed in heads the
/// popup outside the ARIA menu, which holds the items alone.
#[must_use]
pub fn user_menu(locale: &Locale, menu: UserMenu) -> Html {
    let id = menu.id;
    let mut links = vec![LinkItem {
        href: menu.preferences,
        glyph: icon(Glyph::SETTINGS, "16"),
        words: tr(locale, Text::YourPreferences),
        key: None,
        wide_only: false,
    }];
    links.extend(menu.links.into_iter().map(|link| LinkItem {
        href: link.href,
        glyph: icon(link.glyph, "16"),
        words: Word::new(&link.words),
        key: None,
        wide_only: false,
    }));
    let groups = menu
        .choices
        .into_iter()
        .enumerate()
        .map(|(at, group)| GroupView {
            label_id: if at == 0 {
                format!("{id}-choices")
            } else {
                format!("{id}-choices-{at}")
            },
            label: group.label,
            choices: group
                .choices
                .into_iter()
                .map(|choice| ChoiceView {
                    href: choice.href,
                    words: choice.words,
                    chosen: choice.chosen,
                })
                .collect(),
        })
        .collect();
    let themes = Theme::ALL
        .into_iter()
        .map(|choice| ThemeView {
            key: choice.key(),
            checked: choice == menu.theme.current,
            glyph: icon(glyph(choice), "14"),
            label: choice.label(locale),
        })
        .collect();
    // NOTE: no specification governs this: our own design; the item with the `?` key opens
    // the page's shortcut legend where the page has one, and is followed where it has not.
    let shortcuts = menu.shortcuts.map(|href| LinkItem {
        href,
        glyph: icon(Glyph::KEYBOARD, "16"),
        words: tr(locale, Text::KeyboardShortcuts),
        key: Some("?"),
        wide_only: true,
    });
    UserMenuView {
        id,
        list_id: format!("{id}-menu"),
        named: menu.name.is_none().then(|| menu.label.clone()),
        label: menu.label,
        name: menu.name,
        user_glyph: icon(Glyph::CIRCLE_USER, "16"),
        chevron: icon(Glyph::CHEVRON_DOWN, "14"),
        who: menu.who,
        line: menu.line,
        links,
        groups,
        theme_action: menu.theme.action,
        theme_back: menu.theme.back,
        theme_label: tr(locale, Text::ThemeSwitch),
        themes,
        shortcuts,
        sign_out: menu.sign_out,
        sign_out_glyph: icon(Glyph::LOG_OUT, "16"),
        sign_out_words: tr(locale, Text::SignOut),
        cancel: tr(locale, Text::Cancel),
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::step;

    #[test]
    fn the_arrows_wrap_and_home_and_end_go_to_the_ends() {
        // WAI-ARIA Authoring Practices 1.2, the menu pattern, keyboard interaction.
        assert_eq!(Some(1), step("ArrowDown", Some(0), 4));
        assert_eq!(Some(0), step("ArrowDown", Some(3), 4), "wraps forward");
        assert_eq!(Some(3), step("ArrowUp", Some(0), 4), "wraps back");
        assert_eq!(Some(1), step("ArrowUp", Some(2), 4));
        assert_eq!(Some(0), step("Home", Some(2), 4));
        assert_eq!(Some(3), step("End", Some(0), 4));
    }

    #[test]
    fn from_no_item_down_goes_to_the_first_and_up_to_the_last() {
        assert_eq!(Some(0), step("ArrowDown", None, 4));
        assert_eq!(Some(3), step("ArrowUp", None, 4));
    }

    #[test]
    fn a_key_the_menu_leaves_alone_moves_nothing() {
        assert_eq!(None, step("Enter", Some(1), 4));
        assert_eq!(None, step("ArrowDown", None, 0), "no item to move to");
    }
}
