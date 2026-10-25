// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keyboard shortcut legend: the current page's shortcuts in a dialog
//! over it, with the tests a console's own key listener asks first (R1).
//!
//! [`legend_on_key`] (the island `ShortcutLegend`) opens the legend on `?`
//! pressed outside a field and from a control marked [`OPENS_LEGEND`], such
//! as the user menu's Keyboard shortcuts; [`tab_keys`] follows the binder's
//! link tabs on the digit keys. A console with its own keys listens in its
//! own island and opens the legend from its signal; the legend
//! is the modal dialog of [`crate::widget::dialog`], an HTML `dialog`
//! shown over the page it was opened from, which returns the focus there
//! when it closes (WAI-ARIA Authoring Practices, the dialog (modal)
//! pattern). It lists each group of shortcuts in a keys-and-words grid, the
//! keys as printed on the keyboard ([`crate::view::kbd`]).
//!
//! A phone has no keyboard, so below 40 rem no shortcut shows or acts
//! (family rule 6): [`phone_layout`] tells a console's listener so, and a
//! control that opens the legend is hidden there. A single-key shortcut
//! never acts while the person types: [`typing`] tells a listener so. No
//! specification governs the component: our own design.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, Word, tr};
use crate::view::kbd::{ShortcutGroup, shortcut_keys};
use crate::widget::dialog::{Modal, ModalWords, Width, modal_dialog};

/// The id of the legend's dialog; its heading is `shortcuts-title`.
pub const LEGEND_ID: &str = "shortcuts";

/// The attribute of a control a click on which opens the legend in place of
/// following it, such as the user menu's Keyboard shortcuts link; with no
/// WebAssembly the link is followed.
pub const OPENS_LEGEND: &str = "data-ferroui-shortcuts";

/// The attribute of the binder's tabs that the digit keys follow.
pub const TAB_KEYS: &str = "data-ferroui-tab-keys";

/// The link tab the digit `key` names among `count` tabs: `1` the first,
/// `9` the ninth; `None` for another key or a tab the row does not have.
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

/// Returns the legend of `groups` in the words `words`, opened from the
/// keyboard and from a marked control.
///
/// `?` pressed outside a field (not chorded) opens it, and so does a click
/// on a control marked [`OPENS_LEGEND`], never in the phone layout; the
/// dialog returns the focus where it was when it closes. A console draws it
/// through the island `ShortcutLegend`, once on a page.
#[must_use]
pub fn legend_on_key(groups: Vec<ShortcutGroup>, words: LegendWords) -> AnyView {
    let open = RwSignal::new(false);
    Effect::new(move |_| browser::listen(open));
    shortcut_legend(
        groups,
        words,
        open.into(),
        Callback::new(move |()| {
            let _: Option<bool> = open.try_set(false);
        }),
    )
}

/// Follows the binder's link tabs on the keys `1` to `9`.
///
/// A key acts pressed outside a field (not chorded), never in the phone
/// layout or while a dialog is open. A binder draws it beside its tabs; of
/// two binders on one page the first answers.
#[must_use]
pub fn tab_keys() -> AnyView {
    Effect::new(move |_| browser::follow_tabs());
    ().into_any()
}

/// Whether a key event `event` belongs to a page's single-key shortcut: not
/// taken by another listener, not chorded, not typed into a field, not in
/// the phone layout.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(dead_code, reason = "the browser half alone listens for keys")
)]
fn shortcut(event: &leptos::ev::KeyboardEvent) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        !event.default_prevented() && !chorded(event) && !typing(event) && !phone_layout()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _: &leptos::ev::KeyboardEvent = event;
        false
    }
}

mod browser {
    use leptos::prelude::*;

    #[cfg(target_arch = "wasm32")]
    use super::{OPENS_LEGEND, TAB_KEYS, phone_layout, shortcut, tab_at};

    /// Opens the legend `open` on `?` and on a click of a control marked
    /// to open it.
    pub(super) fn listen(open: RwSignal<bool>) {
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::wasm_bindgen::JsCast as _;
            let keys = window_event_listener(leptos::ev::keydown, move |event| {
                if event.key() != "?" || !shortcut(&event) {
                    return;
                }
                event.prevent_default();
                let _: Option<bool> = open.try_set(true);
            });
            let clicks = window_event_listener(leptos::ev::click, move |event| {
                let marked = event
                    .target()
                    .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    .and_then(|target| target.closest(&format!("[{OPENS_LEGEND}]")).ok().flatten());
                if marked.is_none() || phone_layout() {
                    return;
                }
                event.prevent_default();
                let _: Option<bool> = open.try_set(true);
            });
            on_cleanup(move || {
                keys.remove();
                clicks.remove();
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: RwSignal<bool> = open;
    }

    /// Follows the first binder's nth link tab on the digit `n`.
    pub(super) fn follow_tabs() {
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::wasm_bindgen::JsCast as _;
            let keys = window_event_listener(leptos::ev::keydown, move |event| {
                if !shortcut(&event) {
                    return;
                }
                let document = document();
                if document
                    .query_selector("dialog[open]")
                    .ok()
                    .flatten()
                    .is_some()
                {
                    return;
                }
                let Ok(links) = document.query_selector_all(&format!("nav[{TAB_KEYS}] a[href]"))
                else {
                    return;
                };
                let first = document
                    .query_selector(&format!("nav[{TAB_KEYS}]"))
                    .ok()
                    .flatten();
                let tabs: Vec<web_sys::HtmlAnchorElement> = (0..links.length())
                    .filter_map(|index| links.item(index))
                    .filter_map(|node| node.dyn_into::<web_sys::HtmlAnchorElement>().ok())
                    .filter(|link| {
                        link.closest(&format!("nav[{TAB_KEYS}]")).ok().flatten() == first
                    })
                    .collect();
                let Some(tab) = tab_at(&event.key(), tabs.len()).and_then(|at| tabs.get(at)) else {
                    return;
                };
                event.prevent_default();
                if tab.get_attribute("aria-current").as_deref() != Some("page") {
                    crate::widget::submit::follow(&tab.href());
                }
            });
            on_cleanup(move || keys.remove());
        }
    }
}

/// The words the legend draws itself, resolved on the server in the page's
/// locale, so the island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegendWords {
    /// The heading, "Keyboard shortcuts".
    pub title: Word,
    /// The sentence under the heading.
    pub intro: Word,
    /// The word between the alternative keys of one shortcut.
    pub or: Word,
    /// The button that dismisses the legend, "Close": it only shows
    /// information, so there is nothing to cancel.
    pub close: Word,
}

impl LegendWords {
    /// The legend's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            title: tr(Text::KeyboardShortcuts),
            intro: tr(Text::ShortcutsIntro),
            or: tr(Text::KeysOr),
            close: tr(Text::Close),
        }
    }
}

/// Returns the legend of `groups`, in the words `words`, shown while `open`
/// reads true, with `on_dismiss` run on Escape, a press outside it and its
/// close button.
#[must_use]
pub fn shortcut_legend(
    groups: Vec<ShortcutGroup>,
    words: LegendWords,
    open: Signal<bool>,
    on_dismiss: Callback<()>,
) -> AnyView {
    let LegendWords {
        title,
        intro,
        or,
        close,
    } = words;
    let groups = groups
        .into_iter()
        .zip(1_u32..)
        .map(|(group, place)| {
            let heading = format!("{LEGEND_ID}-group-{place}");
            let labelled = heading.clone();
            let rows = group
                .shortcuts
                .into_iter()
                .map(|shortcut| {
                    view! {
                        <dt>{shortcut_keys(&shortcut.keys, &or)}</dt>
                        <dd>{shortcut.does}</dd>
                    }
                })
                .collect_view();
            view! {
                <section class="fui-shortcuts-group" aria-labelledby=labelled>
                    <h3 id=heading>{group.title}</h3>
                    <dl class="fui-shortcuts-list">{rows}</dl>
                </section>
            }
        })
        .collect_view();
    modal_dialog(
        Modal {
            id: LEGEND_ID,
            title: Signal::stored(title),
            danger: false,
            width: Width::Wide,
            body: view! {
                <p class="fui-muted">{intro}</p>
                <div class="fui-shortcuts">{groups}</div>
            }
            .into_any(),
            actions: ().into_any(),
            words: ModalWords { cancel: close },
        },
        open,
        on_dismiss,
    )
}

/// Whether an element of the tag `tag`, an input's `kind` when it is one,
/// takes typed text, where a single-key shortcut must not act.
#[cfg_attr(
    all(not(target_arch = "wasm32"), not(test)),
    expect(
        dead_code,
        reason = "the browser half alone reads the element a key reached"
    )
)]
fn takes_text(tag: &str, kind: &str, content_editable: bool) -> bool {
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

/// Whether the key event `event` reached an element that takes typed text
/// (a field, a text area, a select, an editable region), where a single-key
/// shortcut must not act.
#[must_use]
pub fn typing(event: &leptos::ev::KeyboardEvent) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let Some(element) = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlElement>().ok())
        else {
            return false;
        };
        let tag = element.tag_name().to_ascii_lowercase();
        let kind = element
            .get_attribute("type")
            .unwrap_or_default()
            .to_ascii_lowercase();
        takes_text(&tag, &kind, element.is_content_editable())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _: &leptos::ev::KeyboardEvent = event;
        false
    }
}

/// Whether the key event `event` holds Control, Meta or Alt, so it belongs
/// to the browser or the system rather than to a page's shortcut.
#[must_use]
pub fn chorded(event: &leptos::ev::KeyboardEvent) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        event.ctrl_key() || event.meta_key() || event.alt_key()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _: &leptos::ev::KeyboardEvent = event;
        false
    }
}

/// Whether the page is in its phone layout, below 40 rem, where no shortcut
/// shows or acts (family rule 6).
#[must_use]
pub fn phone_layout() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        window()
            .match_media("(width < 40rem)")
            .ok()
            .flatten()
            .is_some_and(|query| query.matches())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
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
