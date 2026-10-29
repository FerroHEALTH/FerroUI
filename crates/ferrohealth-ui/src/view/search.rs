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
//! script: Enter submits it to the console's find page, which draws the
//! full results in the page with [`find_results`]. A search that can carry
//! clinical content posts it instead ([`SearchBox::post`]), so the query
//! never reaches a URL. The field's label is visually hidden, so its
//! accessible name is the label and never the placeholder.
//!
//! The `search` module owns the keys and the typeahead. Its key (`/` by
//! default, [`is_shortcut`]) from anywhere outside an element that takes
//! text focuses the field with the caret after its text. Where the console
//! names an answer route ([`SearchBox::suggest`]), the field becomes the
//! WAI-ARIA Authoring Practices 1.2 combobox with a listbox popup: typing
//! asks that route for a [`SearchAnswer`] once the words rest for
//! [`SETTLE`] ([`wants`]) and opens it in a panel anchored under the field
//! ([`placed`]), its hits grouped by kind, at most [`PER_GROUP`] of each,
//! and an option that submits to the find page for every result
//! ([`targets`]). The arrows move the active option with the focus kept in
//! the field ([`step`]), Enter follows it, Tab and a press outside close
//! the panel, and Escape closes it or clears the words ([`escaped`]). A
//! submit turns the glyph into the spinner while the next page loads. The
//! route's wire is in `docs/consoles.md` of the FerroUI repository.
//!
//! The `/` cap is hidden while the field holds text or has the focus, while
//! the control is disabled, below 40 rem and while the control is too narrow
//! for it (family rule 6); the shortcut is announced through
//! `aria-keyshortcuts`. No specification governs the panel's shape, the
//! timing or the shortcut: our own design.

use std::time::Duration;

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::notice::{Announce, notice};
use crate::view::template::list::{HeadingCount, ListCard, list_card};

/// The most hits of one kind the panel under the field shows; the find
/// page shows them all.
pub const PER_GROUP: usize = 5;

/// What a search box holds.
///
/// Its words are resolved on the server in the page's locale.
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
    /// The submit's words, which below 40 rem only a screen reader reads;
    /// `None` reads "Find" in the page's locale.
    pub submit: Option<Word>,
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
    /// patient's name, which never goes into a URL. The typeahead then
    /// posts its query too.
    pub post: bool,
    /// The console's route that answers a query with a [`SearchAnswer`] as
    /// JSON, which the panel under the field asks as you type; `None` keeps
    /// the box a plain form.
    pub suggest: Option<String>,
    /// How many characters a query holds before the typeahead asks.
    pub suggest_from: u8,
    /// The words of the panel's option that submits the search to the find
    /// page, around the open `query`, such as "Search patients for
    /// “{query}”" where the submit runs another search than the panel's;
    /// `None` keeps "See all results for “{query}”".
    pub all: Option<Phrase>,
    /// The address the page loads when the answer route answers
    /// `401 Unauthorized`: the session is over, so the console's own
    /// signed-out page takes over, as any other request of the page would
    /// (RFC 9110 §15.5.2). `None` shows the panel's error row. Either way
    /// every other tab hears that the session has ended (the `session`
    /// module).
    pub signed_out: Option<String>,
    /// The glyphs the answer route's hits may carry ([`Mark::Glyph`]), which
    /// the page holds for the `search` module to draw; a hit whose glyph is
    /// not among them shows its name with no glyph.
    pub glyphs: Vec<Glyph>,
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
            submit: None,
            shortcut: Some('/'),
            busy: false,
            disabled: None,
            post: false,
            suggest: None,
            suggest_from: 2,
            all: None,
            signed_out: None,
            glyphs: Vec::new(),
        }
    }
}

/// The words the panel under the field draws itself, resolved on the
/// server in the page's locale, so the `search` module carries no catalog.
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
    /// The words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        let query = || ("query", Phrase::open("query"));
        Self {
            answer: tr(locale, Text::SearchAnswer),
            searching: tr(locale, Text::SearchSearching),
            nothing: Phrase::new(tr_with(locale, Text::SearchNothing, &[query()])),
            all: Phrase::new(tr_with(locale, Text::SearchAll, &[query()])),
            counted: Phrase::new(tr_with(
                locale,
                Text::SearchCounted,
                &[query(), ("count", Phrase::open("count"))],
            )),
            failed: Phrase::new(tr_with(
                locale,
                Text::SearchFailed,
                &[("status", Phrase::open("status"))],
            )),
            unreachable: tr(locale, Text::SearchUnreachable),
            unreadable: tr(locale, Text::SearchUnreadable),
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

/// How long the words rest before the typeahead asks.
pub const SETTLE: Duration = Duration::from_millis(200);

/// What Escape in the search field does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Escape {
    /// Closes the open panel and keeps the words.
    Close,
    /// Clears the words of a field whose panel is closed.
    Clear,
    /// Leaves the key to the page: the field is empty and nothing is open.
    Pass,
}

/// What Escape does in a field whose panel is `open` and whose words are
/// `empty` or not; the focus stays in the field in every case.
#[must_use]
pub fn escaped(open: bool, empty: bool) -> Escape {
    if open {
        Escape::Close
    } else if empty {
        Escape::Pass
    } else {
        Escape::Clear
    }
}

/// Whether the key `key` of an event is the shortcut `shortcut`.
#[must_use]
pub fn is_shortcut(key: &str, shortcut: char) -> bool {
    let mut chars = key.chars();
    chars.next() == Some(shortcut) && chars.next().is_none()
}

/// Whether the words `words` are long enough, at `from` characters, for
/// the typeahead to ask.
#[must_use]
pub fn wants(words: &str, from: u8) -> bool {
    let held = words.trim().chars().count();
    held > 0 && held >= usize::from(from)
}

/// Where an option of the panel leads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A hit, to its page.
    Hit(String),
    /// The find page with every result, by the form's own submit.
    All,
    /// The answer's way past it.
    Way(String),
}

/// The options of the panel for `answer`, in their order: at most
/// [`PER_GROUP`] hits of each kind, then every result, then the way past.
#[must_use]
pub fn targets(answer: &SearchAnswer) -> Vec<Target> {
    let mut found: Vec<Target> = answer
        .groups
        .iter()
        .flat_map(|group| group.hits.iter().take(PER_GROUP))
        .map(|hit| Target::Hit(hit.href.clone()))
        .collect();
    if answer.found() {
        found.push(Target::All);
    }
    if let Some(way) = &answer.more {
        found.push(Target::Way(way.href.clone()));
    }
    found
}

/// The option after `active` among `count`, or before it when not `down`,
/// wrapping past either end; the first or the last when none is active.
#[must_use]
pub fn step(active: Option<usize>, count: usize, down: bool) -> Option<usize> {
    let last = count.checked_sub(1)?;
    Some(match (active, down) {
        (None | Some(0), false) => last,
        (None, true) => 0,
        (Some(at), true) if at >= last => 0,
        (Some(at), true) => at + 1,
        (Some(at), false) => at.min(count).saturating_sub(1),
    })
}

/// Which edge of the field the panel opens from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// From the field's start edge, the default.
    Start,
    /// From the field's end edge, when the start edge leaves no room.
    End,
    /// As wide as the field, when neither edge leaves room.
    Narrow,
}

/// The side a panel `wide` pixels opens from under a field whose edges are
/// at `left` and `right` in a window `view` pixels wide, each edge keeping
/// the 16 px gutter.
#[must_use]
pub fn placed(left: f64, right: f64, wide: f64, view: f64) -> Side {
    // NOTE: no specification governs this: our own design; the 16 px gutter
    // is the page's own.
    if left + wide <= view - 16.0 {
        Side::Start
    } else if right - wide >= 16.0 {
        Side::End
    } else {
        Side::Narrow
    }
}

#[derive(Template)]
#[template(path = "search/search.html")]
struct BoxView<'a> {
    id: &'a str,
    name: &'a str,
    label: &'a str,
    placeholder: &'a str,
    action: &'a str,
    value: &'a str,
    submit: Word,
    post: bool,
    busy: bool,
    disabled: Option<&'a str>,
    shortcut: Option<String>,
    suggest: Option<&'a str>,
    from: u8,
    signed_out: Option<&'a str>,
    words: String,
    answer: Word,
    glyph: Html,
    icons: Vec<(&'static str, Html)>,
}

/// Returns the search box `search` in `locale`, its typeahead panel under
/// it held closed for the `search` module.
#[must_use]
pub fn search_box(locale: &Locale, search: &SearchBox) -> Html {
    let mut words = SearchWords::of(locale);
    if let Some(all) = search.all.clone() {
        words.all = all;
    }
    let off = search.disabled.is_some();
    let mut icons = vec![
        ("search", icon(Glyph::SEARCH, "16")),
        ("circle-alert", icon(Glyph::CIRCLE_ALERT, "16")),
        ("triangle-alert", icon(Glyph::TRIANGLE_ALERT, "16")),
        ("search-x", icon(Glyph::SEARCH_X, "16")),
        ("arrow-right", icon(Glyph::ARROW_RIGHT, "14")),
    ];
    if search.suggest.is_some() {
        icons.extend(
            search
                .glyphs
                .iter()
                .filter_map(|glyph| Some((glyph.name()?, icon(*glyph, "16")))),
        );
    } else {
        icons.truncate(1);
    }
    BoxView {
        id: &search.id,
        name: &search.name,
        label: &search.label,
        placeholder: search.disabled.as_deref().unwrap_or(&search.placeholder),
        action: &search.action,
        value: &search.value,
        submit: search
            .submit
            .clone()
            .unwrap_or_else(|| tr(locale, Text::Find)),
        post: search.post,
        busy: search.busy,
        disabled: search.disabled.as_deref(),
        shortcut: search.shortcut.filter(|_| !off).map(|key| key.to_string()),
        suggest: search.suggest.as_deref().filter(|_| !off),
        from: search.suggest_from,
        signed_out: search.signed_out.as_deref(),
        // NOTE: serde_json `to_string` fails only for a map with a key that is no
        // string, and the words hold none.
        words: serde_json::to_string(&words).unwrap_or_default(),
        answer: words.answer.clone(),
        glyph: icon(Glyph::SEARCH, "16"),
        icons,
    }
    .html()
}

/// Returns `text` with every match of `query` in a `mark` element, which
/// the HTML Living Standard names for the matches of a search.
#[must_use]
pub fn emphasised(text: &str, query: &str) -> Html {
    runs(text, query)
        .into_iter()
        .map(|(run, marked)| {
            if marked {
                Html::raw(format!(
                    "<mark class=\"fui-search-match\">{}</mark>",
                    Html::text(run)
                ))
            } else {
                Html::text(run)
            }
        })
        .collect()
}

/// Returns what one hit shows: its mark, its name over its line with the
/// matches of `query` emphasised, and its tag.
///
/// The find page draws a hit this way, and the `search` module draws a hit
/// of the panel alike.
#[must_use]
pub fn hit_body(hit: &Hit, query: &str) -> Html {
    let mark = match hit.mark {
        Some(Mark::Dot(tone)) => format!(
            "<span aria-hidden=\"true\" class=\"fui-dot {}\"></span>",
            tone.class()
        ),
        Some(Mark::Glyph(glyph)) => icon(glyph, "16").into(),
        None => String::new(),
    };
    let tag = hit.tag.as_deref().map_or_else(String::new, |tag| {
        format!("<span class=\"fui-tag\">{}</span>", Html::text(tag))
    });
    Html::raw(format!(
        "{mark}<span class=\"fui-hit-name\">{}</span><span class=\"fui-small fui-muted fui-hit-line\">{}</span>{tag}",
        emphasised(&hit.name, query),
        emphasised(&hit.line, query)
    ))
}

/// Returns `answer` whole as the find page's results in `locale`.
///
/// One list card per kind holds every hit as a link, with the warning when
/// the answer may not be whole, and the empty state when nothing matched.
/// It is the find page's body under its heading, and what a search shows
/// with no script; it is never an overlay.
#[must_use]
pub fn find_results(locale: &Locale, answer: &SearchAnswer) -> Html {
    let query = &answer.query;
    let unfinished = answer
        .unfinished
        .as_deref()
        .map(|why| notice(Tone::Warn, Announce::Status, None, Html::text(why)))
        .unwrap_or_default();
    let head = answer.head.as_deref().map_or_else(String::new, |head| {
        format!(
            "<p class=\"fui-find-head fui-muted\">{}</p>",
            Html::text(head)
        )
    });
    let cards: Html = answer
        .groups
        .iter()
        .filter(|group| !group.hits.is_empty())
        .map(|group| {
            let count = u64::try_from(group.hits.len()).unwrap_or(u64::MAX);
            let hits: Html = group
                .hits
                .iter()
                .map(|hit| {
                    Html::raw(format!(
                        "<li><a href=\"{}\" class=\"fui-hit\">{}</a></li>",
                        Html::text(&hit.href),
                        hit_body(hit, query)
                    ))
                })
                .collect();
            list_card(
                locale,
                ListCard {
                    title: group.kind.clone(),
                    count: Some(HeadingCount::All(count)),
                    order: None,
                    tabs: None,
                    actions: None,
                    body: Html::raw(format!("<ul class=\"fui-find-list\">{hits}</ul>")),
                    pager: None,
                },
            )
        })
        .collect();
    let nothing = if answer.found() {
        String::new()
    } else {
        let (said, line) = answer.none.clone().unwrap_or_else(|| {
            (
                tr_with(
                    locale,
                    Text::SearchNothing,
                    &[("query", query.as_str().into())],
                )
                .text()
                .to_owned(),
                String::new(),
            )
        });
        let line = if line.is_empty() {
            String::new()
        } else {
            format!("<span>{}</span>", Html::text(&line))
        };
        format!(
            "<div class=\"fui-card fui-find-empty\">{}<div><strong>{}</strong>{line}</div></div>",
            icon(Glyph::SEARCH_X, "20"),
            Html::text(&said)
        )
    };
    let more = answer.more.as_ref().map_or_else(String::new, |way| {
        format!(
            "<a href=\"{}\" class=\"fui-find-more\">{}{}</a>",
            Html::text(&way.href),
            Html::text(&way.words),
            icon(Glyph::ARROW_RIGHT, "14")
        )
    });
    Html::raw(format!(
        "<div class=\"fui-find-results\">{head}{unfinished}{cards}{nothing}{more}</div>"
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        Escape, Hit, HitGroup, SearchAnswer, Side, Target, Way, escaped, is_shortcut, placed, runs,
        step, targets, wants,
    };

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

    #[test]
    fn escape_closes_the_panel_before_it_clears_the_words() {
        assert_eq!(Escape::Close, escaped(true, false), "the words stay");
        assert_eq!(Escape::Close, escaped(true, true));
        assert_eq!(Escape::Clear, escaped(false, false));
        assert_eq!(Escape::Pass, escaped(false, true), "nothing to undo");
    }

    #[test]
    fn only_the_key_itself_is_the_shortcut() {
        assert!(is_shortcut("/", '/'));
        assert!(!is_shortcut("?", '/'));
        assert!(
            !is_shortcut("Slash", '/'),
            "a key's name is not its character"
        );
        assert!(!is_shortcut("", '/'));
    }

    #[test]
    fn the_typeahead_asks_from_its_count_of_characters() {
        assert!(!wants("a", 2));
        assert!(wants("ab", 2));
        assert!(!wants("   ", 1), "spaces alone are no query");
        assert!(wants(" é ", 1), "a character, not a byte");
        assert!(!wants("", 0));
    }

    #[test]
    fn the_arrows_walk_the_options_and_wrap() {
        assert_eq!(Some(0), step(None, 3, true));
        assert_eq!(Some(2), step(None, 3, false));
        assert_eq!(Some(1), step(Some(0), 3, true));
        assert_eq!(Some(0), step(Some(2), 3, true), "past the last");
        assert_eq!(Some(2), step(Some(0), 3, false), "before the first");
        assert_eq!(Some(1), step(Some(2), 3, false));
        assert_eq!(None, step(None, 0, true), "no option");
    }

    /// An answer of `counts` hits in each of its kinds.
    fn answer(counts: &[usize]) -> SearchAnswer {
        let groups = counts
            .iter()
            .enumerate()
            .map(|(kind, count)| HitGroup {
                kind: format!("kind-{kind}"),
                hits: (0..*count)
                    .map(|index| Hit {
                        href: format!("/{kind}/{index}"),
                        name: format!("synthetic-{index}"),
                        line: String::new(),
                        mark: None,
                        tag: None,
                    })
                    .collect(),
            })
            .collect();
        SearchAnswer {
            query: String::from("synthetic"),
            groups,
            ..SearchAnswer::default()
        }
    }

    #[test]
    fn the_panel_shows_five_of_each_kind_then_every_result() {
        let options = targets(&answer(&[7, 2]));
        assert_eq!(8, options.len(), "five, two and every result");
        assert_eq!(Some(&Target::Hit(String::from("/0/4"))), options.get(4));
        assert_eq!(Some(&Target::Hit(String::from("/1/0"))), options.get(5));
        assert_eq!(Some(&Target::All), options.last());
    }

    #[test]
    fn an_answer_with_no_hit_offers_only_its_way() {
        assert_eq!(Vec::<Target>::new(), targets(&answer(&[0])));
        let mut none = answer(&[]);
        none.more = Some(Way {
            href: String::from("/browse"),
            words: String::from("Browse"),
        });
        assert_eq!(vec![Target::Way(String::from("/browse"))], targets(&none));
    }

    #[test]
    fn the_panel_opens_from_the_edge_that_has_room() {
        assert_eq!(Side::Start, placed(500.0, 950.0, 640.0, 1440.0));
        assert_eq!(Side::End, placed(700.0, 1100.0, 640.0, 1200.0));
        assert_eq!(Side::Narrow, placed(200.0, 600.0, 640.0, 800.0));
    }
}
