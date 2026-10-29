// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The replica line (F6): which replica of a replicated product answered a
//! reference page, so an operator knows a read may lag a write.
//!
//! It is one quiet line under the page's title, "Answered by replica
//! gateway-0", plain text in its place in the outline. Only the reference
//! templates draw it, the settings and reference page (T3) and About, each
//! through its own `replica` field, so no other page can. No specification
//! governs the line: our own design.

use crate::html::Html;
use crate::i18n::{Locale, Text, tr};
use crate::icon::{Glyph, icon};

/// The replica that answered a reference page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replica {
    /// The replica's name, such as "gateway-0", drawn in the code face.
    pub name: String,
    /// When it answered, such as "09:42:11", after the name when known.
    pub at: Option<String>,
}

/// Returns the replica line of `replica`: the server glyph, "Answered by
/// replica" and its name, then the time it answered when it is known.
pub(crate) fn replica_line(locale: &Locale, replica: Replica) -> Html {
    let Replica { name, at } = replica;
    let at = at.map(|at| format!(" · {at}")).unwrap_or_default();
    Html::raw(format!(
        "<p class=\"fui-replica-line\">{}<span>{} <code>{}</code>{}</span></p>",
        icon(Glyph::SERVER, "12"),
        tr(locale, Text::ReplicaAnswered),
        Html::text(name),
        Html::text(at)
    ))
}
