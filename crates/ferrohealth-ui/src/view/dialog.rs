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
//!
//! A confirmation that needs the reader's words, such as the required
//! Reason a removal keeps with the revision, is [`danger_form_dialog`]: the
//! console's fields sit between the sentence and the actions, and the
//! whole dialog is the one form its confirm posts, so the fields post with
//! it and the browser holds a required field before the post. A row
//! menu's destructive item opens it by its id, one dialog per row. A
//! console whose actions are server functions draws
//! [`danger_action_dialog`] instead around its own `<ActionForm>`, so the
//! confirmation runs in the page once it has hydrated and posts as a plain
//! form before.
//!
//! The informational dialog ([`info_dialog`]) is the same frame for a page
//! of explanation, such as "How configuration works": the info glyph beside
//! its title, a few paragraphs, and Close as its only action, which takes
//! the focus as it opens.

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

/// The form a confirmation with fields posts.
#[derive(Debug)]
pub struct ConfirmForm {
    /// Where the form posts.
    pub action: String,
    /// The console's fields, drawn between the sentence and the actions,
    /// such as a required Reason and the hidden inputs that name the row.
    pub fields: AnyView,
}

/// Returns the dialog asking `confirmation`, with Cancel and `confirm`.
#[must_use]
pub fn confirm_dialog(confirmation: Confirmation, confirm: impl IntoView) -> impl IntoView {
    frame(confirmation, false, None, confirm)
}

/// Returns the dialog asking `confirmation` about an action that cannot be
/// undone, such as a delete.
///
/// The bad tone's glyph sits beside its question, over Cancel and
/// `confirm`, a [`Variant::Destructive`] button: solid, so the destructive
/// step is unmistakable.
#[must_use]
pub fn danger_dialog(confirmation: Confirmation, confirm: impl IntoView) -> impl IntoView {
    frame(confirmation, true, None, confirm)
}

/// Returns the dialog asking `confirmation` about an action that cannot be
/// undone and needs the reader's words, such as a removal with a required
/// Reason.
///
/// It is [`danger_dialog`] with `form`'s fields between its sentence and
/// its actions, inside one `POST` form to `form.action`, so `confirm`, a
/// submit button, posts the fields with it. Its trigger, such as a row
/// menu's Remove… ([`crate::widget::action_menu::Act::Dialog`]), opens it
/// with no script, and the focus moves to its first field.
///
/// # Examples
///
/// ```
/// use ferrohealth_ui::view::dialog::{Confirmation, ConfirmForm, danger_form_dialog};
/// use leptos::prelude::*;
///
/// let question = Confirmation {
///     id: format!("remove-{}", "node-b").into(),
///     title: "Remove node-b?".into(),
///     message: "Its endpoints go with it.".into(),
/// };
/// let reason = view! { <textarea name="reason" required></textarea> };
/// let form = ConfirmForm {
///     action: String::from("/members/node-b/remove"),
///     fields: reason.into_any(),
/// };
/// let remove = view! { <button type="submit">"Remove"</button> };
/// let _dialog = danger_form_dialog(question, form, remove);
/// ```
#[must_use]
pub fn danger_form_dialog(
    confirmation: Confirmation,
    form: ConfirmForm,
    confirm: impl IntoView,
) -> impl IntoView {
    frame(confirmation, true, Some(form), confirm)
}

/// The class of a confirmation's form, which a console gives the
/// `<ActionForm>` it draws for [`danger_action_dialog`].
pub const DIALOG_FORM: &str = "fui-dialog-form";

/// Returns the dialog asking `confirmation` about an action that cannot be
/// undone and needs the reader's words, inside the form `form` draws: a
/// console's `<ActionForm>`.
///
/// It is [`danger_form_dialog`] for a console whose actions are server
/// functions (`docs/consoles.md`): `form` receives the dialog's parts, its
/// title, its sentence, `fields`, Cancel and `confirm`, and draws its
/// `<ActionForm>` around them with the class [`DIALOG_FORM`]. Once the page
/// has hydrated, `confirm`, a submit button, runs the action in the page
/// with the fields as its arguments, and the dialog closes each time
/// `done` turns true, such as when the action's value is a success. Before
/// that, and with no WebAssembly, the form posts to the server function's
/// address as a plain form, as `<ActionForm>` draws it (leptos 0.8
/// `ActionForm`); give the fields [`crate::widget::action::return_field`]
/// or [`crate::widget::action::answer_field`] for that post. The library
/// holds no action itself (`.claude/rules/leptos-ui.md` §0). No
/// specification governs this: our own design (#500).
///
/// # Examples
///
/// ```
/// use ferrohealth_ui::view::dialog::{Confirmation, DIALOG_FORM, danger_action_dialog};
/// use leptos::prelude::*;
///
/// let html = Owner::new().with(|| {
///     let question = Confirmation {
///         id: "remove-node-b".into(),
///         title: "Remove node-b?".into(),
///         message: "Its endpoints go with it.".into(),
///     };
///     let fields = view! {
///         <input type="hidden" name="member" value="node-b" />
///         <textarea name="reason" required></textarea>
///     };
///     // A console draws `<ActionForm action=remove attr:class=DIALOG_FORM>` here.
///     let form = |parts: AnyView| {
///         view! { <form method="post" action="/api/remove_member" class=DIALOG_FORM>{parts}</form> }
///             .into_any()
///     };
///     let confirm = view! { <button type="submit">"Remove"</button> };
///     danger_action_dialog(question, fields.into_any(), form, Signal::stored(false), confirm)
///         .to_html()
/// });
/// assert!(html.contains(r#"action="/api/remove_member""#));
/// assert!(html.contains(r#"name="reason""#));
/// ```
#[must_use]
pub fn danger_action_dialog(
    confirmation: Confirmation,
    fields: AnyView,
    form: impl FnOnce(AnyView) -> AnyView,
    done: Signal<bool>,
    confirm: impl IntoView,
) -> AnyView {
    let node = NodeRef::<leptos::html::Dialog>::new();
    close_when(done, node);
    dialog(confirmation, true, Some(fields), confirm, Some(node), form)
}

/// Closes the dialog `node` each time `done` turns true, once the page has
/// hydrated.
fn close_when(done: Signal<bool>, node: NodeRef<leptos::html::Dialog>) {
    #[cfg(target_arch = "wasm32")]
    Effect::watch(
        move || done.get(),
        move |done, _, _| {
            if let (true, Some(dialog)) = (*done, node.get_untracked()) {
                dialog.close();
            }
        },
        false,
    );
    #[cfg(not(target_arch = "wasm32"))]
    let _: (Signal<bool>, NodeRef<leptos::html::Dialog>) = (done, node);
}

/// Returns the dialog asking `confirmation`, in the danger form when
/// `danger`, with `form`'s fields inside the form it posts when it has
/// one, then Cancel and `confirm`.
fn frame(
    confirmation: Confirmation,
    danger: bool,
    form: Option<ConfirmForm>,
    confirm: impl IntoView,
) -> impl IntoView {
    match form {
        Some(ConfirmForm { action, fields }) => {
            let wrap = move |parts: AnyView| {
                view! {
                    <form method="post" action=action class=DIALOG_FORM>
                        {parts}
                    </form>
                }
                .into_any()
            };
            dialog(confirmation, danger, Some(fields), confirm, None, wrap)
        }
        None => dialog(confirmation, danger, None, confirm, None, |parts| parts),
    }
}

/// Returns the dialog asking `confirmation`, in the danger form when
/// `danger`: its title, its sentence, `fields`, Cancel and `confirm`, the
/// parts inside what `wrap` draws around them, held in `node` when given.
fn dialog(
    confirmation: Confirmation,
    danger: bool,
    fields: Option<AnyView>,
    confirm: impl IntoView,
    node: Option<NodeRef<leptos::html::Dialog>>,
    wrap: impl FnOnce(AnyView) -> AnyView,
) -> AnyView {
    let Confirmation { id, title, message } = confirmation;
    let title_id = format!("{id}-title");
    let labelled_by = title_id.clone();
    let class = if danger {
        "fui-dialog fui-dialog-danger"
    } else {
        "fui-dialog"
    };
    let parts = view! {
        <h2 id=title_id class="fui-dialog-title">
            {danger.then(|| icon(Glyph::TRIANGLE_ALERT, "18"))}
            {title}
        </h2>
        <p>{message}</p>
        {fields}
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
    };
    let inside = wrap(parts.into_any());
    match node {
        Some(node) => view! {
            <dialog
                id=id.clone()
                closedby="any"
                aria-labelledby=labelled_by
                class=class
                node_ref=node
            >
                {inside}
            </dialog>
        }
        .into_any(),
        None => view! {
            <dialog id=id.clone() closedby="any" aria-labelledby=labelled_by class=class>
                {inside}
            </dialog>
        }
        .into_any(),
    }
}

/// What an informational dialog explains.
#[derive(Debug, Clone)]
pub struct Explanation {
    /// The dialog's id, which its trigger opens with the `show-modal`
    /// command.
    pub id: Cow<'static, str>,
    /// What it explains, such as "How configuration works".
    pub title: Word,
}

/// Returns the dialog explaining `explanation` with `body`, a few
/// paragraphs, and Close as its one action.
///
/// It is 48 rem wide, so two short columns of prose fit side by side, and
/// Close takes the focus as it opens: the dialog asks nothing (WAI-ARIA
/// Authoring Practices, the dialog (modal) pattern, initial focus).
#[must_use]
pub fn info_dialog(explanation: Explanation, body: impl IntoView) -> impl IntoView {
    explain(explanation, body, true)
}

/// Returns the informational dialog explaining `explanation` with `body`,
/// 48 rem wide when `wide` and at the dialog's own width otherwise, such
/// as a flow's short edge rule ([`crate::view::flow::rule_dialog`]).
pub(crate) fn explain(explanation: Explanation, body: impl IntoView, wide: bool) -> impl IntoView {
    let Explanation { id, title } = explanation;
    let title_id = format!("{id}-title");
    let labelled_by = title_id.clone();
    let class = if wide {
        "fui-dialog fui-dialog-wide"
    } else {
        "fui-dialog"
    };
    view! {
        <dialog id=id.clone() closedby="any" aria-labelledby=labelled_by class=class>
            <h2 id=title_id class="fui-dialog-title">
                {icon(Glyph::INFO, "18")}
                {title}
            </h2>
            <div class="fui-modal-body">{body}</div>
            <div class="fui-dialog-actions">
                <button
                    type="button"
                    class=Variant::Secondary.class()
                    command="close"
                    commandfor=id.clone()
                    autofocus=true
                >
                    {icon(Glyph::X, "16")}
                    {tr(Text::Close)}
                </button>
            </div>
        </dialog>
    }
}
