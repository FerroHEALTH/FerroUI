// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The code page.
//!
//! It holds the document pane with its format and view choices and a
//! copy, a bounded, highlighted panel in each of the four languages, and a
//! numbered panel a link targets a line of, under a header with a
//! breadcrumb trail and a title in the code face.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::view::code::{code_lines, code_panel};
use ferrohealth_ui::view::surface::{
    Crumb, Header, card, definitions, grid, page_heading, page_meta, stack,
};
use ferrohealth_ui::widget::document::{DocumentPane, document_pane};
use leptos::prelude::*;

use crate::data::{AQL, JSON, XML, YAML};
use crate::locale::{Msg, tr};

/// The id prefix of the numbered panel's lines.
const LINES: &str = "file-L";

/// The formats the document pane offers, in the page's locale.
fn formats() -> Vec<(&'static str, std::borrow::Cow<'static, str>)> {
    vec![("json", tr(Msg::Json).into()), ("xml", tr(Msg::Xml).into())]
}

/// The document pane over the JSON and XML samples, with a rendered view,
/// drawn by the island [`crate::islands::ObservationDocument`].
fn document() -> AnyView {
    card(
        tr(Msg::DocumentPane),
        view! { <crate::islands::ObservationDocument /> },
    )
    .into_any()
}

/// The document pane over the JSON and XML samples, its format and view
/// chosen in place.
pub(crate) fn observation_document() -> AnyView {
    let format = RwSignal::new("json");
    let text = Signal::derive(move || String::from(if format.get() == "xml" { XML } else { JSON }));
    let language = Signal::derive(move || {
        if format.get() == "xml" {
            Language::Xml
        } else {
            Language::Json
        }
    });
    let rendered = definitions(vec![
        (tr(Msg::Resource), tr(Msg::Observation).into_any()),
        (tr(Msg::Status), "final".into_any()),
        (tr(Msg::Code), tr(Msg::SyntheticMeasurement).into_any()),
        (tr(Msg::Value), tr(Msg::N725BeatsMin).into_any()),
    ])
    .into_any();
    document_pane(
        DocumentPane {
            id: "document",
            label: tr(Msg::Observation).into(),
            formats: formats(),
            format: format.into(),
            text,
            language,
            rendered: Some(rendered),
        },
        move |key| format.set(key),
    )
}

/// A panel in each language.
fn panels() -> AnyView {
    stack(view! {
        {grid(view! {
            {card(tr(Msg::Yaml), code_panel(&tr(Msg::MappingFileYaml), Language::Yaml, YAML))}
            {card(tr(Msg::Json), code_panel(&tr(Msg::ResourceJson), Language::Json, JSON))}
        })}
        {grid(view! {
            {card(tr(Msg::Xml), code_panel(&tr(Msg::CompositionXml), Language::Xml, XML))}
            {card(tr(Msg::Aql), code_panel(&tr(Msg::QueryAql), Language::Aql, AQL))}
        })}
    })
    .into_any()
}

/// The numbered panel, and a link to one of its lines.
fn numbered() -> AnyView {
    card(
        tr(Msg::NumberedLines),
        view! {
            <p>
                {tr(Msg::DiagnosticLinksLine)} <a href=format!("#{LINES}13")>{tr(Msg::Line13)}</a> "."
            </p>
            {code_lines(&tr(Msg::MappingFileNumbered), Language::Yaml, YAML, LINES)}
        },
    )
    .into_any()
}

/// The code page.
#[component]
pub fn CodePage() -> impl IntoView {
    let root =
        crate::app::route_brand().map_or_else(String::new, |brand| format!("/{}", brand.key()));
    let header = page_heading(
        Header {
            title: String::from("gallery.context.v1"),
            subtitle: Some(tr(Msg::BoundedScrollableHighlightedNeverPage)),
            crumbs: vec![
                Crumb::new(tr(Msg::Gallery), root.clone()),
                Crumb::new(tr(Msg::CodeDocuments), format!("{root}/code")),
            ],
            mono: true,
        },
        (),
    )
    .into_any();
    let meta = page_meta(Some(tr(Msg::AnsweredSyntheticReplicaGallery1))).into_any();
    let sections = (document(), panels(), numbered());
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
