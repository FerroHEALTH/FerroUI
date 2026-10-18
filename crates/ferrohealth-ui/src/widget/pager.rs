// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The pager of a listing that keeps its state in the page alone.
//!
//! It is the [`crate::table::model::StateSource::Session`] form: the same
//! controls as the link pager, as buttons that set the paging, so a page
//! that can show patient data never puts its position in a URL or the
//! history. A step that cannot be taken is a disabled button that still says
//! what it is and "(unavailable)".

use leptos::prelude::*;

use crate::table::model::{Paging, Window};
use crate::table::view::{Step, pager, step_label};
use crate::view::button::Variant;

/// Returns the pager over `window` named `label`, whose buttons set
/// `paging`.
#[must_use]
pub fn session_pager(
    label: &'static str,
    window: Window,
    paging: RwSignal<Paging>,
) -> impl IntoView {
    let step = move |each: Step, target: Option<u32>| {
        let available = target.is_some();
        view! {
            <button
                type="button"
                class=Variant::Secondary.small()
                disabled=!available
                on:click=move |_| {
                    if let Some(page) = target {
                        paging.update(|paging| paging.page = page);
                    }
                }
            >
                {step_label(each, available)}
            </button>
        }
        .into_any()
    };
    let size = move |choice: u32, current: bool| {
        let variant = if current {
            Variant::Primary
        } else {
            Variant::Ghost
        };
        view! {
            <button
                type="button"
                class=variant.small()
                aria-pressed=current.to_string()
                on:click=move |_| paging.set(Paging::sized(choice))
            >
                {choice.to_string()}
            </button>
        }
        .into_any()
    };
    pager(label, window, &step, &size)
}
