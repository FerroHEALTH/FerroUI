// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The document pane: a wire document a console reads in one or more
//! formats, shown highlighted, raw or rendered.
//!
//! The formats are links the console answers with the document in that
//! format, the one in force marked current. The views are drawn on the
//! server, every one of them: the highlighted view is the code viewer
//! (B1), the raw view the text as it stands, and the rendered view the
//! console's own drawing, when it has one. The view switch is the island
//! [`crate::island::DocumentViews`] over them, so a reader reaches every
//! view with no WebAssembly through its link, and the island switches them
//! in place once it has hydrated (`crate::widget::document`).

use leptos::prelude::*;

use crate::highlight::Language;
use crate::i18n::{Text, tr, tr_with};
use crate::island::DocumentViews;
use crate::view::code::{Code, code_viewer};
use crate::view::segmented::segmented_links;
use crate::widget::document::{DocumentView, ViewChoice};

/// What a document pane shows.
#[derive(Debug)]
pub struct Document {
    /// The pane's id: each view's id is `id`, a dash and the view's key, and
    /// the highlighted lines' ids are `id`, `-L` and the line's number.
    pub id: String,
    /// The document's name, such as "Observation".
    pub label: String,
    /// The formats the console can send the document in, each the link
    /// that asks for it and its words; one or none draws no choice.
    pub formats: Vec<(String, String)>,
    /// The index in `formats` of the format in force.
    pub format: usize,
    /// The grammar the text is highlighted as.
    pub language: Language,
    /// The document's text, in the format in force.
    pub text: String,
    /// The view shown first, as a deep link names it; a rendered view the
    /// pane does not have opens the highlighted one.
    pub opening: DocumentView,
    /// The console's own drawing of the document, when it has one.
    pub rendered: Option<AnyView>,
}

/// Returns the view `view` of the pane `id`, holding `body`.
fn view_of(id: &str, view: DocumentView, body: AnyView) -> AnyView {
    view! {
        <div class="fui-document-view" id=view.id(id) data-view=view.key()>
            {body}
        </div>
    }
    .into_any()
}

/// Returns the document pane `document`: its format links, its view switch
/// and its views.
#[must_use]
pub fn document_pane(document: Document) -> AnyView {
    let Document {
        id,
        label,
        formats,
        format,
        language,
        text,
        opening,
        rendered,
    } = document;
    let choice =
        (formats.len() > 1).then(|| segmented_links(tr(Text::DocumentFormat), formats, format));
    let opening = match (opening, rendered.is_some()) {
        (DocumentView::Rendered, false) => DocumentView::Highlighted,
        (view, _) => view,
    };
    let mut views = vec![
        ViewChoice {
            view: DocumentView::Highlighted,
            words: tr(Text::DocumentHighlighted),
        },
        ViewChoice {
            view: DocumentView::Raw,
            words: tr(Text::DocumentRaw),
        },
    ];
    if rendered.is_some() {
        views.push(ViewChoice {
            view: DocumentView::Rendered,
            words: tr(Text::DocumentRendered),
        });
    }
    let highlighted = code_viewer(
        None,
        &Code {
            label: tr_with(
                Text::DocumentHighlightedRegion,
                &[("document", label.clone().into())],
            ),
            language,
            text: text.clone(),
            ids: Some(format!("{id}-L")),
            current: None,
            diagnostics: Vec::new(),
        },
    );
    let raw_label = tr_with(
        Text::DocumentRawRegion,
        &[("document", label.clone().into())],
    );
    let raw = view! {
        <div class="fui-code-panel fui-scroll-cue" role="region" aria-label=raw_label tabindex="0">
            <pre>{text}</pre>
        </div>
    }
    .into_any();
    let drawn = view! {
        {view_of(&id, DocumentView::Highlighted, highlighted)}
        {view_of(&id, DocumentView::Raw, raw)}
        {rendered.map(|body| {
            view_of(
                &id,
                DocumentView::Rendered,
                view! { <div class="fui-document-rendered">{body}</div> }.into_any(),
            )
        })}
    }
    .into_any();
    // TODO(#220): the copy island copies the text in force.
    let switch_label = tr(Text::DocumentView);
    let pane_id = id.clone();
    view! {
        <section class="fui-document" id=pane_id aria-label=label>
            {choice}
            <DocumentViews id=id label=switch_label views=views opening=opening>
                {drawn}
            </DocumentViews>
        </section>
    }
    .into_any()
}
