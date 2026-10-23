// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The combobox: choose one of a long list, typing to filter it.
//!
//! Before the page hydrates it is a native `<select>`, which a form posts
//! with no script. Once hydrated it becomes the WAI-ARIA 1.2 combobox
//! pattern over a listbox (WAI-ARIA Authoring Practices, the select-only
//! and the editable combobox with list autocomplete): the input filters the
//! options as the reader types; Down and Up open the list and move through
//! it, Home and End go to its ends, Enter chooses, Escape closes it. Each
//! option shows its words and a muted second line, the chosen one is
//! checked, an empty filter says "No match", and the list scrolls within a
//! bounded height. It draws at most [`ROWS`] options and says when more
//! match, so typing narrows a long list. Clearing the words clears the
//! value, as the native select's empty choice does. A hidden input carries
//! the `name`, so the form posts the same value either way. A short fixed
//! list keeps the plain styled select (`view::form::SELECT`).
//!
//! A list too long to send to the browser whole, such as a code system's
//! concepts, takes a [`Remote`] ([`searched_combobox`]): the combobox writes
//! the words typed and shows the options the console's own search answers,
//! since the library fetches nothing.

use std::borrow::Cow;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::view::form::SELECT;

/// One choice of a combobox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComboOption {
    /// The value the form posts.
    pub value: String,
    /// The words the option shows.
    pub label: String,
    /// A second, muted line, such as a canonical URL.
    pub detail: Option<String>,
}

impl ComboOption {
    /// The option `value` shown as `label`, with no second line.
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            detail: None,
        }
    }

    /// This option, with `detail` as its second line.
    #[must_use]
    pub fn detail(self, detail: impl Into<String>) -> Self {
        Self {
            detail: Some(detail.into()),
            ..self
        }
    }

    /// Whether the option matches the typed `needle`, lower-cased.
    fn matches(&self, needle: &str) -> bool {
        self.label.to_lowercase().contains(needle)
            || self.value.to_lowercase().contains(needle)
            || self
                .detail
                .as_ref()
                .is_some_and(|detail| detail.to_lowercase().contains(needle))
    }
}

/// The most options the list draws at once.
pub const ROWS: usize = 100;

/// The words a combobox draws itself, resolved on the server in the page's
/// locale, so an island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComboboxWords {
    /// The line the list shows when no option matches.
    pub no_match: Word,
    /// The line under the list when more options match than it draws.
    pub more: Word,
}

impl ComboboxWords {
    /// The combobox's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            no_match: tr(Text::ComboboxNoMatch),
            more: tr(Text::ComboboxMore),
        }
    }
}

/// A console's own search behind a combobox, for a list too long to send to
/// the browser whole.
///
/// The combobox writes the words typed into `query` and shows `options` as
/// they stand, filtering none itself; the console answers `query` with the
/// options that match, through its own server function, which the library
/// never defines.
#[derive(Debug, Clone, Copy)]
pub struct Remote {
    /// The words typed, written as the reader types, empty once the list
    /// closes.
    pub query: RwSignal<String>,
    /// The options that match `query`, in their order.
    pub options: Signal<Vec<ComboOption>>,
}

/// What a combobox shows.
#[derive(Debug, Clone)]
pub struct Combobox {
    /// The control's id; the list's is this with `-list`, an option's this
    /// with `-option-` and its place.
    pub id: &'static str,
    /// The name the form posts the value as.
    pub name: &'static str,
    /// The label, such as "Code system".
    pub label: Cow<'static, str>,
    /// The words shown while nothing is chosen.
    pub placeholder: Cow<'static, str>,
    /// The choices, in their order.
    pub options: Vec<ComboOption>,
    /// The value chosen, if any.
    pub selected: RwSignal<Option<String>>,
    /// The words it draws itself ([`ComboboxWords::of_page`]).
    pub words: ComboboxWords,
}

/// What a combobox shows, with ids a console makes at run time, such as one
/// per row of a list, and the help line that describes it.
#[derive(Debug, Clone)]
pub struct KeyedCombobox {
    /// The control's id; the list's is this with `-list`, an option's this
    /// with `-option-` and its place.
    pub id: String,
    /// The name the form posts the value as.
    pub name: String,
    /// The label, such as "Code system".
    pub label: String,
    /// The words shown while nothing is chosen.
    pub placeholder: String,
    /// The choices, in their order.
    pub options: Vec<ComboOption>,
    /// The value chosen, if any.
    pub selected: RwSignal<Option<String>>,
    /// The id of the element that describes the control, such as its help
    /// line, which the input names in `aria-describedby`.
    pub described_by: Option<String>,
    /// The words it draws itself ([`ComboboxWords::of_page`]).
    pub words: ComboboxWords,
}

impl From<Combobox> for KeyedCombobox {
    fn from(combo: Combobox) -> Self {
        Self {
            id: combo.id.to_owned(),
            name: combo.name.to_owned(),
            label: combo.label.into_owned(),
            placeholder: combo.placeholder.into_owned(),
            options: combo.options,
            selected: combo.selected,
            described_by: None,
            words: combo.words,
        }
    }
}

/// Returns the combobox `combo`.
#[must_use]
pub fn combobox(combo: Combobox) -> AnyView {
    keyed_combobox(KeyedCombobox::from(combo))
}

/// Returns the combobox `combo`, whose ids are made at run time.
#[must_use]
pub fn keyed_combobox(combo: KeyedCombobox) -> AnyView {
    drawn(combo, None)
}

/// Returns the combobox `combo` over the console's search `remote`.
///
/// Its `options` are what the native select offers before the page
/// hydrates, the chosen one among them; once it hydrates the list shows
/// what `remote` answers.
#[must_use]
pub fn searched_combobox(combo: KeyedCombobox, remote: Remote) -> AnyView {
    drawn(combo, Some(remote))
}

/// Returns the combobox `combo`, over `remote` when it has one.
fn drawn(combo: KeyedCombobox, remote: Option<Remote>) -> AnyView {
    let enhanced = RwSignal::new(false);
    // An effect runs in the browser alone, after hydration, so the server's
    // native select is what hydrates, and the combobox replaces it.
    Effect::new(move |_| enhanced.set(true));
    let label = combo.label.clone();
    let id = combo.id.clone();
    let body = move || {
        if enhanced.get() {
            enhanced_view(combo.clone(), remote)
        } else {
            native_view(&combo)
        }
    };
    view! {
        <div class="fui-field fui-combobox">
            <label for=id class="fui-label">
                {label}
            </label>
            {body}
        </div>
    }
    .into_any()
}

/// The native select a page shows before it hydrates.
fn native_view(combo: &KeyedCombobox) -> AnyView {
    let selected = combo.selected;
    let chosen = selected.get_untracked();
    let options = combo
        .options
        .iter()
        .map(|option| {
            let on = chosen.as_deref() == Some(option.value.as_str());
            view! {
                <option value=option.value.clone() selected=on>
                    {option.label.clone()}
                </option>
            }
        })
        .collect_view();
    view! {
        <select
            id=combo.id.clone()
            name=combo.name.clone()
            class=SELECT
            aria-describedby=combo.described_by.clone()
            on:change:target=move |event| selected.set(Some(event.target().value()))
        >
            <option value="" selected=chosen.is_none()>
                {combo.placeholder.clone()}
            </option>
            {options}
        </select>
    }
    .into_any()
}

/// The id of the option at `place` of the combobox `id`.
fn option_id(id: &str, place: usize) -> String {
    format!("{id}-option-{place}")
}

/// The state of a hydrated combobox, shared by its handlers.
#[derive(Clone, Copy)]
struct State {
    /// The control's id.
    id: StoredValue<String>,
    /// The choices: the combobox's own, or what the console's search
    /// answered.
    options: Signal<Vec<ComboOption>>,
    /// The console's search, when the list is too long to filter here.
    remote: Option<Remote>,
    /// The words of the chosen option, kept while a search's answer no
    /// longer holds it.
    chosen_label: RwSignal<Option<String>>,
    /// The value chosen.
    selected: RwSignal<Option<String>>,
    /// Whether the list is open.
    open: RwSignal<bool>,
    /// Whether the reader has typed since the list opened.
    typing: RwSignal<bool>,
    /// The words typed.
    query: RwSignal<String>,
    /// The highlighted row, a place in `shown`.
    active: RwSignal<Option<usize>>,
    /// The places of the options the typed words keep, at most [`ROWS`].
    shown: Memo<Vec<usize>>,
    /// Whether more options match than [`ROWS`].
    more: Memo<bool>,
}

impl State {
    /// The state of a combobox over `options`, or over the console's search
    /// `remote`, choosing into `selected`.
    fn new(
        id: String,
        options: Vec<ComboOption>,
        selected: RwSignal<Option<String>>,
        remote: Option<Remote>,
    ) -> Self {
        let chosen_label = RwSignal::new(selected.with_untracked(|value| {
            options
                .iter()
                .find(|option| Some(&option.value) == value.as_ref())
                .map(|option| option.label.clone())
        }));
        let options = remote.map_or_else(|| Signal::stored(options), |remote| remote.options);
        let typing = RwSignal::new(false);
        let query = RwSignal::new(String::new());
        let matches = Memo::new(move |_| {
            let needle = query.get().trim().to_lowercase();
            let filtering = remote.is_none() && typing.get() && !needle.is_empty();
            options.with(|options| {
                options
                    .iter()
                    .enumerate()
                    .filter(|(_, option)| !filtering || option.matches(&needle))
                    .map(|(place, _)| place)
                    .collect::<Vec<_>>()
            })
        });
        let shown =
            Memo::new(move |_| matches.with(|all| all.iter().take(ROWS).copied().collect()));
        let more = Memo::new(move |_| matches.with(Vec::len) > ROWS);
        Self {
            id: StoredValue::new(id),
            options,
            remote,
            chosen_label,
            selected,
            open: RwSignal::new(false),
            typing,
            query,
            active: RwSignal::new(None),
            shown,
            more,
        }
    }

    /// The place of the chosen option, if any.
    fn chosen(self) -> Option<usize> {
        self.selected.with(|value| {
            self.options.with(|options| {
                options
                    .iter()
                    .position(|option| Some(&option.value) == value.as_ref())
            })
        })
    }

    /// The words the input shows: what was typed, or the chosen option's.
    fn words(self) -> String {
        if self.typing.get() {
            return self.query.get();
        }
        if self.selected.with(Option::is_none) {
            return String::new();
        }
        self.chosen()
            .and_then(|place| {
                self.options
                    .with(|options| options.get(place).map(|option| option.label.clone()))
            })
            .or_else(|| self.chosen_label.get())
            .unwrap_or_default()
    }

    /// Takes the words `typed`: they filter the list, or go to the console's
    /// search, and clearing them clears the value, as choosing the native
    /// select's empty option does.
    fn typed(self, typed: String) {
        if typed.trim().is_empty() {
            self.selected.set(None);
            self.chosen_label.set(None);
        }
        if let Some(remote) = self.remote {
            remote.query.set(typed.clone());
        }
        self.query.set(typed);
        self.typing.set(true);
        self.open.set(true);
        self.active
            .set((!self.shown.with_untracked(Vec::is_empty)).then_some(0));
    }

    /// Chooses the option at `place` and closes the list.
    fn choose(self, place: usize) {
        if let Some(option) = self.options.with(|options| options.get(place).cloned()) {
            self.chosen_label.set(Some(option.label));
            self.selected.set(Some(option.value));
        }
        self.close();
    }

    /// Closes the list and forgets the typed words.
    fn close(self) {
        self.open.set(false);
        self.typing.set(false);
        self.query.set(String::new());
        if let Some(remote) = self.remote {
            remote.query.set(String::new());
        }
    }

    /// Opens the list, highlighting the option at `place` or the first.
    fn open_at(self, place: Option<usize>) {
        self.open.set(true);
        let list = self.shown.get_untracked();
        let at = place
            .and_then(|place| list.iter().position(|each| *each == place))
            .or_else(|| (!list.is_empty()).then_some(0));
        self.active.set(at);
    }

    /// Moves the highlight one row down, or up for a negative `by`.
    fn step(self, by: isize) {
        let len = self.shown.with_untracked(Vec::len);
        if len == 0 {
            self.active.set(None);
            return;
        }
        let next = match self.active.get_untracked() {
            None => 0,
            Some(at) if by < 0 => at.saturating_sub(1),
            Some(at) => at.saturating_add(1).min(len.saturating_sub(1)),
        };
        self.active.set(Some(next));
    }

    /// The row of the highlighted option while the list is open.
    fn active_row(self) -> Option<usize> {
        self.open.get().then(|| self.active.get()).flatten()
    }

    /// The id of the highlighted option while the list is open.
    fn active_id(self) -> Option<String> {
        let at = self.active.get()?;
        let place = self.shown.with(|list| list.get(at).copied())?;
        self.open
            .get()
            .then(|| self.id.with_value(|id| option_id(id, place)))
    }

    /// Answers a key the reader pressed in the input.
    fn key(self, event: &leptos::ev::KeyboardEvent) {
        let open = self.open.get_untracked();
        match event.key().as_str() {
            "ArrowDown" | "ArrowUp" => {
                event.prevent_default();
                if !open {
                    self.open_at(self.chosen());
                } else if event.key() == "ArrowDown" {
                    self.step(1);
                } else {
                    self.step(-1);
                }
            }
            "Home" if open => {
                event.prevent_default();
                self.active
                    .set((!self.shown.with_untracked(Vec::is_empty)).then_some(0));
            }
            "End" if open => {
                event.prevent_default();
                self.active
                    .set(self.shown.with_untracked(Vec::len).checked_sub(1));
            }
            "Enter" if open => {
                event.prevent_default();
                let place = self
                    .active
                    .get_untracked()
                    .and_then(|at| self.shown.with_untracked(|list| list.get(at).copied()));
                if let Some(place) = place {
                    self.choose(place);
                }
            }
            "Escape" if open => {
                event.prevent_default();
                self.close();
            }
            _ => {}
        }
    }
}

/// The key of the option at `place`: its place and its value, stable while
/// the typed words change which options show, and new when a search's
/// answer puts another option at the place.
fn key(state: State, place: usize) -> (usize, String) {
    let value = state.options.with_untracked(|options| {
        options
            .get(place)
            .map(|option| option.value.clone())
            .unwrap_or_default()
    });
    (place, value)
}

/// The row of the option at `place`.
///
/// It stays in the list as long as its option is shown; only its
/// highlight and its choice change in place.
fn row(state: State, place: usize) -> AnyView {
    let Some(option) = state
        .options
        .with_untracked(|options| options.get(place).cloned())
    else {
        return ().into_any();
    };
    let value = option.value.clone();
    let is_chosen = move || {
        state
            .selected
            .with(|chosen| chosen.as_deref() == Some(value.as_str()))
    };
    let check = is_chosen.clone();
    let is_active = move || {
        state
            .active
            .get()
            .and_then(|at| state.shown.with(|list| list.get(at).copied()))
            == Some(place)
    };
    let at = move || {
        state
            .shown
            .with_untracked(|list| list.iter().position(|each| *each == place))
    };
    view! {
        <li
            id=state.id.with_value(|id| option_id(id, place))
            role="option"
            class="fui-combobox-option"
            aria-selected=move || is_chosen().to_string()
            data-active=move || is_active().then_some("true")
            on:mousedown=|event| event.prevent_default()
            on:mousemove=move |_| {
                if let Some(at) = at()
                    && state.active.get_untracked() != Some(at)
                {
                    state.active.set(Some(at));
                }
            }
            on:click=move |_| state.choose(place)
        >
            <span class="fui-combobox-text">
                <span class="fui-combobox-label">{option.label}</span>
                {option.detail.map(|detail| view! { <span class="fui-combobox-detail">{detail}</span> })}
            </span>
            <span class="fui-combobox-check" aria-hidden="true">
                {move || check().then(|| icon(Glyph::CHECK, "16"))}
            </span>
        </li>
    }
    .into_any()
}

/// The combobox a hydrated page shows, over `remote` when it has one.
fn enhanced_view(combo: KeyedCombobox, remote: Option<Remote>) -> AnyView {
    let KeyedCombobox {
        id,
        name,
        label,
        placeholder,
        options,
        selected,
        described_by,
        words: ComboboxWords { no_match, more },
    } = combo;
    let list_id = format!("{id}-list");
    let state = State::new(id.clone(), options, selected, remote);
    let list = NodeRef::<leptos::html::Ul>::new();
    Effect::new(move |_| {
        if let Some(row) = state.active_row() {
            reveal(list, row);
        }
    });
    let controls = list_id.clone();
    view! {
        <div class="fui-combobox-box">
            <input
                id=id
                type="text"
                role="combobox"
                class="fui-input fui-combobox-input"
                autocomplete="off"
                aria-autocomplete="list"
                aria-expanded=move || state.open.get().to_string()
                aria-controls=controls
                aria-activedescendant=move || state.active_id()
                aria-describedby=described_by
                placeholder=placeholder
                prop:value=move || state.words()
                on:click=move |_| {
                    if state.open.get_untracked() {
                        state.close();
                    } else {
                        state.open_at(state.chosen());
                    }
                }
                on:input:target=move |event| state.typed(event.target().value())
                on:keydown=move |event| state.key(&event)
                on:blur=move |_| state.close()
            />
            <input type="hidden" name=name prop:value=move || selected.get().unwrap_or_default() />
            <ul
                id=list_id
                node_ref=list
                role="listbox"
                aria-label=label
                class="fui-combobox-list"
                hidden=move || !state.open.get()
            >
                <For
                    each=move || state.shown.get()
                    key=move |place| key(state, *place)
                    children=move |place| row(state, place)
                />
                {move || {
                    state
                        .shown
                        .with(Vec::is_empty)
                        .then(|| {
                            view! {
                                <li
                                    class="fui-combobox-empty"
                                    role="option"
                                    aria-disabled="true"
                                    aria-selected="false"
                                >
                                    {no_match.clone()}
                                </li>
                            }
                        })
                }}
                {move || {
                    state
                        .more
                        .get()
                        .then(|| {
                            view! {
                                <li
                                    class="fui-combobox-empty"
                                    role="option"
                                    aria-disabled="true"
                                    aria-selected="false"
                                >
                                    {more.clone()}
                                </li>
                            }
                        })
                }}
            </ul>
        </div>
    }
    .into_any()
}

/// Scrolls the list `list` so its row `row` is in sight.
fn reveal(list: NodeRef<leptos::html::Ul>, row: usize) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let Some(list) = list.get_untracked() else {
            return;
        };
        let mut option = list.first_element_child();
        for _ in 0..row {
            option = option.and_then(|before| before.next_element_sibling());
        }
        let option = option.and_then(|option| option.dyn_into::<web_sys::HtmlElement>().ok());
        let Some(option) = option else {
            return;
        };
        let top = option.offset_top();
        let bottom = top.saturating_add(option.offset_height());
        let shown_top = list.scroll_top();
        let shown_bottom = shown_top.saturating_add(list.client_height());
        if top < shown_top {
            list.set_scroll_top(top);
        } else if bottom > shown_bottom {
            list.set_scroll_top(bottom.saturating_sub(list.client_height()));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (NodeRef<leptos::html::Ul>, usize) = (list, row);
}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;

    use super::{ComboOption, ROWS, Remote, State};

    fn options(count: usize) -> Vec<ComboOption> {
        (0..count)
            .map(|place| ComboOption::new(format!("code-{place}"), format!("Concept {place}")))
            .collect()
    }

    #[test]
    fn clearing_the_words_clears_the_value() {
        let owner = Owner::new();
        owner.with(|| {
            let selected = RwSignal::new(Some(String::from("code-1")));
            let state = State::new(String::from("c"), options(3), selected, None);
            assert_eq!("Concept 1", state.words());
            state.typed(String::from("Conc"));
            assert_eq!(
                Some(String::from("code-1")),
                selected.get(),
                "typing keeps it"
            );
            state.typed(String::new());
            assert_eq!(None, selected.get(), "an optional field empties again");
            state.close();
            assert_eq!("", state.words());
        });
    }

    #[test]
    fn the_list_draws_at_most_its_rows_and_says_more_match() {
        let owner = Owner::new();
        owner.with(|| {
            let state = State::new(
                String::from("c"),
                options(ROWS + 5),
                RwSignal::new(None),
                None,
            );
            assert_eq!(ROWS, state.shown.with(Vec::len));
            assert!(state.more.get(), "the reader is told to type to narrow it");
            state.typed(String::from("concept 10"));
            assert!(!state.more.get(), "narrowed below the limit");
        });
    }

    #[test]
    fn a_console_search_gets_the_words_and_its_answer_shows_unfiltered() {
        let owner = Owner::new();
        owner.with(|| {
            let query = RwSignal::new(String::new());
            let answer = RwSignal::new(options(2));
            let remote = Remote {
                query,
                options: answer.into(),
            };
            let selected = RwSignal::new(None);
            let state = State::new(String::from("c"), Vec::new(), selected, Some(remote));
            state.typed(String::from("zzz"));
            assert_eq!("zzz", query.get(), "the console's search reads the words");
            assert_eq!(
                vec![0, 1],
                state.shown.get(),
                "its answer is not filtered again"
            );
            state.choose(1);
            answer.set(Vec::new());
            assert_eq!(Some(String::from("code-1")), selected.get());
            assert_eq!("Concept 1", state.words(), "the choice keeps its words");
            assert_eq!("", query.get(), "closing the list ends the search");
        });
    }

    #[test]
    fn an_option_matches_its_words_its_value_and_its_second_line() {
        let option = ComboOption::new("loinc", "LOINC").detail("http://loinc.org");
        assert!(option.matches("loi"));
        assert!(option.matches("loinc.org"));
        assert!(!option.matches("snomed"));
    }
}
