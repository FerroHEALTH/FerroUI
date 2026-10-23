// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The docked decision bar (R4): the page's one decision, docked to the foot
//! of the main area, in reach while the person scrolls.
//!
//! It spans the main area beside the sidebar, never over it, on the raised
//! ground with one top rule and no shadow or margin of its own; on a phone it
//! docks to the window foot, clear of the safe area. A console draws it as the
//! last part of its main area (the shell's dock). It takes up to two fields,
//! a reason and a path, before the decisions; an editor's bar with no field
//! puts its state line in the field's place. Below 40 rem each field takes
//! its rows and the decisions stack under them at full width, the primary
//! first. Previous and next through a queue are never in the bar: they sit in
//! the page header (R5).
//!
//! Every decision is a submit button of a `POST` form, the bar's own or the
//! page's form it names, so the bar works with no WebAssembly. A disabled
//! decision says why as visible text in the bar, named by its
//! `aria-describedby`, never only in a tooltip. A decision's shortcut is a
//! quiet key cap after its label at wide widths, announced through
//! `aria-keyshortcuts`; the console binds the key. The page keeps a focused
//! control clear of the bar (WCAG 2.2 SC 2.4.11, the stylesheet's scroll
//! padding). No specification governs the bar: our own design.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::button::Variant;
use crate::view::form::{described_by, error_id, help_id};
use crate::view::kbd::{aria_keys, kbd};
use crate::view::notice::{Announce, notice};

/// The form a bar's decisions submit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockForm {
    /// A `POST` form of the bar's own, to the console's route `action`.
    Own(String),
    /// The page's form of this id, such as an editor's, which the bar's
    /// fields and decisions join through their `form` attribute.
    Page(String),
}

/// A field of the bar: the reason, or a path.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DockField {
    /// The field's id.
    pub id: String,
    /// The name it submits as.
    pub name: String,
    /// Its label, in the page's locale.
    pub label: String,
    /// The text it holds.
    pub value: String,
    /// The hint inside the empty field, such as "Why you decided this".
    pub placeholder: String,
    /// The key that moves the focus to it, drawn after the label at wide
    /// widths, such as `r`.
    pub shortcut: Option<String>,
    /// Why the server refused what it holds, drawn under it.
    pub error: Option<String>,
    /// The help line under it, when it shows no error.
    pub help: Option<String>,
    /// Whether it waits, while a decision is submitting or done.
    pub disabled: bool,
    /// Whether it takes the focus as the page arrives: the answer to a
    /// decision submitted without the reason it needs.
    pub focus: bool,
}

/// A decision of the bar: a submit button.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// The name it submits as, such as `decision`.
    pub name: String,
    /// The value it submits, such as `link`.
    pub value: String,
    /// Its label, in the page's locale.
    pub label: String,
    /// How much it stands out: the page's one decision is the primary.
    pub variant: Variant,
    /// The glyph before its label.
    pub glyph: Option<Glyph>,
    /// Its key, drawn after the label at wide widths, such as `l`.
    pub shortcut: Option<String>,
    /// Whether it can be taken now.
    pub state: DecisionState,
    /// Whether it leaves the form without keeping what it holds, such as
    /// Discard: drawn with `formnovalidate`, so a required field left
    /// empty never holds the reader on the page (HTML Living Standard,
    /// form submission, the `formnovalidate` attribute).
    pub leaves: bool,
}

impl Default for Decision {
    fn default() -> Self {
        Self {
            name: String::from("decision"),
            value: String::new(),
            label: String::new(),
            variant: Variant::Secondary,
            glyph: None,
            shortcut: None,
            state: DecisionState::Ready,
            leaves: false,
        }
    }
}

/// Whether a decision can be taken now.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DecisionState {
    /// It can be taken.
    #[default]
    Ready,
    /// It is the decision being submitted: the spinner in its glyph's
    /// place, disabled.
    Busy,
    /// It waits while another decision is submitted or done, disabled; the
    /// bar's line says what happens.
    Held,
    /// It cannot be taken, for the reason it holds, such as "Give a reason
    /// to save": disabled, and the bar says why.
    Blocked(String),
}

/// A link of the bar's line, such as "Open the next candidate".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockLink {
    /// Where it leads.
    pub href: String,
    /// Its words.
    pub words: String,
    /// How much it stands out.
    pub variant: Variant,
}

/// The bar's own line: a state, a confirmation, or the answer that someone
/// decided first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockLine {
    /// A state, such as "The set compiles with this draft"; in a bar with
    /// no field it sits in the field's place.
    State {
        /// The glyph before it.
        glyph: Option<Glyph>,
        /// Its words.
        said: String,
    },
    /// The decision is done: the bar confirms it in words, announced
    /// politely, while the next item loads.
    Done(String),
    /// The item changed under the person (a `409`): what happened, and the
    /// ways on.
    Stale {
        /// What happened, and that the decision was not sent.
        said: String,
        /// The next actions.
        ways: Vec<DockLink>,
    },
}

/// What a docked decision bar holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionBar {
    /// The bar's id; the ids of its reasons start with it.
    pub id: String,
    /// The bar's accessible name, such as "Decision".
    pub label: String,
    /// The form its decisions submit.
    pub form: DockForm,
    /// Its fields, none, one or two: the reason, then a path.
    pub fields: Vec<DockField>,
    /// Its line, if it has one.
    pub line: Option<DockLine>,
    /// Its decisions, in the order they read; the primary goes first on a
    /// phone.
    pub decisions: Vec<Decision>,
}

/// The id of the line that says why the `place`th decision, from 1, of the
/// bar `id` is disabled.
#[must_use]
pub fn reason_id(id: &str, place: u32) -> String {
    format!("{id}-why-{place}")
}

/// Returns the docked decision bar `bar`.
#[must_use]
pub fn decision_bar(bar: DecisionBar) -> AnyView {
    let DecisionBar {
        id,
        label,
        form,
        fields,
        line,
        decisions,
    } = bar;
    let (own, form_id) = match form {
        DockForm::Own(action) => (Some(action), format!("{id}-form")),
        DockForm::Page(form) => (None, form),
    };
    let own = own.map(|action| {
        view! { <form id=form_id.clone() method="post" action=action hidden></form> }
    });
    let reasons: Vec<(String, String)> = decisions
        .iter()
        .zip(1_u32..)
        .filter_map(|(decision, place)| match &decision.state {
            DecisionState::Blocked(reason) => Some((reason_id(&id, place), reason.clone())),
            _ => None,
        })
        .collect();
    let bare = fields.is_empty();
    let (in_row, above) = match line {
        Some(DockLine::State { glyph, said }) if bare => {
            (Some(state_line(glyph, said, None)), None)
        }
        other => (None, other),
    };
    let reason_lines = reasons
        .into_iter()
        .map(|(id, reason)| state_line(Some(Glyph::INFO), reason, Some(id)))
        .collect::<Vec<_>>();
    let (in_row, reason_lines) = match (in_row, bare) {
        (None, true) => {
            let mut lines = reason_lines.into_iter();
            (lines.next(), lines.collect())
        }
        (in_row, _) => (in_row, reason_lines),
    };
    let above = above.map(line_view);
    let fields = fields
        .into_iter()
        .map(|field| field_view(field, &form_id))
        .collect_view();
    let decisions = decisions
        .into_iter()
        .zip(1_u32..)
        .map(|(decision, place)| decision_view(decision, &form_id, reason_id(&id, place)))
        .collect_view();
    view! {
        <div id=id class="fui-dock" role="region" aria-label=label>
            {own}
            {above}
            {reason_lines}
            <div class="fui-dock-row">
                {fields}
                {in_row}
                <div class="fui-dock-actions">{decisions}</div>
            </div>
        </div>
    }
    .into_any()
}

/// Returns a line of the bar's state: `glyph` beside `said`, with the id
/// `id` a disabled decision names.
fn state_line(glyph: Option<Glyph>, said: String, id: Option<String>) -> AnyView {
    view! {
        <p id=id class="fui-dock-message">
            {glyph.map(|glyph| icon(glyph, "16"))}
            <span>{said}</span>
        </p>
    }
    .into_any()
}

/// Returns `line` above the bar's row.
fn line_view(line: DockLine) -> AnyView {
    match line {
        DockLine::State { glyph, said } => state_line(glyph, said, None),
        DockLine::Done(said) => view! {
            <p class="fui-dock-message fui-dock-done" role="status">
                {icon(Glyph::CIRCLE_CHECK, "16")}
                <span>{said}</span>
            </p>
        }
        .into_any(),
        DockLine::Stale { said, ways } => {
            let ways = ways
                .into_iter()
                .map(|way| {
                    view! {
                        " "
                        <a href=way.href class=way.variant.small()>
                            {way.words}
                        </a>
                    }
                })
                .collect_view();
            view! {
                <div class="fui-dock-message">
                    {notice(
                        Tone::Warn,
                        Announce::Alert,
                        None,
                        view! {
                            {said}
                            {ways}
                        },
                    )}
                </div>
            }
            .into_any()
        }
    }
}

/// Returns `field` joined to the form `form`.
fn field_view(field: DockField, form: &str) -> impl IntoView {
    let DockField {
        id,
        name,
        label,
        value,
        placeholder,
        shortcut,
        error,
        help,
        disabled,
        focus,
    } = field;
    let help = help.filter(|_| error.is_none());
    let describe = described_by(&id, help.is_some(), error.is_some());
    let keys = shortcut.as_deref().and_then(aria_keys);
    let cap = shortcut.as_deref().map(kbd);
    let error = error.map(|error| {
        view! {
            <p id=error_id(&id) class="fui-error">
                {icon(Glyph::CIRCLE_ALERT, "14")}
                {error}
            </p>
        }
    });
    let invalid = error.is_some().then_some("true");
    let help = help.map(|help| view! { <p id=help_id(&id) class="fui-help">{help}</p> });
    view! {
        <div class="fui-dock-field">
            <label for=id.clone() class="fui-label">
                {label}
                {cap}
            </label>
            <input
                id=id
                name=name
                form=form.to_owned()
                type="text"
                class="fui-input"
                value=value
                placeholder=placeholder
                aria-invalid=invalid
                aria-describedby=describe
                aria-keyshortcuts=keys
                disabled=disabled
                autofocus=focus
            />
            {error}
            {help}
        </div>
    }
}

/// Returns `decision` as a submit button of the form `form`, its reason
/// line `why` when it is disabled for one.
fn decision_view(decision: Decision, form: &str, why: String) -> impl IntoView {
    let Decision {
        name,
        value,
        label,
        variant,
        glyph,
        shortcut,
        state,
        leaves,
    } = decision;
    let busy = state == DecisionState::Busy;
    let keys = shortcut.as_deref().and_then(aria_keys);
    let cap = shortcut.as_deref().filter(|_| !busy).map(kbd);
    let described = matches!(state, DecisionState::Blocked(_)).then_some(why);
    let held = state != DecisionState::Ready;
    let lead = if busy {
        Some(view! { <span class="fui-spinner-ring" aria-hidden="true"></span> }.into_any())
    } else {
        glyph.map(|glyph| icon(glyph, "16").into_any())
    };
    view! {
        <button
            type="submit"
            form=form.to_owned()
            name=name
            value=value
            class=variant.class()
            aria-keyshortcuts=keys
            aria-describedby=described
            aria-busy=busy.then_some("true")
            disabled=held
            formnovalidate=leaves
        >
            {lead}
            {label}
            {cap}
        </button>
    }
}
