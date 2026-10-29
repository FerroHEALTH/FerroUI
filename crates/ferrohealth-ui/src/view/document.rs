// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The document pane: a wire document a console reads in one or more
//! formats, shown highlighted, raw or rendered.
//!
//! The formats are links the console answers with the document in that
//! format, the one in force marked current, and Copy puts the document in
//! that format on the clipboard ([`crate::view::copy`]), read from the raw
//! view the page draws, so the page carries it twice, never three times.
//! The views are drawn on the server, every one of them: the highlighted
//! view is the code viewer (B1), the raw view the text as it stands, and
//! the rendered view the console's own drawing, when it has one. Each
//! switch segment is a link to its view: with no script the link targets
//! the view and the stylesheet shows the view targeted (`:target`); once
//! the `document` module runs, a press shows the view in place, with no
//! jump and no history entry, and marks its segment current. No
//! specification governs the switch: our own design.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::highlight::Language;
use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr, tr_with};
use crate::view::code::{Code, code_viewer};
use crate::view::copy::{CopySource, CopyWords, copy_from};
use crate::view::segmented::segmented_links;

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
    pub words: Word,
}

/// What a document pane shows.
#[derive(Debug)]
pub struct Document {
    /// The pane's id: each view's id is `id`, a dash and the view's key, the
    /// raw view's text is `id` and `-text`, and the highlighted lines' ids
    /// are `id`, `-L` and the line's number.
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
    pub rendered: Option<Html>,
}

/// The text of the raw view's `<pre>` as the server writes it: one more
/// line break before a text that opens with one.
fn raw_text(text: String) -> String {
    // NOTE: HTML Living Standard §13.2.6.4.7: the parser drops one line break right after
    // `<pre>`, so a text that opens with one is written with two.
    if text.starts_with('\n') {
        format!("\n{text}")
    } else {
        text
    }
}

#[derive(Template)]
#[template(path = "document/pane.html")]
struct PaneView {
    id: String,
    label: String,
    choice: Html,
    switch_label: Word,
    views: Vec<ViewChoice>,
    opening: DocumentView,
    highlighted: Html,
    raw_label: Word,
    raw: String,
    rendered: Option<Html>,
    copy: Html,
}

/// Returns the document pane `document`: its format links, its view switch
/// and its views.
#[must_use]
pub fn document_pane(locale: &Locale, document: Document) -> Html {
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
    let choice = if formats.len() > 1 {
        segmented_links(tr(locale, Text::DocumentFormat), formats, format)
    } else {
        Html::empty()
    };
    let opening = match (opening, rendered.is_some()) {
        (DocumentView::Rendered, false) => DocumentView::Highlighted,
        (view, _) => view,
    };
    let mut views = vec![
        ViewChoice {
            view: DocumentView::Highlighted,
            words: tr(locale, Text::DocumentHighlighted),
        },
        ViewChoice {
            view: DocumentView::Raw,
            words: tr(locale, Text::DocumentRaw),
        },
    ];
    if rendered.is_some() {
        views.push(ViewChoice {
            view: DocumentView::Rendered,
            words: tr(locale, Text::DocumentRendered),
        });
    }
    let highlighted = code_viewer(
        locale,
        None,
        &Code {
            label: tr_with(
                locale,
                Text::DocumentHighlightedRegion,
                &[("document", label.clone().into())],
            )
            .into(),
            language,
            text: text.clone(),
            ids: Some(format!("{id}-L")),
            current: None,
            diagnostics: Vec::new(),
        },
    );
    let copy = copy_from(
        format!("{id}-copy"),
        tr(locale, Text::DocumentCopy),
        CopySource::Element(format!("{id}-text")),
        CopyWords::of(locale),
    );
    PaneView {
        raw_label: tr_with(
            locale,
            Text::DocumentRawRegion,
            &[("document", label.clone().into())],
        ),
        id,
        label,
        choice,
        switch_label: tr(locale, Text::DocumentView),
        views,
        opening,
        highlighted,
        raw: raw_text(text),
        rendered,
        copy,
    }
    .html()
}
