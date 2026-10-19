// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The line chart with a legend that shows and hides its series.
//!
//! The drawing is [`crate::view::chart::line_svg`], drawn again when the
//! legend changes; each chip of the legend is a `<button aria-pressed>`
//! with its series' swatch beside its name, so the colour never carries the
//! meaning alone, and hiding one series recolours none, since each keeps
//! its palette index.

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::view::chart::{LineChart, PALETTE, figures, has_gaps, line_svg};

/// How many of `count` series `shown` keeps, a missing flag counting as
/// shown.
fn shown_count(shown: RwSignal<Vec<bool>>, count: usize) -> usize {
    shown.with(|flags| {
        (0..count)
            .filter(|index| flags.get(*index).copied().unwrap_or(true))
            .count()
    })
}

/// Returns the line chart `chart`, its legend toggling `shown`, one flag
/// per series; the last series shown cannot be hidden.
#[must_use]
pub fn line_chart(chart: LineChart, shown: RwSignal<Vec<bool>>) -> AnyView {
    let table = figures(&chart).into_any();
    let count = chart.series.len();
    let chips = chart
        .series
        .iter()
        .enumerate()
        .map(|(index, series)| {
            let swatch = format!("fui-swatch fui-series-{}", series.palette % PALETTE);
            let on = move || shown.with(|flags| flags.get(index).copied().unwrap_or(true));
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
                {tr(Text::ChartNoValue)}
            </span>
        }
    });
    let drawn = move || shown.with(|flags| line_svg(&chart, flags));
    view! {
        <figure class="fui-chart">
            <div class="fui-chart-plot" inner_html=drawn></div>
            <figcaption class="fui-legend" aria-label=tr(Text::ChartSeries)>
                {chips}
                {note}
            </figcaption>
            {table}
        </figure>
    }
    .into_any()
}
