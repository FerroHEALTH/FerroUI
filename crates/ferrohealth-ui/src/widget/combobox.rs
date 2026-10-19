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
//! bounded height. A hidden input carries the `name`, so the form posts the
//! same value either way. A short fixed list keeps the plain styled select
//! (`view::form::SELECT`).

use std::borrow::Cow;

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::view::form::SELECT;

/// One choice of a combobox.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    let enhanced = RwSignal::new(false);
    // An effect runs in the browser alone, after hydration, so the server's
    // native select is what hydrates, and the combobox replaces it.
    Effect::new(move |_| enhanced.set(true));
    let label = combo.label.clone();
    let id = combo.id.clone();
    let body = move || {
        if enhanced.get() {
            enhanced_view(combo.clone())
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
    /// The choices.
    options: StoredValue<Vec<ComboOption>>,
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
    /// The places of the options the typed words keep.
    shown: Memo<Vec<usize>>,
}

impl State {
    /// The state of a combobox over `options`, choosing into `selected`.
    fn new(id: String, options: Vec<ComboOption>, selected: RwSignal<Option<String>>) -> Self {
        let options = StoredValue::new(options);
        let typing = RwSignal::new(false);
        let query = RwSignal::new(String::new());
        let shown = Memo::new(move |_| {
            let needle = query.get().trim().to_lowercase();
            let filtering = typing.get() && !needle.is_empty();
            options.with_value(|options| {
                options
                    .iter()
                    .enumerate()
                    .filter(|(_, option)| !filtering || option.matches(&needle))
                    .map(|(place, _)| place)
                    .collect::<Vec<_>>()
            })
        });
        Self {
            id: StoredValue::new(id),
            options,
            selected,
            open: RwSignal::new(false),
            typing,
            query,
            active: RwSignal::new(None),
            shown,
        }
    }

    /// The place of the chosen option, if any.
    fn chosen(self) -> Option<usize> {
        self.selected.with(|value| {
            self.options.with_value(|options| {
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
        self.chosen()
            .and_then(|place| {
                self.options
                    .with_value(|options| options.get(place).map(|option| option.label.clone()))
            })
            .unwrap_or_default()
    }

    /// Chooses the option at `place` and closes the list.
    fn choose(self, place: usize) {
        if let Some(value) = self
            .options
            .with_value(|options| options.get(place).map(|option| option.value.clone()))
        {
            self.selected.set(Some(value));
        }
        self.close();
    }

    /// Closes the list and forgets the typed words.
    fn close(self) {
        self.open.set(false);
        self.typing.set(false);
        self.query.set(String::new());
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

/// The rows of the list: the options kept, or "No match".
fn rows(state: State, no_match: String) -> AnyView {
    let list = state.shown.get();
    if list.is_empty() {
        return view! {
            <li class="fui-combobox-empty" role="option" aria-disabled="true" aria-selected="false">
                {no_match}
            </li>
        }
        .into_any();
    }
    list.into_iter()
        .enumerate()
        .map(|(at, place)| row(state, at, place))
        .collect_view()
        .into_any()
}

/// The row at `at` of the list, the option at `place`.
fn row(state: State, at: usize, place: usize) -> AnyView {
    let Some(option) = state
        .options
        .with_value(|options| options.get(place).cloned())
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
    view! {
        <li
            id=state.id.with_value(|id| option_id(id, place))
            role="option"
            class="fui-combobox-option"
            aria-selected=move || is_chosen().to_string()
            data-active=move || (state.active.get() == Some(at)).then_some("true")
            on:mousedown=|event| event.prevent_default()
            on:mousemove=move |_| state.active.set(Some(at))
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

/// The combobox a hydrated page shows.
fn enhanced_view(combo: KeyedCombobox) -> AnyView {
    let KeyedCombobox {
        id,
        name,
        label,
        placeholder,
        options,
        selected,
        described_by,
    } = combo;
    let list_id = format!("{id}-list");
    let state = State::new(id.clone(), options, selected);
    let no_match = tr(Text::ComboboxNoMatch);
    let revealed = list_id.clone();
    Effect::new(move |_| {
        if let Some(option) = state.active_id() {
            reveal(&revealed, &option);
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
                on:input:target=move |event| {
                    state.query.set(event.target().value());
                    state.typing.set(true);
                    state.open.set(true);
                    state.active.set((!state.shown.with_untracked(Vec::is_empty)).then_some(0));
                }
                on:keydown=move |event| state.key(&event)
                on:blur=move |_| state.close()
            />
            <input type="hidden" name=name prop:value=move || selected.get().unwrap_or_default() />
            <ul
                id=list_id
                role="listbox"
                aria-label=label
                class="fui-combobox-list"
                hidden=move || !state.open.get()
            >
                {move || rows(state, no_match.clone())}
            </ul>
        </div>
    }
    .into_any()
}

/// Scrolls the list `list` so its option `option` is in sight.
fn reveal(list: &str, option: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let found = |id: &str| {
            document()
                .get_element_by_id(id)
                .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        };
        let (Some(list), Some(option)) = (found(list), found(option)) else {
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
    let _: (&str, &str) = (list, option);
}

#[cfg(test)]
mod tests {
    use super::ComboOption;

    #[test]
    fn an_option_matches_its_words_its_value_and_its_second_line() {
        let option = ComboOption::new("loinc", "LOINC").detail("http://loinc.org");
        assert!(option.matches("loi"));
        assert!(option.matches("loinc.org"));
        assert!(!option.matches("snomed"));
    }
}
