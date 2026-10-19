// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The sign-in cards of a brand: the basic form, the provider's button, and
//! both methods together, each with a notice; the basic card also shows a
//! refusal.

use ferrohealth_ui::style::Brand;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::shell::{centred, lockup, mark};
use ferrohealth_ui::view::sign_in::{Basic, Notice, Provider, SignIn, sign_in};
use leptos::prelude::*;
use leptos_meta::{Stylesheet, Title};
use leptos_router::hooks::use_params_map;

use crate::app::{brand_glyph, footer, product, theme_path};
use crate::locale::{Msg, tr};

/// Which methods a sign-in card shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variant {
    /// The basic form alone.
    Basic,
    /// The provider's button alone.
    Provider,
    /// Both, with "or" between them.
    Both,
}

impl Variant {
    /// The variant `key` names.
    fn parse(key: &str) -> Option<Self> {
        match key {
            "basic" => Some(Self::Basic),
            "provider" => Some(Self::Provider),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

/// The card of `brand` with the methods of `variant`.
fn card(brand: Brand, variant: Variant) -> AnyView {
    let basic = matches!(variant, Variant::Basic | Variant::Both).then(|| Basic {
        action: "#",
        next: format!("/{}", brand.key()),
    });
    let provider = matches!(variant, Variant::Provider | Variant::Both).then(|| Provider {
        href: format!("/{}", brand.key()),
        label: if variant == Variant::Both {
            tr(Msg::SignOidc)
        } else {
            tr(Msg::SignIn)
        },
    });
    let notices = match variant {
        Variant::Basic => vec![
            Notice::new(Tone::Warn, tr(Msg::YourSessionEndedSignAgain)).id("session-expired"),
            Notice::new(Tone::Warn, tr(Msg::CouldNotReadServerS)),
        ],
        Variant::Provider | Variant::Both => {
            vec![Notice::new(Tone::Accent, tr(Msg::SyntheticDeployment))]
        }
    };
    let refusal = (variant == Variant::Basic).then(|| tr(Msg::WrongUsernamePassword));
    sign_in(SignIn {
        lockup: lockup(
            "/",
            mark(brand_glyph(brand)),
            brand.product(),
            tr(Msg::Console),
        )
        .into_any(),
        notices,
        refusal,
        basic,
        provider,
        links: vec![
            (tr(Msg::Documentation), String::from("/")),
            (tr(Msg::ApiReference), String::from("/")),
        ],
    })
    .into_any()
}

/// The sign-in page of the brand and the variant the route names.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn SignInPage() -> impl IntoView {
    let params = use_params_map();
    let named = params.with_untracked(|params| {
        let brand = params.get_str("brand").and_then(Brand::parse)?;
        let variant = params.get_str("variant").and_then(Variant::parse)?;
        Some((brand, variant))
    });
    let (brand, variant) = named.unwrap_or((Brand::Bridge, Variant::Both));
    view! {
        <Title text=format!("{} · {} · {}", tr(Msg::SignIn), brand.product(), product()) />
        <Stylesheet id="brand-theme" href=theme_path(brand) />
        {centred(card(brand, variant), footer())}
    }
}
