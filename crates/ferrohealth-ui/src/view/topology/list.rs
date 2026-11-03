// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The topology as a list: the lanes and their items, row by row.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::flow::FlowState;

use super::tools::state_word;
use super::{RowAction, Topology, TopologyCard, TopologyGroup};

/// One row of the List twin.
#[derive(Debug)]
struct Row<'a> {
    level: u8,
    card: &'a TopologyCard,
    group: Option<&'a TopologyGroup>,
    detail: Option<String>,
    posinset: usize,
    setsize: usize,
}

/// The lane row of the List: a side's title and its quiet line, a heading
/// for the rows under it and no item of the tree.
fn lane_row(title: &str, line: &str) -> String {
    format!(
        "<li role=\"presentation\" class=\"fui-tree-lane\"><span>{}</span><span class=\"fui-small fui-muted\">{}</span></li>",
        Html::text(title),
        Html::text(line)
    )
}

#[derive(Template)]
#[template(
    source = r#"<li role="treeitem"{% if let Some(expanded) = expanded %} aria-expanded="{{ expanded }}"{% endif %} aria-level="{{ level }}" aria-posinset="{{ posinset }}" aria-setsize="{{ setsize }}" class="{{ class }}{% if muted %} is-excluded{% endif %}"><span class="fui-tree-name">{% if let Some(fold) = fold %}{{ fold }}{% endif %}{{ glyph }}<a href="{{ href }}">{{ name }}</a></span><span>{{ state }}</span><span class="fui-small fui-muted">{% if let Some(measure) = measure %}{{ measure }}{% endif %}</span><span class="fui-small fui-muted">{% if let Some(note) = note %}{{ note }}{% endif %}</span><span class="fui-tree-act">{% if let Some(action) = action %}<form method="post" action="{{ action.action }}"><input type="hidden" name="{{ action.name }}" value="{{ action.value }}"><button type="submit" class="{{ ghost }}">{{ action_glyph }}{{ action.label }}</button></form>{% endif %}</span></li>"#,
    ext = "html"
)]
struct ItemView<'a> {
    expanded: Option<bool>,
    level: u8,
    posinset: usize,
    setsize: usize,
    class: &'static str,
    muted: bool,
    fold: Option<Html>,
    glyph: Html,
    href: &'a str,
    name: Html,
    state: Html,
    measure: Option<&'a str>,
    note: Option<&'a str>,
    action: Option<&'a RowAction>,
    action_glyph: Html,
    ghost: &'static str,
}

/// One item row of the List: the name with its glyph as a link to its
/// page, a group's fold as a link, the state, the measure, the note and
/// the action.
fn item_row(locale: &Locale, row: Row<'_>) -> String {
    let Row {
        level,
        card,
        group,
        detail,
        posinset,
        setsize,
    } = row;
    let fold = group.and_then(|group| {
        let href = group.toggle.as_deref()?;
        let (glyph, text) = if group.collapsed {
            (Glyph::CHEVRON_RIGHT, Text::TopologyExpand)
        } else {
            (Glyph::CHEVRON_DOWN, Text::TopologyCollapse)
        };
        let name = tr_with(locale, text, &[("title", card.name.clone().into())]);
        Some(Html::raw(format!(
            "<a href=\"{}\" aria-label=\"{}\" tabindex=\"-1\" class=\"fui-tree-fold\">{}</a>",
            Html::text(href),
            Html::text(name.text()),
            icon(glyph, "14")
        )))
    });
    let name = if card.mono {
        Html::raw(format!("<code>{}</code>", Html::text(&card.name)))
    } else if level == 1 {
        Html::raw(format!("<strong>{}</strong>", Html::text(&card.name)))
    } else {
        Html::text(&card.name)
    };
    let class = match level {
        1 => "fui-tree-row",
        2 => "fui-tree-row fui-tree-l2",
        _ => "fui-tree-row fui-tree-l3",
    };
    ItemView {
        expanded: group.map(|group| !group.collapsed),
        level,
        posinset,
        setsize,
        class,
        muted: card.state == FlowState::Off,
        fold,
        glyph: icon(card.glyph, "16"),
        href: &card.href,
        name,
        state: state_word(locale, card.state, detail),
        measure: card.measure.as_deref(),
        note: card.note.as_deref(),
        action: card.action.as_ref(),
        action_glyph: card
            .action
            .as_ref()
            .map_or_else(Html::default, |action| icon(action.glyph, "14")),
        ghost: Variant::Ghost.small(),
    }
    .html()
    .into()
}

/// The List twin: the same graph as an indented tree, the hub first, then
/// each lane with its groups and their children, folded as the map is.
pub(super) fn list(locale: &Locale, topology: &Topology) -> String {
    let mut rows = vec![item_row(
        locale,
        Row {
            level: 1,
            card: &topology.hub,
            group: None,
            detail: None,
            posinset: 1,
            setsize: 1,
        },
    )];
    if let Some(callers) = &topology.callers {
        rows.push(lane_row(&callers.title, &callers.line));
        let size = callers.cards.len();
        for (index, card) in callers.cards.iter().enumerate() {
            rows.push(item_row(
                locale,
                Row {
                    level: 2,
                    card,
                    group: None,
                    detail: None,
                    posinset: index + 1,
                    setsize: size,
                },
            ));
        }
    }
    for side in [&topology.start, &topology.end].into_iter().flatten() {
        rows.push(lane_row(&side.title, &side.line));
        let size = side.groups.len();
        for (index, group) in side.groups.iter().enumerate() {
            rows.push(item_row(
                locale,
                Row {
                    level: 2,
                    card: &group.card,
                    group: Some(group),
                    detail: group.card.edge_label.clone(),
                    posinset: index + 1,
                    setsize: size,
                },
            ));
            if group.collapsed {
                continue;
            }
            let count = group.children.len();
            for (place, child) in group.children.iter().enumerate() {
                rows.push(item_row(
                    locale,
                    Row {
                        level: 3,
                        card: child,
                        group: None,
                        detail: child
                            .edge_label
                            .clone()
                            .filter(|_| child.state.is_problem()),
                        posinset: place + 1,
                        setsize: count,
                    },
                ));
            }
        }
    }
    format!(
        "<ul role=\"tree\" aria-label=\"{}\" class=\"fui-topology-list\">{}</ul>",
        Html::text(&topology.list_label),
        rows.concat()
    )
}
