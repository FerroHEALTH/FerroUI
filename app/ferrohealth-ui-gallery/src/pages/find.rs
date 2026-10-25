// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The find page, as a console draws it: the top bar's search submits here,
//! and the page lists every mapping file and template that holds the words,
//! one list card per kind, in the page.
//!
//! It is what a search shows with no WebAssembly, and where the typeahead's
//! "See all results" leads. With no query it answers the gallery's sample
//! query, so its screenshot shows results.

use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::search::find_results;
use ferrohealth_ui::view::surface::{Crumb, Header, page_heading};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::data::SEARCHED;
use crate::locale::{Msg, tr, tr_with};

/// The find page.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn FindPage() -> impl IntoView {
    let brand = crate::app::route_brand().unwrap_or(ferrohealth_ui::style::Brand::Bridge);
    let query = use_query_map();
    let typed = query.with_untracked(|query| query.get("q"));
    let typed = typed
        .filter(|typed| !typed.trim().is_empty())
        .unwrap_or_else(|| String::from(SEARCHED));
    let root = format!("/{}", brand.key());
    let header = page_heading(
        Header {
            title: tr(Msg::FindTitle).into(),
            subtitle: Some(tr_with(Msg::FindSubtitle, &[("query", typed.as_str().into())]).into()),
            crumbs: vec![Crumb::new(tr(Msg::Gallery), root)],
            ..Header::default()
        },
        ActionGroup::none(),
    )
    .into_any();
    let results = find_results(crate::search::answer(brand, &typed));
    view! {
        {crate::pages::title(tr(Msg::FindTitle))}
        {header}
        {results}
    }
}
