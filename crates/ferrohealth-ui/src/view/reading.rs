// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The clinical reading of a canonical document (E3).
//!
//! A composition read without its template: each RM section is a heading
//! with its archetype node id as a quiet hint that breaks only after a
//! dot; each `ELEMENT` is a label and value row, two columns that become
//! one on a phone. A quantity keeps its unit as a person reads it (°C, mm
//! Hg) with its UCUM code as the tooltip, a coded value is the M2 coding
//! the console draws, and a null flavour is the hollow ring and its words
//! (family rule 12; openEHR RM, Data Types Information Model, § Null
//! flavours, `ELEMENT.null_flavour`). A nested `CLUSTER` is indented with a
//! hairline edge, and its heading is one level below its parent's, so the
//! page outline follows the document (WCAG 2.2 SC 1.3.1). Bookkeeping is
//! folded away and says so once in a foot. No specification governs the
//! look: our own design.

use leptos::prelude::*;

use crate::i18n::{Arg, Text, tr_with};

/// The value of one element.
#[derive(Debug)]
pub enum Value {
    /// Words, as the document holds them.
    Text(String),
    /// A quantity: its magnitude as the document writes it, its unit as a
    /// person reads it ("mm Hg") and its UCUM code ("mm[Hg]"), which the
    /// unit carries as its tooltip.
    Quantity {
        /// The magnitude.
        magnitude: String,
        /// The unit as a person reads it.
        unit: String,
        /// The unit's UCUM code.
        ucum: String,
    },
    /// A coded value: the M2 coding the console draws, inline.
    Coded(AnyView),
    /// No value, and why: the null flavour in words, such as "No
    /// information: unknown".
    Null(String),
    /// Any other value the console draws, such as a timestamp.
    Other(AnyView),
}

/// One element: its label and its value.
#[derive(Debug)]
pub struct Element {
    /// The element's name.
    pub label: String,
    /// Its value.
    pub value: Value,
}

/// One RM section of the reading: an entry, or a cluster inside one.
#[derive(Debug)]
pub struct Section {
    /// Its name, the heading.
    pub title: String,
    /// Its archetype node id, the quiet hint after the heading, when it has
    /// one.
    pub node: Option<String>,
    /// Its elements, in the document's order.
    pub elements: Vec<Element>,
    /// The clusters inside it, each indented under its elements.
    pub sections: Vec<Section>,
}

/// A heading level in the page outline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Level {
    /// A level-two heading.
    Two,
    /// A level-three heading, the reading's default: under the page's
    /// title and its tab.
    #[default]
    Three,
    /// A level-four heading.
    Four,
    /// A level-five heading.
    Five,
    /// A level-six heading, the deepest HTML has; a deeper cluster stays
    /// at six.
    Six,
}

impl Level {
    /// The level one step deeper, six at most.
    #[must_use]
    pub const fn deeper(self) -> Self {
        match self {
            Self::Two => Self::Three,
            Self::Three => Self::Four,
            Self::Four => Self::Five,
            Self::Five | Self::Six => Self::Six,
        }
    }
}

/// What a clinical reading shows.
#[derive(Debug)]
pub struct Reading {
    /// The top-level sections, in the document's order.
    pub sections: Vec<Section>,
    /// The level of the top-level headings; each cluster's is one deeper.
    pub level: Level,
    /// The one foot that says what is folded away, such as the uids and the
    /// language, with its way to the full document.
    pub folded: Option<AnyView>,
}

/// Returns the archetype node id `id`, an archetype id or an ADL 2 HRID,
/// in mono as the quiet hint after a heading: it breaks only after a dot,
/// never inside a segment, and the whole id is its tooltip.
fn archetype(id: String) -> AnyView {
    let segments = id.split('.').map(str::to_owned).collect::<Vec<_>>();
    let last = segments.len().saturating_sub(1);
    let parts = segments
        .into_iter()
        .enumerate()
        .map(|(index, segment)| {
            let dot = (index < last).then(|| view! { "." <wbr /> });
            view! {
                <span>{segment}</span>
                {dot}
            }
        })
        .collect_view();
    view! {
        <code class="fui-archetype fui-reading-node" title=id>
            {parts}
        </code>
    }
    .into_any()
}

/// Returns the value `value` of one element.
fn value_view(value: Value) -> AnyView {
    match value {
        Value::Text(words) => words.into_any(),
        Value::Quantity {
            magnitude,
            unit,
            ucum,
        } => {
            let tooltip = tr_with(Text::ReadingUcum, &[("code", Arg::from(ucum))]);
            view! {
                <span class="fui-reading-value">{magnitude}</span>
                " "
                <span class="fui-reading-unit" title=tooltip>
                    {unit}
                </span>
            }
            .into_any()
        }
        Value::Null(words) => view! { <span class="fui-reading-null">{words}</span> }.into_any(),
        Value::Coded(view) | Value::Other(view) => view,
    }
}

/// Returns the heading `title` at `level`, its node hint after it.
fn heading(level: Level, title: String, node: Option<AnyView>) -> AnyView {
    match level {
        Level::Two => view! { <h2 class="fui-reading-title">{title}{node}</h2> }.into_any(),
        Level::Three => view! { <h3 class="fui-reading-title">{title}{node}</h3> }.into_any(),
        Level::Four => view! { <h4 class="fui-reading-title">{title}{node}</h4> }.into_any(),
        Level::Five => view! { <h5 class="fui-reading-title">{title}{node}</h5> }.into_any(),
        Level::Six => view! { <h6 class="fui-reading-title">{title}{node}</h6> }.into_any(),
    }
}

/// Returns the section `section` with its heading at `level`.
fn section_view(section: Section, level: Level) -> AnyView {
    let Section {
        title,
        node,
        elements,
        sections,
    } = section;
    let rows = (!elements.is_empty()).then(|| {
        let rows = elements
            .into_iter()
            .map(|Element { label, value }| {
                view! {
                    <div>
                        <dt>{label}</dt>
                        <dd>{value_view(value)}</dd>
                    </div>
                }
            })
            .collect_view();
        view! { <dl class="fui-reading-rows">{rows}</dl> }
    });
    let nested = sections
        .into_iter()
        .map(|inner| section_view(inner, level.deeper()))
        .collect_view();
    view! {
        <section class="fui-reading-section">
            {heading(level, title, node.map(archetype))}
            {rows}
            {nested}
        </section>
    }
    .into_any()
}

/// Returns the clinical reading `reading`. In a tab panel it is a part of
/// its own, its sections split by hairlines.
#[must_use]
pub fn reading(reading: Reading) -> AnyView {
    let Reading {
        sections,
        level,
        folded,
    } = reading;
    let sections = sections
        .into_iter()
        .map(|section| section_view(section, level))
        .collect_view();
    let folded = folded.map(|folded| {
        view! {
            <div class="fui-reading-section">
                <p class="fui-small fui-muted">{folded}</p>
            </div>
        }
    });
    view! { <div class="fui-reading">{sections}{folded}</div> }.into_any()
}
