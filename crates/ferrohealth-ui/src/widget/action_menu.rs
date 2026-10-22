// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The menu (F13) behind every More, every split button and every row's
//! ⋯, and the select as a button with its listbox (R29).
//!
//! Both are a `<details>` whose `<summary>` is the trigger, so the browser
//! opens and closes them with no script, and every item is a link, a submit
//! button of a plain form, or a button that opens a confirmation dialog:
//! each works before the page hydrates. The `<details>` of every menu on a
//! page share one `name`, so opening one closes the others (HTML Living
//! Standard, the `details` element). The island [`crate::island::MenuButton`]
//! adds what HTML leaves out, after the WAI-ARIA Authoring Practices menu
//! button and listbox patterns: the focus moves to the first item as the
//! menu opens, the arrows, Home, End and a letter walk the items, a disabled
//! one among them, which takes no press, Escape closes it and returns the focus to its trigger, a
//! focus or a press outside closes it, choosing an item closes it, the menu
//! flips above or to the other edge where it has no room, and the phone's
//! bottom sheet shows its Cancel.
//!
//! The menu is anchored 4 px from its trigger; below 40 rem it is a bottom
//! sheet over its own scrim, with its title and Cancel (F13). Destructive
//! items come last after a separator, in the danger tone, with an ellipsis
//! when a confirmation follows (family rule 7). A disabled item says its
//! reason on a second line, never in a tooltip alone. No specification
//! governs the layout: our own design.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Arg, Text, tr, tr_with};
use crate::icon::{Glyph, icon};

/// The `name` every menu's `<details>` shares, so one opens at a time.
const GROUP: &str = "fui-menu";

/// The ellipsis after the words of an action a confirmation follows.
// NOTE: COMPONENTS.md family rule 7: a destructive action whose
// confirmation follows ends in an ellipsis, a mark of punctuation, not a word.
const ELLIPSIS: &str = "\u{2026}";

/// What an action does when it is chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// Follows the link to this address.
    Link(String),
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
    /// Opens the confirmation dialog with this id as a modal
    /// ([`crate::view::dialog`]); its words end in an ellipsis.
    Dialog(String),
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
            Act::Link(_) | Act::Post { .. } | Act::Submit(_) => self.label.clone(),
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
    /// Its name, such as "More actions for the file": the name of its
    /// trigger and its list, and the title of the phone's sheet.
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

/// How a choice of a listbox takes effect, with or without WebAssembly.
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
    pub label: String,
    /// A muted second line, such as "Latest".
    pub note: Option<String>,
    /// The language its words are in, when it is not the page's, such as a
    /// language named in its own (WCAG 2.2 SC 3.1.2).
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
    /// The words the trigger shows, such as "English" or "1.1.0 (latest of
    /// 2)".
    pub value: String,
    /// The choices, in order.
    pub choices: Vec<Choice>,
}

/// What a menu's trigger is: the island draws it, so it carries words alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trigger {
    /// The title row's More button, saying `words` beside a chevron.
    More {
        /// Its words, such as "More".
        words: String,
    },
    /// A table row's ⋯ icon button.
    Row,
    /// A split button's chevron, opening the variants of its action.
    Variants,
    /// A select button showing `value`, opening a listbox.
    Select {
        /// The words of the choice in force.
        value: String,
    },
    /// The top bar's version select showing `value`, opening a menu of
    /// `menuitemradio` choices ([`crate::widget::version`]); its name is
    /// its tooltip too.
    Version {
        /// The version in view, such as "R4".
        value: String,
    },
}

impl Trigger {
    /// Whether it opens a listbox rather than a menu.
    const fn is_listbox(&self) -> bool {
        matches!(self, Self::Select { .. })
    }
}

/// Returns the hidden inputs of a plain form, one per field.
fn hidden_fields(fields: Vec<(String, String)>) -> AnyView {
    fields
        .into_iter()
        .map(|(name, value)| view! { <input type="hidden" name=name value=value /> })
        .collect_view()
        .into_any()
}

/// Returns the inside of an item: its icon, its words and its second line.
fn item_inside(glyph: Option<Glyph>, words: String, line: Option<String>) -> AnyView {
    view! {
        {glyph.map(|glyph| icon(glyph, "16"))}
        <span class="fui-menu-item-text">
            <span>{words}</span>
            {line.map(|line| view! { <span class="fui-menu-item-desc">{line}</span> })}
        </span>
    }
    .into_any()
}

/// Returns the item `item`, in the danger tone when `danger`.
fn item(item: Item, danger: bool) -> AnyView {
    let class = if danger {
        "fui-menu-item fui-menu-danger"
    } else {
        "fui-menu-item"
    };
    let words = item.action.words();
    let Action { glyph, act, .. } = item.action;
    if let Some(reason) = item.disabled {
        let inside = item_inside(glyph, words, Some(reason));
        return view! {
            <button type="button" role="menuitem" class=class aria-disabled="true">
                {inside}
            </button>
        }
        .into_any();
    }
    let inside = item_inside(glyph, words, item.note);
    match act {
        Act::Link(href) => view! {
            <a role="menuitem" href=href class=class>
                {inside}
            </a>
        }
        .into_any(),
        Act::Post { action, fields } => view! {
            <form method="post" action=action class="fui-menu-form">
                {hidden_fields(fields)}
                <button type="submit" role="menuitem" class=class>
                    {inside}
                </button>
            </form>
        }
        .into_any(),
        Act::Submit(form) => view! {
            <button type="submit" form=form role="menuitem" class=class>
                {inside}
            </button>
        }
        .into_any(),
        Act::Dialog(target) => view! {
            <button
                type="button"
                role="menuitem"
                class=class
                command="show-modal"
                commandfor=target.clone()
                aria-controls=target
                aria-haspopup="dialog"
            >
                {inside}
            </button>
        }
        .into_any(),
    }
}

/// Returns the items of `menu`: its items, then its destructive ones after
/// a separator.
#[must_use]
pub fn items(menu: Menu) -> AnyView {
    let separated = !menu.items.is_empty() && !menu.destructive.is_empty();
    let plain = menu
        .items
        .into_iter()
        .map(|each| item(each, false))
        .collect_view();
    let danger = menu
        .destructive
        .into_iter()
        .map(|each| item(each, true))
        .collect_view();
    view! {
        {plain}
        {separated.then(|| view! { <div class="fui-menu-sep" role="separator"></div> })}
        {danger}
    }
    .into_any()
}

/// Returns the options of a listbox: one per choice, the one in force
/// selected and checked.
#[must_use]
pub fn options(choices: Vec<Choice>) -> AnyView {
    choices
        .into_iter()
        .map(|choice| {
            let selected = choice.selected.to_string();
            let inside = view! {
                <span class="fui-listbox-text">
                    <span>{choice.label}</span>
                    {choice.note.map(|note| view! { <span class="fui-listbox-note">{note}</span> })}
                </span>
                {choice.selected.then(|| icon(Glyph::CHECK, "14"))}
            };
            match choice.pick {
                Pick::Link(href) => view! {
                    <a
                        role="option"
                        href=href
                        lang=choice.lang
                        aria-selected=selected
                        class="fui-listbox-option"
                    >
                        {inside}
                    </a>
                }
                .into_any(),
                Pick::Post { action, fields } => view! {
                    <form method="post" action=action class="fui-menu-form">
                        {hidden_fields(fields)}
                        <button
                            type="submit"
                            role="option"
                            lang=choice.lang
                            aria-selected=selected
                            class="fui-listbox-option"
                        >
                            {inside}
                        </button>
                    </form>
                }
                .into_any(),
            }
        })
        .collect_view()
        .into_any()
}

/// Returns the menu `menu` behind `trigger`, named `name`; nothing for a
/// menu with no item.
fn placed(menu: Menu, trigger: Trigger, name: String) -> AnyView {
    if menu.is_empty() {
        return ().into_any();
    }
    let id = menu.id.clone();
    let label = menu.label.clone();
    let cancel = tr(Text::Cancel);
    view! {
        <crate::island::MenuButton id label name trigger cancel>
            {items(menu)}
        </crate::island::MenuButton>
    }
    .into_any()
}

/// Returns the title row's More button and its menu (F14); nothing for a
/// menu with no item.
#[must_use]
pub fn more_menu(menu: Menu) -> AnyView {
    let name = menu.label.clone();
    placed(
        menu,
        Trigger::More {
            words: tr(Text::MenuMore),
        },
        name,
    )
}

/// Returns a table row's ⋯ button and its menu, the button named by the
/// menu's label, such as "Actions for vitals.v1"; nothing for a menu with
/// no item.
#[must_use]
pub fn row_menu(menu: Menu) -> AnyView {
    let name = menu.label.clone();
    placed(menu, Trigger::Row, name)
}

/// Returns the chevron of a split button for `action` and its menu of the
/// action's variants; nothing for a menu with no item.
#[must_use]
pub fn variants_menu(action: &str, menu: Menu) -> AnyView {
    let name = tr_with(
        Text::MenuVariants,
        &[("action", Arg::from(action.to_owned()))],
    );
    placed(menu, Trigger::Variants, name)
}

/// Returns the select button `listbox` and its listbox (R29).
///
/// The button shows the choice in force and is named by the label and that
/// choice; each choice is a link or a plain form post, so choosing works
/// with no WebAssembly.
#[must_use]
pub fn listbox(listbox: Listbox) -> AnyView {
    let Listbox {
        id,
        label,
        value,
        choices,
    } = listbox;
    let name = tr_with(
        Text::SelectName,
        &[
            ("label", Arg::from(label.clone())),
            ("value", Arg::from(value.clone())),
        ],
    );
    let cancel = tr(Text::Cancel);
    view! {
        <crate::island::MenuButton id label name trigger=Trigger::Select { value } cancel>
            {options(choices)}
        </crate::island::MenuButton>
    }
    .into_any()
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

/// The item a typed `letter` moves to: the next after `current` whose words
/// start with it, wrapping, or `None` when no item does.
#[must_use]
pub fn typed(letter: &str, current: Option<usize>, words: &[String]) -> Option<usize> {
    let letter = letter.to_lowercase();
    if letter.chars().count() != 1 || letter.trim().is_empty() {
        return None;
    }
    let count = words.len();
    let start = current.map_or(0, |at| at + 1);
    (0..count).map(|step| (start + step) % count).find(|at| {
        words
            .get(*at)
            .is_some_and(|each| each.trim_start().to_lowercase().starts_with(&letter))
    })
}

/// Returns the menu or listbox `id` behind `trigger`, around `entries`.
///
/// `label` names its list and titles the phone's sheet, `name` names its
/// trigger, `entries` are the items or options the server drew, and the
/// sheet's Cancel says `cancel`. A console draws it through [`more_menu`],
/// [`row_menu`], [`variants_menu`] or [`listbox`].
#[must_use]
pub fn menu_button(
    id: String,
    label: String,
    name: String,
    trigger: Trigger,
    cancel: String,
    entries: AnyView,
) -> AnyView {
    let details = NodeRef::<leptos::html::Details>::new();
    let list = NodeRef::<leptos::html::Div>::new();
    let ready = RwSignal::new(None::<bool>);
    // NOTE: leptos-ui.md §15.8: Cancel and `aria-expanded` need the island,
    // so they appear once it runs, reading whether the menu is already open.
    Effect::new(move || ready.set(Some(browser::prepare(details, list))));
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
        let trigger_class = match trigger {
            Trigger::Row => "fui-icon-button",
            Trigger::Variants => "fui-button fui-button-secondary fui-split-toggle",
            Trigger::More { .. } | Trigger::Select { .. } => {
                "fui-button fui-button-secondary fui-more"
            }
            Trigger::Version { .. } => "fui-version-select",
        };
        (
            if version {
                "fui-anchor fui-version-anchor"
            } else {
                "fui-anchor"
            },
            trigger_class,
            "fui-menu fui-menu-compact fui-menu-anchored fui-menu-sheet",
            "menu",
            "menu",
        )
    };
    let title = matches!(trigger, Trigger::Row | Trigger::Version { .. }).then(|| name.clone());
    let face = match trigger {
        Trigger::More { words } => view! {
            {words}
            {icon(Glyph::CHEVRON_DOWN, "16")}
        }
        .into_any(),
        Trigger::Row => icon(Glyph::ELLIPSIS, "16").into_any(),
        Trigger::Variants => icon(Glyph::CHEVRON_DOWN, "16").into_any(),
        Trigger::Select { value } => view! {
            <span>{value}</span>
            {icon(Glyph::CHEVRON_DOWN, "16")}
        }
        .into_any(),
        Trigger::Version { value } => view! {
            {value}
            {icon(Glyph::CHEVRON_DOWN, "14")}
        }
        .into_any(),
    };
    view! {
        <details
            class=anchor
            name=GROUP
            node_ref=details
            on:toggle=move |_| {
                let open = browser::toggled(details, list, listbox);
                if ready.get_untracked().is_some() {
                    ready.set(Some(open));
                }
            }
            on:keydown=move |event| browser::key(details, list, &event)
            on:focusout=move |event| browser::left(details, &event)
            on:click=move |event| browser::clicked(details, &event)
        >
            <summary
                class=trigger_class
                aria-haspopup=haspopup
                aria-controls=id.clone()
                aria-label=name
                title=title
                aria-expanded=move || ready.get().map(|open| open.to_string())
            >
                {face}
            </summary>
            <div class=popup_class>
                <p class="fui-menu-sheet-title" aria-hidden="true">
                    {label.clone()}
                </p>
                <div id=id role=role aria-label=label class="fui-menu-items" node_ref=list>
                    {entries}
                </div>
                <button
                    type="button"
                    class="fui-menu-cancel"
                    hidden=move || ready.get().is_none()
                    on:click=move |_| browser::close(details)
                >
                    {cancel}
                </button>
            </div>
        </details>
    }
    .into_any()
}

/// What the browser code adds to a menu. Each runs in a handler or an
/// effect, so the server and the hydrated page draw the same.
mod browser {
    use leptos::prelude::*;
    #[cfg(target_arch = "wasm32")]
    use leptos::wasm_bindgen::JsCast as _;

    /// The menu's `<details>`.
    type Details = NodeRef<leptos::html::Details>;
    /// The menu's list of items.
    type List = NodeRef<leptos::html::Div>;

    #[cfg(target_arch = "wasm32")]
    mod dom {
        use leptos::prelude::*;
        use leptos::wasm_bindgen::JsCast as _;

        /// The items of `list` the keyboard walks: every option and every
        /// menu item, a disabled one too.
        // NOTE: WAI-ARIA Authoring Practices, "Focusability of disabled controls":
        // a disabled menu item stays focusable, so its reason is reached.
        pub(super) fn walked(list: &web_sys::HtmlElement) -> Vec<web_sys::HtmlElement> {
            let Ok(found) = list
                .query_selector_all("[role='menuitem'], [role='menuitemradio'], [role='option']")
            else {
                return Vec::new();
            };
            (0..found.length())
                .filter_map(|at| found.item(at))
                .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
                .collect()
        }

        /// The place of the focused element among `items`.
        pub(super) fn focused(items: &[web_sys::HtmlElement]) -> Option<usize> {
            let active = document().active_element()?;
            items.iter().position(|item| {
                let element: &web_sys::Element = item;
                *element == active
            })
        }

        /// Moves the focus to `element`.
        pub(super) fn focus(element: &web_sys::HtmlElement) {
            if element.focus().is_err() {
                leptos::logging::warn!("the browser refused the focus on a menu");
            }
        }

        /// The trigger of `details`.
        pub(super) fn summary(
            details: &web_sys::HtmlDetailsElement,
        ) -> Option<web_sys::HtmlElement> {
            details
                .query_selector("summary")
                .ok()
                .flatten()
                .and_then(|summary| summary.dyn_into::<web_sys::HtmlElement>().ok())
        }

        /// Turns the popup of `details`, `list` its items, to the other
        /// edge or above where it has no room, unless it is the phone's
        /// sheet.
        pub(super) fn place(
            details: &web_sys::HtmlDetailsElement,
            list: &web_sys::HtmlElement,
            listbox: bool,
        ) {
            let Some(popup) = list.parent_element() else {
                return;
            };
            let (up, side) = if listbox {
                ("is-up", "is-end")
            } else {
                ("fui-menu-up", "fui-menu-start")
            };
            let classes = popup.class_list();
            if classes.remove_2(up, side).is_err() {
                return;
            }
            let sheet = window()
                .match_media("(width < 40rem)")
                .ok()
                .flatten()
                .is_some_and(|query| query.matches());
            if sheet {
                return;
            }
            let width = window()
                .inner_width()
                .ok()
                .and_then(|width| width.as_f64())
                .unwrap_or_default();
            let height = window()
                .inner_height()
                .ok()
                .and_then(|height| height.as_f64())
                .unwrap_or_default();
            let anchor = details.get_bounding_client_rect();
            let shown = popup.get_bounding_client_rect();
            let flips = super::super::flips(
                (anchor.top(), anchor.bottom()),
                (shown.left(), shown.right(), shown.height()),
                (width, height),
            );
            let mut added = Ok(());
            if flips.0 {
                added = added.and(classes.add_1(up));
            }
            if flips.1 {
                added = added.and(classes.add_1(side));
            }
            if added.is_err() {
                leptos::logging::warn!("the browser refused to place a menu");
            }
        }
    }

    /// Takes the walked items out of the browser's tab order, so the arrows
    /// walk them, and says whether the menu is open.
    pub(super) fn prepare(details: Details, list: List) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(list) = list.get_untracked() {
                for item in dom::walked(&list) {
                    item.set_tab_index(-1);
                }
            }
            details
                .get_untracked()
                .is_some_and(|details| details.open())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _: (Details, List) = (details, list);
            false
        }
    }

    /// Places the menu as it opens and moves the focus to its first item,
    /// or a listbox's choice in force; says whether it is open.
    pub(super) fn toggled(details: Details, list: List, listbox: bool) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            let (Some(details), Some(list)) = (details.get_untracked(), list.get_untracked())
            else {
                return false;
            };
            if !details.open() {
                return false;
            }
            dom::place(&details, &list, listbox);
            let items = dom::walked(&list);
            let chosen = items
                .iter()
                .position(|item| item.get_attribute("aria-selected").as_deref() == Some("true"));
            let wanted = if dom::focused(&items).is_some() {
                None
            } else {
                chosen.or_else(|| (!items.is_empty()).then_some(0))
            };
            if let Some(item) = wanted.and_then(|at| items.get(at)) {
                dom::focus(item);
            }
            true
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _: (Details, List, bool) = (details, list, listbox);
            false
        }
    }

    /// Closes the menu and returns the focus to its trigger.
    pub(super) fn close(details: Details) {
        #[cfg(target_arch = "wasm32")]
        {
            let Some(details) = details.get_untracked() else {
                return;
            };
            details.set_open(false);
            if let Some(summary) = dom::summary(&details) {
                dom::focus(&summary);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: Details = details;
    }

    /// Answers a key pressed on the trigger or in the menu.
    pub(super) fn key(details: Details, list: List, event: &leptos::ev::KeyboardEvent) {
        #[cfg(target_arch = "wasm32")]
        {
            let (Some(element), Some(items_of)) = (details.get_untracked(), list.get_untracked())
            else {
                return;
            };
            let key = event.key();
            let on_trigger = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .is_some_and(|target| target.tag_name().eq_ignore_ascii_case("summary"));
            if key == "Escape" && element.open() {
                event.prevent_default();
                close(details);
                return;
            }
            if on_trigger {
                if !element.open() && matches!(key.as_str(), "ArrowDown" | "ArrowUp") {
                    event.prevent_default();
                    element.set_open(true);
                    let items = dom::walked(&items_of);
                    let at = if key == "ArrowUp" {
                        items.len().checked_sub(1)
                    } else {
                        (!items.is_empty()).then_some(0)
                    };
                    if let Some(item) = at.and_then(|at| items.get(at)) {
                        dom::focus(item);
                    }
                }
                return;
            }
            if key == "Tab" {
                element.set_open(false);
                return;
            }
            let items = dom::walked(&items_of);
            let current = dom::focused(&items);
            if key == " "
                && let Some(link) = current
                    .and_then(|at| items.get(at))
                    .filter(|item| item.tag_name().eq_ignore_ascii_case("a"))
            {
                event.prevent_default();
                link.click();
                return;
            }
            let next = super::moved(&key, current, items.len()).or_else(|| {
                let words: Vec<String> =
                    items.iter().map(web_sys::HtmlElement::inner_text).collect();
                super::typed(&key, current, &words)
            });
            if let Some(item) = next.and_then(|at| items.get(at)) {
                event.prevent_default();
                dom::focus(item);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (Details, List, &leptos::ev::KeyboardEvent) = (details, list, event);
    }

    /// Closes the menu when the focus leaves it.
    pub(super) fn left(details: Details, event: &leptos::ev::FocusEvent) {
        #[cfg(target_arch = "wasm32")]
        {
            let Some(element) = details.get_untracked() else {
                return;
            };
            let inside = event
                .related_target()
                .and_then(|target| target.dyn_into::<web_sys::Node>().ok())
                .is_some_and(|target| element.contains(Some(&target)));
            if element.open() && !inside {
                element.set_open(false);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (Details, &leptos::ev::FocusEvent) = (details, event);
    }

    /// Closes the menu when one of its items is chosen; a disabled item
    /// takes no press.
    pub(super) fn clicked(details: Details, event: &leptos::ev::MouseEvent) {
        #[cfg(target_arch = "wasm32")]
        {
            let item = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .and_then(|target| {
                    target
                        .closest("[role='menuitem'], [role='menuitemradio'], [role='option']")
                        .ok()
                        .flatten()
                });
            let Some(item) = item else {
                return;
            };
            if item.get_attribute("aria-disabled").as_deref() == Some("true") {
                event.prevent_default();
                return;
            }
            close(details);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (Details, &leptos::ev::MouseEvent) = (details, event);
    }
}

/// Whether an open popup flips above its anchor and to its other edge.
///
/// It flips above when it runs past the viewport's foot and fits above the
/// anchor, and to the other edge when it runs past either side. `anchor` is
/// its top and bottom, `popup` its left, right and height, `viewport` the
/// width and height, in CSS pixels.
#[must_use]
pub fn flips(anchor: (f64, f64), popup: (f64, f64, f64), viewport: (f64, f64)) -> (bool, bool) {
    let (top, bottom) = anchor;
    let (left, right, height) = popup;
    let (width, tall) = viewport;
    let up = bottom + height > tall && top - height >= 0.0;
    let side = left < 0.0 || right > width;
    (up, side)
}

#[cfg(test)]
mod tests {
    use super::{Act, Action, flips, moved, typed};

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
    fn a_letter_moves_to_the_next_item_it_starts() {
        let words = ["Copy path", "Download", "Check", "Delete"].map(String::from);
        assert_eq!(
            Some(2),
            typed("c", Some(0), &words),
            "the next, not the first"
        );
        assert_eq!(Some(0), typed("C", Some(2), &words), "wraps, any case");
        assert_eq!(Some(1), typed("d", None, &words));
        assert_eq!(None, typed("x", None, &words));
        assert_eq!(None, typed("ArrowDown", None, &words), "a named key");
        assert_eq!(None, typed(" ", None, &words));
    }

    #[test]
    fn a_popup_flips_only_where_it_has_no_room() {
        let viewport = (1440.0, 900.0);
        assert_eq!(
            (false, false),
            flips((100.0, 136.0), (900.0, 1100.0, 200.0), viewport)
        );
        assert_eq!(
            (true, false),
            flips((800.0, 836.0), (900.0, 1100.0, 200.0), viewport),
            "no room below"
        );
        assert_eq!(
            (false, false),
            flips((100.0, 836.0), (900.0, 1100.0, 200.0), (1440.0, 900.0)),
            "no room above either: it stays below"
        );
        assert_eq!(
            (false, true),
            flips((100.0, 136.0), (-40.0, 160.0, 200.0), viewport),
            "past the start edge"
        );
        assert_eq!(
            (false, true),
            flips((100.0, 136.0), (1300.0, 1500.0, 200.0), viewport),
            "past the end edge"
        );
    }

    #[test]
    fn an_action_a_confirmation_follows_ends_in_an_ellipsis() {
        let delete = Action {
            glyph: None,
            label: String::from("Delete the file"),
            act: Act::Dialog(String::from("delete")),
        };
        assert_eq!("Delete the file\u{2026}", delete.words());
        let open = Action {
            glyph: None,
            label: String::from("Open"),
            act: Act::Link(String::from("/files")),
        };
        assert_eq!("Open", open.words());
    }
}
