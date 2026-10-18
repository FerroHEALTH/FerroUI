// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The code page: a bounded, highlighted panel in each of the four
//! languages, and a numbered panel a link targets a line of.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::view::code::{code_lines, code_panel};
use ferrohealth_ui::view::surface::{card, page_header, stack};
use leptos::prelude::*;

use crate::data::{AQL, JSON, XML, YAML};

/// The id prefix of the numbered panel's lines.
const LINES: &str = "file-L";

/// A panel in each language.
fn panels() -> AnyView {
    stack(view! {
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
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn CodePage() -> impl IntoView {
    let header = page_header(
        "Code panels",
        Some(String::from(
            "Bounded, scrollable and highlighted, never at the page's full height.",
        )),
        (),
    )
    .into_any();
    let sections = (panels(), numbered());
    view! {
        {super::title("Code panels")}
        {header}
        {stack(view! {
            {sections.0}
            {sections.1}
        })}
    }
}
