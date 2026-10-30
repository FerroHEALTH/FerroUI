// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keyboard shortcut legend: the current page's shortcuts in a dialog
//! over it, with the tests a console's own key listener asks first (R1).
//!
//! [`shortcut_legend`] draws the legend closed; its `shortcuts` module opens
//! it on `?` pressed outside a field and from a control marked
//! [`OPENS_LEGEND`], such as the user menu's Keyboard shortcuts, which with
//! no script is a link to the console's legend page. The `tabs` module
//! follows the binder's link tabs on the digit keys ([`TAB_KEYS`]). The
//! legend is a modal `dialog` shown over the page it was opened from, which
//! returns the focus there when it closes (WAI-ARIA Authoring Practices, the
//! dialog (modal) pattern). It lists each group of shortcuts in a
//! keys-and-words grid, the keys as printed on the keyboard
//! ([`crate::view::kbd`]).
//!
//! The legend also loads the `keys` module ([`KEYS_JOB`]), which binds the
//! keys every control of the page announces in `aria-keyshortcuts` (WAI-ARIA
//! 1.2): a chord such as `Ctrl Enter` or `Ctrl s` presses its control from
//! inside a field too, a single key such as `l` or `]` presses its control,
//! or focuses it when it is a field, outside a field alone, and a control
//! inside a work-list row answers only while its row holds the focus. A
//! console's go-to pairs ([`GoTo`], drawn by [`shortcut_legend_with_go_to`])
//! are a first key, then a letter, then a link of its navigation: the
//! legend lists each as that link, which the module follows. The library's
//! own controls that announce a key (a dock decision, Run, a keyed side
//! panel opener) load the module themselves.
//!
//! A phone has no keyboard, so below 40 rem no shortcut shows or acts
//! (family rule 6), and a control that opens the legend is hidden there. A
//! single-key shortcut never acts while the person types ([`takes_text`]).
//! No specification governs the component: our own design.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr};
use crate::view::dialog::{Modal, Width, modal_with};
use crate::view::kbd::{Shortcut, ShortcutGroup, shortcut_keys};

/// The id of the legend's dialog; its heading is `shortcuts-title`.
pub const LEGEND_ID: &str = "shortcuts";

/// The attribute of a control a click on which opens the legend in place of
/// following it, such as the user menu's Keyboard shortcuts link; with no
/// script the link is followed.
pub const OPENS_LEGEND: &str = "data-fui-shortcuts";

/// The job of the module that binds the keys a control announces: a
/// console's own control that announces a key on a page with no legend
/// writes `data-fui="keys"` beside its `aria-keyshortcuts`.
pub const KEYS_JOB: &str = "keys";

/// One go-to pair: the letter after the first key, and the link of the
/// navigation it follows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoToPair {
    /// The second key, a letter such as `d`.
    pub key: String,
    /// Where it leads: the address of a link of the navigation.
    pub href: String,
    /// What it leads to, in the page's locale, such as "Dashboard".
    pub does: String,
}

/// A console's go-to pairs: a first key such as `g`, then a letter, then
/// a link of its navigation, listed in the legend under their title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoTo {
    /// The group's title in the legend, in the page's locale, such as
    /// "Go to".
    pub title: String,
    /// The first key of every pair, such as `g`.
    pub lead: String,
    /// The pairs, in the order the legend lists them.
    pub pairs: Vec<GoToPair>,
}

impl GoTo {
    /// The pairs as a group of shortcuts (`g d` and its words), for a
    /// console's own page of its shortcuts.
    #[must_use]
    pub fn group(&self) -> ShortcutGroup {
        ShortcutGroup {
            title: self.title.clone(),
            shortcuts: self
                .pairs
                .iter()
                .map(|pair| Shortcut {
                    keys: format!("{} {}", self.lead, pair.key),
                    does: pair.does.clone(),
                })
                .collect(),
        }
    }
}

/// The attribute of the binder's tabs that the digit keys follow.
pub const TAB_KEYS: &str = "data-fui-tab-keys";

/// The link tab the digit `key` names among `count` tabs: `1` the first,
/// `9` the ninth; `None` for another key or a tab the row does not have.
/// The `tabs` module answers the keys by the same rule.
#[must_use]
pub fn tab_at(key: &str, count: usize) -> Option<usize> {
    let mut chars = key.chars();
    let digit = chars.next()?.to_digit(10)?;
    if chars.next().is_some() || digit == 0 {
        return None;
    }
    let at = usize::try_from(digit).ok()?.checked_sub(1)?;
    (at < count).then_some(at)
}

/// Whether an element of the tag `tag`, an input's `kind` when it is one,
/// takes typed text, where a single-key shortcut must not act. The modules
/// ask the same question of the element a key reached.
#[must_use]
pub fn takes_text(tag: &str, kind: &str, content_editable: bool) -> bool {
    if content_editable {
        return true;
    }
    match tag {
        "textarea" | "select" => true,
        "input" => !matches!(
            kind,
            "checkbox"
                | "radio"
                | "button"
                | "submit"
                | "reset"
                | "range"
                | "color"
                | "file"
                | "image"
        ),
        _ => false,
    }
}

struct GroupView {
    title: String,
    rows: Vec<(Html, String)>,
}

struct GoView {
    lead: String,
    title: String,
    rows: Vec<(Html, String, String, String)>,
}

#[derive(Template)]
#[template(path = "shortcuts/legend.html")]
struct LegendBody {
    legend_id: &'static str,
    intro: Word,
    groups: Vec<GroupView>,
    go: Option<GoView>,
}

/// Returns the legend of `groups`, drawn closed, which the `shortcuts`
/// module opens as a modal and Close closes.
///
/// A console draws it once on a page that has shortcuts.
#[must_use]
pub fn shortcut_legend(locale: &Locale, groups: Vec<ShortcutGroup>) -> Html {
    legend(locale, groups, None)
}

/// Returns the legend of `groups` and, after them, the go-to pairs
/// `go_to`, each listed as its link, which the `keys` module follows on
/// the pair's keys.
#[must_use]
pub fn shortcut_legend_with_go_to(
    locale: &Locale,
    groups: Vec<ShortcutGroup>,
    go_to: GoTo,
) -> Html {
    legend(locale, groups, Some(go_to))
}

/// Returns the legend of `groups` and `go_to`.
fn legend(locale: &Locale, groups: Vec<ShortcutGroup>, go_to: Option<GoTo>) -> Html {
    let or = tr(locale, Text::KeysOr);
    let go = go_to.map(|go_to| GoView {
        rows: go_to
            .pairs
            .into_iter()
            .map(|pair| {
                let keys = shortcut_keys(&format!("{} {}", go_to.lead, pair.key), or.clone());
                (keys, pair.href, pair.key.to_lowercase(), pair.does)
            })
            .collect(),
        lead: go_to.lead.to_lowercase(),
        title: go_to.title,
    });
    let body = LegendBody {
        legend_id: LEGEND_ID,
        intro: tr(locale, Text::ShortcutsIntro),
        groups: groups
            .into_iter()
            .map(|group| GroupView {
                title: group.title,
                rows: group
                    .shortcuts
                    .into_iter()
                    .map(|shortcut| (shortcut_keys(&shortcut.keys, or.clone()), shortcut.does))
                    .collect(),
            })
            .collect(),
        go,
    }
    .html();
    modal_with(
        Modal {
            id: LEGEND_ID.into(),
            title: tr(locale, Text::KeyboardShortcuts),
            danger: false,
            width: Width::Wide,
            body,
            actions: Html::empty(),
            open: false,
            cancel: None,
        },
        tr(locale, Text::Close),
        "dialog shortcuts keys",
    )
}

#[cfg(test)]
mod tests {
    use super::{tab_at, takes_text};

    #[test]
    fn a_digit_names_its_tab_from_one_and_nothing_else_does() {
        assert_eq!(Some(0), tab_at("1", 4));
        assert_eq!(Some(3), tab_at("4", 4));
        assert_eq!(None, tab_at("5", 4), "a tab the row does not have");
        assert_eq!(None, tab_at("0", 4));
        assert_eq!(None, tab_at("12", 20), "one key, one digit");
        assert_eq!(None, tab_at("?", 4));
    }

    #[test]
    fn a_field_takes_text_and_a_button_or_a_box_does_not() {
        assert!(
            takes_text("input", "", false),
            "an input with no type is text"
        );
        assert!(takes_text("input", "search", false));
        assert!(takes_text("textarea", "", false));
        assert!(takes_text("select", "", false));
        assert!(takes_text("div", "", true), "an editable region");
        assert!(!takes_text("input", "checkbox", false));
        assert!(!takes_text("button", "submit", false));
        assert!(!takes_text("a", "", false));
    }
}
