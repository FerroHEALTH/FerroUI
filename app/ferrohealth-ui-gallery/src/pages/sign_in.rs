// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The sign-in page of a brand in every state R19 draws.
//!
//! The states are ready, on the way to the provider, signed out, expired, full, unavailable, refused,
//! rejected, the provider or the server behind unreachable, and the basic
//! form after a wrong password and while paused.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::view::session::session_ended;
use ferrohealth_ui::view::shell::{lockup_plain, mark};
use ferrohealth_ui::view::sign_in::{Basic, Notice, Provider, Retry, SignIn, State, sign_in};

use crate::app::{Page, brand_glyph};
use crate::locale::Msg;
use crate::visit::Visit;

/// Every state of the sign-in page the gallery draws, by the key its path
/// names.
pub const STATES: [&str; 12] = [
    "provider",
    "basic",
    "redirecting",
    "signed-out",
    "expired",
    "full",
    "unavailable",
    "refused",
    "rejected",
    "unreachable",
    "upstream",
    "locked",
];

/// A notice saying the gallery's message `text`.
fn says(visit: &Visit, text: Msg) -> Notice {
    Notice::new(visit.tr(text))
}

/// A notice saying `text`, with the muted `detail` after it.
fn says_more(visit: &Visit, text: Msg, detail: Msg) -> Notice {
    Notice {
        detail: Some(visit.tr(detail)),
        ..says(visit, text)
    }
}

/// The state the key `key` names, `None` for a key that names none.
fn state(visit: &Visit, key: &str, here: &str) -> Option<State> {
    let says = |text| says(visit, text);
    let says_more = |text, detail| says_more(visit, text, detail);
    Some(match key {
        "provider" => State::Ready,
        "basic" => State::Refused(says(Msg::StateWrong)),
        "redirecting" => State::Redirecting(Some(says(Msg::StateRedirecting))),
        "signed-out" => State::SignedOut(says(Msg::StateSignedOut)),
        "expired" => State::Expired(says(Msg::StateExpired)),
        "full" => State::Full(says(Msg::StateFull)),
        "unavailable" => State::Unavailable(says_more(
            Msg::StateUnavailable,
            Msg::StateUnavailableDetail,
        )),
        "refused" => State::Refused(says_more(Msg::StateRefused, Msg::StateRefusedDetail)),
        "rejected" => State::Rejected(says_more(Msg::StateRejected, Msg::StateRejectedDetail)),
        "unreachable" => State::ProviderUnreachable(says(Msg::StateProviderUnreachable)),
        "upstream" => State::UpstreamUnreachable(Notice {
            retry: Some(Retry {
                label: visit.tr(Msg::TryAgain),
                href: here.to_owned(),
            }),
            ..says(Msg::StateUpstream)
        }),
        "locked" => State::Locked(says(Msg::StateLocked)),
        _ => return None,
    })
}

/// The card of `brand` in the state `key` names.
fn card(visit: &Visit, brand: Brand, key: &str) -> Html {
    let tr = |msg| visit.tr(msg);
    let here = format!("/sign-in/{}/{key}", brand.key());
    let state = state(visit, key, &here).unwrap_or_default();
    let basic = matches!(key, "basic" | "locked").then(|| Basic {
        action: format!("/sign-in/{}/basic", brand.key()),
        next: format!("/{}", brand.key()),
        divider: Some(tr(Msg::BasicDivider)),
        submit: None,
        // The synthetic username the gallery fills in again after a refusal.
        username: String::from("demo.steward"),
        error: (key == "basic").then(|| tr(Msg::StateWrongTries)),
    });
    let label = if matches!(key, "refused" | "unreachable") {
        tr(Msg::TryAgain)
    } else {
        tr(Msg::SignInWith)
    };
    let line = (key == "unreachable").then(|| {
        Html::raw(format!(
            "<p class=\"fui-small fui-sign-in-line\"><a href=\"/{}\">{}</a>. {}</p>",
            brand.key(),
            Html::text(tr(Msg::ReadWithout)),
            Html::text(tr(Msg::ReadWithoutLine))
        ))
    });
    sign_in(
        &visit.locale,
        SignIn {
            lockup: lockup_plain(mark(brand_glyph(brand)), brand.product(), tr(Msg::Gallery)),
            title: Some(tr(Msg::SignInTitle)),
            state,
            provider: Some(Provider {
                href: format!("/sign-in/{}/redirecting", brand.key()),
                label,
                busy: tr(Msg::SignInTaking),
                glyph: Glyph::LOG_IN,
            }),
            basic,
            line,
            note: Some(tr(Msg::SignInNote)),
            links: vec![
                (tr(Msg::Documentation), String::from("/")),
                (tr(Msg::PrivacyNotice), String::from("/")),
            ],
            caption: Some(tr(Msg::SignInCaption)),
        },
    )
}

/// The sign-in page of the brand and the state the path names.
#[must_use]
pub fn page(visit: &Visit) -> Page {
    let named = visit.segment(1).and_then(Brand::parse).zip(
        visit
            .segment(2)
            .filter(|key| STATES.contains(key))
            .map(str::to_owned),
    );
    let (brand, key) = named.unwrap_or((Brand::Bridge, String::from("provider")));
    let ended = (key == "signed-out")
        .then(session_ended)
        .unwrap_or_default();
    Page::centred(
        visit.tr(Msg::SignIn),
        Html::join([ended, card(visit, brand, &key)]),
    )
}
