// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The layouts page.
//!
//! It shows three reading panes around the tree view, a list beside the item
//! it opens, a runner's form beside its answer with the help switch and the
//! picker, rows of fields, and the stacks at each step.

use std::collections::BTreeSet;

use crate::locale::{Msg, Words, tr};
use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::change::{Change, change_table};
use ferrohealth_ui::view::chart::proportion;
use ferrohealth_ui::view::code::{Code, CodeHead, Gravity, LineDiagnostic, code_section};
use ferrohealth_ui::view::form::{FORM, INPUT, MEDIUM, NARROW, PROSE, SELECT, field, help_id};
use ferrohealth_ui::view::help::{HelpWords, explained_check_field, explained_field, help_switch};
use ferrohealth_ui::view::layout::{
    FieldRow, Gap, Pane, bullet_list, divided_list, field_row, main_aside, master_detail, stack_at,
    three_panes, tile_list,
};
use ferrohealth_ui::view::status::pill;
use ferrohealth_ui::view::surface::{
    SectionHead, add_row, card, definitions, or_none, page_header, panel_section,
    panel_sections_card, section_intro, split,
};
use ferrohealth_ui::widget::picker::{Picker, picker_field};
use ferrohealth_ui::widget::tree::{
    Children, Tree, TreeKind, TreeNode, TreeWords, path_key, tree_view,
};
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

/// The synthetic hierarchy in `words`: the top read at once, a branch read
/// when it opens.
fn hierarchy(brand: &str, words: &Words) -> Vec<TreeNode> {
    vec![
        concept(brand, "SYN-100", &words.get(Msg::SyntheticFinding)).children(vec![
            concept(brand, "SYN-110", &words.get(Msg::SyntheticFindingArm)),
            concept(brand, "SYN-120", &words.get(Msg::SyntheticFindingLeg)).unread(),
        ]),
        concept(brand, "SYN-200", &words.get(Msg::SyntheticProcedure)).unread(),
        concept(brand, "SYN-300", &words.get(Msg::SyntheticBodySite)),
    ]
}

/// The children in `words` read under each item of `open` not read yet,
/// as a console's server reads a level when it opens; `above` holds the
/// keys of the items over `nodes`.
fn read_children(
    brand: &str,
    words: &Words,
    nodes: &mut [TreeNode],
    above: &[String],
    open: &BTreeSet<String>,
) {
    for node in nodes.iter_mut() {
        let keys = [above, std::slice::from_ref(&node.key)].concat();
        if node.children == Children::Unread && open.contains(&path_key(&keys)) {
            let key = node.key.clone();
            node.children = Children::Read {
                nodes: vec![
                    concept(
                        brand,
                        &format!("{key}-1"),
                        &words.get(Msg::SyntheticChildReadOpening),
                    ),
                    concept(
                        brand,
                        &format!("{key}-2"),
                        &words.get(Msg::AnotherSyntheticChild),
                    ),
                ],
                more: None,
            };
        } else if let Children::Read { nodes, .. } = &mut node.children {
            read_children(brand, words, nodes, &keys, open);
        }
    }
}

/// The hierarchy of the brand `brand` in `words` with the items `open`
/// read, each toggle a link to the page with its item opened or closed.
fn read_hierarchy(
    brand: &str,
    words: &Words,
    open: &BTreeSet<String>,
    selected: Option<&str>,
) -> Vec<TreeNode> {
    fn toggles(
        brand: &str,
        nodes: &mut [TreeNode],
        above: &[String],
        open: &BTreeSet<String>,
        selected: Option<&str>,
    ) {
        for node in nodes.iter_mut() {
            let keys = [above, std::slice::from_ref(&node.key)].concat();
            if node.opens() {
                let key = path_key(&keys);
                let mut flipped = open.clone();
                if !flipped.remove(&key) {
                    flipped.insert(key);
                }
                node.toggle_href = Some(address(brand, &flipped, selected));
            }
            if let Children::Read { nodes, .. } = &mut node.children {
                toggles(brand, nodes, &keys, open, selected);
            }
        }
    }
    let mut nodes = hierarchy(brand, words);
    read_children(brand, words, &mut nodes, &[], open);
    toggles(brand, &mut nodes, &[], open, selected);
    nodes
}

/// The keys from the top to the first item among `nodes` whose key is
/// `key`.
fn keys_to(nodes: &[TreeNode], key: &str) -> Option<Vec<String>> {
    nodes.iter().find_map(|node| {
        if node.key == key {
            Some(vec![node.key.clone()])
        } else if let Children::Read { nodes, .. } = &node.children {
            keys_to(nodes, key).map(|below| [vec![node.key.clone()], below].concat())
        } else {
            None
        }
    })
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
    let tree = view! { <crate::islands::ConceptTree
        brand
        concept
        open
        words=tree_words()
        tree=TreeWords::of_page()
    /> };
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

/// The words of the concept tree's island.
fn tree_words() -> Words {
    Words::of(&[
        Msg::SyntheticFinding,
        Msg::SyntheticFindingArm,
        Msg::SyntheticFindingLeg,
        Msg::SyntheticProcedure,
        Msg::SyntheticBodySite,
        Msg::SyntheticChildReadOpening,
        Msg::AnotherSyntheticChild,
        Msg::SyntheticHierarchy,
    ])
}

/// The words of the runner form's island.
fn runner_words() -> Words {
    Words::of(&[
        Msg::SyntheticSigns,
        Msg::SyntheticSites,
        Msg::SyntheticUnits,
        Msg::UseFirstValueSet,
        Msg::ValueSet,
        Msg::ChooseValueSetServerHolds,
        Msg::CanonicalValueSetList,
        Msg::ChooseOneServerHolds,
        Msg::AnotherCanonical,
        Msg::English,
        Msg::Dutch,
        Msg::ListValueSet,
        Msg::PageSize,
        Msg::HowManyConceptsOnePage,
        Msg::DisplayLanguage,
        Msg::LanguageDisplays,
        Msg::WhyYouRun,
        Msg::NoteKeptBesideRun,
        Msg::IncludeInactiveConcepts,
        Msg::InactiveConceptListedMarked,
        Msg::List,
    ])
}

/// The hierarchy of the brand `brand` in `words` and `tree_words`, with the concept
/// `concept` selected and the items `open` names open, keeping its open items in the address
/// as they change.
///
/// The tree's items follow its open items, each level read as it opens,
/// and an `Effect::watch` keeps the address in step (the Leptos book,
/// `reactivity/14_create_effect`).
pub(crate) fn concept_tree(
    brand: String,
    concept: Option<String>,
    open: Option<String>,
    words: Words,
    tree_words: TreeWords,
) -> AnyView {
    let label = words.get(Msg::SyntheticHierarchy);
    let expanded = RwSignal::new(open_in(open));
    let brand = StoredValue::new(brand);
    let words = StoredValue::new(words);
    let chosen = StoredValue::new(concept);
    let nodes = Memo::new(move |_| {
        expanded.with(|open| {
            brand.with_value(|brand| {
                words.with_value(|words| {
                    chosen
                        .with_value(|chosen| read_hierarchy(brand, words, open, chosen.as_deref()))
                })
            })
        })
    });
    Effect::watch(
        move || expanded.get(),
        move |open, _, _| {
            let url = brand.with_value(|brand| {
                chosen.with_value(|chosen| address(brand, open, chosen.as_deref()))
            });
            replace_address(&url);
        },
        false,
    );
    tree_view(Tree {
        id: String::from("gallery-tree"),
        label,
        kind: TreeKind::Concept,
        editing: false,
        nodes: nodes.into(),
        expanded,
        selected: Signal::derive(move || {
            chosen.with_value(|chosen| {
                chosen
                    .as_deref()
                    .and_then(|code| nodes.with(|nodes| keys_to(nodes, code)))
                    .map(|keys| path_key(&keys))
            })
        }),
        open_field: None,
        words: tree_words,
    })
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
/// The picker and the button that resets it share one state, so the form
/// is the island [`crate::islands::RunnerForm`]; the help switch needs no
/// script.
fn runner_form() -> AnyView {
    let brand = brand_key();
    view! { <crate::islands::RunnerForm brand words=runner_words() help=HelpWords::of_page() /> }
        .into_any()
}

/// The runner's form on the layouts page of the brand `brand`, in `words`
/// and the help switch's `help`.
pub(crate) fn runner_form_in(brand: &str, words: &Words, help: HelpWords) -> AnyView {
    let options = vec![
        (
            String::from("http://example.org/vs/signs"),
            words.get(Msg::SyntheticSigns),
        ),
        (
            String::from("http://example.org/vs/sites"),
            words.get(Msg::SyntheticSites),
        ),
        (
            String::from("http://example.org/vs/units"),
            words.get(Msg::SyntheticUnits),
        ),
    ];
    let choices = Signal::stored(options);
    let first = String::from("http://example.org/vs/signs");
    let value = RwSignal::new(first.clone());
    // The console's side: a value set from outside, which the picker's
    // select follows after the reader has chosen another.
    let reset = view! {
        <button
            type="button"
            id="run-value-set-reset"
            class=Variant::Secondary.class()
            on:click=move |_| value.set(first.clone())
        >
            {words.get(Msg::UseFirstValueSet)}
        </button>
    };
    let picker = picker_field(Picker {
        id: String::from("run-value-set"),
        name: String::from("url"),
        label: words.get(Msg::ValueSet).into(),
        choose_label: words.get(Msg::ChooseValueSetServerHolds).into(),
        help: Some(words.get(Msg::CanonicalValueSetList).into()),
        choices,
        value,
        empty_word: words.get(Msg::ChooseOneServerHolds).into(),
        other_word: words.get(Msg::AnotherCanonical).into(),
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
            <option value="en">{words.get(Msg::English)}</option>
            <option value="nl">{words.get(Msg::Dutch)}</option>
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
            {split(view! { <h2 class="fui-card-title">{words.get(Msg::ListValueSet)}</h2> }, help_switch("run-help", help))}
            {picker}
            <div>{reset}</div>
            {explained_field("run-count", words.get(Msg::PageSize), words.get(Msg::HowManyConceptsOnePage), count)}
            {explained_field("run-language", words.get(Msg::DisplayLanguage), words.get(Msg::LanguageDisplays), language)}
            {explained_field("run-note", words.get(Msg::WhyYouRun), words.get(Msg::NoteKeptBesideRun), note)}
            {explained_check_field("run-inactive", words.get(Msg::IncludeInactiveConcepts), words.get(Msg::InactiveConceptListedMarked), inactive)}
            <button type="submit" class=Variant::Primary.class()>
                {label(Some(Glyph::PLAY), words.get(Msg::List))}
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

/// A panel of three sections (B10): a table, a list and a code viewer,
/// each running to the panel's edges under its heading row.
fn sections_panel() -> AnyView {
    let staged = [
        Change {
            key: String::from("members.node-a.base"),
            from: Some(String::from("https://node-a.example.org/")),
            to: Some(String::from("https://node-a2.example.org/")),
        },
        Change {
            key: String::from("routing.timeout"),
            from: Some(String::from("30s")),
            to: None,
        },
    ];
    let table = panel_section(
        Some(SectionHead {
            title: tr(Msg::StagedChanges),
            count: Some(2),
            actions: None,
        }),
        change_table(tr(Msg::StagedConfigurationAgainstRunningOne), &staged),
    );
    let steps = [
        tr(Msg::Tight),
        tr(Msg::Default),
        tr(Msg::Loose),
        tr(Msg::Section),
    ]
    .into_iter()
    .map(IntoAny::into_any)
    .collect();
    let list = panel_section(
        Some(SectionHead {
            title: tr(Msg::SpacingSteps),
            count: Some(4),
            actions: None,
        }),
        view! {
            {section_intro(tr(Msg::PanelSectionsIntro))}
            {divided_list(None, steps)}
            {add_row(
                view! {
                    <button type="button" class=Variant::Ghost.small()>
                        {tr(Msg::AddStep)}
                    </button>
                },
            )}
        },
    );
    let [path, ..] = crate::data::FILE_NAMES;
    let source = code_section(
        CodeHead {
            title: String::from(path),
            tags: vec![tr(Msg::Yaml)],
            ..CodeHead::default()
        },
        &Code {
            label: tr(Msg::MappingFileYaml),
            language: Language::Yaml,
            text: String::from(crate::data::YAML),
            ids: None,
            current: None,
            diagnostics: vec![LineDiagnostic {
                line: 19,
                gravity: Gravity::Warning,
                code: String::from(crate::data::FINDING_CODES[1]),
                message: tr(Msg::FindingStartTwice),
            }],
        },
    );
    panel_sections_card(
        tr(Msg::PanelSections),
        view! {
            {table}
            {list}
            {source}
        },
    )
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
        ActionGroup::none(),
    )
    .into_any();
    let sections = (
        sections_panel(),
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
                {sections.7}
                {super::records::timeline_section()}
            },
        )}
    }
}
