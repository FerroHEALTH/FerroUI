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
//! [`crate::widget::tree`], drawn by the island
//! [`crate::island::TreeView`]. Every move is a link or a form, so a
//! browser walks with no WebAssembly. No specification governs the look:
//! our own design.

use leptos::prelude::*;

use crate::i18n::{Text, tr, tr_with};
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
    pub actions: Option<AnyView>,
    /// What it holds: [`pane_body`] for padded content, or a tree or a
    /// list that runs to its edges.
    pub body: AnyView,
}

impl BrowserPane {
    /// The pane `title` holding `body`.
    #[must_use]
    pub fn new(title: impl Into<String>, body: impl IntoView + 'static) -> Self {
        Self {
            title: title.into(),
            actions: None,
            body: body.into_any(),
        }
    }

    /// This pane, with `actions` at the end of its head.
    #[must_use]
    pub fn actions(self, actions: impl IntoView + 'static) -> Self {
        Self {
            actions: Some(actions.into_any()),
            ..self
        }
    }
}

/// Returns the browser of `panes`, laid out as `layout`, each pane's
/// heading at `heading`.
#[must_use]
pub fn browser(layout: Panes, heading: Heading, panes: Vec<BrowserPane>) -> AnyView {
    let panes = panes
        .into_iter()
        .map(|pane| {
            let BrowserPane {
                title,
                actions,
                body,
            } = pane;
            let head = match heading {
                Heading::Two => {
                    view! { <h2 class="fui-browser-title">{title.clone()}</h2> }.into_any()
                }
                Heading::Three => {
                    view! { <h3 class="fui-browser-title">{title.clone()}</h3> }.into_any()
                }
            };
            let actions = actions.map(|actions| view! { <div class="fui-actions">{actions}</div> });
            view! {
                <section class="fui-browser-pane" aria-label=title>
                    <div class="fui-browser-head">{head} {actions}</div>
                    {body}
                </section>
            }
        })
        .collect_view();
    view! {
        <div class=layout.class()>
            {panes}
        </div>
    }
    .into_any()
}

/// Returns `children` padded as a pane's body.
#[must_use]
pub fn pane_body(children: impl IntoView + 'static) -> AnyView {
    view! { <div class="fui-browser-body">{children}</div> }.into_any()
}

/// Returns the find field at the top of a pane.
///
/// It is one search field with the glyph inside, its label for assistive
/// technology alone, submitting on Enter as `find` says (a `GET` form, or a
/// `POST` form for words that can be clinical).
#[must_use]
pub fn browser_find(find: Find<'_>) -> AnyView {
    let carried = hidden_fields(find.fields, &[find.name, PAGE_KEY]);
    let id = find.id.to_owned();
    let label = find.label.to_owned();
    view! {
        <form
            role="search"
            method=method(find.source)
            action=find.action.map(str::to_owned)
            class="fui-browser-find"
        >
            {carried}
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
        </form>
    }
    .into_any()
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
pub fn match_list(label: impl Into<String>, matches: Vec<AnyView>) -> AnyView {
    let rows = matches
        .into_iter()
        .map(|each| view! { <li>{each}</li> })
        .collect_view();
    view! {
        <ul class="fui-match-list" aria-label=label.into()>
            {rows}
        </ul>
    }
    .into_any()
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
    pub units: Option<AnyView>,
    /// Its FLAT path, when the template has a web template.
    pub flat_path: Option<String>,
    /// A coded node's value set, one M2 coding each.
    pub codes: Vec<AnyView>,
    /// The more row after the codes, when the console's server pages a
    /// large value set; the heading then counts its total.
    pub more: Option<MoreCodes>,
    /// The query builder on this path.
    pub query: String,
}

/// Returns the inspector pane of the node `node`: its facts, a coded
/// node's value set, and Build a query on this path in its head.
#[must_use]
pub fn path_node(node: PathNode) -> BrowserPane {
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
        (tr(Text::PathNodePath), path(&at)),
        (
            tr(Text::PathNodeRmType),
            view! { <span class="fui-rm-type">{rm_type}</span> }.into_any(),
        ),
        (tr(Text::PathNodeOccurrences), occurrences.into_any()),
    ];
    if let Some(units) = units {
        rows.push((tr(Text::PathNodeUnits), units));
    }
    if let Some(flat) = flat_path {
        rows.push((tr(Text::PathNodeFlat), path(&flat)));
    }
    let facts = definitions(rows);
    let body = if codes.is_empty() {
        view! { <div class="fui-browser-body">{facts}</div> }.into_any()
    } else {
        let total = more
            .as_ref()
            .map_or_else(|| codes.len().into(), |more| more.total.into());
        let heading = tr_with(Text::PathNodeValueSet, &[("count", total)]);
        let codes = codes
            .into_iter()
            .map(|code| view! { <li>{code}</li> })
            .collect_view();
        let more = more.map(more_codes);
        view! {
            <div class="fui-browser-body fui-stack">
                {facts}
                <div>
                    <p class="fui-label">{heading}</p>
                    <ul class="fui-code-list">{codes} {more}</ul>
                </div>
            </div>
        }
        .into_any()
    };
    let action = view! {
        <a href=query class=Variant::Secondary.small()>
            {label(Some(Glyph::SQUARE_TERMINAL), tr(Text::PathNodeQuery))}
        </a>
    };
    BrowserPane::new(name, body).actions(action)
}
