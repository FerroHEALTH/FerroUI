// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The browser: panes of one panel, split by hairlines, stacked under
//! 64 rem (`design/COMPONENTS.md` M1 and E5).
//!
//! FerroTERM's concept browser puts the search, the hierarchy and the
//! concept side by side ([`Panes::Three`]), and over a flat code system
//! the search and the concept, which gets the room of two
//! ([`Panes::Flat`]). FerroEHR's path browser puts a template's paths
//! beside the node inspector ([`Panes::Two`], [`path_node`]). Each pane is
//! a labelled section with its head; the tree in a pane is
//! [`crate::view::tree`]. Every move is a link or a form, so a browser
//! walks with no script. No specification governs the look:
//! our own design.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::table::filter::Find;
use crate::table::model::PAGE_KEY;
use crate::table::view::{hidden_fields, method};
use crate::view::button::{Variant, label};
use crate::view::coding::{MoreCodes, more_codes};
use crate::view::form::INPUT;
use crate::view::path::path;
use crate::view::surface::definitions;

/// How many panes a browser lays out from 64 rem up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Panes {
    /// The search, the hierarchy and the concept (M1).
    #[default]
    Three,
    /// The search and the concept over a flat code system, the concept
    /// with the room of two (M1).
    Flat,
    /// A tree beside its inspector (E5).
    Two,
}

impl Panes {
    /// The classes of the browser.
    fn class(self) -> String {
        let modifier = match self {
            Self::Three => None,
            Self::Flat => Some("is-two-pane"),
            Self::Two => Some("is-two"),
        };
        std::iter::once("fui-browser")
            .chain(modifier)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// The level of a pane's heading, under the heading of the page or the
/// section the browser sits in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Heading {
    /// An `<h2>`, for a browser right under the page's title.
    #[default]
    Two,
    /// An `<h3>`, for a browser inside a section.
    Three,
}

/// One pane of a browser.
#[derive(Debug)]
pub struct BrowserPane {
    /// Its title, which names its section too.
    pub title: String,
    /// The actions at the end of its head.
    pub actions: Option<Html>,
    /// What it holds: [`pane_body`] for padded content, or a tree or a
    /// list that runs to its edges.
    pub body: Html,
}

impl BrowserPane {
    /// The pane `title` holding `body`.
    #[must_use]
    pub fn new(title: impl Into<String>, body: Html) -> Self {
        Self {
            title: title.into(),
            actions: None,
            body,
        }
    }

    /// This pane, with `actions` at the end of its head.
    #[must_use]
    pub fn actions(self, actions: Html) -> Self {
        Self {
            actions: Some(actions),
            ..self
        }
    }
}

/// Returns the browser of `panes`, laid out as `layout`, each pane's
/// heading at `heading`.
#[must_use]
pub fn browser(layout: Panes, heading: Heading, panes: Vec<BrowserPane>) -> Html {
    let level = match heading {
        Heading::Two => "h2",
        Heading::Three => "h3",
    };
    let panes: Html = panes
        .into_iter()
        .map(|pane| {
            let title = Html::text(&pane.title);
            let actions = pane.actions.map_or_else(String::new, |actions| {
                format!("<div class=\"fui-actions\">{actions}</div>")
            });
            Html::raw(format!(
                "<section aria-label=\"{title}\" class=\"fui-browser-pane\"><div class=\"fui-browser-head\"><{level} class=\"fui-browser-title\">{title}</{level}>{actions}</div>{body}</section>",
                body = pane.body,
            ))
        })
        .collect();
    Html::raw(format!("<div class=\"{}\">{panes}</div>", layout.class()))
}

/// Returns `children` padded as a pane's body.
#[must_use]
pub fn pane_body(children: Html) -> Html {
    Html::raw(format!(
        "<div class=\"fui-browser-body\">{}</div>",
        String::from(children)
    ))
}

#[derive(Template)]
#[template(
    source = r#"<form role="search" method="{{ method }}"{% if let Some(action) = action %} action="{{ action }}"{% endif %} class="fui-browser-find">{{ carried }}<div class="fui-list-find"><span class="fui-search-glyph">{{ glyph }}</span><label for="{{ id }}" class="fui-sr-only">{{ label }}</label><input id="{{ id }}" name="{{ name }}" type="search" class="{{ input }}" value="{{ value }}" placeholder="{{ label }}" autocomplete="off" spellcheck="false" enterkeyhint="search"></div></form>"#,
    ext = "html"
)]
struct FindView<'a> {
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

/// Returns the find field at the top of a pane.
///
/// It is one search field with the glyph inside, its label for assistive
/// technology alone, submitting on Enter as `find` says (a `GET` form, or a
/// `POST` form for words that can be clinical).
#[must_use]
pub fn browser_find(find: Find<'_>) -> Html {
    FindView {
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

/// Returns the matches of a search, named `label`: one row each, the row
/// the target, the one in view in the accent's tint.
///
/// Each match is what its row shows, such as an M2 coding with a row link.
/// The match the browser reads marks its link `aria-current="true"`
/// (WAI-ARIA 1.2), as [`crate::view::coding::Coding::current`] does, and
/// its row takes the tint from that mark, so assistive technology hears
/// "current" on the link a keyboard reaches.
#[must_use]
pub fn match_list(label: &str, matches: Vec<Html>) -> Html {
    let rows: Html = matches
        .into_iter()
        .map(|each| Html::raw(format!("<li>{each}</li>")))
        .collect();
    Html::raw(format!(
        "<ul aria-label=\"{}\" class=\"fui-match-list\">{rows}</ul>",
        Html::text(label)
    ))
}

/// One node of a template, as the path browser's inspector reads it (E5).
#[derive(Debug)]
pub struct PathNode {
    /// The node's name, the inspector's title.
    pub name: String,
    /// Its archetype path.
    pub path: String,
    /// Its RM type, such as `DV_QUANTITY`.
    pub rm_type: String,
    /// Its occurrences, such as `0..1`.
    pub occurrences: String,
    /// Its units, as a person reads them, with the UCUM code in the
    /// tooltip.
    pub units: Option<Html>,
    /// Its FLAT path, when the template has a web template.
    pub flat_path: Option<String>,
    /// A coded node's value set, one M2 coding each.
    pub codes: Vec<Html>,
    /// The more row after the codes, when the console's server pages a
    /// large value set; the heading then counts its total.
    pub more: Option<MoreCodes>,
    /// Where Build a query on this path leads, the query builder on this
    /// path; `None` where no query can be built on the template (a
    /// demographic template, such as a PERSON's), which draws no button.
    pub query: Option<String>,
}

/// Returns the inspector pane of the node `node`, in `locale`: its facts, a
/// coded node's value set, and Build a query on this path in its head
/// where the node has a query.
#[must_use]
pub fn path_node(locale: &Locale, node: PathNode) -> BrowserPane {
    let PathNode {
        name,
        path: at,
        rm_type,
        occurrences,
        units,
        flat_path,
        codes,
        more,
        query,
    } = node;
    let mut rows = vec![
        (tr(locale, Text::PathNodePath), path(&at)),
        (
            tr(locale, Text::PathNodeRmType),
            Html::raw(format!(
                "<span class=\"fui-rm-type\">{}</span>",
                Html::text(&rm_type)
            )),
        ),
        (
            tr(locale, Text::PathNodeOccurrences),
            Html::text(&occurrences),
        ),
    ];
    if let Some(units) = units {
        rows.push((tr(locale, Text::PathNodeUnits), units));
    }
    if let Some(flat) = flat_path {
        rows.push((tr(locale, Text::PathNodeFlat), path(&flat)));
    }
    let facts = definitions(rows);
    let body = if codes.is_empty() {
        Html::raw(format!("<div class=\"fui-browser-body\">{facts}</div>"))
    } else {
        let total = more
            .as_ref()
            .map_or_else(|| codes.len().into(), |more| more.total.into());
        let heading = tr_with(locale, Text::PathNodeValueSet, &[("count", total)]);
        let codes: Html = codes
            .into_iter()
            .map(|code| Html::raw(format!("<li>{code}</li>")))
            .collect();
        let more = more.map(more_codes).unwrap_or_default();
        Html::raw(format!(
            "<div class=\"fui-browser-body fui-stack\">{facts}<div><p class=\"fui-label\">{heading}</p><ul class=\"fui-code-list\">{codes}{more}</ul></div></div>"
        ))
    };
    let pane = BrowserPane::new(name, body);
    match query {
        Some(query) => pane.actions(Html::raw(format!(
            "<a href=\"{}\" class=\"{}\">{}</a>",
            Html::text(&query),
            Variant::Secondary.small(),
            label(
                Some(Glyph::SQUARE_TERMINAL),
                tr(locale, Text::PathNodeQuery)
            )
        ))),
        None => pane,
    }
}
