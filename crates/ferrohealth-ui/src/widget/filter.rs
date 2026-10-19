// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The filter over the rows of the current page.
//!
//! A search box whose words narrow what the page shows, with a live count of
//! the rows kept. The filter names no patient and lives in the page alone; a
//! filter the server applies is a form of the console's own.

use leptos::prelude::*;

use crate::view::form::INPUT;

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
/// `count` the sentence of how many rows it keeps.
#[must_use]
pub fn table_filter(
    id: &'static str,
    label: impl Into<String>,
    words: RwSignal<String>,
    count: Signal<String>,
) -> impl IntoView {
    let label = label.into();
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
            <p role="status" class="fui-muted">
                {move || count.get()}
            </p>
        </div>
    }
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
