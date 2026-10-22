// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The top bar's search, in each state the gallery shows.
//!
//! The address's `search` query picks the state, so a screenshot or a
//! journey reaches each one by its link: `busy` while a search runs,
//! `disabled` while the index rebuilds, `hits` with the answer open, and
//! `none` with an answer that found nothing. With no `search` the box is
//! empty and ready.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::view::search::{Hit, HitGroup, Mark, SearchAnswer, SearchBox, Way, search_box};
use leptos::prelude::*;

use crate::data::{SEARCHED, SEARCHED_BUSY, SEARCHED_NONE, TEMPLATE, rows};
use crate::locale::{Msg, tr, tr_with};

/// The id of the top bar's search field.
pub const SEARCH_ID: &str = "gallery-search";

/// Returns the top bar's search of `brand`'s pages in the state `state`
/// names.
#[must_use]
pub fn gallery_search(brand: Brand, state: Option<&str>) -> AnyView {
    let tables = format!("/{}/tables", brand.key());
    let mut search = SearchBox {
        id: String::from(SEARCH_ID),
        label: tr(Msg::SearchMappingFiles),
        placeholder: tr(Msg::FileTemplatePath),
        action: tables.clone(),
        ..SearchBox::default()
    };
    let answer = match state {
        Some("busy") => {
            search.value = String::from(SEARCHED_BUSY);
            search.busy = true;
            None
        }
        Some("disabled") => {
            search.disabled = Some(tr(Msg::SearchIndexRebuilding));
            None
        }
        Some("hits") => {
            search.value = String::from(SEARCHED);
            Some(found(&tables))
        }
        Some("none") => {
            search.value = String::from(SEARCHED_NONE);
            Some(SearchAnswer {
                none: Some((
                    tr_with(Msg::SearchNoFile, &[("query", SEARCHED_NONE.into())]),
                    tr(Msg::SearchNoFileLine),
                )),
                more: Some(Way {
                    href: tables,
                    words: tr(Msg::SearchBrowse),
                }),
                ..SearchAnswer::default()
            })
        }
        _ => None,
    };
    search_box(search, answer)
}

/// The answer for [`SEARCHED`]: three mapping files and a template, with
/// the way to every result in the tables page at `tables`.
fn found(tables: &str) -> SearchAnswer {
    let files = rows()
        .into_iter()
        .take(3)
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
    SearchAnswer {
        head: Some(tr_with(Msg::SearchFoundHead, &[("query", SEARCHED.into())])),
        groups: vec![
            HitGroup {
                kind: tr(Msg::SearchKindFiles),
                hits: files,
            },
            HitGroup {
                kind: tr(Msg::SearchKindTemplates),
                hits: vec![Hit {
                    href: format!("{tables}?q={template}"),
                    name: String::from(template),
                    line: tr(Msg::SearchTemplateLine),
                    mark: Some(Mark::Glyph(Glyph::FILE_TEXT)),
                    tag: Some(String::from(version)),
                }],
            },
        ],
        none: None,
        more: Some(Way {
            href: format!("{tables}?q={SEARCHED}"),
            words: tr(Msg::SearchAllResults),
        }),
    }
}
