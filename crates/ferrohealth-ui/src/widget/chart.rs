// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The line chart with a legend that shows and hides its series.
//!
//! The drawing is [`crate::view::chart::line_svg`]'s, drawn once on the
//! server with every series in groups of its own, and the figures table
//! beside it reads the same for assistive technology. Only the legend is an
//! island ([`crate::island::ChartLegend`]). Before the page hydrates it is a
//! static key, each series' swatch beside its name, so a sighted reader of
//! a page with no WebAssembly keeps the colours' meaning (leptos-ui.md
//! §15.8). Once hydrated each entry is a `<button aria-pressed>`, and a
//! press sets the `hidden` attribute of that series' groups alone, so the
//! browser parses no SVG again; hiding one series recolours none, since
//! each keeps its palette index, and the last series shown cannot be
//! hidden.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, Word, tr};
use crate::view::chart::{
    FiguresWords, LineChart, figures, has_gaps, line_svg_grouped, mark_path, series_look,
};

/// The words a line chart draws itself, resolved on the server in the
/// page's locale, so an island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartWords {
    /// The legend's note on a gap in a line.
    pub no_value: Word,
    /// The legend's name.
    pub series: Word,
    /// The figures table's words.
    pub figures: FiguresWords,
}

impl ChartWords {
    /// The chart's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            no_value: tr(Text::ChartNoValue),
            series: tr(Text::ChartSeries),
            figures: FiguresWords::of_page(),
        }
    }
}

/// One entry of a chart's legend: a series' name and its colour.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegendEntry {
    /// The series' name, as the table shows it.
    pub name: String,
    /// The series' palette index, which pins its colour.
    pub palette: u8,
}

/// The id of the plot of the chart `id`, which the legend shows and hides
/// series in.
fn plot_id(id: &str) -> String {
    format!("{id}-plot")
}

/// How many of `count` series `shown` keeps, a missing flag counting as
/// shown.
fn shown_count(shown: RwSignal<Vec<bool>>, count: usize) -> usize {
    shown.with(|flags| {
        (0..count)
            .filter(|index| flags.get(*index).copied().unwrap_or(true))
            .count()
    })
}

/// Whether `shown` keeps the series at `index`, a missing flag counting as
/// shown.
fn is_shown(shown: RwSignal<Vec<bool>>, index: usize) -> bool {
    shown.with(|flags| flags.get(index).copied().unwrap_or(true))
}

/// Returns the line chart `chart` in `words`: its drawing and its figures
/// table, drawn on the server, with the legend island between them.
#[must_use]
pub fn line_chart(chart: &LineChart, words: ChartWords) -> AnyView {
    let ChartWords {
        no_value,
        series: label,
        figures: figure_words,
    } = words;
    let table = figures(chart, figure_words).into_any();
    let entries = chart
        .series
        .iter()
        .map(|series| LegendEntry {
            name: series.name.clone(),
            palette: series.palette,
        })
        .collect::<Vec<_>>();
    let gap = has_gaps(chart).then_some(no_value);
    let plot = plot_id(chart.id);
    let target = plot.clone();
    let drawn = line_svg_grouped(chart, &[]);
    view! {
        <figure class="fui-chart">
            <div class="fui-chart-plot" id=plot inner_html=drawn></div>
            <crate::island::ChartLegend plot=target label=String::from(label) entries gap />
            {table}
        </figure>
    }
    .into_any()
}

/// Returns a series' swatch: its point mark `shape` in the colour `class`
/// names, as the plot draws it.
fn swatch_of(class: &str, shape: &'static str) -> impl IntoView + use<> {
    view! {
        <svg class=class.to_owned() viewBox="-5 -5 10 10" focusable="false" aria-hidden="true">
            <path d=shape></path>
        </svg>
    }
}

/// Returns the entry of the series `entry` at `index`: its static key
/// before the page hydrates, its toggle of `shown` once it has.
fn entry(
    entry: LegendEntry,
    index: usize,
    count: usize,
    shown: RwSignal<Vec<bool>>,
    hydrated: RwSignal<bool>,
) -> impl IntoView {
    let (tone, mark) = series_look(entry.palette);
    let swatch = format!("fui-swatch fui-series-{tone}");
    let shape = mark_path(mark);
    let on = move || is_shown(shown, index);
    // The last series shown cannot be hidden, so the chart never empties
    // itself; its chip stays pressed and refuses the press.
    let last = move || on() && shown_count(shown, count) == 1;
    let name = entry.name;
    move || {
        if hydrated.get() {
            view! {
                <button
                    type="button"
                    class="fui-legend-chip"
                    data-series=name.clone()
                    aria-pressed=move || on().to_string()
                    disabled=last
                    on:click=move |_| {
                        if last() {
                            return;
                        }
                        shown
                            .update(|flags| {
                                if let Some(flag) = flags.get_mut(index) {
                                    *flag = !*flag;
                                }
                            });
                    }
                >
                    {swatch_of(&swatch, shape)}
                    {name.clone()}
                </button>
            }
            .into_any()
        } else {
            view! {
                <span class="fui-legend-key" data-series=name.clone()>
                    {swatch_of(&swatch, shape)}
                    {name.clone()}
                </span>
            }
            .into_any()
        }
    }
}

/// Returns the legend of the plot `plot`, named `label`, one entry per
/// series in `entries`, with `gap` saying what a break in a line means when
/// the chart has one. A console draws it through [`line_chart`].
#[must_use]
pub fn legend(
    plot: String,
    label: String,
    entries: Vec<LegendEntry>,
    gap: Option<Word>,
) -> AnyView {
    let count = entries.len();
    let shown = RwSignal::new(vec![true; count]);
    let hydrated = RwSignal::new(false);
    Effect::new(move || hydrated.set(true));
    let plot = StoredValue::new(plot);
    Effect::new(move || {
        shown.with(|flags| plot.with_value(|plot| show_in_place(plot, flags, count)));
    });
    let chips = entries
        .into_iter()
        .enumerate()
        .map(|(index, each)| entry(each, index, count, shown, hydrated))
        .collect_view();
    let note = gap.map(|gap| {
        view! {
            <span class="fui-legend-note">
                <span class="fui-swatch-gap" aria-hidden="true"></span>
                {gap}
            </span>
        }
    });
    view! {
        <div class="fui-legend" role="group" aria-label=label>
            {chips}
            {note}
        </div>
    }
    .into_any()
}

/// Hides or shows each of `count` series drawn in the plot `plot` as
/// `flags` says, by the `hidden` attribute of its two groups, leaving the
/// drawing as it is.
fn show_in_place(plot: &str, flags: &[bool], count: usize) {
    #[cfg(target_arch = "wasm32")]
    {
        // NOTE: leptos_macro 0.8 `src/lib.rs`, island limitation 1: the plot is drawn on
        // the server outside the island, out of reach of a node reference, so it is found by id.
        let Some(plot) = document().get_element_by_id(plot) else {
            return;
        };
        for index in 0..count {
            let hide = !flags.get(index).copied().unwrap_or(true);
            for part in ["lines", "marks"] {
                let group = plot.query_selector(&format!("[data-series-{part}=\"{index}\"]"));
                let done = match group {
                    Ok(Some(group)) if hide => group.set_attribute("hidden", ""),
                    Ok(Some(group)) => group.remove_attribute("hidden"),
                    Ok(None) => Ok(()),
                    Err(error) => Err(error),
                };
                if done.is_err() {
                    leptos::logging::warn!("the browser refused to show or hide a chart series");
                }
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (&str, &[bool], usize) = (plot, flags, count);
}
