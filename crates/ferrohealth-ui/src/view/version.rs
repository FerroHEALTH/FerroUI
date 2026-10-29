// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The version select (M3): the FHIR version a console reads through, as
//! one compact control in the top bar.
//!
//! The control shows the version alone ("R4" and a chevron), with its label
//! and release as its tooltip and its accessible name, and no prefix text.
//! Its menu (F13) lists each root with its release in a second column, a
//! check on the one in view and "default" on the default; each choice is a
//! link, so choosing works with no script, and the `menu` module adds the
//! menu button pattern of the WAI-ARIA Authoring Practices 1.2 and returns
//! the focus to the control. Below 40 rem the control leaves the top bar
//! and the user menu holds the same choices as a segmented control
//! ([`VersionSelect::choices`] into [`crate::view::menu::UserMenu::choices`]). No specification governs
//! the layout: our own design.

use crate::html::Html;
use crate::i18n::{Arg, Locale, Text, Word, tr_with};
use crate::icon::{Glyph, icon_classed};
use crate::view::action_menu::{MenuButton, Trigger, menu_button};
use crate::view::menu::{Choice, Choices};

/// One root a console can read through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRoot {
    /// The root's name, such as "R4".
    pub name: String,
    /// Its release, such as "FHIR 4.0.1".
    pub release: String,
    /// The link that reads through it.
    pub href: String,
    /// Whether it is the server's default root.
    pub default: bool,
}

/// What the version select shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionSelect {
    /// The menu's id, which the control opens.
    pub id: String,
    /// What is chosen, such as "FHIR version": the menu's name and the
    /// start of the control's.
    pub label: String,
    /// The roots, in the order the menu lists them.
    pub roots: Vec<VersionRoot>,
    /// The name of the root in view.
    pub current: String,
}

impl VersionSelect {
    /// The root in view, or the first root when `current` names none.
    fn in_view(&self) -> Option<&VersionRoot> {
        self.roots
            .iter()
            .find(|root| root.name == self.current)
            .or_else(|| self.roots.first())
    }

    /// The same choices as the user menu holds them below 40 rem.
    #[must_use]
    pub fn choices(&self) -> Choices {
        let chosen = self.in_view().map(|root| root.name.clone());
        Choices {
            label: self.label.clone(),
            choices: self
                .roots
                .iter()
                .map(|root| Choice {
                    words: root.name.clone(),
                    href: root.href.clone(),
                    chosen: chosen.as_deref() == Some(root.name.as_str()),
                })
                .collect(),
        }
    }
}

/// Returns the choice of `root`: the check when it is `chosen`, its name,
/// and its release with "default" on the default.
fn item(locale: &Locale, root: VersionRoot, chosen: bool) -> String {
    let VersionRoot {
        name,
        release,
        href,
        default,
    } = root;
    let line = if default {
        tr_with(
            locale,
            Text::VersionDefault,
            &[("note", Arg::from(release))],
        )
    } else {
        Word::new(release)
    };
    format!(
        "<a role=\"menuitemradio\" href=\"{}\" aria-checked=\"{chosen}\" class=\"fui-menu-item fui-version-item\">{}<span class=\"fui-menu-item-text\"><span class=\"fui-menu-item-label\">{}</span></span><span class=\"fui-version-item-line\">{line}</span></a>",
        Html::text(href),
        icon_classed(Glyph::CHECK, "16", "fui-version-check"),
        Html::text(name)
    )
}

/// Returns the version select `select` for the top bar, a console's own
/// control beside the status pill in [`crate::view::shell::TopBar::end`];
/// nothing when it has no root.
#[must_use]
pub fn version_select(locale: &Locale, select: VersionSelect) -> Html {
    let Some(shown) = select.in_view().cloned() else {
        return Html::empty();
    };
    let name = tr_with(
        locale,
        Text::VersionName,
        &[
            ("label", Arg::from(select.label.clone())),
            ("value", Arg::from(shown.name.clone())),
            ("note", Arg::from(shown.release)),
        ],
    );
    let VersionSelect {
        id, label, roots, ..
    } = select;
    let items: String = roots
        .into_iter()
        .map(|root| {
            let chosen = root.name == shown.name;
            item(locale, root, chosen)
        })
        .collect::<Vec<_>>()
        .concat();
    menu_button(
        locale,
        MenuButton {
            id,
            label,
            name: name.into(),
            trigger: Trigger::Version { value: shown.name },
            copies: false,
        },
        Html::raw(items),
    )
}
