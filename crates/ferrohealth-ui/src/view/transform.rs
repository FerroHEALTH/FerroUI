// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The transform pair (B5) and the parts of an operation's answer.
//!
//! An operation is its options in one row ([`transform_options`]) with Run
//! at the row's end ([`run_button`]), then two flat panes split by a
//! hairline ([`transform_pair`]): the input and the answer, each with a
//! sticky head (its title, its format tags, the answer's state, its own
//! actions). Before the first run one quiet line stands where the answer
//! will be, a run in progress says so, and a refusal sits in the answer
//! pane ([`Answer`]). From 64 rem the panes stand side by side, below it one
//! above the other; on a phone Run spans the row. Each pane's code scrolls
//! under the scroll cue and its head stays.
//!
//! The options row aligns its fields on their label line, so an error
//! under one field moves no other; Run and a group of check boxes
//! ([`field_checks`]) sit at the row's foot. A dense row ([`Density::Dense`])
//! keeps Run on the row with four or more fields, and an id field
//! ([`crate::view::form::Field::holds_id`]) starts at 24 rem, wide enough
//! for a whole uuid. The answer names the request it came from
//! ([`request`]) with Copy, and the raw answer in a disclosure under it;
//! every verdict is a load report ([`crate::view::load_report`]).
//!
//! The console wraps the options and the panes in one form, so Run posts
//! the options and the input with no WebAssembly. No specification governs
//! the pair: our own design.

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::island::CopyButton;
use crate::view::empty::{Busy, busy};
use crate::view::kbd::{aria_keys, kbd};
use crate::view::load_report::Outcome;
use crate::view::surface::disclosure;
use crate::widget::copy::{CopySource, CopyWords};

/// How many fields an options row holds, which sets how they share it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Density {
    /// Up to three fields, each its own width, Run at the row's end.
    #[default]
    Plain,
    /// Four or more fields sharing the row, so Run stays on it at wide
    /// widths (`is-dense`).
    Dense,
}

/// Returns the options row: `fields` with `run` at its end, its fields
/// sharing the row as `density` says.
#[must_use]
pub fn transform_options(
    density: Density,
    fields: impl IntoView + 'static,
    run: impl IntoView + 'static,
) -> AnyView {
    let fields = fields.into_any();
    let run = run.into_any();
    let dense = density == Density::Dense;
    view! {
        <div class="fui-transform-options" class:is-dense=dense>
            {fields}
            {run}
        </div>
    }
    .into_any()
}

/// Returns the check boxes `checks` stacked as one field of an options
/// row, at the row's foot.
#[must_use]
pub fn field_checks(checks: impl IntoView + 'static) -> AnyView {
    let checks = checks.into_any();
    view! { <div class="fui-field fui-field-checks">{checks}</div> }.into_any()
}

/// Whether Run can be pressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunState {
    /// It posts the options and the input.
    Ready,
    /// A run is in progress: it waits, busy and disabled, and says so.
    Running,
    /// It cannot run, for the reason given, such as conversions being off
    /// on the server; the reason is its description.
    Off(String),
}

/// What Run says and does.
#[derive(Debug, Clone)]
pub struct Run {
    /// The button's id.
    pub id: String,
    /// What it does in plain words, such as "Convert to FHIR".
    pub label: String,
    /// What it says while a run is in progress, such as "Converting".
    pub busy: String,
    /// The glyph beside its label.
    pub glyph: Glyph,
    /// The keys that press it, drawn as key caps after the label and
    /// announced through `aria-keyshortcuts`, such as `Ctrl Enter`; the
    /// console binds them.
    pub shortcut: Option<String>,
    /// The operation it calls, its tooltip, such as "Runs $tofhir".
    pub operation: Option<String>,
    /// Whether it can be pressed.
    pub state: RunState,
}

/// Returns Run at the end of an options row: the form's submit button.
#[must_use]
pub fn run_button(run: Run) -> AnyView {
    let Run {
        id,
        label,
        busy,
        glyph,
        shortcut,
        operation,
        state,
    } = run;
    let button = match state {
        RunState::Running => view! {
            <button
                id=id
                type="button"
                class="fui-button fui-button-primary"
                aria-busy="true"
                disabled
            >
                <span class="fui-spinner-ring" aria-hidden="true"></span>
                {busy}
            </button>
        }
        .into_any(),
        RunState::Off(reason) => {
            let why = format!("{id}-why");
            view! {
                <button
                    id=id
                    type="submit"
                    class="fui-button fui-button-primary"
                    title=reason.clone()
                    aria-describedby=why.clone()
                    disabled
                >
                    {icon(glyph, "16")}
                    {label}
                </button>
                <span id=why class="fui-sr-only">
                    {reason}
                </span>
            }
            .into_any()
        }
        RunState::Ready => {
            let keys = shortcut.as_deref().and_then(aria_keys);
            let caps = shortcut.as_deref().map(kbd);
            view! {
                <button
                    id=id
                    type="submit"
                    class="fui-button fui-button-primary"
                    title=operation
                    aria-keyshortcuts=keys
                >
                    {icon(glyph, "16")}
                    {label}
                    {caps}
                </button>
            }
            .into_any()
        }
    };
    view! { <div class="fui-transform-run">{button}</div> }.into_any()
}

/// The answer's state, in its pane's head.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerState {
    /// How the run went: its mark's shape and colour.
    pub outcome: Outcome,
    /// The state in a word or two, with a capital, such as "Converted".
    pub word: String,
    /// What follows it, muted, such as "1 Observation · 84 ms · 200".
    pub detail: Option<String>,
}

/// One pane of a transform pair.
#[derive(Debug)]
pub struct TransformPane {
    /// Its title, a level-three heading, such as "Composition".
    pub title: String,
    /// The format tags after the title, such as "FLAT" or "FHIR R4".
    pub tags: Vec<String>,
    /// The answer's state, in the answer pane once there is an answer.
    pub state: Option<AnswerState>,
    /// The pane's own actions at its head's end, such as Copy.
    pub actions: Option<AnyView>,
    /// What it holds: an editor, a code panel, or an [`Answer`].
    pub body: AnyView,
}

/// What the answer pane holds.
#[derive(Debug)]
pub enum Answer {
    /// Before the first run: the one quiet line, such as "Convert to see
    /// the Bundle."
    Waiting(String),
    /// A run is in progress: the busy mark with its words.
    Running(String),
    /// The answer, such as a code panel.
    Answered(AnyView),
    /// The refusal ([`crate::view::refusal::refusal`]).
    Refused(AnyView),
}

impl Answer {
    /// Returns the answer as a pane's body.
    #[must_use]
    pub fn view(self) -> AnyView {
        match self {
            Self::Waiting(line) => transform_quiet(line),
            Self::Running(words) => {
                view! { <div class="fui-transform-body">{busy(words, Busy::Inline)}</div> }
                    .into_any()
            }
            Self::Answered(body) => body,
            Self::Refused(refusal) => {
                view! { <div class="fui-transform-body">{refusal}</div> }.into_any()
            }
        }
    }
}

/// Returns the one quiet line that stands where an answer will be, before
/// the first run; also under an options row with no panes.
#[must_use]
pub fn transform_quiet(line: impl Into<String>) -> AnyView {
    view! { <p class="fui-transform-empty">{line.into()}</p> }.into_any()
}

/// Returns the answer's state: its mark and word, then its detail.
fn answer_state(state: AnswerState) -> AnyView {
    let AnswerState {
        outcome,
        word,
        detail,
    } = state;
    let tone = outcome.tone().class();
    let mark = format!("fui-state {tone}");
    let dot = format!("fui-dot {tone}");
    let detail = detail.map(|detail| view! { <span class="fui-muted">{detail}</span> });
    view! {
        <span class="fui-transform-status">
            <span class=mark>
                <span class=dot aria-hidden="true"></span>
                {word}
            </span>
            {detail}
        </span>
    }
    .into_any()
}

/// Returns one pane: its sticky head over its body.
fn transform_pane(pane: TransformPane) -> AnyView {
    let TransformPane {
        title,
        tags,
        state,
        actions,
        body,
    } = pane;
    let tags = tags.into_iter().map(crate::view::tag::tag).collect_view();
    let state = state.map(answer_state);
    let actions = actions.map(|actions| view! { <div class="fui-actions">{actions}</div> });
    view! {
        <section class="fui-transform-pane">
            <div class="fui-transform-head">
                <h3 class="fui-transform-title">{title}</h3>
                {tags}
                {state}
                {actions}
            </div>
            {body}
        </section>
    }
    .into_any()
}

/// Returns the pair: `input` beside `answer` from 64 rem, a hairline
/// between them, and one above the other below it.
#[must_use]
pub fn transform_pair(input: TransformPane, answer: TransformPane) -> AnyView {
    let input = transform_pane(input);
    let answer = transform_pane(answer);
    view! { <div class="fui-transform">{input} {answer}</div> }.into_any()
}

/// The request an answer came from.
#[derive(Debug)]
pub struct Request {
    /// The request as sent, such as `GET …/CodeSystem/$lookup?code=1`.
    pub text: String,
    /// The id of its Copy button, which copies the text; `None` draws none.
    pub copy: Option<String>,
    /// The raw answer under it: the disclosure's summary and its body, such
    /// as a code viewer.
    pub raw: Option<(String, AnyView)>,
}

/// Returns `text` with a break opportunity after each slash and each
/// ampersand, so a long request breaks there and never inside a name.
fn breakable(text: &str) -> AnyView {
    let mut out = Vec::new();
    let mut part = String::new();
    for c in text.chars() {
        part.push(c);
        if matches!(c, '/' | '&') {
            let done = std::mem::take(&mut part);
            out.push(view! { {done}<wbr /> }.into_any());
        }
    }
    if !part.is_empty() {
        out.push(part.into_any());
    }
    out.collect_view().into_any()
}

/// Returns the request an answer came from, in the code face, with its
/// Copy button and the raw answer in a disclosure under it.
#[must_use]
pub fn request(request: Request) -> AnyView {
    let Request { text, copy, raw } = request;
    let copy = copy.map(|id| {
        view! {
            <CopyButton
                id=id
                label=tr(Text::DocumentCopy)
                source=CopySource::Text(text.clone())
                words=CopyWords::of_page()
            />
        }
    });
    let line = breakable(&text);
    let raw = raw.map(|(summary, body)| disclosure(summary, body));
    view! {
        <div class="fui-request">
            <p class="fui-alert-request">
                <code>{line}</code>
            </p>
            {copy}
        </div>
        {raw}
    }
    .into_any()
}
