// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The user menu (F13), the one place in the top bar that sets the theme.
//!
//! A button with the operator's name opens who is signed in over an ARIA
//! menu of Your preferences, the theme as three choices, Keyboard shortcuts
//! with its key, and Sign out, last after a separator (family rule 7). The theme is set here and in Your preferences alone (family
//! rule 5). Below 40 rem the menu opens as a bottom sheet with Cancel.
//!
//! The menu is an HTML popover its button opens, so the browser opens it,
//! closes it on Escape or a click outside and returns the focus to the
//! button, before the page has hydrated and with no WebAssembly. Every item
//! works without it: the links lead on, and the theme and Sign out are
//! plain forms that post to the console's routes. The island
//! [`crate::island::UserMenuButton`] adds the menu button pattern of the
//! WAI-ARIA Authoring Practices 1.2: the button says whether the menu is
//! open, once the island runs and never before, the focus moves to the first item as it opens, Up and Down move
//! between the items and wrap, Home and End go to the first and the last,
//! and Tab closes the menu.

use leptos::prelude::*;

use crate::i18n::{Text, tr};
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
    /// The line under it: the role and the identity provider, such as
    /// "Bridge operator · FerroHEALTH ID".
    pub line: String,
    /// The link to Your preferences.
    pub preferences: String,
    /// The theme choice.
    pub theme: ThemeChoice,
    /// The groups the menu holds below 40 rem alone, one per control the
    /// top bar shows from 40 rem up (a brand, the FHIR version of
    /// [`crate::widget::version::VersionSelect::choices`]), in order.
    pub choices: Vec<Choices>,
    /// The link to the keyboard shortcuts, shown from 40 rem up (family
    /// rule 6), if the console has them.
    pub shortcuts: Option<String>,
    /// The console's sign-out route, a plain form post it answers with a
    /// redirect, or `None` for a console with no sign-in.
    pub sign_out: Option<String>,
}

/// The item a key moves the focus to from `current` of `count` items.
///
/// Down and Up move and wrap, Home and End go to the ends, and from no item
/// Down goes to the first and Up to the last; any other key is `None`.
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

/// The id of the ARIA menu inside the user menu's popup `id`.
fn list_id(id: &str) -> String {
    format!("{id}-menu")
}

/// Returns a menu item that leads to `href`, with `glyph`, `words` and the
/// key `key` it answers to, if any.
fn link_item(
    href: String,
    glyph: Glyph,
    words: String,
    key: Option<&'static str>,
    class: &'static str,
) -> impl IntoView + use<> {
    view! {
        <a role="menuitem" href=href class=class>
            {icon(glyph, "16")}
            <span class="fui-menu-item-text">
                <span class="fui-menu-item-label">{words}</span>
            </span>
            {key.map(|key| view! { <kbd class="fui-kbd">{key}</kbd> })}
        </a>
    }
}

/// Returns the theme as three `menuitemradio` choices, the one in force
/// checked, each a submit button of one plain form.
fn theme_group(id: &str, theme: ThemeChoice) -> impl IntoView + use<> {
    let label_id = format!("{id}-theme");
    let buttons = Theme::ALL
        .into_iter()
        .map(|choice| {
            view! {
                <button
                    type="submit"
                    name="theme"
                    value=choice.key()
                    role="menuitemradio"
                    class="fui-segment"
                    aria-checked=(choice == theme.current).to_string()
                >
                    {icon(glyph(choice), "14")}
                    {choice.label()}
                </button>
            }
        })
        .collect_view();
    view! {
        <form method="post" action=theme.action class="fui-menu-theme">
            <input type="hidden" name="back" value=theme.back />
            <div class="fui-menu-label" id=label_id.clone()>
                {tr(Text::ThemeSwitch)}
            </div>
            <div role="group" aria-labelledby=label_id class="fui-segmented">
                {buttons}
            </div>
        </form>
    }
}

/// Returns the group `choices`, the menu's group number `at`, as
/// `menuitemradio` links, the one in force checked, shown below 40 rem
/// alone.
fn choices_group(id: &str, at: usize, choices: Choices) -> impl IntoView + use<> {
    let label_id = if at == 0 {
        format!("{id}-choices")
    } else {
        format!("{id}-choices-{at}")
    };
    let links = choices
        .choices
        .into_iter()
        .map(|choice| {
            view! {
                <a
                    role="menuitemradio"
                    href=choice.href
                    class="fui-segment"
                    aria-checked=choice.chosen.to_string()
                >
                    {choice.words}
                </a>
            }
        })
        .collect_view();
    view! {
        <div class="fui-menu-version">
            <div class="fui-menu-label" id=label_id.clone()>
                {choices.label}
            </div>
            <div role="group" aria-labelledby=label_id class="fui-segmented">
                {links}
            </div>
        </div>
    }
}

/// Returns Sign out, a plain form posting to `action`.
fn sign_out_item(action: String) -> impl IntoView {
    view! {
        <div class="fui-menu-sep" role="separator"></div>
        <form method="post" action=action class="fui-menu-form">
            <button type="submit" role="menuitem" class="fui-menu-item">
                {icon(Glyph::LOG_OUT, "16")}
                <span class="fui-menu-item-text">
                    <span class="fui-menu-item-label">{tr(Text::SignOut)}</span>
                </span>
            </button>
        </form>
    }
}

/// Returns the user menu `menu`: its button and, behind it, the menu the
/// island [`crate::island::UserMenuButton`] opens.
///
/// Who is signed in heads the popup outside the ARIA menu, which holds the
/// items alone. Both are drawn on the server and handed to the island as
/// its children, so their words and links never reach the WebAssembly.
#[must_use]
pub fn user_menu(menu: UserMenu) -> AnyView {
    let head = view! {
        <div class="fui-menu-header">
            <strong>{menu.who}</strong>
            <span>{menu.line}</span>
        </div>
    };
    let list_id = list_id(menu.id);
    let label = menu.label.clone();
    let preferences = link_item(
        menu.preferences,
        Glyph::SETTINGS,
        tr(Text::YourPreferences),
        None,
        "fui-menu-item",
    );
    let choices = menu
        .choices
        .into_iter()
        .enumerate()
        .map(|(at, choices)| choices_group(menu.id, at, choices))
        .collect_view();
    let theme = theme_group(menu.id, menu.theme);
    let shortcuts = menu.shortcuts.map(|href| {
        link_item(
            href,
            Glyph::KEYBOARD,
            tr(Text::KeyboardShortcuts),
            Some("?"),
            "fui-menu-item fui-wide-only",
        )
    });
    let sign_out = menu.sign_out.map(sign_out_item);
    view! {
        <crate::island::UserMenuButton
            id=menu.id.to_owned()
            label=menu.label
            name=menu.name
            cancel=tr(Text::Cancel)
        >
            {head}
            <div class="fui-menu-sep"></div>
            <div id=list_id role="menu" aria-label=label class="fui-menu-items">
                {preferences}
                {choices}
                {theme}
                {shortcuts}
                {sign_out}
            </div>
        </crate::island::UserMenuButton>
    }
    .into_any()
}

/// Returns the menu button `id` and its popup, with the menu button's
/// behaviour.
///
/// The button shows `name` beside its icon, or is named `label` with none;
/// the popup, its sheet titled `label`, holds `items`, among them the ARIA
/// menu whose id is `id` and `-menu`, and its sheet's Cancel says
/// `cancel`. A console draws it through [`user_menu`].
#[must_use]
pub fn menu_button(
    id: String,
    label: String,
    name: Option<String>,
    cancel: String,
    items: AnyView,
) -> AnyView {
    let open = RwSignal::new(None::<bool>);
    let trigger = NodeRef::<leptos::html::Button>::new();
    let menu = NodeRef::<leptos::html::Div>::new();
    // NOTE: leptos-ui.md §15.8: `aria-expanded` needs the island, so it
    // appears once it runs, reading whether the menu is already open.
    Effect::new(move || open.set(Some(browser::is_open(menu))));
    let on_toggle = move |_| {
        let now = browser::is_open(menu);
        open.set(Some(now));
        if now {
            browser::focus_item(menu, 0);
        } else {
            browser::return_focus(menu, trigger);
        }
    };
    let on_menu_key = move |event: leptos::ev::KeyboardEvent| {
        let key = event.key();
        if key == "Tab" {
            browser::close(menu);
            return;
        }
        if let Some(next) = step(&key, browser::focused_item(menu), browser::item_count(menu)) {
            event.prevent_default();
            browser::focus_item(menu, next);
        }
    };
    let on_trigger_key = move |event: leptos::ev::KeyboardEvent| {
        let key = event.key();
        if key == "ArrowDown" || key == "ArrowUp" {
            event.prevent_default();
            browser::open_at(menu, key == "ArrowUp");
        }
    };
    let named = name.is_none().then(|| label.clone());
    let title = name.clone();
    let shown = name.map(|name| view! { <span class="fui-operator-name">{name}</span> });
    let target = id.clone();
    let cancel_target = id.clone();
    let controls = list_id(&id);
    let sheet_title = label;
    view! {
        <span class="fui-anchor">
            <button
                type="button"
                class="fui-button fui-button-ghost fui-operator"
                popovertarget=target
                aria-haspopup="menu"
                aria-expanded=move || open.get().map(|open| open.to_string())
                aria-controls=controls
                aria-label=named
                title=title
                node_ref=trigger
                on:keydown=on_trigger_key
            >
                {icon(Glyph::CIRCLE_USER, "16")}
                {shown}
                {icon(Glyph::CHEVRON_DOWN, "14")}
            </button>
            <div
                id=id
                popover="auto"
                class="fui-menu fui-menu-compact fui-menu-anchored fui-menu-sheet fui-user-menu"
                node_ref=menu
                on:toggle=on_toggle
                on:keydown=on_menu_key
            >
                <p class="fui-menu-sheet-title" aria-hidden="true">
                    {sheet_title}
                </p>
                {items}
                <button
                    type="button"
                    class="fui-menu-cancel"
                    popovertarget=cancel_target
                    popovertargetaction="hide"
                >
                    {cancel}
                </button>
            </div>
        </span>
    }
    .into_any()
}

/// What the browser code adds to the menu: the items are the island's
/// server children, out of reach of a node reference, so they are found
/// inside the menu by their role.
mod browser {
    use leptos::prelude::*;

    /// The selector of the menu's items.
    #[cfg(target_arch = "wasm32")]
    const ITEMS: &str = r#"[role="menuitem"], [role="menuitemradio"]"#;

    /// The items of `menu` the page shows, in order.
    #[cfg(target_arch = "wasm32")]
    fn items(menu: NodeRef<leptos::html::Div>) -> Vec<web_sys::HtmlElement> {
        use leptos::wasm_bindgen::JsCast as _;
        let Some(menu) = menu.get_untracked() else {
            return Vec::new();
        };
        let Ok(found) = menu.query_selector_all(ITEMS) else {
            return Vec::new();
        };
        (0..found.length())
            .filter_map(|index| found.item(index))
            .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
            // NOTE: CSSOM View, `offsetParent`: an item hidden at this width has none.
            .filter(|item| item.offset_parent().is_some())
            .collect()
    }

    /// Whether `menu` is open.
    pub(super) fn is_open(menu: NodeRef<leptos::html::Div>) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            menu.get_untracked()
                .is_some_and(|menu| menu.matches(":popover-open").unwrap_or(false))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _: NodeRef<leptos::html::Div> = menu;
            false
        }
    }

    /// The number of items of `menu` the page shows.
    pub(super) fn item_count(menu: NodeRef<leptos::html::Div>) -> u32 {
        #[cfg(target_arch = "wasm32")]
        {
            u32::try_from(items(menu).len()).unwrap_or(u32::MAX)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _: NodeRef<leptos::html::Div> = menu;
            0
        }
    }

    /// The place of the focused item among the items of `menu`, if one has
    /// the focus.
    pub(super) fn focused_item(menu: NodeRef<leptos::html::Div>) -> Option<u32> {
        #[cfg(target_arch = "wasm32")]
        {
            let active = document().active_element()?;
            let at = items(menu)
                .iter()
                .position(|item| AsRef::<web_sys::Element>::as_ref(item) == &active)?;
            u32::try_from(at).ok()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _: NodeRef<leptos::html::Div> = menu;
            None
        }
    }

    /// Moves the focus to item `at` of `menu`.
    pub(super) fn focus_item(menu: NodeRef<leptos::html::Div>, at: u32) {
        #[cfg(target_arch = "wasm32")]
        {
            let item = usize::try_from(at)
                .ok()
                .and_then(|at| items(menu).into_iter().nth(at));
            if let Some(item) = item
                && item.focus().is_err()
            {
                leptos::logging::warn!("the browser refused the focus on a menu item");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (NodeRef<leptos::html::Div>, u32) = (menu, at);
    }

    /// Opens `menu`, the focus on its last item when `last` and on its
    /// first otherwise.
    pub(super) fn open_at(menu: NodeRef<leptos::html::Div>, last: bool) {
        #[cfg(target_arch = "wasm32")]
        {
            let Some(element) = menu.get_untracked() else {
                return;
            };
            if !is_open(menu) && element.show_popover().is_err() {
                leptos::logging::warn!("the browser refused to open the menu");
                return;
            }
            let at = if last {
                item_count(menu).saturating_sub(1)
            } else {
                0
            };
            focus_item(menu, at);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (NodeRef<leptos::html::Div>, bool) = (menu, last);
    }

    /// Closes `menu` if it is open.
    pub(super) fn close(menu: NodeRef<leptos::html::Div>) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(element) = menu.get_untracked()
                && is_open(menu)
                && element.hide_popover().is_err()
            {
                leptos::logging::warn!("the browser refused to close the menu");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: NodeRef<leptos::html::Div> = menu;
    }

    /// Returns the focus to `trigger` when the closed `menu` held it, or
    /// when it fell to the page's body.
    pub(super) fn return_focus(
        menu: NodeRef<leptos::html::Div>,
        trigger: NodeRef<leptos::html::Button>,
    ) {
        #[cfg(target_arch = "wasm32")]
        {
            let (Some(element), Some(trigger)) = (menu.get_untracked(), trigger.get_untracked())
            else {
                return;
            };
            let held = document().active_element().is_none_or(|active| {
                element.contains(Some(&active))
                    || document()
                        .body()
                        .is_some_and(|body| AsRef::<web_sys::Element>::as_ref(&body) == &active)
            });
            if held && trigger.focus().is_err() {
                leptos::logging::warn!("the browser refused the focus on the menu button");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (NodeRef<leptos::html::Div>, NodeRef<leptos::html::Button>) = (menu, trigger);
    }
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
