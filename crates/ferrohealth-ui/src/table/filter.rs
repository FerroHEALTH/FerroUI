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
//!
//! A filter menu ([`filter_menu`]) keeps the rows of several choices at
//! once: a `<details>` whose summary is its button, holding a checkbox per
//! choice with the number of rows it keeps and a submit, in a `GET` form of
//! its own or in the find field's form, so it works with no script. F9's
//! Status menu is one, its states as the choices.

use leptos::prelude::*;

use crate::i18n::number::grouped;
use crate::i18n::{Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::table::model::{PAGE_KEY, StateSource};
use crate::table::view::{hidden_fields, method};
use crate::view::button::Variant;
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

/// One choice of a filter menu: a checkbox with its words and the number
/// of rows it keeps.
#[derive(Debug)]
pub struct FilterChoice {
    /// The value the checkbox sends under the menu's name, such as `loinc`.
    pub value: String,
    /// The choice's words, such as "LOINC".
    pub label: Word,
    /// How many rows the choice keeps.
    pub count: u64,
    /// Whether the listing in force keeps the choice's rows.
    pub checked: bool,
    /// A mark before the words, hidden or named by itself, such as a
    /// state's line and dot; none for a plain choice.
    pub mark: Option<AnyView>,
}

impl FilterChoice {
    /// A plain choice `value`, worded `label`, keeping `count` rows.
    #[must_use]
    pub fn new(
        value: impl Into<String>,
        label: impl Into<Word>,
        count: u64,
        checked: bool,
    ) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            count,
            checked,
            mark: None,
        }
    }

    /// The choice with `mark` drawn before its words.
    #[must_use]
    pub fn mark(self, mark: impl IntoView + 'static) -> Self {
        Self {
            mark: Some(mark.into_any()),
            ..self
        }
    }
}

/// Where a filter menu's checkboxes are sent.
#[derive(Debug, Clone, Copy)]
pub enum MenuForm<'a> {
    /// A `GET` form of the menu's own, to `action` (the page's own address
    /// when `None`), carrying `fields` (the listing's find words, size and
    /// sort); the menu's name and `page` among them are left out, since a
    /// new filter starts from the first page.
    Own {
        /// The URL the form gets.
        action: Option<&'a str>,
        /// The other fields the form carries.
        fields: &'a [(String, String)],
    },
    /// The form around the menu, such as [`find_form`]'s when the menu is
    /// one of its tools: its submit sends the find words too.
    Enclosing,
}

/// What a filter menu offers.
#[derive(Debug)]
pub struct FilterMenu<'a> {
    /// The menu's id; each checkbox is `id`, a hyphen and its value.
    pub id: &'a str,
    /// The name each checked choice is sent under, such as `source`.
    pub name: &'a str,
    /// The glyph before the button's words: [`Glyph::LIST_FILTER`] for a
    /// listing's menu.
    pub glyph: Glyph,
    /// What the menu filters by, its button's words, such as "Source".
    pub label: Word,
    /// The menu's accessible name, such as "Show sources".
    pub title: String,
    /// The choices, in the order the menu lists them.
    pub choices: Vec<FilterChoice>,
    /// A quiet line above the submit, such as a legend; none for most.
    pub note: Option<Word>,
    /// Where the checkboxes are sent.
    pub form: MenuForm<'a>,
}

/// Returns the filter menu `menu` (T1's filter menu, F9's Status menu): a
/// `<details>` whose summary is the button, holding each choice as a
/// checkbox with its count and a submit that shows the checked ones.
///
/// While some choice is unchecked the button says how many are kept ("Source
/// · 2 of 5") and takes the accent. It works with no script, since the
/// summary opens it and the submit sends the form (HTML Living Standard,
/// the `details` element and form submission).
#[must_use]
pub fn filter_menu(menu: FilterMenu<'_>) -> AnyView {
    let FilterMenu {
        id,
        name,
        glyph,
        label,
        title,
        choices,
        note,
        form,
    } = menu;
    let total = choices.len();
    let kept = choices.iter().filter(|choice| choice.checked).count();
    let filtered = kept < total;
    let words: Word = if filtered {
        tr_with(
            Text::FilterMenuSome,
            &[
                ("label", label.text().into()),
                ("shown", kept.into()),
                ("total", total.into()),
            ],
        )
    } else {
        label
    };
    let items = choices
        .into_iter()
        .map(|choice| {
            let input_id = format!("{id}-{}", choice.value);
            let label_for = input_id.clone();
            view! {
                <label class="fui-menu-item fui-menu-check" for=label_for>
                    <input
                        type="checkbox"
                        id=input_id
                        name=name.to_owned()
                        value=choice.value
                        checked=choice.checked
                    />
                    {choice.mark}
                    <span class="fui-menu-item-text">
                        <span class="fui-menu-item-label">{choice.label}</span>
                    </span>
                    <span class="fui-menu-count">{grouped(choice.count)}</span>
                </label>
            }
        })
        .collect_view();
    let body = view! {
        <div class="fui-menu-items">{items}</div>
        <div class="fui-menu-sep" role="separator"></div>
        <div class="fui-menu-foot">
            {note.map(|note| view! { <p class="fui-small fui-muted">{note}</p> })}
            <button type="submit" class=Variant::Secondary.small()>
                {tr(Text::FilterMenuShow)}
            </button>
        </div>
    }
    .into_any();
    let class = "fui-menu fui-menu-compact fui-menu-anchored fui-menu-start";
    let panel = match form {
        MenuForm::Own { action, fields } => {
            let carried = hidden_fields(fields, &[name, PAGE_KEY]);
            view! {
                <form method="get" action=action.map(str::to_owned) class=class aria-label=title>
                    {carried}
                    {body}
                </form>
            }
            .into_any()
        }
        MenuForm::Enclosing => view! {
            <div role="group" class=class aria-label=title>
                {body}
            </div>
        }
        .into_any(),
    };
    view! {
        <details class="fui-anchor">
            <summary class=Variant::Secondary.class() class:is-filtered=filtered>
                {icon(glyph, "16")}
                {words}
                {icon(Glyph::CHEVRON_DOWN, "16")}
            </summary>
            {panel}
        </details>
    }
    .into_any()
}
