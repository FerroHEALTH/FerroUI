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
/// and Up move and wrap, Home and End go to the ends, and from no item Down
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

/// Returns a menu item that leads to `href`, with `glyph`, `words` and the
/// key `key` it answers to, if any.
fn link_item(href: &str, glyph: Glyph, words: &Word, key: Option<&str>, class: &str) -> Html {
    // NOTE: no specification governs this: our own design; the item with the `?` key opens
    // the page's shortcut legend where the page has one, and is followed where it has not.
    let opens = if key == Some("?") {
        " data-fui-shortcuts=\"\""
    } else {
        ""
    };
    let cap = key.map_or(String::new(), |key| {
        format!("<kbd class=\"fui-kbd\">{}</kbd>", Html::text(key))
    });
    Html::raw(format!(
        "<a role=\"menuitem\" href=\"{}\" class=\"{class}\"{opens}>{}<span class=\"fui-menu-item-text\"><span class=\"fui-menu-item-label\">{words}</span></span>{cap}</a>",
        Html::text(href),
        icon(glyph, "16")
    ))
}

/// Returns the theme as three `menuitemradio` choices, the one in force
/// checked, each a submit button of one plain form.
fn theme_group(locale: &Locale, id: &str, theme: &ThemeChoice) -> Html {
    let buttons: String = Theme::ALL
        .into_iter()
        .map(|choice| {
            format!(
                "<button type=\"submit\" name=\"theme\" value=\"{}\" role=\"menuitemradio\" class=\"fui-segment\" aria-checked=\"{}\">{}{}</button>",
                choice.key(),
                choice == theme.current,
                icon(glyph(choice), "14"),
                choice.label(locale)
            )
        })
        .collect();
    Html::raw(format!(
        "<form method=\"post\" action=\"{}\" class=\"fui-menu-theme\"><input type=\"hidden\" name=\"back\" value=\"{}\"><div class=\"fui-menu-label\" id=\"{id}-theme\">{}</div><div role=\"group\" aria-labelledby=\"{id}-theme\" class=\"fui-segmented\">{buttons}</div></form>",
        Html::text(&theme.action),
        Html::text(&theme.back),
        tr(locale, Text::ThemeSwitch)
    ))
}

/// Returns the group `choices`, the menu's group number `at`, as
/// `menuitemradio` links, the one in force checked, shown below 40 rem
/// alone.
fn choices_group(id: &str, at: usize, choices: &Choices) -> Html {
    let label_id = if at == 0 {
        format!("{id}-choices")
    } else {
        format!("{id}-choices-{at}")
    };
    let links: String = choices
        .choices
        .iter()
        .map(|choice| {
            format!(
                "<a role=\"menuitemradio\" href=\"{}\" class=\"fui-segment\" aria-checked=\"{}\">{}</a>",
                Html::text(&choice.href),
                choice.chosen,
                Html::text(&choice.words)
            )
        })
        .collect();
    Html::raw(format!(
        "<div class=\"fui-menu-version\"><div class=\"fui-menu-label\" id=\"{label_id}\">{}</div><div role=\"group\" aria-labelledby=\"{label_id}\" class=\"fui-segmented\">{links}</div></div>",
        Html::text(&choices.label)
    ))
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
    items: Html,
    cancel: Word,
}

/// Returns the user menu `menu`: its button and, behind it, the menu the
/// `user_menu` module opens with the keys. Who is signed in heads the
/// popup outside the ARIA menu, which holds the items alone.
#[must_use]
pub fn user_menu(locale: &Locale, menu: UserMenu) -> Html {
    let id = menu.id;
    let mut items = link_item(
        &menu.preferences,
        Glyph::SETTINGS,
        &tr(locale, Text::YourPreferences),
        None,
        "fui-menu-item",
    );
    for link in &menu.links {
        items = items.then(link_item(
            &link.href,
            link.glyph,
            &Word::new(&link.words),
            None,
            "fui-menu-item",
        ));
    }
    for (at, choices) in menu.choices.iter().enumerate() {
        items = items.then(choices_group(id, at, choices));
    }
    items = items.then(theme_group(locale, id, &menu.theme));
    if let Some(href) = &menu.shortcuts {
        items = items.then(link_item(
            href,
            Glyph::KEYBOARD,
            &tr(locale, Text::KeyboardShortcuts),
            Some("?"),
            "fui-menu-item fui-wide-only",
        ));
    }
    if let Some(action) = &menu.sign_out {
        items = items.then(Html::raw(format!(
            "<div class=\"fui-menu-sep\" role=\"separator\"></div><form method=\"post\" action=\"{}\" class=\"fui-menu-form\"><button type=\"submit\" role=\"menuitem\" class=\"fui-menu-item\">{}<span class=\"fui-menu-item-text\"><span class=\"fui-menu-item-label\">{}</span></span></button></form>",
            Html::text(action),
            icon(Glyph::LOG_OUT, "16"),
            tr(locale, Text::SignOut)
        )));
    }
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
        items,
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
        assert_eq!(Some(0), step("ArrowDown", None, 4));
        assert_eq!(Some(3), step("ArrowUp", None, 4));
        assert_eq!(None, step("Enter", Some(1), 4));
        assert_eq!(None, step("ArrowDown", None, 0), "no item to move to");
    }
}
