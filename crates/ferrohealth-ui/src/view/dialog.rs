// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The confirmation dialog: an HTML `dialog` with a title, a sentence, a
//! cancel and the console's own confirming control.
//!
//! Its trigger is an action whose act is
//! [`crate::view::action_menu::Act::Dialog`]: a title row's primary
//! ([`crate::view::action::action_button`]) or, for a destructive action,
//! an item of the More menu, last, never a danger button beside the
//! primary (family rule 7). The trigger opens it as a modal and Cancel
//! closes it with no script, through the `show-modal` and `close` commands:
//! the page behind is inert, the focus moves to its first control, Escape
//! and a press outside close it, and the browser returns the focus to the
//! trigger (HTML Living Standard, the `dialog` element and the `command`
//! attribute; WAI-ARIA Authoring Practices, the dialog (modal) pattern).
//! The confirming control is the console's: a submit button of a form that
//! posts.
//!
//! A confirmation that needs the reader's words, such as the required
//! Reason a removal keeps with the revision, is [`danger_form_dialog`]: the
//! console's fields sit between the sentence and the actions, and the
//! whole dialog is the one form its confirm posts, so the fields post with
//! it and the browser holds a required field before the post. A row
//! menu's destructive item opens it by its id, one dialog per row.
//!
//! The informational dialog ([`info_dialog`]) is the same frame for a page
//! of explanation, such as "How configuration works": the info glyph beside
//! its title, a few paragraphs, and Close as its only action, which takes
//! the focus as it opens. The modal a page draws open ([`modal_dialog`]),
//! such as a form a post refused, opens as a modal once the `dialog` module
//! runs and shows in place before that.

use std::borrow::Cow;

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr};
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
    pub fields: Html,
}

/// The class of a confirmation's form.
pub const DIALOG_FORM: &str = "fui-dialog-form";

#[derive(Template)]
#[template(path = "dialog/dialog.html", block = "confirm")]
struct ConfirmView {
    id: Cow<'static, str>,
    class: &'static str,
    action: Option<String>,
    form_class: &'static str,
    glyph: Html,
    title: Word,
    message: Word,
    fields: Html,
    secondary: &'static str,
    x: Html,
    cancel: Word,
    confirm: Html,
}

/// Returns the dialog asking `confirmation`, with Cancel and `confirm`.
#[must_use]
pub fn confirm_dialog(locale: &Locale, confirmation: Confirmation, confirm: Html) -> Html {
    frame(locale, confirmation, false, None, confirm)
}

/// Returns the dialog asking `confirmation` about an action that cannot be
/// undone, such as a delete.
///
/// The bad tone's glyph sits beside its question, over Cancel and
/// `confirm`, a [`Variant::Destructive`] button: solid, so the destructive
/// step is unmistakable.
#[must_use]
pub fn danger_dialog(locale: &Locale, confirmation: Confirmation, confirm: Html) -> Html {
    frame(locale, confirmation, true, None, confirm)
}

/// Returns the dialog asking `confirmation` about an action that cannot be
/// undone and needs the reader's words, such as a removal with a required
/// Reason.
///
/// It is [`danger_dialog`] with `form`'s fields between its sentence and
/// its actions, inside one `POST` form to `form.action`, so `confirm`, a
/// submit button, posts the fields with it. Its trigger, such as a row
/// menu's Remove… ([`crate::view::action_menu::Act::Dialog`]), opens it
/// with no script, and the focus moves to its first field.
///
/// # Examples
///
/// ```
/// use ferrohealth_ui::html::Html;
/// use ferrohealth_ui::i18n::Locale;
/// use ferrohealth_ui::view::dialog::{Confirmation, ConfirmForm, danger_form_dialog};
///
/// let question = Confirmation {
///     id: format!("remove-{}", "node-b").into(),
///     title: "Remove node-b?".into(),
///     message: "Its endpoints go with it.".into(),
/// };
/// let form = ConfirmForm {
///     action: String::from("/members/node-b/remove"),
///     fields: Html::raw(String::from(r#"<textarea name="reason" required></textarea>"#)),
/// };
/// let remove = Html::raw(String::from(r#"<button type="submit">Remove</button>"#));
/// let dialog = danger_form_dialog(&Locale::english(), question, form, remove);
/// assert!(dialog.as_str().contains(r#"action="/members/node-b/remove""#));
/// ```
#[must_use]
pub fn danger_form_dialog(
    locale: &Locale,
    confirmation: Confirmation,
    form: ConfirmForm,
    confirm: Html,
) -> Html {
    frame(locale, confirmation, true, Some(form), confirm)
}

/// Returns the dialog asking `confirmation`, in the danger form when
/// `danger`, with `form`'s fields inside the form it posts when it has
/// one, then Cancel and `confirm`.
fn frame(
    locale: &Locale,
    confirmation: Confirmation,
    danger: bool,
    form: Option<ConfirmForm>,
    confirm: Html,
) -> Html {
    let Confirmation { id, title, message } = confirmation;
    let (action, fields) = form.map_or((None, Html::empty()), |form| {
        (Some(form.action), form.fields)
    });
    ConfirmView {
        id,
        class: if danger {
            "fui-dialog fui-dialog-danger"
        } else {
            "fui-dialog"
        },
        action,
        form_class: DIALOG_FORM,
        glyph: if danger {
            icon(Glyph::TRIANGLE_ALERT, "18")
        } else {
            Html::empty()
        },
        title,
        message,
        fields,
        secondary: Variant::Secondary.class(),
        x: icon(Glyph::X, "16"),
        cancel: tr(locale, Text::Cancel),
        confirm,
    }
    .html()
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

#[derive(Template)]
#[template(path = "dialog/dialog.html", block = "explain")]
struct ExplainView {
    id: Cow<'static, str>,
    class: &'static str,
    glyph: Html,
    title: Word,
    body: Html,
    secondary: &'static str,
    x: Html,
    close: Word,
}

/// Returns the dialog explaining `explanation` with `body`, a few
/// paragraphs, and Close as its one action.
///
/// It is 48 rem wide, so two short columns of prose fit side by side, and
/// Close takes the focus as it opens: the dialog asks nothing (WAI-ARIA
/// Authoring Practices, the dialog (modal) pattern, initial focus).
#[must_use]
pub fn info_dialog(locale: &Locale, explanation: Explanation, body: Html) -> Html {
    explain(locale, explanation, body, true)
}

/// Returns the informational dialog explaining `explanation` with `body`,
/// 48 rem wide when `wide` and at the dialog's own width otherwise, such
/// as a flow's short edge rule ([`crate::view::flow::rule_dialog`]).
pub(crate) fn explain(locale: &Locale, explanation: Explanation, body: Html, wide: bool) -> Html {
    let Explanation { id, title } = explanation;
    ExplainView {
        id,
        class: if wide {
            "fui-dialog fui-dialog-wide"
        } else {
            "fui-dialog"
        },
        glyph: icon(Glyph::INFO, "18"),
        title,
        body,
        secondary: Variant::Secondary.class(),
        x: icon(Glyph::X, "16"),
        close: tr(locale, Text::Close),
    }
    .html()
}

/// How wide a modal dialog is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Width {
    /// A confirmation's width, 28 rem.
    #[default]
    Narrow,
    /// A form's width, 48 rem, for a text area or a table.
    Wide,
}

/// What a modal dialog the page draws shows.
#[derive(Debug)]
pub struct Modal {
    /// The dialog's id; its heading is `id` and `-title`.
    pub id: Cow<'static, str>,
    /// The heading.
    pub title: Word,
    /// Whether it is the danger form, the bad tone's glyph beside the
    /// heading, for an action that cannot be undone.
    pub danger: bool,
    /// How wide it is.
    pub width: Width,
    /// The body: the message, or the form's fields.
    pub body: Html,
    /// The actions, the confirming control last: the console's own.
    pub actions: Html,
    /// Whether the page arrives with it open, such as a form a post
    /// refused, drawn again with its answer.
    pub open: bool,
    /// Where Cancel leads when the page holds the dialog's state, such as
    /// the page without the query that opened it; `None` closes it in
    /// place.
    pub cancel: Option<String>,
}

#[derive(Template)]
#[template(path = "dialog/dialog.html", block = "modal")]
struct ModalView {
    id: Cow<'static, str>,
    class: String,
    open: bool,
    glyph: Html,
    title: Word,
    body: Html,
    cancel: Option<String>,
    secondary: &'static str,
    x: Html,
    cancel_words: Word,
    actions: Html,
}

/// Returns the modal dialog `modal`.
///
/// A trigger opens it with the `show-modal` command; one the page draws
/// open is shown in place with no script, and the `dialog` module opens it
/// as a modal as the page loads, so the page behind turns inert.
#[must_use]
pub fn modal_dialog(locale: &Locale, modal: Modal) -> Html {
    let Modal {
        id,
        title,
        danger,
        width,
        body,
        actions,
        open,
        cancel,
    } = modal;
    let mut class = String::from("fui-dialog fui-modal");
    if danger {
        class.push_str(" fui-dialog-danger");
    }
    if width == Width::Wide {
        class.push_str(" fui-dialog-wide");
    }
    ModalView {
        id,
        class,
        open,
        glyph: if danger {
            icon(Glyph::TRIANGLE_ALERT, "18")
        } else {
            Html::empty()
        },
        title,
        body,
        cancel,
        secondary: Variant::Secondary.class(),
        x: icon(Glyph::X, "16"),
        cancel_words: tr(locale, Text::Cancel),
        actions,
    }
    .html()
}
