// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The side panel: content opened from the inline end of the page.
//!
//! It holds what the operator opens from a button that is no navigation,
//! such as the session's access and scopes; a page draws it through
//! [`crate::island::SidePanel`]. Until the page has hydrated, its opener is
//! a link to the content's own route, so a reader with no WebAssembly
//! reaches the content as a page. Once it has hydrated, the opener is a
//! button that shows the panel as a modal HTML `dialog` through the
//! `show-modal` command: the page behind is inert, the focus moves to its
//! Close button, the browser closes it on Escape and on a press outside and
//! returns the focus to the opener (HTML Living Standard, the `dialog`
//! element and the `command` attribute; WAI-ARIA Authoring Practices, the
//! dialog (modal) pattern). From 40 rem it is 24 rem wide at the inline
//! end; below that it is a sheet over the whole viewport. Its body is the
//! server's, built once and kept, so a field in it keeps what was typed
//! while it is closed.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// What opens a side panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opener {
    /// Its words, in the page's locale.
    pub label: String,
    /// The glyph before its words, if any.
    pub glyph: Option<Glyph>,
    /// How much it stands out.
    pub variant: Variant,
    /// The route of the panel's content as a page of its own, which the
    /// opener leads to until the page has hydrated.
    pub href: String,
}

/// What a side panel shows.
#[derive(Debug)]
pub struct SidePanel {
    /// The panel's id: its heading is `id` and `-title`, its scrolling body
    /// `id` and `-body`.
    pub id: String,
    /// The heading, in the page's locale.
    pub title: String,
    /// What opens it.
    pub opener: Opener,
    /// The body, drawn on the server.
    pub body: AnyView,
    /// The words the panel draws itself ([`SidePanelWords::of_page`]).
    pub words: SidePanelWords,
}

/// The words a side panel draws itself, resolved on the server in the
/// page's locale, so an island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidePanelWords {
    /// The Close button's name.
    pub close: Word,
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

/// Returns the side panel `panel` and its opener: a link to the content's
/// route until the page has hydrated, then the button that opens it.
#[must_use]
pub fn side_panel(panel: SidePanel) -> AnyView {
    let SidePanel {
        id,
        title,
        opener:
            Opener {
                label,
                glyph,
                variant,
                href,
            },
        body,
        words: SidePanelWords { close },
    } = panel;
    let hydrated = RwSignal::new(false);
    // NOTE: the Leptos book, "Guide: Islands"; an effect runs in the browser alone.
    Effect::new(move |_| hydrated.set(true));
    let title_id = format!("{id}-title");
    let labelled_by = title_id.clone();
    let body_id = format!("{id}-body");
    let link_label = label.clone();
    let (target, controls, close_target) = (id.clone(), id.clone(), id.clone());
    view! {
        <a href=href class=variant.class() hidden=move || hydrated.get()>
            {glyph.map(|glyph| icon(glyph, "16"))}
            {link_label}
        </a>
        <button
            type="button"
            class=variant.class()
            command="show-modal"
            commandfor=target
            aria-controls=controls
            aria-haspopup="dialog"
            hidden=move || !hydrated.get()
        >
            {glyph.map(|glyph| icon(glyph, "16"))}
            {label}
        </button>
        <dialog id=id closedby="any" aria-labelledby=labelled_by class="fui-side-panel">
            <div class="fui-side-panel-head">
                <h2 id=title_id class="fui-card-title">
                    {title}
                </h2>
                <button
                    type="button"
                    class="fui-icon-button fui-side-panel-close"
                    command="close"
                    commandfor=close_target
                    aria-label=close
                >
                    {icon(Glyph::X, "16")}
                </button>
            </div>
            <div id=body_id class="fui-side-panel-body">
                {body}
            </div>
        </dialog>
    }
    .into_any()
}
