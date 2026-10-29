// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The menu (F13) behind every More, every split button and every row's
//! ⋯, and the select as a button with its listbox (R29).
//!
//! Both are a `<details>` whose `<summary>` is the trigger, so the browser
//! opens and closes them with no script, and every item is a link, a submit
//! button of a plain form or of the console's own form (by `formaction`
//! where it posts elsewhere), or a button that opens a confirmation dialog.
//! The `<details>` of every menu on a page share one `name`, so opening one
//! closes the others (HTML Living Standard, the `details` element). The
//! `menu` module adds what HTML leaves out, after the WAI-ARIA Authoring
//! Practices menu button and listbox patterns: the focus moves to the first
//! item as the menu opens, the arrows, Home, End and a letter walk the
//! items, a disabled one among them, Escape closes it and returns the
//! focus, a focus or a press outside closes it, choosing an item closes it,
//! the menu flips where it has no room, and the phone's bottom sheet shows
//! its Cancel.
//!
//! The menu is anchored 4 px from its trigger; below 40 rem it is a bottom
//! sheet with its title and Cancel. Destructive items come last after a
//! separator, in the danger tone, with an ellipsis when a confirmation
//! follows (family rule 7). A disabled item says its reason on a second
//! line, never in a tooltip alone. No specification governs the layout:
//! our own design.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Arg, Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};

/// The ellipsis after the words of an action a confirmation follows.
// NOTE: COMPONENTS.md family rule 7: a destructive action whose
// confirmation follows ends in an ellipsis, a mark of punctuation, not a word.
const ELLIPSIS: &str = "\u{2026}";

/// What an action does when it is chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// Follows the link to this address.
    Link(String),
    /// Downloads what this address answers, such as an export.
    Download(String),
    /// Posts a plain form to `action` with each of `fields` as a hidden
    /// input, named and valued.
    Post {
        /// Where the form posts.
        action: String,
        /// The hidden inputs, each its name and value.
        fields: Vec<(String, String)>,
    },
    /// Submits the form with this id, such as an editor's own form.
    Submit(String),
    /// Submits the console's own form to another address by `formaction`,
    /// so no form of the item's own sits inside the console's.
    SubmitTo {
        /// Where the form is submitted, in place of its own `action`.
        action: String,
        /// The id of the console's form, when the item sits outside it.
        form: Option<String>,
    },
    /// Opens the confirmation dialog with this id as a modal
    /// ([`crate::view::dialog`]); its words end in an ellipsis.
    Dialog(String),
    /// Puts this text on the clipboard and says so in a toast. Only script
    /// can copy, so the item is hidden until the menu's module runs.
    Copy(String),
}

/// One action: its icon, its words and what it does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    /// The icon beside the words, when there is one.
    pub glyph: Option<Glyph>,
    /// The words, without an ellipsis: one is added where a confirmation
    /// follows.
    pub label: String,
    /// What choosing it does.
    pub act: Act,
}

impl Action {
    /// The action `label` with `glyph`, doing `act`.
    #[must_use]
    pub fn new(glyph: Option<Glyph>, label: impl Into<String>, act: Act) -> Self {
        Self {
            glyph,
            label: label.into(),
            act,
        }
    }

    /// The words as they are drawn: with an ellipsis where a confirmation
    /// follows.
    #[must_use]
    pub fn words(&self) -> String {
        match self.act {
            Act::Dialog(_) => format!("{}{ELLIPSIS}", self.label),
            Act::Link(_)
            | Act::Download(_)
            | Act::Post { .. }
            | Act::Submit(_)
            | Act::SubmitTo { .. }
            | Act::Copy(_) => self.label.clone(),
        }
    }
}

/// One item of a menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// What it shows and does.
    pub action: Action,
    /// A muted second line, when the action needs one.
    pub note: Option<String>,
    /// Why it cannot be chosen now, shown as its second line in place of
    /// the note; `None` for an item that can be chosen.
    pub disabled: Option<String>,
}

impl Item {
    /// The item that does `action`.
    #[must_use]
    pub const fn new(action: Action) -> Self {
        Self {
            action,
            note: None,
            disabled: None,
        }
    }

    /// This item, with `note` as its second line.
    #[must_use]
    pub fn note(self, note: impl Into<String>) -> Self {
        Self {
            note: Some(note.into()),
            ..self
        }
    }

    /// This item, disabled for `reason`, which shows as its second line.
    #[must_use]
    pub fn disabled(self, reason: impl Into<String>) -> Self {
        Self {
            disabled: Some(reason.into()),
            ..self
        }
    }
}

impl From<Action> for Item {
    fn from(action: Action) -> Self {
        Self::new(action)
    }
}

/// What a menu holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Menu {
    /// The menu's id, which its trigger controls.
    pub id: String,
    /// Its name, such as "More actions for the file".
    pub label: String,
    /// The items, in order.
    pub items: Vec<Item>,
    /// The destructive items, drawn last after a separator in the danger
    /// tone.
    pub destructive: Vec<Item>,
}

impl Menu {
    /// The menu `id`, named `label`, of `items`, with no destructive item.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>, items: Vec<Item>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items,
            destructive: Vec::new(),
        }
    }

    /// This menu, with `destructive` last after a separator.
    #[must_use]
    pub fn destructive(self, destructive: Vec<Item>) -> Self {
        Self {
            destructive,
            ..self
        }
    }

    /// Whether the menu holds no item, so it is not drawn.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.destructive.is_empty()
    }
}

/// How a choice of a listbox takes effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pick {
    /// Follows the link to this address, such as `?version=1.0.0`.
    Link(String),
    /// Posts a plain form to `action` with each of `fields` as a hidden
    /// input, such as a preference the console keeps in a cookie.
    Post {
        /// Where the form posts.
        action: String,
        /// The hidden inputs, each its name and value.
        fields: Vec<(String, String)>,
    },
}

/// One choice of a listbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// Its words.
    pub label: Word,
    /// A muted second line, such as "Latest".
    pub note: Option<Word>,
    /// The language its words are in, when it is not the page's (WCAG 2.2
    /// SC 3.1.2).
    pub lang: Option<String>,
    /// Whether it is the choice in force.
    pub selected: bool,
    /// How choosing it takes effect.
    pub pick: Pick,
}

/// What a select button and its listbox show (R29).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listbox {
    /// The listbox's id, which its trigger controls.
    pub id: String,
    /// What is chosen, such as "Language": the name of the list and the
    /// title of the phone's sheet.
    pub label: String,
    /// The words the trigger shows, such as "English".
    pub value: Word,
    /// The choices, in order.
    pub choices: Vec<Choice>,
}

/// What a menu's trigger is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trigger {
    /// The title row's More button, saying `words` beside a chevron.
    More {
        /// Its words, such as "More".
        words: Word,
    },
    /// A table row's ⋯ icon button.
    Row,
    /// A split button's chevron, opening the variants of its action.
    Variants,
    /// A small secondary button saying `words` after `glyph`.
    Labelled {
        /// Its icon.
        glyph: Glyph,
        /// Its words, such as "Export".
        words: Word,
    },
    /// A select button showing `value`, opening a listbox.
    Select {
        /// The words of the choice in force.
        value: Word,
    },
    /// The top bar's version select showing `value`, opening a menu of
    /// `menuitemradio` choices ([`crate::view::version`]).
    Version {
        /// The version in view, such as "R4".
        value: String,
    },
}

impl Trigger {
    const fn is_listbox(&self) -> bool {
        matches!(self, Self::Select { .. })
    }
}

/// Returns the hidden inputs of a plain form, one per field.
fn hidden_fields(fields: &[(String, String)]) -> String {
    fields
        .iter()
        .map(|(name, value)| {
            format!(
                "<input type=\"hidden\" name=\"{}\" value=\"{}\">",
                Html::text(name),
                Html::text(value)
            )
        })
        .collect()
}

/// Returns the inside of an item: its icon, its words and its second line.
fn item_inside(glyph: Option<Glyph>, words: &str, line: Option<&str>) -> String {
    let line = line.map_or(String::new(), |line| {
        format!("<span class=\"fui-menu-item-desc\">{}</span>", Html::text(line))
    });
    format!(
        "{}<span class=\"fui-menu-item-text\"><span>{}</span>{line}</span>",
        glyph.map(|glyph| icon(glyph, "16")).unwrap_or_default(),
        Html::text(words)
    )
}

/// Returns the item `item`, in the danger tone when `danger`.
fn item(item: &Item, danger: bool) -> Html {
    let class = if danger {
        "fui-menu-item fui-menu-danger"
    } else {
        "fui-menu-item"
    };
    let words = item.action.words();
    let glyph = item.action.glyph;
    if let Some(reason) = &item.disabled {
        return Html::raw(format!(
            "<button type=\"button\" role=\"menuitem\" class=\"{class}\" aria-disabled=\"true\">{}</button>",
            item_inside(glyph, &words, Some(reason))
        ));
    }
    let inside = item_inside(glyph, &words, item.note.as_deref());
    Html::raw(match &item.action.act {
        Act::Link(href) => format!(
            "<a role=\"menuitem\" href=\"{}\" class=\"{class}\">{inside}</a>",
            Html::text(href)
        ),
        Act::Download(href) => format!(
            "<a role=\"menuitem\" href=\"{}\" rel=\"external\" download=\"\" class=\"{class}\">{inside}</a>",
            Html::text(href)
        ),
        Act::Post { action, fields } => format!(
            "<form method=\"post\" action=\"{}\" class=\"fui-menu-form\">{}<button type=\"submit\" role=\"menuitem\" class=\"{class}\">{inside}</button></form>",
            Html::text(action),
            hidden_fields(fields)
        ),
        Act::Submit(form) => format!(
            "<button type=\"submit\" form=\"{}\" role=\"menuitem\" class=\"{class}\">{inside}</button>",
            Html::text(form)
        ),
        Act::SubmitTo { action, form } => format!(
            "<button type=\"submit\"{} formaction=\"{}\" role=\"menuitem\" class=\"{class}\">{inside}</button>",
            form.as_ref()
                .map_or(String::new(), |form| format!(" form=\"{}\"", Html::text(form))),
            Html::text(action)
        ),
        Act::Dialog(target) => format!(
            "<button type=\"button\" role=\"menuitem\" class=\"{class}\" command=\"show-modal\" commandfor=\"{}\" aria-controls=\"{}\" aria-haspopup=\"dialog\">{inside}</button>",
            Html::text(target),
            Html::text(target)
        ),
        // NOTE: a control only script can work is hidden until the menu's module shows it.
        Act::Copy(text) => format!(
            "<button type=\"button\" role=\"menuitem\" class=\"{class}\" data-fui-copy=\"{}\" hidden>{inside}</button>",
            Html::text(text)
        ),
    })
}

/// Whether `menu` holds a copy item, whose module needs the copy words.
fn copies(menu: &Menu) -> bool {
    menu.items
        .iter()
        .chain(&menu.destructive)
        .any(|item| matches!(item.action.act, Act::Copy(_)) && item.disabled.is_none())
}

/// Returns the items of `menu`: its items, then its destructive ones after
/// a separator.
#[must_use]
pub fn items(menu: &Menu) -> Html {
    let separated = !menu.items.is_empty() && !menu.destructive.is_empty();
    let mut out: Html = menu.items.iter().map(|each| item(each, false)).collect();
    if separated {
        out = out.then(Html::raw(String::from(
            "<div class=\"fui-menu-sep\" role=\"separator\"></div>",
        )));
    }
    out.then(menu.destructive.iter().map(|each| item(each, true)).collect())
}

/// Returns the options of a listbox: one per choice, the one in force
/// selected and checked.
#[must_use]
pub fn options(choices: &[Choice]) -> Html {
    choices
        .iter()
        .map(|choice| {
            let note = choice.note.as_ref().map_or(String::new(), |note| {
                format!("<span class=\"fui-listbox-note\">{note}</span>")
            });
            let inside = format!(
                "<span class=\"fui-listbox-text\"><span>{}</span>{note}</span>{}",
                choice.label,
                if choice.selected {
                    icon(Glyph::CHECK, "14")
                } else {
                    Html::empty()
                }
            );
            let lang = choice
                .lang
                .as_ref()
                .map_or(String::new(), |lang| format!(" lang=\"{}\"", Html::text(lang)));
            Html::raw(match &choice.pick {
                Pick::Link(href) => format!(
                    "<a role=\"option\" href=\"{}\"{lang} aria-selected=\"{}\" class=\"fui-listbox-option\">{inside}</a>",
                    Html::text(href),
                    choice.selected
                ),
                Pick::Post { action, fields } => format!(
                    "<form method=\"post\" action=\"{}\" class=\"fui-menu-form\">{}<button type=\"submit\" role=\"option\"{lang} aria-selected=\"{}\" class=\"fui-listbox-option\">{inside}</button></form>",
                    Html::text(action),
                    hidden_fields(fields),
                    choice.selected
                ),
            })
        })
        .collect()
}

/// Returns the menu `menu` behind `trigger`, named `name`; nothing for a
/// menu with no item.
fn placed(locale: &Locale, menu: &Menu, trigger: Trigger, name: String) -> Html {
    if menu.is_empty() {
        return Html::empty();
    }
    menu_button(
        locale,
        MenuButton {
            id: menu.id.clone(),
            label: menu.label.clone(),
            name,
            trigger,
            copies: copies(menu),
        },
        items(menu),
    )
}

/// Returns the title row's More button and its menu (F14).
#[must_use]
pub fn more_menu(locale: &Locale, menu: &Menu) -> Html {
    placed(
        locale,
        menu,
        Trigger::More {
            words: tr(locale, Text::MenuMore),
        },
        menu.label.clone(),
    )
}

/// Returns a table row's ⋯ button and its menu, the button named by the
/// menu's label, such as "Actions for vitals.v1".
#[must_use]
pub fn row_menu(locale: &Locale, menu: &Menu) -> Html {
    placed(locale, menu, Trigger::Row, menu.label.clone())
}

/// Returns the chevron of a split button for `action` and its menu of the
/// action's variants.
#[must_use]
pub fn variants_menu(locale: &Locale, action: &str, menu: &Menu) -> Html {
    let name = tr_with(
        locale,
        Text::MenuVariants,
        &[("action", Arg::from(action.to_owned()))],
    );
    placed(locale, menu, Trigger::Variants, name.into())
}

/// Returns a small secondary button saying `words` after `glyph` and its
/// menu, such as the results pane's Export with its formats (E7).
#[must_use]
pub fn labelled_menu(locale: &Locale, glyph: Glyph, words: Word, menu: &Menu) -> Html {
    placed(
        locale,
        menu,
        Trigger::Labelled { glyph, words },
        menu.label.clone(),
    )
}

/// Returns the select button `listbox` and its listbox (R29): the button
/// shows the choice in force and is named by the label and that choice;
/// each choice is a link or a plain form post.
#[must_use]
pub fn listbox(locale: &Locale, listbox: &Listbox) -> Html {
    let name = tr_with(
        locale,
        Text::SelectName,
        &[
            ("label", Arg::from(listbox.label.clone())),
            ("value", Arg::from(listbox.value.clone())),
        ],
    );
    menu_button(
        locale,
        MenuButton {
            id: listbox.id.clone(),
            label: listbox.label.clone(),
            name: name.into(),
            trigger: Trigger::Select {
                value: listbox.value.clone(),
            },
            copies: false,
        },
        options(&listbox.choices),
    )
}

/// Where a key moves the focus among `count` items from `current`, wrapping
/// at the ends, or `None` for a key the menu leaves alone.
#[must_use]
pub fn moved(key: &str, current: Option<usize>, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    match (key, current) {
        ("ArrowDown", Some(at)) if at < last => Some(at + 1),
        ("ArrowDown" | "Home", _) => Some(0),
        ("ArrowUp", Some(at)) if at > 0 => Some(at - 1),
        ("ArrowUp" | "End", _) => Some(last),
        _ => None,
    }
}

/// Whether an open popup flips above its anchor and to its other edge:
/// above when it runs past the viewport's foot and fits above the anchor,
/// to the other edge when it runs past either side.
#[must_use]
pub fn flips(anchor: (f64, f64), popup: (f64, f64, f64), viewport: (f64, f64)) -> (bool, bool) {
    let (top, bottom) = anchor;
    let (left, right, height) = popup;
    let (width, tall) = viewport;
    let up = bottom + height > tall && top - height >= 0.0;
    let side = left < 0.0 || right > width;
    (up, side)
}

/// The classes of the trigger of a menu, which is no listbox.
const fn menu_trigger_class(trigger: &Trigger) -> &'static str {
    match trigger {
        Trigger::Row => "fui-icon-button",
        Trigger::Variants => "fui-button fui-button-secondary fui-split-toggle",
        Trigger::Labelled { .. } => "fui-button fui-button-secondary fui-button-small",
        Trigger::More { .. } | Trigger::Select { .. } => {
            "fui-button fui-button-secondary fui-more"
        }
        Trigger::Version { .. } => "fui-version-select",
    }
}

/// Returns what `trigger` shows: its words, its value or its icon.
fn face(trigger: &Trigger) -> Html {
    match trigger {
        Trigger::More { words } => Html::from(words.clone()).then(icon(Glyph::CHEVRON_DOWN, "16")),
        Trigger::Row => icon(Glyph::ELLIPSIS, "16"),
        Trigger::Variants => icon(Glyph::CHEVRON_DOWN, "16"),
        Trigger::Labelled { glyph, words } => icon(*glyph, "16").then(words.clone().into()),
        Trigger::Select { value } => {
            Html::raw(format!("<span>{value}</span>")).then(icon(Glyph::CHEVRON_DOWN, "16"))
        }
        Trigger::Version { value } => Html::text(value).then(icon(Glyph::CHEVRON_DOWN, "14")),
    }
}

/// What a menu button draws around its entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuButton {
    /// The id of the menu or listbox, which the trigger controls.
    pub id: String,
    /// The name of its list and the title of the phone's sheet.
    pub label: String,
    /// The name of its trigger.
    pub name: String,
    /// What the trigger is.
    pub trigger: Trigger,
    /// Whether the entries hold a copy item, whose outcome the module says
    /// in a toast with the copy words.
    pub copies: bool,
}

#[derive(Template)]
#[template(path = "menu/menu_button.html")]
struct MenuButtonView {
    anchor: &'static str,
    trigger_class: &'static str,
    popup_class: &'static str,
    role: &'static str,
    haspopup: &'static str,
    id: String,
    label: String,
    name: String,
    title: Option<String>,
    face: Html,
    entries: Html,
    cancel: Word,
    copied: Option<String>,
    copy_refused: String,
}

/// Returns the menu or listbox `button` around `entries`, the items or
/// options the console drew. A console draws it through [`more_menu`],
/// [`row_menu`], [`variants_menu`], [`labelled_menu`] or [`listbox`].
#[must_use]
pub fn menu_button(locale: &Locale, button: MenuButton, entries: Html) -> Html {
    let MenuButton {
        id,
        label,
        name,
        trigger,
        copies,
    } = button;
    let listbox = trigger.is_listbox();
    let version = matches!(trigger, Trigger::Version { .. });
    let (anchor, trigger_class, popup_class, role, haspopup) = if listbox {
        (
            "fui-select-anchor",
            "fui-select fui-select-button",
            "fui-listbox fui-menu-sheet",
            "listbox",
            "listbox",
        )
    } else {
        (
            if version {
                "fui-anchor fui-version-anchor"
            } else {
                "fui-anchor"
            },
            menu_trigger_class(&trigger),
            "fui-menu fui-menu-compact fui-menu-anchored fui-menu-sheet",
            "menu",
            "menu",
        )
    };
    let title = matches!(trigger, Trigger::Row | Trigger::Version { .. }).then(|| name.clone());
    MenuButtonView {
        anchor,
        trigger_class,
        popup_class,
        role,
        haspopup,
        id,
        label,
        name,
        title,
        face: face(&trigger),
        entries,
        cancel: tr(locale, Text::Cancel),
        copied: copies.then(|| tr(locale, Text::CopySaidCopied).text().to_owned()),
        copy_refused: tr(locale, Text::CopySaidRefused).text().to_owned(),
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::{Act, Action, flips, moved};

    #[test]
    fn the_arrows_wrap_and_home_and_end_go_to_the_ends() {
        assert_eq!(Some(0), moved("ArrowDown", None, 3), "from the trigger");
        assert_eq!(Some(1), moved("ArrowDown", Some(0), 3));
        assert_eq!(Some(0), moved("ArrowDown", Some(2), 3), "wraps forward");
        assert_eq!(Some(2), moved("ArrowUp", Some(0), 3), "wraps back");
        assert_eq!(Some(0), moved("Home", Some(2), 3));
        assert_eq!(Some(2), moved("End", Some(0), 3));
        assert_eq!(None, moved("Enter", Some(0), 3));
        assert_eq!(None, moved("ArrowDown", None, 0), "no item to move to");
    }

    #[test]
    fn a_popup_flips_only_where_it_has_no_room() {
        let viewport = (1440.0, 900.0);
        assert_eq!((false, false), flips((100.0, 136.0), (900.0, 1100.0, 200.0), viewport));
        assert_eq!((true, false), flips((800.0, 836.0), (900.0, 1100.0, 200.0), viewport));
        assert_eq!((false, false), flips((100.0, 836.0), (900.0, 1100.0, 200.0), viewport));
        assert_eq!((false, true), flips((100.0, 136.0), (-40.0, 160.0, 200.0), viewport));
        assert_eq!((false, true), flips((100.0, 136.0), (1300.0, 1500.0, 200.0), viewport));
    }

    #[test]
    fn an_action_a_confirmation_follows_ends_in_an_ellipsis() {
        let delete = Action::new(None, "Delete the file", Act::Dialog(String::from("delete")));
        assert_eq!("Delete the file\u{2026}", delete.words());
        let open = Action::new(None, "Open", Act::Link(String::from("/files")));
        assert_eq!("Open", open.words());
    }
}
