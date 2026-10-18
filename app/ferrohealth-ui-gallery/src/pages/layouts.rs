// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The layouts page.
//!
//! It shows three reading panes around the tree view, a list beside the item
//! it opens, a runner's form beside its answer with the help switch and the
//! picker, rows of fields, and the stacks at each step.

use std::collections::BTreeSet;

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::chart::proportion;
use ferrohealth_ui::view::form::{
    FORM, INPUT, MEDIUM, NARROW, PROSE, SELECT, check_field, field, help_id,
};
use ferrohealth_ui::view::layout::{
    FieldRow, Gap, Pane, bullet_list, divided_list, field_row, main_aside, master_detail, stack_at,
    three_panes, tile_list,
};
use ferrohealth_ui::view::status::pill;
use ferrohealth_ui::view::surface::{card, definitions, or_none, page_header, split};
use ferrohealth_ui::widget::help::{explained_field, help_switch, provide_help};
use ferrohealth_ui::widget::picker::{Picker, picker_field};
use ferrohealth_ui::widget::tree::{Tree, TreeNode, tree_view_collapsing};
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_query_map};

/// The page of the brand in force, with `query`.
fn here(query: &str) -> String {
    let brand = crate::app::route_brand().map_or("ferrobridge", |brand| brand.key());
    format!("/{brand}/layouts{query}")
}

/// A synthetic concept of the tree, a link to itself.
fn concept(code: &str, words: &str) -> TreeNode {
    TreeNode::new(code, words)
        .code(code)
        .href(here(&format!("?concept={code}")))
}

/// The synthetic hierarchy: the top read at once, a branch read when it
/// opens.
fn hierarchy() -> Vec<TreeNode> {
    vec![
        concept("SYN-100", "Synthetic finding").children(vec![
            concept("SYN-110", "Synthetic finding of the arm"),
            concept("SYN-120", "Synthetic finding of the leg").expandable(),
        ]),
        concept("SYN-200", "Synthetic procedure").expandable(),
        concept("SYN-300", "Synthetic body site"),
    ]
}

/// The children read when the item `key` opens, the first time.
fn read_children(nodes: &mut [TreeNode], key: &str) {
    for node in nodes.iter_mut() {
        if node.key == key && node.children.is_empty() {
            node.children = vec![
                concept(&format!("{key}-1"), "Synthetic child, read on opening"),
                concept(&format!("{key}-2"), "Another synthetic child"),
            ];
        } else {
            read_children(&mut node.children, key);
        }
    }
}

/// The open items the address names (`?open=a,b`), the top item when it
/// names none.
fn open_in(query: Option<String>) -> BTreeSet<String> {
    match query {
        Some(list) => list
            .split(',')
            .filter(|key| !key.is_empty())
            .map(str::to_owned)
            .collect(),
        None => BTreeSet::from([String::from("SYN-100")]),
    }
}

/// The address of the layouts page with the open items `open` and the
/// selected concept `selected`.
fn address(open: &BTreeSet<String>, selected: Option<&str>) -> String {
    let open = open
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(",");
    match selected {
        Some(code) => here(&format!("?open={open}&concept={code}")),
        None => here(&format!("?open={open}")),
    }
}

/// The concept browser: a search, the hierarchy and the concept.
///
/// The tree keeps its open items in the address, so a reload or a shared
/// link opens the same items: opening adds an item, closing removes it.
fn browser() -> AnyView {
    let query = use_query_map();
    let selected = Signal::derive(move || query.with(|query| query.get("concept")));
    let opened = open_in(query.with_untracked(|query| query.get("open")));
    let mut first = hierarchy();
    for key in &opened {
        read_children(&mut first, key);
    }
    let nodes = RwSignal::new(first);
    let expanded = RwSignal::new(opened);
    let navigate = use_navigate();
    let remember = move || {
        let url = address(
            &expanded.get_untracked(),
            selected.get_untracked().as_deref(),
        );
        navigate(
            &url,
            NavigateOptions {
                replace: true,
                scroll: false,
                ..NavigateOptions::default()
            },
        );
    };
    let on_open = remember.clone();
    let tree = tree_view_collapsing(
        Tree {
            id: "gallery-tree",
            label: "Synthetic hierarchy",
            nodes: nodes.into(),
            expanded,
            selected,
            on_expand: Some(Callback::new(move |key: String| {
                nodes.update(|nodes| read_children(nodes, &key));
                on_open();
            })),
            actions: None,
        },
        Callback::new(move |_key: String| remember()),
    );
    let found = ["SYN-110", "SYN-120", "SYN-300"]
        .into_iter()
        .map(|code| {
            view! {
                <a href=here(&format!("?concept={code}"))>
                    <code>{code}</code>
                </a>
            }
            .into_any()
        })
        .collect();
    let search = view! {
        {field(
            "browse-find",
            "Find a concept",
            None,
            view! { <input id="browse-find" name="find" type="search" class=INPUT /> },
        )}
        {divided_list(Some("Matches"), found)}
    };
    let detail = move || {
        let code = selected.get().unwrap_or_else(|| String::from("SYN-100"));
        view! {
            <h2 class="fui-card-title">{code.clone()}</h2>
            {definitions(vec![
                ("Code", view! { <code>{code}</code> }.into_any()),
                ("System", String::from("http://example.org/synthetic").into_any()),
                ("Status", String::from("active").into_any()),
                ("Parent", or_none(String::new()).into_any()),
            ])}
        }
    };
    card(
        "Three panes and the tree",
        three_panes(
            Pane::new("Search", search),
            Pane::new("Hierarchy", tree),
            Pane::new("Concept", detail),
        ),
    )
    .into_any()
}

/// A list of value sets beside the one it opens.
fn listing() -> AnyView {
    let items = ["Synthetic signs", "Synthetic sites", "Synthetic units"]
        .into_iter()
        .map(|name| view! { <a href=here("")>{name}</a> }.into_any())
        .collect();
    card(
        "Master and detail",
        master_detail(
            divided_list(Some("Value sets"), items),
            stack_at(
                Gap::Default,
                view! {
                    {definitions(vec![
                        ("Name", String::from("Synthetic signs").into_any()),
                        ("Version", String::from("1.0.0").into_any()),
                        ("Concepts", String::from("12").into_any()),
                    ])}
                    {bullet_list(vec![
                        String::from("Composed of two synthetic code systems.").into_any(),
                        String::from("Excludes the inactive concepts.").into_any(),
                    ])}
                },
            ),
        ),
    )
    .into_any()
}

/// The runner's form: the help switch, the picker, a number, a select, a
/// prose text area and a checkbox with help.
fn runner_form() -> AnyView {
    let ferrohealth_ui::widget::help::HelpShown(_shown) = provide_help();
    let choices = Signal::derive(|| {
        vec![
            (
                String::from("http://example.org/vs/signs"),
                String::from("Synthetic signs"),
            ),
            (
                String::from("http://example.org/vs/sites"),
                String::from("Synthetic sites"),
            ),
            (
                String::from("http://example.org/vs/units"),
                String::from("Synthetic units"),
            ),
        ]
    });
    let value = RwSignal::new(String::from("http://example.org/vs/signs"));
    let picker = picker_field(Picker {
        id: "run-value-set",
        name: "url",
        label: "Value set",
        choose_label: "Choose a value set this server holds",
        help: Some("The canonical of the value set to list."),
        choices,
        value,
        empty_word: "Choose one this server holds",
        other_word: "Another canonical",
    });
    let count = view! {
        <input
            id="run-count"
            name="count"
            inputmode="numeric"
            class=format!("{INPUT} {NARROW}")
            value="25"
            aria-describedby=help_id("run-count")
        />
    };
    let language = view! {
        <select id="run-language" name="language" class=format!("{SELECT} {MEDIUM}") aria-describedby=help_id("run-language")>
            <option value="en">"English"</option>
            <option value="nl">"Dutch"</option>
        </select>
    };
    let note = view! {
        <textarea id="run-note" name="note" rows="2" class=PROSE aria-describedby=help_id("run-note")></textarea>
    };
    let inactive = view! {
        <input id="run-inactive" name="inactive" type="checkbox" aria-describedby=help_id("run-inactive") />
    };
    view! {
        <form method="get" action=here("") class=FORM>
            {split(view! { <h2 class="fui-card-title">"List a value set"</h2> }, help_switch())}
            {picker}
            {explained_field("run-count", "Page size", "How many concepts one page lists, 1 to 1000.", count)}
            {explained_field("run-language", "Display language", "The language of the displays.", language)}
            {explained_field("run-note", "Why you run it", "A note kept beside the run.", note)}
            {check_field("run-inactive", "Include inactive concepts", "An inactive concept is listed and marked.", inactive)}
            <button type="submit" class=Variant::Primary.class()>
                {label(Some(Glyph::PLAY), "List")}
            </button>
        </form>
    }
    .into_any()
}

/// The runner's answer: latencies with their proportion bars.
fn runner_answer() -> AnyView {
    let rows = [
        ("lookup", 12_u8, "12 ms"),
        ("expand", 100, "96 ms"),
        ("translate", 33, "31 ms"),
    ]
    .into_iter()
    .map(|(name, share, figure)| {
        view! {
            {split(name, view! { <span class="fui-num">{figure}</span> })}
            {proportion(share, None)}
        }
        .into_any()
    })
    .collect();
    view! {
        <h2 class="fui-card-title">"Latency"</h2>
        {divided_list(Some("Latency per operation"), rows)}
        {proportion(0, Some(String::from("No time spent, 0 percent")))}
        <h2 class="fui-card-title">"Nodes"</h2>
        {tile_list(
            "The nodes that answered",
            vec![
                view! {
                    <code class="fui-break">"synthetic-node-a"</code>
                    {pill(ferrohealth_ui::tone::Tone::Ok, "answered")}
                    <span class="fui-muted fui-num">"12 ms"</span>
                }
                .into_any(),
                view! {
                    <code class="fui-break">"synthetic-node-c"</code>
                    {pill(ferrohealth_ui::tone::Tone::Warn, "timed out")}
                    <span class="fui-muted fui-num">"30 s"</span>
                }
                .into_any(),
            ],
        )}
    }
    .into_any()
}

/// Rows of fields with columns of their own, as an editor draws them.
fn rows() -> AnyView {
    let input = |id: &'static str, value: &'static str| {
        view! { <input id=id name=id class=INPUT value=value /> }
    };
    let remove = || {
        view! {
            <button type="button" class=Variant::Danger.class()>
                {label(Some(Glyph::TRASH), "Remove")}
            </button>
        }
    };
    let up = || {
        view! {
            <button type="button" class=Variant::Secondary.class()>
                {label(Some(Glyph::ARROW_UP), "Up")}
            </button>
        }
    };
    card(
        "Rows of fields",
        stack_at(
            Gap::Default,
            view! {
                {field_row(
                    FieldRow::CodeDisplay,
                    view! {
                        {field("row-code", "Code", None, input("row-code", "SYN-110"))}
                        {field("row-display", "Display", None, input("row-display", "Synthetic finding of the arm"))}
                        {up()}
                        {remove()}
                    },
                )}
                {field_row(
                    FieldRow::CodeDisplayDefinition,
                    view! {
                        {field("row-code-2", "Code", None, input("row-code-2", "SYN-120"))}
                        {field("row-display-2", "Display", None, input("row-display-2", "Synthetic finding of the leg"))}
                        {field("row-definition-2", "Definition", None, input("row-definition-2", "A finding invented for the gallery."))}
                        {remove()}
                    },
                )}
                {field_row(
                    FieldRow::Halves,
                    view! {
                        {field("row-system", "System", None, input("row-system", "http://example.org/synthetic"))}
                        {field("row-version", "Version", None, input("row-version", "1.0.0"))}
                    },
                )}
            },
        ),
    )
    .into_any()
}

/// The stacks at each step of space.
fn stacks() -> AnyView {
    let lines = |gap: Gap, name: &'static str| {
        view! {
            <div class="fui-well">
                {stack_at(gap, view! {
                    <p>{name}</p>
                    <p class="fui-muted">"A second line."</p>
                    <p class="fui-muted">"A third line."</p>
                })}
            </div>
        }
    };
    card(
        "Stacks",
        stack_at(
            Gap::Default,
            view! {
                {lines(Gap::Tight, "Tight")}
                {lines(Gap::Default, "Default")}
                {lines(Gap::Loose, "Loose")}
                {lines(Gap::Section, "Section")}
            },
        ),
    )
    .into_any()
}

/// The layouts page.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn LayoutsPage() -> impl IntoView {
    let header = page_header(
        "Layouts",
        Some(String::from(
            "Three panes and the tree, a list beside its item, a form beside its answer, rows of fields and stacks.",
        )),
        (),
    )
    .into_any();
    let sections = (
        browser(),
        listing(),
        card("Main and aside", main_aside(runner_form(), runner_answer())).into_any(),
        rows(),
        stacks(),
    );
    view! {
        {super::title("Layouts")}
        {header}
        {stack_at(
            Gap::Loose,
            view! {
                {sections.0}
                {sections.1}
                {sections.2}
                {sections.3}
                {sections.4}
            },
        )}
    }
}
