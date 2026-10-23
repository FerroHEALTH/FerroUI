// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The results pane of a query (E7).
//!
//! Every screen that runs a query shows its answer in one B10 section: the
//! row count in the heading; Table or Chart as a segmented control, and
//! Export CSV and Export JSON, as its actions; the row window in one quiet
//! line ("Rows 1 to 3 of 3, inside the query's own LIMIT 50"); then the
//! table edge to edge, or the base line chart with one series per numeric
//! column over the time column, inset. The switch keeps its state where
//! the table model keeps a listing's
//! ([`crate::table::model::StateSource`]): as links in the URL, or as a
//! plain form the console answers from its session, so it works with no
//! WebAssembly either way. The chart is
//! [`crate::widget::chart::line_chart`], which carries its accessible name
//! and its figures table. No specification governs the pane: our own
//! design.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::icon::Glyph;
use crate::view::button::{Variant, label};
use crate::view::segmented::{SegmentedForm, segmented_form, segmented_links};
use crate::view::surface::{SectionHead, panel_section};

/// The name the switch's form posts the chosen view as, `table` or
/// `chart`, beside `back`.
pub const SHOWN: &str = "shown";

/// How the answer is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shown {
    /// As its table.
    Table,
    /// As a line chart.
    Chart,
}

impl Shown {
    /// The value the switch posts or names for this view.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::Chart => "chart",
        }
    }
}

/// Where the Table or Chart switch keeps its state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Switch {
    /// In the URL: each view is a link.
    Url {
        /// The page showing the table.
        table: String,
        /// The page showing the chart.
        chart: String,
    },
    /// In the console's session: each view is a submit button of one
    /// plain `POST` form, which posts [`SHOWN`] and `back`.
    Session {
        /// The console's route that keeps the choice.
        action: String,
        /// The page to return to.
        back: String,
    },
}

/// What a results pane shows.
#[derive(Debug)]
pub struct ResultsPane {
    /// The section's title, such as "Results".
    pub title: String,
    /// How many rows the answer holds, after the title. With none, the
    /// chart and the exports are unavailable and the pane says why.
    pub count: u64,
    /// Whether the answer has a numeric column the chart can draw. With
    /// none, the switch is left out, the table is shown and the pane says
    /// why there is no chart.
    pub numeric: bool,
    /// How the answer is shown.
    pub shown: Shown,
    /// Where the switch keeps its state.
    pub switch: Switch,
    /// The address of the answer as CSV.
    pub csv: String,
    /// The address of the answer as JSON.
    pub json: String,
    /// The row window in one sentence, such as "Rows 1 to 3 of 3, inside
    /// the query's own LIMIT 50."
    pub window: String,
    /// The answer as [`ResultsPane::shown`] says: a
    /// [`crate::table::view::data_table`] or a
    /// [`crate::widget::chart::line_chart`]; always the table when there
    /// is nothing to chart (no row, or no numeric column).
    pub body: AnyView,
}

/// Returns the Table or Chart switch.
fn switch_view(switch: Switch, shown: Shown) -> AnyView {
    let views = [
        (Shown::Table, tr(Text::ResultsTable)),
        (Shown::Chart, tr(Text::ResultsChart)),
    ];
    match switch {
        Switch::Url { table, chart } => {
            let current = usize::from(shown == Shown::Chart);
            let [(_, table_words), (_, chart_words)] = views;
            segmented_links(
                tr(Text::ResultsShowAs),
                vec![(table, table_words), (chart, chart_words)],
                current,
            )
        }
        Switch::Session { action, back } => segmented_form(SegmentedForm {
            name: SHOWN,
            label: Cow::Owned(tr(Text::ResultsShowAs)),
            prefix: None,
            choices: views
                .into_iter()
                .map(|(view, words)| (view.key().to_owned(), words))
                .collect(),
            current: shown.key().to_owned(),
            action,
            back,
        }),
    }
}

/// Returns an export link `words` to `href`, or the same control disabled,
/// saying "(unavailable)" to assistive technology, when there is nothing to
/// export.
fn export(href: String, words: String, available: bool) -> AnyView {
    if available {
        return view! {
            <a href=href class=Variant::Secondary.small()>
                {label(Some(Glyph::DOWNLOAD), words)}
            </a>
        }
        .into_any();
    }
    view! {
        <span class=Variant::Secondary.small() aria-disabled="true">
            {label(Some(Glyph::DOWNLOAD), words)}
            <span class="fui-sr-only">" " {tr(Text::StepUnavailable)}</span>
        </span>
    }
    .into_any()
}

/// Returns the results pane `pane`, a part of a tab panel
/// ([`crate::view::surface::panel_sections`]).
///
/// An answer with no row offers no chart and its exports disabled, and one
/// with no numeric column offers no chart; the pane says why in a quiet
/// line, never only in a disabled control (`design/COMPONENTS.md`, R4).
#[must_use]
pub fn results_pane(pane: ResultsPane) -> AnyView {
    let ResultsPane {
        title,
        count,
        numeric,
        shown,
        switch,
        csv,
        json,
        window,
        body,
    } = pane;
    let rows = count > 0;
    let chartable = rows && numeric;
    let shown = if chartable { shown } else { Shown::Table };
    let why = match (rows, numeric) {
        (false, _) => Some(tr(Text::ResultsNoRows)),
        (true, false) => Some(tr(Text::ResultsNoChart)),
        (true, true) => None,
    };
    let actions = view! {
        {chartable.then(|| switch_view(switch, shown))}
        {export(csv, tr(Text::ResultsExportCsv), rows)}
        {export(json, tr(Text::ResultsExportJson), rows)}
    }
    .into_any();
    let body = match shown {
        Shown::Table => body,
        Shown::Chart => view! { <div class="fui-panel-body">{body}</div> }.into_any(),
    };
    panel_section(
        Some(SectionHead {
            title,
            count: Some(count),
            actions: Some(actions),
        }),
        view! {
            <p class="fui-results-window">{window}</p>
            {why.map(|why| view! { <p class="fui-results-window">{why}</p> })}
            {body}
        },
    )
}
