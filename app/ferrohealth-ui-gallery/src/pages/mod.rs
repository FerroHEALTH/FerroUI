// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's pages.
//!
//! They are the overview of the static views, the forms and the editor, the
//! tables and their paging, the code panels, the operation parts, and the
//! sign-in cards. Each
//! page is built from erased sections, so rustc sees a shallow tree.

pub mod charts;
pub mod code;
pub mod dock;
pub mod forms;
pub mod layouts;
pub mod operations;
pub mod overlays;
pub mod overview;
pub mod preferences;
pub mod records;
pub mod sign_in;
pub mod tables;
pub mod templates;
pub mod trees;

use crate::locale::{Msg, tr};
use ferrohealth_ui::style::Brand;
use leptos::prelude::*;
use leptos_meta::Title;

/// The root's one title format: a page's words, then the product; a page
/// that sets no words of its own is titled by the product alone.
pub fn title_format(product: String) -> impl Fn(String) -> String + Send + Sync + 'static {
    move |page| {
        if page.is_empty() {
            product.clone()
        } else {
            format!("{page} · {product}")
        }
    }
}

/// The document title of the page `name` in the brand `brand`, the
/// gallery's for none, which the root's [`title_format`] completes.
#[must_use]
pub fn title_in(name: impl Into<String>, brand: Option<Brand>) -> impl IntoView {
    let name = name.into();
    let brand = brand.map_or(tr(Msg::Gallery), |brand| brand.product().to_owned());
    view! { <Title text=format!("{name} · {brand}") /> }
}

/// The document title of the page `name` in the brand the route names.
#[must_use]
pub fn title(name: impl Into<String>) -> impl IntoView {
    title_in(name, crate::app::route_brand())
}
