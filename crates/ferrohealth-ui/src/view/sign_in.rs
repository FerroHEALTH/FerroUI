// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The sign-in card, after FerroEHR's sign-in screen: the product's lockup,
//! a notice, the refusal, the basic form, the OAuth 2.0 or OpenID Connect
//! button, and links.
//!
//! A console shows the methods it has: the basic form alone (a user name
//! and a password), the provider's button alone, or both with "or" between
//! them; with neither it says so instead of drawing a blank card. Both post
//! or link to the console's server, which holds the session; the page keeps
//! no credential and no token. The form is a plain `POST` with plain,
//! uncontrolled inputs, so it works before the page hydrates and hydration
//! never wipes what was typed, and it carries the autocomplete tokens a
//! password manager reads (HTML, `autocomplete`, `username` and
//! `current-password`) and the page to return to as a hidden `next`. The
//! button is a plain link marked `rel="external"`, so a client-side router
//! leaves it to the server.

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::button::Variant;
use crate::view::form::INPUT;

/// The basic method: the form that posts a user name and a password.
#[derive(Debug, Clone)]
pub struct Basic {
    /// The console's route the form posts to.
    pub action: &'static str,
    /// The page to return to once signed in, posted as `next`.
    pub next: String,
}

/// The OAuth 2.0 or OpenID Connect method: the button that starts the
/// sign-in at the console's own route.
#[derive(Debug, Clone)]
pub struct Provider {
    /// The console's sign-in route, with the page to return to in its query.
    pub href: String,
    /// The button's words, such as "Sign in with OIDC" or "Sign in".
    pub label: String,
}

/// A notice above the methods.
#[derive(Debug, Clone)]
pub struct Notice {
    /// Its tone: `Warn` for a session that ended or a method list that
    /// could not be read, `Accent` for a deployment's own words.
    pub tone: Tone,
    /// Its words; a line break in them breaks the line.
    pub text: String,
    /// A stable id, such as `session-expired`, for a journey to find it.
    pub id: Option<&'static str>,
}

impl Notice {
    /// A notice in `tone` saying `text`, with no id.
    #[must_use]
    pub fn new(tone: Tone, text: impl Into<String>) -> Self {
        Self {
            tone,
            text: text.into(),
            id: None,
        }
    }

    /// This notice, with the id `id`.
    #[must_use]
    pub fn id(self, id: &'static str) -> Self {
        Self {
            id: Some(id),
            ..self
        }
    }
}

/// The id of the refusal of the last attempt, for a journey to find it.
pub const REFUSAL_ID: &str = "sign-in-refused";

/// What the sign-in card shows.
#[derive(Debug)]
pub struct SignIn {
    /// The product's lockup, from [`crate::view::shell::lockup`].
    pub lockup: AnyView,
    /// The notices, in order, such as "Your session ended. Sign in again to
    /// continue." beside the deployment's own words.
    pub notices: Vec<Notice>,
    /// The refusal of the last attempt, such as "Wrong username or
    /// password.".
    pub refusal: Option<String>,
    /// The basic method, when the console has it.
    pub basic: Option<Basic>,
    /// The provider's method, when the console has it.
    pub provider: Option<Provider>,
    /// Links under the methods, such as the documentation, as words and
    /// href, both of which a deployment may configure; each leaves the
    /// console's router alone.
    pub links: Vec<(String, String)>,
}

/// Returns the basic form posting to `basic.action`.
fn basic_form(basic: Basic) -> impl IntoView {
    let username = tr(Text::SignInUsername);
    let password = tr(Text::SignInPassword);
    view! {
        <form method="post" action=basic.action class="fui-form">
            <div class="fui-field">
                <label for="sign-in-username" class="fui-label">
                    {username.clone()}
                </label>
                <input
                    id="sign-in-username"
                    name="username"
                    class=INPUT
                    placeholder=username
                    autocomplete="username"
                    autocapitalize="off"
                    spellcheck="false"
                />
            </div>
            <div class="fui-field">
                <label for="sign-in-password" class="fui-label">
                    {password.clone()}
                </label>
                <input
                    id="sign-in-password"
                    name="password"
                    type="password"
                    class=INPUT
                    placeholder=password
                    autocomplete="current-password"
                />
            </div>
            <input type="hidden" name="next" value=basic.next />
            <button type="submit" class=Variant::Primary.class()>
                {icon(Glyph::LOG_IN, "16")}
                {tr(Text::SignInSubmit)}
            </button>
        </form>
    }
}

/// Returns the sign-in card `sign_in`, centred by the page around it.
#[must_use]
pub fn sign_in(sign_in: SignIn) -> impl IntoView {
    let notices = sign_in
        .notices
        .into_iter()
        .map(|notice| {
            let class = format!("fui-sign-in-notice {}", notice.tone.class());
            view! {
                <div role="status" class=class id=notice.id>
                    {notice.text}
                </div>
            }
        })
        .collect_view();
    let refusal = sign_in.refusal.map(|refusal| {
        view! {
            <div role="alert" class="fui-alert" id=REFUSAL_ID>
                <p class="fui-alert-title">{icon(Glyph::CIRCLE_ALERT, "16")} {refusal}</p>
            </div>
        }
    });
    let both = sign_in.basic.is_some() && sign_in.provider.is_some();
    let none = sign_in.basic.is_none() && sign_in.provider.is_none();
    let basic = sign_in.basic.map(basic_form);
    let divider = both.then(|| view! { <div class="fui-divider">{tr(Text::SignInOr)}</div> });
    let provider = sign_in.provider.map(|provider| {
        let variant = if both {
            Variant::Secondary
        } else {
            Variant::Primary
        };
        view! {
            <a href=provider.href rel="external" class=variant.class()>
                {icon(Glyph::LOG_IN, "16")}
                {provider.label}
            </a>
        }
    });
    let unavailable = none.then(|| {
        view! {
            <div role="alert" class="fui-alert">
                <p>{tr(Text::SignInNone)}</p>
            </div>
        }
    });
    let links = (!sign_in.links.is_empty()).then(|| {
        let links = sign_in
            .links
            .into_iter()
            .map(|(label, href)| {
                view! {
                    <a href=href rel="external noopener">
                        {label}
                    </a>
                }
            })
            .collect_view();
        view! { <div class="fui-sign-in-links">{links}</div> }
    });
    view! {
        <div class="fui-card fui-sign-in">
            <div class="fui-sign-in-lockup">{sign_in.lockup}</div>
            {notices}
            {refusal}
            {basic}
            {divider}
            {provider}
            {unavailable}
            {links}
        </div>
    }
}
