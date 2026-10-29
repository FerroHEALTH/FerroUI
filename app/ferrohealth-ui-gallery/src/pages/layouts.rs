// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The layouts page.
//!
//! It shows three reading panes around the tree view, a list beside the item
//! it opens, a runner's form beside its answer with the help switch and the
//! picker, rows of fields, and the stacks at each step.

use std::collections::BTreeSet;

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
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
use ferrohealth_ui::view::picker::{Picker, picker_field};
use ferrohealth_ui::view::status::pill;
use ferrohealth_ui::view::surface::{
    SectionHead, add_row, card, definitions, or_none, page_header, panel_section,
    panel_sections_card, section_intro, split,
};
use ferrohealth_ui::view::template::list::HeadingCount;
use ferrohealth_ui::view::tree::{
    Children, Tree, TreeKind, TreeNode, TreeWords, path_key, tree_view,
};

use crate::app::Page;
use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

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
fn hierarchy(v: &Visit, brand: &str) -> Vec<TreeNode> {
    vec![
        concept(brand, "SYN-100", &v.tr(Msg::SyntheticFinding)).children(vec![
            concept(brand, "SYN-110", &v.tr(Msg::SyntheticFindingArm)),
            concept(brand, "SYN-120", &v.tr(Msg::SyntheticFindingLeg)).unread(),
        ]),
        concept(brand, "SYN-200", &v.tr(Msg::SyntheticProcedure)).unread(),
        concept(brand, "SYN-300", &v.tr(Msg::SyntheticBodySite)),
    ]
}

/// The children read under each item of `open` not read yet, as a
/// console's server reads a level when it opens; `above` holds the keys of
/// the items over `nodes`.
fn read_children(
    v: &Visit,
    brand: &str,
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
                        &v.tr(Msg::SyntheticChildReadOpening),
                    ),
                    concept(
                        brand,
                        &format!("{key}-2"),
                        &v.tr(Msg::AnotherSyntheticChild),
                    ),
                ],
                more: None,
            };
        } else if let Children::Read { nodes, .. } = &mut node.children {
            read_children(v, brand, nodes, &keys, open);
        }
    }
}

/// The hierarchy of the brand `brand` with the items `open` read, each
/// toggle a link to the page with its item opened or closed.
fn read_hierarchy(
    v: &Visit,
    brand: &str,
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
    let mut nodes = hierarchy(v, brand);
    read_children(v, brand, &mut nodes, &[], open);
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
fn open_in(query: Option<&str>) -> BTreeSet<String> {
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

/// The hierarchy with the concept the address names selected and the
/// items it names open, each level read as it opens.
fn concept_tree(v: &Visit, brand: &str) -> Html {
    let chosen = v.param("concept");
    let open = open_in(v.param("open"));
    let nodes = read_hierarchy(v, brand, &open, chosen);
    let selected = chosen
        .and_then(|code| keys_to(&nodes, code))
        .map(|keys| path_key(&keys));
    tree_view(&Tree {
        id: String::from("gallery-tree"),
        label: v.tr(Msg::SyntheticHierarchy),
        kind: TreeKind::Concept,
        editing: false,
        nodes,
        open,
        selected,
        open_field: None,
        words: TreeWords::of(&v.locale),
    })
}

/// The concept browser: a search, the hierarchy and the concept.
///
/// The tree keeps its open items in the address, so a reload or a shared
/// link opens the same items: opening adds an item, closing removes it.
fn browser(v: &Visit) -> Html {
    let brand = v.brand.key();
    let found = ["SYN-110", "SYN-120", "SYN-300"]
        .into_iter()
        .map(|code| {
            h(format!(
                "<a href=\"{}\"><code>{code}</code></a>",
                t(here(brand, &format!("?concept={code}")))
            ))
        })
        .collect();
    let search = Html::join([
        field(
            "browse-find",
            v.tr(Msg::FindConcept),
            None,
            h(format!(
                "<input id=\"browse-find\" name=\"find\" type=\"search\" class=\"{INPUT}\">"
            )),
        ),
        divided_list(Some(v.tr(Msg::Matches).into()), found),
    ]);
    let code = v.param("concept").unwrap_or("SYN-100").to_owned();
    let detail = h(format!(
        "<h2 class=\"fui-card-title\">{}</h2>{}",
        t(&code),
        definitions(vec![
            (v.tr(Msg::Code), h(format!("<code>{}</code>", t(&code)))),
            (v.tr(Msg::System), t("http://example.org/synthetic")),
            (v.tr(Msg::Status), t("active")),
            (v.tr(Msg::Parent), or_none(&v.locale, String::new())),
        ])
    ));
    card(
        v.tr(Msg::ThreePanesTree),
        three_panes(
            Pane::new(v.tr(Msg::Search), search),
            Pane::new(v.tr(Msg::Hierarchy), concept_tree(v, brand)),
            Pane::new(v.tr(Msg::Concept), detail),
        ),
    )
}

/// A list of value sets beside the one it opens.
fn listing(v: &Visit) -> Html {
    let back = t(here(v.brand.key(), ""));
    let items = [
        v.tr(Msg::SyntheticSigns),
        v.tr(Msg::SyntheticSites),
        v.tr(Msg::SyntheticUnits),
    ]
    .into_iter()
    .map(|name| h(format!("<a href=\"{back}\">{}</a>", t(name))))
    .collect();
    card(
        v.tr(Msg::MasterDetail),
        master_detail(
            divided_list(Some(v.tr(Msg::ValueSets).into()), items),
            stack_at(
                Gap::Default,
                Html::join([
                    definitions(vec![
                        (v.tr(Msg::Name), t(v.tr(Msg::SyntheticSigns))),
                        (v.tr(Msg::Version), t("1.0.0")),
                        (v.tr(Msg::Concepts), t("12")),
                    ]),
                    bullet_list(vec![
                        t(v.tr(Msg::ComposedTwoSyntheticCodeSystems)),
                        t(v.tr(Msg::ExcludesInactiveConcepts)),
                    ]),
                ]),
            ),
        ),
    )
}

/// The runner's form: the help switch, the picker, a number, a select, a
/// prose text area and a checkbox with help. Its reset button puts the
/// picker back on the first value set, as the form was drawn.
fn runner_form(v: &Visit) -> Html {
    let brand = v.brand.key();
    let options = vec![
        (
            String::from("http://example.org/vs/signs"),
            v.tr(Msg::SyntheticSigns),
        ),
        (
            String::from("http://example.org/vs/sites"),
            v.tr(Msg::SyntheticSites),
        ),
        (
            String::from("http://example.org/vs/units"),
            v.tr(Msg::SyntheticUnits),
        ),
    ];
    let reset = h(format!(
        "<button type=\"reset\" id=\"run-value-set-reset\" class=\"{}\">{}</button>",
        Variant::Secondary.class(),
        t(v.tr(Msg::UseFirstValueSet))
    ));
    let picker = picker_field(Picker {
        id: String::from("run-value-set"),
        name: String::from("url"),
        label: v.tr(Msg::ValueSet).into(),
        choose_label: v.tr(Msg::ChooseValueSetServerHolds).into(),
        help: Some(v.tr(Msg::CanonicalValueSetList).into()),
        choices: options,
        value: String::from("http://example.org/vs/signs"),
        empty_word: v.tr(Msg::ChooseOneServerHolds).into(),
        other_word: v.tr(Msg::AnotherCanonical).into(),
    });
    let count = h(format!(
        "<input id=\"run-count\" name=\"count\" inputmode=\"numeric\" class=\"{INPUT} {NARROW}\" value=\"25\" aria-describedby=\"{}\">",
        help_id("run-count")
    ));
    let language = h(format!(
        "<select id=\"run-language\" name=\"language\" class=\"{SELECT} {MEDIUM}\" aria-describedby=\"{}\"><option value=\"en\">{}</option><option value=\"nl\">{}</option></select>",
        help_id("run-language"),
        t(v.tr(Msg::English)),
        t(v.tr(Msg::Dutch))
    ));
    let note = h(format!(
        "<textarea id=\"run-note\" name=\"note\" rows=\"2\" class=\"{PROSE}\" aria-describedby=\"{}\"></textarea>",
        help_id("run-note")
    ));
    let inactive = h(format!(
        "<input id=\"run-inactive\" name=\"inactive\" type=\"checkbox\" aria-describedby=\"{}\">",
        help_id("run-inactive")
    ));
    h(format!(
        "<form method=\"get\" action=\"{}\" class=\"{FORM}\">{}{picker}<div>{reset}</div>{}{}{}{}<button type=\"submit\" class=\"{}\">{}</button></form>",
        t(here(brand, "")),
        split(
            h(format!(
                "<h2 class=\"fui-card-title\">{}</h2>",
                t(v.tr(Msg::ListValueSet))
            )),
            help_switch("run-help", HelpWords::of(&v.locale))
        ),
        explained_field(
            "run-count",
            v.tr(Msg::PageSize),
            v.tr(Msg::HowManyConceptsOnePage),
            count
        ),
        explained_field(
            "run-language",
            v.tr(Msg::DisplayLanguage),
            v.tr(Msg::LanguageDisplays),
            language
        ),
        explained_field(
            "run-note",
            v.tr(Msg::WhyYouRun),
            v.tr(Msg::NoteKeptBesideRun),
            note
        ),
        explained_check_field(
            "run-inactive",
            v.tr(Msg::IncludeInactiveConcepts),
            v.tr(Msg::InactiveConceptListedMarked),
            inactive
        ),
        Variant::Primary.class(),
        label(Some(Glyph::PLAY), v.tr(Msg::List)),
    ))
}

/// The runner's answer: latencies with their proportion bars.
fn runner_answer(v: &Visit) -> Html {
    let rows = [
        ("lookup", 12_u8, v.tr(Msg::N12Ms)),
        ("expand", 100, v.tr(Msg::N96Ms)),
        ("translate", 33, v.tr(Msg::N31Ms)),
    ]
    .into_iter()
    .map(|(name, share, figure)| {
        Html::join([
            split(
                t(name),
                h(format!("<span class=\"fui-num\">{}</span>", t(figure))),
            ),
            proportion(share, None),
        ])
    })
    .collect();
    let node = |name: &str, tone: Tone, state: &str, time: Msg| {
        h(format!(
            "<code class=\"fui-break\">{name}</code>{}<span class=\"fui-muted fui-num\">{}</span>",
            pill(tone, state),
            t(v.tr(time))
        ))
    };
    Html::join([
        h(format!(
            "<h2 class=\"fui-card-title\">{}</h2>",
            t(v.tr(Msg::Latency))
        )),
        divided_list(Some(v.tr(Msg::LatencyPerOperation).into()), rows),
        proportion(0, Some(v.tr(Msg::NoTimeSpent0Percent))),
        h(format!(
            "<h2 class=\"fui-card-title\">{}</h2>",
            t(v.tr(Msg::Nodes))
        )),
        tile_list(
            v.tr(Msg::NodesAnswered),
            vec![
                node("synthetic-node-a", Tone::Ok, "answered", Msg::N12Ms),
                node(
                    "synthetic-node-c",
                    Tone::Warn,
                    &v.tr(Msg::TimedOut),
                    Msg::N30S,
                ),
            ],
        ),
    ])
}

/// Rows of fields with columns of their own, as an editor draws them.
fn rows(v: &Visit) -> Html {
    let input = |id: &'static str, value: &str| {
        h(format!(
            "<input id=\"{id}\" name=\"{id}\" class=\"{INPUT}\" value=\"{}\">",
            t(value)
        ))
    };
    let remove = || {
        h(format!(
            "<button type=\"button\" class=\"{}\">{}</button>",
            Variant::Danger.class(),
            label(Some(Glyph::TRASH), v.tr(Msg::Remove))
        ))
    };
    let up = h(format!(
        "<button type=\"button\" class=\"{}\">{}</button>",
        Variant::Secondary.class(),
        label(Some(Glyph::ARROW_UP), v.tr(Msg::Up))
    ));
    card(
        v.tr(Msg::RowsFields),
        stack_at(
            Gap::Default,
            Html::join([
                field_row(
                    FieldRow::CodeDisplay,
                    Html::join([
                        field(
                            "row-code",
                            v.tr(Msg::Code),
                            None,
                            input("row-code", "SYN-110"),
                        ),
                        field(
                            "row-display",
                            v.tr(Msg::Display),
                            None,
                            input("row-display", &v.tr(Msg::SyntheticFindingArm)),
                        ),
                        up,
                        remove(),
                    ]),
                ),
                field_row(
                    FieldRow::CodeDisplayDefinition,
                    Html::join([
                        field(
                            "row-code-2",
                            v.tr(Msg::Code),
                            None,
                            input("row-code-2", "SYN-120"),
                        ),
                        field(
                            "row-display-2",
                            v.tr(Msg::Display),
                            None,
                            input("row-display-2", &v.tr(Msg::SyntheticFindingLeg)),
                        ),
                        field(
                            "row-definition-2",
                            v.tr(Msg::Definition),
                            None,
                            input("row-definition-2", &v.tr(Msg::FindingInventedGallery)),
                        ),
                        remove(),
                    ]),
                ),
                field_row(
                    FieldRow::Halves,
                    Html::join([
                        field(
                            "row-system",
                            v.tr(Msg::System),
                            None,
                            input("row-system", "http://example.org/synthetic"),
                        ),
                        field(
                            "row-version",
                            v.tr(Msg::Version),
                            None,
                            input("row-version", "1.0.0"),
                        ),
                    ]),
                ),
            ]),
        ),
    )
}

/// The stacks at each step of space.
fn stacks(v: &Visit) -> Html {
    let lines = |gap: Gap, name: String| {
        h(format!(
            "<div class=\"fui-well\">{}</div>",
            stack_at(
                gap,
                h(format!(
                    "<p>{}</p><p class=\"fui-muted\">{}</p><p class=\"fui-muted\">{}</p>",
                    t(name),
                    t(v.tr(Msg::SecondLine)),
                    t(v.tr(Msg::ThirdLine))
                ))
            )
        ))
    };
    card(
        v.tr(Msg::Stacks),
        stack_at(
            Gap::Default,
            Html::join([
                lines(Gap::Tight, v.tr(Msg::Tight)),
                lines(Gap::Default, v.tr(Msg::Default)),
                lines(Gap::Loose, v.tr(Msg::Loose)),
                lines(Gap::Section, v.tr(Msg::Section)),
            ]),
        ),
    )
}

/// Lists whose items a layout class sets apart, a grid and a stack, which
/// draw no marker and keep no indent.
fn laid_out_lists(v: &Visit) -> Html {
    let steps = [
        v.tr(Msg::Tight),
        v.tr(Msg::Default),
        v.tr(Msg::Loose),
        v.tr(Msg::Section),
    ];
    let tiles: Html = steps
        .iter()
        .map(|step| h(format!("<li class=\"fui-well\">{}</li>", t(step))))
        .collect();
    let lines: Html = steps
        .iter()
        .map(|step| h(format!("<li>{}</li>", t(step))))
        .collect();
    card(
        v.tr(Msg::ListsLaidOut),
        h(format!(
            "<ul class=\"fui-grid\">{tiles}</ul><ul class=\"fui-stack fui-gap-tight\">{lines}</ul>"
        )),
    )
}

/// A fieldset that is a card's own child, right under the card's title,
/// which keeps the card's rhythm above it.
fn fieldset_in_card(v: &Visit) -> Html {
    card(
        v.tr(Msg::FieldsetUnderTitle),
        h(format!(
            "<fieldset class=\"fui-fieldset\"><legend>{}</legend>{}</fieldset>",
            t(v.tr(Msg::TheCode)),
            field(
                "fieldset-code",
                v.tr(Msg::Code),
                None,
                h(format!(
                    "<input id=\"fieldset-code\" name=\"code\" class=\"{INPUT}\">"
                )),
            )
        )),
    )
}

/// A panel of three sections (B10): a table, a list and a code viewer,
/// each running to the panel's edges under its heading row.
fn sections_panel(v: &Visit) -> Html {
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
        &v.locale,
        Some(SectionHead {
            title: v.tr(Msg::StagedChanges),
            count: Some(HeadingCount::All(2)),
            actions: None,
        }),
        change_table(
            &v.locale,
            v.tr(Msg::StagedConfigurationAgainstRunningOne),
            &staged,
        ),
    );
    let steps = [
        v.tr(Msg::Tight),
        v.tr(Msg::Default),
        v.tr(Msg::Loose),
        v.tr(Msg::Section),
    ]
    .into_iter()
    .map(t)
    .collect();
    let list = panel_section(
        &v.locale,
        Some(SectionHead {
            title: v.tr(Msg::SpacingSteps),
            count: Some(HeadingCount::All(4)),
            actions: None,
        }),
        Html::join([
            section_intro(v.tr(Msg::PanelSectionsIntro)),
            divided_list(None, steps),
            add_row(h(format!(
                "<button type=\"button\" class=\"{}\">{}</button>",
                Variant::Ghost.small(),
                t(v.tr(Msg::AddStep))
            ))),
        ]),
    );
    let [path, ..] = crate::data::FILE_NAMES;
    let source = code_section(
        &v.locale,
        CodeHead {
            title: String::from(path),
            tags: vec![v.tr(Msg::Yaml)],
            ..CodeHead::default()
        },
        &Code {
            label: v.tr(Msg::MappingFileYaml),
            language: Language::Yaml,
            text: String::from(crate::data::YAML),
            ids: None,
            current: None,
            diagnostics: vec![LineDiagnostic {
                line: 19,
                gravity: Gravity::Warning,
                code: String::from(crate::data::FINDING_CODES[1]),
                message: v.tr(Msg::FindingStartTwice),
            }],
        },
    );
    panel_sections_card(v.tr(Msg::PanelSections), Html::join([table, list, source]))
}

/// The layouts page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let header = page_header(
        &v.locale,
        v.tr(Msg::Layouts),
        Some(v.tr(Msg::ThreePanesTreeListBeside)),
        ActionGroup::none(),
    );
    let body = stack_at(
        Gap::Loose,
        Html::join([
            sections_panel(v),
            browser(v),
            listing(v),
            card(
                v.tr(Msg::MainAside),
                main_aside(runner_form(v), runner_answer(v)),
            ),
            rows(v),
            stacks(v),
            laid_out_lists(v),
            fieldset_in_card(v),
            super::records::timeline_section(v),
        ]),
    );
    Page::shell(v.tr(Msg::Layouts), Html::join([header, body]))
}
