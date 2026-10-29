// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The action group of a title row (F14): one primary action, then a More
//! menu with the rest.
//!
//! The primary is what this reader can do first. A split primary carries a
//! menu of the variants of its one action. A popover form (R17 anchored)
//! may sit beside the primary when both act on what the page is writing,
//! such as Run and Save to the CDR on a query. More holds every other action,
//! its destructive ones last after a separator in the danger tone, so no
//! danger button ever sits beside the primary (family rule 7). Every action
//! is a link, a plain form post, a submit of a form on the page or a button
//! that opens a confirmation dialog, so the group works with no script; the
//! menus are [`crate::view::action_menu`]'s. No specification governs the
//! layout: our own design.

use std::hash::{DefaultHasher, Hash as _, Hasher as _};

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::Locale;
use crate::view::action_menu::{Act, Action, Menu, more_menu, variants_menu};
use crate::view::button::{Variant, label};
use crate::view::copy::{CopySource, CopyWords, copy_from};
use crate::view::popover::{PopoverForm, popover_form};

/// What a title row offers.
#[derive(Debug, Default)]
pub struct ActionGroup {
    /// The one primary action, when the reader can do one here.
    pub primary: Option<Action>,
    /// The variants of the primary action, which make it a split button;
    /// read only beside a primary.
    pub variants: Option<Menu>,
    /// A popover form beside the primary, such as Save to the CDR beside
    /// Run, its trigger a secondary button.
    pub popover: Option<PopoverForm>,
    /// The More menu with every other action; left out when it holds none.
    pub more: Option<Menu>,
}

impl ActionGroup {
    /// The group with no action, which draws nothing.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// The group of the primary action `action` alone.
    #[must_use]
    pub fn primary(action: Action) -> Self {
        Self {
            primary: Some(action),
            ..Self::default()
        }
    }

    /// The group of a More menu alone, for a page whose reader has no
    /// primary action.
    #[must_use]
    pub fn more_only(menu: Menu) -> Self {
        Self {
            more: Some(menu),
            ..Self::default()
        }
    }

    /// This group, with `menu` as its More menu.
    #[must_use]
    pub fn more(self, menu: Menu) -> Self {
        Self {
            more: Some(menu),
            ..self
        }
    }

    /// This group, with the popover form `form` beside its primary.
    #[must_use]
    pub fn popover(self, form: PopoverForm) -> Self {
        Self {
            popover: Some(form),
            ..self
        }
    }

    /// This group, its primary a split button with `menu` holding the
    /// variants of its action.
    #[must_use]
    pub fn variants(self, menu: Menu) -> Self {
        Self {
            variants: Some(menu),
            ..self
        }
    }

    /// Whether the group draws nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.primary.is_none()
            && self.popover.is_none()
            && self.more.as_ref().is_none_or(Menu::is_empty)
    }
}

/// Returns the action `action` as a button of `variant`: a link, a plain
/// form's submit, a submit of the form it names, or the button that opens
/// its confirmation.
#[must_use]
pub fn action_button(locale: &Locale, action: Action, variant: Variant) -> Html {
    button_of(locale, action, variant.class())
}

/// Returns the action `action` as [`action_button`] draws it, as a small
/// button of `variant`, for a card's header row.
#[must_use]
pub fn small_action_button(locale: &Locale, action: Action, variant: Variant) -> Html {
    button_of(locale, action, variant.small())
}

#[derive(Template)]
#[template(path = "action/group.html", block = "button")]
struct ButtonView {
    act: Act,
    class: &'static str,
    inside: Html,
}

/// The button [`action_button`] and [`small_action_button`] draw, with the
/// classes `class`; a copy is the copy button
/// ([`crate::view::copy::copy_from`]), hidden until its module runs.
fn button_of(locale: &Locale, action: Action, class: &'static str) -> Html {
    if let Act::Copy(text) = action.act {
        let mut hasher = DefaultHasher::new();
        (&action.label, &text).hash(&mut hasher);
        let id = format!("fui-copy-{:x}", hasher.finish());
        return copy_from(
            id,
            action.label,
            CopySource::Text(text),
            CopyWords::of(locale),
        );
    }
    let words = action.words();
    ButtonView {
        inside: label(action.glyph, words),
        act: action.act,
        class,
    }
    .html()
}

#[derive(Template)]
#[template(path = "action/group.html", block = "group")]
struct GroupView {
    primary: Option<Html>,
    variants: Option<Html>,
    popover: Html,
    more: Html,
}

/// Returns the action group `group`: its primary, split when it has
/// variants, its popover form, then its More menu; nothing for a group
/// with no action.
#[must_use]
pub fn action_group(locale: &Locale, group: ActionGroup) -> Html {
    if group.is_empty() {
        return Html::empty();
    }
    let ActionGroup {
        primary,
        variants,
        popover,
        more,
    } = group;
    let variants = variants
        .filter(|menu| !menu.is_empty())
        .zip(primary.as_ref())
        .map(|(menu, primary)| variants_menu(locale, &primary.label, &menu));
    GroupView {
        primary: primary.map(|primary| action_button(locale, primary, Variant::Primary)),
        variants,
        popover: popover.map(popover_form).unwrap_or_default(),
        more: more
            .map(|menu| more_menu(locale, &menu))
            .unwrap_or_default(),
    }
    .html()
}
