// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The confirmation dialog: an HTML popover with a title, a sentence, a
//! cancel and the console's own confirming control.
//!
//! The trigger opens it and Cancel closes it with no script, the browser
//! closes it on Escape and returns the focus to the trigger (HTML, the
//! `popover` attribute), so it works before the page hydrates. The
//! confirming control is the console's: a submit button of a form that
//! posts, or a button whose listener acts and closes the dialog.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// What a confirmation dialog asks.
#[derive(Debug, Clone, Copy)]
pub struct Confirmation {
    /// The dialog's id, which its trigger opens.
    pub id: &'static str,
    /// The question, such as "Delete the stored query?".
    pub title: &'static str,
    /// The sentence that says what confirming does.
    pub message: &'static str,
}

/// Returns the button that opens the dialog `id`, of `variant`, with
/// `glyph` beside `label`.
#[must_use]
pub fn confirm_trigger(
    id: &'static str,
    variant: Variant,
    glyph: Glyph,
    label: &'static str,
) -> impl IntoView + use<> {
    view! {
        <button type="button" class=variant.class() popovertarget=id aria-controls=id>
            {icon(glyph, "16")}
            {label}
        </button>
    }
}

/// Returns the dialog asking `confirmation`, with Cancel and `confirm`.
#[must_use]
pub fn confirm_dialog(confirmation: Confirmation, confirm: impl IntoView) -> impl IntoView {
    frame(confirmation, false, confirm)
}

/// Returns the dialog asking `confirmation` about an action that cannot be
/// undone, such as a delete.
///
/// The bad tone's glyph sits beside its question, over Cancel and
/// `confirm`, a [`Variant::Destructive`] button: solid, so the destructive
/// step is unmistakable.
#[must_use]
pub fn danger_dialog(confirmation: Confirmation, confirm: impl IntoView) -> impl IntoView {
    frame(confirmation, true, confirm)
}

/// Returns the dialog asking `confirmation`, in the danger form when
/// `danger`, with Cancel and `confirm`.
fn frame(confirmation: Confirmation, danger: bool, confirm: impl IntoView) -> impl IntoView {
    let title_id = format!("{}-title", confirmation.id);
    let labelled_by = title_id.clone();
    let class = if danger {
        "fui-dialog fui-dialog-danger"
    } else {
        "fui-dialog"
    };
    view! {
        <div
            id=confirmation.id
            popover="auto"
            role="dialog"
            aria-labelledby=labelled_by
            class=class
        >
            <h2 id=title_id class="fui-dialog-title">
                {danger.then(|| icon(Glyph::TRIANGLE_ALERT, "18"))}
                {confirmation.title}
            </h2>
            <p>{confirmation.message}</p>
            <div class="fui-dialog-actions">
                <button
                    type="button"
                    class=Variant::Secondary.class()
                    popovertarget=confirmation.id
                    popovertargetaction="hide"
                >
                    {icon(Glyph::X, "16")}
                    "Cancel"
                </button>
                {confirm}
            </div>
        </div>
    }
}
