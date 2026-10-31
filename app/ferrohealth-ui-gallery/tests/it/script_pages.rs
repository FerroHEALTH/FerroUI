// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The jobs each gallery page asks for, held against
//! `scripts/checks/script-pages.list`, over which `script-budget.sh`
//! measures the modules any one page loads (FerroUI #547).
//!
//! The test walks the gallery in-process from `/` by every same-origin link
//! a page draws and collects the `data-fui` jobs of each page, so the budget
//! never measures pages the gallery no longer draws. No specification
//! governs this: our own design.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;

use axum::body::Body;
use ferrohealth_ui::style::Brand;
use http::{Request, StatusCode, header};
use tower::ServiceExt as _;

/// The committed page list.
const LIST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scripts/checks/script-pages.list"
);

/// Path prefixes the walk never asks for: the assets, the favicon and the
/// export, which are no pages, and the session routes, which change state.
const SKIPPED: [&str; 4] = ["/ferrohealth-ui/", "/favicon", "/export", "/session"];

/// The values of every `name="…"` attribute in `html`, in order.
fn attributes<'a>(html: &'a str, name: &str) -> Vec<&'a str> {
    let marker = format!(" {name}=\"");
    let mut values = Vec::new();
    let mut rest = html;
    while let Some((_, after)) = rest.split_once(marker.as_str()) {
        let Some((value, next)) = after.split_once('"') else {
            break;
        };
        values.push(value);
        rest = next;
    }
    values
}

/// The path `href` names, without its query and fragment, when it is a
/// same-origin page the walk asks for.
fn page_path(href: &str) -> Option<&str> {
    let path = href.split(['?', '#']).next()?;
    let walked = path.starts_with('/')
        && !path.starts_with("//")
        && !SKIPPED.iter().any(|skipped| path.starts_with(skipped));
    walked.then_some(path)
}

/// `path` with its brand segment written as `:brand`, so the five brands of
/// one page share a line.
fn generic(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            if Brand::ALL.iter().any(|brand| brand.key() == segment) {
                ":brand"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Every page the gallery draws from `/` as a list line: its generic path
/// and the jobs it asks for, in order.
async fn drawn() -> Result<BTreeSet<String>, Box<dyn Error>> {
    let gallery = ferrohealth_ui_gallery::server::router();
    let mut seen = BTreeSet::from(["/".to_owned()]);
    let mut queue = VecDeque::from(["/".to_owned()]);
    let mut pages: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    while let Some(path) = queue.pop_front() {
        let response = gallery
            .clone()
            .oneshot(Request::get(&path).body(Body::empty())?)
            .await?;
        let html = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|kind| kind.to_str().ok())
            .is_some_and(|kind| kind.starts_with("text/html"));
        if response.status() != StatusCode::OK || !html {
            continue;
        }
        let body = axum::body::to_bytes(response.into_body(), 8 * 1024 * 1024).await?;
        let body = String::from_utf8(body.to_vec())?;
        let jobs = pages.entry(generic(&path)).or_default();
        for value in attributes(&body, "data-fui") {
            jobs.extend(value.split_whitespace().map(str::to_owned));
        }
        for href in attributes(&body, "href") {
            if let Some(next) = page_path(href)
                && seen.insert(next.to_owned())
            {
                queue.push_back(next.to_owned());
            }
        }
    }
    Ok(pages
        .into_iter()
        .map(|(path, jobs)| {
            std::iter::once(path)
                .chain(jobs)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect())
}

#[tokio::test]
async fn the_page_list_the_script_budget_measures_is_what_the_gallery_draws()
-> Result<(), Box<dyn Error>> {
    let drawn = drawn().await?;
    assert!(drawn.len() > 1, "the walk found no page beyond /");
    let list = std::fs::read_to_string(LIST)?;
    let committed: BTreeSet<String> = list
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect();
    assert!(
        committed == drawn,
        "scripts/checks/script-pages.list is not what the gallery draws; its pages are:\n{}",
        drawn.iter().cloned().collect::<Vec<_>>().join("\n")
    );
    Ok(())
}
