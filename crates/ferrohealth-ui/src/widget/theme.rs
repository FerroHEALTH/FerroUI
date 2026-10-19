// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The theme toggle that swaps the theme in place.
//!
//! It draws the markup of [`crate::view::theme_switch::theme_toggle`], so
//! before the page hydrates a press posts the form and the console's server
//! keeps the choice in its cookie. Once hydrated, a press sets `data-theme`
//! on `<html>` at once, moves the focus to the button that now shows, and
//! hands the theme to `on_choose`, where the console persists it in the
//! same cookie (the library fetches nothing and touches no browser storage).

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::theme::Theme;
use crate::view::theme_switch::{TO_DARK, TO_LIGHT};

/// Sets `theme` on `<html>` and moves the focus to the button `then`.
fn swap(theme: Theme, then: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let root = document().document_element();
        if let Some(root) = root
            && root.set_attribute("data-theme", theme.key()).is_err()
        {
            leptos::logging::warn!("the browser refused the theme on <html>");
        }
        let button = document()
            .get_element_by_id(then)
            .and_then(|button| button.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(button) = button
            && button.focus().is_err()
        {
            leptos::logging::warn!("the browser refused the focus on the theme toggle");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (Theme, &str) = (theme, then);
}

/// Returns the theme toggle posting to `action` with the page `back`,
/// which swaps the theme in place once hydrated and reports it to
/// `on_choose`.
#[must_use]
pub fn theme_toggle(action: String, back: String, on_choose: Callback<Theme>) -> AnyView {
    let to_dark = tr(Text::ThemeToDark);
    let to_light = tr(Text::ThemeToLight);
    let choose = move |theme: Theme, then: &'static str| {
        move |event: leptos::ev::MouseEvent| {
            event.prevent_default();
            swap(theme, then);
            on_choose.run(theme);
        }
    };
    view! {
        <form method="post" action=action class="fui-theme-toggle">
            <input type="hidden" name="back" value=back />
            <button
                type="submit"
                id=TO_DARK
                name="theme"
                value=Theme::Dark.key()
                class="fui-icon-button fui-theme-to-dark"
                aria-label=to_dark.clone()
                title=to_dark
                on:click=choose(Theme::Dark, TO_LIGHT)
            >
                {icon(Glyph::MOON, "16")}
            </button>
            <button
                type="submit"
                id=TO_LIGHT
                name="theme"
                value=Theme::Light.key()
                class="fui-icon-button fui-theme-to-light"
                aria-label=to_light.clone()
                title=to_light
                on:click=choose(Theme::Light, TO_DARK)
            >
                {icon(Glyph::SUN, "16")}
            </button>
        </form>
    }
    .into_any()
}
