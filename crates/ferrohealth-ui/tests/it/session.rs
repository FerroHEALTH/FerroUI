// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The session's end as the server draws it (#444): the warning closed, an
//! alert dialog named by its heading and described by its sentence, Stay
//! signed in taking the focus when it opens, and Sign out a plain post that
//! loads the signed-out page as a document.

use ferrohealth_ui::style::STYLESHEET;
use ferrohealth_ui::widget::session::{Left, WARN_FLOOR, WARNING_ID, Watch, session_watch};
use ferrohealth_ui::widget::submit::RELOADS;

use crate::markup::{has, tags};
use crate::render::html;

/// The watch of a console whose session route is `/session`.
fn watch() -> Watch {
    Watch::new(
        "/session",
        "/sign-in",
        "/sign-out",
        Left {
            idle: 900,
            absolute: 28_800,
        },
    )
}

#[test]
fn the_server_draws_the_warning_closed_as_a_named_and_described_alert_dialog() {
    let drawn = html(|| session_watch(watch()));
    let dialogs = tags(&drawn, "dialog");
    assert_eq!(1, dialogs.len(), "{drawn}");
    let dialog = &dialogs[0];
    assert_eq!(Some(WARNING_ID), dialog.get("id").map(String::as_str));
    assert_eq!(
        Some("alertdialog"),
        dialog.get("role").map(String::as_str),
        "WAI-ARIA APG, the alert dialog pattern"
    );
    assert!(!dialog.contains_key("open"), "drawn closed: {drawn}");
    let title = format!("{WARNING_ID}-title");
    let ends = format!("{WARNING_ID}-ends");
    assert_eq!(Some(&title), dialog.get("aria-labelledby"));
    assert_eq!(Some(&ends), dialog.get("aria-describedby"));
    assert!(has(&drawn, "h2", &[("id", title.as_str())]), "{drawn}");
    assert!(has(&drawn, "p", &[("id", ends.as_str())]), "{drawn}");
    assert!(drawn.contains("Your session is about to end"), "{drawn}");
    assert!(
        drawn.contains("2:00"),
        "the sentence names the warning's time: {drawn}"
    );
}

#[test]
fn stay_signed_in_takes_the_focus_and_sign_out_posts_as_a_document() {
    let drawn = html(|| session_watch(watch()));
    assert!(
        has(&drawn, "button", &[("type", "button"), ("autofocus", "")]),
        "the least destructive action takes the focus (APG alert dialog): {drawn}"
    );
    let form = format!("{WARNING_ID}-sign-out");
    assert!(
        has(
            &drawn,
            "form",
            &[
                ("id", form.as_str()),
                ("method", "post"),
                ("action", "/sign-out"),
                (RELOADS, ""),
            ]
        ),
        "{drawn}"
    );
    assert!(
        has(
            &drawn,
            "button",
            &[("type", "submit"), ("form", form.as_str())]
        ),
        "{drawn}"
    );
    assert!(drawn.contains("Stay signed in") && drawn.contains("Sign out"));
}

#[test]
fn the_warning_never_gives_fewer_than_twenty_seconds_and_writes_only_defined_classes() {
    let drawn = html(|| {
        session_watch(Watch {
            warn_at: 5,
            ..watch()
        })
    });
    assert!(
        drawn.contains(&format!("0:{WARN_FLOOR}")),
        "SC 2.2.1: at least 20 seconds to answer: {drawn}"
    );
    for class in [
        "fui-dialog",
        "fui-modal",
        "fui-dialog-actions",
        "fui-modal-body",
    ] {
        assert!(STYLESHEET.contains(&format!(".{class}")), "{class}");
    }
    assert!(!drawn.contains(" style=\""), "{drawn}");
}
