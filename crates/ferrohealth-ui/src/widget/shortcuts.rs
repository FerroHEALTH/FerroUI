// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keyboard shortcut legend: the current page's shortcuts in a dialog
//! over it, with the tests a console's own key listener asks first (R1).
//!
//! The library binds no key. A console listens for its keys in its own
//! island and opens the legend from its signal, usually on `?`; the legend
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
    use super::takes_text;

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
