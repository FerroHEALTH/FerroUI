// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The modal dialog a signal opens: a confirmation or a form whose screen's
//! state decides when it shows, such as a row's Delete that names the row.
//!
//! It is an HTML `dialog` shown with `showModal()`, so it sits in the top
//! layer over a dimmed page, the page behind is inert, the focus moves in,
//! and the browser returns it to the control that had it when the dialog
//! closes (HTML Living Standard, the `dialog` element). Escape and a press
//! on the dim around it run `on_dismiss`, and only they do: closing it from
//! the signal runs nothing. The server draws it closed, so hydration
//! matches, and the browser code runs in an effect and in handlers alone
//! (WAI-ARIA Authoring Practices, the dialog (modal) pattern).

use leptos::prelude::*;

use crate::icon::{Glyph, icon};

/// How wide a modal dialog is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Width {
    /// A confirmation's width, 28 rem.
    #[default]
    Narrow,
    /// A form's width, 48 rem, for a text area or a table.
    Wide,
}

/// What a modal dialog shows.
#[derive(Debug)]
pub struct Modal {
    /// The dialog's id; its heading is `id` and `-title`.
    pub id: &'static str,
    /// The heading.
    pub title: Signal<String>,
    /// Whether it is the danger form, the bad tone's glyph beside the
    /// heading, for an action that cannot be undone.
    pub danger: bool,
    /// How wide it is.
    pub width: Width,
    /// The body: the message, or the form's fields.
    pub body: AnyView,
    /// The actions, the confirming control last: the console's own.
    pub actions: AnyView,
}

/// Returns the modal dialog `modal`, shown while `open` reads true, with
/// `on_dismiss` run on Escape, a press outside it and its Cancel.
#[must_use]
pub fn modal_dialog(modal: Modal, open: Signal<bool>, on_dismiss: Callback<()>) -> AnyView {
    let Modal {
        id,
        title,
        danger,
        width,
        body,
        actions,
    } = modal;
    let node = NodeRef::<leptos::html::Dialog>::new();
    Effect::new(move |_| sync(node, open.get()));
    let title_id = format!("{id}-title");
    let labelled_by = title_id.clone();
    let mut class = String::from("fui-dialog fui-modal");
    if danger {
        class.push_str(" fui-dialog-danger");
    }
    if width == Width::Wide {
        class.push_str(" fui-dialog-wide");
    }
    view! {
        <dialog
            id=id
            class=class
            aria-labelledby=labelled_by
            node_ref=node
            on:cancel=move |event: leptos::ev::Event| {
                event.prevent_default();
                on_dismiss.run(());
            }
            on:click=move |event| {
                if pressed_outside(node, &event) {
                    on_dismiss.run(());
                }
            }
        >
            <h2 id=title_id class="fui-dialog-title">
                {danger.then(|| icon(Glyph::TRIANGLE_ALERT, "18"))}
                {move || title.get()}
            </h2>
            <div class="fui-modal-body">{body}</div>
            <div class="fui-dialog-actions">
                <button
                    type="button"
                    class="fui-button fui-button-secondary"
                    on:click=move |_| on_dismiss.run(())
                >
                    {icon(Glyph::X, "16")}
                    "Cancel"
                </button>
                {actions}
            </div>
        </dialog>
    }
    .into_any()
}

/// Shows the dialog `node` as a modal while `open`, and closes it otherwise.
fn sync(node: NodeRef<leptos::html::Dialog>, open: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(dialog) = node.get() else {
            return;
        };
        if open && !dialog.open() {
            if dialog.show_modal().is_err() {
                leptos::logging::warn!("the browser refused to show the dialog");
            }
        } else if !open && dialog.open() {
            dialog.close();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (NodeRef<leptos::html::Dialog>, bool) = (node, open);
}

/// Whether `event`, a press on the dialog `node`, landed on the dim around
/// it rather than on its box: the dialog element is the target only there.
fn pressed_outside(node: NodeRef<leptos::html::Dialog>, event: &leptos::ev::MouseEvent) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let Some(dialog) = node.get_untracked() else {
            return false;
        };
        let target = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok());
        let on_dialog = target.is_some_and(|target| {
            let dialog: &web_sys::Element = dialog.as_ref();
            target == *dialog
        });
        if !on_dialog {
            return false;
        }
        let rect = dialog.get_bounding_client_rect();
        let (x, y) = (f64::from(event.client_x()), f64::from(event.client_y()));
        x < rect.left() || x > rect.right() || y < rect.top() || y > rect.bottom()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _: (NodeRef<leptos::html::Dialog>, &leptos::ev::MouseEvent) = (node, event);
        false
    }
}
