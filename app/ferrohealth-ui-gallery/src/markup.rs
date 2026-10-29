// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The two shorthands the pages write their own markup with, beside the
//! library's parts: markup made here and text escaped for it.

use ferrohealth_ui::html::Html;

/// The markup `markup`, made by the page from escaped parts.
#[must_use]
pub const fn h(markup: String) -> Html {
    Html::raw(markup)
}

/// `text` escaped as a text node or an attribute value.
#[must_use]
pub fn t(text: impl AsRef<str>) -> Html {
    Html::text(text)
}
