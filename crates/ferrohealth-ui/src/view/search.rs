// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The global search of a top bar, one joined control (#144), and the
//! answer it opens under it.
//!
//! The control is a bordered group with the control radius: the search
//! glyph inside the field, the field, a quiet `/` key cap, and the submit
//! joined on the field's end edge, so field and button read as one control;
//! hover and focus outline the whole group. It is a plain `GET` form with
//! `role="search"` (WAI-ARIA 1.2, the `search` landmark), so it works with no
//! WebAssembly: Enter submits it to the console's route, which reads the
//! query. A search that can carry clinical content posts it instead
//! ([`SearchBox::post`]), so the query never reaches a URL. The field's label is visually hidden, so its accessible name is the
//! label and never the placeholder.
//!
//! The control is the island [`crate::island::Search`], which owns its keys
//! ([`crate::widget::search`]): `/` from anywhere outside an editable element
//! focuses the field, Escape clears and leaves it, Enter on an open answer
//! follows its first hit, and a submit turns the glyph into the spinner while
//! the next page loads. The `/` cap is hidden while the field holds text or
//! has the focus, while the control is disabled, and below 40 rem (family
//! rule 6); the shortcut is announced through `aria-keyshortcuts`.
//!
//! The answer is drawn by the server, under the control, when the console
//! has one for the query (the search result row, one overlay surface): the
//! hits grouped by kind, the first one selected for Enter, and the way to the
//! full results. No specification governs the shape: our own design.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;

/// What a search box holds.
///
/// Its words are resolved on the server in the page's locale: they cross to
/// the browser as the island's props.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchBox {
    /// The field's id; the answer's ids start with it.
    pub id: String,
    /// The query key the field submits as, such as `q`.
    pub name: String,
    /// The field's label, read by a screen reader.
    pub label: String,
    /// The hint inside the empty field, such as "A code, a canonical, or a
    /// phrase".
    pub placeholder: String,
    /// The console's route the form submits to.
    pub action: String,
    /// The text already searched for, on a result page.
    pub value: String,
    /// The submit's words, which below 40 rem only a screen reader reads.
    pub submit: String,
    /// The key that focuses the field from anywhere on the page, drawn as
    /// a quiet key cap in the field; `None` draws no cap and binds no key.
    pub shortcut: Option<char>,
    /// Whether a search is running: the glyph is the spinner and the
    /// submit waits.
    pub busy: bool,
    /// Why the search cannot be used now, such as "Search waits while the
    /// index is rebuilding": the field and the submit are disabled and the
    /// field shows the reason in place of its hint.
    pub disabled: Option<String>,
    /// Whether the form posts its query instead of putting it in the
    /// address: for a search that can carry clinical content, such as a
    /// patient's name, which never goes into a URL (`leptos-ui.md` §5).
    pub post: bool,
}

impl Default for SearchBox {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::from("q"),
            label: String::new(),
            placeholder: String::new(),
            action: String::new(),
            value: String::new(),
            submit: tr(Text::Find),
            shortcut: Some('/'),
            busy: false,
            disabled: None,
            post: false,
        }
    }
}

/// The mark a hit starts with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// The dot of a state in `tone`, whose word the hit's line says
    /// (family rule 12).
    Dot(Tone),
    /// A glyph of the hit's kind, beside its name.
    Glyph(Glyph),
}

/// One hit of an answer: a link to the thing found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// Where the hit leads.
    pub href: String,
    /// The thing's name.
    pub name: String,
    /// The line under the name, such as "Born 1985-03-14, female".
    pub line: String,
    /// The mark before the name, if the hit has one.
    pub mark: Option<Mark>,
    /// A short tag at the end, such as an identifier, if the hit has one.
    pub tag: Option<String>,
}

/// The hits of one kind, under the kind's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitGroup {
    /// The kind, such as "Organisations".
    pub kind: String,
    /// The hits, in the order the console ranked them.
    pub hits: Vec<Hit>,
}

/// A link with its words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Way {
    /// Where it leads.
    pub href: String,
    /// Its words, such as "All results in Search".
    pub words: String,
}

/// The answer a search box opens under it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchAnswer {
    /// The line at the top, such as "2 patients · name and birth year".
    pub head: Option<String>,
    /// The hits, grouped by kind; a group's kind is drawn when there is
    /// more than one group.
    pub groups: Vec<HitGroup>,
    /// What the answer says when nothing was found: a sentence, and the
    /// line under it.
    pub none: Option<(String, String)>,
    /// The way to the full results, or to a search that may find more.
    pub more: Option<Way>,
}

impl SearchAnswer {
    /// Whether the answer holds a hit.
    #[must_use]
    pub fn found(&self) -> bool {
        self.groups.iter().any(|group| !group.hits.is_empty())
    }
}

/// The id of the answer under the search box `id`.
#[must_use]
pub fn answer_id(id: &str) -> String {
    format!("{id}-results")
}

/// The id of the `place`th hit, from 1, of the answer under the search box
/// `id`; the first one is the hit Enter follows.
#[must_use]
pub fn hit_id(id: &str, place: u32) -> String {
    format!("{id}-hit-{place}")
}

/// Returns the search box `search`, and under it `answer` when the console
/// has one for the query.
#[must_use]
pub fn search_box(search: SearchBox, answer: Option<SearchAnswer>) -> AnyView {
    let selected = answer
        .as_ref()
        .filter(|answer| answer.found())
        .map(|_| hit_id(&search.id, 1));
    let open = answer.is_some();
    let answer = answer.map(|answer| search_answer(&search.id, answer));
    view! {
        <crate::island::Search search selected open>
            {answer}
        </crate::island::Search>
    }
    .into_any()
}

/// Returns `answer` as the panel under the search box `id`.
#[must_use]
pub fn search_answer(id: &str, answer: SearchAnswer) -> AnyView {
    let SearchAnswer {
        head,
        groups,
        none,
        more,
    } = answer;
    let named = groups.len() > 1;
    let mut place = 0_u32;
    let groups = groups
        .into_iter()
        .zip(1_u32..)
        .map(|(group, index)| {
            let kind_id = format!("{id}-kind-{index}");
            let hits = group
                .hits
                .into_iter()
                .map(|hit| {
                    place = place.saturating_add(1);
                    hit_view(hit_id(id, place), place == 1, hit)
                })
                .collect_view();
            let (labelled_by, label) = if named {
                (Some(kind_id.clone()), None)
            } else {
                (None, Some(group.kind.clone()))
            };
            let heading = named.then(|| {
                view! {
                    <p id=kind_id class="fui-search-results-group">
                        {group.kind}
                    </p>
                }
            });
            view! {
                {heading}
                <ul aria-labelledby=labelled_by aria-label=label>
                    {hits}
                </ul>
            }
        })
        .collect_view();
    let none = none.map(|(said, line)| {
        view! {
            <div class="fui-search-results-empty">
                {icon(Glyph::SEARCH_X, "16")}
                <div>
                    <strong>{said}</strong>
                    <span>{line}</span>
                </div>
            </div>
        }
    });
    let more = more.map(|way| {
        view! {
            <a href=way.href class="fui-search-results-more">
                {way.words}
                {icon(Glyph::ARROW_RIGHT, "14")}
            </a>
        }
    });
    view! {
        <div id=answer_id(id) class="fui-search-results" role="region" aria-label=tr(Text::SearchAnswer)>
            {head.map(|head| view! { <p class="fui-search-results-head">{head}</p> })}
            {groups}
            {none}
            {more}
        </div>
    }
    .into_any()
}

/// Returns `hit` as the link `id`, marked current when it is the `first`.
fn hit_view(id: String, first: bool, hit: Hit) -> impl IntoView {
    let Hit {
        href,
        name,
        line,
        mark,
        tag,
    } = hit;
    let mark = mark.map(|mark| match mark {
        Mark::Dot(tone) => {
            let class = format!("fui-dot {}", tone.class());
            view! { <span class=class aria-hidden="true"></span> }.into_any()
        }
        Mark::Glyph(glyph) => icon(glyph, "16").into_any(),
    });
    view! {
        <li>
            <a href=href id=id aria-current=first.then_some("true")>
                {mark}
                <span class="fui-search-results-name">{name}</span>
                <span class="fui-small fui-muted">{line}</span>
                {tag.map(|tag| view! { <span class="fui-tag">{tag}</span> })}
            </a>
        </li>
    }
}
