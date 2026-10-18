// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The pagers of a listing that keeps its state in the page alone.
//!
//! They are the [`crate::table::model::StateSource::Session`] form: the same
//! controls as the link pagers, as buttons that set the paging, so a page
//! that can show patient data never puts its position in a URL or the
//! history. A step that cannot be taken is a disabled button that still says
//! what it is and "(unavailable)".

use leptos::prelude::*;

use crate::table::model::{OpenWindow, Paging, Window};
use crate::table::view::{Listing, Step, open_pager, pager, step_label};
use crate::view::button::Variant;

/// Returns the step `each` as a button that sets `paging` to `target`,
/// disabled when there is no target.
fn session_step(paging: RwSignal<Paging>, each: Step, target: Option<u32>) -> AnyView {
    let available = target.is_some();
    view! {
        <button
            type="button"
            class=Variant::Secondary.small()
            data-page=each.key()
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
}

/// Returns the size `choice` as a button that sets `paging` to its first
/// page.
fn session_size(paging: RwSignal<Paging>, choice: u32, current: bool) -> AnyView {
    view! {
        <button
            type="button"
            class="fui-pager-size"
            aria-pressed=current.to_string()
            on:click=move |_| paging.set(Paging::sized(choice))
        >
            {choice.to_string()}
        </button>
    }
    .into_any()
}

/// Returns the pager over `window` of `listing`, whose buttons set
/// `paging`.
#[must_use]
pub fn session_pager(
    listing: impl Into<Listing>,
    window: Window,
    paging: RwSignal<Paging>,
) -> impl IntoView {
    pager(
        listing,
        window,
        &move |each, target| session_step(paging, each, target),
        &move |choice, current| session_size(paging, choice, current),
    )
}

/// Returns the pager over `window`, a listing whose total the server does
/// not report, whose buttons set `paging`.
#[must_use]
pub fn session_open_pager(
    listing: impl Into<Listing>,
    window: OpenWindow,
    paging: RwSignal<Paging>,
) -> impl IntoView {
    open_pager(
        listing,
        window,
        &move |each, target| session_step(paging, each, target),
        &move |choice, current| session_size(paging, choice, current),
    )
}
