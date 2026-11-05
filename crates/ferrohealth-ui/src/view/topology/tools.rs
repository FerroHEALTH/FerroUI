// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The topology's tools beside the map: the minimap, the cluster controls,
//! the status menu, the view switch and the quick look.

use crate::html::Html;
use crate::i18n::{Arg, Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::filter_menu::{FilterChoice, FilterMenu, MenuForm, filter_menu};
use crate::view::flow::{FlowState, line_sample};

use super::layout::{Layout, MINIMAP, MINIMAP_TALL};
use super::{ClusterLink, QuickLook, StatusMenu, TopologyView};

/// The minimap of `layout`: a rectangle per card in its tone, and the
/// frame of what the map shows, the whole drawing at first. A tall
/// federation narrows it to keep it inside its tallest measure.
pub(super) fn minimap(locale: &Locale, layout: &Layout<'_>) -> Html {
    let scale = (MINIMAP / layout.width.max(1.0)).min(MINIMAP_TALL / layout.height.max(1.0));
    let (width, height) = (layout.width * scale, layout.height * scale);
    let rects: Vec<String> = layout
        .cards
        .iter()
        .map(|laid| {
            format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"1\" class=\"fui-minimap-node {}\"></rect>",
                laid.place.x * scale,
                laid.place.y * scale,
                laid.place.width * scale,
                (laid.place.height * scale).max(2.0),
                laid.card.state.tone().class()
            )
        })
        .collect();
    let (outer_width, outer_height) = (width + 8.0, height + 8.0);
    Html::raw(format!(
        "<div role=\"img\" aria-label=\"{}\" class=\"fui-topology-minimap\"><svg viewBox=\"-4 -4 {outer_width:.0} {outer_height:.0}\" width=\"{outer_width:.0}\" height=\"{outer_height:.0}\" aria-hidden=\"true\">{}<rect data-scale=\"{scale:.6}\" x=\"0\" y=\"0\" width=\"{width:.1}\" height=\"{height:.1}\" rx=\"2\" class=\"fui-minimap-view\"></rect></svg></div>",
        Html::text(tr(locale, Text::TopologyMinimapFrame).text()),
        rects.concat()
    ))
}

/// One icon button of the zoom cluster with its tooltip above it, waiting
/// for the `topology` module, which `wait` says.
fn cluster_button(glyph: Glyph, words: &Word, zoom: &'static str, wait: &Word) -> String {
    format!(
        "<span class=\"fui-anchor\"><button type=\"button\" aria-label=\"{}\" data-zoom=\"{zoom}\" disabled title=\"{}\" class=\"fui-icon-button\">{}</button><span role=\"tooltip\" class=\"fui-tooltip fui-tooltip-above\">{words}</span></span>",
        Html::text(words.text()),
        Html::text(wait.text()),
        icon(glyph, "16")
    )
}

/// One link of the zoom cluster that the server answers, pressed while on.
fn cluster_link(glyph: Glyph, words: &Word, link: &ClusterLink) -> String {
    format!(
        "<span class=\"fui-anchor\"><a href=\"{}\" aria-label=\"{}\" aria-pressed=\"{}\" class=\"fui-icon-button\">{}</a><span role=\"tooltip\" class=\"fui-tooltip fui-tooltip-above\">{words}</span></span>",
        Html::text(&link.href),
        Html::text(words.text()),
        link.on,
        icon(glyph, "16")
    )
}

/// The zoom cluster: zoom in and out, the readout and Fit, which wait for
/// the `topology` module, then the server's links.
pub(super) fn cluster(
    locale: &Locale,
    collapse_all: Option<&ClusterLink>,
    minimap: Option<&ClusterLink>,
) -> String {
    let wait = tr(locale, Text::TopologyZoomWait);
    let level = tr_with(
        locale,
        Text::TopologyZoomLevel,
        &[("value", Arg::from(100))],
    );
    let readout = tr_with(locale, Text::TopologyZoom, &[("value", Arg::from(100))]);
    let links = collapse_all.is_some() || minimap.is_some();
    let collapse = collapse_all.map_or_else(String::new, |link| {
        let (glyph, text) = if link.on {
            (Glyph::CHEVRON_DOWN, Text::TopologyExpandAll)
        } else {
            (Glyph::CHEVRON_UP, Text::TopologyCollapseAll)
        };
        cluster_link(glyph, &tr(locale, text), link)
    });
    let minimap = minimap.map_or_else(String::new, |link| {
        cluster_link(Glyph::LAYERS, &tr(locale, Text::TopologyMinimap), link)
    });
    let sep = if links {
        "<span aria-hidden=\"true\" class=\"fui-topology-zoom-sep\"></span>"
    } else {
        ""
    };
    format!(
        "<div role=\"toolbar\" aria-label=\"{controls}\" class=\"fui-topology-zoom\">{zoom_in}{zoom_out}<output aria-label=\"{level}\" data-zoom-readout=\"\" class=\"fui-topology-zoom-level\">{readout}</output><button type=\"button\" data-zoom=\"fit\" disabled title=\"{wait_text}\" data-title=\"{fit_tip}\" class=\"{ghost}\">{fit}</button>{sep}{collapse}{minimap}</div>",
        controls = Html::text(tr(locale, Text::TopologyControls).text()),
        zoom_in = cluster_button(Glyph::PLUS, &tr(locale, Text::TopologyZoomIn), "in", &wait),
        zoom_out = cluster_button(
            Glyph::MINUS,
            &tr(locale, Text::TopologyZoomOut),
            "out",
            &wait
        ),
        level = Html::text(level.text()),
        wait_text = Html::text(wait.text()),
        fit_tip = Html::text(tr(locale, Text::TopologyFitWhole).text()),
        ghost = Variant::Ghost.small(),
        fit = tr(locale, Text::TopologyFit),
    )
}

/// The health dot of `state` with its word as its accessible name; a state
/// switched off has no shape.
fn state_dot(locale: &Locale, state: FlowState) -> String {
    let word = state.word(locale);
    if state == FlowState::Off {
        return format!("<span class=\"fui-sr-only\">{word}</span>");
    }
    let text = Html::text(word.text());
    format!(
        "<span role=\"img\" aria-label=\"{text}\" title=\"{text}\" class=\"fui-dot {}\"></span>",
        state.tone().class()
    )
}

/// A state as its shape and its word (F12), with a quiet detail after it;
/// a state switched off draws no shape, as its muted row says.
pub(super) fn state_word(locale: &Locale, state: FlowState, detail: Option<String>) -> Html {
    let word = state.word(locale);
    if state == FlowState::Off {
        let detail = detail.map_or_else(String::new, |detail| {
            format!(
                " <span class=\"fui-status-detail\">{}</span>",
                Html::text(&detail)
            )
        });
        return Html::raw(format!(
            "<span class=\"fui-state fui-tone-neutral is-muted\">{word}{detail}</span>"
        ));
    }
    crate::view::state::state(state.into(), word, detail)
}

/// The Status menu: the filter menu of the states, each its line, its dot,
/// its word and its count.
pub(super) fn status_menu(locale: &Locale, menu: &StatusMenu) -> Html {
    let choices = menu
        .states
        .iter()
        .map(|each| {
            FilterChoice::new(
                each.state.key(),
                each.state.word(locale),
                u64::from(each.count),
                each.shown,
            )
            .mark(Html::raw(format!(
                "{}{}",
                line_sample(each.state),
                state_dot(locale, each.state)
            )))
        })
        .collect();
    filter_menu(
        locale,
        FilterMenu {
            id: "topology-show",
            name: &menu.name,
            glyph: Glyph::FUNNEL,
            label: tr(locale, Text::TopologyStatus),
            title: tr(locale, Text::TopologyStates).text().to_owned(),
            choices,
            note: Some(tr(locale, Text::TopologyLegend)),
            form: MenuForm::Own {
                action: Some(&menu.action),
                fields: &[],
            },
        },
    )
}

/// The Map and List switch: two links, the view in force pressed, and in
/// the automatic view the stylesheet presses the List below 48 rem.
pub(super) fn view_switch(
    locale: &Locale,
    view_in_force: TopologyView,
    (map, list): &(String, String),
) -> String {
    let map_on = view_in_force != TopologyView::List;
    let class = if view_in_force == TopologyView::Auto {
        "fui-segmented fui-view-switch fui-view-auto"
    } else {
        "fui-segmented fui-view-switch"
    };
    format!(
        "<div role=\"group\" aria-label=\"{label}\" class=\"{class}\"><a href=\"{map}\" aria-pressed=\"{map_on}\" class=\"fui-segment fui-view-map\">{map_glyph}{map_words}</a><a href=\"{list}\" aria-pressed=\"{list_on}\" class=\"fui-segment fui-view-list\">{list_glyph}{list_words}</a></div>",
        label = Html::text(tr(locale, Text::TopologyView).text()),
        map = Html::text(map),
        list = Html::text(list),
        list_on = !map_on,
        map_glyph = icon(Glyph::NETWORK, "14"),
        map_words = tr(locale, Text::TopologyViewMap),
        list_glyph = icon(Glyph::LIST_CHECKS, "14"),
        list_words = tr(locale, Text::TopologyViewList),
    )
}

/// The quick look beside the canvas.
pub(super) fn quick_look(locale: &Locale, look: QuickLook) -> String {
    let QuickLook {
        card: _,
        title,
        mono,
        line,
        close,
        body,
    } = look;
    let title = if mono {
        format!("<code>{}</code>", Html::text(&title))
    } else {
        Html::text(&title).to_string()
    };
    let line = line.map_or_else(String::new, |line| {
        format!("<p class=\"fui-small fui-muted\">{}</p>", Html::text(&line))
    });
    let close_words = Html::text(tr(locale, Text::TopologyLookClose).text());
    format!(
        "<aside aria-labelledby=\"topology-look-title\" class=\"fui-topology-panel\"><div class=\"fui-side-panel-head\"><div><h2 id=\"topology-look-title\" class=\"fui-card-title\">{title}</h2>{line}</div><a href=\"{close}\" aria-label=\"{close_words}\" title=\"{close_words}\" class=\"fui-icon-button\">{x}</a></div><div class=\"fui-side-panel-body fui-stack\">{body}</div></aside>",
        close = Html::text(&close),
        x = icon(Glyph::X, "16"),
        body = String::from(body),
    )
}
