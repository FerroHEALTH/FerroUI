// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The terminology page.
//!
//! It holds the coding stacked, inline, inactive, with no display and with
//! a long canonical, and a row's name over its canonical; a value set's
//! clauses read-only and in a composer, with their filter rows, their codes
//! and a broken invariant; the root matrix with a root that did not answer;
//! and a concept map's mappings read-only and in an editor, one with no
//! relationship. Its top bar carries the FHIR version select
//! ([`version_select`]), which the user menu holds on a phone.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::icon::icon;
use ferrohealth_ui::table::view::{
    Body, Column, DataTable, Role, Sorting, data_table, group_row, title_cell,
};
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::clause::{
    Clause, ClauseCode, ClauseIssue, ClauseKind, clause, clause_codes, clause_fields, clause_part,
    clauses,
};
use ferrohealth_ui::view::coding::{Coding, Layout, MoreCodes, canonical, coding};
use ferrohealth_ui::view::form::{ERROR, Field, INPUT, SELECT, error_id};
use ferrohealth_ui::view::mapping::{
    MapElement, Mapping, Relationship, map_element, map_elements, mapping_columns, mapping_row,
};
use ferrohealth_ui::view::root_matrix::{MatrixRoot, MatrixRow, RootMatrix, root_matrix};
use ferrohealth_ui::view::surface::{
    Crumb, Header, SectionHead, add_row, card, definitions, page_heading, panel_section,
    panel_sections_card, section_intro, stack,
};
use ferrohealth_ui::widget::repeat::{Fields, ROW, Repeat, RepeatAdd, RepeatRow, RowEnd, repeat};
use ferrohealth_ui::widget::version::{VersionRoot, VersionSelect};
use leptos::prelude::*;

use crate::locale::{Msg, tr, tr_with};
use crate::terminology_data::{
    ABDOMEN, BALLOT_ROOT, CHEST, CODE_SYSTEMS, COUGH, DEFAULT_ROOT, FEVER, FINDINGS, LEGACY,
    LEGACY_CHEST, LEGACY_COUGH, LONG_CANONICAL, MALAISE, MAPPINGS, ROOTS, SECURITY, SOFTWARE,
    URN_CANONICAL, VALUE_SETS, VERSIONED_UID, WHEEZE, base, display, legacy_display,
};

/// The id of the version select's menu.
const VERSION_MENU: &str = "fhir-version";

/// The query key the version select's links set.
const VERSION_KEY: &str = "fhir";

/// The FHIR version select of the page at `back`, `chosen` in view: the
/// top bar's control and, through [`VersionSelect::choices`], the user
/// menu's group on a phone.
#[must_use]
pub fn version_select(back: &str, chosen: Option<&str>) -> VersionSelect {
    VersionSelect {
        id: String::from(VERSION_MENU),
        label: tr(Msg::TmFhirVersion),
        roots: ROOTS
            .iter()
            .map(|(name, release, _)| VersionRoot {
                name: (*name).to_owned(),
                release: (*release).to_owned(),
                href: format!("{back}?{VERSION_KEY}={name}"),
                default: *name == DEFAULT_ROOT,
            })
            .collect(),
        current: chosen.unwrap_or(DEFAULT_ROOT).to_owned(),
    }
}

/// The key of the query that chooses the FHIR version.
#[must_use]
pub const fn version_key() -> &'static str {
    VERSION_KEY
}

/// The synthetic finding `code` with its display.
fn finding(code: &str) -> Coding {
    Coding::new(code, display(code))
}

/// The codings in each of their forms, and a row's name over its
/// canonical.
fn codings() -> AnyView {
    let forms = definitions(vec![
        (
            tr(Msg::TmStacked),
            coding(finding(COUGH).system(FINDINGS), Layout::Stacked),
        ),
        (tr(Msg::TmInline), coding(finding(WHEEZE), Layout::Inline)),
        (
            tr(Msg::TmInactive),
            coding(finding(MALAISE).system(FINDINGS).inactive(), Layout::Inline),
        ),
        (
            tr(Msg::TmNoDisplay),
            coding(
                Coding {
                    display: None,
                    code: String::from(ABDOMEN),
                    system: None,
                    inactive: false,
                    href: None,
                    current: false,
                },
                Layout::Stacked,
            ),
        ),
        (tr(Msg::TmCanonical), canonical(LONG_CANONICAL)),
        (
            tr(Msg::TmVersionedUid),
            coding(
                Coding::new(VERSIONED_UID, tr(Msg::TmSyntheticEncounter)),
                Layout::Stacked,
            ),
        ),
        (tr(Msg::TmUrnCanonical), canonical(URN_CANONICAL)),
    ])
    .into_any();
    let columns = [Column::owned("title", tr(Msg::TmTitle)).role(Role::Title)];
    let rows = VALUE_SETS
        .iter()
        .map(|(title, url)| {
            let head = title_cell(Some("#codings"), (*title).to_owned(), Some(canonical(url)));
            view! { <tr>{head}</tr> }
        })
        .collect_view()
        .into_any();
    let caption = tr(Msg::TmValueSets);
    let table = data_table(DataTable {
        caption: &caption,
        columns: &columns,
        sorting: Sorting::default(),
        body: Body::Rows(rows),
    })
    .into_any();
    view! {
        <div id="codings">
            {card(tr(Msg::TmCodings), view! { {forms} {table} })}
        </div>
    }
    .into_any()
}

/// The source of a clause from the synthetic findings, as words.
fn findings_source() -> String {
    tr_with(Msg::TmFrom, &[("name", FINDINGS.into())])
}

/// How many codes the listed include holds, of which the page shows three.
const LISTED: u32 = 40;

/// A value set's clauses as its page reads them.
fn clauses_read() -> AnyView {
    let source = findings_source();
    let include = clause(Clause {
        kind: ClauseKind::Include,
        title: view! { <a href="#codings">{source.clone()}</a> }.into_any(),
        source: source.clone(),
        sentence: tr_with(
            Msg::TmIncludesBelow,
            &[("value", CHEST.into()), ("name", display(CHEST).into())],
        )
        .into_any(),
        body: None,
        actions: None,
        issues: Vec::new(),
    });
    let exclude = clause(Clause {
        kind: ClauseKind::Exclude,
        title: view! { <a href="#codings">{source.clone()}</a> }.into_any(),
        source: source.clone(),
        sentence: view! {
            {tr_with(Msg::TmExcludesCodes, &[("count", 1.into())])}
            " "
            {coding(finding(WHEEZE), Layout::Inline)}
        }
        .into_any(),
        body: None,
        actions: None,
        issues: Vec::new(),
    });
    let listed = clause(Clause {
        kind: ClauseKind::Include,
        title: source.clone().into_any(),
        source,
        sentence: tr_with(Msg::TmIncludesCodes, &[("count", LISTED.into())]).into_any(),
        body: Some(clause_codes(
            [COUGH, FEVER, MALAISE]
                .into_iter()
                .map(|code| ClauseCode {
                    coding: if code == MALAISE {
                        finding(code).inactive()
                    } else {
                        finding(code)
                    },
                    remove: None,
                })
                .collect(),
            Some(MoreCodes::of_page(3, LISTED, 25, "#codings")),
        )),
        actions: None,
        issues: Vec::new(),
    });
    let section = panel_section(
        Some(SectionHead {
            title: tr(Msg::TmSelects),
            count: Some(3),
            actions: None,
        }),
        clauses(view! { {include} {exclude} {listed} }),
    );
    panel_sections_card(tr(Msg::TmValueSet), section)
}

/// A select of `options`, each its value and words, `chosen` selected.
fn select(id: &str, name: &str, options: Vec<(String, String)>, chosen: &str) -> AnyView {
    refused_select(id, name, options, chosen, false)
}

/// [`select`], refused with its error line when `refused`.
fn refused_select(
    id: &str,
    name: &str,
    options: Vec<(String, String)>,
    chosen: &str,
    refused: bool,
) -> AnyView {
    let options = options
        .into_iter()
        .map(|(value, words)| {
            let selected = value == chosen;
            view! {
                <option value=value selected=selected>
                    {words}
                </option>
            }
        })
        .collect_view();
    view! {
        <select
            id=id.to_owned()
            name=name.to_owned()
            class=SELECT
            aria-invalid=refused.then_some("true")
            aria-describedby=refused.then(|| error_id(id))
        >
            {options}
        </select>
    }
    .into_any()
}

/// One filter row of the include in the composer, `key` its key.
fn filter_row(key: &str, value: &str) -> RepeatRow {
    let property = format!("filter-{key}-property");
    let operator = format!("filter-{key}-op");
    let typed = format!("filter-{key}-value");
    let fields = view! {
        {Field::new(property.clone(), tr(Msg::TmProperty))
            .view(
                select(
                    &property,
                    &format!("filter[{key}][property]"),
                    vec![
                        (String::from("concept"), tr(Msg::TmPropConcept)),
                        (String::from("parent"), tr(Msg::TmPropParent)),
                        (String::from("inactive"), tr(Msg::TmPropInactive)),
                    ],
                    "concept",
                ),
            )}
        {Field::new(operator.clone(), tr(Msg::TmOperator))
            .view(
                select(
                    &operator,
                    &format!("filter[{key}][op]"),
                    vec![
                        (String::from("is-a"), tr(Msg::TmOpIsA)),
                        (String::from("descendent-of"), tr(Msg::TmOpBelow)),
                        (String::from("="), tr(Msg::TmOpEquals)),
                    ],
                    "is-a",
                ),
            )}
        {Field::new(typed.clone(), tr(Msg::TmValue))
            .view(
                view! {
                    <input
                        id=typed
                        name=format!("filter[{key}][value]")
                        type="text"
                        class=format!("{INPUT} fui-mono")
                        value=value.to_owned()
                    />
                },
            )}
    }
    .into_any();
    RepeatRow {
        key: key.to_owned(),
        fields,
        end: RowEnd::Remove {
            label: tr(Msg::TmRemoveFilter),
            name: String::from("remove-filter"),
            value: key.to_owned(),
        },
        after: None,
    }
}

/// The Remove of a code of a clause in the composer.
fn remove_code(code: &str) -> AnyView {
    let label = tr_with(Msg::TmRemoveCode, &[("value", code.to_owned().into())]);
    let title = label.clone();
    view! {
        <button
            type="submit"
            class="fui-icon-button"
            name="remove-code"
            value=code.to_owned()
            formnovalidate=true
            aria-label=label
            title=title
        >
            {icon(Glyph::X, "16")}
        </button>
    }
    .into_any()
}

/// The Remove of a whole clause in the composer.
fn remove_clause(key: &str) -> AnyView {
    let label = tr(Msg::TmRemoveClause);
    let title = label.clone();
    view! {
        <button
            type="submit"
            class="fui-icon-button"
            name="remove-clause"
            value=key.to_owned()
            formnovalidate=true
            aria-label=label
            title=title
        >
            {icon(Glyph::TRASH, "16")}
        </button>
    }
    .into_any()
}

/// The code system and version fields of the clause `key` in the
/// composer.
fn source_fields(key: &str) -> AnyView {
    let system = format!("clause-{key}-system");
    let version = format!("clause-{key}-version");
    clause_fields(view! {
        {Field::new(system.clone(), tr(Msg::TmCodeSystem))
            .view(
                select(
                    &system,
                    &format!("clause[{key}][system]"),
                    vec![(String::from("sf"), FINDINGS.to_owned())],
                    "sf",
                ),
            )}
        {Field::new(version.clone(), tr(Msg::TmVersion))
            .view(
                select(
                    &version,
                    &format!("clause[{key}][version]"),
                    vec![(String::from("3"), tr(Msg::TmVersionDefault))],
                    "3",
                ),
            )}
    })
}

/// The include of the composer: its source, a filter row, a code and the
/// invariant the two break together.
fn composed_include() -> AnyView {
    let source = findings_source();
    let filters = repeat(Repeat {
        id: String::from("include-filters"),
        label: tr(Msg::TmFiltersOf),
        fields: Fields::Three,
        head: Vec::new(),
        rows: vec![filter_row("f1", CHEST)],
        blank: Some(filter_row(ROW, "")),
        add: Some(RepeatAdd {
            label: tr(Msg::TmAddFilter),
            name: String::from("add-filter"),
            value: String::from("include"),
        }),
        empty: tr(Msg::TmNoFilters),
        form: None,
    });
    clause(Clause {
        kind: ClauseKind::Include,
        title: source.clone().into_any(),
        source: source.clone(),
        sentence: tr_with(
            Msg::TmIncludesBelow,
            &[("value", CHEST.into()), ("name", display(CHEST).into())],
        )
        .into_any(),
        body: Some(
            view! {
                {source_fields("c1")}
                {clause_part(tr(Msg::TmFilters))}
                {filters}
                {clause_part(tr(Msg::TmCodes))}
                {clause_codes(
                    vec![
                        ClauseCode {
                            coding: finding(FEVER),
                            remove: Some(remove_code(FEVER)),
                        },
                    ],
                    None,
                )}
            }
            .into_any(),
        ),
        actions: Some(remove_clause("c1")),
        issues: vec![ClauseIssue {
            key: String::from("vsd-3"),
            text: tr(Msg::TmVsd3),
        }],
    })
}

/// A value set's clauses in its composer: the include with a filter row,
/// a code and the invariant they break together, and the exclude with its
/// code.
fn composer() -> AnyView {
    let include = composed_include();
    let source = findings_source();
    let exclude = clause(Clause {
        kind: ClauseKind::Exclude,
        title: source.clone().into_any(),
        source,
        sentence: view! {
            {tr_with(Msg::TmExcludesCodes, &[("count", 1.into())])}
            " "
            {coding(finding(WHEEZE), Layout::Inline)}
        }
        .into_any(),
        body: Some(
            view! {
                {source_fields("c2")}
                {clause_part(tr(Msg::TmCodes))}
                {clause_codes(
                    vec![
                        ClauseCode {
                            coding: finding(WHEEZE),
                            remove: Some(remove_code(WHEEZE)),
                        },
                    ],
                    None,
                )}
            }
            .into_any(),
        ),
        actions: Some(remove_clause("c2")),
        issues: Vec::new(),
    });
    let adds = add_row(view! {
        <button type="submit" class=Variant::Ghost.small() name="add-clause" value="include">
            {icon(Glyph::PLUS, "16")}
            {tr(Msg::TmAddInclude)}
        </button>
        <button type="submit" class=Variant::Ghost.small() name="add-clause" value="exclude">
            {icon(Glyph::PLUS, "16")}
            {tr(Msg::TmAddExclude)}
        </button>
    });
    let section = panel_section(
        Some(SectionHead {
            title: tr(Msg::TmComposer),
            count: Some(2),
            actions: None,
        }),
        view! {
            {section_intro(tr(Msg::TmComposerIntro))}
            {clauses(view! { {include} {exclude} })}
            {adds}
        },
    );
    view! {
        <form method="post" action="#" id="value-set-composer">
            {panel_sections_card(tr(Msg::TmEditValueSet), section)}
        </form>
    }
    .into_any()
}

/// What each root of the synthetic server declares, its last root down.
fn matrix() -> AnyView {
    let roots: Vec<MatrixRoot> = ROOTS
        .iter()
        .map(|(name, release, answered)| MatrixRoot {
            name: (*name).to_owned(),
            release: (*release).to_owned(),
            answered: *answered,
        })
        .collect();
    let same = |value: &str| -> Vec<Option<AnyView>> {
        ROOTS
            .iter()
            .map(|_| Some(value.to_owned().into_any()))
            .collect()
    };
    let rows = vec![
        MatrixRow {
            name: tr(Msg::TmSoftware).into_any(),
            cells: same(SOFTWARE),
        },
        MatrixRow {
            name: tr(Msg::TmBase).into_any(),
            cells: ROOTS
                .iter()
                .map(|(name, _, _)| Some(view! { <code>{base(name)}</code> }.into_any()))
                .collect(),
        },
        MatrixRow {
            name: tr(Msg::TmCodeSystems).into_any(),
            cells: same(CODE_SYSTEMS),
        },
        MatrixRow {
            name: tr(Msg::TmSecurity).into_any(),
            cells: same(SECURITY),
        },
        MatrixRow {
            name: tr(Msg::TmServes).into_any(),
            cells: ROOTS
                .iter()
                .map(|(name, _, _)| (*name == BALLOT_ROOT).then(|| tr(Msg::TmBallot).into_any()))
                .collect(),
        },
    ];
    let body = root_matrix(RootMatrix {
        caption: tr(Msg::TmDeclares),
        corner: tr(Msg::TmDeclaration),
        roots,
        rows,
    });
    let section = panel_section(
        Some(SectionHead {
            title: tr(Msg::TmDeclares),
            count: None,
            actions: None,
        }),
        body,
    );
    panel_sections_card(tr(Msg::TmServer), section)
}

/// A mapping of the synthetic concept map.
fn mapping(row: &crate::terminology_data::Mapped) -> Mapping {
    let (source, equivalence, target, comment) = *row;
    Mapping {
        source: Coding::new(source, legacy_display(source)),
        relationship: Relationship::from_r4(equivalence).unwrap_or(Relationship::Unmatched),
        target: target.map(finding),
        comment: comment.map(str::to_owned),
    }
}

/// The concept map's mappings as its page reads them, in one group.
fn mappings() -> AnyView {
    let columns = mapping_columns(true);
    let heading = tr_with(
        Msg::TmGroup,
        &[("name", LEGACY.into()), ("value", FINDINGS.into())],
    );
    let group = group_row(&columns, heading);
    let rows = MAPPINGS
        .iter()
        .map(|row| mapping_row(&columns, mapping(row)))
        .collect_view();
    let caption = tr(Msg::TmMappings);
    let table = data_table(DataTable {
        caption: &caption,
        columns: &columns,
        sorting: Sorting::default(),
        body: Body::Bodies(view! { <tbody>{group} {rows}</tbody> }.into_any()),
    })
    .into_any();
    let section = panel_section(
        Some(SectionHead {
            title: tr(Msg::TmMappings),
            count: Some(u64::try_from(MAPPINGS.len()).unwrap_or(u64::MAX)),
            actions: None,
        }),
        table,
    );
    panel_sections_card(tr(Msg::TmConceptMap), section)
}

/// The relationship choices of a target row in the editor, each its R4
/// code and its words with a capital.
fn relationship_options() -> Vec<(String, String)> {
    let mut options = vec![(String::new(), tr(Msg::TmChoose))];
    for code in [
        "equivalent",
        "equal",
        "wider",
        "subsumes",
        "narrower",
        "specializes",
        "inexact",
        "unmatched",
        "disjoint",
    ] {
        if let Some(relationship) = Relationship::from_r4(code) {
            let words = relationship.words();
            let mut letters = words.text().chars();
            let capital = letters
                .next()
                .map(|first| first.to_uppercase().chain(letters).collect::<String>())
                .unwrap_or_default();
            options.push((code.to_owned(), capital));
        }
    }
    options
}

/// The target row `key` of the source `source` in the editor: its code,
/// its relationship and its comment, with an error line when `missing`.
fn target_row(source: &str, key: &str, target: Target, missing: bool) -> RepeatRow {
    let (code, relationship, comment) = target;
    let target_id = format!("target-{source}-{key}-code");
    let relationship_id = format!("target-{source}-{key}-rel");
    let comment_id = format!("target-{source}-{key}-comment");
    let name = |part: &str| format!("target[{source}][{key}][{part}]");
    let chosen = if missing { "" } else { relationship };
    let relation = refused_select(
        &relationship_id,
        &name("relationship"),
        relationship_options(),
        chosen,
        missing,
    );
    let fields = view! {
        {Field::new(target_id.clone(), tr(Msg::TmTargetCode))
            .view(
                view! {
                    <input
                        id=target_id
                        name=name("code")
                        type="text"
                        class=format!("{INPUT} fui-mono")
                        value=code.to_owned()
                    />
                },
            )}
        {Field::new(relationship_id.clone(), tr(Msg::TmRelationship)).view(relation)}
        {Field::new(comment_id.clone(), tr(Msg::TmComment))
            .view(
                view! {
                    <input
                        id=comment_id
                        name=name("comment")
                        type="text"
                        class=INPUT
                        value=comment.to_owned()
                        placeholder=tr(Msg::TmOptional)
                    />
                },
            )}
    }
    .into_any();
    let after = missing.then(|| {
        let line = tr_with(
            Msg::TmChooseRelationship,
            &[("value", code.to_owned().into())],
        );
        view! {
            <p id=error_id(&relationship_id) class=ERROR>
                {line}
            </p>
        }
        .into_any()
    });
    RepeatRow {
        key: key.to_owned(),
        fields,
        end: RowEnd::Remove {
            label: tr_with(Msg::TmRemoveTarget, &[("value", source.to_owned().into())]),
            name: String::from("remove-target"),
            value: format!("{source}/{key}"),
        },
        after,
    }
}

/// A target of a source code in the editor: its code, its relationship's
/// R4 code and its comment.
type Target = (&'static str, &'static str, &'static str);

/// One source code in the editor with its target rows, its one target
/// missing its relationship when `missing`.
fn element(source: &str, target: Target, missing: bool) -> AnyView {
    let label = tr_with(Msg::TmTargetsOf, &[("value", source.to_owned().into())]);
    let targets = repeat(Repeat {
        id: format!("targets-{source}"),
        label,
        fields: Fields::Three,
        head: Vec::new(),
        rows: vec![target_row(source, "t1", target, missing)],
        blank: Some(target_row(source, ROW, ("", "", ""), false)),
        add: Some(RepeatAdd {
            label: tr(Msg::TmAddTarget),
            name: String::from("add-target"),
            value: source.to_owned(),
        }),
        empty: tr(Msg::TmNoTargets),
        form: None,
    });
    let remove = tr_with(Msg::TmRemoveSource, &[("value", source.to_owned().into())]);
    let title = remove.clone();
    let actions = view! {
        <button
            type="submit"
            class="fui-icon-button"
            name="remove-source"
            value=source.to_owned()
            formnovalidate=true
            aria-label=remove
            title=title
        >
            {icon(Glyph::TRASH, "16")}
        </button>
    }
    .into_any();
    map_element(MapElement {
        source: Coding::new(source, legacy_display(source)),
        actions: Some(actions),
        targets,
        after: None,
    })
}

/// The concept map's editor: one group, each source code a block with its
/// target rows, the second missing its relationship.
fn map_editor() -> AnyView {
    let fields = clause_fields(view! {
        {Field::new("group-source", tr(Msg::TmSourceSystem))
            .view(select("group-source", "group[source]", vec![(String::from("legacy"), LEGACY.to_owned())], "legacy"))}
        {Field::new("group-target", tr(Msg::TmTargetSystem))
            .view(select("group-target", "group[target]", vec![(String::from("sf"), FINDINGS.to_owned())], "sf"))}
    });
    let elements = map_elements(view! {
        {element(LEGACY_COUGH, (COUGH, "equivalent", ""), false)}
        {element(LEGACY_CHEST, (WHEEZE, "narrower", ""), true)}
    });
    let section = panel_section(
        Some(SectionHead {
            title: tr_with(
                Msg::TmEditorGroup,
                &[("name", LEGACY.into()), ("value", FINDINGS.into())],
            ),
            count: None,
            actions: None,
        }),
        view! {
            {fields}
            {clause_part(tr(Msg::TmCodes))}
            {elements}
        },
    );
    view! {
        <form method="post" action="#" id="concept-map-editor">
            {panel_sections_card(tr(Msg::TmEditConceptMap), section)}
        </form>
    }
    .into_any()
}

/// The terminology page.
#[component]
pub fn TerminologyPage() -> impl IntoView {
    let root =
        crate::app::route_brand().map_or_else(String::new, |brand| format!("/{}", brand.key()));
    let header = page_heading(
        Header {
            title: tr(Msg::Terminology).into(),
            subtitle: Some(tr(Msg::TmSubtitle).into()),
            subtitle_view: None,
            crumbs: vec![Crumb::new(tr(Msg::Gallery), root)],
            mono: false,
            badge: None,
        },
        ActionGroup::none(),
    )
    .into_any();
    let sections = (
        codings(),
        clauses_read(),
        composer(),
        matrix(),
        mappings(),
        map_editor(),
    );
    view! {
        {super::title(tr(Msg::Terminology))}
        {header}
        {stack(
            view! {
                {sections.0}
                {sections.1}
                {sections.2}
                {sections.3}
                {sections.4}
                {sections.5}
            },
        )}
    }
}
