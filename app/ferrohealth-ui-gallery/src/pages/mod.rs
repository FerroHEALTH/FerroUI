// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's pages.
//!
//! They are the overview of the static views, the forms and the editor, the
//! tables and their paging, the code panels, and the sign-in cards. Each
//! page is built from erased sections, so rustc sees a shallow tree.

pub mod charts;
pub mod code;
pub mod forms;
pub mod layouts;
pub mod overlays;
pub mod overview;
pub mod sign_in;
pub mod tables;
pub mod templates;

use crate::locale::{Msg, tr};
use leptos::prelude::*;
use leptos_meta::Title;

/// The title of a page named `name` in the brand the route names.
#[must_use]
pub fn titled(name: &str) -> String {
    let brand =
        crate::app::route_brand().map_or(tr(Msg::Gallery), |brand| brand.product().to_owned());
    format!("{name} · {brand} · {}", crate::app::product())
}

/// The document title of the page `name`.
#[must_use]
pub fn title(name: impl Into<String>) -> impl IntoView {
    let name = name.into();
    view! { <Title text=titled(&name) /> }
}
