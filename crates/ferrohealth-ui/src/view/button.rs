// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The buttons: four variants, as classes a `<button>` or an `<a>` takes,
//! and the link that looks like a button.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};

/// What a button is for, which sets how much it stands out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
    /// The one action a screen is for.
    Primary,
    /// An action beside the primary one.
    #[default]
    Secondary,
    /// An action that removes or refuses something.
    Danger,
    /// An action that does not stand out: a close, a dismiss.
    Ghost,
}

impl Variant {
    /// Every variant, in the order the gallery shows them.
    pub const ALL: [Self; 4] = [Self::Primary, Self::Secondary, Self::Danger, Self::Ghost];

    /// The classes a `<button>` or an `<a>` of this variant takes.
    #[must_use]
    pub const fn class(self) -> &'static str {
        match self {
            Self::Primary => "fui-button fui-button-primary",
            Self::Secondary => "fui-button fui-button-secondary",
            Self::Danger => "fui-button fui-button-danger",
            Self::Ghost => "fui-button fui-button-ghost",
        }
    }

    /// The classes of a small button of this variant, for a table row.
    #[must_use]
    pub const fn small(self) -> &'static str {
        match self {
            Self::Primary => "fui-button fui-button-primary fui-button-small",
            Self::Secondary => "fui-button fui-button-secondary fui-button-small",
            Self::Danger => "fui-button fui-button-danger fui-button-small",
            Self::Ghost => "fui-button fui-button-ghost fui-button-small",
        }
    }

    /// The variant's name, as the gallery labels it.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Danger => "Danger",
            Self::Ghost => "Ghost",
        }
    }
}

/// Returns the inside of a button: the icon `glyph`, when there is one,
/// beside the word `label`.
#[must_use]
pub fn label(glyph: Option<Glyph>, label: &'static str) -> impl IntoView + use<> {
    view! {
        {glyph.map(|glyph| icon(glyph, "16"))}
        {label}
    }
}

/// Returns a link to `href` drawn as a button of `variant`, with `glyph`
/// beside its `label`.
#[must_use]
pub fn link_button(
    href: String,
    variant: Variant,
    glyph: Option<Glyph>,
    text: &'static str,
) -> impl IntoView + use<> {
    view! {
        <a href=href class=variant.class()>
            {label(glyph, text)}
        </a>
    }
}
