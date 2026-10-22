// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The code page.
//!
//! It holds the document pane with its format links and its views, the
//! code viewer with a long line, a current line and two findings under
//! their lines, and a viewer in each of the four languages, under a header
//! with a breadcrumb trail and a title in the code face.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::view::code::{Code, CodeHead, Digest, Gravity, LineDiagnostic, code_card};
use ferrohealth_ui::view::document::{Document, document_pane};
use ferrohealth_ui::view::surface::{
    Crumb, Header, card, definitions, grid, page_heading, page_meta, stack,
};
use ferrohealth_ui::widget::document::DocumentView;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::data::{
    AQL, FHIRCONNECT, FILE_NAMES, FINDING_CODES, JSON, MAPPING, MAPPING_DIGEST, MAPPING_PATH, XML,
    YAML,
};
use crate::locale::{Msg, tr};

/// The id prefix of the viewer's lines.
const LINES: &str = "file-L";

/// The document pane over the JSON and XML samples, in the format `xml`
/// names, with a rendered view, its format links leading back to `here`.
fn document(here: &str, xml: bool) -> AnyView {
    let rendered = definitions(vec![
        (tr(Msg::Resource), tr(Msg::Observation).into_any()),
        (tr(Msg::Status), "final".into_any()),
        (tr(Msg::Code), tr(Msg::SyntheticMeasurement).into_any()),
        (tr(Msg::Value), tr(Msg::N725BeatsMin).into_any()),
    ])
    .into_any();
    let pane = document_pane(Document {
        id: String::from("document"),
        label: tr(Msg::Observation),
        formats: vec![
            (format!("{here}?format=json"), tr(Msg::Json)),
            (format!("{here}?format=xml"), tr(Msg::Xml)),
        ],
        format: usize::from(xml),
        language: if xml { Language::Xml } else { Language::Json },
        text: String::from(if xml { XML } else { JSON }),
        opening: DocumentView::Highlighted,
        rendered: Some(rendered),
    });
    card(tr(Msg::DocumentPane), pane).into_any()
}

/// The code viewer over the mapping file: a long line, the linked line 13
/// current, and an error and a warning under their lines.
fn viewer() -> AnyView {
    let [retries, start] = FINDING_CODES;
    let viewer = code_card(
        CodeHead {
            title: String::from(MAPPING_PATH),
            tags: vec![String::from(FHIRCONNECT), tr(Msg::OwnerConsole)],
            digest: Some(Digest {
                text: String::from(MAPPING_DIGEST),
                title: tr(Msg::DigestOfBytes),
            }),
            actions: None,
        },
        &Code {
            label: tr(Msg::TheMappingFile),
            language: Language::Yaml,
            text: String::from(MAPPING),
            ids: Some(String::from(LINES)),
            current: Some(13),
            diagnostics: vec![
                LineDiagnostic {
                    line: 18,
                    gravity: Gravity::Error,
                    code: String::from(retries),
                    message: tr(Msg::FindingRetries),
                },
                LineDiagnostic {
                    line: 20,
                    gravity: Gravity::Warning,
                    code: String::from(start),
                    message: tr(Msg::FindingStartTwice),
                },
            ],
        },
    );
    stack(view! {
        <p>{tr(Msg::DiagnosticLinksLine)} <a href=format!("#{LINES}13")>{tr(Msg::Line13)}</a> "."</p>
        {viewer}
    })
    .into_any()
}

/// The tag of a viewer in `language`.
fn language_tag(language: Language) -> String {
    tr(match language {
        Language::Yaml => Msg::Yaml,
        Language::Json => Msg::Json,
        Language::Xml => Msg::Xml,
        Language::Aql | Language::Plain => Msg::Aql,
    })
}

/// A viewer of `text` in `language`, its head naming the file `title`,
/// its box named `label`.
fn each(title: &str, label: Msg, language: Language, text: &str) -> AnyView {
    code_card(
        CodeHead {
            title: title.to_owned(),
            tags: vec![language_tag(language)],
            ..CodeHead::default()
        },
        &Code {
            label: tr(label),
            language,
            text: text.to_owned(),
            ids: None,
            current: None,
            diagnostics: Vec::new(),
        },
    )
}

/// A viewer in each language, named by its file.
fn languages() -> AnyView {
    let [yaml, json, xml, aql] = FILE_NAMES;
    stack(view! {
        {grid(view! {
            {each(yaml, Msg::MappingFileYaml, Language::Yaml, YAML)}
            {each(json, Msg::ResourceJson, Language::Json, JSON)}
        })}
        {grid(view! {
            {each(xml, Msg::CompositionXml, Language::Xml, XML)}
            {each(aql, Msg::QueryAql, Language::Aql, AQL)}
        })}
    })
    .into_any()
}

/// The code page.
#[component]
pub fn CodePage() -> impl IntoView {
    let root =
        crate::app::route_brand().map_or_else(String::new, |brand| format!("/{}", brand.key()));
    let here = format!("{root}/code");
    let xml = use_query_map().with_untracked(|query| query.get("format").as_deref() == Some("xml"));
    let header = page_heading(
        Header {
            title: String::from("gallery.context.v1"),
            subtitle: Some(tr(Msg::BoundedScrollableHighlightedNeverPage)),
            crumbs: vec![
                Crumb::new(tr(Msg::Gallery), root.clone()),
                Crumb::new(tr(Msg::CodeDocuments), here.clone()),
            ],
            mono: true,
            badge: None,
        },
        (),
    )
    .into_any();
    let meta = page_meta(Some(tr(Msg::AnsweredSyntheticReplicaGallery1))).into_any();
    let sections = (document(&here, xml), viewer(), languages());
    view! {
        {super::title(tr(Msg::CodeDocuments))}
        {header}
        {meta}
        {stack(
            view! {
                {sections.0}
                {sections.1}
                {sections.2}
            },
        )}
    }
}
