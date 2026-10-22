// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The action group of a title row (F14): one primary action, then a More
//! menu with the rest.
//!
//! The primary is what this reader can do first. A split primary carries a
//! menu of the variants of its one action. More holds every other action,
//! its destructive ones last after a separator in the danger tone, so no
//! danger button ever sits beside the primary (family rule 7). Every action
//! is a link, a plain form post, a submit of a form on the page or a button
//! that opens a confirmation dialog, so the group works with no
//! WebAssembly; the menus are [`crate::widget::menu`]'s. No specification
//! governs the layout: our own design.

use leptos::prelude::*;

use crate::view::button::{Variant, label};
use crate::widget::action_menu::{Act, Action, Menu, more_menu, variants_menu};

/// What a title row offers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActionGroup {
    /// The one primary action, when the reader can do one here.
    pub primary: Option<Action>,
    /// The variants of the primary action, which make it a split button;
    /// read only beside a primary.
    pub variants: Option<Menu>,
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
        self.primary.is_none() && self.more.as_ref().is_none_or(Menu::is_empty)
    }
}

/// Returns the action `action` as a button of `variant`: a link, a plain
/// form's submit, a submit of the form it names, or the button that opens
/// its confirmation.
#[must_use]
pub fn action_button(action: Action, variant: Variant) -> AnyView {
    button_of(action, variant.class())
}

/// Returns the action `action` as [`action_button`] draws it, as a small
/// button of `variant`, for a card's header row.
#[must_use]
pub fn small_action_button(action: Action, variant: Variant) -> AnyView {
    button_of(action, variant.small())
}

/// The button [`action_button`] and [`small_action_button`] draw, with the
/// classes `class`.
fn button_of(action: Action, class: &'static str) -> AnyView {
    let words = action.words();
    let inside = label(action.glyph, words);
    match action.act {
        Act::Link(href) => view! {
            <a href=href class=class>
                {inside}
            </a>
        }
        .into_any(),
        Act::Post { action, fields } => {
            let fields = fields
                .into_iter()
                .map(|(name, value)| view! { <input type="hidden" name=name value=value /> })
                .collect_view();
            view! {
                <form method="post" action=action class="fui-action-form">
                    {fields}
                    <button type="submit" class=class>
                        {inside}
                    </button>
                </form>
            }
            .into_any()
        }
        Act::Submit(form) => view! {
            <button type="submit" form=form class=class>
                {inside}
            </button>
        }
        .into_any(),
        Act::Dialog(target) => view! {
            <button
                type="button"
                class=class
                command="show-modal"
                commandfor=target.clone()
                aria-controls=target
                aria-haspopup="dialog"
            >
                {inside}
            </button>
        }
        .into_any(),
    }
}

/// Returns the action group `group`: its primary, split when it has
/// variants, then its More menu; nothing for a group with no action.
#[must_use]
pub fn action_group(group: ActionGroup) -> AnyView {
    if group.is_empty() {
        return ().into_any();
    }
    let ActionGroup {
        primary,
        variants,
        more,
    } = group;
    let primary = primary.map(|primary| {
        let variants = variants.filter(|menu| !menu.is_empty());
        match variants {
            Some(menu) => {
                let name = primary.label.clone();
                let button = action_button(primary, Variant::Primary);
                view! { <span class="fui-split-button">{button} {variants_menu(&name, menu)}</span> }
                    .into_any()
            }
            None => action_button(primary, Variant::Primary),
        }
    });
    view! {
        <div class="fui-action-group">
            {primary}
            {more.map(more_menu)}
        </div>
    }
    .into_any()
}
