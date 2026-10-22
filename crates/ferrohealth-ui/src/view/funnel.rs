// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The ETL run report (B7): where the inputs went, as one stacked bar with
//! a legend of big figures, and the rows per table as a bar list with the
//! numbers right-aligned.
//!
//! The bars are SVG drawn here, each length an attribute, so no style
//! attribute is written. The stacked bar is an image named by a sentence
//! the caller writes, and the legend prints every part's figure; a row of
//! the bar list prints its number beside its bar, which repeats it and is
//! hidden from assistive technology. Nothing rests on colour alone (WCAG
//! 2.2 SC 1.1.1, SC 1.4.1). No specification governs the drawing: our own
//! design.

use leptos::prelude::*;

use crate::tone::Tone;

/// One part of a stacked bar, such as the inputs written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunnelPart {
    /// Its words in the legend, such as "Written".
    pub label: String,
    /// Its count, which sets its share of the bar.
    pub count: u64,
    /// The count as the page writes numbers, such as "1,180".
    pub figure: String,
    /// Its tone: the good tone for written, neutral for skipped, the bad
    /// tone for refused.
    pub tone: Tone,
}

/// What a stacked bar shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Funnel {
    /// The total as the page writes numbers, such as "1,240".
    pub figure: String,
    /// The words after the total, such as "compositions read".
    pub words: String,
    /// The bar's accessible name: a sentence that reads every part, such as
    /// "Of 1,240 compositions read: 1,180 written, 52 skipped, 8 refused".
    pub label: String,
    /// The parts, in the order they are drawn.
    pub parts: Vec<FunnelPart>,
}

/// One row of a bar list, such as a table and the rows written to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bar {
    /// Its name, such as a table's name.
    pub name: String,
    /// Its count, which sets its bar's length against the largest.
    pub count: u64,
    /// The count as the page writes numbers.
    pub figure: String,
}

/// The tone class of a part of the stacked bar and its swatch.
fn tone_class(base: &str, tone: Tone) -> String {
    format!("{base} {}", tone.class())
}

/// `count` as a percentage of `whole`.
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "a count past 2^53 rounds to the nearest double, a difference no bar's length can show"
)]
fn share(count: u64, whole: u64) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    count as f64 / whole as f64 * 100.0
}

/// The least share of the stacked bar a part with any count takes, in per
/// cent, so three refusals among two million still show.
const LEAST: f64 = 1.5;

/// The widths of parts of `counts`, in per cent of the bar: each its share,
/// a part with any count at least [`LEAST`], the room that takes coming off
/// the widest part.
fn widths(counts: &[u64]) -> Vec<f64> {
    let whole = counts.iter().sum::<u64>();
    let mut widths: Vec<f64> = counts.iter().map(|count| share(*count, whole)).collect();
    let mut owed = 0.0;
    for (width, count) in widths.iter_mut().zip(counts) {
        if *count > 0 && *width < LEAST {
            owed += LEAST - *width;
            *width = LEAST;
        }
    }
    if let Some(widest) = widths.iter_mut().max_by(|a, b| a.total_cmp(b)) {
        *widest = (*widest - owed).max(LEAST);
    }
    widths
}

/// Returns the stacked bar of `funnel`: the total, the bar of its parts
/// split by a hairline, and the legend with each part's swatch, words and
/// figure.
///
/// A part with a count of zero draws no segment and keeps its legend entry,
/// so a run with nothing refused still says "0"; a part with any count
/// takes at least 1.5 % of the bar and a stroke of its colour around it,
/// so it never vanishes.
#[must_use]
pub fn funnel(funnel: &Funnel) -> AnyView {
    let counts: Vec<u64> = funnel.parts.iter().map(|part| part.count).collect();
    let whole = counts.iter().sum::<u64>();
    let mut start = 0.0;
    let mut segments = Vec::new();
    for (part, width) in funnel.parts.iter().zip(widths(&counts)) {
        if width > 0.0 {
            segments.push((start, width, part, share(part.count, whole) < LEAST));
        }
        start += width;
    }
    let drawn = segments
        .iter()
        .enumerate()
        .map(|(index, (start, width, part, least))| {
            let base = if *least {
                "fui-funnel-part fui-funnel-least"
            } else {
                "fui-funnel-part"
            };
            let class = tone_class(base, part.tone);
            let x = format!("{start:.2}%");
            let cut = (index > 0).then(|| {
                let at = x.clone();
                view! {
                    <line class="fui-funnel-cut" x1=at.clone() x2=at y1="0" y2="100%"></line>
                }
            });
            let width = format!("{width:.2}%");
            let tip = format!("{}: {}", part.label, part.figure);
            view! {
                <rect class=class x=x y="0" width=width height="100%">
                    <title>{tip}</title>
                </rect>
                {cut}
            }
        })
        .collect_view();
    let legend = funnel
        .parts
        .iter()
        .map(|part| {
            let swatch = tone_class("fui-funnel-swatch", part.tone);
            view! {
                <div>
                    <dt>
                        <span class=swatch aria-hidden="true"></span>
                        {part.label.clone()}
                    </dt>
                    <dd>{part.figure.clone()}</dd>
                </div>
            }
        })
        .collect_view();
    view! {
        <div class="fui-funnel">
            <p>
                <strong class="fui-num">{funnel.figure.clone()}</strong>
                " "
                {funnel.words.clone()}
            </p>
            <svg
                class="fui-funnel-bar"
                width="100%"
                height="12"
                focusable="false"
                role="img"
                aria-label=funnel.label.clone()
            >
                {drawn}
            </svg>
            <dl class="fui-funnel-legend">{legend}</dl>
        </div>
    }
    .into_any()
}

/// Returns the bar list of `bars`, named `label`: per row the name, a bar
/// whose length is its count against the largest, and the number
/// right-aligned.
///
/// On a phone the bar takes its own line under the name.
#[must_use]
pub fn bar_list(label: &str, bars: &[Bar]) -> AnyView {
    let largest = bars.iter().map(|bar| bar.count).max().unwrap_or(0);
    let rows = bars
        .iter()
        .map(|bar| {
            let width = format!("{:.1}%", share(bar.count, largest));
            view! {
                <li>
                    <span class="fui-barlist-name">{bar.name.clone()}</span>
                    <svg
                        class="fui-barlist-track"
                        width="100%"
                        height="8"
                        focusable="false"
                        aria-hidden="true"
                    >
                        <rect class="fui-barlist-fill" width=width height="8" rx="4"></rect>
                    </svg>
                    <span class="fui-barlist-num fui-num">{bar.figure.clone()}</span>
                </li>
            }
        })
        .collect_view();
    view! { <ul class="fui-barlist" aria-label=label.to_owned()>{rows}</ul> }.into_any()
}

#[cfg(test)]
mod tests {
    use leptos::prelude::*;

    use super::{Funnel, FunnelPart, LEAST, funnel, share, widths};
    use crate::tone::Tone;

    #[test]
    fn a_share_of_nothing_is_zero_and_a_whole_is_a_hundred() {
        assert!(share(3, 0).abs() < f64::EPSILON);
        assert!((share(8, 8) - 100.0).abs() < f64::EPSILON);
        assert!((share(1, 4) - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn a_tiny_part_keeps_a_visible_width_taken_from_the_widest() {
        let drawn = widths(&[2_000_000, 0, 3]);
        let near =
            |at: usize, width: f64| drawn.get(at).is_some_and(|got| (got - width).abs() < 1e-9);
        assert!(near(2, LEAST), "three in two million shows: {drawn:?}");
        assert!(near(1, 0.0), "nothing draws nothing: {drawn:?}");
        let total: f64 = drawn.iter().sum();
        assert!(
            (total - 100.0).abs() < 1e-9,
            "the parts still fill the bar: {drawn:?}"
        );
    }

    #[test]
    fn three_refusals_in_two_million_draw_a_part_that_shows() {
        let part = |label: &str, count: u64, tone: Tone| FunnelPart {
            label: label.to_owned(),
            count,
            figure: count.to_string(),
            tone,
        };
        let shown = Owner::new().with(|| {
            funnel(&Funnel {
                figure: String::from("2,000,003"),
                words: String::from("read"),
                label: String::from("Of 2,000,003 read: 3 refused"),
                parts: vec![
                    part("Written", 2_000_000, Tone::Ok),
                    part("Refused", 3, Tone::Bad),
                ],
            })
            .to_html()
        });
        assert!(
            shown.contains(r#"<rect x="98.50%" y="0" width="1.50%" height="100%" class="fui-funnel-part fui-funnel-least fui-tone-bad">"#),
            "{shown}"
        );
    }
}
