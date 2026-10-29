// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The value set clause (M4): one compose clause of a value set, an include
//! or an exclude.
//!
//! A clause is its mark, its source as the title, a plain sentence of what
//! it selects, then its body: the filter rows (F7 rows of
//! [`crate::view::repeat`]) and its codes, each an M2 coding. Include is a
//! solid accent edge and Exclude a dashed warn edge, with the word beside,
//! so the kind never rests on colour (WCAG 2.2 SC 1.4.1). Each invariant
//! the clause breaks is named by its own number inline, two or more as a
//! list. A list of codes longer than a page ends in the more row
//! ([`MoreCodes`]), which the console's server answers. A clause is read-only
//! on a value set's page and editable in a composer, where its body holds
//! the fields; clauses run edge to edge in their section (B10). No
//! specification governs the layout: our own design.

use crate::html::Html;
use crate::i18n::{Arg, Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::coding::{Coding, Layout, MoreCodes, coding, more_codes};
use crate::view::status::badge;

/// Whether a clause adds codes or takes them back out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClauseKind {
    /// It adds codes: `ValueSet.compose.include`.
    Include,
    /// It takes codes back out: `ValueSet.compose.exclude`.
    Exclude,
}

impl ClauseKind {
    /// The mark's word in the page's locale.
    fn word(self, locale: &Locale) -> Word {
        tr(locale, match self {
            Self::Include => Text::ClauseInclude,
            Self::Exclude => Text::ClauseExclude,
        })
    }

    /// The tone of the mark: the accent for an include, warn for an
    /// exclude.
    const fn tone(self) -> Tone {
        match self {
            Self::Include => Tone::Accent,
            Self::Exclude => Tone::Warn,
        }
    }
}

/// An invariant a clause breaks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClauseIssue {
    /// The invariant's own number, such as `vsd-3`.
    pub key: String,
    /// What is wrong and what to do, in a sentence.
    pub text: String,
}

/// What a clause shows.
#[derive(Debug)]
pub struct Clause {
    /// Whether it includes or excludes.
    pub kind: ClauseKind,
    /// Its source as the title, which may hold a link to the code system.
    pub title: Html,
    /// The source in plain words, for the clause's accessible name.
    pub source: String,
    /// The plain sentence of what it selects, which may hold a code or a
    /// coding.
    pub sentence: Html,
    /// The filter rows, the codes and, in a composer, the fields; `None`
    /// for a clause the sentence says whole.
    pub body: Option<Html>,
    /// The actions at the end of its head, such as "Remove this clause".
    pub actions: Option<Html>,
    /// The invariants it breaks, in the order the console checks them.
    pub issues: Vec<ClauseIssue>,
}

/// One code of a clause: its coding and, in a composer, its Remove.
#[derive(Debug)]
pub struct ClauseCode {
    /// The code.
    pub coding: Coding,
    /// The button that removes it, in a composer.
    pub remove: Option<Html>,
}

/// Returns the clause `clause`.
#[must_use]
pub fn clause(locale: &Locale, clause: Clause) -> Html {
    let Clause {
        kind,
        title,
        source,
        sentence,
        body,
        actions,
        issues,
    } = clause;
    let label = tr_with(
        locale,
        match kind {
            ClauseKind::Include => Text::ClauseIncludeLabel,
            ClauseKind::Exclude => Text::ClauseExcludeLabel,
        },
        &[("title", Arg::from(source))],
    );
    let mark = badge(kind.tone(), kind.word(locale));
    let actions = actions
        .map(|actions| format!("<div class=\"fui-actions\">{actions}</div>"))
        .unwrap_or_default();
    let exclude = if kind == ClauseKind::Exclude {
        " is-exclude"
    } else {
        ""
    };
    Html::raw(format!(
        "<section aria-label=\"{}\" class=\"fui-clause{exclude}\"><div class=\"fui-clause-head\">{mark}<h3 class=\"fui-clause-title\">{title}</h3>{actions}</div><p class=\"fui-clause-sentence\">{sentence}</p>{}{}</section>",
        Html::text(label.text()),
        body.unwrap_or_default(),
        clause_issues(issues)
    ))
}

/// Returns the invariants `issues` a clause breaks: one as a note, two or
/// more as a list of them, or nothing.
fn clause_issues(issues: Vec<ClauseIssue>) -> String {
    let said = |ClauseIssue { key, text }: ClauseIssue| {
        format!(
            "{}<span><code>{}</code>{}</span>",
            icon(Glyph::CIRCLE_X, "14"),
            Html::text(key),
            Html::text(text)
        )
    };
    let mut issues = issues.into_iter();
    let Some(first) = issues.next() else {
        return String::new();
    };
    let Some(second) = issues.next() else {
        return format!(
            "<p role=\"note\" class=\"fui-clause-issue\">{}</p>",
            said(first)
        );
    };
    let items: String = [first, second]
        .into_iter()
        .chain(issues)
        .map(|issue| format!("<li class=\"fui-clause-issue\">{}</li>", said(issue)))
        .collect::<Vec<_>>()
        .concat();
    format!("<ul class=\"fui-clause-issues\">{items}</ul>")
}

/// Returns the clauses of a value set, edge to edge in their section with
/// a hairline between them.
#[must_use]
pub fn clauses(clauses: Html) -> Html {
    let clauses = String::from(clauses);
    Html::raw(format!("<div class=\"fui-clauses\">{clauses}</div>"))
}

/// Returns the small heading over a part of a clause's body, such as
/// "Filters" or "Codes".
#[must_use]
pub fn clause_part(heading: impl Into<String>) -> Html {
    Html::raw(format!(
        "<p class=\"fui-clause-sub\">{}</p>",
        Html::text(heading.into())
    ))
}

/// Returns two fields of a clause side by side, such as its code system
/// and version, stacked on a phone.
#[must_use]
pub fn clause_fields(fields: Html) -> Html {
    let fields = String::from(fields);
    Html::raw(format!("<div class=\"fui-clause-grid\">{fields}</div>"))
}

/// Returns the codes of a clause, one per line, each an inline M2 coding
/// with its Remove at the end in a composer, ending in the more row `more`
/// when the console's server pages them.
#[must_use]
pub fn clause_codes(locale: &Locale, codes: Vec<ClauseCode>, more: Option<MoreCodes>) -> Html {
    let items: String = codes
        .into_iter()
        .map(
            |ClauseCode {
                 coding: each,
                 remove,
             }| {
                format!(
                    "<li>{}{}</li>",
                    coding(locale, each, Layout::Inline),
                    remove.unwrap_or_default()
                )
            },
        )
        .collect::<Vec<_>>()
        .concat();
    let more = more.map(more_codes).unwrap_or_default();
    Html::raw(format!("<ul class=\"fui-clause-codes\">{items}{more}</ul>"))
}
