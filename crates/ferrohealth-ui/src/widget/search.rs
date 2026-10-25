// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keys, the typeahead and the busy state of the top bar's search box,
//! the island [`crate::island::Search`] draws through [`search_control`].
//!
//! The search owns its own shortcut (#144): its key (`/` by default) from
//! anywhere outside an element that takes text focuses the field with the
//! caret after its text, unless a listener of the page took the key first.
//! With an answer route ([`SearchBox::suggest`]) the field is a combobox
//! with a listbox popup (WAI-ARIA Authoring Practices 1.2, the combobox
//! pattern): typing asks the route once the words rest for [`SETTLE`], and
//! the panel opens under the field with a row while it waits, the hits
//! grouped by kind, a row when nothing matched, and a row for an error.
//! `ArrowDown` and `ArrowUp` move the active option through every group with
//! the focus kept in the field (`aria-activedescendant`), Enter follows the
//! active option and otherwise submits the form, Tab and a press outside
//! close the panel, and Escape closes an open panel and on a closed one
//! clears the words, the focus staying in the field each time (WCAG 2.2 SC
//! 2.4.3). A submit turns the glyph into the spinner while the next page
//! loads. With no WebAssembly the form submits to the find page as it is
//! ([`crate::view::search`]). No specification governs the timing or the
//! shortcut: our own design; the shortcut is announced through
//! `aria-keyshortcuts` (WAI-ARIA 1.2).

use std::time::Duration;

use leptos::either::Either;
use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::view::search::{
    PER_GROUP, SearchAnswer, SearchBox, SearchWords, answer_id, hit_body, hit_id,
};

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

/// Why the typeahead has no answer to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    /// The console answered with the HTTP status.
    Status(u16),
    /// The console could not be reached.
    Unreachable,
    /// The console's answer was not a search answer.
    Unreadable,
}

/// What the panel holds.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(dead_code, reason = "only the browser receives an answer")
)]
enum Phase {
    /// Nothing was asked.
    Idle,
    /// The words were asked and the answer has not come.
    Asking,
    /// The console's answer.
    Answered(SearchAnswer),
    /// The console gave no answer.
    Failed(Failure),
}

/// What the typeahead asks with.
#[derive(Debug, Clone)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(dead_code, reason = "only the browser asks")
)]
struct Ask {
    /// The console's answer route.
    endpoint: String,
    /// The query key.
    name: String,
    /// Whether the words travel in a post's body.
    post: bool,
    /// How many characters a query holds before it is asked.
    from: u8,
    /// The address the page loads on a `401` answer, if any.
    signed_out: Option<String>,
}

/// The typeahead's state, shared by the field's handlers.
#[derive(Debug, Clone, Copy)]
struct Typeahead {
    /// What the panel holds.
    phase: RwSignal<Phase>,
    /// Whether the panel is open.
    open: RwSignal<bool>,
    /// The option the keys or the pointer are on.
    active: RwSignal<Option<usize>>,
    /// The wait before the next ask.
    pending: StoredValue<Option<TimeoutHandle>>,
    /// The number of the latest ask; an answer to an older one is dropped.
    asked: StoredValue<u32>,
    /// What the typeahead asks with, when the console names a route.
    ask: StoredValue<Option<Ask>>,
}

impl Typeahead {
    /// Closes the panel.
    fn close(self) {
        self.open.set(false);
        self.active.set(None);
    }

    /// Drops what was asked and what was answered.
    fn reset(self) {
        if let Some(handle) = self.pending.get_value() {
            handle.clear();
        }
        self.pending.set_value(None);
        self.asked
            .update_value(|asked| *asked = asked.wrapping_add(1));
        self.phase.set(Phase::Idle);
        self.close();
    }

    /// The options of the panel's answer.
    fn targets(self) -> Vec<Target> {
        self.phase.with_untracked(|phase| match phase {
            Phase::Answered(answer) => targets(answer),
            Phase::Idle | Phase::Asking | Phase::Failed(_) => Vec::new(),
        })
    }
}

/// Returns the search box `search`, its panel under it drawn in `words`.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "the form, its field's combobox state and its submit are one view whose parts share the typeahead's signals"
)]
pub fn search_control(search: SearchBox, words: SearchWords) -> impl IntoView {
    let SearchBox {
        id,
        name,
        label,
        placeholder,
        action,
        value,
        submit,
        shortcut,
        busy,
        disabled,
        post,
        suggest,
        suggest_from,
        all: _,
        signed_out,
    } = search;
    let field = NodeRef::<leptos::html::Input>::new();
    let form = NodeRef::<leptos::html::Form>::new();
    let panel = NodeRef::<leptos::html::Div>::new();
    let busy = RwSignal::new(busy);
    let off = disabled.is_some();
    let typeahead = suggest.is_some() && !off;
    let state = Typeahead {
        phase: RwSignal::new(Phase::Idle),
        open: RwSignal::new(false),
        active: RwSignal::new(None),
        pending: StoredValue::new(None),
        asked: StoredValue::new(0),
        ask: StoredValue::new(suggest.filter(|_| !off).map(|endpoint| Ask {
            endpoint,
            name: name.clone(),
            post,
            from: suggest_from,
            signed_out,
        })),
    };
    on_cleanup(move || {
        if let Some(handle) = state.pending.try_get_value().flatten() {
            handle.clear();
        }
    });
    let hydrated = RwSignal::new(false);
    let side = RwSignal::new(Side::Start);
    Effect::new(move |_| {
        hydrated.set(true);
        listen(field, shortcut, busy);
    });
    follow_active(&id, state);
    place_on_open(form, panel, state, side);
    let choose = move |at: usize| pick(at, state, form, busy);
    let on_key = move |event: leptos::ev::KeyboardEvent| keyed(&event, field, state, choose);
    let named = submit.clone();
    let why = format!("{id}-why");
    let described = off.then(|| why.clone());
    let reason = disabled.clone();
    let hint = shortcut
        .filter(|_| !off)
        .map(|key| view! { <kbd class="fui-kbd fui-search-hint" aria-hidden="true">{key.to_string()}</kbd> });
    let keys = shortcut.filter(|_| !off).map(|key| key.to_string());
    let shown = disabled.clone().unwrap_or(placeholder);
    let glyph = move || {
        if busy.get() {
            Either::Left(view! { <span class="fui-spinner-ring" aria-hidden="true"></span> })
        } else {
            Either::Right(icon(Glyph::SEARCH, "16"))
        }
    };
    let combobox = move || typeahead && hydrated.get();
    let list = answer_id(&id);
    let controls = list.clone();
    let owner = id.clone();
    let active_id = move || {
        let at = state.active.get()?;
        state
            .phase
            .with(|phase| match phase {
                Phase::Answered(answer) => targets(answer).get(at).map(|target| (*target).clone()),
                Phase::Idle | Phase::Asking | Phase::Failed(_) => None,
            })
            .map(|target| target_id(&owner, &target, at))
    };
    let drawn = typeahead.then(|| panel_view(&id, list, words, state, side, panel, choose));
    view! {
        <form
            role="search"
            method=if post { "post" } else { "get" }
            action=action
            class="fui-search"
            aria-busy=move || busy.get().then_some("true")
            node_ref=form
            on:submit=move |_| {
                state.close();
                busy.set(true);
            }
        >
            <label for=id.clone() class="fui-sr-only">
                {label}
            </label>
            <div class="fui-search-field">
                <span class="fui-search-glyph">{glyph}</span>
                <input
                    id=id
                    name=name
                    type="search"
                    class="fui-input"
                    placeholder=shown
                    value=value
                    autocomplete="off"
                    spellcheck="false"
                    role=move || combobox().then_some("combobox")
                    aria-autocomplete=move || combobox().then_some("list")
                    aria-expanded=move || {
                        combobox().then(|| if state.open.get() { "true" } else { "false" })
                    }
                    aria-controls=move || combobox().then(|| controls.clone())
                    aria-activedescendant=move || if combobox() { active_id() } else { None }
                    aria-keyshortcuts=keys
                    aria-describedby=described
                    title=reason.clone()
                    disabled=off
                    node_ref=field
                    on:keydown=on_key
                    on:input=move |_| typed(field, state)
                    on:focus=move |_| reopen(state)
                    on:blur=move |_| state.close()
                />
                {hint}
            </div>
            <button
                type="submit"
                class="fui-search-submit"
                aria-label=named
                disabled=move || off || busy.get()
            >
                {icon(Glyph::SEARCH, "16")}
                <span class="fui-search-word">{submit}</span>
            </button>
            {reason.map(|reason| view! { <span id=why class="fui-sr-only">{reason}</span> })}
            {drawn}
        </form>
    }
}

/// The id of the option `target`, the `at`th of the panel under the search
/// box `id`.
fn target_id(id: &str, target: &Target, at: usize) -> String {
    match target {
        Target::Hit(_) => hit_id(
            id,
            u32::try_from(at).map_or(u32::MAX, |at| at.saturating_add(1)),
        ),
        Target::All => format!("{id}-all"),
        Target::Way(_) => format!("{id}-more"),
    }
}

/// Returns the panel under the search box `id`: the row of what is
/// happening, and the listbox `list` of the options, drawn in `words`.
fn panel_view(
    id: &str,
    list: String,
    words: SearchWords,
    state: Typeahead,
    side: RwSignal<Side>,
    panel: NodeRef<leptos::html::Div>,
    pick: impl Fn(usize) + Copy + Send + Sync + 'static,
) -> AnyView {
    let SearchWords {
        answer: named,
        searching,
        nothing,
        all,
        counted,
        failed,
        unreachable,
        unreadable,
    } = words;
    let owner = id.to_owned();
    let failure = move |failure: Failure| match failure {
        Failure::Status(status) => failed.fill(&[("status", &status.to_string())]),
        Failure::Unreachable => unreachable.clone(),
        Failure::Unreadable => unreadable.clone(),
    };
    let said_nothing = nothing.clone();
    let said = move || {
        state.phase.with(|phase| match phase {
            Phase::Idle => String::new(),
            Phase::Asking => searching.text().to_owned(),
            Phase::Answered(answer) if answer.found() => {
                let count = targets(answer)
                    .iter()
                    .filter(|target| matches!(target, Target::Hit(_)))
                    .count()
                    .to_string();
                counted
                    .fill(&[("query", &answer.query), ("count", &count)])
                    .text()
                    .to_owned()
            }
            Phase::Answered(answer) => answer.none.as_ref().map_or_else(
                || {
                    said_nothing
                        .fill(&[("query", &answer.query)])
                        .text()
                        .to_owned()
                },
                |(sentence, _)| sentence.clone(),
            ),
            Phase::Failed(why) => failure(*why).text().to_owned(),
        })
    };
    let searching = said.clone();
    let rows = move || {
        state.phase.with(|phase| match phase {
            Phase::Idle => ().into_any(),
            Phase::Asking => view! {
                <p class="fui-search-results-status">
                    <span class="fui-spinner-ring" aria-hidden="true"></span>
                    {searching()}
                </p>
            }
            .into_any(),
            Phase::Failed(_) => view! {
                <p class="fui-search-results-status fui-tone-bad">
                    {icon(Glyph::CIRCLE_ALERT, "16")}
                    {searching()}
                </p>
            }
            .into_any(),
            Phase::Answered(answer) => answered_rows(answer, &nothing).into_any(),
        })
    };
    let options = move || {
        state.phase.with(|phase| match phase {
            Phase::Answered(answer) => options_view(&owner, answer, &all, state, pick).into_any(),
            Phase::Idle | Phase::Asking | Phase::Failed(_) => ().into_any(),
        })
    };
    let class = move || match side.get() {
        Side::Start => "fui-search-results",
        Side::End => "fui-search-results is-end",
        Side::Narrow => "fui-search-results is-narrow",
    };
    view! {
        <div
            class=class
            id=format!("{id}-panel")
            hidden=move || !state.open.get()
            node_ref=panel
            on:mousedown=|event| event.prevent_default()
        >
            {rows}
            <div role="listbox" id=list aria-label=named class="fui-search-results-list">
                {options}
            </div>
        </div>
        <span role="status" class="fui-sr-only">
            {said}
        </span>
    }
    .into_any()
}

/// Returns the rows over the options of `answer`: its head, the warning
/// when it may not be whole, and the empty row in `nothing` when nothing
/// matched.
fn answered_rows(answer: &SearchAnswer, nothing: &crate::i18n::Phrase) -> impl IntoView + use<> {
    let head = answer
        .head
        .clone()
        .map(|head| view! { <p class="fui-search-results-head">{head}</p> });
    let unfinished = answer.unfinished.clone().map(|why| {
        view! {
            <p class="fui-search-results-status fui-tone-warn">
                {icon(Glyph::TRIANGLE_ALERT, "16")}
                {why}
            </p>
        }
    });
    let empty = (!answer.found()).then(|| {
        let (sentence, line) = answer.none.clone().unwrap_or_else(|| {
            (
                nothing.fill(&[("query", &answer.query)]).text().to_owned(),
                String::new(),
            )
        });
        let line = (!line.is_empty()).then(|| view! { <span>{line}</span> });
        view! {
            <div class="fui-search-results-empty">
                {icon(Glyph::SEARCH_X, "16")}
                <div>
                    <strong>{sentence}</strong>
                    {line}
                </div>
            </div>
        }
    });
    view! {
        {head}
        {unfinished}
        {empty}
    }
}

/// Returns the options of `answer` under the search box `id`: each kind a
/// group under its heading, then the option for every result in `all`, then
/// the way past; `pick` follows an option.
fn options_view(
    id: &str,
    answer: &SearchAnswer,
    all: &crate::i18n::Phrase,
    state: Typeahead,
    pick: impl Fn(usize) + Copy + Send + Sync + 'static,
) -> AnyView {
    let query = answer.query.clone();
    let mut at = 0_usize;
    let groups = answer
        .groups
        .iter()
        .filter(|group| !group.hits.is_empty())
        .zip(1_u32..)
        .map(|(group, index)| {
            let kind_id = format!("{id}-kind-{index}");
            let labelled = kind_id.clone();
            let hits = group
                .hits
                .iter()
                .take(PER_GROUP)
                .map(|hit| {
                    let place = at;
                    at += 1;
                    let option = target_id(id, &Target::Hit(String::new()), place);
                    option_view(option, place, "fui-hit", state, pick, hit_body(hit, &query))
                })
                .collect_view();
            view! {
                <div role="group" aria-labelledby=labelled>
                    <div role="presentation" id=kind_id class="fui-search-results-group">
                        {group.kind.clone()}
                    </div>
                    {hits}
                </div>
            }
        })
        .collect_view();
    let every = answer.found().then(|| {
        let place = at;
        at += 1;
        let words = all.fill(&[("query", &answer.query)]);
        let body = view! {
            <span>{words}</span>
            {icon(Glyph::ARROW_RIGHT, "14")}
        }
        .into_any();
        option_view(
            target_id(id, &Target::All, place),
            place,
            "fui-search-results-more",
            state,
            pick,
            body,
        )
    });
    let more = answer.more.clone().map(|way| {
        let body = view! {
            <span>{way.words}</span>
            {icon(Glyph::ARROW_RIGHT, "14")}
        }
        .into_any();
        option_view(
            target_id(id, &Target::Way(way.href), at),
            at,
            "fui-search-results-more",
            state,
            pick,
            body,
        )
    });
    view! {
        {groups}
        {every}
        {more}
    }
    .into_any()
}

/// Returns the option `id`, the `at`th, in the class `class`, holding
/// `body`: active under the pointer, followed by `pick` on a press.
fn option_view(
    id: String,
    at: usize,
    class: &'static str,
    state: Typeahead,
    pick: impl Fn(usize) + Copy + Send + Sync + 'static,
    body: AnyView,
) -> AnyView {
    let on = move || state.active.get() == Some(at);
    view! {
        <div
            role="option"
            id=id
            class=class
            aria-selected=move || if on() { "true" } else { "false" }
            on:mousemove=move |_| {
                if !on() {
                    state.active.set(Some(at));
                }
            }
            on:click=move |_| pick(at)
        >
            {body}
        </div>
    }
    .into_any()
}

/// Answers a key in `field`: the arrows walk the options, Enter follows the
/// active one through `pick`, Escape closes or clears, Tab closes.
fn keyed(
    event: &leptos::ev::KeyboardEvent,
    field: NodeRef<leptos::html::Input>,
    state: Typeahead,
    pick: impl Fn(usize),
) {
    let key = event.key();
    match key.as_str() {
        "ArrowDown" | "ArrowUp" => {
            let count = state.targets().len();
            if count == 0 {
                return;
            }
            event.prevent_default();
            if !state.open.get_untracked() {
                state.open.set(true);
            }
            let down = key == "ArrowDown";
            state
                .active
                .set(step(state.active.get_untracked(), count, down));
        }
        "Enter" => {
            if state.open.get_untracked()
                && let Some(at) = state.active.get_untracked()
            {
                event.prevent_default();
                pick(at);
            }
        }
        "Escape" => escape(event, field, state),
        "Tab" => state.close(),
        _ => {}
    }
}

/// Follows the option `at` of the panel: a hit's or the way's page, or the
/// form's own submit for every result.
fn pick(at: usize, state: Typeahead, form: NodeRef<leptos::html::Form>, busy: RwSignal<bool>) {
    let Some(target) = state.targets().into_iter().nth(at) else {
        return;
    };
    state.close();
    match target {
        Target::Hit(href) | Target::Way(href) => {
            busy.set(true);
            go(&href);
        }
        Target::All => submit_form(form),
    }
}

/// Answers Escape in `field`: closes the panel when it is open, and
/// otherwise clears the words, the focus staying in the field.
fn escape(
    event: &leptos::ev::KeyboardEvent,
    field: NodeRef<leptos::html::Input>,
    state: Typeahead,
) {
    let empty = field_value(field).is_none_or(|words| words.is_empty());
    match escaped(state.open.get_untracked(), empty) {
        Escape::Close => {
            event.prevent_default();
            state.close();
        }
        Escape::Clear => {
            event.prevent_default();
            clear(field);
            state.reset();
        }
        Escape::Pass => {}
    }
}

/// Opens the panel again when the field takes the focus back over an
/// answer or an error.
fn reopen(state: Typeahead) {
    let shown = state
        .phase
        .with_untracked(|phase| matches!(phase, Phase::Answered(_) | Phase::Failed(_)));
    if shown {
        state.open.set(true);
    }
}

/// Answers new words in `field`: asks for them once they rest, or closes
/// the panel when they are too short to ask.
fn typed(field: NodeRef<leptos::html::Input>, state: Typeahead) {
    state.active.set(None);
    let Some(ask) = state.ask.get_value() else {
        return;
    };
    let words = field_value(field).unwrap_or_default();
    if let Some(handle) = state.pending.get_value() {
        handle.clear();
    }
    state.pending.set_value(None);
    if !wants(&words, ask.from) {
        state.reset();
        return;
    }
    let handle = set_timeout_with_handle(move || send(state, ask, words), SETTLE);
    if handle.is_err() {
        leptos::logging::warn!("the browser refused the search's wait");
    }
    state.pending.set_value(handle.ok());
}

/// Asks the console's route for the words `words` and shows the answer,
/// unless a newer ask has started.
fn send(state: Typeahead, ask: Ask, words: String) {
    // NOTE: no specification governs this: our own design; the wait can end after a
    // navigation disposed the search, so the state is written only while it lives.
    if state
        .asked
        .try_update_value(|asked| *asked = asked.wrapping_add(1))
        .is_none()
    {
        return;
    }
    let Some(mine) = state.asked.try_get_value() else {
        return;
    };
    drop(state.phase.try_set(Phase::Asking));
    let _: Option<bool> = state.open.try_set(true);
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        let answered = fetch(&ask, words.trim()).await;
        if state.asked.try_get_value() != Some(mine) {
            return;
        }
        if let (Err(Failure::Status(401)), Some(to)) = (&answered, ask.signed_out.as_deref()) {
            if window().location().assign(to).is_err() {
                leptos::logging::warn!("the browser refused the signed-out page");
            }
            return;
        }
        state.phase.try_set(match answered {
            Ok(answer) => Phase::Answered(answer),
            Err(failure) => Phase::Failed(failure),
        });
    });
    #[cfg(not(target_arch = "wasm32"))]
    drop((ask, words, mine));
}

/// Asks the route `ask` names for the words `words`: a `GET` with them in
/// the query, or a `POST` with them in a form body, that takes JSON.
#[cfg(target_arch = "wasm32")]
async fn fetch(ask: &Ask, words: &str) -> Result<SearchAnswer, Failure> {
    use leptos::wasm_bindgen::{JsCast as _, JsValue};
    use wasm_bindgen_futures::JsFuture;
    let query = web_sys::UrlSearchParams::new().map_err(|error| unreachable(&error))?;
    query.append(&ask.name, words);
    let encoded = String::from(query.to_string());
    let headers = web_sys::Headers::new().map_err(|error| unreachable(&error))?;
    headers
        .set("Accept", "application/json")
        .map_err(|error| unreachable(&error))?;
    let init = web_sys::RequestInit::new();
    init.set_credentials(web_sys::RequestCredentials::SameOrigin);
    let url = if ask.post {
        init.set_method("POST");
        headers
            .set("Content-Type", "application/x-www-form-urlencoded")
            .map_err(|error| unreachable(&error))?;
        init.set_body(&JsValue::from_str(&encoded));
        ask.endpoint.clone()
    } else {
        init.set_method("GET");
        let joint = if ask.endpoint.contains('?') { '&' } else { '?' };
        format!("{}{joint}{encoded}", ask.endpoint)
    };
    init.set_headers(&headers);
    let request = web_sys::Request::new_with_str_and_init(&url, &init)
        .map_err(|error| unreachable(&error))?;
    let response: web_sys::Response = JsFuture::from(window().fetch_with_request(&request))
        .await
        .map_err(|error| unreachable(&error))?
        .dyn_into()
        .map_err(|error| unreachable(&error))?;
    if !response.ok() {
        return Err(Failure::Status(response.status()));
    }
    let text = response.text().map_err(|error| unreachable(&error))?;
    let text = JsFuture::from(text)
        .await
        .ok()
        .and_then(|text| text.as_string())
        .ok_or(Failure::Unreadable)?;
    leptos::serde_json::from_str(&text).map_err(|error| {
        let (line, column) = (error.line(), error.column());
        leptos::logging::warn!("the search's answer is unreadable at {line}:{column}");
        Failure::Unreadable
    })
}

/// The failure of a request the browser refused with `error`, which the
/// console log keeps.
#[cfg(target_arch = "wasm32")]
fn unreachable(error: &leptos::wasm_bindgen::JsValue) -> Failure {
    leptos::logging::warn!("the browser refused the search's request: {error:?}");
    Failure::Unreachable
}

/// Keeps the active option of the panel under the search box `id` in view
/// inside the panel.
fn follow_active(id: &str, state: Typeahead) {
    #[cfg(target_arch = "wasm32")]
    {
        let id = id.to_owned();
        Effect::new(move |_| {
            let Some(at) = state.active.get() else {
                return;
            };
            let Some(target) = state.targets().into_iter().nth(at) else {
                return;
            };
            let Some(option) = document().get_element_by_id(&target_id(&id, &target, at)) else {
                return;
            };
            let options = web_sys::ScrollIntoViewOptions::new();
            options.set_block(web_sys::ScrollLogicalPosition::Nearest);
            option.scroll_into_view_with_scroll_into_view_options(&options);
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (&str, Typeahead) = (id, state);
}

/// Places the panel each time it opens: from the field's start edge, or
/// from its end edge or as wide as the field when the window has no room.
fn place_on_open(
    form: NodeRef<leptos::html::Form>,
    panel: NodeRef<leptos::html::Div>,
    state: Typeahead,
    side: RwSignal<Side>,
) {
    #[cfg(target_arch = "wasm32")]
    Effect::watch(
        move || state.open.get(),
        move |open, _, _| {
            if !*open {
                return;
            }
            side.set(Side::Start);
            request_animation_frame(move || {
                let (Some(form), Some(panel)) = (form.get_untracked(), panel.get_untracked())
                else {
                    return;
                };
                let phone = window()
                    .match_media("(width < 48rem)")
                    .ok()
                    .flatten()
                    .is_some_and(|query| query.matches());
                if phone {
                    return;
                }
                let view = window()
                    .inner_width()
                    .ok()
                    .and_then(|width| width.as_f64())
                    .unwrap_or(f64::MAX);
                let field = form.get_bounding_client_rect();
                let wide = panel.get_bounding_client_rect().width();
                side.set(placed(field.left(), field.right(), wide, view));
            });
        },
        false,
    );
    #[cfg(not(target_arch = "wasm32"))]
    let _: (
        NodeRef<leptos::html::Form>,
        NodeRef<leptos::html::Div>,
        Typeahead,
        RwSignal<Side>,
    ) = (form, panel, state, side);
}

/// Listens on the window for the key `shortcut`, which focuses `field`, and
/// for the page coming back from the history, which ends `busy`.
fn listen(field: NodeRef<leptos::html::Input>, shortcut: Option<char>, busy: RwSignal<bool>) {
    #[cfg(target_arch = "wasm32")]
    {
        let back = window_event_listener(leptos::ev::pageshow, move |_| busy.set(false));
        let Some(shortcut) = shortcut else {
            on_cleanup(move || back.remove());
            return;
        };
        let keys = window_event_listener(leptos::ev::keydown, move |event| {
            if event.default_prevented()
                || !is_shortcut(&event.key(), shortcut)
                || crate::widget::shortcuts::chorded(&event)
                || crate::widget::shortcuts::typing(&event)
            {
                return;
            }
            let Some(input) = field.get_untracked() else {
                return;
            };
            if input.disabled() {
                return;
            }
            event.prevent_default();
            if input.focus().is_err() {
                leptos::logging::warn!("the browser refused the focus on the search field");
            }
            let end = u32::try_from(input.value().encode_utf16().count()).unwrap_or(u32::MAX);
            if input.set_selection_range(end, end).is_err() {
                leptos::logging::warn!("the browser refused the caret in the search field");
            }
        });
        on_cleanup(move || {
            back.remove();
            keys.remove();
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (NodeRef<leptos::html::Input>, Option<char>, RwSignal<bool>) = (field, shortcut, busy);
}

/// The text in `field`, once it is in the page.
fn field_value(field: NodeRef<leptos::html::Input>) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        field.get_untracked().map(|input| input.value())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _: NodeRef<leptos::html::Input> = field;
        None
    }
}

/// Clears `field`, which keeps the focus.
fn clear(field: NodeRef<leptos::html::Input>) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(input) = field.get_untracked() {
            input.set_value("");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: NodeRef<leptos::html::Input> = field;
}

/// Goes to `href`, as a press on a link to it does.
fn go(href: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        crate::widget::submit::follow(href);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = href;
}

/// Submits `form` as its submit button does, so the submit's handlers run.
fn submit_form(form: NodeRef<leptos::html::Form>) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(form) = form.get_untracked()
            && form.request_submit().is_err()
        {
            leptos::logging::warn!("the browser refused to submit the search");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: NodeRef<leptos::html::Form> = form;
}

#[cfg(test)]
mod tests {
    use super::{Escape, Side, Target, escaped, is_shortcut, placed, step, targets, wants};
    use crate::view::search::{Hit, HitGroup, SearchAnswer, Way};

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
