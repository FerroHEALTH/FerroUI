// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The trees page.
//!
//! It shows the concept browser (M1) over a deep path with an inactive
//! concept, over a long level beside a level being read, and over a flat
//! code system; a directory's folder tree (E4) read and in an edit; and a
//! template's path browser (E5) with a coded node selected. Every tree is
//! the library's island `TreeView`. The first browser keeps its open items
//! and its concept in the address, so each fold is a link the server
//! answers with no WebAssembly.

use std::collections::BTreeSet;

use ferrohealth_ui::island::TreeView;
use ferrohealth_ui::table::filter::Find;
use ferrohealth_ui::table::model::StateSource;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::browser::{
    BrowserPane, Heading, Match, Panes, PathNode, browser, browser_find, match_list, pane_body,
    path_node,
};
use ferrohealth_ui::view::coding::{Coding, Layout, coding, inactive_mark};
use ferrohealth_ui::view::path::archetype;
use ferrohealth_ui::view::surface::{
    Crumb, Header, SectionHead, definitions, page_heading, panel_section, section_intro, stack,
    titled_panel,
};
use ferrohealth_ui::widget::tree::{Edit, Mark, More, TreeData, TreeKind, TreeNode, TreeWords};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::locale::{Msg, tr, tr_with};
use crate::trees_data::{
    COUNTRIES, COUNTRIES_VERSION, Concept, FINDINGS, FINDINGS_MATCHED, FINDINGS_OPEN,
    FINDINGS_SELECTED, FINDINGS_SYSTEM, FINDINGS_TOP, FINDINGS_VERSION, NATIONAL_CHEST,
    NATIONAL_LEVEL, NATIONAL_SKIN, NATIONAL_TOP, NATIONAL_VERSION, PARENTS, POSITION_CODES,
    POSITION_FLAT, POSITION_OCCURRENCES, POSITION_PATH, POSITION_TYPE, REFERENCES, ROOT_ARCHETYPE,
    TEMPLATE, TEMPLATE_OPEN, TEMPLATE_SELECTED, reason_code,
};

/// The key of the brand the route names, FerroTERM's for none.
fn brand_key() -> String {
    crate::app::route_brand()
        .map_or("ferroterm", |brand| brand.key())
        .to_owned()
}

/// The trees page of the brand `brand` with the open items `open` and the
/// concept `concept`.
fn address(brand: &str, open: &BTreeSet<String>, concept: &str) -> String {
    let open = open
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(",");
    format!("/{brand}/trees?open={open}&concept={concept}")
}

/// The synthetic finding `code`.
fn finding(code: &str) -> Option<Concept> {
    FINDINGS.into_iter().find(|(each, _, _)| *each == code)
}

/// The children of the finding `code`, in code order.
fn children_of(code: &str) -> Vec<&'static str> {
    PARENTS
        .into_iter()
        .filter(|(_, parent)| *parent == code)
        .map(|(child, _)| child)
        .collect()
}

/// The finding `code` as a tree item, its children read when `open` names
/// it, as a console's server reads one level at a time; each toggle leads
/// to the page with its item opened or closed.
fn deep_node(brand: &str, code: &str, open: &BTreeSet<String>, concept: &str) -> Option<TreeNode> {
    let (code, display, inactive) = finding(code)?;
    let mut node = TreeNode::new(code, tr(display))
        .code(code)
        .href(address(brand, open, code));
    if inactive {
        node = node.inactive();
    }
    let children = children_of(code);
    if children.is_empty() {
        return Some(node);
    }
    let mut flipped = open.clone();
    if !flipped.remove(code) {
        flipped.insert(code.to_owned());
    }
    node = node.toggle_href(address(brand, &flipped, concept));
    Some(if open.contains(code) {
        node.children(
            children
                .into_iter()
                .filter_map(|child| deep_node(brand, child, open, concept))
                .collect(),
        )
    } else {
        node.unread()
    })
}

/// The open items the address names, the walk to the selected concept when
/// it names none.
fn open_in(query: Option<String>) -> BTreeSet<String> {
    query.map_or_else(
        || FINDINGS_OPEN.into_iter().map(str::to_owned).collect(),
        |list| {
            list.split(',')
                .filter(|key| !key.is_empty())
                .map(str::to_owned)
                .collect()
        },
    )
}

/// A row of a search's matches: an inline M2 coding, its display the
/// row's link.
fn found(href: String, display: String, code: &str) -> AnyView {
    coding(Coding::new(code, display).href(href), Layout::Inline)
}

/// The search pane, searching for `value` in a field `id`, with `matches`.
fn search_pane(id: &'static str, value: &'static str, matches: Vec<Match>) -> BrowserPane {
    let label = tr(Msg::TreesSearchDesignations);
    let field = browser_find(Find {
        id,
        name: "q",
        label: &label,
        value,
        source: StateSource::Url,
        action: None,
        fields: &[],
    });
    if matches.is_empty() {
        return BrowserPane::new(
            tr(Msg::TreesSearch),
            pane_body(view! {
                {field}
                <p class="fui-small fui-muted">{tr(Msg::TreesSearchToSee)}</p>
            }),
        );
    }
    let count = tr_with(Msg::TreesConceptsFound, &[("count", matches.len().into())]);
    BrowserPane::new(
        tr(Msg::TreesSearch),
        view! {
            {pane_body(view! {
                {field}
                <p class="fui-small fui-muted" role="status">{count}</p>
            })}
            {match_list(tr(Msg::TreesMatched), matches)}
        },
    )
}

/// The concept pane of the concept `code` named `display`, read from
/// version `version`, with its `facts`.
fn concept_pane(
    display: String,
    code: &str,
    version: &str,
    inactive: bool,
    facts: Vec<(String, AnyView)>,
) -> BrowserPane {
    let read = tr_with(Msg::TreesReadFrom, &[("version", version.into())]);
    let mark = inactive.then(inactive_mark);
    BrowserPane::new(
        tr(Msg::TreesTheConcept),
        pane_body(view! {
            <h3 class="fui-concept-title">{display}</h3>
            <p class="fui-small fui-muted">
                <code>{code.to_owned()}</code>
                " · "
                {read}
                " "
                {mark}
            </p>
            {definitions(facts)}
        }),
    )
}

/// The browser over a deep path, its open items and concept from the
/// address.
fn deep_browser() -> AnyView {
    let brand = brand_key();
    let query = use_query_map();
    let concept = query
        .with_untracked(|query| query.get("concept"))
        .filter(|code| finding(code).is_some())
        .unwrap_or_else(|| String::from(FINDINGS_SELECTED));
    let open = open_in(query.with_untracked(|query| query.get("open")));
    let matches = FINDINGS_MATCHED
        .into_iter()
        .filter_map(finding)
        .map(|(code, display, _)| Match {
            body: found(address(&brand, &open, code), tr(display), code),
            selected: code == concept,
        })
        .collect();
    let nodes = deep_node(&brand, FINDINGS_TOP, &open, &concept)
        .into_iter()
        .collect();
    let tree = view! {
        <TreeView
            tree=TreeData {
                id: String::from("trees-deep"),
                label: tr(Msg::TreesHierarchyOfFindings),
                kind: TreeKind::Concept,
                editing: false,
                nodes,
                open: open.clone(),
                selected: Some(concept.clone()),
                open_field: None,
            }
            words=TreeWords::of_page()
        />
    };
    let (display, inactive) = finding(&concept)
        .map_or((String::new(), false), |(_, display, inactive)| {
            (tr(display), inactive)
        });
    let parent = PARENTS
        .into_iter()
        .find(|(child, _)| *child == concept)
        .and_then(|(_, parent)| finding(parent));
    let mut facts = vec![(
        tr(Msg::TreesSystem),
        view! { <code>{FINDINGS_SYSTEM}</code> }.into_any(),
    )];
    if let Some((code, display, _)) = parent {
        facts.push((
            tr(Msg::TreesParent),
            found(address(&brand, &open, code), tr(display), code),
        ));
    }
    let children = children_of(&concept);
    if !children.is_empty() {
        facts.push((
            tr(Msg::TreesChildren),
            children.len().to_string().into_any(),
        ));
    }
    titled_panel(
        tr(Msg::TreesDeepPath),
        (),
        browser(
            Panes::Three,
            Heading::Two,
            vec![
                search_pane("trees-deep-find", "cough", matches),
                BrowserPane::new(tr(Msg::TreesHierarchy), tree),
                concept_pane(display, &concept, FINDINGS_VERSION, inactive, facts),
            ],
        ),
    )
    .into_any()
}

/// The browser over a long level, beside a level being read.
fn long_browser() -> AnyView {
    let (total, shown, next) = NATIONAL_LEVEL;
    let reasons = (1..=shown)
        .map(|index| {
            let code = reason_code(index);
            TreeNode::new(
                code.clone(),
                tr_with(Msg::TreesReason, &[("index", index.into())]),
            )
            .code(code)
        })
        .collect();
    let chest = TreeNode::new(NATIONAL_CHEST, tr(Msg::TreesChestReferral))
        .code(NATIONAL_CHEST)
        .paged(reasons, More::of_page(shown, total, next, "#trees-long"));
    let skin = TreeNode::new(NATIONAL_SKIN, tr(Msg::TreesSkinReferral))
        .code(NATIONAL_SKIN)
        .unread();
    let root = TreeNode::new(NATIONAL_TOP, tr(Msg::TreesNationalFinding))
        .code(NATIONAL_TOP)
        .children(vec![chest, skin]);
    let tree = view! {
        <TreeView
            tree=TreeData {
                id: String::from("trees-long"),
                label: tr(Msg::TreesHierarchyOfNational),
                kind: TreeKind::Concept,
                editing: false,
                nodes: vec![root],
                open: [NATIONAL_TOP, NATIONAL_CHEST, NATIONAL_SKIN]
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
                selected: Some(String::from(NATIONAL_CHEST)),
                open_field: None,
            }
            words=TreeWords::of_page()
        />
    };
    let facts = vec![
        (
            tr(Msg::TreesParent),
            found(
                String::from("#trees-long"),
                tr(Msg::TreesNationalFinding),
                NATIONAL_TOP,
            ),
        ),
        (tr(Msg::TreesChildren), total.to_string().into_any()),
    ];
    view! {
        <div id="trees-long">
            {titled_panel(
                tr(Msg::TreesLongLevel),
                (),
                browser(
                    Panes::Three,
                    Heading::Two,
                    vec![
                        search_pane("trees-long-find", "", Vec::new()),
                        BrowserPane::new(tr(Msg::TreesHierarchy), tree),
                        concept_pane(
                            tr(Msg::TreesChestReferral),
                            NATIONAL_CHEST,
                            NATIONAL_VERSION,
                            false,
                            facts,
                        ),
                    ],
                ),
            )}
        </div>
    }
    .into_any()
}

/// The browser over a flat code system: the search and the concept.
fn flat_browser() -> AnyView {
    let matches = COUNTRIES
        .into_iter()
        .enumerate()
        .map(|(at, (code, words))| Match {
            body: found(String::from("#trees-flat"), tr(words), code),
            selected: at == 0,
        })
        .collect();
    let (code, words) = COUNTRIES[0];
    let facts = vec![(tr(Msg::TreesDisplay), tr(words).into_any())];
    view! {
        <div id="trees-flat">
            {titled_panel(
                tr(Msg::TreesFlatSystem),
                (),
                browser(
                    Panes::Flat,
                    Heading::Two,
                    vec![
                        search_pane("trees-flat-find", "netherlands", matches),
                        concept_pane(tr(words), code, COUNTRIES_VERSION, false, facts),
                    ],
                ),
            )}
        </div>
    }
    .into_any()
}

/// A synthetic directory, in an edit when `editing` says so: a folder
/// renamed and a reference removed.
fn directory(editing: bool) -> Vec<TreeNode> {
    let [vitals, laboratory, summary] = REFERENCES.map(|(key, words, uid)| {
        TreeNode::new(key, tr(words))
            .mark(Mark::Reference)
            .code(uid)
            .href(String::from("#trees-directory"))
            .edits(vec![Edit::Remove])
    });
    let folder = |key: &str, words: Msg, count: String| {
        TreeNode::new(key, tr(words))
            .mark(Mark::Folder)
            .count(count)
            .edits(vec![
                Edit::AddFolder,
                Edit::AddReference,
                Edit::Rename,
                Edit::Remove,
            ])
    };
    let items = |count: u32| tr_with(Msg::TreesItems, &[("count", count.into())]);
    let mut summary = summary;
    let mut follow_up = folder("follow-up", Msg::TreesFollowUp, items(2));
    let mut admission = folder("admission", Msg::TreesAdmission, items(1));
    if editing {
        summary = summary.removed().edits(vec![Edit::Undo]);
        follow_up = follow_up.renaming(tr(Msg::TreesFollowUpRenamed));
        admission = admission.count(tr(Msg::TreesEmpty));
    }
    let episodes = folder(
        "episodes",
        Msg::TreesEpisodes,
        tr_with(Msg::TreesFolders, &[("count", 2.into())]),
    )
    .children(vec![
        follow_up.children(vec![vitals, laboratory]),
        admission.children(vec![summary]),
    ]);
    let correspondence = folder(
        "correspondence",
        Msg::TreesCorrespondence,
        tr(Msg::TreesEmpty),
    );
    vec![
        TreeNode::new("root", tr(Msg::TreesRoot))
            .mark(Mark::Folder)
            .edits(vec![Edit::AddFolder, Edit::AddReference])
            .children(vec![episodes, correspondence]),
    ]
}

/// The directory's folder tree as the island draws it, `id`, in an edit
/// when `editing` says so.
fn folder_tree(id: &str, editing: bool) -> AnyView {
    let open = ["root", "episodes", "follow-up", "admission"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    view! {
        <TreeView
            tree=TreeData {
                id: id.to_owned(),
                label: tr(Msg::TreesDirectoryOfEhr),
                kind: TreeKind::Folder,
                editing,
                nodes: directory(editing),
                open,
                selected: None,
                open_field: editing.then(|| String::from("open")),
            }
            words=TreeWords::of_page()
        />
    }
    .into_any()
}

/// A directory, read and in an edit.
fn directories() -> AnyView {
    let read = panel_section(
        Some(SectionHead {
            title: tr(Msg::TreesFoldersSection),
            count: None,
            actions: None,
        }),
        folder_tree("trees-directory", false),
    );
    // NOTE: the gallery holds no edit, so its form reads the page again with the action.
    let edit = panel_section(
        Some(SectionHead {
            title: tr(Msg::TreesInEdit),
            count: None,
            actions: None,
        }),
        view! {
            {section_intro(tr(Msg::TreesChangesStay))}
            <form method="get" action="#trees-directory-edit">
                {folder_tree("trees-directory-edit", true)}
            </form>
        },
    );
    view! {
        <section id="trees-directory" class="fui-card fui-panel" aria-label=tr(Msg::TreesDirectory)>
            {read}
            {edit}
        </section>
    }
    .into_any()
}

/// The node `key` of the synthetic template, with the nodes under it.
fn template_node(key: &str) -> Option<TreeNode> {
    let (key, _, words, rm_type) = TEMPLATE.into_iter().find(|(each, ..)| *each == key)?;
    let node = TreeNode::new(key, tr(words))
        .tag(rm_type)
        .href(format!("#{key}"));
    let children: Vec<TreeNode> = TEMPLATE
        .into_iter()
        .filter(|(_, parent, ..)| *parent == Some(key))
        .filter_map(|(child, ..)| template_node(child))
        .collect();
    Some(if children.is_empty() {
        node
    } else {
        node.children(children)
    })
}

/// The template's path browser, with the coded node Position selected.
fn path_browser() -> AnyView {
    let root = template_node(TEMPLATE[0].0);
    let label = tr(Msg::TreesFindNode);
    let find = browser_find(Find {
        id: "trees-path-find",
        name: "path",
        label: &label,
        value: "",
        source: StateSource::Url,
        action: None,
        fields: &[],
    });
    let tree = view! {
        <TreeView
            tree=TreeData {
                id: String::from("trees-paths"),
                label: tr(Msg::TreesPathsOf),
                kind: TreeKind::Path,
                editing: false,
                nodes: root.into_iter().collect(),
                open: TEMPLATE_OPEN.into_iter().map(str::to_owned).collect(),
                selected: Some(String::from(TEMPLATE_SELECTED)),
                open_field: None,
            }
            words=TreeWords::of_page()
        />
    };
    let codes = POSITION_CODES
        .into_iter()
        .map(|(words, code)| coding(Coding::new(code, tr(words)), Layout::Inline))
        .collect();
    let node = path_node(PathNode {
        name: tr(Msg::TreesPosition),
        path: String::from(POSITION_PATH),
        rm_type: String::from(POSITION_TYPE),
        occurrences: String::from(POSITION_OCCURRENCES),
        units: None,
        flat_path: Some(String::from(POSITION_FLAT)),
        codes,
        query: String::from("#trees-paths"),
    });
    let template = panel_section(
        None,
        definitions(vec![(
            tr(Msg::TreesRootArchetype),
            archetype(ROOT_ARCHETYPE),
        )]),
    );
    titled_panel(
        tr(Msg::TreesTemplatePaths),
        (),
        view! {
            {browser(
                Panes::Two,
                Heading::Three,
                vec![BrowserPane::new(tr(Msg::TreesPaths), pane_body(view! { {find} {tree} })), node],
            )}
            {template}
        },
    )
    .into_any()
}

/// The trees page.
#[component]
pub fn TreesPage() -> impl IntoView {
    let root =
        crate::app::route_brand().map_or_else(String::new, |brand| format!("/{}", brand.key()));
    let header = page_heading(
        Header {
            title: tr(Msg::Trees),
            subtitle: Some(tr(Msg::TreesSubtitle)),
            crumbs: vec![Crumb::new(tr(Msg::Gallery), root)],
            mono: false,
            badge: None,
        },
        ActionGroup::none(),
    )
    .into_any();
    let sections = (
        deep_browser(),
        long_browser(),
        flat_browser(),
        directories(),
        path_browser(),
    );
    view! {
        {super::title(tr(Msg::Trees))}
        {header}
        {stack(
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
