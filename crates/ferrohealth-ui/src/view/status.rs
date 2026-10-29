// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The status marks: the pill, the badge, the status indicator and the
//! banner. Each is a word first and a tint second (WCAG 2.2 SC 1.4.1).

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::Word;
use crate::icon::{Glyph, icon};
use crate::tone::Tone;

#[derive(Template)]
#[template(source = r#"<span class="{{ class }}">{{ word }}</span>"#, ext = "html")]
struct Marked {
    class: &'static str,
    word: Word,
}

/// Returns a status pill: a dot and `word` on the ground of `tone`.
#[must_use]
pub fn pill(tone: Tone, word: &str) -> Html {
    Marked {
        class: tone.pill(),
        word: Word::new(word),
    }
    .html()
}

/// Returns a badge: `word` on the ground of `tone`, with no dot.
#[must_use]
pub fn badge(tone: Tone, word: impl Into<Word>) -> Html {
    Marked {
        class: tone.badge(),
        word: word.into(),
    }
    .html()
}

#[derive(Template)]
#[template(
    source = r#"<span class="fui-status {{ tone }}">{{ glyph }}{{ word }}{% if let Some(detail) = detail %}<span class="fui-status-detail">{{ detail }}</span>{% endif %}</span>"#,
    ext = "html"
)]
struct Status {
    tone: &'static str,
    glyph: Html,
    word: String,
    detail: Option<String>,
}

/// Returns a status indicator: `glyph` beside `word`, in `tone`, with a
/// muted `detail` after it when there is one.
#[must_use]
pub fn status(tone: Tone, glyph: Glyph, word: &str, detail: Option<String>) -> Html {
    Status {
        tone: tone.class(),
        glyph: icon(glyph, "14"),
        word: word.to_owned(),
        detail,
    }
    .html()
}

/// The link a banner ends with, after its sentence: where the reader
/// learns more, such as "Details".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerLink {
    /// Its words.
    pub words: String,
    /// Where it leads.
    pub href: String,
}

#[derive(Template)]
#[template(
    source = r#"<div role="note" aria-label="{{ title }}" class="fui-banner {{ tone }}">{{ glyph }}<p><strong>{{ title }}: </strong>{{ body }}{% if let Some(link) = link %} · <a href="{{ link.href }}">{{ link.words }}</a>{% endif %}</p></div>"#,
    ext = "html"
)]
struct Banner {
    tone: &'static str,
    glyph: Html,
    title: String,
    body: String,
    link: Option<BannerLink>,
}

/// Returns a banner across the page in `tone`: `glyph`, the bold `title`
/// and the sentence `body`, then `link` when there is one.
///
/// A sandbox's banner says it must hold no real patient data, and its
/// link after a middle dot leads to where the open safeguards are named.
#[must_use]
pub fn banner(
    tone: Tone,
    glyph: Glyph,
    title: impl Into<String>,
    body: &str,
    link: Option<BannerLink>,
) -> Html {
    Banner {
        tone: tone.class(),
        glyph: icon(glyph, "16"),
        title: title.into(),
        body: body.to_owned(),
        link,
    }
    .html()
}

/// The state of the product a console reaches, as the top bar's status pill
/// shows it: a shape, a colour and a word together (family rule 12, F12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// It answered: a filled dot.
    Up,
    /// It answered with a fault: a diamond.
    Degraded,
    /// Its state is not known: a diamond.
    Unknown,
    /// It did not answer: a cross.
    Unreachable,
    /// It answered and refused the console, such as a `401`: a cross.
    Refused,
    /// The console does not probe it: a hollow ring.
    NotProbed,
}

impl Reach {
    /// Every state, in the order the gallery shows them.
    pub const ALL: [Self; 6] = [
        Self::Up,
        Self::Degraded,
        Self::Unknown,
        Self::Unreachable,
        Self::Refused,
        Self::NotProbed,
    ];

    /// The tone the state is drawn in, whose pill draws its shape.
    #[must_use]
    pub const fn tone(self) -> Tone {
        match self {
            Self::Up => Tone::Ok,
            Self::Degraded | Self::Unknown => Tone::Warn,
            Self::Unreachable | Self::Refused => Tone::Bad,
            Self::NotProbed => Tone::Neutral,
        }
    }
}

/// What the top bar's status pill says.
#[derive(Debug, Clone)]
pub struct StatusPill {
    /// The product's state.
    pub reach: Reach,
    /// The state in two words with a capital, such as "Bridge up" or
    /// "Bridge refused 401".
    pub words: String,
    /// What the tooltip adds after the words, such as the version and the
    /// address the console reaches; never drawn in the pill itself.
    pub detail: Option<String>,
}

#[derive(Template)]
#[template(
    source = r#"<span class="{{ class }} fui-topbar-status" title="{{ tooltip }}"><span class="fui-roomy-text">{{ words }}</span></span>"#,
    ext = "html"
)]
struct TopbarStatus {
    class: &'static str,
    tooltip: String,
    words: String,
}

/// Returns the top bar's status pill: the state's shape and its words, with
/// the words and the detail in its tooltip.
///
/// From 64 rem up the words show beside the shape. Below it the pill keeps
/// its shape alone, and the words stay in the page, visually hidden, as its
/// accessible name and in its tooltip (F12).
#[must_use]
pub fn status_pill(pill: StatusPill) -> Html {
    let tooltip = match pill.detail {
        Some(detail) => format!("{} · {detail}", pill.words),
        None => pill.words.clone(),
    };
    TopbarStatus {
        class: pill.reach.tone().pill(),
        tooltip,
        words: pill.words,
    }
    .html()
}
