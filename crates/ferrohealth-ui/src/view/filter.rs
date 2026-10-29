// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The in-place filter over rows that are already in the page.
//!
//! A search box whose words narrow the rows the page already holds, with a
//! live count of the rows kept. It is only for a short set the server drew
//! whole (a picker's options, a tree's loaded level), never for a listing:
//! a listing's filter is [`crate::table::filter::find_form`], a form the
//! server answers. The `filter` module hides each row of the element the
//! filter names whose text does not hold the words; without the module the
//! filter does nothing, so the rows it narrows read whole without it.
//!
//! The count shows at once, and assistive technology hears it once the
//! typing pauses ([`SETTLE`]): a `role="status"` region of its own takes
//! the count only then, so a word typed letter by letter is not read out
//! letter by letter (WCAG 2.2 SC 4.1.3, WAI-ARIA 1.2 `status`).

use std::time::Duration;

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::Phrase;
use crate::view::form::INPUT;

/// How long the typing pauses before the status region takes the count.
pub const SETTLE: Duration = Duration::from_millis(600);

/// The attribute of a row the filter narrows: the texts it searches, when
/// they are not the row's own text.
pub const ROW_TEXT: &str = "data-fui-filter-text";

/// Whether `row`'s searchable texts hold the filter `words`, ignoring case;
/// an empty filter keeps every row. The `filter` module keeps a row by the
/// same rule.
#[must_use]
pub fn keeps(words: &str, texts: &[&str]) -> bool {
    let words = words.trim().to_lowercase();
    words.is_empty()
        || texts
            .iter()
            .any(|text| text.to_lowercase().contains(&words))
}

/// What a filter box narrows and says.
#[derive(Debug, Clone)]
pub struct TableFilter {
    /// The box's id and name.
    pub id: &'static str,
    /// Its label.
    pub label: String,
    /// The id of the element whose rows it narrows: each child carrying
    /// [`ROW_TEXT`], or each child when none does.
    pub rows: String,
    /// The sentence of how many rows it keeps as the page arrives, such as
    /// "12 of 12 files on this page.".
    pub count: String,
    /// The same sentence with the placeholders `kept` and `total` left open
    /// ([`Phrase::open`]), which the module fills as the rows narrow.
    pub phrase: Phrase,
    /// The words the box holds as the page arrives.
    pub words: String,
}

#[derive(Template)]
#[template(
    source = r#"<div data-fui="filter" data-fui-rows="{{ rows }}" data-fui-phrase="{{ phrase }}" class="fui-table-tools"><div class="fui-field"><label for="{{ id }}" class="fui-label">{{ label }}</label><input id="{{ id }}" name="{{ id }}" type="search" autocomplete="off" value="{{ words }}" class="{{ input }}"></div><p aria-hidden="true" class="fui-muted">{{ count }}</p><p role="status" class="fui-sr-only">{{ count }}</p></div>"#,
    ext = "html"
)]
struct FilterView {
    id: &'static str,
    label: String,
    rows: String,
    phrase: String,
    words: String,
    input: &'static str,
    count: String,
}

/// Returns the filter box `filter`, with the count of the rows it keeps in
/// sight and said to assistive technology once it has held for [`SETTLE`].
#[must_use]
pub fn table_filter(filter: TableFilter) -> Html {
    FilterView {
        id: filter.id,
        label: filter.label,
        rows: filter.rows,
        phrase: filter.phrase.fill(&[]).text().to_owned(),
        words: filter.words,
        input: INPUT,
        count: filter.count,
    }
    .html()
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
