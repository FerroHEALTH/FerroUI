// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The federation page's fan-out outcome: every node answering, one failing
//! and every one failing.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::view::fanout::{Answer, Fanout, FanoutNode, fanout};
use ferrohealth_ui::view::surface::card;

use crate::locale::Msg;
use crate::visit::Visit;

/// One node of a fan-out: its name, how it answered, its word and term,
/// what it gave and why it failed.
fn node(
    v: &Visit,
    name: &str,
    answer: Answer,
    word: Msg,
    term: &str,
    measure: Msg,
    reason: Option<Msg>,
) -> FanoutNode {
    FanoutNode {
        name: name.to_owned(),
        answer,
        word: v.tr(word),
        term: Some(term.to_owned()),
        measure: v.tr(measure),
        reason: reason.map(|reason| v.tr(reason)),
    }
}

/// The fan-out of one query: every node answered, one failed, or all did.
pub(super) fn outcome(v: &Visit, failing: usize) -> Html {
    let failed = |at: usize, name: &str, rows: Msg| {
        if at < failing {
            node(
                v,
                name,
                Answer::Failed,
                Msg::FedNodeError,
                "node-error",
                Msg::FedMs390,
                Some(Msg::FedAnswered503),
            )
        } else {
            node(
                v,
                name,
                Answer::Answered,
                Msg::FedAnswered,
                "active",
                rows,
                None,
            )
        }
    };
    let nodes = vec![
        failed(3, "node-a-pub", Msg::FedRows3),
        failed(1, "node-a-lab-pub", Msg::FedRows1),
        failed(0, "node-b-pub", Msg::FedRows2),
        failed(2, "node-c-pub", Msg::FedRows0),
        node(
            v,
            "node-e-pub",
            Answer::Skipped,
            Msg::FedExcluded,
            "excluded",
            Msg::FedNotAsked,
            Some(Msg::FedExcludedByOperator),
        ),
    ];
    let (title, line, missing) = match failing {
        0 => (
            Msg::FedAllAnswered,
            Msg::FedLineComplete,
            Msg::FedMissingNone,
        ),
        1 => (Msg::FedOneFailing, Msg::FedLinePartial, Msg::FedMissingOne),
        _ => (Msg::FedAllFailing, Msg::FedLineFailed, Msg::FedMissingAll),
    };
    let body = fanout(
        &v.locale,
        &Fanout {
            label: v.tr(Msg::FedWhatEachDid),
            line: v.tr(line),
            missing: v.tr(missing),
            nodes,
        },
    );
    card(v.tr(title), body)
}

/// The fan-out of a query no node was asked for, every node excluded,
/// each named by a long host name.
pub(super) fn outcome_none_asked(v: &Visit) -> Html {
    let nodes = ["node-eu-west-1", "node-eu-central-2", "node-eu-north-3"]
        .map(|host| {
            node(
                v,
                &format!("{host}.ferrofed.internal.example.org"),
                Answer::Skipped,
                Msg::FedExcluded,
                "excluded",
                Msg::FedNotAsked,
                Some(Msg::FedExcludedByOperator),
            )
        })
        .to_vec();
    let body = fanout(
        &v.locale,
        &Fanout {
            label: v.tr(Msg::FedWhatEachDid),
            line: v.tr(Msg::FedLineNoneAsked),
            missing: String::new(),
            nodes,
        },
    );
    card(v.tr(Msg::FedNoneAsked), body)
}
