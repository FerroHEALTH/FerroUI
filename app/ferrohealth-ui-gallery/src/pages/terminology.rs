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

use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::{Glyph, icon};
use ferrohealth_ui::table::mapping::{
    MapElement, Mapping, Relationship, map_element, map_elements, mapping_columns, mapping_row,
};
use ferrohealth_ui::table::view::{
    Body, Column, DataTable, Role, data_table, group_row, title_cell,
};
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::clause::{
    Clause, ClauseCode, ClauseIssue, ClauseKind, clause, clause_codes, clause_fields, clause_part,
    clauses,
};
use ferrohealth_ui::view::coding::{Coding, Layout, MoreCodes, canonical, coding};
use ferrohealth_ui::view::form::{ERROR, Field, INPUT, SELECT, error_id};
use ferrohealth_ui::view::repeat::{Fields, ROW, Repeat, RepeatAdd, RepeatRow, RowEnd, repeat};
use ferrohealth_ui::view::root_matrix::{MatrixRoot, MatrixRow, RootMatrix, root_matrix};
use ferrohealth_ui::view::surface::{
    Container, Crumb, Header, SectionHead, card, definitions, page_heading, panel_section,
    panel_sections_card, section_intro,
};
use ferrohealth_ui::view::template::list::HeadingCount;
use ferrohealth_ui::view::version::{VersionRoot, VersionSelect};

use crate::locale::Msg;
use crate::page::Page;
use crate::terminology_data::{
    ABDOMEN, BALLOT_ROOT, CHEST, CODE_SYSTEMS, COUGH, DEFAULT_ROOT, FEVER, FINDINGS, LEGACY,
    LEGACY_CHEST, LEGACY_COUGH, LONG_CANONICAL, MALAISE, MAPPINGS, ROOTS, SECURITY, SOFTWARE,
    URN_CANONICAL, VALUE_SETS, VERSIONED_UID, WHEEZE, base, display, legacy_display,
};
use crate::visit::Visit;

/// The id of the version select's menu.
const VERSION_MENU: &str = "fhir-version";

/// The query key the version select's links set.
const VERSION_KEY: &str = "fhir";

/// The FHIR version select of the page at `back`, `chosen` in view: the
/// top bar's control and, through [`VersionSelect::choices`], the user
/// menu's group on a phone.
#[must_use]
pub fn version_select(v: &Visit, back: &str, chosen: Option<&str>) -> VersionSelect {
    VersionSelect {
        id: String::from(VERSION_MENU),
        label: v.tr(Msg::TmFhirVersion),
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
fn codings(v: &Visit) -> Html {
    let forms = definitions(vec![
        (
            v.tr(Msg::TmStacked),
            coding(&v.locale, finding(COUGH).system(FINDINGS), Layout::Stacked),
        ),
        (
            v.tr(Msg::TmInline),
            coding(&v.locale, finding(WHEEZE), Layout::Inline),
        ),
        (
            v.tr(Msg::TmInactive),
            coding(
                &v.locale,
                finding(MALAISE).system(FINDINGS).inactive(),
                Layout::Inline,
            ),
        ),
        (
            v.tr(Msg::TmNoDisplay),
            coding(
                &v.locale,
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
        (v.tr(Msg::TmCanonical), canonical(LONG_CANONICAL)),
        (
            v.tr(Msg::TmVersionedUid),
            coding(
                &v.locale,
                Coding::new(VERSIONED_UID, v.tr(Msg::TmSyntheticEncounter)),
                Layout::Stacked,
            ),
        ),
        (v.tr(Msg::TmUrnCanonical), canonical(URN_CANONICAL)),
    ]);
    let columns = [Column::owned("title", v.tr(Msg::TmTitle)).role(Role::Title)];
    let rows = VALUE_SETS
        .iter()
        .map(|(title, url)| {
            let head = title_cell(
                None,
                Some("#codings"),
                Html::text(title),
                Some(canonical(url)),
            );
            Html::raw(format!("<tr>{head}</tr>"))
        })
        .collect();
    let caption = v.tr(Msg::TmValueSets);
    let table = data_table(DataTable::new(&caption, &columns, Body::Rows(rows)));
    Html::raw(format!(
        "<div id=\"codings\">{}</div>",
        card(v.tr(Msg::TmCodings), Html::join([forms, table]))
    ))
}

/// The source of a clause from the synthetic findings, as words.
fn findings_source(v: &Visit) -> String {
    v.tr_with(Msg::TmFrom, &[("name", FINDINGS.into())])
}

/// How many codes the listed include holds, of which the page shows three.
const LISTED: u32 = 40;

/// A value set's clauses as its page reads them.
fn clauses_read(v: &Visit) -> Html {
    let source = findings_source(v);
    let include = clause(
        &v.locale,
        Clause {
            kind: ClauseKind::Include,
            title: Html::raw(format!("<a href=\"#codings\">{}</a>", Html::text(&source))),
            source: source.clone(),
            sentence: Html::text(v.tr_with(
                Msg::TmIncludesBelow,
                &[("value", CHEST.into()), ("name", display(CHEST).into())],
            )),
            body: None,
            actions: None,
            issues: Vec::new(),
        },
    );
    let exclude = clause(
        &v.locale,
        Clause {
            kind: ClauseKind::Exclude,
            title: Html::raw(format!("<a href=\"#codings\">{}</a>", Html::text(&source))),
            source: source.clone(),
            sentence: Html::raw(format!(
                "{} {}",
                Html::text(v.tr_with(Msg::TmExcludesCodes, &[("count", 1.into())])),
                coding(&v.locale, finding(WHEEZE), Layout::Inline)
            )),
            body: None,
            actions: None,
            issues: Vec::new(),
        },
    );
    let listed = clause(
        &v.locale,
        Clause {
            kind: ClauseKind::Include,
            title: Html::text(&source),
            source,
            sentence: Html::text(v.tr_with(Msg::TmIncludesCodes, &[("count", LISTED.into())])),
            body: Some(clause_codes(
                &v.locale,
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
                Some(MoreCodes::of(&v.locale, 3, LISTED, 25, "#codings")),
            )),
            actions: None,
            issues: Vec::new(),
        },
    );
    let section = panel_section(
        &v.locale,
        Some(SectionHead {
            title: v.tr(Msg::TmSelects),
            count: Some(HeadingCount::All(3)),
            actions: None,
        }),
        clauses(Html::join([include, exclude, listed])),
    );
    panel_sections_card(v.tr(Msg::TmValueSet), section)
}

/// A select of `options`, each its value and words, `chosen` selected, and
/// refused with its error line when `refused`.
fn select(
    id: &str,
    name: &str,
    options: Vec<(String, String)>,
    chosen: &str,
    refused: bool,
) -> Html {
    let options: Html = options
        .into_iter()
        .map(|(value, words)| {
            let selected = if value == chosen { " selected" } else { "" };
            Html::raw(format!(
                "<option value=\"{}\"{selected}>{}</option>",
                Html::text(value),
                Html::text(words)
            ))
        })
        .collect();
    let refusal = if refused {
        format!(
            " aria-invalid=\"true\" aria-describedby=\"{}\"",
            error_id(id)
        )
    } else {
        String::new()
    };
    Html::raw(format!(
        "<select id=\"{}\" name=\"{}\" class=\"{SELECT}\"{refusal}>{options}</select>",
        Html::text(id),
        Html::text(name)
    ))
}

/// One filter row of the include in the composer, `key` its key.
fn filter_row(v: &Visit, key: &str, value: &str) -> RepeatRow {
    let property = format!("filter-{key}-property");
    let operator = format!("filter-{key}-op");
    let typed = format!("filter-{key}-value");
    let fields = Html::join([
        Field::new(property.clone(), v.tr(Msg::TmProperty)).view(select(
            &property,
            &format!("filter[{key}][property]"),
            vec![
                (String::from("concept"), v.tr(Msg::TmPropConcept)),
                (String::from("parent"), v.tr(Msg::TmPropParent)),
                (String::from("inactive"), v.tr(Msg::TmPropInactive)),
            ],
            "concept",
false,
        )),
        Field::new(operator.clone(), v.tr(Msg::TmOperator)).view(select(
            &operator,
            &format!("filter[{key}][op]"),
            vec![
                (String::from("is-a"), v.tr(Msg::TmOpIsA)),
                (String::from("descendent-of"), v.tr(Msg::TmOpBelow)),
                (String::from("="), v.tr(Msg::TmOpEquals)),
            ],
            "is-a",
false,
        )),
        Field::new(typed.clone(), v.tr(Msg::TmValue)).view(Html::raw(format!(
            "<input id=\"{typed}\" name=\"filter[{key}][value]\" type=\"text\" class=\"{INPUT} fui-mono\" value=\"{}\">",
            Html::text(value)
        ))),
    ]);
    RepeatRow {
        key: key.to_owned(),
        heading: None,
        fields,
        end: RowEnd::Remove {
            label: v.tr(Msg::TmRemoveFilter),
            name: String::from("remove-filter"),
            value: key.to_owned(),
        },
        after: None,
    }
}

/// The Remove of a code of a clause in the composer.
fn remove_code(v: &Visit, code: &str) -> Html {
    let label = Html::text(v.tr_with(Msg::TmRemoveCode, &[("value", code.to_owned().into())]));
    Html::raw(format!(
        "<button type=\"submit\" class=\"fui-icon-button\" name=\"remove-code\" value=\"{}\" formnovalidate aria-label=\"{label}\" title=\"{label}\">{}</button>",
        Html::text(code),
        icon(Glyph::X, "16")
    ))
}

/// The Remove of a whole clause in the composer.
fn remove_clause(v: &Visit, key: &str) -> Html {
    let label = Html::text(v.tr(Msg::TmRemoveClause));
    Html::raw(format!(
        "<button type=\"submit\" class=\"fui-icon-button\" name=\"remove-clause\" value=\"{}\" formnovalidate aria-label=\"{label}\" title=\"{label}\">{}</button>",
        Html::text(key),
        icon(Glyph::TRASH, "16")
    ))
}

/// The code system and version fields of the clause `key` in the
/// composer.
fn source_fields(v: &Visit, key: &str) -> Html {
    let system = format!("clause-{key}-system");
    let version = format!("clause-{key}-version");
    clause_fields(Html::join([
        Field::new(system.clone(), v.tr(Msg::TmCodeSystem)).view(select(
            &system,
            &format!("clause[{key}][system]"),
            vec![(String::from("sf"), FINDINGS.to_owned())],
            "sf",
            false,
        )),
        Field::new(version.clone(), v.tr(Msg::TmVersion)).view(select(
            &version,
            &format!("clause[{key}][version]"),
            vec![(String::from("3"), v.tr(Msg::TmVersionDefault))],
            "3",
            false,
        )),
    ]))
}

/// The include of the composer: its source, a filter row, a code and the
/// invariant the two break together.
fn composed_include(v: &Visit) -> Html {
    let source = findings_source(v);
    let filters = repeat(Repeat {
        id: String::from("include-filters"),
        label: v.tr(Msg::TmFiltersOf),
        fields: Fields::Three,
        head: Vec::new(),
        rows: vec![filter_row(v, "f1", CHEST)],
        token: ROW,
        blank: Some(filter_row(v, ROW, "")),
        add: Some(RepeatAdd {
            label: v.tr(Msg::TmAddFilter),
            name: String::from("add-filter"),
            value: String::from("include"),
        }),
        empty: v.tr(Msg::TmNoFilters),
        form: None,
    });
    clause(
        &v.locale,
        Clause {
            kind: ClauseKind::Include,
            title: Html::text(&source),
            source: source.clone(),
            sentence: Html::text(v.tr_with(
                Msg::TmIncludesBelow,
                &[("value", CHEST.into()), ("name", display(CHEST).into())],
            )),
            body: Some(Html::join([
                source_fields(v, "c1"),
                clause_part(v.tr(Msg::TmFilters)),
                filters,
                clause_part(v.tr(Msg::TmCodes)),
                clause_codes(
                    &v.locale,
                    vec![ClauseCode {
                        coding: finding(FEVER),
                        remove: Some(remove_code(v, FEVER)),
                    }],
                    None,
                ),
            ])),
            actions: Some(remove_clause(v, "c1")),
            issues: vec![ClauseIssue {
                key: String::from("vsd-3"),
                text: v.tr(Msg::TmVsd3),
            }],
        },
    )
}

/// A value set's clauses in its composer: the include with a filter row,
/// a code and the invariant they break together, and the exclude with its
/// code.
fn composer(v: &Visit) -> Html {
    let include = composed_include(v);
    let source = findings_source(v);
    let exclude = clause(
        &v.locale,
        Clause {
            kind: ClauseKind::Exclude,
            title: Html::text(&source),
            source,
            sentence: Html::raw(format!(
                "{} {}",
                Html::text(v.tr_with(Msg::TmExcludesCodes, &[("count", 1.into())])),
                coding(&v.locale, finding(WHEEZE), Layout::Inline)
            )),
            body: Some(Html::join([
                source_fields(v, "c2"),
                clause_part(v.tr(Msg::TmCodes)),
                clause_codes(
                    &v.locale,
                    vec![ClauseCode {
                        coding: finding(WHEEZE),
                        remove: Some(remove_code(v, WHEEZE)),
                    }],
                    None,
                ),
            ])),
            actions: Some(remove_clause(v, "c2")),
            issues: Vec::new(),
        },
    );
    let add = |value: &str, words: Msg| {
        Html::raw(format!(
            "<button type=\"submit\" class=\"{}\" name=\"add-clause\" value=\"{value}\">{}{}</button>",
            Variant::Ghost.small(),
            icon(Glyph::PLUS, "16"),
            Html::text(v.tr(words))
        ))
    };
    let adds = Container::AddRow.wrap(Html::join([
        add("include", Msg::TmAddInclude),
        add("exclude", Msg::TmAddExclude),
    ]));
    let section = panel_section(
        &v.locale,
        Some(SectionHead {
            title: v.tr(Msg::TmComposer),
            count: Some(HeadingCount::All(2)),
            actions: None,
        }),
        Html::join([
            section_intro(v.tr(Msg::TmComposerIntro)),
            clauses(Html::join([include, exclude])),
            adds,
        ]),
    );
    Html::raw(format!(
        "<form method=\"post\" action=\"#\" id=\"value-set-composer\">{}</form>",
        panel_sections_card(v.tr(Msg::TmEditValueSet), section)
    ))
}

/// What each root of the synthetic server declares, its last root down.
fn matrix(v: &Visit) -> Html {
    let roots: Vec<MatrixRoot> = ROOTS
        .iter()
        .map(|(name, release, answered)| MatrixRoot {
            name: (*name).to_owned(),
            release: (*release).to_owned(),
            answered: *answered,
        })
        .collect();
    let same = |value: &str| -> Vec<Option<Html>> {
        ROOTS.iter().map(|_| Some(Html::text(value))).collect()
    };
    let rows = vec![
        MatrixRow {
            name: Html::text(v.tr(Msg::TmSoftware)),
            cells: same(SOFTWARE),
        },
        MatrixRow {
            name: Html::text(v.tr(Msg::TmBase)),
            cells: ROOTS
                .iter()
                .map(|(name, _, _)| {
                    Some(Html::raw(format!(
                        "<code>{}</code>",
                        Html::text(base(name))
                    )))
                })
                .collect(),
        },
        MatrixRow {
            name: Html::text(v.tr(Msg::TmCodeSystems)),
            cells: same(CODE_SYSTEMS),
        },
        MatrixRow {
            name: Html::text(v.tr(Msg::TmSecurity)),
            cells: same(SECURITY),
        },
        MatrixRow {
            name: Html::text(v.tr(Msg::TmServes)),
            cells: ROOTS
                .iter()
                .map(|(name, _, _)| (*name == BALLOT_ROOT).then(|| Html::text(v.tr(Msg::TmBallot))))
                .collect(),
        },
    ];
    let body = root_matrix(
        &v.locale,
        RootMatrix {
            caption: v.tr(Msg::TmDeclares),
            corner: v.tr(Msg::TmDeclaration),
            roots,
            rows,
        },
    );
    let section = panel_section(
        &v.locale,
        Some(SectionHead {
            title: v.tr(Msg::TmDeclares),
            count: None,
            actions: None,
        }),
        body,
    );
    panel_sections_card(v.tr(Msg::TmServer), section)
}

/// A mapping of the synthetic concept map.
fn mapping(row: &crate::terminology_data::Mapped) -> Mapping {
    let (source, relationship, target, comment) = *row;
    Mapping {
        source: Coding::new(source, legacy_display(source)),
        relationship,
        target: target.map(finding),
        comment: comment.map(str::to_owned),
    }
}

/// The concept map's mappings as its page reads them, in one group.
fn mappings(v: &Visit) -> Html {
    let columns = mapping_columns(&v.locale, true);
    let heading = v.tr_with(
        Msg::TmGroup,
        &[("name", LEGACY.into()), ("value", FINDINGS.into())],
    );
    let group = group_row(&columns, Html::text(heading));
    let rows: Html = MAPPINGS
        .iter()
        .map(|row| mapping_row(&v.locale, &columns, mapping(row)))
        .collect();
    let caption = v.tr(Msg::TmMappings);
    let table = data_table(DataTable::new(
        &caption,
        &columns,
        Body::Bodies(Html::raw(format!("<tbody>{group} {rows}</tbody>"))),
    ));
    let section = panel_section(
        &v.locale,
        Some(SectionHead {
            title: v.tr(Msg::TmMappings),
            count: Some(HeadingCount::All(
                u64::try_from(MAPPINGS.len()).unwrap_or(u64::MAX),
            )),
            actions: None,
        }),
        table,
    );
    panel_sections_card(v.tr(Msg::TmConceptMap), section)
}

/// The relationship choices of a target row in the editor, each its R4
/// code and its words with a capital.
fn relationship_options(v: &Visit) -> Vec<(String, String)> {
    let mut options = vec![(String::new(), v.tr(Msg::TmChoose))];
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
            let words = relationship.words(&v.locale);
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
fn target_row(v: &Visit, source: &str, key: &str, target: Target, missing: bool) -> RepeatRow {
    let (code, relationship, comment) = target;
    let target_id = format!("target-{source}-{key}-code");
    let relationship_id = format!("target-{source}-{key}-rel");
    let comment_id = format!("target-{source}-{key}-comment");
    let name = |part: &str| format!("target[{source}][{key}][{part}]");
    let chosen = if missing { "" } else { relationship };
    let relation = select(
        &relationship_id,
        &name("relationship"),
        relationship_options(v),
        chosen,
        missing,
    );
    let fields = Html::join([
        Field::new(target_id.clone(), v.tr(Msg::TmTargetCode)).view(Html::raw(format!(
            "<input id=\"{target_id}\" name=\"{}\" type=\"text\" class=\"{INPUT} fui-mono\" value=\"{}\">",
            Html::text(name("code")),
            Html::text(code)
        ))),
        Field::new(relationship_id.clone(), v.tr(Msg::TmRelationship)).view(relation),
        Field::new(comment_id.clone(), v.tr(Msg::TmComment)).view(Html::raw(format!(
            "<input id=\"{comment_id}\" name=\"{}\" type=\"text\" class=\"{INPUT}\" value=\"{}\" placeholder=\"{}\">",
            Html::text(name("comment")),
            Html::text(comment),
            Html::text(v.tr(Msg::TmOptional))
        ))),
    ]);
    let after = missing.then(|| {
        let line = v.tr_with(
            Msg::TmChooseRelationship,
            &[("value", code.to_owned().into())],
        );
        Html::raw(format!(
            "<p id=\"{}\" class=\"{ERROR}\">{}</p>",
            error_id(&relationship_id),
            Html::text(line)
        ))
    });
    RepeatRow {
        key: key.to_owned(),
        heading: None,
        fields,
        end: RowEnd::Remove {
            label: v.tr_with(Msg::TmRemoveTarget, &[("value", source.to_owned().into())]),
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
fn element(v: &Visit, source: &str, target: Target, missing: bool) -> Html {
    let label = v.tr_with(Msg::TmTargetsOf, &[("value", source.to_owned().into())]);
    let targets = repeat(Repeat {
        id: format!("targets-{source}"),
        label,
        fields: Fields::Three,
        head: Vec::new(),
        rows: vec![target_row(v, source, "t1", target, missing)],
        token: ROW,
        blank: Some(target_row(v, source, ROW, ("", "", ""), false)),
        add: Some(RepeatAdd {
            label: v.tr(Msg::TmAddTarget),
            name: String::from("add-target"),
            value: source.to_owned(),
        }),
        empty: v.tr(Msg::TmNoTargets),
        form: None,
    });
    let remove = Html::text(v.tr_with(Msg::TmRemoveSource, &[("value", source.to_owned().into())]));
    let actions = Html::raw(format!(
        "<button type=\"submit\" class=\"fui-icon-button\" name=\"remove-source\" value=\"{}\" formnovalidate aria-label=\"{remove}\" title=\"{remove}\">{}</button>",
        Html::text(source),
        icon(Glyph::TRASH, "16")
    ));
    map_element(
        &v.locale,
        MapElement {
            source: Coding::new(source, legacy_display(source)),
            actions: Some(actions),
            targets,
            after: None,
        },
    )
}

/// The concept map's editor: one group, each source code a block with its
/// target rows, the second missing its relationship.
fn map_editor(v: &Visit) -> Html {
    let fields = clause_fields(Html::join([
        Field::new("group-source", v.tr(Msg::TmSourceSystem)).view(select(
            "group-source",
            "group[source]",
            vec![(String::from("legacy"), LEGACY.to_owned())],
            "legacy",
            false,
        )),
        Field::new("group-target", v.tr(Msg::TmTargetSystem)).view(select(
            "group-target",
            "group[target]",
            vec![(String::from("sf"), FINDINGS.to_owned())],
            "sf",
            false,
        )),
    ]));
    let elements = map_elements(Html::join([
        element(v, LEGACY_COUGH, (COUGH, "equivalent", ""), false),
        element(v, LEGACY_CHEST, (WHEEZE, "narrower", ""), true),
    ]));
    let section = panel_section(
        &v.locale,
        Some(SectionHead {
            title: v.tr_with(
                Msg::TmEditorGroup,
                &[("name", LEGACY.into()), ("value", FINDINGS.into())],
            ),
            count: None,
            actions: None,
        }),
        Html::join([fields, clause_part(v.tr(Msg::TmCodes)), elements]),
    );
    Html::raw(format!(
        "<form method=\"post\" action=\"#\" id=\"concept-map-editor\">{}</form>",
        panel_sections_card(v.tr(Msg::TmEditConceptMap), section)
    ))
}

/// The terminology page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let root = format!("/{}", v.brand.key());
    let header = page_heading(
        &v.locale,
        Header {
            title: v.tr(Msg::Terminology).into(),
            subtitle: Some(v.tr(Msg::TmSubtitle).into()),
            subtitle_view: None,
            crumbs: vec![Crumb::new(v.tr(Msg::Gallery), root)],
            mono: false,
            badge: None,
        },
        ActionGroup::default(),
    );
    let body = Container::Stack.wrap(Html::join([
        codings(v),
        clauses_read(v),
        composer(v),
        matrix(v),
        mappings(v),
        map_editor(v),
    ]));
    Page::shell(v.tr(Msg::Terminology), Html::join([header, body]))
}
