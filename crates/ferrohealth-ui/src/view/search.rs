// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The global search box of a top bar: one field and a Find button.
//!
//! It is a plain `GET` form with `role="search"`, so it works before the
//! page hydrates and needs no script; a console's router may take it over.
//! The field's label is visually hidden, so the accessible name is the
//! label and never the placeholder. The component knows nothing of what is
//! searched: the console's route reads the query.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::form::INPUT;

/// What a search box holds.
#[derive(Debug, Clone)]
pub struct SearchBox {
    /// The field's id.
    pub id: &'static str,
    /// The query key the field submits as, such as `q`.
    pub name: &'static str,
    /// The field's label, read by a screen reader.
    pub label: &'static str,
    /// The hint inside the empty field, such as "A code, a canonical, or a
    /// phrase".
    pub placeholder: &'static str,
    /// The console's route the form submits to.
    pub action: String,
    /// The text already searched for, on a result page.
    pub value: String,
}

/// Returns the search box `search`.
#[must_use]
pub fn search_box(search: SearchBox) -> AnyView {
    view! {
        <form role="search" method="get" action=search.action class="fui-search">
            <label for=search.id class="fui-sr-only">
                {search.label}
            </label>
            <input
                id=search.id
                name=search.name
                type="search"
                class=INPUT
                placeholder=search.placeholder
                value=search.value
                autocomplete="off"
                spellcheck="false"
            />
            <button type="submit" class=Variant::Secondary.class()>
                {icon(Glyph::SEARCH, "16")}
                "Find"
            </button>
        </form>
    }
    .into_any()
}
