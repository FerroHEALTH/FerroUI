// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The layouts page.
//!
//! It shows three reading panes around the tree view, a list beside the item
//! it opens, a runner's form beside its answer with the help switch and the
//! picker, rows of fields, and the stacks at each step.

use std::collections::BTreeSet;

use crate::locale::{Msg, tr};
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::chart::proportion;
use ferrohealth_ui::view::form::{FORM, INPUT, MEDIUM, NARROW, PROSE, SELECT, field, help_id};
use ferrohealth_ui::view::layout::{
    FieldRow, Gap, Pane, bullet_list, divided_list, field_row, main_aside, master_detail, stack_at,
    three_panes, tile_list,
};
use ferrohealth_ui::view::status::pill;
use ferrohealth_ui::view::surface::{card, definitions, or_none, page_header, split};
use ferrohealth_ui::widget::help::{
    explained_check_field, explained_field, help_switch, provide_help,
};
use ferrohealth_ui::widget::picker::{Picker, picker_field};
use ferrohealth_ui::widget::tree::{Tree, TreeNode, tree_view_collapsing};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

/// The key of the brand the route names, FerroBRIDGE's for none.
fn brand_key() -> String {
    crate::app::route_brand()
        .map_or("ferrobridge", |brand| brand.key())
        .to_owned()
}

/// The layouts page of the brand `brand`, with `query`.
fn here(brand: &str, query: &str) -> String {
    format!("/{brand}/layouts{query}")
}

/// A synthetic concept of the tree, a link to itself.
fn concept(brand: &str, code: &str, words: &str) -> TreeNode {
    TreeNode::new(code, words)
        .code(code)
        .href(here(brand, &format!("?concept={code}")))
}

/// The synthetic hierarchy: the top read at once, a branch read when it
/// opens.
fn hierarchy(brand: &str) -> Vec<TreeNode> {
    vec![
        concept(brand, "SYN-100", &tr(Msg::SyntheticFinding)).children(vec![
            concept(brand, "SYN-110", &tr(Msg::SyntheticFindingArm)),
            concept(brand, "SYN-120", &tr(Msg::SyntheticFindingLeg)).expandable(),
        ]),
        concept(brand, "SYN-200", &tr(Msg::SyntheticProcedure)).expandable(),
        concept(brand, "SYN-300", &tr(Msg::SyntheticBodySite)),
    ]
}

/// The children read when the item `key` opens, the first time.
fn read_children(brand: &str, nodes: &mut [TreeNode], key: &str) {
    for node in nodes.iter_mut() {
        if node.key == key && node.children.is_empty() {
            node.children = vec![
                concept(
                    brand,
                    &format!("{key}-1"),
                    &tr(Msg::SyntheticChildReadOpening),
                ),
                concept(brand, &format!("{key}-2"), &tr(Msg::AnotherSyntheticChild)),
            ];
        } else {
            read_children(brand, &mut node.children, key);
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
fn address(brand: &str, open: &BTreeSet<String>, selected: Option<&str>) -> String {
    let open = open
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(",");
    match selected {
        Some(code) => here(brand, &format!("?open={open}&concept={code}")),
        None => here(brand, &format!("?open={open}")),
    }
}

/// The concept browser: a search, the hierarchy and the concept.
///
/// The tree keeps its open items in the address, so a reload or a shared
/// link opens the same items: opening adds an item, closing removes it.
fn browser() -> AnyView {
    let brand = brand_key();
    let query = use_query_map();
    let concept = query.with_untracked(|query| query.get("concept"));
    let open = query.with_untracked(|query| query.get("open"));
    let found = ["SYN-110", "SYN-120", "SYN-300"]
        .into_iter()
        .map(|code| {
            view! {
                <a href=here(&brand, &format!("?concept={code}"))>
                    <code>{code}</code>
                </a>
            }
            .into_any()
        })
        .collect();
    let search = view! {
        {field(
            "browse-find",
            tr(Msg::FindConcept),
            None,
            view! { <input id="browse-find" name="find" type="search" class=INPUT /> },
        )}
        {divided_list(Some(tr(Msg::Matches).into()), found)}
    };
    let code = concept.clone().unwrap_or_else(|| String::from("SYN-100"));
    let detail = view! {
        <h2 class="fui-card-title">{code.clone()}</h2>
        {definitions(vec![
            (tr(Msg::Code), view! { <code>{code}</code> }.into_any()),
            (tr(Msg::System), String::from("http://example.org/synthetic").into_any()),
            (tr(Msg::Status), String::from("active").into_any()),
            (tr(Msg::Parent), or_none(String::new()).into_any()),
        ])}
    };
    let tree = view! { <crate::islands::ConceptTree brand concept open /> };
    card(
        tr(Msg::ThreePanesTree),
        three_panes(
            Pane::new(tr(Msg::Search), search),
            Pane::new(tr(Msg::Hierarchy), tree),
            Pane::new(tr(Msg::Concept), detail),
        ),
    )
    .into_any()
}

/// The hierarchy of the brand `brand`, with the concept `concept` selected
/// and the items `open` names open, keeping its open items in the address
/// as they change.
pub(crate) fn concept_tree(
    brand: String,
    concept: Option<String>,
    open: Option<String>,
) -> AnyView {
    let selected = Signal::stored(concept);
    let opened = open_in(open);
    let mut first = hierarchy(&brand);
    for key in &opened {
        read_children(&brand, &mut first, key);
    }
    let nodes = RwSignal::new(first);
    let expanded = RwSignal::new(opened);
    let brand = StoredValue::new(brand);
    let remember = move || {
        let url = brand.with_value(|brand| {
            address(
                brand,
                &expanded.get_untracked(),
                selected.get_untracked().as_deref(),
            )
        });
        replace_address(&url);
    };
    tree_view_collapsing(
        Tree {
            id: "gallery-tree",
            label: tr(Msg::SyntheticHierarchy).into(),
            nodes: nodes.into(),
            expanded,
            selected,
            on_expand: Some(Callback::new(move |key: String| {
                brand.with_value(|brand| nodes.update(|nodes| read_children(brand, nodes, &key)));
                remember();
            })),
            actions: None,
        },
        Callback::new(move |_key: String| remember()),
    )
}

/// Replaces the page's address with `url`, keeping the page and its
/// scroll, so a reload or a shared link opens the same items.
fn replace_address(url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        let replaced = window().history().and_then(|history| {
            history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(url))
        });
        if replaced.is_err() {
            leptos::logging::warn!("the browser refused to keep the open items in the address");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = url;
}

/// A list of value sets beside the one it opens.
fn listing() -> AnyView {
    let items = [
        tr(Msg::SyntheticSigns),
        tr(Msg::SyntheticSites),
        tr(Msg::SyntheticUnits),
    ]
    .into_iter()
    .map(|name| view! { <a href=here(&brand_key(), "")>{name}</a> }.into_any())
    .collect();
    card(
        tr(Msg::MasterDetail),
        master_detail(
            divided_list(Some(tr(Msg::ValueSets).into()), items),
            stack_at(
                Gap::Default,
                view! {
                    {definitions(vec![
                        (tr(Msg::Name), tr(Msg::SyntheticSigns).into_any()),
                        (tr(Msg::Version), String::from("1.0.0").into_any()),
                        (tr(Msg::Concepts), String::from("12").into_any()),
                    ])}
                    {bullet_list(vec![
                        tr(Msg::ComposedTwoSyntheticCodeSystems).into_any(),
                        tr(Msg::ExcludesInactiveConcepts).into_any(),
                    ])}
                },
            ),
        ),
    )
    .into_any()
}

/// The runner's form: the help switch, the picker, a number, a select, a
/// prose text area and a checkbox with help.
///
/// The help switch and the fields it explains share one state, so the form
/// is the island [`crate::islands::RunnerForm`].
fn runner_form() -> AnyView {
    let brand = brand_key();
    view! { <crate::islands::RunnerForm brand /> }.into_any()
}

/// The runner's form on the layouts page of the brand `brand`.
pub(crate) fn runner_form_in(brand: &str) -> AnyView {
    let ferrohealth_ui::widget::help::HelpShown(_shown) = provide_help();
    let choices = Signal::derive(|| {
        vec![
            (
                String::from("http://example.org/vs/signs"),
                tr(Msg::SyntheticSigns),
            ),
            (
                String::from("http://example.org/vs/sites"),
                tr(Msg::SyntheticSites),
            ),
            (
                String::from("http://example.org/vs/units"),
                tr(Msg::SyntheticUnits),
            ),
        ]
    });
    let value = RwSignal::new(String::from("http://example.org/vs/signs"));
    let picker = picker_field(Picker {
        id: "run-value-set",
        name: "url",
        label: tr(Msg::ValueSet).into(),
        choose_label: tr(Msg::ChooseValueSetServerHolds).into(),
        help: Some(tr(Msg::CanonicalValueSetList).into()),
        choices,
        value,
        empty_word: tr(Msg::ChooseOneServerHolds).into(),
        other_word: tr(Msg::AnotherCanonical).into(),
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
            <option value="en">{tr(Msg::English)}</option>
            <option value="nl">{tr(Msg::Dutch)}</option>
        </select>
    };
    let note = view! {
        <textarea id="run-note" name="note" rows="2" class=PROSE aria-describedby=help_id("run-note")></textarea>
    };
    let inactive = view! {
        <input id="run-inactive" name="inactive" type="checkbox" aria-describedby=help_id("run-inactive") />
    };
    view! {
        <form method="get" action=here(brand, "") class=FORM>
            {split(view! { <h2 class="fui-card-title">{tr(Msg::ListValueSet)}</h2> }, help_switch())}
            {picker}
            {explained_field("run-count", tr(Msg::PageSize), tr(Msg::HowManyConceptsOnePage), count)}
            {explained_field("run-language", tr(Msg::DisplayLanguage), tr(Msg::LanguageDisplays), language)}
            {explained_field("run-note", tr(Msg::WhyYouRun), tr(Msg::NoteKeptBesideRun), note)}
            {explained_check_field("run-inactive", tr(Msg::IncludeInactiveConcepts), tr(Msg::InactiveConceptListedMarked), inactive)}
            <button type="submit" class=Variant::Primary.class()>
                {label(Some(Glyph::PLAY), tr(Msg::List))}
            </button>
        </form>
    }
    .into_any()
}

/// The runner's answer: latencies with their proportion bars.
fn runner_answer() -> AnyView {
    let rows = [
        ("lookup", 12_u8, tr(Msg::N12Ms)),
        ("expand", 100, tr(Msg::N96Ms)),
        ("translate", 33, tr(Msg::N31Ms)),
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
        <h2 class="fui-card-title">{tr(Msg::Latency)}</h2>
        {divided_list(Some(tr(Msg::LatencyPerOperation).into()), rows)}
        {proportion(0, Some(tr(Msg::NoTimeSpent0Percent)))}
        <h2 class="fui-card-title">{tr(Msg::Nodes)}</h2>
        {tile_list(
            tr(Msg::NodesAnswered),
            vec![
                view! {
                    <code class="fui-break">"synthetic-node-a"</code>
                    {pill(ferrohealth_ui::tone::Tone::Ok, "answered")}
                    <span class="fui-muted fui-num">{tr(Msg::N12Ms)}</span>
                }
                .into_any(),
                view! {
                    <code class="fui-break">"synthetic-node-c"</code>
                    {pill(ferrohealth_ui::tone::Tone::Warn, &tr(Msg::TimedOut))}
                    <span class="fui-muted fui-num">{tr(Msg::N30S)}</span>
                }
                .into_any(),
            ],
        )}
    }
    .into_any()
}

/// Rows of fields with columns of their own, as an editor draws them.
fn rows() -> AnyView {
    let input = |id: &'static str, value: &str| {
        let value = value.to_owned();
        view! { <input id=id name=id class=INPUT value=value /> }
    };
    let remove = || {
        view! {
            <button type="button" class=Variant::Danger.class()>
                {label(Some(Glyph::TRASH), tr(Msg::Remove))}
            </button>
        }
    };
    let up = || {
        view! {
            <button type="button" class=Variant::Secondary.class()>
                {label(Some(Glyph::ARROW_UP), tr(Msg::Up))}
            </button>
        }
    };
    card(
        tr(Msg::RowsFields),
        stack_at(
            Gap::Default,
            view! {
                {field_row(
                    FieldRow::CodeDisplay,
                    view! {
                        {field("row-code", tr(Msg::Code), None, input("row-code", "SYN-110"))}
                        {field("row-display", tr(Msg::Display), None, input("row-display", &tr(Msg::SyntheticFindingArm)))}
                        {up()}
                        {remove()}
                    },
                )}
                {field_row(
                    FieldRow::CodeDisplayDefinition,
                    view! {
                        {field("row-code-2", tr(Msg::Code), None, input("row-code-2", "SYN-120"))}
                        {field("row-display-2", tr(Msg::Display), None, input("row-display-2", &tr(Msg::SyntheticFindingLeg)))}
                        {field("row-definition-2", tr(Msg::Definition), None, input("row-definition-2", &tr(Msg::FindingInventedGallery)))}
                        {remove()}
                    },
                )}
                {field_row(
                    FieldRow::Halves,
                    view! {
                        {field("row-system", tr(Msg::System), None, input("row-system", "http://example.org/synthetic"))}
                        {field("row-version", tr(Msg::Version), None, input("row-version", "1.0.0"))}
                    },
                )}
            },
        ),
    )
    .into_any()
}

/// The stacks at each step of space.
fn stacks() -> AnyView {
    let lines = |gap: Gap, name: String| {
        view! {
            <div class="fui-well">
                {stack_at(gap, view! {
                    <p>{name}</p>
                    <p class="fui-muted">{tr(Msg::SecondLine)}</p>
                    <p class="fui-muted">{tr(Msg::ThirdLine)}</p>
                })}
            </div>
        }
    };
    card(
        tr(Msg::Stacks),
        stack_at(
            Gap::Default,
            view! {
                {lines(Gap::Tight, tr(Msg::Tight))}
                {lines(Gap::Default, tr(Msg::Default))}
                {lines(Gap::Loose, tr(Msg::Loose))}
                {lines(Gap::Section, tr(Msg::Section))}
            },
        ),
    )
    .into_any()
}

/// Lists whose items a layout class sets apart, a grid and a stack, which
/// draw no marker and keep no indent.
fn laid_out_lists() -> AnyView {
    let steps = [
        tr(Msg::Tight),
        tr(Msg::Default),
        tr(Msg::Loose),
        tr(Msg::Section),
    ];
    let tiles = steps
        .iter()
        .map(|step| view! { <li class="fui-well">{step.clone()}</li> })
        .collect_view();
    let lines = steps
        .iter()
        .map(|step| view! { <li>{step.clone()}</li> })
        .collect_view();
    card(
        tr(Msg::ListsLaidOut),
        view! {
            <ul class="fui-grid">{tiles}</ul>
            <ul class="fui-stack fui-gap-tight">{lines}</ul>
        },
    )
    .into_any()
}

/// A fieldset that is a card's own child, right under the card's title,
/// which keeps the card's rhythm above it.
fn fieldset_in_card() -> AnyView {
    card(
        tr(Msg::FieldsetUnderTitle),
        view! {
            <fieldset class="fui-fieldset">
                <legend>{tr(Msg::TheCode)}</legend>
                {field(
                    "fieldset-code",
                    tr(Msg::Code),
                    None,
                    view! { <input id="fieldset-code" name="code" class=INPUT /> },
                )}
            </fieldset>
        },
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
        tr(Msg::Layouts),
        Some(tr(Msg::ThreePanesTreeListBeside)),
        (),
    )
    .into_any();
    let sections = (
        browser(),
        listing(),
        card(
            tr(Msg::MainAside),
            main_aside(runner_form(), runner_answer()),
        )
        .into_any(),
        rows(),
        stacks(),
        laid_out_lists(),
        fieldset_in_card(),
    );
    view! {
        {super::title(tr(Msg::Layouts))}
        {header}
        {stack_at(
            Gap::Loose,
            view! {
                {sections.0}
                {sections.1}
                {sections.2}
                {sections.3}
                {sections.4}
                {sections.5}
                {sections.6}
            },
        )}
    }
}
