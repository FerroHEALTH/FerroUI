// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The combobox: choose one of a long list, typing to filter it.
//!
//! With no script it is a native `<select>`, which a form posts. Once the
//! `combobox` module runs it becomes the WAI-ARIA 1.2 combobox
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
//! concepts, takes a [`Remote`] ([`searched_combobox`]): the typed words
//! post with the console's form under the remote's name, and the page the
//! console answers with holds the options its own search found, which the
//! combobox shows unfiltered; the library fetches nothing.

use std::borrow::Cow;

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr};
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
    /// Whether choosing it asks for a typed value instead, as the last
    /// option of a pick-or-type field ([`crate::view::pick_or_type`]) does.
    #[serde(default)]
    pub other: bool,
}

impl ComboOption {
    /// The option `value` shown as `label`, with no second line.
    #[must_use]
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            detail: None,
            other: false,
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

    /// This option, marked as the one that asks for a typed value.
    #[must_use]
    pub fn other(self) -> Self {
        Self {
            other: true,
            ..self
        }
    }

    /// Whether the option matches the typed `needle`, lower-cased; the
    /// `combobox` module keeps an option by the same rule.
    #[must_use]
    pub fn matches(&self, needle: &str) -> bool {
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

/// The words a combobox draws itself, in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComboboxWords {
    /// The line the list shows when no option matches.
    pub no_match: Word,
    /// The line under the list when more options match than it draws.
    pub more: Word,
}

impl ComboboxWords {
    /// The combobox's words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        Self {
            no_match: tr(locale, Text::ComboboxNoMatch),
            more: tr(locale, Text::ComboboxMore),
        }
    }
}

/// A console's own search behind a combobox, for a list too long to send to
/// the browser whole.
///
/// The typed words post with the console's form as `param`; the console
/// answers with the page whose combobox holds the options its search found
/// for them, which the combobox shows as they stand, filtering none itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    /// The name the typed words post as, such as `q`.
    pub param: String,
    /// The words the options were found for, which the input holds as the
    /// page arrives.
    pub query: String,
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
    pub selected: Option<String>,
    /// The words it draws itself ([`ComboboxWords::of`]).
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
    pub selected: Option<String>,
    /// The id of the element that describes the control, such as its help
    /// line, which the input names in `aria-describedby`.
    pub described_by: Option<String>,
    /// The words it draws itself ([`ComboboxWords::of`]).
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
pub fn combobox(combo: Combobox) -> Html {
    keyed_combobox(KeyedCombobox::from(combo))
}

/// Returns the combobox `combo`, whose ids are made at run time.
#[must_use]
pub fn keyed_combobox(combo: KeyedCombobox) -> Html {
    drawn(combo, None)
}

/// Returns the combobox `combo` over the console's search `remote`.
///
/// Its `options` are what the console's search found for `remote.query`,
/// the chosen one among them.
#[must_use]
pub fn searched_combobox(combo: KeyedCombobox, remote: Remote) -> Html {
    drawn(combo, Some(remote))
}

#[derive(Template)]
#[template(path = "combobox/combobox.html")]
struct ComboboxView {
    id: String,
    name: String,
    label: String,
    placeholder: String,
    options: Vec<ComboOption>,
    selected: Option<String>,
    described_by: Option<String>,
    select: &'static str,
    no_match: String,
    more: String,
    remote: Option<String>,
    query: String,
    check: Html,
}

/// Returns the combobox `combo`, over `remote` when it has one: the native
/// select with its options, which the `combobox` module turns into the
/// combobox.
fn drawn(combo: KeyedCombobox, remote: Option<Remote>) -> Html {
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
    let (remote, query) = remote.map_or((None, String::new()), |remote| {
        (Some(remote.param), remote.query)
    });
    ComboboxView {
        id,
        name,
        label,
        placeholder,
        options,
        selected,
        described_by,
        select: SELECT,
        no_match: no_match.into(),
        more: more.into(),
        remote,
        query,
        check: icon(Glyph::CHECK, "16"),
    }
    .html()
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
