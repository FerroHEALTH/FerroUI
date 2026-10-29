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

use crate::highlight::{self, Language};
use crate::html::Html;

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
pub fn hl7v2_raw(label: impl Into<String>, message: &str) -> Html {
    let text = message.replace("\r\n", "\n").replace('\r', "\n");
    let text = text.strip_suffix('\n').unwrap_or(&text);
    let lines: Vec<String> = highlight::lines(Language::Hl7v2, text)
        .iter()
        .map(|tokens| highlight::line_html(tokens))
        .collect();
    Html::raw(format!(
        "<pre role=\"region\" aria-label=\"{}\" tabindex=\"0\" class=\"fui-hl7-raw\">{}</pre>",
        Html::text(label.into()),
        lines.join("\n")
    ))
}

/// Returns one segment's block: its id and name over its fields.
fn segment(segment: Hl7Segment) -> Html {
    let Hl7Segment { id, name, fields } = segment;
    let fields: String = fields
        .into_iter()
        .map(
            |Hl7Field {
                 position,
                 name,
                 value,
             }| {
                format!(
                    "<div><dt>{}</dt><dd class=\"fui-hl7-name\">{}</dd><dd>{}</dd></div>",
                    Html::text(position),
                    Html::text(name),
                    Html::text(value)
                )
            },
        )
        .collect::<Vec<_>>()
        .concat();
    Html::raw(format!(
        "<li class=\"fui-hl7-seg\"><p class=\"fui-hl7-seg-head\"><span class=\"fui-hl7-seg-id\">{}</span> <span>{}</span></p><dl class=\"fui-hl7-fields\">{fields}</dl></li>",
        Html::text(id),
        Html::text(name)
    ))
}

/// Returns the message as read, one block per segment of `segments`, as a
/// list named `label`.
#[must_use]
pub fn hl7v2_segments(label: impl Into<String>, segments: Vec<Hl7Segment>) -> Html {
    let segments: Html = segments.into_iter().map(segment).collect();
    Html::raw(format!(
        "<ul aria-label=\"{}\" class=\"fui-hl7-segs\">{segments}</ul>",
        Html::text(label.into())
    ))
}
