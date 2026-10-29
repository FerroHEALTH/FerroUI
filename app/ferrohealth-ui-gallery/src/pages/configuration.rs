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

use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::view::{
    Body, Column, DataTable, Role, Sorting, cell, data_table, title_cell,
};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::action_menu::{Act, Action};
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::countdown::Countdown;
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
use ferrohealth_ui::view::template::settings::{
    How, Setting, file_line, setting_row, settings_panel,
};

use crate::app::Page;
use crate::data::{
    BROKEN_FILE, CONFIG_FILE, FILE_SECRETS, MAPPING_FILES, NEW_SECRET_FILE, SECRET_FILE, TRIAL,
};
use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

/// The address of a page of the gallery's current brand, after `root`.
fn at(root: &str, path: &str) -> String {
    format!("{root}/{path}")
}

/// The steps `steps`, each a state, a title and a line, as one lifecycle.
fn steps(v: &Visit, steps: [(StepState, Msg, Msg); 4]) -> Lifecycle {
    Lifecycle {
        label: v.tr(Msg::CfWhere),
        steps: steps
            .into_iter()
            .map(|(state, title, line)| Step::new(state, v.tr(title), v.tr(line)))
            .collect(),
    }
}

/// A revision applied by its approver, confirmed or not as `last` says.
fn applied(v: &Visit, last: (StepState, Msg, Msg)) -> Lifecycle {
    steps(
        v,
        [
            (StepState::Done, Msg::CfSaved, Msg::CfT1002),
            (StepState::Done, Msg::CfApproved, Msg::CfReviewer1009),
            (StepState::Done, Msg::CfApplied, Msg::CfT121242),
            last,
        ],
    )
}

/// The lifecycle in each of its states, one section each.
fn lifecycles(v: &Visit) -> Html {
    let frames = [
        (
            Msg::CfNoApproval,
            steps(
                v,
                [
                    (StepState::Done, Msg::CfSaved, Msg::CfByYou0951),
                    (StepState::Skipped, Msg::CfApproval, Msg::CfNotNeeded),
                    (StepState::Current, Msg::CfApply, Msg::CfAnyOperator),
                    (StepState::Skipped, Msg::CfConfirm, Msg::CfNotNeeded),
                ],
            ),
        ),
        (
            Msg::CfWaitingApproval,
            steps(
                v,
                [
                    (StepState::Done, Msg::CfSaved, Msg::CfByOperator1002),
                    (StepState::Current, Msg::CfApproval, Msg::CfAnotherOperator),
                    (StepState::Waiting, Msg::CfApply, Msg::CfAfterApproval),
                    (StepState::Waiting, Msg::CfConfirm, Msg::CfWithin2Min),
                ],
            ),
        ),
        (
            Msg::CfOnTrial,
            applied(v, (StepState::Current, Msg::CfConfirm, Msg::CfBy141442)),
        ),
        (
            Msg::CfConfirmed,
            applied(v, (StepState::Done, Msg::CfConfirmedStep, Msg::CfByYou1358)),
        ),
        (
            Msg::CfRolledBack,
            applied(v, (StepState::Failed, Msg::CfNotConfirmed, Msg::CfDeadline)),
        ),
    ];
    let sections = frames
        .into_iter()
        .map(|(title, steps)| {
            panel_section(
                &v.locale,
                Some(SectionHead {
                    title: v.tr(title),
                    ..SectionHead::default()
                }),
                lifecycle(&v.locale, steps),
            )
        })
        .collect();
    panel_sections_card(v.tr(Msg::CfLifecycle), sections)
}

/// The pending cards: waiting for a second operator, on trial with its
/// countdown, confirmed and rolled back.
fn pending(v: &Visit, root: &str) -> Html {
    let waiting = pending_card(
        &v.locale,
        PendingCard {
            title: v.tr(Msg::CfWaitTitle),
            line: v.tr(Msg::CfWaitLine),
            glyph: Glyph::USERS,
            tone: None,
            lifecycle: steps(
                v,
                [
                    (StepState::Done, Msg::CfSaved, Msg::CfByOperator1002),
                    (StepState::Current, Msg::CfApproval, Msg::CfAnotherOperator),
                    (StepState::Waiting, Msg::CfApply, Msg::CfAfterApproval),
                    (StepState::Waiting, Msg::CfConfirm, Msg::CfWithin2Min),
                ],
            ),
        },
        Some(h(format!(
            "<button type=\"button\" class=\"{}\" disabled aria-describedby=\"pending-wait-why\">{}</button>",
            Variant::Primary.small(),
            label(Some(Glyph::CHECK), v.tr(Msg::CfApply10))
        ))),
        Some(h(format!(
            "<p id=\"pending-wait-why\" class=\"fui-small fui-muted\">{}</p>",
            t(v.tr(Msg::CfApplyWhy))
        ))),
    );
    let (deadline, total, left) = TRIAL;
    let trial = pending_card(
        &v.locale,
        PendingCard {
            title: v.tr(Msg::CfTrialTitle),
            line: v.tr(Msg::CfTrialLine),
            glyph: Glyph::CLOCK,
            tone: Some(Tone::Warn),
            lifecycle: applied(v, (StepState::Current, Msg::CfConfirm, Msg::CfBy141442)),
        },
        Some(trial_actions(
            &v.locale,
            Countdown::new(&v.locale, "trial-countdown", left, total, deadline),
            Action::new(
                Some(Glyph::ROTATE_CCW),
                v.tr(Msg::CfRollBackNow),
                Act::Link(at(root, "configuration#rolled-back")),
            ),
            Action::new(
                Some(Glyph::CHECK),
                v.tr(Msg::CfConfirm10),
                Act::Link(at(root, "configuration#confirmed")),
            ),
        )),
        None,
    );
    let confirmed = pending_card(
        &v.locale,
        PendingCard {
            title: v.tr(Msg::CfConfirmedTitle),
            line: v.tr(Msg::CfConfirmedLine),
            glyph: Glyph::CIRCLE_CHECK,
            tone: Some(Tone::Ok),
            lifecycle: applied(v, (StepState::Done, Msg::CfConfirmedStep, Msg::CfByYou1358)),
        },
        None,
        None,
    );
    let rolled_back = pending_card(
        &v.locale,
        PendingCard {
            title: v.tr(Msg::CfRolledBackTitle),
            line: v.tr(Msg::CfRolledBackLine),
            glyph: Glyph::ROTATE_CCW,
            tone: Some(Tone::Bad),
            lifecycle: applied(v, (StepState::Failed, Msg::CfNotConfirmed, Msg::CfDeadline)),
        },
        Some(h(format!(
            "<a href=\"{}\" class=\"{}\">{}</a>",
            t(at(root, "configuration")),
            Variant::Secondary.small(),
            label(None, v.tr(Msg::CfCompare))
        ))),
        None,
    );
    h(format!(
        "<div id=\"pending\" class=\"fui-stack\">{waiting}{trial}<div id=\"confirmed\">{confirmed}</div><div id=\"rolled-back\">{rolled_back}</div></div>"
    ))
}

/// The secret field in every state, one section each.
fn secrets(v: &Visit) -> Html {
    let set = || SecretState::Set(String::from(SECRET_FILE));
    let fields = [
        (
            Msg::CfKept,
            SecretField::new("secret-kept", v.tr(Msg::CfPassword), set()),
        ),
        (
            Msg::CfSetValue,
            SecretField::new("secret-value", v.tr(Msg::CfPassword), set())
                .choose(SecretChoice::Value),
        ),
        (
            Msg::CfFromFile,
            SecretField::new("secret-file", v.tr(Msg::CfPrivateKey), set())
                .choose(SecretChoice::File)
                .path(NEW_SECRET_FILE),
        ),
        (
            Msg::CfClear,
            SecretField::new("secret-clear", v.tr(Msg::CfPassword), set())
                .choose(SecretChoice::Clear),
        ),
        (
            Msg::CfNotSet,
            SecretField::new(
                "secret-unset",
                v.tr(Msg::CfClientSecret),
                SecretState::Unset,
            ),
        ),
        (
            Msg::CfUnreadable,
            SecretField::new(
                "secret-broken",
                v.tr(Msg::CfPrivateKey),
                SecretState::Unreadable(String::from(BROKEN_FILE)),
            ),
        ),
        (
            Msg::CfRefused,
            SecretField::new("secret-refused", v.tr(Msg::CfPassword), set())
                .refuse_value(&v.locale),
        ),
    ];
    let sections = fields
        .into_iter()
        .map(|(title, field)| {
            panel_section(
                &v.locale,
                Some(SectionHead {
                    title: v.tr(title),
                    ..SectionHead::default()
                }),
                secret_field(&v.locale, &field, None),
            )
        })
        .collect();
    panel_sections_card(v.tr(Msg::CfSecrets), sections)
}

/// The read-only settings of a console in file mode, each secret by
/// reference.
fn read_only(v: &Visit, root: &str) -> Html {
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
            &v.locale,
            Setting {
                label: v.tr(name),
                description: v.tr(description),
                value: secret_state(&v.locale, &state),
                keys: vec![String::from(key)],
                next_start: false,
                source: None,
            },
            false,
        )
    })
    .collect();
    card(
        v.tr(Msg::CfReadOnly),
        settings_panel(
            file_line(&v.locale, CONFIG_FILE, How::Page(at(root, "configuration"))),
            None,
            rows,
        ),
    )
}

/// The mapping files, the provisioned ones locked.
fn provenance(v: &Visit, root: &str) -> Html {
    let columns = [
        Column::text("file", v.tr(Msg::CfFile)),
        Column::text("template", v.tr(Msg::CfTemplate)),
        Column {
            role: Role::Sub,
            ..Column::text("source", v.tr(Msg::CfSource))
        },
    ];
    let provisioned = at(root, "configuration/provisioned");
    let rows = MAPPING_FILES
        .into_iter()
        .map(|(file, template, locked)| {
            let source = if locked {
                Tag::new(v.tr(Msg::CfProvisioned))
                    .locked(v.tr(Msg::CfOwnedByDirectory))
                    .view()
            } else {
                tag(v.tr(Msg::CfDraft))
            };
            let href = locked.then_some(provisioned.as_str());
            h(format!(
                "<tr>{}{}{}</tr>",
                title_cell(
                    href,
                    h(format!("<span class=\"fui-mono\">{file}</span>")),
                    None
                ),
                cell(
                    &columns[1],
                    h(format!("<span class=\"fui-mono\">{template}</span>"))
                ),
                cell(&columns[2], source),
            ))
        })
        .collect();
    let caption = v.tr(Msg::CfMappingFiles);
    let table = data_table(DataTable {
        caption: &caption,
        columns: &columns,
        sorting: Sorting::default(),
        body: Body::Rows(rows),
    });
    h(format!(
        "<section class=\"fui-card fui-panel\" id=\"provenance\"><div class=\"fui-panel-head\"><h2 class=\"fui-card-title\">{}</h2><div class=\"fui-actions\">{}</div></div>{table}</section>",
        t(v.tr(Msg::CfProvenance)),
        card_action(provisioned.clone(), v.tr(Msg::CfOpenProvisioned))
    ))
}

/// The configuration page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let root = format!("/{}", v.brand.key());
    let header = page_heading(
        &v.locale,
        Header {
            title: v.tr(Msg::Configuration).into(),
            subtitle: Some(v.tr(Msg::CfSubtitle).into()),
            crumbs: vec![Crumb::new(v.tr(Msg::Gallery), root.clone())],
            ..Header::default()
        },
        ActionGroup::none(),
    );
    let body = stack(Html::join([
        pending(v, &root),
        lifecycles(v),
        secrets(v),
        read_only(v, &root),
        provenance(v, &root),
    ]));
    Page::shell(v.tr(Msg::Configuration), Html::join([header, body]))
}

/// The page of a provisioned mapping file, its title row carrying the lock
/// tag.
#[must_use]
pub fn provisioned(v: &Visit) -> Page {
    let root = format!("/{}", v.brand.key());
    let page = detail_page(
        &v.locale,
        DetailPage {
            header: Header {
                title: String::from(MAPPING_FILES[0].0).into(),
                subtitle: Some(v.tr(Msg::CfProvisionedSubtitle).into()),
                subtitle_view: None,
                crumbs: vec![
                    Crumb::new(v.tr(Msg::Gallery), root.clone()),
                    Crumb::new(v.tr(Msg::Configuration), at(&root, "configuration")),
                ],
                mono: true,
                badge: Some(
                    Tag::new(v.tr(Msg::CfProvisioned))
                        .locked(v.tr(Msg::CfOwnedByDirectory))
                        .view(),
                ),
            },
            actions: ActionGroup::none(),
            stepper: None,
            tabs: None,
            body: card(
                v.tr(Msg::CfAboutFile),
                h(format!("<p>{}</p>", t(v.tr(Msg::CfProvisionedBody)))),
            ),
        },
    );
    Page::shell(v.tr(Msg::CfProvisionedTitle), page)
}
