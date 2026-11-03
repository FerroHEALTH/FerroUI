// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The legend of a chart: a swatch and a name per series.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::Word;

use super::{mark_path, series_look};

/// One entry of a chart's legend: a series' name and its colour.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegendEntry {
    /// The series' name, as the table shows it.
    pub name: String,
    /// The series' palette index, which pins its colour.
    pub palette: u8,
}

/// Returns a series' swatch: its point mark `shape` in the colour `class`
/// names, as the plot draws it.
fn swatch_of(class: &str, shape: &str) -> Html {
    Html::raw(format!(
        "<svg viewBox=\"-5 -5 10 10\" focusable=\"false\" aria-hidden=\"true\" class=\"{class}\"><path d=\"{shape}\"></path></svg>"
    ))
}

#[derive(Template)]
#[template(path = "chart/chart.html", block = "legend")]
struct LegendView<'a> {
    label: String,
    plot: &'a str,
    entries: Vec<(String, Html)>,
    gap: Option<Word>,
}

/// Returns the legend of the plot `plot`, named `label`, one entry per
/// series in `entries`.
///
/// `gap` says what a break in a line means when the chart has one. It is
/// a static key the `chart` module turns into toggles; a console draws it
/// through [`line_chart`](super::line_chart).
#[must_use]
pub fn legend(plot: &str, label: String, entries: Vec<LegendEntry>, gap: Option<Word>) -> Html {
    LegendView {
        label,
        plot,
        entries: entries
            .into_iter()
            .map(|entry| {
                let (tone, mark) = series_look(entry.palette);
                let swatch = swatch_of(&format!("fui-swatch fui-series-{tone}"), mark_path(mark));
                (entry.name, swatch)
            })
            .collect(),
        gap,
    }
    .html()
}
