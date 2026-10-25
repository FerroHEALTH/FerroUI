// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The top bar's search, in each state the gallery shows, and the answer
//! its typeahead and its find page draw.
//!
//! The address's `search` query picks the state, so a screenshot or a
//! journey reaches each one by its link: `busy` while a search runs and
//! `disabled` while the index rebuilds. With no `search` the box is ready,
//! holding the words the find page answers. The typeahead asks the
//! gallery's answer route ([`SUGGEST_ROUTE`]), which finds the mapping files
//! and the template whose names hold the words.

use ferrohealth_ui::i18n::Phrase;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::view::search::{Hit, HitGroup, Mark, SearchAnswer, SearchBox, Way, search_box};
use leptos::prelude::*;

use crate::data::{SEARCHED_BUSY, TEMPLATE, rows};
use crate::locale::{Msg, tr, tr_with};

/// The id of the top bar's search field.
pub const SEARCH_ID: &str = "gallery-search";

/// The path under which a brand's answer route answers, as
/// `/suggest/ferrobridge?q=…`.
pub const SUGGEST_ROUTE: &str = "/suggest";

/// Words the answer route answers slowly, so a journey sees the row the
/// panel shows while it waits.
pub const SLOW: &str = "zz-slow";

/// Words the answer route refuses, so a journey sees the panel's error row.
pub const FAILING: &str = "zz-fail";

/// Words the answer route answers `401` to, as for a session that is over,
/// so a journey sees the page load the brand's expired sign-in.
pub const SIGNED_OUT: &str = "zz-signed-out";

/// The path of `brand`'s find page.
#[must_use]
pub fn find_path(brand: Brand) -> String {
    format!("/{}/find", brand.key())
}

/// Returns the top bar's search of `brand`'s pages in the state `state`
/// names, holding the words `typed`.
#[must_use]
pub fn gallery_search(brand: Brand, state: Option<&str>, typed: Option<&str>) -> AnyView {
    let mut search = SearchBox {
        id: String::from(SEARCH_ID),
        label: tr(Msg::SearchMappingFiles),
        placeholder: tr(Msg::FileTemplatePath),
        action: find_path(brand),
        value: typed.map(str::to_owned).unwrap_or_default(),
        suggest: Some(format!("{SUGGEST_ROUTE}/{}", brand.key())),
        signed_out: Some(format!("/sign-in/{}/expired", brand.key())),
        all: (brand == Brand::Pix).then(|| {
            Phrase::new(tr_with(
                Msg::SearchPatientsFor,
                &[("query", Phrase::open("query"))],
            ))
        }),
        ..SearchBox::default()
    };
    match state {
        Some("busy") => {
            search.value = String::from(SEARCHED_BUSY);
            search.busy = true;
        }
        Some("disabled") => {
            search.disabled = Some(tr(Msg::SearchIndexRebuilding));
        }
        _ => {}
    }
    search_box(search)
}

/// What `brand`'s gallery holds under the words `query`: the mapping files
/// and the template whose names hold them, without regard to case, with the
/// way to every file when nothing does.
#[must_use]
pub fn answer(brand: Brand, query: &str) -> SearchAnswer {
    let tables = format!("/{}/tables", brand.key());
    let needle = query.trim().to_lowercase();
    let files: Vec<Hit> = rows()
        .into_iter()
        .filter(|row| !needle.is_empty() && row.name.to_lowercase().contains(&needle))
        .map(|row| Hit {
            href: format!("{tables}?q={}", row.name),
            line: tr_with(
                Msg::SearchFileLine,
                &[("kind", row.kind.into()), ("lines", row.lines.into())],
            ),
            name: row.name,
            mark: Some(Mark::Glyph(Glyph::FILE_CODE)),
            tag: None,
        })
        .collect();
    let (template, version) = TEMPLATE;
    let templates: Vec<Hit> = (!needle.is_empty() && template.contains(&needle))
        .then(|| Hit {
            href: format!("{tables}?q={template}"),
            name: String::from(template),
            line: tr(Msg::SearchTemplateLine),
            mark: Some(Mark::Glyph(Glyph::FILE_TEXT)),
            tag: Some(String::from(version)),
        })
        .into_iter()
        .collect();
    let groups: Vec<HitGroup> = [
        (tr(Msg::SearchKindFiles), files),
        (tr(Msg::SearchKindTemplates), templates),
    ]
    .into_iter()
    .filter(|(_, hits)| !hits.is_empty())
    .map(|(kind, hits)| HitGroup { kind, hits })
    .collect();
    let found = !groups.is_empty();
    SearchAnswer {
        query: query.trim().to_owned(),
        head: None,
        groups,
        none: (!found).then(|| {
            (
                tr_with(Msg::SearchNoFile, &[("query", query.trim().into())]),
                tr(Msg::SearchNoFileLine),
            )
        }),
        unfinished: None,
        more: (!found).then(|| Way {
            href: tables,
            words: tr(Msg::SearchBrowse),
        }),
    }
}

#[cfg(test)]
mod tests {
    use ferrohealth_ui::style::Brand;

    use super::answer;

    #[test]
    fn the_answer_holds_the_names_that_hold_the_words() {
        let found = answer(Brand::Bridge, " SYNTHETIC-00 ");
        assert_eq!("SYNTHETIC-00", found.query);
        assert_eq!(9, found.total(), "synthetic-001 to synthetic-009");
        let template = answer(Brand::Bridge, "vital");
        assert_eq!(1, template.total());
        let none = answer(Brand::Bridge, "zz-unknown");
        assert!(!none.found());
        assert!(none.none.is_some() && none.more.is_some(), "{none:?}");
    }
}
