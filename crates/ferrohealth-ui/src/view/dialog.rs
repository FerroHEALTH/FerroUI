// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The confirmation dialog: an HTML `dialog` with a title, a sentence, a
//! cancel and the console's own confirming control.
//!
//! Its trigger is an action whose act is
//! [`crate::widget::action_menu::Act::Dialog`]: a title row's primary
//! ([`crate::view::action::action_button`]) or, for a destructive action,
//! an item of the More menu, last, never a danger button beside the
//! primary (family rule 7). The trigger opens it as a modal and Cancel
//! closes it with no script, through the `show-modal` and `close` commands:
//! the page behind is inert, the focus moves to its first control, Escape
//! and a press outside close it, and the browser returns the focus to the
//! trigger (HTML Living Standard, the `dialog` element and the `command`
//! attribute; WAI-ARIA Authoring Practices, the dialog (modal) pattern), so
//! it works before the page hydrates. The confirming control is the
//! console's: a submit button of a form that posts, or a button whose
//! listener acts and closes the dialog.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::i18n::{Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// What a confirmation dialog asks.
#[derive(Debug, Clone)]
pub struct Confirmation {
    /// The dialog's id, which its trigger opens: a static id or one made
    /// at run time, such as one per table row.
    pub id: Cow<'static, str>,
    /// The question, such as "Delete the stored query?".
    pub title: Word,
    /// The sentence that says what confirming does.
    pub message: Word,
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
    let Confirmation { id, title, message } = confirmation;
    let title_id = format!("{id}-title");
    let labelled_by = title_id.clone();
    let class = if danger {
        "fui-dialog fui-dialog-danger"
    } else {
        "fui-dialog"
    };
    view! {
        <dialog id=id.clone() closedby="any" aria-labelledby=labelled_by class=class>
            <h2 id=title_id class="fui-dialog-title">
                {danger.then(|| icon(Glyph::TRIANGLE_ALERT, "18"))}
                {title}
            </h2>
            <p>{message}</p>
            <div class="fui-dialog-actions">
                <button
                    type="button"
                    class=Variant::Secondary.class()
                    command="close"
                    commandfor=id.clone()
                >
                    {icon(Glyph::X, "16")}
                    {tr(Text::Cancel)}
                </button>
                {confirm}
            </div>
        </dialog>
    }
}
