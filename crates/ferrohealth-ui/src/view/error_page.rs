// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The error page every console draws a refused or failed page on (R20).
//!
//! It says what happened in plain words first: the status as one quiet line
//! of its words then its number ("Not allowed · 403"), the title naming the
//! error, then what it means for the reader. A 403 says who you are and the
//! scope or role the page needs, a 404 the address that is not known and
//! what it may mean, a 500 that the console cannot tell and who can fix it.
//! A quiet hint carries the raw scope or key, the way on is one or more
//! links (the first the primary), and the time and the request id sit in a
//! closed disclosure. The page renders in the shell for a signed-in reader
//! and in the centred layout for a signed-out visitor; the console writes
//! the same title as the document's. No specification governs the page:
//! our own design.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::{Variant, link_button};

/// What went wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The reader may not see the page: 403.
    Forbidden,
    /// No page at the address: 404.
    NotFound,
    /// The console failed: 500.
    Fault,
}

impl Status {
    /// The HTTP status code a console answers with.
    #[must_use]
    pub const fn code(self) -> u16 {
        match self {
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Fault => 500,
        }
    }

    /// The glyph beside the status.
    const fn glyph(self) -> Glyph {
        match self {
            Self::Forbidden => Glyph::BAN,
            Self::NotFound => Glyph::COMPASS,
            Self::Fault => Glyph::SERVER_CRASH,
        }
    }

    /// The status in words, then its number: "Not allowed · 403".
    #[must_use]
    pub fn line(self, locale: &Locale) -> Word {
        let words = match self {
            Self::Forbidden => tr(locale, Text::ErrorForbidden),
            Self::NotFound => tr(locale, Text::ErrorNotFound),
            Self::Fault => tr(locale, Text::ErrorFault),
        };
        tr_with(
            locale,
            Text::ErrorStatus,
            &[
                ("words", words.into()),
                ("status", self.code().to_string().into()),
            ],
        )
    }
}

/// One way on from the error: a link's words, where it leads and its glyph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Way {
    /// The link's words, such as "Every mapping file".
    pub label: String,
    /// Where it leads: the one home of the thing (family rule 1).
    pub href: String,
    /// The glyph before the words, when there is one.
    pub glyph: Option<Glyph>,
}

/// The technical detail of a failed request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detail {
    /// When it happened, as the console writes a time.
    pub time: String,
    /// The id of the request, as the server logged it.
    pub request_id: String,
}

/// What an error page shows.
#[derive(Debug)]
pub struct ErrorPage {
    /// What went wrong.
    pub status: Status,
    /// The error in plain words, such as "This view needs the operator
    /// scope": the page's first heading and the document's title.
    pub title: String,
    /// What it means for the reader and what to do.
    pub body: Html,
    /// A quiet line with the raw scope, role or key, when it helps.
    pub hint: Option<Html>,
    /// The ways on, the first the primary; at least one.
    pub ways: Vec<Way>,
    /// The time and the request id, when the console has them.
    pub detail: Option<Detail>,
}

/// Returns the error page (R20): the glyph, the status line, the title,
/// the body and its hint, the ways on, and the detail folded away.
#[must_use]
pub fn error_page(locale: &Locale, page: ErrorPage) -> Html {
    let ErrorPage {
        status,
        title,
        body,
        hint,
        ways,
        detail,
    } = page;
    let ways = ways
        .into_iter()
        .enumerate()
        .map(|(at, way)| {
            let variant = if at == 0 {
                Variant::Primary
            } else {
                Variant::Secondary
            };
            link_button(way.href, variant, way.glyph, way.label)
        })
        .collect();
    PageView {
        glyph: icon(status.glyph(), "24"),
        line: status.line(locale),
        title,
        body,
        hint,
        ways,
        detail,
        detail_words: tr(locale, Text::ErrorDetail),
        time_words: tr(locale, Text::ErrorTime),
        request_words: tr(locale, Text::ErrorRequest),
    }
    .html()
}

#[derive(Template)]
#[template(path = "error_page/page.html")]
struct PageView {
    glyph: Html,
    line: Word,
    title: String,
    body: Html,
    hint: Option<Html>,
    ways: Html,
    detail: Option<Detail>,
    detail_words: Word,
    time_words: Word,
    request_words: Word,
}
