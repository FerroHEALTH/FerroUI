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

use crate::app::{FOOTER, PRODUCT, brand_glyph, theme_path};

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
        label: String::from(if variant == Variant::Both {
            "Sign in with OIDC"
        } else {
            "Sign in"
        }),
    });
    let notices = match variant {
        Variant::Basic => vec![
            Notice::new(Tone::Warn, "Your session ended. Sign in again to continue.")
                .id("session-expired"),
            Notice::new(
                Tone::Warn,
                "Could not read the server's sign-in methods, so both are offered.",
            ),
        ],
        Variant::Provider | Variant::Both => vec![Notice::new(
            Tone::Accent,
            "A synthetic deployment.\nSign in with the gallery's own account: synthetic / synthetic.",
        )],
    };
    let refusal = (variant == Variant::Basic).then(|| String::from("Wrong username or password."));
    sign_in(SignIn {
        lockup: lockup("/", mark(brand_glyph(brand)), brand.product(), "Console").into_any(),
        notices,
        refusal,
        basic,
        provider,
        links: vec![
            (String::from("Documentation"), String::from("/")),
            (String::from("API reference"), String::from("/")),
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
        <Title text=format!("Sign in · {} · {PRODUCT}", brand.product()) />
        <Stylesheet id="brand-theme" href=theme_path(brand) />
        {centred(card(brand, variant), FOOTER)}
    }
}
