// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The configuration page and its provisioned file.
//!
//! The page holds the revision lifecycle in each of its states, the
//! pending card waiting for a second operator, on trial with its
//! commit-confirm countdown, confirmed and rolled back, the write-only
//! secret field in every state with its read-only form, and the provenance
//! tag in a table. The provisioned file's page carries the tag in its title
//! row.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::view::{
    Body, Column, DataTable, Role, Sorting, cell, data_table, title_cell,
};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::lifecycle::{
    Lifecycle, PendingCard, Step, StepState, lifecycle, pending_card, trial_actions,
};
use ferrohealth_ui::view::secret::{
    SecretChoice, SecretField, SecretState, secret_field, secret_state,
};
use ferrohealth_ui::view::surface::{
    Crumb, Header, SectionHead, card, card_action, page_heading, panel_section,
    panel_sections_card, stack,
};
use ferrohealth_ui::view::tag::{Tag, tag};
use ferrohealth_ui::view::template::detail::{DetailPage, detail_page};
use ferrohealth_ui::view::template::settings::{Setting, file_line, setting_row, settings_panel};
use ferrohealth_ui::widget::action_menu::{Act, Action};
use ferrohealth_ui::widget::countdown::Countdown;
use leptos::prelude::*;

use crate::data::{
    BROKEN_FILE, CONFIG_FILE, FILE_SECRETS, MAPPING_FILES, NEW_SECRET_FILE, SECRET_FILE, TRIAL,
};
use crate::locale::{Msg, tr};

/// The address of a page of the gallery's current brand, after `root`.
fn at(root: &str, path: &str) -> String {
    format!("{root}/{path}")
}

/// The steps `steps`, each a state, a title and a line, as one lifecycle.
fn steps(steps: [(StepState, Msg, Msg); 4]) -> Lifecycle {
    Lifecycle {
        label: tr(Msg::CfWhere),
        steps: steps
            .into_iter()
            .map(|(state, title, line)| Step::new(state, tr(title), tr(line)))
            .collect(),
    }
}

/// A revision applied by its approver, confirmed or not as `last` says.
fn applied(last: (StepState, Msg, Msg)) -> Lifecycle {
    steps([
        (StepState::Done, Msg::CfSaved, Msg::CfT1002),
        (StepState::Done, Msg::CfApproved, Msg::CfReviewer1009),
        (StepState::Done, Msg::CfApplied, Msg::CfT121242),
        last,
    ])
}

/// The lifecycle in each of its states, one section each.
fn lifecycles() -> AnyView {
    let frames = [
        (
            Msg::CfNoApproval,
            steps([
                (StepState::Done, Msg::CfSaved, Msg::CfByYou0951),
                (StepState::Skipped, Msg::CfApproval, Msg::CfNotNeeded),
                (StepState::Current, Msg::CfApply, Msg::CfAnyOperator),
                (StepState::Skipped, Msg::CfConfirm, Msg::CfNotNeeded),
            ]),
        ),
        (
            Msg::CfWaitingApproval,
            steps([
                (StepState::Done, Msg::CfSaved, Msg::CfByOperator1002),
                (StepState::Current, Msg::CfApproval, Msg::CfAnotherOperator),
                (StepState::Waiting, Msg::CfApply, Msg::CfAfterApproval),
                (StepState::Waiting, Msg::CfConfirm, Msg::CfWithin2Min),
            ]),
        ),
        (
            Msg::CfOnTrial,
            applied((StepState::Current, Msg::CfConfirm, Msg::CfBy141442)),
        ),
        (
            Msg::CfConfirmed,
            applied((StepState::Done, Msg::CfConfirmedStep, Msg::CfByYou1358)),
        ),
        (
            Msg::CfRolledBack,
            applied((StepState::Failed, Msg::CfNotConfirmed, Msg::CfDeadline)),
        ),
    ];
    let sections = frames
        .into_iter()
        .map(|(title, steps)| {
            panel_section(
                Some(SectionHead {
                    title: tr(title),
                    ..SectionHead::default()
                }),
                lifecycle(steps),
            )
        })
        .collect_view();
    panel_sections_card(tr(Msg::CfLifecycle), sections)
}

/// The pending cards: waiting for a second operator, on trial with its
/// countdown, confirmed and rolled back.
fn pending(root: &str) -> AnyView {
    let waiting = pending_card(
        PendingCard {
            title: tr(Msg::CfWaitTitle),
            line: tr(Msg::CfWaitLine),
            glyph: Glyph::USERS,
            tone: None,
            lifecycle: steps([
                (StepState::Done, Msg::CfSaved, Msg::CfByOperator1002),
                (StepState::Current, Msg::CfApproval, Msg::CfAnotherOperator),
                (StepState::Waiting, Msg::CfApply, Msg::CfAfterApproval),
                (StepState::Waiting, Msg::CfConfirm, Msg::CfWithin2Min),
            ]),
        },
        Some(
            view! {
                <button
                    type="button"
                    class=Variant::Primary.small()
                    disabled
                    aria-describedby="pending-wait-why"
                >
                    {label(Some(Glyph::CHECK), tr(Msg::CfApply10))}
                </button>
            }
            .into_any(),
        ),
        Some(
            view! {
                <p id="pending-wait-why" class="fui-small fui-muted">
                    {tr(Msg::CfApplyWhy)}
                </p>
            }
            .into_any(),
        ),
    );
    let (deadline, total, left) = TRIAL;
    let trial = pending_card(
        PendingCard {
            title: tr(Msg::CfTrialTitle),
            line: tr(Msg::CfTrialLine),
            glyph: Glyph::CLOCK,
            tone: Some(Tone::Warn),
            lifecycle: applied((StepState::Current, Msg::CfConfirm, Msg::CfBy141442)),
        },
        Some(trial_actions(
            Countdown::new("trial-countdown", left, total, deadline),
            Action::new(
                Some(Glyph::ROTATE_CCW),
                tr(Msg::CfRollBackNow),
                Act::Link(at(root, "configuration#rolled-back")),
            ),
            Action::new(
                Some(Glyph::CHECK),
                tr(Msg::CfConfirm10),
                Act::Link(at(root, "configuration#confirmed")),
            ),
        )),
        None,
    );
    let confirmed = pending_card(
        PendingCard {
            title: tr(Msg::CfConfirmedTitle),
            line: tr(Msg::CfConfirmedLine),
            glyph: Glyph::CIRCLE_CHECK,
            tone: Some(Tone::Ok),
            lifecycle: applied((StepState::Done, Msg::CfConfirmedStep, Msg::CfByYou1358)),
        },
        None,
        None,
    );
    let rolled_back = pending_card(
        PendingCard {
            title: tr(Msg::CfRolledBackTitle),
            line: tr(Msg::CfRolledBackLine),
            glyph: Glyph::ROTATE_CCW,
            tone: Some(Tone::Bad),
            lifecycle: applied((StepState::Failed, Msg::CfNotConfirmed, Msg::CfDeadline)),
        },
        Some(
            view! {
                <a href=at(root, "configuration") class=Variant::Secondary.small()>
                    {label(None, tr(Msg::CfCompare))}
                </a>
            }
            .into_any(),
        ),
        None,
    );
    view! {
        <div id="pending" class="fui-stack">
            {waiting}
            {trial}
            <div id="confirmed">{confirmed}</div>
            <div id="rolled-back">{rolled_back}</div>
        </div>
    }
    .into_any()
}

/// The secret field in every state, one section each.
fn secrets() -> AnyView {
    let set = || SecretState::Set(String::from(SECRET_FILE));
    let fields = [
        (
            Msg::CfKept,
            SecretField::new("secret-kept", tr(Msg::CfPassword), set()),
        ),
        (
            Msg::CfSetValue,
            SecretField::new("secret-value", tr(Msg::CfPassword), set())
                .choose(SecretChoice::Value),
        ),
        (
            Msg::CfFromFile,
            SecretField::new("secret-file", tr(Msg::CfPrivateKey), set())
                .choose(SecretChoice::File)
                .path(NEW_SECRET_FILE),
        ),
        (
            Msg::CfClear,
            SecretField::new("secret-clear", tr(Msg::CfPassword), set())
                .choose(SecretChoice::Clear),
        ),
        (
            Msg::CfNotSet,
            SecretField::new("secret-unset", tr(Msg::CfClientSecret), SecretState::Unset),
        ),
        (
            Msg::CfUnreadable,
            SecretField::new(
                "secret-broken",
                tr(Msg::CfPrivateKey),
                SecretState::Unreadable(String::from(BROKEN_FILE)),
            ),
        ),
        (
            Msg::CfRefused,
            SecretField::new("secret-refused", tr(Msg::CfPassword), set()).refuse_value(),
        ),
    ];
    let sections = fields
        .into_iter()
        .map(|(title, field)| {
            panel_section(
                Some(SectionHead {
                    title: tr(title),
                    ..SectionHead::default()
                }),
                secret_field(field, None),
            )
        })
        .collect_view();
    panel_sections_card(tr(Msg::CfSecrets), sections)
}

/// The read-only settings of a console in file mode, each secret by
/// reference.
fn read_only(root: &str) -> AnyView {
    let [db, broker, oidc, session] = FILE_SECRETS;
    let named = |file: Option<&str>| file.map(String::from).unwrap_or_default();
    let rows = [
        (
            Msg::CfDatabase,
            Msg::CfDatabaseDesc,
            SecretState::Set(named(db.1)),
            db.0,
        ),
        (
            Msg::CfBroker,
            Msg::CfBrokerDesc,
            SecretState::Redacted,
            broker.0,
        ),
        (Msg::CfOidc, Msg::CfOidcDesc, SecretState::Unset, oidc.0),
        (
            Msg::CfSessionKey,
            Msg::CfSessionKeyDesc,
            SecretState::Unreadable(named(session.1)),
            session.0,
        ),
    ]
    .into_iter()
    .map(|(name, description, state, key)| {
        setting_row(
            Setting {
                label: tr(name),
                description: tr(description),
                value: secret_state(&state),
                keys: vec![String::from(key)],
                next_start: false,
            },
            false,
        )
    })
    .collect();
    card(
        tr(Msg::CfReadOnly),
        settings_panel(
            file_line(String::from(CONFIG_FILE), at(root, "configuration")),
            None,
            rows,
        ),
    )
    .into_any()
}

/// The mapping files, the provisioned ones locked.
fn provenance(root: &str) -> AnyView {
    let columns = [
        Column::text("file", tr(Msg::CfFile)),
        Column::text("template", tr(Msg::CfTemplate)),
        Column {
            role: Role::Sub,
            ..Column::text("source", tr(Msg::CfSource))
        },
    ];
    let provisioned = at(root, "configuration/provisioned");
    let rows = MAPPING_FILES
        .into_iter()
        .map(|(file, template, locked)| {
            let source = if locked {
                Tag::new(tr(Msg::CfProvisioned))
                    .locked(tr(Msg::CfOwnedByDirectory))
                    .view()
            } else {
                tag(tr(Msg::CfDraft))
            };
            let href = locked.then_some(provisioned.as_str());
            view! {
                <tr>
                    {title_cell(href, view! { <span class="fui-mono">{file}</span> }, None)}
                    {cell(&columns[1], view! { <span class="fui-mono">{template}</span> })}
                    {cell(&columns[2], source)}
                </tr>
            }
        })
        .collect_view()
        .into_any();
    let caption = tr(Msg::CfMappingFiles);
    let table = data_table(DataTable {
        caption: &caption,
        columns: &columns,
        sorting: Sorting::default(),
        body: Body::Rows(rows),
    })
    .into_any();
    view! {
        <section class="fui-card fui-panel" id="provenance">
            <div class="fui-panel-head">
                <h2 class="fui-card-title">{tr(Msg::CfProvenance)}</h2>
                <div class="fui-actions">{card_action(provisioned, tr(Msg::CfOpenProvisioned))}</div>
            </div>
            {table}
        </section>
    }
    .into_any()
}

/// The configuration page.
#[component]
pub fn ConfigurationPage() -> impl IntoView {
    let root =
        crate::app::route_brand().map_or_else(String::new, |brand| format!("/{}", brand.key()));
    let header = page_heading(
        Header {
            title: tr(Msg::Configuration).into(),
            subtitle: Some(tr(Msg::CfSubtitle).into()),
            crumbs: vec![Crumb::new(tr(Msg::Gallery), root.clone())],
            ..Header::default()
        },
        ActionGroup::none(),
    )
    .into_any();
    let sections = (
        pending(&root),
        lifecycles(),
        secrets(),
        read_only(&root),
        provenance(&root),
    );
    view! {
        {super::title(tr(Msg::Configuration))}
        {header}
        {stack(
            view! {
                {sections.0}
                {sections.1}
                {sections.2}
                {sections.3}
                {sections.4}
            },
        )}
    }
}

/// The page of a provisioned mapping file, its title row carrying the lock
/// tag.
#[component]
pub fn ProvisionedPage() -> impl IntoView {
    let root =
        crate::app::route_brand().map_or_else(String::new, |brand| format!("/{}", brand.key()));
    let page = detail_page(DetailPage {
        header: Header {
            title: String::from(MAPPING_FILES[0].0).into(),
            subtitle: Some(tr(Msg::CfProvisionedSubtitle).into()),
            subtitle_view: None,
            crumbs: vec![
                Crumb::new(tr(Msg::Gallery), root.clone()),
                Crumb::new(tr(Msg::Configuration), at(&root, "configuration")),
            ],
            mono: true,
            badge: Some(
                Tag::new(tr(Msg::CfProvisioned))
                    .locked(tr(Msg::CfOwnedByDirectory))
                    .view(),
            ),
        },
        actions: ActionGroup::none(),
        stepper: None,
        tabs: None,
        body: card(
            tr(Msg::CfAboutFile),
            view! { <p>{tr(Msg::CfProvisionedBody)}</p> },
        )
        .into_any(),
    });
    view! {
        {super::title(tr(Msg::CfProvisionedTitle))}
        {page}
    }
}
