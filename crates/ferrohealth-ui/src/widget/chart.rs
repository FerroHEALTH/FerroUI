// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The line chart with a legend that shows and hides its series.
//!
//! The drawing is [`crate::view::chart::line_svg`]'s, drawn once with every
//! series in groups of its own, and a press of the legend sets the `hidden`
//! attribute of that series' groups alone, so the browser parses no SVG
//! again; each chip of the legend is a `<button aria-pressed>` with its series'
//! swatch beside its name, so the colour never carries the meaning alone,
//! and hiding one series recolours none, since each keeps its palette
//! index. The chips stay hidden until the page hydrates, so none offers a
//! press that does nothing; the figures table reads the same either way.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, tr};
use crate::view::chart::{FiguresWords, LineChart, PALETTE, figures, has_gaps, line_svg_grouped};

/// The words a line chart draws itself, resolved on the server in the
/// page's locale, so an island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartWords {
    /// The legend's note on a gap in a line.
    pub no_value: String,
    /// The legend's name.
    pub series: String,
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

/// Returns the line chart `chart` in `words`, its legend toggling `shown`,
/// one flag per series; the last series shown cannot be hidden.
#[must_use]
pub fn line_chart(chart: LineChart, shown: RwSignal<Vec<bool>>, words: ChartWords) -> AnyView {
    let ChartWords {
        no_value,
        series: legend,
        figures: figure_words,
    } = words;
    let table = figures(&chart, figure_words).into_any();
    let count = chart.series.len();
    let hydrated = RwSignal::new(false);
    Effect::new(move || hydrated.set(true));
    let chips = chart
        .series
        .iter()
        .enumerate()
        .map(|(index, series)| {
            let swatch = format!("fui-swatch fui-series-{}", series.palette % PALETTE);
            let on = move || is_shown(shown, index);
            // The last series shown cannot be hidden, so the chart never
            // empties itself; its chip stays pressed and refuses the press.
            let last = move || on() && shown_count(shown, count) == 1;
            view! {
                <button
                    type="button"
                    class="fui-legend-chip"
                    data-series=series.name.clone()
                    aria-pressed=move || on().to_string()
                    disabled=last
                    hidden=move || !hydrated.get()
                    on:click=move |_| {
                        if last() {
                            return;
                        }
                        shown
                            .update(|flags| {
                                if flags.len() < count {
                                    flags.resize(count, true);
                                }
                                if let Some(flag) = flags.get_mut(index) {
                                    *flag = !*flag;
                                }
                            });
                    }
                >
                    <span class=swatch aria-hidden="true"></span>
                    {series.name.clone()}
                </button>
            }
        })
        .collect_view();
    let note = has_gaps(&chart).then(|| {
        view! {
            <span class="fui-legend-note">
                <span class="fui-swatch-gap" aria-hidden="true"></span>
                {no_value}
            </span>
        }
    });
    let plot = NodeRef::<leptos::html::Div>::new();
    let drawn = shown.with_untracked(move |flags| line_svg_grouped(&chart, flags));
    Effect::new(move || shown.with(|flags| show_in_place(plot, flags, count)));
    view! {
        <figure class="fui-chart">
            <div class="fui-chart-plot" node_ref=plot inner_html=drawn></div>
            <figcaption class="fui-legend" aria-label=legend>
                {chips}
                {note}
            </figcaption>
            {table}
        </figure>
    }
    .into_any()
}

/// Hides or shows each of `count` series drawn in `plot` as `flags` says,
/// by the `hidden` attribute of its two groups, leaving the drawing as it is.
fn show_in_place(plot: NodeRef<leptos::html::Div>, flags: &[bool], count: usize) {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(plot) = plot.get() else {
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
    let _: (NodeRef<leptos::html::Div>, &[bool], usize) = (plot, flags, count);
}
