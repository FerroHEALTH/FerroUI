// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The side panel: content the operator opens from a menu or a button that
//! is no navigation, such as the session's access and scopes, from the
//! inline end of the page.
//!
//! It is an HTML popover, so a button anywhere opens it with no script, the
//! browser closes it on Escape and on a press outside, and returns the
//! focus to the button (HTML, the `popover` attribute). The browser half
//! moves the focus to its Close button as it opens, through the directive
//! [`crate::widget::focus::focus_on_open`]. Its body is built once
//! and kept, so a field in it keeps what was typed while it is closed.

use std::borrow::Cow;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::widget::focus::focus_on_open;

/// What a side panel shows.
#[derive(Debug)]
pub struct SidePanel {
    /// The panel's id, which its trigger opens; its heading is `id` and
    /// `-title`.
    pub id: &'static str,
    /// The id of its scrolling body, which a journey reads.
    pub body_id: &'static str,
    /// The heading.
    pub title: Cow<'static, str>,
    /// The body.
    pub body: AnyView,
    /// The words the panel draws itself ([`SidePanelWords::of_page`]).
    pub words: SidePanelWords,
}

/// The words a side panel draws itself, resolved on the server in the
/// page's locale, so an island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidePanelWords {
    /// The Close button's name.
    pub close: String,
}

impl SidePanelWords {
    /// The panel's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            close: tr(Text::Close),
        }
    }
}

/// Returns a button of `class` that opens the side panel `id`, with `glyph`
/// beside `label`.
#[must_use]
pub fn side_panel_trigger(
    id: &'static str,
    class: &'static str,
    glyph: Glyph,
    label: impl Into<String>,
) -> AnyView {
    let label = label.into();
    view! {
        <button type="button" class=class popovertarget=id aria-controls=id>
            {icon(glyph, "16")}
            {label}
        </button>
    }
    .into_any()
}

/// Returns the side panel `panel`.
#[must_use]
pub fn side_panel(panel: SidePanel) -> AnyView {
    let SidePanel {
        id,
        body_id,
        title,
        body,
        words: SidePanelWords { close },
    } = panel;
    let title_id = format!("{id}-title");
    let labelled_by = title_id.clone();
    view! {
        <aside
            id=id
            popover="auto"
            role="dialog"
            aria-labelledby=labelled_by
            class="fui-side-panel"
            use:focus_on_open=".fui-side-panel-close"
        >
            <div class="fui-side-panel-head">
                <h2 id=title_id class="fui-card-title">
                    {title}
                </h2>
                <button
                    type="button"
                    class="fui-icon-button fui-side-panel-close"
                    popovertarget=id
                    popovertargetaction="hide"
                    aria-label=close
                >
                    {icon(Glyph::X, "16")}
                </button>
            </div>
            <div id=body_id class="fui-side-panel-body">
                {body}
            </div>
        </aside>
    }
    .into_any()
}
