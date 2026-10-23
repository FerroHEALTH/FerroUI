// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page's long form and the docked decision bar it saves through.
//!
//! The bar is the page's one decision, at the foot of the main area; its
//! decisions submit the long form, a `GET` form back to the page, so a save
//! shows its outcome with no WebAssembly. The bar shows when the address asks
//! for it, so the page's other cards keep the whole window: its `dock`
//! query picks a state for the screenshots (`idle`, `missing`, `disabled`,
//! `submitting`, `done`, `stale`, `bare` with no field, `path` with two),
//! and after a submit the state follows what was submitted: a save with a
//! reason is done, a save without one asks for it.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::dock::{
    Decision, DecisionBar, DecisionState, DockField, DockForm, DockLine, DockLink, decision_bar,
};
use ferrohealth_ui::view::form::{FORM, INPUT, keyed_field};
use ferrohealth_ui::view::surface::card;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::data::NEW_PATH;
use crate::locale::{Msg, tr, tr_with};

/// The id of the long form the bar saves.
const FORM_ID: &str = "long-form";

/// The state the bar is drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// A reason given, every decision ready.
    Idle,
    /// A save without a reason: the field takes the focus with its error.
    Missing,
    /// No reason yet, and Save says why it waits.
    Disabled,
    /// The save is on its way.
    Submitting,
    /// The save is done.
    Done,
    /// Someone saved first.
    Stale,
    /// An editor's bar with no field: its state line in the field's place.
    Bare,
    /// A reason and a path.
    Path,
}

/// The state the address asks for: its `dock` query, or what its form
/// submitted; `None` when it asks for neither, and the page draws no bar.
fn asked() -> Option<State> {
    use_query_map().with_untracked(|query| {
        let state = match query.get("dock").as_deref() {
            Some("idle") => State::Idle,
            Some("missing") => State::Missing,
            Some("disabled") => State::Disabled,
            Some("submitting") => State::Submitting,
            Some("done") => State::Done,
            Some("stale") => State::Stale,
            Some("bare") => State::Bare,
            Some("path") => State::Path,
            _ => match (query.get("decision").as_deref(), query.get("reason")) {
                (Some("save"), Some(reason)) if !reason.trim().is_empty() => State::Done,
                (Some("save"), _) => State::Missing,
                (Some(_), _) => State::Idle,
                (None, _) => return None,
            },
        };
        Some(state)
    })
}

/// A long form whose actions are the page's docked bar: a control the
/// keyboard reaches is scrolled clear of the bar. Its first field is
/// required, which Save waits on and Discard leaves.
pub(crate) fn long_form() -> AnyView {
    let fields = (1..=14_u32)
        .map(|index| {
            let id = format!("long-field-{index}");
            let input_id = id.clone();
            keyed_field(
                id,
                tr_with(Msg::SyntheticSetting, &[("index", index.into())]),
                None,
                view! {
                    <input
                        id=input_id
                        type="text"
                        class=INPUT
                        value=format!("value-{index}")
                        required=index == 1
                    />
                },
            )
        })
        .collect_view();
    card(
        tr(Msg::LongForm),
        view! {
            <form method="get" class=FORM id=FORM_ID>
                {fields}
            </form>
        },
    )
    .into_any()
}

/// The docked bar that saves the long form, in the state the address asks
/// for, when it asks for one.
pub(crate) fn dock() -> Option<AnyView> {
    let state = asked()?;
    let held = matches!(state, State::Submitting | State::Done | State::Stale);
    let reason = DockField {
        id: String::from("dock-reason"),
        name: String::from("reason"),
        label: tr(Msg::DockReason),
        value: match state {
            State::Missing | State::Disabled => String::new(),
            _ => tr(Msg::DockReasonSample),
        },
        placeholder: tr(Msg::DockReasonPlaceholder),
        shortcut: Some(String::from("r")),
        error: (state == State::Missing).then(|| tr(Msg::DockReasonMissing)),
        help: Some(tr(Msg::DockReasonHelp)),
        disabled: held,
        focus: state == State::Missing,
    };
    let fields = match state {
        State::Bare => Vec::new(),
        State::Path => vec![
            reason,
            DockField {
                id: String::from("dock-path"),
                name: String::from("path"),
                label: tr(Msg::DockPath),
                value: String::from(NEW_PATH),
                help: Some(tr(Msg::DockPathHelp)),
                ..DockField::default()
            },
        ],
        _ => vec![reason],
    };
    let line = match state {
        State::Done => Some(DockLine::Done(tr(Msg::DockSaved))),
        State::Stale => Some(DockLine::Stale {
            said: tr(Msg::DockStale),
            ways: vec![DockLink {
                href: String::from("?"),
                words: tr(Msg::DockOpenCurrent),
                variant: Variant::Secondary,
            }],
        }),
        State::Bare => Some(DockLine::State {
            glyph: Some(Glyph::PENCIL),
            said: tr(Msg::DockDraftChanges),
        }),
        _ => None,
    };
    let discard = Decision {
        value: String::from("discard"),
        label: tr(Msg::DockDiscard),
        glyph: Some(Glyph::X),
        state: if held {
            DecisionState::Held
        } else {
            DecisionState::Ready
        },
        leaves: true,
        ..Decision::default()
    };
    let save = Decision {
        value: String::from("save"),
        label: if state == State::Submitting {
            tr(Msg::DockSaving)
        } else {
            tr(Msg::SaveSettings)
        },
        variant: Variant::Primary,
        glyph: Some(Glyph::SAVE),
        shortcut: Some(String::from("Ctrl s")),
        state: match state {
            State::Submitting => DecisionState::Busy,
            State::Done | State::Stale => DecisionState::Held,
            State::Disabled => DecisionState::Blocked(tr(Msg::DockGiveReason)),
            _ => DecisionState::Ready,
        },
        ..Decision::default()
    };
    Some(decision_bar(DecisionBar {
        id: String::from("gallery-dock"),
        label: tr(Msg::DockLabel),
        form: DockForm::Page(String::from(FORM_ID)),
        fields,
        line,
        decisions: vec![discard, save],
    }))
}
