// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The in-place filter over rows that are already in the browser.
//!
//! A search box whose words narrow the rows a widget already holds as its
//! own data, with a live count of the rows kept. It is only for rows that
//! are island data anyway (a picker's options, a tree's loaded level),
//! never for a listing: a listing's filter is
//! [`crate::table::filter::find_form`], a form the server answers, so it
//! works with no WebAssembly and no listing row reaches the page as island
//! props. Without the WebAssembly this filter does nothing, so the rows it
//! narrows must read whole without it.
//!
//! The count shows at once, and assistive technology hears it once the
//! typing pauses ([`SETTLE`]): a `role="status"` region of its own takes
//! the count only then, so a word typed letter by letter is not read out
//! letter by letter (WCAG 2.2 SC 4.1.3, WAI-ARIA 1.2 `status`).

use std::time::Duration;

use leptos::prelude::*;

use crate::view::form::INPUT;

/// How long the typing pauses before the status region takes the count.
pub const SETTLE: Duration = Duration::from_millis(600);

/// Whether `row`'s searchable texts hold the filter `words`, ignoring case;
/// an empty filter keeps every row.
#[must_use]
pub fn keeps(words: &str, texts: &[&str]) -> bool {
    let words = words.trim().to_lowercase();
    words.is_empty()
        || texts
            .iter()
            .any(|text| text.to_lowercase().contains(&words))
}

/// Returns the filter box `id` labelled `label`, bound to `words`, with
/// `count` the sentence of how many rows it keeps, said to assistive
/// technology once the count has held for [`SETTLE`].
#[must_use]
pub fn table_filter(
    id: &'static str,
    label: impl Into<String>,
    words: RwSignal<String>,
    count: Signal<String>,
) -> impl IntoView {
    let label = label.into();
    let said = RwSignal::new(count.get_untracked());
    settle(count, said);
    view! {
        <div class="fui-table-tools">
            <div class="fui-field">
                <label for=id class="fui-label">
                    {label}
                </label>
                <input
                    id=id
                    name=id
                    type="search"
                    class=INPUT
                    autocomplete="off"
                    prop:value=move || words.get()
                    on:input:target=move |event| words.set(event.target().value())
                />
            </div>
            <p class="fui-muted" aria-hidden="true">
                {move || count.get()}
            </p>
            <p role="status" class="fui-sr-only">
                {move || said.get()}
            </p>
        </div>
    }
}

/// Keeps `said` one [`SETTLE`] behind `count`: each change of the count
/// restarts the wait, so only the count the typing paused on is said.
fn settle(count: Signal<String>, said: RwSignal<String>) {
    #[cfg(target_arch = "wasm32")]
    {
        let pending = StoredValue::new(None::<TimeoutHandle>);
        Effect::watch(
            move || count.get(),
            move |now, _, _| {
                if let Some(handle) = pending.get_value() {
                    handle.clear();
                }
                let now = now.clone();
                let handle = set_timeout_with_handle(move || said.set(now), SETTLE);
                if handle.is_err() {
                    leptos::logging::warn!("the browser refused the filter's wait");
                }
                pending.set_value(handle.ok());
            },
            false,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (Signal<String>, RwSignal<String>) = (count, said);
}

#[cfg(test)]
mod tests {
    use super::keeps;

    #[test]
    fn a_filter_matches_any_text_ignoring_case() {
        assert!(keeps("", &["anything"]), "an empty filter keeps all");
        assert!(keeps("  OBS ", &["Observation"]), "case and spaces");
        assert!(keeps("node", &["a", "Node B"]), "any text");
        assert!(!keeps("zz", &["a", "b"]), "no match");
    }
}
