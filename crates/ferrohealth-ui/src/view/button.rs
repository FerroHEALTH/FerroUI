// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The buttons: the variants, as classes a `<button>` or an `<a>` takes,
//! and the link that looks like a button.
//!
//! A disabled button keeps its variant's look, faded. When a reason keeps
//! it disabled, say the reason in a help line under it and name that line
//! with `aria-describedby`, so a reader who reaches the button hears why.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr};
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

    /// The variant's name in `locale`, as the gallery labels it.
    #[must_use]
    pub fn name(self, locale: &Locale) -> Word {
        tr(
            locale,
            match self {
                Self::Primary => Text::VariantPrimary,
                Self::Secondary => Text::VariantSecondary,
                Self::Danger => Text::VariantDanger,
                Self::Destructive => Text::VariantDestructive,
                Self::Ghost => Text::VariantGhost,
            },
        )
    }
}

/// Returns the inside of a button: the icon `glyph`, when there is one,
/// beside the word `label`.
#[must_use]
pub fn label(glyph: Option<Glyph>, label: impl Into<Word>) -> Html {
    let label: Word = label.into();
    Html::join([
        glyph.map(|glyph| icon(glyph, "16")).unwrap_or_default(),
        label.into(),
    ])
}

#[derive(Template)]
#[template(
    source = r#"<a href="{{ href }}" class="{{ class }}">{{ inside }}</a>"#,
    ext = "html"
)]
struct LinkButton {
    href: String,
    class: &'static str,
    inside: Html,
}

/// Returns a link to `href` drawn as a button of `variant`, with `glyph`
/// beside its `label`.
#[must_use]
pub fn link_button(
    href: String,
    variant: Variant,
    glyph: Option<Glyph>,
    text: impl Into<String>,
) -> Html {
    LinkButton {
        href,
        class: variant.class(),
        inside: label(glyph, text.into()),
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::{Variant, link_button};
    use crate::icon::Glyph;

    #[test]
    fn a_link_button_carries_its_variant_and_escapes_its_words() {
        let html = link_button(
            String::from("/a?b=1&c=2"),
            Variant::Primary,
            Some(Glyph::PLUS),
            "New <thing>",
        );
        assert!(html.as_str().starts_with(r#"<a href="/a?b=1&amp;c=2" class="fui-button fui-button-primary">"#) || html.as_str().starts_with(r#"<a href="/a?b=1&#38;c=2" class="fui-button fui-button-primary">"#), "{html}");
        assert!(html.as_str().contains("New &lt;thing&gt;</a>"), "{html}");
        assert!(html.as_str().contains("<svg"), "{html}");
    }
}
