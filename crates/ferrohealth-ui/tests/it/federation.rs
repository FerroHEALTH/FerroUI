// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The federation parts: the fan-out outcome (F5), the replica line (F6),
//! the nested sidebar section (F10) and the topology with its List twin
//! (F9). Each state is a shape, a colour and a word (family rule 12), every
//! action is a link or a form, and nothing writes a style attribute.

use ferrohealth_ui::i18n::Word;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::fanout::{Answer, Fanout, FanoutNode, Verdict, fanout};
use ferrohealth_ui::view::flow::FlowState;
use ferrohealth_ui::view::replica::Replica;
use ferrohealth_ui::view::surface::Header;
use ferrohealth_ui::view::template::settings::{How, SettingsPage, Source, settings_page};
use ferrohealth_ui::view::topology::{
    ClusterLink, QuickLook, RowAction, StatusCount, StatusMenu, Topology, TopologyCallers,
    TopologyCard, TopologyGroup, TopologySide, TopologyView, topology,
};
use ferrohealth_ui::view::nav::{Entry, Group, Nest, NestItem, Sidebar, sidebar};

use crate::render;

/// `view` rendered as a server render runs, its hydration marks left out.
fn html(view: impl FnOnce() -> AnyView) -> String {
    render::html(view).replace("", "")
}

/// `view` rendered in a page tree, its hydration marks left out.
fn page_html(view: impl FnOnce() -> AnyView) -> String {
    render::page_html(view).replace("", "")
}

fn node(name: &str, answer: Answer) -> FanoutNode {
    FanoutNode {
        name: name.to_owned(),
        answer,
        word: String::from(match answer {
            Answer::Answered => "Answered",
            Answer::Failed => "Node error",
            Answer::Skipped => "Excluded",
        }),
        term: Some(String::from("active")),
        measure: String::from("3 rows · 41 ms"),
        reason: (answer == Answer::Failed).then(|| String::from("It answered 503.")),
    }
}

fn outcome(nodes: Vec<FanoutNode>) -> String {
    html({
        fanout(Fanout {
            label: String::from("What each endpoint did"),
            line: String::from("2 of 3 asked nodes answered"),
            missing: String::from("node-b answered 503."),
            nodes,
        })
    })
}

#[test]
fn a_fan_out_says_incomplete_answer_in_words_before_the_rows_when_a_node_failed() {
    let partial = outcome(vec![
        node("node-a", Answer::Answered),
        node("node-b", Answer::Failed),
        node("node-e", Answer::Skipped),
    ]);
    let notice = partial.find("Incomplete answer.").expect("{partial}");
    let rows = partial.find("fui-fanout-list").expect("{partial}");
    assert!(notice < rows, "the words come before the rows: {partial}");
    assert!(partial.contains("fui-pill fui-tone-warn"), "{partial}");
    assert!(partial.contains(">Incomplete<"), "{partial}");
    assert!(
        partial.contains(r#"<span class="fui-fanout-error">It answered 503.</span>"#),
        "{partial}"
    );
    assert!(partial.contains("is-excluded"), "{partial}");
    assert!(!partial.contains("style="), "{partial}");

    let complete = outcome(vec![node("node-a", Answer::Answered)]);
    assert!(!complete.contains("Incomplete answer."), "{complete}");
    assert!(complete.contains(">Complete<"), "{complete}");

    let failed = outcome(vec![
        node("node-a", Answer::Failed),
        node("node-b", Answer::Failed),
    ]);
    assert!(failed.contains("Incomplete answer."), "{failed}");
    assert!(failed.contains(">Failed<"), "{failed}");
}

#[test]
fn each_node_says_its_state_as_a_pill_with_its_shape_and_word() {
    let shown = outcome(vec![
        node("node-a", Answer::Answered),
        node("node-b", Answer::Failed),
    ]);
    assert!(
        shown.contains(r#"<span class="fui-pill fui-tone-ok">Answered</span>"#),
        "{shown}"
    );
    assert!(
        shown.contains(r#"<span class="fui-pill fui-tone-bad">Node error</span>"#),
        "{shown}"
    );
}

#[test]
fn the_verdict_follows_the_failures() {
    let answered = node("a", Answer::Answered);
    let failed = node("b", Answer::Failed);
    let skipped = node("c", Answer::Skipped);
    assert_eq!(
        Verdict::Complete,
        Verdict::of(&[answered.clone(), skipped.clone()])
    );
    assert_eq!(
        Verdict::Incomplete,
        Verdict::of(&[answered, failed.clone()])
    );
    assert_eq!(Verdict::Failed, Verdict::of(&[failed, skipped]));
}

#[test]
fn a_reference_page_draws_the_replica_line_under_its_header() {
    let page = html({
        settings_page(SettingsPage {
            header: Header {
                title: Word::from("Self-description"),
                ..Header::default()
            },
            actions: ActionGroup::none(),
            source: Source::File {
                file: String::from("ferrofed.toml"),
                how: How::Page(String::from("/how")),
            },
            tabs: None,
            settings: Vec::new(),
            replica: Some(Replica {
                name: String::from("gateway-0"),
                at: None,
            }),
        })
    });
    let header = page.find("</header>").expect("{page}");
    let line = page.find("fui-replica-line").expect("{page}");
    assert!(header < line, "{page}");
    assert!(
        page.contains("Answered by replica <code>gateway-0</code>"),
        "{page}"
    );
}

fn item(label: &str, href: &str, mono: bool) -> NestItem {
    NestItem {
        label: label.to_owned(),
        href: href.to_owned(),
        state: FlowState::Up,
        state_words: String::from("Reachable"),
        mono,
    }
}

fn navigation(current: &str, collapsed: bool) -> String {
    navigation_of(
        current,
        collapsed,
        vec![
            item("node-a", "/nodes/node-a", true),
            NestItem {
                state: FlowState::Off,
                state_words: String::from("Excluded"),
                ..item("node-e", "/nodes/node-e", true)
            },
        ],
    )
}

fn navigation_of(current: &str, collapsed: bool, children: Vec<NestItem>) -> String {
    html({
        sidebar(Sidebar {
            id: "sidebar",
            label: String::from("Console"),
            groups: vec![Group {
                id: "nav-fed",
                title: "Federation".into(),
                entries: vec![Entry {
                    href: "/organisations".into(),
                    label: "Organisations".into(),
                    glyph: Glyph::USERS,
                    exact: false,
                    count: None,
                }],
            }],
            current: current.to_owned(),
            rail: Some(ferrohealth_ui::view::nav::Rail {
                collapsed,
                action: String::from("/sidebar"),
                back: current.to_owned(),
            }),
            nest: Some(Nest {
                under: "/organisations".into(),
                label: String::from("Voorbeeldstad and its nodes"),
                parent: NestItem {
                    state_words: String::from("Worst node: reachable"),
                    ..item("Voorbeeldstad", "/organisations/voorbeeldstad", false)
                },
                count_words: format!("{} nodes", children.len()),
                children,
                all: String::from("All nodes"),
            }),
        })
    })
}

#[test]
fn the_nested_section_marks_the_entry_as_the_location_and_the_node_as_the_page() {
    let page = navigation("/nodes/node-a", false);
    assert!(
        page.contains(r#"<a href="/organisations" aria-current="location" class="fui-nav-entry">"#),
        "{page}"
    );
    assert!(
        page.contains(r#"<a href="/organisations/voorbeeldstad" aria-current="location" title="Voorbeeldstad" class="fui-nav-entry fui-nav-parent">"#),
        "{page}"
    );
    assert!(
        page.contains(
            r#"<a href="/nodes/node-a" aria-current="page" title="node-a" class="fui-nav-entry fui-nav-child">"#
        ),
        "{page}"
    );
    assert!(
        page.contains(r#"<ul aria-label="Voorbeeldstad and its nodes" class="fui-nav-tree">"#),
        "{page}"
    );
    assert!(
        page.contains(r#"role="img" aria-label="Reachable" title="Reachable""#),
        "each dot is a shape named by its word: {page}"
    );
    assert!(
        page.contains(r#"<span class="fui-sr-only">Excluded</span>"#),
        "an item switched off says its word and draws no shape: {page}"
    );
    assert!(
        page.contains(r#"<span class="fui-sr-only">2 nodes</span>"#),
        "{page}"
    );

    let parent = navigation("/organisations/voorbeeldstad", false);
    assert!(
        parent.contains(
            r#"aria-current="page" title="Voorbeeldstad" class="fui-nav-entry fui-nav-parent""#
        ),
        "{parent}"
    );
}

#[test]
fn a_nested_section_with_hundreds_of_children_draws_a_few_the_page_and_a_link_to_all() {
    // A parent with 300 nodes buried the groups under it (shell edge audit, row 6).
    let children: Vec<NestItem> = (0..300)
        .map(|at| {
            item(
                &format!("node-{at:03}"),
                &format!("/nodes/node-{at:03}"),
                true,
            )
        })
        .collect();
    let page = navigation_of("/nodes/node-250", false, children);
    let drawn = page.matches("fui-nav-entry fui-nav-child\"").count();
    assert_eq!(
        ferrohealth_ui::view::nav::NEST_SHOWN,
        drawn,
        "the section draws a few children: {page}"
    );
    assert!(
        page.contains(r#"<a href="/nodes/node-250" aria-current="page" title="node-250""#),
        "the page in view is always drawn: {page}"
    );
    assert!(
        page.contains(r#"<a href="/organisations/voorbeeldstad" class="fui-nav-entry fui-nav-child fui-nav-all"><span class="fui-nav-word">All nodes</span></a>"#),
        "a link leads to every child: {page}"
    );
    assert!(
        page.contains(r#"<span class="fui-sr-only">300 nodes</span>"#),
        "the count still says all of them: {page}"
    );
    let few = navigation("/nodes/node-a", false);
    assert!(
        !few.contains("fui-nav-all"),
        "a few children need no link: {few}"
    );
}

#[test]
fn the_rail_keeps_the_nested_section_for_its_drawer_and_the_stylesheet_hides_it() {
    let page = navigation("/nodes/node-a", true);
    assert!(page.contains("fui-sidebar-rail"), "{page}");
    assert!(page.contains("fui-nav-tree"), "the drawer keeps it: {page}");
    let css = ferrohealth_ui::style::STYLESHEET;
    assert!(
        css.contains(
            "@media (width >= 48rem) {\n    .fui-sidebar-rail .fui-nav-tree {\n      display: none;"
        ),
        "the rail leaves it out from 48 rem up"
    );
}

fn card(id: &str, state: FlowState, label: Option<&str>) -> TopologyCard {
    TopologyCard {
        id: id.to_owned(),
        glyph: Glyph::SERVER,
        name: id.to_owned(),
        line: format!("Line of {id}"),
        state,
        href: format!("/nodes/{id}"),
        mono: true,
        edge_label: label.map(str::to_owned),
        look: Some(format!("?look={id}")),
        measure: Some(String::from("median 38 ms")),
        note: Some(String::from("probed 13 s ago")),
        action: Some(RowAction {
            label: String::from("Probe now"),
            glyph: Glyph::ACTIVITY,
            action: String::from("/probe"),
            name: String::from("node"),
            value: id.to_owned(),
        }),
    }
}

fn group(
    id: &str,
    state: FlowState,
    children: Vec<TopologyCard>,
    collapsed: bool,
) -> TopologyGroup {
    TopologyGroup {
        card: TopologyCard {
            mono: false,
            action: None,
            ..card(
                id,
                state,
                children
                    .iter()
                    .find_map(|child| child.edge_label.as_deref()),
            )
        },
        children,
        collapsed,
        toggle: Some(format!("?collapse={id}")),
    }
}

fn federation(collapsed: bool, look: bool, view: TopologyView) -> String {
    html({
        topology(Topology {
            id: String::from("fed"),
            label: String::from("The federation as a map"),
            list_label: String::from("The federation as a list"),
            hub: TopologyCard {
                look: None,
                action: None,
                ..card("gateway", FlowState::Up, None)
            },
            callers: Some(TopologyCallers {
                title: String::from("Calls the gateway"),
                line: String::from("who calls it"),
                cards: vec![card("ncp", FlowState::NotChecked, None)],
            }),
            start: None,
            end: Some(TopologySide {
                title: String::from("Organisations"),
                line: String::from("and their nodes"),
                groups: vec![
                    group(
                        "oost",
                        FlowState::Degraded,
                        vec![card("node-b", FlowState::Degraded, Some("503"))],
                        collapsed,
                    ),
                    group(
                        "proefdorp",
                        FlowState::Unreachable,
                        vec![card("node-c", FlowState::Unreachable, Some("no answer"))],
                        false,
                    ),
                    group(
                        "stad",
                        FlowState::Up,
                        vec![card("node-a", FlowState::Up, Some("never shown"))],
                        false,
                    ),
                ],
            }),
            view,
            views: (String::from("?view=map"), String::from("?view=list")),
            status: StatusMenu {
                action: String::from("/topology"),
                name: String::from("show"),
                states: vec![
                    StatusCount {
                        state: FlowState::Up,
                        count: 1,
                        shown: true,
                    },
                    StatusCount {
                        state: FlowState::Degraded,
                        count: 1,
                        shown: false,
                    },
                ],
            },
            tools: None,
            updated: None,
            collapse_all: Some(ClusterLink {
                href: String::from("?collapse=all"),
                on: false,
            }),
            minimap: Some(ClusterLink {
                href: String::from("?minimap=on"),
                on: true,
            }),
            look: look.then(|| QuickLook {
                card: String::from("node-b"),
                title: String::from("node-b"),
                mono: true,
                line: None,
                close: String::from("?"),
                body: view! { <p>"Answers 503."</p> },
            }),
        })
    })
}

#[test]
fn the_map_draws_each_edge_state_and_marks_and_labels_only_the_problem_edges() {
    let page = federation(false, false, TopologyView::Auto);
    for class in [
        "fui-edge fui-edge-ok",
        "fui-edge fui-edge-warn",
        "fui-edge fui-edge-bad",
        "fui-edge fui-edge-neutral",
    ] {
        assert!(page.contains(class), "{class}: {page}");
    }
    assert!(page.contains("fui-edge-mark fui-edge-mark-bad"), "{page}");
    assert!(page.contains("fui-edge-mark fui-edge-mark-warn"), "{page}");
    assert!(
        page.contains(">503</text>") && page.contains(">no answer</text>"),
        "{page}"
    );
    assert!(
        !page.contains(">never shown</text>"),
        "an edge that is up carries no label: {page}"
    );
    assert!(
        page.contains(r#"role="img" aria-labelledby="fed-title fed-desc""#),
        "{page}"
    );
    assert!(
        page.contains("The List view reads the same graph as a tree."),
        "{page}"
    );
    assert!(page.contains(r#"class="fui-path-pulse""#), "{page}");
    assert!(!page.contains("style="), "{page}");
}

#[test]
fn a_collapsed_group_is_one_card_whose_edge_carries_the_mark_and_its_fold_is_a_link() {
    let open = federation(false, false, TopologyView::Auto);
    let folded = federation(true, false, TopologyView::Auto);
    assert!(
        open.contains(r#"aria-label="node-b, Line of node-b: Degraded. Opens its page.""#),
        "{open}"
    );
    assert!(
        !folded.contains(r#"aria-label="node-b, Line of node-b: Degraded. Opens its page.""#),
        "the folded group hides its child on the map: {folded}"
    );
    assert!(
        folded.contains(r#"<a href="?collapse=oost" aria-label="Expand oost">"#),
        "{folded}"
    );
    assert!(open.contains(r#"aria-label="Collapse oost""#), "{open}");
    assert!(
        folded.contains("fui-edge-mark fui-edge-mark-warn"),
        "the folded group's edge carries its worst child's mark: {folded}"
    );
}

#[test]
fn the_quick_look_opens_beside_the_canvas_and_marks_its_card() {
    let page = federation(false, true, TopologyView::Map);
    assert!(page.contains("fui-topology-map has-panel"), "{page}");
    assert!(
        page.contains(
            r#"<aside aria-labelledby="topology-look-title" class="fui-topology-panel">"#
        ),
        "{page}"
    );
    assert!(page.contains("is-selected"), "{page}");
    assert!(
        page.contains(r#"<a href="?look=node-b" aria-label="Details of node-b">"#),
        "{page}"
    );
    assert!(page.contains("fui-topology-view-map"), "{page}");
}

#[test]
fn the_zoom_cluster_waits_for_the_script_and_its_server_links_work_without_it() {
    let page = federation(false, false, TopologyView::Auto);
    assert!(
        page.contains(r#"data-zoom="in""#) && page.contains(r#"data-zoom="fit""#),
        "{page}"
    );
    assert_eq!(
        3,
        page.matches("data-zoom=").count(),
        "zoom in, zoom out and Fit: {page}"
    );
    assert_eq!(
        3,
        page.matches("disabled title=").count(),
        "each waits for the script and says why: {page}"
    );
    assert!(page.contains(r#"<a href="?collapse=all" aria-label="Collapse every group" aria-pressed="false" class="fui-icon-button">"#), "{page}");
    assert!(page.contains("fui-tooltip fui-tooltip-above"), "{page}");
    assert!(
        page.contains("fui-topology-minimap"),
        "the minimap is on: {page}"
    );
    assert!(
        page.contains(r#"aria-keyshortcuts="+ - 0 i Space""#),
        "{page}"
    );
}

#[test]
fn the_status_menu_is_the_legend_as_a_get_form_of_the_states() {
    let page = federation(false, false, TopologyView::Auto);
    assert!(
        page.contains(r#"<form method="get" action="/topology""#),
        "{page}"
    );
    assert!(page.contains(r#"name="show" value="up" checked"#), "{page}");
    assert!(page.contains(r#"name="show" value="degraded""#), "{page}");
    assert!(page.contains("Status · 1 of 2"), "{page}");
    assert!(page.contains("fui-line-sample"), "{page}");
}

#[test]
fn the_list_twin_is_a_tree_of_the_same_graph_with_the_same_actions() {
    let page = federation(false, false, TopologyView::List);
    assert!(
        page.contains(r#"role="tree" aria-label="The federation as a list""#),
        "{page}"
    );
    assert!(page.contains(r#"aria-level="3""#), "{page}");
    assert!(page.contains(r#"aria-expanded="true""#), "{page}");
    assert!(
        page.contains(r#"<a href="/nodes/node-b"><code>node-b</code></a>"#),
        "{page}"
    );
    assert!(
        page.contains(r#"<form method="post" action="/probe">"#),
        "{page}"
    );
    assert!(page.contains("fui-state fui-tone-bad"), "{page}");
    let folded = federation(true, false, TopologyView::List);
    assert!(folded.contains(r#"aria-expanded="false""#), "{folded}");
    assert!(
        !folded.contains("<code>node-b</code>"),
        "the List folds as the map does: {folded}"
    );
}
