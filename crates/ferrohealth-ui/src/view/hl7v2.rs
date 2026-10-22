// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The HL7 v2 message viewer (B8): a message raw and parsed.
//!
//! The raw view ([`hl7v2_raw`]) is the message in the code face, one
//! segment per line, the segment ids and the separators tinted by the HL7
//! v2 highlighter ([`crate::highlight::Language::Hl7v2`]); a line never
//! wraps and scrolls sideways under the scroll cue, and the box is
//! focusable so the keyboard scrolls it (WCAG 2.2 SC 2.1.1). The parsed
//! view ([`hl7v2_segments`]) is one block per segment, its id and name,
//! then its fields by position, name and value; on a phone the names drop
//! and the positions and values stay. The library parses nothing: the
//! console reads the message with its own HL7 v2 model and hands the
//! segments over. No specification governs the viewer: our own design.

use leptos::prelude::*;

use crate::highlight::{self, Language};

/// One field of a parsed segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hl7Field {
    /// Its position, such as `PID-3`.
    pub position: String,
    /// Its name in the HL7 v2 definitions, such as "Patient identifier
    /// list"; dropped on a phone.
    pub name: String,
    /// Its value as the message carries it, separators included.
    pub value: String,
}

/// One parsed segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hl7Segment {
    /// Its id, such as `PID`.
    pub id: String,
    /// Its name, such as "Patient identification".
    pub name: String,
    /// Its fields that carry a value, by position.
    pub fields: Vec<Hl7Field>,
}

/// Returns the message `message` raw, one segment per line, in a scrolling
/// box named `label`.
///
/// A carriage return, a line feed or both end a segment.
#[must_use]
pub fn hl7v2_raw(label: impl Into<String>, message: &str) -> AnyView {
    let text = message.replace("\r\n", "\n").replace('\r', "\n");
    let text = text.strip_suffix('\n').unwrap_or(&text);
    let lines = highlight::lines(Language::Hl7v2, text);
    let last = lines.len().saturating_sub(1);
    let lines = lines
        .into_iter()
        .enumerate()
        .map(|(index, tokens)| {
            let tokens = tokens
                .into_iter()
                .map(|token| match token.kind.class() {
                    Some(class) => view! { <span class=class>{token.text}</span> }.into_any(),
                    None => token.text.into_any(),
                })
                .collect_view();
            let end = (index < last).then_some("\n");
            view! { {tokens}{end} }
        })
        .collect_view();
    view! {
        <pre class="fui-hl7-raw" role="region" aria-label=label.into() tabindex="0">
            {lines}
        </pre>
    }
    .into_any()
}

/// Returns one segment's block: its id and name over its fields.
fn segment(segment: Hl7Segment) -> AnyView {
    let Hl7Segment { id, name, fields } = segment;
    let fields = fields
        .into_iter()
        .map(
            |Hl7Field {
                 position,
                 name,
                 value,
             }| {
                view! {
                    <div>
                        <dt>{position}</dt>
                        <dd class="fui-hl7-name">{name}</dd>
                        <dd>{value}</dd>
                    </div>
                }
            },
        )
        .collect_view();
    view! {
        <li class="fui-hl7-seg">
            <p class="fui-hl7-seg-head">
                <span class="fui-hl7-seg-id">{id}</span>
                " "
                <span>{name}</span>
            </p>
            <dl class="fui-hl7-fields">{fields}</dl>
        </li>
    }
    .into_any()
}

/// Returns the message as read, one block per segment of `segments`, as a
/// list named `label`.
#[must_use]
pub fn hl7v2_segments(label: impl Into<String>, segments: Vec<Hl7Segment>) -> AnyView {
    let segments = segments.into_iter().map(segment).collect_view();
    view! {
        <ul class="fui-hl7-segs" aria-label=label.into()>
            {segments}
        </ul>
    }
    .into_any()
}
