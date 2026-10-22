// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The value set clause (M4): one compose clause of a value set, an include
//! or an exclude.
//!
//! A clause is its mark, its source as the title, a plain sentence of what
//! it selects, then its body: the filter rows (F7 rows of
//! [`crate::widget::repeat`]) and its codes, each an M2 coding. Include is a
//! solid accent edge and Exclude a dashed warn edge, with the word beside,
//! so the kind never rests on colour (WCAG 2.2 SC 1.4.1). An invariant the
//! clause breaks is named by its own number inline. A clause is read-only
//! on a value set's page and editable in a composer, where its body holds
//! the fields; clauses run edge to edge in their section (B10). No
//! specification governs the layout: our own design.

use leptos::prelude::*;

use crate::i18n::{Arg, Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::coding::{Coding, Layout, coding};
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
    fn word(self) -> String {
        tr(match self {
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
    pub title: AnyView,
    /// The source in plain words, for the clause's accessible name.
    pub source: String,
    /// The plain sentence of what it selects, which may hold a code or a
    /// coding.
    pub sentence: AnyView,
    /// The filter rows, the codes and, in a composer, the fields; `None`
    /// for a clause the sentence says whole.
    pub body: Option<AnyView>,
    /// The actions at the end of its head, such as "Remove this clause".
    pub actions: Option<AnyView>,
    /// The invariant it breaks, if any.
    pub issue: Option<ClauseIssue>,
}

/// One code of a clause: its coding and, in a composer, its Remove.
#[derive(Debug)]
pub struct ClauseCode {
    /// The code.
    pub coding: Coding,
    /// The button that removes it, in a composer.
    pub remove: Option<AnyView>,
}

/// Returns the clause `clause`.
#[must_use]
pub fn clause(clause: Clause) -> AnyView {
    let Clause {
        kind,
        title,
        source,
        sentence,
        body,
        actions,
        issue,
    } = clause;
    let label = tr_with(
        match kind {
            ClauseKind::Include => Text::ClauseIncludeLabel,
            ClauseKind::Exclude => Text::ClauseExcludeLabel,
        },
        &[("title", Arg::from(source))],
    );
    let word = kind.word();
    let mark = badge(kind.tone(), &word);
    let actions = actions.map(|actions| view! { <div class="fui-actions">{actions}</div> });
    let issue = issue.map(|ClauseIssue { key, text }| {
        view! {
            <p class="fui-clause-issue" role="note">
                {icon(Glyph::CIRCLE_X, "14")}
                <span>
                    <code>{key}</code>
                    {text}
                </span>
            </p>
        }
    });
    view! {
        <section class="fui-clause" class:is-exclude=kind == ClauseKind::Exclude aria-label=label>
            <div class="fui-clause-head">
                {mark} <h3 class="fui-clause-title">{title}</h3> {actions}
            </div>
            <p class="fui-clause-sentence">{sentence}</p>
            {body}
            {issue}
        </section>
    }
    .into_any()
}

/// Returns the clauses of a value set, edge to edge in their section with
/// a hairline between them.
#[must_use]
pub fn clauses(clauses: impl IntoView + 'static) -> AnyView {
    let clauses = clauses.into_any();
    view! { <div class="fui-clauses">{clauses}</div> }.into_any()
}

/// Returns the small heading over a part of a clause's body, such as
/// "Filters" or "Codes".
#[must_use]
pub fn clause_part(heading: impl Into<String>) -> AnyView {
    view! { <p class="fui-clause-sub">{heading.into()}</p> }.into_any()
}

/// Returns two fields of a clause side by side, such as its code system
/// and version, stacked on a phone.
#[must_use]
pub fn clause_fields(fields: impl IntoView + 'static) -> AnyView {
    let fields = fields.into_any();
    view! { <div class="fui-clause-grid">{fields}</div> }.into_any()
}

/// Returns the codes of a clause, one per line, each an inline M2 coding
/// with its Remove at the end in a composer.
#[must_use]
pub fn clause_codes(codes: Vec<ClauseCode>) -> AnyView {
    let items = codes
        .into_iter()
        .map(
            |ClauseCode {
                 coding: each,
                 remove,
             }| {
                let shown = coding(each, Layout::Inline);
                view! { <li>{shown} {remove}</li> }
            },
        )
        .collect_view();
    view! { <ul class="fui-clause-codes">{items}</ul> }.into_any()
}
