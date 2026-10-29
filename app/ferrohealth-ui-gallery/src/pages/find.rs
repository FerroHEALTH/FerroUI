// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The find page, as a console draws it: the top bar's search submits here,
//! and the page lists every mapping file and template that holds the words,
//! one list card per kind, in the page.
//!
//! It is what a search shows with no script, and where the typeahead's
//! "See all results" leads. With no query it answers the gallery's sample
//! query, so its screenshot shows results.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::search::find_results;
use ferrohealth_ui::view::surface::{Crumb, Header, page_heading};

use crate::app::Page;
use crate::data::SEARCHED;
use crate::locale::Msg;
use crate::visit::Visit;

/// The find page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let brand = v.brand;
    let typed = v
        .param("q")
        .filter(|typed| !typed.trim().is_empty())
        .unwrap_or(SEARCHED)
        .to_owned();
    let root = format!("/{}", brand.key());
    let header = page_heading(
        &v.locale,
        Header {
            title: v.tr(Msg::FindTitle).into(),
            subtitle: Some(
                v.tr_with(Msg::FindSubtitle, &[("query", typed.as_str().into())])
                    .into(),
            ),
            crumbs: vec![Crumb::new(v.tr(Msg::Gallery), root)],
            ..Header::default()
        },
        ActionGroup::none(),
    );
    let results = find_results(&v.locale, &crate::search::answer(&v.locale, brand, &typed));
    Page::shell(v.tr(Msg::FindTitle), Html::join([header, results]))
}
