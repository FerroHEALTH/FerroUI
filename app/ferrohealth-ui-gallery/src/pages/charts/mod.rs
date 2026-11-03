// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The charts page: the charts and the figures drawn as SVG.
//!
//! A line chart of three synthetic series over a week, its legend showing
//! and hiding each, and one of eight lanes of small values over twelve
//! days; the activity line of a dashboard, the ETL run report with
//! refusals, with none and with a tiny part, the flow at rest and with
//! problem edges, a busy bridge of thirty callers and thirty upstreams as
//! a flow whose lanes fold, and the score breakdown, among close
//! thresholds too, and its compact cell. The query's `open` names the busy
//! flow's lanes unfolded (`callers`, `upstreams`).

mod flows;
mod lines;
mod reports;
mod scores;

use ferrohealth_ui::html::Html;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::surface::{Container, page_header};

use crate::locale::Msg;
use crate::page::Page;
use crate::visit::Visit;

use flows::{bridge_flow, busy_flow};
use lines::{lines, many_lines};
use reports::{activity, run_report, tiny_report};
use scores::{breakdown, close_breakdown, score_cells};

/// The charts page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let header = page_header(
        &v.locale,
        v.tr(Msg::Charts),
        Some(v.tr(Msg::DrawnSvgServerScaledCard)),
        ActionGroup::default(),
    );
    let open: Vec<String> = v
        .param("open")
        .unwrap_or_default()
        .split(',')
        .filter(|lane| !lane.is_empty())
        .map(str::to_owned)
        .collect();
    let body = Container::Stack.wrap(Html::join([
        lines(v),
        many_lines(v),
        activity(v),
        run_report(v, true),
        run_report(v, false),
        tiny_report(v),
        bridge_flow(v, false),
        bridge_flow(v, true),
        busy_flow(v, &open),
        breakdown(v, false),
        breakdown(v, true),
        close_breakdown(v),
        score_cells(v),
    ]));
    Page::shell(v.tr(Msg::Charts), Html::join([header, body]))
}
