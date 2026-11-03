// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The flow on a phone: the cards as a list, lane by lane.

use crate::html::Html;
use crate::i18n::{Locale, Text, Word, tr_with};
use crate::icon::{Glyph, icon};

use super::FlowState;
use super::draw::toggle_words;
use super::plan::{Plan, Shown};

/// One row of the phone's list: a link with its state as a dot and a
/// word, its name and its line.
fn list_row(locale: &Locale, href: &str, state: FlowState, name: &Word, line: &str) -> Html {
    let muted = if state == FlowState::Off {
        " is-muted"
    } else {
        ""
    };
    let word = Html::text(state.word(locale).text());
    Html::raw(format!(
        "<li><a href=\"{}\" class=\"fui-row-link\"><span role=\"img\" aria-label=\"{word}\" title=\"{word}\" class=\"fui-dot {}{muted}\"></span><span class=\"fui-pipeline-name\">{name}</span><span class=\"fui-pipeline-line\">{}</span></a></li>",
        Html::text(href),
        state.tone().class(),
        Html::text(line)
    ))
}

/// The phone's form of `plan`: a heading per lane and a row per card, each
/// a link to its page with its state as a dot and a word, folded as the
/// drawing is, and an open lane's fold link last.
pub(super) fn list(locale: &Locale, plan: &Plan<'_>, label: &str) -> Html {
    let lanes: Html = plan
        .lanes
        .iter()
        .map(|lane| {
            let rows: Html = lane
                .shown
                .iter()
                .map(|shown| match shown {
                    Shown::Card(card) => list_row(
                        locale,
                        &card.href,
                        card.state,
                        &Word::new(card.name.clone()),
                        &card.line,
                    ),
                    Shown::Fold(folded) => list_row(
                        locale,
                        folded.href,
                        folded.state(),
                        &tr_with(
                            locale,
                            Text::FlowFoldName,
                            &[("count", folded.count().into())],
                        ),
                        &folded.line(),
                    ),
                })
                .collect();
            let fold = lane
                .toggle
                .filter(|(folded, _)| !folded)
                .map_or_else(Html::empty, |(_, href)| {
                    Html::raw(format!(
                        "<li><a href=\"{}\" class=\"fui-row-link\">{}<span class=\"fui-pipeline-name\">{}</span></a></li>",
                        Html::text(href),
                        icon(Glyph::CHEVRON_UP, "16"),
                        Html::text(toggle_words(locale, false, lane.title))
                    ))
                });
            Html::raw(format!(
                "<li><p class=\"fui-pipeline-lane\">{}</p><ul>{rows} {fold}</ul></li>",
                Html::text(lane.title)
            ))
        })
        .collect();
    Html::raw(format!(
        "<ul aria-label=\"{}\" class=\"fui-pipeline-list\">{lanes}</ul>",
        Html::text(label)
    ))
}
