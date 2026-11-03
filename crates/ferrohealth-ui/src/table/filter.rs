// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The find field over a listing: a form the server answers.
//!
//! The field is the toolbar's find field (`design/COMPONENTS.md`, T1 and
//! § The table): a `GET` form whose words land in the query, so the server
//! draws the rows that match and the address bar keeps the filter, or a
//! `POST` form when the words can be clinical, which never go in a URL. It
//! filters the whole listing on the server and works with no script; Enter
//! submits it (HTML Living Standard, implicit submission).
//!
//! A filter menu ([`crate::view::filter_menu`]) keeps the rows of several choices at
//! once: a `<details>` whose summary is its button, holding a checkbox per
//! choice with the number of rows it keeps and a submit, in a `GET` form of
//! its own or in the find field's form.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::icon::{Glyph, icon};
use crate::query::PAGE_KEY;
use crate::table::model::StateSource;
use crate::table::view::method;
use crate::view::form::INPUT;
use crate::view::form::hidden_fields;

/// What a find field shows and where it submits.
#[derive(Debug, Clone, Copy)]
pub struct Find<'a> {
    /// The field's id, unique on the page.
    pub id: &'a str,
    /// The field's name: the query key of a `GET` form (`q`), or the
    /// argument of the console's handler.
    pub name: &'a str,
    /// What the field finds, its label for assistive technology and its
    /// placeholder.
    pub label: &'a str,
    /// The words in force, which the field shows.
    pub value: &'a str,
    /// Where the listing keeps its filter: [`StateSource::Url`] submits a
    /// `GET` form; [`StateSource::Session`] posts it, for words that can
    /// be clinical.
    pub source: StateSource,
    /// The URL the form submits to; `None` submits to the page's own
    /// address.
    pub action: Option<&'a str>,
    /// The other fields the form carries, such as the listing's size and
    /// sort. The field's own name and `page` among them are left out.
    pub fields: &'a [(String, String)],
}

#[derive(Template)]
#[template(
    source = r#"<form role="search" method="{{ method }}"{% if let Some(action) = action %} action="{{ action }}"{% endif %} class="fui-list-toolbar">{{ carried }}<div class="fui-list-toolbar-start"><div class="fui-list-find"><span class="fui-search-glyph">{{ glyph }}</span><label for="{{ id }}" class="fui-sr-only">{{ label }}</label><input id="{{ id }}" name="{{ name }}" type="search" class="{{ input }}" value="{{ value }}" placeholder="{{ label }}" autocomplete="off" spellcheck="false" enterkeyhint="search"></div>{{ tools }}</div></form>"#,
    ext = "html"
)]
struct Toolbar {
    method: &'static str,
    action: Option<String>,
    carried: Html,
    glyph: Html,
    id: String,
    label: String,
    name: String,
    input: &'static str,
    value: String,
    tools: Html,
}

/// Returns the toolbar over a listing with its find field `find`, and
/// `tools` beside it (filter menus, a toggle).
///
/// The tools must add no second text field, since a form with two text
/// fields does not submit on Enter.
#[must_use]
pub fn find_form(find: Find<'_>, tools: Html) -> Html {
    Toolbar {
        method: method(find.source),
        action: find.action.map(str::to_owned),
        carried: hidden_fields(find.fields, &[find.name, PAGE_KEY]),
        glyph: icon(Glyph::SEARCH, "16"),
        id: find.id.to_owned(),
        label: find.label.to_owned(),
        name: find.name.to_owned(),
        input: INPUT,
        value: find.value.to_owned(),
        tools,
    }
    .html()
}

#[derive(Template)]
#[template(
    source = r#"<form role="search" method="{{ method }}"{% if let Some(action) = action %} action="{{ action }}"{% endif %} class="fui-browser-find">{{ carried }}<div class="fui-list-find"><span class="fui-search-glyph">{{ glyph }}</span><label for="{{ id }}" class="fui-sr-only">{{ label }}</label><input id="{{ id }}" name="{{ name }}" type="search" class="{{ input }}" value="{{ value }}" placeholder="{{ label }}" autocomplete="off" spellcheck="false" enterkeyhint="search"></div></form>"#,
    ext = "html"
)]
struct BrowserFindView<'a> {
    method: &'static str,
    action: Option<&'a str>,
    carried: Html,
    glyph: Html,
    id: &'a str,
    label: &'a str,
    name: &'a str,
    input: &'static str,
    value: &'a str,
}

/// Returns the find field at the top of a pane of the concept browser
/// ([`crate::view::browser`]).
///
/// It is one search field with the glyph inside, its label for assistive
/// technology alone, submitting on Enter as `find` says (a `GET` form, or a
/// `POST` form for words that can be clinical).
#[must_use]
pub fn browser_find(find: Find<'_>) -> Html {
    BrowserFindView {
        method: method(find.source),
        action: find.action,
        carried: hidden_fields(find.fields, &[find.name, PAGE_KEY]),
        glyph: icon(Glyph::SEARCH, "16"),
        id: find.id,
        label: find.label,
        name: find.name,
        input: INPUT,
        value: find.value,
    }
    .html()
}
