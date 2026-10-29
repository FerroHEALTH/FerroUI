// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The notice: a warning, an information, a state or a refusal inline in a
//! section, beside what it is about, in any tone.
//!
//! It is the alert's shape in the tone it is given, the tone's glyph beside
//! the words, so the meaning never rides on colour alone. A state is
//! `role="status"`, read politely; a refusal is `role="alert"` (WAI-ARIA
//! 1.2, the `status` and `alert` roles). Line breaks in the words are kept,
//! since an upstream diagnostic carries one violation per line.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;

/// How a notice is announced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Announce {
    /// A state, read when the reader is free (`role="status"`).
    #[default]
    Status,
    /// A refusal or a fault, read at once (`role="alert"`).
    Alert,
}

impl Announce {
    /// The role the notice carries.
    #[must_use]
    pub const fn role(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Alert => "alert",
        }
    }
}

/// The glyph a tone's notice is drawn with.
#[must_use]
pub const fn glyph(tone: Tone) -> Glyph {
    match tone {
        Tone::Ok => Glyph::CIRCLE_CHECK,
        Tone::Warn => Glyph::TRIANGLE_ALERT,
        Tone::Bad => Glyph::CIRCLE_ALERT,
        Tone::Info | Tone::Neutral | Tone::Accent => Glyph::INFO,
    }
}

#[derive(Template)]
#[template(
    source = r#"<div role="{{ role }}"{% if let Some(id) = id %} id="{{ id }}"{% endif %} class="fui-notice {{ tone }}">{{ glyph }}<div>{{ body }}</div></div>"#,
    ext = "html"
)]
struct Notice {
    role: &'static str,
    id: Option<&'static str>,
    tone: &'static str,
    glyph: Html,
    body: Html,
}

/// Returns a notice in `tone`, announced as `announce`, holding `body`
/// (words, and a link when there is a way forward), with the stable `id`
/// when one is given.
#[must_use]
pub fn notice(tone: Tone, announce: Announce, id: Option<&'static str>, body: Html) -> Html {
    Notice {
        role: announce.role(),
        id,
        tone: tone.class(),
        glyph: icon(glyph(tone), "16"),
        body,
    }
    .html()
}

/// Returns [`notice`] holding the plain `words`.
#[must_use]
pub fn notice_text(tone: Tone, announce: Announce, words: impl AsRef<str>) -> Html {
    notice(tone, announce, None, Html::text(words))
}
