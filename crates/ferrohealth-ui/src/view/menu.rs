// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The operator's menu: a button with the operator's name, opening an HTML
//! popover the browser opens, closes on Escape and returns the focus from,
//! with no script (HTML, the `popover` attribute).

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::view::button::Variant;

/// What the operator's menu shows.
#[derive(Debug, Clone)]
pub struct OperatorMenu {
    /// The menu's id, which its button opens.
    pub id: &'static str,
    /// The operator's name.
    pub name: String,
    /// The line under the name, such as "Signed in to this console".
    pub subtitle: &'static str,
    /// The console's sign-out route, a plain form post it answers with a
    /// redirect, or `None` for a console with no sign-in.
    pub sign_out: Option<String>,
}

/// Returns the operator's menu button and its popover, with `extra` (such as
/// the theme switch on a narrow screen) above the sign-out.
#[must_use]
pub fn operator_menu(menu: OperatorMenu, extra: impl IntoView) -> impl IntoView {
    let sign_out = menu.sign_out.map(|action| {
        let class = format!("{} fui-sign-out", Variant::Secondary.class());
        view! {
            <form method="post" action=action>
                <button type="submit" class=class>
                    {icon(Glyph::LOG_OUT, "16")}
                    "Sign out"
                </button>
            </form>
        }
    });
    view! {
        <button
            type="button"
            class=Variant::Secondary.class()
            popovertarget=menu.id
            aria-controls=menu.id
        >
            {icon(Glyph::CIRCLE_USER, "16")}
            <span>{menu.name.clone()}</span>
            {icon(Glyph::CHEVRON_DOWN, "14")}
        </button>
        <div id=menu.id popover="auto" class="fui-menu">
            <div>
                <p class="fui-menu-title">{menu.name}</p>
                <p class="fui-muted">{menu.subtitle}</p>
            </div>
            {extra}
            {sign_out}
        </div>
    }
}
