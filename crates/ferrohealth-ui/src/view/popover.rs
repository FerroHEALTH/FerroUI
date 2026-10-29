// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The popover form: a small form inside an anchor, such as Save to the
//! CDR, that opens under its trigger.
//!
//! It is an HTML `dialog` its trigger opens as a modal through the
//! `show-modal` command, so it opens with no script, the page behind is
//! inert while it shows, the focus moves to its first control, the browser
//! closes it on Escape and on a press outside and returns the focus to the
//! trigger (HTML Living Standard, the `dialog` element and the `command`
//! attribute; WAI-ARIA Authoring Practices, the dialog (modal) pattern). It
//! opens 4 px under its trigger, aligned to the trigger's end edge, placed
//! against it by CSS anchor positioning, and centred where the browser has
//! none; below 40 rem it rises from the foot of the screen as a bottom sheet
//! over a scrim. The form inside is the console's, so it posts with no
//! script too. No specification governs the layout: our own design.

use std::borrow::Cow;

use askama::Template;

use crate::html::{Html, Render as _};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// What a popover form shows.
#[derive(Debug)]
pub struct PopoverForm {
    /// The popover's id, which its trigger opens; its title is `id` and
    /// `-title`. A static id or one made at run time, such as one per
    /// table row.
    pub id: Cow<'static, str>,
    /// The trigger's look.
    pub variant: Variant,
    /// The trigger's icon.
    pub glyph: Glyph,
    /// The trigger's words, such as "Save to the CDR".
    pub label: Cow<'static, str>,
    /// The popover's title.
    pub title: Cow<'static, str>,
    /// The form: the console's own, with its fields and its submit.
    pub body: Html,
}

#[derive(Template)]
#[template(path = "popover/form.html")]
struct PopoverView {
    id: Cow<'static, str>,
    class: &'static str,
    glyph: Html,
    label: Cow<'static, str>,
    title: Cow<'static, str>,
    body: Html,
}

/// Returns the popover form `form` with its trigger, in one anchor.
#[must_use]
pub fn popover_form(form: PopoverForm) -> Html {
    let PopoverForm {
        id,
        variant,
        glyph,
        label,
        title,
        body,
    } = form;
    PopoverView {
        id,
        class: variant.class(),
        glyph: icon(glyph, "16"),
        label,
        title,
        body,
    }
    .html()
}
