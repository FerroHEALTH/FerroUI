// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The find field over a listing: a form the server answers.
//!
//! The field is the toolbar's find field (`design/COMPONENTS.md`, T1 and
//! § The table): a `GET` form whose words land in the query, so the server
//! draws the rows that match and the address bar keeps the filter, or a
//! `POST` form when the words can be clinical, which never go in a URL. It
//! filters the whole listing on the server, works with no script, and
//! sends no row to the browser as island data (Leptos book, `router/20_form`
//! and `progressive_enhancement`, "URL-driven state"). Enter submits it: a
//! form with one text field submits on Enter with no button (HTML Living
//! Standard, implicit submission).

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::table::model::{PAGE_KEY, StateSource};
use crate::table::view::{hidden_fields, method};
use crate::view::form::INPUT;

/// What a find field shows and where it submits.
#[derive(Debug, Clone, Copy)]
pub struct Find<'a> {
    /// The field's id, unique on the page.
    pub id: &'a str,
    /// The field's name: the query key of a `GET` form (`q`), or the
    /// argument of the console's handler.
    pub name: &'a str,
    /// What the field finds, its label for assistive technology and its
    /// placeholder ("File, template or path").
    pub label: &'a str,
    /// The words in force, which the field shows.
    pub value: &'a str,
    /// Where the listing keeps its filter: [`StateSource::Url`] submits a
    /// `GET` form; [`StateSource::Session`] posts it, for words that can
    /// be clinical (`.claude/rules/leptos-ui.md` §5).
    pub source: StateSource,
    /// The URL the form submits to; `None` submits to the page's own
    /// address.
    pub action: Option<&'a str>,
    /// The other fields the form carries, such as the listing's size and
    /// sort. The field's own name and `page` among them are left out: a new
    /// filter starts from the first page.
    pub fields: &'a [(String, String)],
}

/// Returns the toolbar over a listing with its find field `find`, and
/// `tools` beside it (filter menus, a toggle).
///
/// The tools must add no second text field, or a submit button with them,
/// since a form with two text fields does not submit on Enter.
#[must_use]
pub fn find_form(find: Find<'_>, tools: impl IntoView + 'static) -> AnyView {
    toolbar(find, tools.into_any())
}

/// The toolbar [`find_form`] draws, one concrete body for every set of tools.
fn toolbar(find: Find<'_>, tools: AnyView) -> AnyView {
    let carried = hidden_fields(find.fields, &[find.name, PAGE_KEY]);
    let id = find.id.to_owned();
    let label = find.label.to_owned();
    view! {
        <form
            role="search"
            method=method(find.source)
            action=find.action.map(str::to_owned)
            class="fui-list-toolbar"
        >
            {carried}
            <div class="fui-list-toolbar-start">
                <div class="fui-list-find">
                    <span class="fui-search-glyph">{icon(Glyph::SEARCH, "16")}</span>
                    <label for=id.clone() class="fui-sr-only">
                        {label.clone()}
                    </label>
                    <input
                        id=id
                        name=find.name.to_owned()
                        type="search"
                        class=INPUT
                        value=find.value.to_owned()
                        placeholder=label
                        autocomplete="off"
                        spellcheck="false"
                        enterkeyhint="search"
                    />
                </div>
                {tools}
            </div>
        </form>
    }
    .into_any()
}
