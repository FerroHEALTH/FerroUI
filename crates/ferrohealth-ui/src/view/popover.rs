// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The popover form: a small form inside an anchor, such as Save to the
//! CDR, that opens under its trigger.
//!
//! It is an HTML popover, so the trigger opens it with no script, the
//! browser closes it on Escape and on a press outside and returns the focus
//! to the trigger (HTML Living Standard, the `popover` attribute). It opens
//! 4 px under its trigger, aligned to the trigger's end edge, placed against
//! it by CSS anchor positioning; below 40 rem it rises from the foot of the
//! screen as a bottom sheet over a scrim. The form inside is the console's,
//! so it posts with no script too. No specification governs the layout: our
//! own design.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// What a popover form shows.
#[derive(Debug)]
pub struct PopoverForm {
    /// The popover's id, which its trigger opens; its title is `id` and
    /// `-title`.
    pub id: &'static str,
    /// The trigger's look.
    pub variant: Variant,
    /// The trigger's icon.
    pub glyph: Glyph,
    /// The trigger's words, such as "Save to the CDR".
    pub label: Cow<'static, str>,
    /// The popover's title.
    pub title: Cow<'static, str>,
    /// The form: the console's own, with its fields and its submit.
    pub body: AnyView,
}

/// Returns the popover form `form` with its trigger, in one anchor.
#[must_use]
pub fn popover_form(form: PopoverForm) -> AnyView {
    let PopoverForm {
        id,
        variant,
        glyph,
        label,
        title,
        body,
    } = form;
    let title_id = format!("{id}-title");
    let labelled_by = title_id.clone();
    view! {
        <span class="fui-anchor">
            <button type="button" class=variant.class() popovertarget=id aria-controls=id>
                {icon(glyph, "16")}
                {label}
            </button>
            <div id=id popover="auto" class="fui-popover" role="dialog" aria-labelledby=labelled_by>
                <p id=title_id class="fui-popover-title">
                    {title}
                </p>
                {body}
            </div>
        </span>
    }
    .into_any()
}
