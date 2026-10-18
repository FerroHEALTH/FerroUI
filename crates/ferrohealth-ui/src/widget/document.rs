// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The document pane: a wire document shown highlighted, raw or rendered,
//! with its format choice and a copy.
//!
//! It is built from the library's parts: the format selector and the view
//! choice are segmented controls, the highlighted view is the code panel's
//! drawing, the raw view the text as it stands, and Copy the copy button.
//! A console that can draw the document its own way passes that view, and
//! the pane offers it as Rendered. Every view is built once and the ones
//! not chosen are hidden, so a rendered view keeps its state. The body is a
//! bounded box the keyboard scrolls, its lines not wrapped.

use leptos::prelude::*;

use crate::highlight::{Language, Lines, html};
use crate::widget::copy::copy_button;
use crate::widget::segmented::segmented;

/// The views a pane offers, with and without a rendered one.
const VIEWS: &[(&str, &str)] = &[("highlighted", "Highlighted"), ("raw", "Raw")];
const VIEWS_RENDERED: &[(&str, &str)] = &[
    ("highlighted", "Highlighted"),
    ("raw", "Raw"),
    ("rendered", "Rendered"),
];

/// What a document pane shows.
#[derive(Debug)]
pub struct DocumentPane {
    /// The pane's id: its views are `id` and `-highlighted`, `-raw` and
    /// `-rendered`, its copy button `id` and `-copy`.
    pub id: &'static str,
    /// The pane's name, such as "Composition".
    pub label: &'static str,
    /// The formats the console can fetch the document in, each its key and
    /// its words; one or none draws no selector.
    pub formats: &'static [(&'static str, &'static str)],
    /// The format in force.
    pub format: Signal<&'static str>,
    /// The document's text, in the format in force.
    pub text: Signal<String>,
    /// The grammar the text is highlighted as.
    pub language: Signal<Language>,
    /// The console's own drawing of the document, when it has one.
    pub rendered: Option<AnyView>,
}

/// Returns the document pane `pane`, open on its highlighted view;
/// `on_format` receives the format a press on the selector chooses, which
/// the console fetches.
#[must_use]
pub fn document_pane(
    pane: DocumentPane,
    on_format: impl Fn(&'static str) + Clone + Send + Sync + 'static,
) -> AnyView {
    document_pane_opening(pane, "highlighted", Signal::stored(true), on_format)
}

/// Returns the document pane `pane` open on the view `opening`.
///
/// `opening` is `highlighted`, `raw` or `rendered`, as a deep link names it.
/// The rendered view is offered only while `rendered_available` holds, as a
/// format with no rendered reading does not; a pane on a view it cannot show
/// opens on the highlighted one.
#[must_use]
pub fn document_pane_opening(
    pane: DocumentPane,
    opening: &'static str,
    rendered_available: Signal<bool>,
    on_format: impl Fn(&'static str) + Clone + Send + Sync + 'static,
) -> AnyView {
    let DocumentPane {
        id,
        label,
        formats,
        format,
        text,
        language,
        rendered,
    } = pane;
    let has_rendered = rendered.is_some();
    let offers_rendered = move || has_rendered && rendered_available.get();
    let opening = match opening {
        "raw" => "raw",
        "rendered" => "rendered",
        _ => "highlighted",
    };
    let chosen = RwSignal::new(opening);
    // The view on screen: the one chosen, or the highlighted one while the
    // chosen rendered view is not offered.
    let view_in_force = Memo::new(move |_| match chosen.get() {
        "rendered" if !offers_rendered() => "highlighted",
        other => other,
    });
    let selector = (formats.len() > 1).then(|| segmented("Format", formats, format, on_format));
    let switch = move || {
        let views = if offers_rendered() {
            VIEWS_RENDERED
        } else {
            VIEWS
        };
        segmented("View", views, view_in_force.into(), move |key| {
            chosen.set(key);
        })
    };
    let highlighted = move || {
        let language = language.get();
        text.with(|text| html(language, text, Lines::default(), |_| None))
    };
    let copy_id = format!("{id}-copy");
    let rendered = rendered.map(|body| {
        view! {
            <div
                id=format!("{id}-rendered")
                class="fui-document-rendered"
                hidden=move || view_in_force.get() != "rendered"
            >
                {body}
            </div>
        }
    });
    view! {
        <section class="fui-document" id=id aria-label=label>
            <div class="fui-document-tools">
                {selector} {switch} <span class="fui-document-copy">
                    {copy_button(copy_id, "Copy", text)}
                </span>
            </div>
            <div
                id=format!("{id}-highlighted")
                class="fui-code-panel fui-document-body"
                role="region"
                aria-label=format!("{label}, highlighted")
                tabindex="0"
                hidden=move || view_in_force.get() != "highlighted"
            >
                <pre inner_html=highlighted></pre>
            </div>
            <div
                id=format!("{id}-raw")
                class="fui-code-panel fui-document-body"
                role="region"
                aria-label=format!("{label}, raw")
                tabindex="0"
                hidden=move || view_in_force.get() != "raw"
            >
                <pre>{move || text.get()}</pre>
            </div>
            {rendered}
        </section>
    }
    .into_any()
}
