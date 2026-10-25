// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The global search of a top bar, one joined control (#144), the answer
//! panel it opens under the field as you type, and the find page's results.
//!
//! The control is a bordered group with the control radius: the search
//! glyph inside the field, the field, a quiet `/` key cap, and the submit
//! joined on the field's end edge, so field and button read as one control;
//! hover and focus outline the whole group. It is a plain `GET` form with
//! `role="search"` (WAI-ARIA 1.2, the `search` landmark), so it works with no
//! WebAssembly: Enter submits it to the console's find page, which draws the
//! full results in the page with [`find_results`]. A search that can carry
//! clinical content posts it instead ([`SearchBox::post`]), so the query
//! never reaches a URL. The field's label is visually hidden, so its
//! accessible name is the label and never the placeholder.
//!
//! The control is the island [`crate::island::Search`], which owns its keys
//! and its typeahead ([`crate::widget::search`]). Once the page has
//! hydrated and the console names an answer route ([`SearchBox::suggest`]),
//! typing asks that route for a [`SearchAnswer`] and opens it in a panel
//! anchored under the field: the WAI-ARIA Authoring Practices 1.2 combobox
//! with a listbox popup, its hits grouped by kind, at most [`PER_GROUP`] of
//! each, and an option that submits to the find page for every result. The
//! route's wire is in `docs/consoles.md` of the FerroUI repository.
//!
//! The `/` cap is hidden while the field holds text or has the focus, while
//! the control is disabled, below 40 rem and while the control is too narrow
//! for it (family rule 6); the shortcut is announced through
//! `aria-keyshortcuts`. No specification governs the panel's shape: our own
//! design.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Phrase, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::notice::{Announce, notice};
use crate::view::template::list::{HeadingCount, ListCard, list_card};

/// The most hits of one kind the panel under the field shows; the find
/// page shows them all.
pub const PER_GROUP: usize = 5;

/// What a search box holds.
///
/// Its words are resolved on the server in the page's locale: they cross to
/// the browser as the island's props.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchBox {
    /// The field's id; the panel's ids start with it.
    pub id: String,
    /// The query key the field submits as, such as `q`.
    pub name: String,
    /// The field's label, read by a screen reader.
    pub label: String,
    /// The hint inside the empty field, such as "A code, a canonical, or a
    /// phrase".
    pub placeholder: String,
    /// The console's find page the form submits to.
    pub action: String,
    /// The text already searched for, on the find page.
    pub value: String,
    /// The submit's words, which below 40 rem only a screen reader reads.
    pub submit: Word,
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
    /// patient's name, which never goes into a URL (`leptos-ui.md` §5). The
    /// typeahead then posts its query too.
    pub post: bool,
    /// The console's route that answers a query with a [`SearchAnswer`] as
    /// JSON, which the panel under the field asks as you type; `None` keeps
    /// the box a plain form.
    pub suggest: Option<String>,
    /// How many characters a query holds before the typeahead asks.
    pub suggest_from: u8,
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
            suggest: None,
            suggest_from: 2,
        }
    }
}

/// The words the panel under the field draws itself, resolved on the
/// server in the page's locale, so the island carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchWords {
    /// The accessible name of the panel's list of hits.
    pub answer: Word,
    /// The row while the console is asked.
    pub searching: Word,
    /// The row when nothing matches, around the open `query`.
    pub nothing: Phrase,
    /// The option that submits to the find page, around the open `query`.
    pub all: Phrase,
    /// What a screen reader hears once hits arrive, around the open `query`
    /// and `count`.
    pub counted: Phrase,
    /// The row when the console answered with an error, around the open
    /// `status`.
    pub failed: Phrase,
    /// The row when the console could not be reached.
    pub unreachable: Word,
    /// The row when the console's answer could not be read.
    pub unreadable: Word,
}

impl SearchWords {
    /// The words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        let query = || ("query", Phrase::open("query"));
        Self {
            answer: tr(Text::SearchAnswer),
            searching: tr(Text::SearchSearching),
            nothing: Phrase::new(tr_with(Text::SearchNothing, &[query()])),
            all: Phrase::new(tr_with(Text::SearchAll, &[query()])),
            counted: Phrase::new(tr_with(
                Text::SearchCounted,
                &[query(), ("count", Phrase::open("count"))],
            )),
            failed: Phrase::new(tr_with(
                Text::SearchFailed,
                &[("status", Phrase::open("status"))],
            )),
            unreachable: tr(Text::SearchUnreachable),
            unreadable: tr(Text::SearchUnreadable),
        }
    }
}

/// The mark a hit starts with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mark {
    /// The dot of a state in `tone`, whose word the hit's line says
    /// (family rule 12).
    Dot(Tone),
    /// A glyph of the hit's kind, beside its name.
    Glyph(Glyph),
}

/// One hit of an answer: a link to the thing found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HitGroup {
    /// The kind, such as "Organisations".
    pub kind: String,
    /// The hits, in the order the console ranked them: every one, which the
    /// panel cuts to [`PER_GROUP`] and the find page shows whole.
    pub hits: Vec<Hit>,
}

/// A link with its words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Way {
    /// Where it leads.
    pub href: String,
    /// Its words, such as "Browse every file".
    pub words: String,
}

/// What a console found for a query: the panel under the field draws it as
/// you type, the console's answer route sends it as JSON, and the find page
/// draws it whole with [`find_results`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchAnswer {
    /// The words answered, which every hit's name and line emphasise.
    pub query: String,
    /// The line at the top, such as "2 patients · name and birth year".
    pub head: Option<String>,
    /// The hits, grouped by kind, each kind under its heading.
    pub groups: Vec<HitGroup>,
    /// What the answer says when nothing was found: a sentence, and the
    /// line under it; the library's "No matches" when `None`.
    pub none: Option<(String, String)>,
    /// Why the answer may not be whole, such as "Templates did not answer";
    /// drawn as a warning over the hits, never left out.
    pub unfinished: Option<String>,
    /// A way past the answer, such as a listing that may hold more.
    pub more: Option<Way>,
}

impl SearchAnswer {
    /// Whether the answer holds a hit.
    #[must_use]
    pub fn found(&self) -> bool {
        self.groups.iter().any(|group| !group.hits.is_empty())
    }

    /// How many hits the answer holds.
    #[must_use]
    pub fn total(&self) -> usize {
        self.groups.iter().map(|group| group.hits.len()).sum()
    }
}

/// The id of the panel's list of hits under the search box `id`.
#[must_use]
pub fn answer_id(id: &str) -> String {
    format!("{id}-results")
}

/// The id of the `place`th hit, from 1, of the panel under the search box
/// `id`.
#[must_use]
pub fn hit_id(id: &str, place: u32) -> String {
    format!("{id}-hit-{place}")
}

/// Returns the search box `search`, its typeahead panel under it once the
/// page hydrates.
#[must_use]
pub fn search_box(search: SearchBox) -> AnyView {
    let words = SearchWords::of_page();
    view! { <crate::island::Search search words /> }.into_any()
}

/// The runs of `text`, each marked when it is a match of `query` without
/// regard to case; the runs joined are `text`.
#[must_use]
pub fn runs<'a>(text: &'a str, query: &str) -> Vec<(&'a str, bool)> {
    let needle: Vec<char> = query.trim().chars().flat_map(char::to_lowercase).collect();
    if needle.is_empty() {
        return vec![(text, false)];
    }
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut found = Vec::new();
    let mut plain = 0;
    let mut at = 0;
    while let Some(&(start, _)) = chars.get(at) {
        let Some(end_at) = match_at(&chars, at, &needle) else {
            at += 1;
            continue;
        };
        let end = chars.get(end_at).map_or(text.len(), |&(byte, _)| byte);
        found.extend(text.get(plain..start).map(|run| (run, false)));
        found.extend(text.get(start..end).map(|run| (run, true)));
        plain = end;
        at = end_at;
    }
    found.extend(text.get(plain..).map(|run| (run, false)));
    found.retain(|(run, _)| !run.is_empty());
    found
}

/// The index after a match of `needle` that starts at the character `at`
/// of `chars`, if one starts there.
fn match_at(chars: &[(usize, char)], at: usize, needle: &[char]) -> Option<usize> {
    let mut matched = 0;
    let mut next = at;
    while matched < needle.len() {
        let &(_, c) = chars.get(next)?;
        for lower in c.to_lowercase() {
            if needle.get(matched) != Some(&lower) {
                return None;
            }
            matched += 1;
        }
        next += 1;
    }
    Some(next)
}

/// Returns `text` with every match of `query` in a `mark` element, which
/// the HTML Living Standard names for the matches of a search.
#[must_use]
pub fn emphasised(text: &str, query: &str) -> AnyView {
    runs(text, query)
        .into_iter()
        .map(|(run, marked)| {
            let run = run.to_owned();
            if marked {
                view! { <mark class="fui-search-match">{run}</mark> }.into_any()
            } else {
                run.into_any()
            }
        })
        .collect_view()
        .into_any()
}

/// Returns what one hit shows: its mark, its name over its line with the
/// matches of `query` emphasised, and its tag; the panel and the find page
/// both draw a hit this way.
#[must_use]
pub fn hit_body(hit: &Hit, query: &str) -> AnyView {
    let mark = hit.mark.map(|mark| match mark {
        Mark::Dot(tone) => {
            let class = format!("fui-dot {}", tone.class());
            view! { <span class=class aria-hidden="true"></span> }.into_any()
        }
        Mark::Glyph(glyph) => icon(glyph, "16").into_any(),
    });
    let tag = hit
        .tag
        .clone()
        .map(|tag| view! { <span class="fui-tag">{tag}</span> });
    view! {
        {mark}
        <span class="fui-hit-name">{emphasised(&hit.name, query)}</span>
        <span class="fui-small fui-muted fui-hit-line">{emphasised(&hit.line, query)}</span>
        {tag}
    }
    .into_any()
}

/// Returns `answer` whole as the find page's results, in the page: one
/// list card per kind with every hit as a link, the warning when the answer
/// may not be whole, and the empty state when nothing matched.
///
/// It is the find page's body under its heading, and what a search shows
/// with no WebAssembly; it is never an overlay.
#[must_use]
pub fn find_results(answer: SearchAnswer) -> AnyView {
    let SearchAnswer {
        query,
        head,
        groups,
        none,
        unfinished,
        more,
    } = answer;
    let unfinished = unfinished.map(|why| notice(Tone::Warn, Announce::Status, None, why));
    let head = head.map(|head| view! { <p class="fui-find-head fui-muted">{head}</p> });
    let found = groups.iter().any(|group| !group.hits.is_empty());
    let cards = groups
        .into_iter()
        .filter(|group| !group.hits.is_empty())
        .map(|group| {
            let count = u64::try_from(group.hits.len()).unwrap_or(u64::MAX);
            let hits = group
                .hits
                .iter()
                .map(|hit| {
                    view! {
                        <li>
                            <a class="fui-hit" href=hit.href.clone()>
                                {hit_body(hit, &query)}
                            </a>
                        </li>
                    }
                })
                .collect_view();
            list_card(ListCard {
                title: group.kind,
                count: Some(HeadingCount::All(count)),
                order: None,
                tabs: None,
                actions: None,
                body: view! { <ul class="fui-find-list">{hits}</ul> }.into_any(),
                pager: None,
            })
        })
        .collect_view();
    let nothing = (!found).then(|| {
        let (said, line) = none.unwrap_or_else(|| {
            (
                tr_with(Text::SearchNothing, &[("query", query.as_str().into())])
                    .text()
                    .to_owned(),
                String::new(),
            )
        });
        let line = (!line.is_empty()).then(|| view! { <span>{line}</span> });
        view! {
            <div class="fui-card fui-find-empty">
                {icon(Glyph::SEARCH_X, "20")}
                <div>
                    <strong>{said}</strong>
                    {line}
                </div>
            </div>
        }
    });
    let more = more.map(|way| {
        view! {
            <a href=way.href class="fui-find-more">
                {way.words}
                {icon(Glyph::ARROW_RIGHT, "14")}
            </a>
        }
    });
    view! {
        <div class="fui-find-results">
            {head}
            {unfinished}
            {cards}
            {nothing}
            {more}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::runs;

    #[test]
    fn a_match_is_marked_without_regard_to_case() {
        assert_eq!(
            vec![("Vital ", false), ("Signs", true), (" v1", false)],
            runs("Vital Signs v1", "signs")
        );
        assert_eq!(vec![("abc", true)], runs("abc", "ABC"));
    }

    #[test]
    fn every_match_is_marked_and_the_runs_join_to_the_text() {
        let text = "Größe größer GRÖSSE";
        let marked = runs(text, "grö");
        assert_eq!(
            text,
            marked.iter().map(|(run, _)| *run).collect::<String>(),
            "the runs join to the text"
        );
        assert_eq!(
            vec!["Grö", "grö", "GRÖ"],
            marked
                .iter()
                .filter(|(_, hit)| *hit)
                .map(|(run, _)| *run)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn an_empty_query_marks_nothing() {
        assert_eq!(vec![("abc", false)], runs("abc", "  "));
        assert_eq!(vec![("abc", false)], runs("abc", "abcd"));
        assert_eq!(Vec::<(&str, bool)>::new(), runs("", "a"));
    }
}
