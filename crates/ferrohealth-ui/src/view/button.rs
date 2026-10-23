// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The buttons: the variants, as classes a `<button>` or an `<a>` takes,
//! and the link that looks like a button.
//!
//! A disabled button keeps its variant's look, faded. When a reason keeps
//! it disabled, say the reason in a help line under it and name that line
//! with `aria-describedby`, so a reader who reaches the button hears why.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, Word, tr};
use crate::icon::{Glyph, icon};

/// What a button is for, which sets how much it stands out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Variant {
    /// The one action a screen is for.
    Primary,
    /// An action beside the primary one.
    #[default]
    Secondary,
    /// An action that removes or refuses something: the outline an inline
    /// remove takes, such as "Remove this clause" in a form.
    Danger,
    /// The confirmation of an action that removes something, in a danger
    /// dialog alone: solid, so the destructive step is unmistakable. An
    /// inline remove takes [`Variant::Danger`].
    Destructive,
    /// An action that does not stand out: a close, a dismiss.
    Ghost,
}

impl Variant {
    /// Every variant, in the order the gallery shows them.
    pub const ALL: [Self; 5] = [
        Self::Primary,
        Self::Secondary,
        Self::Danger,
        Self::Destructive,
        Self::Ghost,
    ];

    /// The classes a `<button>` or an `<a>` of this variant takes.
    #[must_use]
    pub const fn class(self) -> &'static str {
        match self {
            Self::Primary => "fui-button fui-button-primary",
            Self::Secondary => "fui-button fui-button-secondary",
            Self::Danger => "fui-button fui-button-danger",
            Self::Destructive => "fui-button fui-button-destructive",
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
            Self::Destructive => "fui-button fui-button-destructive fui-button-small",
            Self::Ghost => "fui-button fui-button-ghost fui-button-small",
        }
    }

    /// The variant's name in the page's locale, as the gallery labels it.
    #[must_use]
    pub fn name(self) -> Word {
        tr(match self {
            Self::Primary => Text::VariantPrimary,
            Self::Secondary => Text::VariantSecondary,
            Self::Danger => Text::VariantDanger,
            Self::Destructive => Text::VariantDestructive,
            Self::Ghost => Text::VariantGhost,
        })
    }
}

/// Returns the inside of a button: the icon `glyph`, when there is one,
/// beside the word `label`.
#[must_use]
pub fn label<L: Into<Word>>(glyph: Option<Glyph>, label: L) -> impl IntoView + use<L> {
    let label: Word = label.into();
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
    text: impl Into<String>,
) -> AnyView {
    let text = text.into();
    view! {
        <a href=href class=variant.class()>
            {label(glyph, text)}
        </a>
    }
    .into_any()
}
