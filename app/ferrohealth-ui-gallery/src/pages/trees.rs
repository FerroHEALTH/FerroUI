// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The trees page.
//!
//! It shows the concept browser (M1) over a deep path with an inactive
//! concept, over a long level beside a level being read, and over a flat
//! code system; a directory's folder tree (E4) read and in an edit; and a
//! template's path browser (E5) with a coded node selected. Every tree is
//! the library's `tree_view`. The first browser keeps its open items and
//! its concept in the address, so each fold is a link the server answers
//! with no script.

use std::collections::BTreeSet;

use ferrohealth_ui::html::Html;
use ferrohealth_ui::table::filter::Find;
use ferrohealth_ui::table::model::StateSource;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::browser::{
    BrowserPane, Heading, Panes, PathNode, browser, browser_find, match_list, pane_body, path_node,
};
use ferrohealth_ui::view::coding::{Coding, Layout, coding, inactive_mark};
use ferrohealth_ui::view::path::archetype;
use ferrohealth_ui::view::surface::{
    Crumb, Header, SectionHead, definitions, page_heading, panel_section, section_intro, stack,
    titled_panel,
};
use ferrohealth_ui::view::tree::{
    Edit, Mark, More, Tree, TreeKind, TreeNode, TreeWords, path_key, tree_view,
};

use crate::app::Page;
use crate::locale::Msg;
use crate::markup::{h, t};
use crate::trees_data::{
    COUNTRIES, COUNTRIES_VERSION, Concept, FINDINGS, FINDINGS_MATCHED, FINDINGS_OPEN,
    FINDINGS_SELECTED, FINDINGS_SYSTEM, FINDINGS_TOP, FINDINGS_VERSION, NATIONAL_CHEST,
    NATIONAL_LEVEL, NATIONAL_SKIN, NATIONAL_TOP, NATIONAL_VERSION, PARENTS, POSITION_CODES,
    POSITION_FLAT, POSITION_OCCURRENCES, POSITION_PATH, POSITION_TYPE, REFERENCES, ROOT_ARCHETYPE,
    TEMPLATE, TEMPLATE_OPEN, TEMPLATE_SELECTED, reason_code,
};
use crate::visit::Visit;

/// The key of the brand the route names, FerroTERM's for none.
fn brand_key(v: &Visit) -> String {
    v.brand.key().to_owned()
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

/// The codes from the top finding to the finding `code`.
fn walk_to(code: &str) -> Vec<&str> {
    let mut walk = vec![code];
    let mut at = code;
    while let Some((_, parent)) = PARENTS.into_iter().find(|(child, _)| *child == at) {
        walk.insert(0, parent);
        at = parent;
    }
    walk
}

/// The path key of the finding `code` in the tree.
fn path_of(code: &str) -> String {
    path_key(&walk_to(code))
}

/// The finding `code` under the findings `above` as a tree item, its
/// children read when `open` names its path key, as a console's server
/// reads one level at a time; each toggle leads to the page with its item
/// opened or closed.
fn deep_node(
    v: &Visit,
    brand: &str,
    above: &[&str],
    code: &str,
    open: &BTreeSet<String>,
    concept: &str,
) -> Option<TreeNode> {
    let (code, display, inactive) = finding(code)?;
    let keys = [above, &[code]].concat();
    let key = path_key(&keys);
    let mut node = TreeNode::new(code, v.tr(display))
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
    if !flipped.remove(&key) {
        flipped.insert(key.clone());
    }
    node = node.toggle_href(address(brand, &flipped, concept));
    Some(if open.contains(&key) {
        node.children(
            children
                .into_iter()
                .filter_map(|child| deep_node(v, brand, &keys, child, open, concept))
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
        || FINDINGS_OPEN.into_iter().map(path_of).collect(),
        |list| {
            list.split(',')
                .filter(|key| !key.is_empty())
                .map(str::to_owned)
                .collect()
        },
    )
}

/// A row of a search's matches: an inline M2 coding, its display the
/// row's link, marked current when it is the concept in view.
fn found(v: &Visit, href: String, display: String, code: &str, current: bool) -> Html {
    coding(
        &v.locale,
        Coding::new(code, display).href(href).current(current),
        Layout::Inline,
    )
}

/// The search pane, searching for `value` in a field `id`, with `matches`.
fn search_pane(
    v: &Visit,
    id: &'static str,
    value: &'static str,
    matches: Vec<Html>,
) -> BrowserPane {
    let label = v.tr(Msg::TreesSearchDesignations);
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
            v.tr(Msg::TreesSearch),
            pane_body(h(format!(
                "{field}<p class=\"fui-small fui-muted\">{}</p>",
                t(v.tr(Msg::TreesSearchToSee))
            ))),
        );
    }
    let count = v.tr_with(Msg::TreesConceptsFound, &[("count", matches.len().into())]);
    BrowserPane::new(
        v.tr(Msg::TreesSearch),
        Html::join([
            pane_body(h(format!(
                "{field}<p class=\"fui-small fui-muted\" role=\"status\">{}</p>",
                t(count)
            ))),
            match_list(&v.tr(Msg::TreesMatched), matches),
        ]),
    )
}

/// The concept pane of the concept `code` named `display`, read from
/// version `version`, with its `facts`.
fn concept_pane(
    v: &Visit,
    display: String,
    code: &str,
    version: &str,
    inactive: bool,
    facts: Vec<(String, Html)>,
) -> BrowserPane {
    let read = v.tr_with(Msg::TreesReadFrom, &[("version", version.into())]);
    let mark = if inactive {
        inactive_mark(&v.locale)
    } else {
        Html::empty()
    };
    BrowserPane::new(
        v.tr(Msg::TreesTheConcept),
        pane_body(h(format!(
            "<h3 class=\"fui-concept-title\">{}</h3><p class=\"fui-small fui-muted\"><code>{}</code> · {} {mark}</p>{}",
            t(display),
            t(code),
            t(read),
            definitions(facts)
        ))),
    )
}

/// The browser over a deep path, its open items and concept from the
/// address.
fn deep_browser(v: &Visit) -> Html {
    let brand = brand_key(v);
    let concept = v
        .param("concept")
        .filter(|code| finding(code).is_some())
        .unwrap_or(FINDINGS_SELECTED)
        .to_owned();
    let open = open_in(v.param("open").map(str::to_owned));
    let matches = FINDINGS_MATCHED
        .into_iter()
        .filter_map(finding)
        .map(|(code, display, _)| {
            found(
                v,
                address(&brand, &open, code),
                v.tr(display),
                code,
                code == concept,
            )
        })
        .collect();
    let nodes = deep_node(v, &brand, &[], FINDINGS_TOP, &open, &concept)
        .into_iter()
        .collect();
    let tree = tree_view(&Tree {
        id: String::from("trees-deep"),
        label: v.tr(Msg::TreesHierarchyOfFindings),
        kind: TreeKind::Concept,
        editing: false,
        nodes,
        open: open.clone(),
        selected: Some(path_of(&concept)),
        open_field: None,
        words: TreeWords::of(&v.locale),
    });
    let (display, inactive) = finding(&concept)
        .map_or((String::new(), false), |(_, display, inactive)| {
            (v.tr(display), inactive)
        });
    let parent = PARENTS
        .into_iter()
        .find(|(child, _)| *child == concept)
        .and_then(|(_, parent)| finding(parent));
    let mut facts = vec![(
        v.tr(Msg::TreesSystem),
        h(format!("<code>{FINDINGS_SYSTEM}</code>")),
    )];
    if let Some((code, display, _)) = parent {
        facts.push((
            v.tr(Msg::TreesParent),
            found(v, address(&brand, &open, code), v.tr(display), code, false),
        ));
    }
    let children = children_of(&concept);
    if !children.is_empty() {
        facts.push((v.tr(Msg::TreesChildren), t(children.len().to_string())));
    }
    titled_panel(
        v.tr(Msg::TreesDeepPath),
        Html::empty(),
        browser(
            Panes::Three,
            Heading::Two,
            vec![
                search_pane(v, "trees-deep-find", "cough", matches),
                BrowserPane::new(v.tr(Msg::TreesHierarchy), tree),
                concept_pane(v, display, &concept, FINDINGS_VERSION, inactive, facts),
            ],
        ),
    )
}

/// The browser over a long level, beside a level being read.
fn long_browser(v: &Visit) -> Html {
    let (total, shown, next) = NATIONAL_LEVEL;
    let reasons = (1..=shown)
        .map(|index| {
            let code = reason_code(index);
            TreeNode::new(
                code.clone(),
                v.tr_with(Msg::TreesReason, &[("index", index.into())]),
            )
            .code(code)
        })
        .collect();
    let chest = TreeNode::new(NATIONAL_CHEST, v.tr(Msg::TreesChestReferral))
        .code(NATIONAL_CHEST)
        .paged(
            reasons,
            More::of(&v.locale, shown, total, next, "#trees-long"),
        );
    let skin = TreeNode::new(NATIONAL_SKIN, v.tr(Msg::TreesSkinReferral))
        .code(NATIONAL_SKIN)
        .unread();
    let root = TreeNode::new(NATIONAL_TOP, v.tr(Msg::TreesNationalFinding))
        .code(NATIONAL_TOP)
        .children(vec![chest, skin]);
    let tree = tree_view(&Tree {
        id: String::from("trees-long"),
        label: v.tr(Msg::TreesHierarchyOfNational),
        kind: TreeKind::Concept,
        editing: false,
        nodes: vec![root],
        open: [
            path_key(&[NATIONAL_TOP]),
            path_key(&[NATIONAL_TOP, NATIONAL_CHEST]),
            path_key(&[NATIONAL_TOP, NATIONAL_SKIN]),
        ]
        .into_iter()
        .collect(),
        selected: Some(path_key(&[NATIONAL_TOP, NATIONAL_CHEST])),
        open_field: None,
        words: TreeWords::of(&v.locale),
    });
    let facts = vec![
        (
            v.tr(Msg::TreesParent),
            found(
                v,
                String::from("#trees-long"),
                v.tr(Msg::TreesNationalFinding),
                NATIONAL_TOP,
                false,
            ),
        ),
        (v.tr(Msg::TreesChildren), t(total.to_string())),
    ];
    h(format!(
        "<div id=\"trees-long\">{}</div>",
        titled_panel(
            v.tr(Msg::TreesLongLevel),
            Html::empty(),
            browser(
                Panes::Three,
                Heading::Two,
                vec![
                    search_pane(v, "trees-long-find", "", Vec::new()),
                    BrowserPane::new(v.tr(Msg::TreesHierarchy), tree),
                    concept_pane(
                        v,
                        v.tr(Msg::TreesChestReferral),
                        NATIONAL_CHEST,
                        NATIONAL_VERSION,
                        false,
                        facts,
                    ),
                ],
            ),
        )
    ))
}

/// The browser over a flat code system: the search and the concept.
fn flat_browser(v: &Visit) -> Html {
    let matches = COUNTRIES
        .into_iter()
        .enumerate()
        .map(|(at, (code, words))| {
            found(v, String::from("#trees-flat"), v.tr(words), code, at == 0)
        })
        .collect();
    let (code, words) = COUNTRIES[0];
    let facts = vec![(v.tr(Msg::TreesDisplay), t(v.tr(words)))];
    h(format!(
        "<div id=\"trees-flat\">{}</div>",
        titled_panel(
            v.tr(Msg::TreesFlatSystem),
            Html::empty(),
            browser(
                Panes::Flat,
                Heading::Two,
                vec![
                    search_pane(v, "trees-flat-find", "netherlands", matches),
                    concept_pane(v, v.tr(words), code, COUNTRIES_VERSION, false, facts),
                ],
            ),
        )
    ))
}

/// A synthetic directory, in an edit when `editing` says so: a folder
/// renamed and a reference removed.
fn directory(v: &Visit, editing: bool) -> Vec<TreeNode> {
    let [vitals, laboratory, summary] = REFERENCES.map(|(key, words, uid)| {
        TreeNode::new(key, v.tr(words))
            .mark(Mark::Reference)
            .code(uid)
            .href(String::from("#trees-directory"))
            .edits(vec![Edit::Remove])
    });
    let folder = |key: &str, words: Msg, count: String| {
        TreeNode::new(key, v.tr(words))
            .mark(Mark::Folder)
            .count(count)
            .edits(vec![
                Edit::AddFolder,
                Edit::AddReference,
                Edit::Rename,
                Edit::Remove,
            ])
    };
    let items = |count: u32| v.tr_with(Msg::TreesItems, &[("count", count.into())]);
    let mut summary = summary;
    let mut follow_up = folder("follow-up", Msg::TreesFollowUp, items(2));
    let mut admission = folder("admission", Msg::TreesAdmission, items(1));
    if editing {
        summary = summary.removed().edits(vec![Edit::Undo]);
        follow_up = follow_up.renaming(v.tr(Msg::TreesFollowUpRenamed));
        admission = admission.count(v.tr(Msg::TreesEmpty));
    }
    let episodes = folder(
        "episodes",
        Msg::TreesEpisodes,
        v.tr_with(Msg::TreesFolders, &[("count", 2.into())]),
    )
    .children(vec![
        follow_up.children(vec![vitals, laboratory]),
        admission.children(vec![summary]),
    ]);
    let correspondence = folder(
        "correspondence",
        Msg::TreesCorrespondence,
        v.tr(Msg::TreesEmpty),
    );
    vec![
        TreeNode::new("root", v.tr(Msg::TreesRoot))
            .mark(Mark::Folder)
            .edits(vec![Edit::AddFolder, Edit::AddReference])
            .children(vec![episodes, correspondence]),
    ]
}

/// The directory's folder tree as the module draws it, `id`, in an edit
/// when `editing` says so.
fn folder_tree(v: &Visit, id: &str, editing: bool) -> Html {
    let open = [
        &["root"][..],
        &["root", "episodes"],
        &["root", "episodes", "follow-up"],
        &["root", "episodes", "admission"],
    ]
    .into_iter()
    .map(path_key)
    .collect();
    tree_view(&Tree {
        id: id.to_owned(),
        label: v.tr(Msg::TreesDirectoryOfEhr),
        kind: TreeKind::Folder,
        editing,
        nodes: directory(v, editing),
        open,
        selected: None,
        open_field: editing.then(|| String::from("open")),
        words: TreeWords::of(&v.locale),
    })
}

/// A directory, read and in an edit.
fn directories(v: &Visit) -> Html {
    let read = panel_section(
        &v.locale,
        Some(SectionHead {
            title: v.tr(Msg::TreesFoldersSection),
            count: None,
            actions: None,
        }),
        folder_tree(v, "trees-directory", false),
    );
    // NOTE: the gallery holds no edit, so its form reads the page again with the action.
    let edit = panel_section(
        &v.locale,
        Some(SectionHead {
            title: v.tr(Msg::TreesInEdit),
            count: None,
            actions: None,
        }),
        h(format!(
            "{}<form method=\"get\" action=\"#trees-directory-edit\">{}</form>",
            section_intro(v.tr(Msg::TreesChangesStay)),
            folder_tree(v, "trees-directory-edit", true)
        )),
    );
    h(format!(
        "<section id=\"trees-directory\" class=\"fui-card fui-panel\" aria-label=\"{}\">{read}{edit}</section>",
        t(v.tr(Msg::TreesDirectory))
    ))
}

/// The node `key` of the synthetic template, with the nodes under it.
fn template_node(v: &Visit, key: &str) -> Option<TreeNode> {
    let (key, _, words, rm_type) = TEMPLATE.into_iter().find(|(each, ..)| *each == key)?;
    let node = TreeNode::new(key, v.tr(words))
        .tag(rm_type)
        .href(format!("#{key}"));
    let children: Vec<TreeNode> = TEMPLATE
        .into_iter()
        .filter(|(_, parent, ..)| *parent == Some(key))
        .filter_map(|(child, ..)| template_node(v, child))
        .collect();
    Some(if children.is_empty() {
        node
    } else {
        node.children(children)
    })
}

/// The template's path browser, with the coded node Position selected.
fn path_browser(v: &Visit) -> Html {
    let root = template_node(v, TEMPLATE[0].0);
    let label = v.tr(Msg::TreesFindNode);
    let find = browser_find(Find {
        id: "trees-path-find",
        name: "path",
        label: &label,
        value: "",
        source: StateSource::Url,
        action: None,
        fields: &[],
    });
    let tree = tree_view(&Tree {
        id: String::from("trees-paths"),
        label: v.tr(Msg::TreesPathsOf),
        kind: TreeKind::Path,
        editing: false,
        nodes: root.into_iter().collect(),
        open: TEMPLATE_OPEN.into_iter().map(str::to_owned).collect(),
        selected: Some(String::from(TEMPLATE_SELECTED)),
        open_field: None,
        words: TreeWords::of(&v.locale),
    });
    let codes = POSITION_CODES
        .into_iter()
        .map(|(words, code)| coding(&v.locale, Coding::new(code, v.tr(words)), Layout::Inline))
        .collect();
    let node = path_node(
        &v.locale,
        PathNode {
            name: v.tr(Msg::TreesPosition),
            path: String::from(POSITION_PATH),
            rm_type: String::from(POSITION_TYPE),
            occurrences: String::from(POSITION_OCCURRENCES),
            units: None,
            flat_path: Some(String::from(POSITION_FLAT)),
            codes,
            more: None,
            query: Some(String::from("#trees-paths")),
        },
    );
    let template = panel_section(
        &v.locale,
        None,
        definitions(vec![(
            v.tr(Msg::TreesRootArchetype),
            archetype(ROOT_ARCHETYPE),
        )]),
    );
    titled_panel(
        v.tr(Msg::TreesTemplatePaths),
        Html::empty(),
        Html::join([
            browser(
                Panes::Two,
                Heading::Three,
                vec![
                    BrowserPane::new(v.tr(Msg::TreesPaths), pane_body(Html::join([find, tree]))),
                    node,
                ],
            ),
            template,
        ]),
    )
}

/// The trees page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let root = format!("/{}", v.brand.key());
    let header = page_heading(
        &v.locale,
        Header {
            title: v.tr(Msg::Trees).into(),
            subtitle: Some(v.tr(Msg::TreesSubtitle).into()),
            subtitle_view: None,
            crumbs: vec![Crumb::new(v.tr(Msg::Gallery), root)],
            mono: false,
            badge: None,
        },
        ActionGroup::none(),
    );
    let body = stack(Html::join([
        deep_browser(v),
        long_browser(v),
        flat_browser(v),
        directories(v),
        path_browser(v),
    ]));
    Page::shell(v.tr(Msg::Trees), Html::join([header, body]))
}
