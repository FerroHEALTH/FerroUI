// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The empty state, the busy mark and the skeleton that stands in for
//! content a page is still fetching.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::icon::{Glyph, icon};

#[derive(Template)]
#[template(
    source = r#"<div class="fui-empty{% if quiet %} fui-empty-quiet{% endif %}"><span class="fui-empty-glyph">{{ glyph }}</span><h2>{{ title }}</h2><p class="fui-empty-body">{{ body }}</p>{{ action }}</div>"#,
    ext = "html"
)]
struct Empty {
    quiet: bool,
    glyph: Html,
    title: String,
    body: String,
    action: Html,
}

/// Returns an empty state: `glyph` in an accent round, `title` under a
/// level-two heading, the sentence `body`, and `action` when there is one.
#[must_use]
pub fn empty_state(
    glyph: Glyph,
    title: impl Into<String>,
    body: impl Into<String>,
    action: Html,
) -> Html {
    Empty {
        quiet: false,
        glyph: icon(glyph, "24"),
        title: title.into(),
        body: body.into(),
        action,
    }
    .html()
}

/// Returns a quiet empty state for a section of a page with nothing in it:
/// `glyph`, `title` under a level-two heading and the one `line`, with no
/// frame, inside the card that holds it.
#[must_use]
pub fn empty_quiet(glyph: Glyph, title: impl Into<String>, line: impl Into<String>) -> Html {
    Empty {
        quiet: true,
        glyph: icon(glyph, "20"),
        title: title.into(),
        body: line.into(),
        action: Html::empty(),
    }
    .html()
}

/// Returns the busy indicator a section shows while its answer arrives: a
/// turning ring beside the words `label`, announced politely; the ring
/// stands still for a reader who asks for less motion.
#[must_use]
pub fn spinner(label: impl Into<String>) -> Html {
    busy(label, Busy::Inline)
}

/// Where a busy mark sits, which sets its size and whether its words show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Busy {
    /// Inside a line of text, its words beside the ring.
    Inline,
    /// Under a section heading, a step larger, its words beside the ring.
    Section,
    /// Beside a control that is waiting, such as a sign-in button that
    /// posts: the ring alone in sight, its words read by a screen reader.
    Control,
}

#[derive(Template)]
#[template(
    source = r#"<span class="{{ class }}" role="status"><span class="fui-spinner-ring" aria-hidden="true"></span><span class="{{ words }}">{{ label }}</span></span>"#,
    ext = "html"
)]
struct Spinner {
    class: &'static str,
    words: &'static str,
    label: String,
}

/// Returns the busy mark for something waiting on the server.
///
/// It is a turning ring and the sentence `label` that says what it waits
/// for, in a polite status region. The ring stands still under
/// `prefers-reduced-motion`.
#[must_use]
pub fn busy(label: impl Into<String>, at: Busy) -> Html {
    let (class, words) = match at {
        Busy::Inline => ("fui-spinner", "fui-spinner-text"),
        Busy::Section => ("fui-spinner fui-spinner-section", "fui-spinner-text"),
        Busy::Control => ("fui-spinner", "fui-sr-only"),
    };
    Spinner {
        class,
        words,
        label: label.into(),
    }
    .html()
}

#[derive(Template)]
#[template(
    source = r#"<div aria-busy="true"><span class="fui-sr-only" role="status">{{ label }}</span>{% for _ in 0..lines %}<span class="fui-skeleton"></span>{% endfor %}</div>"#,
    ext = "html"
)]
struct Skeleton {
    lines: u8,
    label: String,
}

/// Returns `lines` skeleton bars standing in for content that is loading,
/// announced as busy, with the sentence `label` for a screen reader.
#[must_use]
pub fn skeleton(lines: u8, label: impl Into<String>) -> Html {
    Skeleton {
        lines,
        label: label.into(),
    }
    .html()
}
