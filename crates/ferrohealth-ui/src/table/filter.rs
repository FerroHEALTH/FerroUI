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
//! A filter menu ([`filter_menu`]) keeps the rows of several choices at
//! once: a `<details>` whose summary is its button, holding a checkbox per
//! choice with the number of rows it keeps and a submit, in a `GET` form of
//! its own or in the find field's form.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::number::grouped;
use crate::i18n::{Locale, Text, Word, tr, tr_with};
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
    /// A mark before the words, such as a state's dot; none for a plain
    /// choice.
    pub mark: Option<Html>,
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
    pub fn mark(self, mark: Html) -> Self {
        Self {
            mark: Some(mark),
            ..self
        }
    }
}

/// Where a filter menu's checkboxes are sent.
#[derive(Debug, Clone, Copy)]
pub enum MenuForm<'a> {
    /// A `GET` form of the menu's own, to `action` (the page's own address
    /// when `None`), carrying `fields`; the menu's name and `page` among
    /// them are left out.
    Own {
        /// The URL the form gets.
        action: Option<&'a str>,
        /// The other fields the form carries.
        fields: &'a [(String, String)],
    },
    /// The form around the menu, such as [`find_form`]'s when the menu is
    /// one of its tools.
    Enclosing,
}

/// What a filter menu offers.
#[derive(Debug)]
pub struct FilterMenu<'a> {
    /// The menu's id; each checkbox is `id`, a hyphen and its value.
    pub id: &'a str,
    /// The name each checked choice is sent under, such as `source`.
    pub name: &'a str,
    /// The glyph before the button's words.
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

#[derive(Template)]
#[template(
    source = r#"<label class="fui-menu-item fui-menu-check" for="{{ id }}"><input type="checkbox" id="{{ id }}" name="{{ name }}" value="{{ value }}"{% if checked %} checked{% endif %}>{% if let Some(mark) = mark %}{{ mark }}{% endif %}<span class="fui-menu-item-text"><span class="fui-menu-item-label">{{ label }}</span></span><span class="fui-menu-count">{{ count }}</span></label>"#,
    ext = "html"
)]
struct ChoiceView {
    id: String,
    name: String,
    value: String,
    checked: bool,
    mark: Option<Html>,
    label: Word,
    count: String,
}

#[derive(Template)]
#[template(
    source = r#"<details data-fui="menu" class="fui-anchor"><summary class="{{ button }}{% if filtered %} is-filtered{% endif %}">{{ glyph }}{{ words }}{{ chevron }}</summary>{% if own %}<form method="get"{% if let Some(action) = action %} action="{{ action }}"{% endif %} class="{{ class }}" aria-label="{{ title }}">{{ carried }}{{ body }}</form>{% else %}<div role="group" class="{{ class }}" aria-label="{{ title }}">{{ body }}</div>{% endif %}</details>"#,
    ext = "html"
)]
struct MenuView {
    button: &'static str,
    filtered: bool,
    glyph: Html,
    words: Word,
    chevron: Html,
    own: bool,
    action: Option<String>,
    class: &'static str,
    title: String,
    carried: Html,
    body: Html,
}

/// Returns the filter menu `menu` (T1's filter menu, F9's Status menu).
///
/// It is a `<details>` whose summary is the button, holding each choice as
/// a checkbox with its count and a submit that shows the checked ones.
/// While some choice is unchecked the button says how many are kept
/// ("Source · 2 of 5") and takes the accent.
#[must_use]
pub fn filter_menu(locale: &Locale, menu: FilterMenu<'_>) -> Html {
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
            locale,
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
    let items: Html = choices
        .into_iter()
        .map(|choice| {
            ChoiceView {
                id: format!("{id}-{}", choice.value),
                name: name.to_owned(),
                value: choice.value,
                checked: choice.checked,
                mark: choice.mark,
                label: choice.label,
                count: grouped(locale, choice.count),
            }
            .html()
        })
        .collect();
    let note = note
        .map(|note| Html::raw(format!("<p class=\"fui-small fui-muted\">{note}</p>")))
        .unwrap_or_default();
    let body = Html::raw(format!(
        "<div class=\"fui-menu-items\">{items}</div><div class=\"fui-menu-sep\" role=\"separator\"></div><div class=\"fui-menu-foot\">{note}<button type=\"submit\" class=\"{}\">{}</button></div>",
        Variant::Secondary.small(),
        tr(locale, Text::FilterMenuShow)
    ));
    let (own, action, carried) = match form {
        MenuForm::Own { action, fields } => (
            true,
            action.map(str::to_owned),
            hidden_fields(fields, &[name, PAGE_KEY]),
        ),
        MenuForm::Enclosing => (false, None, Html::empty()),
    };
    MenuView {
        button: Variant::Secondary.class(),
        filtered,
        glyph: icon(glyph, "16"),
        words,
        chevron: icon(Glyph::CHEVRON_DOWN, "16"),
        own,
        action,
        class: "fui-menu fui-menu-compact fui-menu-anchored fui-menu-start",
        title,
        carried,
        body,
    }
    .html()
}
