// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The sign-in page of a brand in every state R19 draws.
//!
//! The states are ready, on the way to the provider, signed out, expired, full, unavailable, refused,
//! rejected, the provider or the server behind unreachable, and the basic
//! form after a wrong password and while paused.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::island::SessionEnded;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::view::shell::{centred, lockup_plain, mark};
use ferrohealth_ui::view::sign_in::{Basic, Notice, Provider, Retry, SignIn, State, sign_in};
use leptos::prelude::*;
use leptos_meta::Stylesheet;
use leptos_router::hooks::use_params_map;

use crate::app::{brand_glyph, footer, theme_path};
use crate::locale::{Msg, tr};

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
fn says(text: Msg) -> Notice {
    Notice::new(tr(text))
}

/// A notice saying `text`, with the muted `detail` after it.
fn says_more(text: Msg, detail: Msg) -> Notice {
    Notice {
        detail: Some(tr(detail)),
        ..says(text)
    }
}

/// The state the key `key` names, `None` for a key that names none.
fn state(key: &str, here: &str) -> Option<State> {
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
                label: tr(Msg::TryAgain),
                href: here.to_owned(),
            }),
            ..says(Msg::StateUpstream)
        }),
        "locked" => State::Locked(says(Msg::StateLocked)),
        _ => return None,
    })
}

/// The card of `brand` in the state `key` names.
fn card(brand: Brand, key: &str) -> AnyView {
    let here = format!("/sign-in/{}/{key}", brand.key());
    let state = state(key, &here).unwrap_or_default();
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
        view! {
            <p class="fui-small fui-sign-in-line">
                <a href=format!("/{}", brand.key())>{tr(Msg::ReadWithout)}</a>
                ". "
                {tr(Msg::ReadWithoutLine)}
            </p>
        }
        .into_any()
    });
    sign_in(SignIn {
        lockup: lockup_plain(mark(brand_glyph(brand)), brand.product(), tr(Msg::Gallery))
            .into_any(),
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
    })
    .into_any()
}

/// The sign-in page of the brand and the state the route names.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn SignInPage() -> impl IntoView {
    let params = use_params_map();
    let named = params.with_untracked(|params| {
        let brand = params.get_str("brand").and_then(Brand::parse)?;
        let key = params
            .get_str("variant")
            .filter(|key| STATES.contains(key))?;
        Some((brand, key.to_owned()))
    });
    let (brand, key) = named.unwrap_or((Brand::Bridge, String::from("provider")));
    let ended = (key == "signed-out").then(|| view! { <SessionEnded /> });
    view! {
        {ended}
        {super::title_in(tr(Msg::SignIn), Some(brand))}
        <Stylesheet id="brand-theme" href=theme_path(brand) />
        {centred(card(brand, &key), footer())}
    }
}
