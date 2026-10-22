// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The document pane's view switch: the island that shows one of a
//! document's views in place.
//!
//! The views (highlighted, raw, rendered) are drawn on the server by
//! [`crate::view::document::document_pane`] and handed to the island
//! [`crate::island::DocumentViews`] as its children, every one of them, so
//! none reaches the WebAssembly. Each switch segment is a link to its view.
//! With no WebAssembly the link targets the view and the stylesheet shows
//! the view targeted (`:target`); once the island has hydrated, a press
//! shows the view in place, with no jump and no history entry, and marks
//! its segment current. No specification governs the switch: our own
//! design.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// One view of a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentView {
    /// The text highlighted in the code viewer, its lines numbered.
    Highlighted,
    /// The text as it stands.
    Raw,
    /// The console's own drawing of the document.
    Rendered,
}

impl DocumentView {
    /// The view's key, which its element and its segment carry as
    /// `data-view` and its id ends with.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Highlighted => "highlighted",
            Self::Raw => "raw",
            Self::Rendered => "rendered",
        }
    }

    /// The id of this view of the pane `id`.
    #[must_use]
    pub fn id(self, pane: &str) -> String {
        format!("{pane}-{}", self.key())
    }
}

/// One segment of the switch: a view and its words in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewChoice {
    /// The view it shows.
    pub view: DocumentView,
    /// Its words.
    pub words: String,
}

/// Returns the view switch named `label` of the pane `id`, one link per
/// choice in `views`, over `views_drawn`, the views the server drew; the
/// view `opening` shows first.
///
/// A console draws it as the island [`crate::island::DocumentViews`],
/// through [`crate::view::document::document_pane`].
#[must_use]
pub fn view_switch(
    id: &str,
    label: String,
    views: Vec<ViewChoice>,
    opening: DocumentView,
    views_drawn: AnyView,
) -> AnyView {
    let chosen = RwSignal::new(opening);
    let live = RwSignal::new(false);
    // NOTE: the Leptos book, "Guide: Islands"; an effect runs in the browser alone, so the
    // stylesheet's `:target` floor holds until the island has hydrated.
    Effect::new(move |_| live.set(true));
    let segments = views
        .into_iter()
        .map(|ViewChoice { view, words }| {
            let href = format!("#{}", view.id(id));
            view! {
                <a
                    href=href
                    class="fui-segment"
                    data-view=view.key()
                    aria-current=move || (chosen.get() == view).then_some("true")
                    on:click=move |event| {
                        event.prevent_default();
                        chosen.set(view);
                    }
                >
                    {words}
                </a>
            }
        })
        .collect_view();
    view! {
        <div role="group" aria-label=label class="fui-segmented fui-document-switch">
            {segments}
        </div>
        <div
            class="fui-document-views"
            class:is-live=move || live.get()
            data-shown=move || chosen.get().key()
        >
            {views_drawn}
        </div>
    }
    .into_any()
}
