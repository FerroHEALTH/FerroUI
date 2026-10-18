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

/// The id prefix of the numbered panel's lines.
const LINES: &str = "file-L";

/// The formats the document pane offers.
const FORMATS: &[(&str, &str)] = &[("json", "JSON"), ("xml", "XML")];

/// The document pane over the JSON and XML samples, with a rendered view.
fn document() -> AnyView {
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
        ("Resource", "Observation".into_any()),
        ("Status", "final".into_any()),
        ("Code", "A synthetic measurement".into_any()),
        ("Value", "72.5 beats/min".into_any()),
    ])
    .into_any();
    card(
        "Document pane",
        document_pane(
            DocumentPane {
                id: "document",
                label: "Observation",
                formats: FORMATS,
                format: format.into(),
                text,
                language,
                rendered: Some(rendered),
            },
            move |key| format.set(key),
        ),
    )
    .into_any()
}

/// A panel in each language.
fn panels() -> AnyView {
    grid(view! {
        {card("YAML", code_panel("A mapping file, YAML", Language::Yaml, YAML))}
        {card("JSON", code_panel("A resource, JSON", Language::Json, JSON))}
        {card("XML", code_panel("A composition, XML", Language::Xml, XML))}
        {card("AQL", code_panel("A query, AQL", Language::Aql, AQL))}
    })
    .into_any()
}

/// The numbered panel, and a link to one of its lines.
fn numbered() -> AnyView {
    card(
        "Numbered lines",
        view! {
            <p>
                "A diagnostic links to its line: " <a href=format!("#{LINES}13")>"line 13"</a> "."
            </p>
            {code_lines("The mapping file, numbered", Language::Yaml, YAML, LINES)}
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
            subtitle: Some(String::from(
                "Bounded, scrollable and highlighted, never at the page's full height.",
            )),
            crumbs: vec![
                Crumb::new("Gallery", root.clone()),
                Crumb::new("Code and documents", format!("{root}/code")),
            ],
            mono: true,
        },
        (),
    )
    .into_any();
    let meta = page_meta(Some(String::from(
        "Answered by the synthetic replica gallery-1.",
    )))
    .into_any();
    let sections = (document(), panels(), numbered());
    view! {
        {super::title("Code and documents")}
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
