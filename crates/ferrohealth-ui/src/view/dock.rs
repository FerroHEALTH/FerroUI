// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The docked decision bar (R4): the page's one decision, docked to the foot
//! of the main area, in reach while the person scrolls.
//!
//! It spans the main area beside the sidebar, never over it, on the raised
//! ground with one top rule and no shadow or margin of its own; on a phone it
//! docks to the window foot, clear of the safe area. A console draws it as the
//! last part of its main area (the shell's dock) or of the main area's
//! column, the two places the stylesheet looks for it. It takes up to two
//! fields, a reason and a path, before the decisions; an editor's bar with no
//! field puts its state line in the field's place. Below 40 rem each field takes
//! its rows and the decisions stack under them at full width, the primary
//! first. Previous and next through a queue are never in the bar: they sit in
//! the page header (R5).
//!
//! Every decision is a submit button of a `POST` form, the bar's own or the
//! page's form it names, so the bar works with no script. A disabled
//! decision says why as visible text in the bar, named by its
//! `aria-describedby`, never only in a tooltip. A decision's shortcut is a
//! quiet key cap after its label at wide widths, announced through
//! `aria-keyshortcuts`, and the `keys` module binds it from 40 rem up: a
//! single key such as `l` presses the decision outside a field, a chord
//! such as `Ctrl s` from inside one too, and a field's key such as `r`
//! moves the focus to it ([`crate::view::shortcuts`]). The page keeps a
//! focused control clear of the bar (WCAG 2.2 SC 2.4.11, the stylesheet's
//! scroll padding). No specification governs the bar: our own design.

use askama::Template;

use crate::html::{Html, Render as _};
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
    /// widths and bound by the `keys` module, such as `r`.
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
    /// Its key, drawn after the label at wide widths and bound by the
    /// `keys` module, such as `l` or `Ctrl s`.
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

struct FieldView {
    id: String,
    name: String,
    label: String,
    cap: Html,
    value: String,
    placeholder: String,
    error: Option<String>,
    error_id: String,
    help: Option<String>,
    help_id: String,
    describe: Option<String>,
    keys: Option<String>,
    disabled: bool,
    focus: bool,
}

struct DecisionView {
    name: String,
    value: String,
    keys: Option<String>,
    described: Option<String>,
    busy: bool,
    held: bool,
    leaves: bool,
    class: &'static str,
    glyph: Html,
    label: String,
    cap: Html,
}

#[derive(Template)]
#[template(path = "dock/bar.html", block = "bar")]
struct BarView {
    id: String,
    label: String,
    own: Option<String>,
    form_id: String,
    above: Html,
    reason_lines: Html,
    fields: Vec<FieldView>,
    alert: Html,
    in_row: Html,
    decisions: Vec<DecisionView>,
}

/// Returns the docked decision bar `bar`.
#[must_use]
pub fn decision_bar(bar: DecisionBar) -> Html {
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
            (Some(state_line(glyph, &said, None)), None)
        }
        other => (None, other),
    };
    let reason_lines = reasons
        .into_iter()
        .map(|(id, reason)| state_line(Some(Glyph::INFO), &reason, Some(id)))
        .collect::<Vec<_>>();
    let (in_row, reason_lines) = match (in_row, bare) {
        (None, true) => {
            let mut lines = reason_lines.into_iter();
            (lines.next(), lines.collect())
        }
        (in_row, _) => (in_row, reason_lines),
    };
    let fields = fields.into_iter().map(field_view).collect();
    let decisions = decisions
        .into_iter()
        .zip(1_u32..)
        .map(|(decision, place)| decision_view(decision, reason_id(&id, place)))
        .collect();
    BarView {
        id,
        label,
        own,
        form_id,
        above: above.map(line_view).unwrap_or_default(),
        reason_lines: Html::join(reason_lines),
        fields,
        alert: icon(Glyph::CIRCLE_ALERT, "14"),
        in_row: in_row.unwrap_or_default(),
        decisions,
    }
    .html()
}

#[derive(Template)]
#[template(path = "dock/bar.html", block = "state_line")]
struct StateLine<'a> {
    id: Option<String>,
    glyph: Html,
    said: &'a str,
}

/// Returns a line of the bar's state: `glyph` beside `said`, with the id
/// `id` a disabled decision names.
fn state_line(glyph: Option<Glyph>, said: &str, id: Option<String>) -> Html {
    StateLine {
        id,
        glyph: glyph.map(|glyph| icon(glyph, "16")).unwrap_or_default(),
        said,
    }
    .html()
}

#[derive(Template)]
#[template(path = "dock/bar.html", block = "done")]
struct DoneLine {
    glyph: Html,
    said: String,
}

#[derive(Template)]
#[template(path = "dock/bar.html", block = "stale")]
struct StaleLine {
    said: String,
    ways: Vec<DockLink>,
}

/// Returns `line` above the bar's row.
fn line_view(line: DockLine) -> Html {
    match line {
        DockLine::State { glyph, said } => state_line(glyph, &said, None),
        DockLine::Done(said) => DoneLine {
            glyph: icon(Glyph::CIRCLE_CHECK, "16"),
            said,
        }
        .html(),
        DockLine::Stale { said, ways } => Html::raw(format!(
            "<div class=\"fui-dock-message\">{}</div>",
            notice(
                Tone::Warn,
                Announce::Alert,
                None,
                StaleLine { said, ways }.html()
            )
        )),
    }
}

/// Returns `field` as the template writes it.
fn field_view(field: DockField) -> FieldView {
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
    FieldView {
        describe: described_by(&id, help.is_some(), error.is_some()),
        keys: shortcut.as_deref().and_then(aria_keys),
        cap: shortcut.as_deref().map(kbd).unwrap_or_default(),
        error_id: error_id(&id),
        help_id: help_id(&id),
        id,
        name,
        label,
        value,
        placeholder,
        error,
        help,
        disabled,
        focus,
    }
}

/// Returns `decision` as the template writes it, its reason line `why`
/// when it is disabled for one.
fn decision_view(decision: Decision, why: String) -> DecisionView {
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
    DecisionView {
        name,
        value,
        keys: shortcut.as_deref().and_then(aria_keys),
        described: matches!(state, DecisionState::Blocked(_)).then_some(why),
        busy,
        held: state != DecisionState::Ready,
        leaves,
        class: variant.class(),
        glyph: glyph.map(|glyph| icon(glyph, "16")).unwrap_or_default(),
        label,
        cap: shortcut
            .as_deref()
            .filter(|_| !busy)
            .map(kbd)
            .unwrap_or_default(),
    }
}
