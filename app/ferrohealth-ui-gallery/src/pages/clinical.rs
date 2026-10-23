// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The clinical records page.
//!
//! It holds the subject line masked with its Reveal, with no external
//! subject and with a cleared state (E1); the version history with an
//! older version open, At a time under it, and a deleted composition (E2);
//! a composition read with nested clusters (E3); the query builder with a
//! nested negated group (E6); one answer as a table and as a chart (E7);
//! and the scopes an app holds, narrow, broad, malformed and refused (E8).
//! Its record is the synthetic data of [`crate::clinical_data`].

use ferrohealth_ui::calendar::{Bounds, Day, Weekday};
use ferrohealth_ui::island::DatePicker;
use ferrohealth_ui::table::view::{Body, Column, DataTable, Role, cell, data_table, title_cell};
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::chart::{ACCENT, INFO, LineChart, Series};
use ferrohealth_ui::view::coding::{Coding, Layout, coding};
use ferrohealth_ui::view::conditions::{
    Choice, Condition, ConditionGroup, Match, Operand, condition_groups,
};
use ferrohealth_ui::view::reading::{Element, Level, Reading, Section, Value, reading};
use ferrohealth_ui::view::results::{Export, ResultsPane, Shown, Switch, results_pane};
use ferrohealth_ui::view::scope::{Grant, ScopeGrant, scope_grants};
use ferrohealth_ui::view::subject::{Subject, SubjectLine, subject_line};
use ferrohealth_ui::view::surface::{
    Crumb, Header, SectionHead, page_heading, panel_section, panel_sections_card, stack,
    titled_panel,
};
use ferrohealth_ui::view::template::list::HeadingCount;
use ferrohealth_ui::view::time::{relative, timestamp};
use ferrohealth_ui::view::versions::{
    AtTime, Standing, Version, VersionHistory, at_time_label, older_version, version_history,
};
use ferrohealth_ui::widget::chart::{ChartWords, line_chart};
use ferrohealth_ui::widget::date::{DateField, DateWords};
use ferrohealth_ui::widget::identifier::{
    Domain, Identifier, IdentifierWords, KEEP, Masked, Reveal, Revealed, identifier,
    identifier_static,
};
use leptos::prelude::*;

use crate::clinical_data::{
    AnswerRow, BLOOD_PRESSURE, BODY_TEMPERATURE, CALIBRATED, CALIBRATION, CLINICIAN,
    CODED_OPERATORS, COLUMNS, COMPOSER, Coded, DELETION, DEVICE, DEVICE_NAME, DIASTOLIC, FOREIGN,
    NOW, PATH_POSITION, PATH_RATE, PATH_SYSTOLIC, PATH_TEMPERATURE, POSITION, POSITIONS, PULSE,
    QUANTITY_OPERATORS, Quantity, RATE, REGULARITY, ROWS, SCOPES, SETTING, START_TIME,
    SUBJECT_NAMESPACE, SUBJECT_VALUE, SYSTOLIC, TEMPERATURE, VERSIONS, VersionRow,
};
use crate::locale::{Msg, tr, tr_with};

/// The seconds a revealed subject shows.
const REVEAL_SECONDS: u32 = 30;

/// The subject's identifier, masked as the console's server masks it.
fn masked_subject() -> Masked {
    Masked::new(Domain::new(SUBJECT_NAMESPACE), SUBJECT_VALUE, KEEP)
}

/// The interactive subject identifier the island
/// [`crate::islands::SubjectIdentifier`] draws: Reveal answered after a
/// short pause, as the console's audited round trip would be.
pub(crate) fn subject_identifier(brand: &str, ident: IdentifierWords) -> AnyView {
    let state = RwSignal::new(Reveal::Masked);
    let on_reveal = Callback::new(move |()| {
        set_timeout(
            move || {
                state.set(Reveal::Shown(Revealed::new(
                    SUBJECT_VALUE.to_owned(),
                    REVEAL_SECONDS,
                )));
            },
            std::time::Duration::from_millis(400),
        );
    });
    identifier(
        Identifier {
            id: String::from("subject-identifier"),
            masked: masked_subject(),
            show_domain: true,
            state,
            fallback: format!("/{brand}/clinical#subject"),
            words: ident,
        },
        on_reveal,
    )
}

/// The subject line in its three states: masked with its Reveal, with no
/// external subject, and with a cleared state.
fn subjects(brand: &str) -> AnyView {
    let brand = brand.to_owned();
    let masked = subject_line(SubjectLine {
        subject: Subject::External(
            view! {
                <crate::islands::SubjectIdentifier brand ident=IdentifierWords::of_page() />
            }
            .into_any(),
        ),
        queryable: true,
        modifiable: true,
    });
    let none = subject_line(SubjectLine {
        subject: Subject::None,
        queryable: true,
        modifiable: true,
    });
    let cleared = subject_line(SubjectLine {
        subject: Subject::External(identifier_static(&masked_subject(), true)),
        queryable: false,
        modifiable: true,
    });
    let section = |title: Msg, body: AnyView| {
        panel_section(
            Some(SectionHead {
                title: tr(title),
                count: None,
                actions: None,
            }),
            view! { <div class="fui-panel-body">{body}</div> },
        )
    };
    view! {
        <div id="subject">
            {panel_sections_card(
                tr(Msg::ClSubject),
                view! {
                    {section(Msg::ClSubjectMasked, masked)}
                    {section(Msg::ClSubjectNone, none)}
                    {section(Msg::ClSubjectCleared, cleared)}
                },
            )}
        </div>
    }
    .into_any()
}

/// One row of the history.
fn version(row: &VersionRow, standing: Standing, read: bool, change: Msg) -> Version {
    let href = (standing != Standing::Deleted).then(|| format!("#version-{}", row.number));
    let contribution = row.contribution;
    Version {
        number: row.number.to_owned(),
        standing,
        href,
        change: tr(change),
        committer: String::from(if standing == Standing::Deleted {
            CLINICIAN
        } else {
            COMPOSER
        }),
        when: relative(row.at, NOW).into_any(),
        contribution: view! {
            <a href="#contribution">
                <code>{contribution}</code>
            </a>
        }
        .into_any(),
        read,
    }
}

/// The change type of the version `number` of the composition.
fn change_of(number: &str) -> Msg {
    if number == "1" {
        Msg::ClChangeCreation
    } else {
        Msg::ClChangeModification
    }
}

/// The day the gallery takes as today.
#[expect(
    clippy::expect_used,
    reason = "2026-10-21 is a day of the calendar, by inspection"
)]
fn today() -> Day {
    Day::new(2026, 10, 21).expect("the gallery's today should be a day")
}

/// The At a time field: a date and time that forbids the future.
fn at_field() -> AnyView {
    let field = DateField {
        id: String::from("history-at"),
        name: String::from("at"),
        label: at_time_label().into(),
        value: String::from("2026-10-21 08:05"),
        help: None,
        error: None,
        today: today(),
        bounds: Bounds {
            earliest: None,
            latest: Some(today()),
        },
        first_day: Weekday::Monday,
        with_time: true,
        quick: Vec::new(),
        chosen: None,
        words: DateWords::of_page(true),
    };
    view! { <DatePicker field /> }.into_any()
}

/// The version history with version 2 open, and a deleted composition's.
fn histories(brand: &str) -> AnyView {
    let latest = format!("/{brand}/clinical#versions");
    let rows = VERSIONS
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let standing = if index == 0 {
                Standing::Current
            } else {
                Standing::Older
            };
            version(row, standing, row.number == "2", change_of(row.number))
        })
        .collect();
    let count = u32::try_from(VERSIONS.len()).unwrap_or(u32::MAX);
    let open = version_history(VersionHistory {
        caption: tr(Msg::ClVersionsTitle),
        versions: rows,
        at_time: Some(AtTime {
            action: format!("/{brand}/clinical"),
            field: at_field(),
            opens: tr(Msg::ClVersionsOpens),
        }),
    });
    let deleted_rows = std::iter::once(version(
        &DELETION,
        Standing::Deleted,
        false,
        Msg::ClChangeDeletion,
    ))
    .chain(
        VERSIONS
            .iter()
            .map(|row| version(row, Standing::Older, false, change_of(row.number))),
    )
    .collect();
    let deleted = version_history(VersionHistory {
        caption: tr(Msg::ClVersionsDeleted),
        versions: deleted_rows,
        at_time: None,
    });
    view! {
        <div id="versions">
            {panel_sections_card(
                tr(Msg::ClVersions),
                view! {
                    {panel_section(None, older_version("2", count, latest))}
                    {panel_section(
                        Some(SectionHead {
                            title: tr(Msg::ClVersionsTitle),
                            count: Some(HeadingCount::All(u64::from(count))),
                            actions: None,
                        }),
                        open,
                    )}
                    {panel_section(
                        Some(SectionHead {
                            title: tr(Msg::ClVersionsDeleted),
                            count: Some(HeadingCount::All(u64::from(count) + 1)),
                            actions: None,
                        }),
                        deleted,
                    )}
                },
            )}
        </div>
    }
    .into_any()
}

/// A quantity's value.
fn quantity((magnitude, unit, ucum): Quantity) -> Value {
    Value::Quantity {
        magnitude: magnitude.to_owned(),
        unit: unit.to_owned(),
        ucum: ucum.to_owned(),
    }
}

/// A coded value's value.
fn coded((display, code, terminology): Coded) -> Value {
    Value::Coded(coding(
        Coding::new(code, display).system(terminology),
        Layout::Inline,
    ))
}

/// An element `label` holding `value`.
fn element(label: Msg, value: Value) -> Element {
    Element {
        label: tr(label),
        value,
    }
}

/// The composition read with nested clusters.
fn composition() -> AnyView {
    let context = Section {
        title: tr(Msg::ClContext),
        node: None,
        elements: vec![
            element(
                Msg::ClStartTime,
                Value::Other(timestamp(START_TIME).into_any()),
            ),
            element(Msg::ClSetting, coded(SETTING)),
            element(Msg::ClComposer, Value::Text(String::from(COMPOSER))),
        ],
        sections: Vec::new(),
    };
    let [systolic, _] = SYSTOLIC;
    let [diastolic, _] = DIASTOLIC;
    let calibration = Section {
        title: tr(Msg::ClCalibration),
        node: Some(String::from(CALIBRATION)),
        elements: vec![element(
            Msg::ClLastCalibrated,
            Value::Other(timestamp(CALIBRATED).into_any()),
        )],
        sections: Vec::new(),
    };
    let device = Section {
        title: tr(Msg::ClDevice),
        node: Some(String::from(DEVICE)),
        elements: vec![element(
            Msg::ClDeviceName,
            Value::Text(String::from(DEVICE_NAME)),
        )],
        sections: vec![calibration],
    };
    let blood = Section {
        title: tr(Msg::ClBloodPressure),
        node: Some(String::from(BLOOD_PRESSURE)),
        elements: vec![
            element(Msg::ClSystolic, quantity(systolic)),
            element(Msg::ClDiastolic, quantity(diastolic)),
            element(Msg::ClPosition, coded(POSITION)),
            element(Msg::ClCuffSize, Value::Null(tr(Msg::ClNoInformation))),
        ],
        sections: vec![device],
    };
    let pulse = Section {
        title: tr(Msg::ClPulse),
        node: Some(String::from(PULSE)),
        elements: vec![
            element(Msg::ClRate, quantity(RATE)),
            element(Msg::ClRegularity, coded(REGULARITY)),
        ],
        sections: Vec::new(),
    };
    let temperature = Section {
        title: tr(Msg::ClBodyTemperature),
        node: Some(String::from(TEMPERATURE)),
        elements: vec![element(Msg::ClTemperature, quantity(BODY_TEMPERATURE))],
        sections: Vec::new(),
    };
    let folded = view! {
        {tr(Msg::ClFolded)}
        " "
        <a href="#reading">{tr(Msg::ClShowDocument)}</a>
    }
    .into_any();
    view! {
        <div id="reading">
            {titled_panel(
                tr(Msg::ClReading),
                (),
                reading(Reading {
                    sections: vec![context, blood, pulse, temperature],
                    level: Level::Three,
                    folded: Some(folded),
                }),
            )}
        </div>
    }
    .into_any()
}

/// A choice of `options`, each its own words, `chosen` chosen.
fn plain_choice(options: &[&str], chosen: &str) -> Choice {
    Choice {
        options: options
            .iter()
            .map(|each| ((*each).to_owned(), (*each).to_owned()))
            .collect(),
        chosen: chosen.to_owned(),
    }
}

/// A condition `key` on `path`, compared by `operator` with `operand`.
fn condition(
    key: &str,
    (words, archetype_path): (&str, &str),
    operator: Choice,
    operand: Operand,
) -> Condition {
    Condition {
        key: key.to_owned(),
        path: plain_choice(&[words], words),
        archetype_path: archetype_path.to_owned(),
        operator,
        operand,
    }
}

/// A typed value `value` in `unit`.
fn typed(value: &str, unit: &str) -> Operand {
    Operand::Typed {
        value: value.to_owned(),
        unit: Some(unit.to_owned()),
    }
}

/// The query builder: All of a pressure and a position, and a negated Any
/// of a pulse and a temperature.
fn builder(brand: &str) -> AnyView {
    let inner = ConditionGroup {
        key: String::from("g2"),
        number: 2,
        matching: Match::Any,
        negated: true,
        conditions: vec![
            condition(
                "c3",
                PATH_RATE,
                plain_choice(QUANTITY_OPERATORS, ">"),
                typed("100", RATE.1),
            ),
            condition(
                "c4",
                PATH_TEMPERATURE,
                plain_choice(QUANTITY_OPERATORS, ">"),
                typed("38", BODY_TEMPERATURE.1),
            ),
        ],
        groups: Vec::new(),
    };
    let positions = Choice {
        options: POSITIONS
            .iter()
            .map(|(code, words)| ((*code).to_owned(), (*words).to_owned()))
            .collect(),
        chosen: String::from(POSITION.1),
    };
    let outer = ConditionGroup {
        key: String::from("g1"),
        number: 1,
        matching: Match::All,
        negated: false,
        conditions: vec![
            condition(
                "c1",
                PATH_SYSTOLIC,
                plain_choice(QUANTITY_OPERATORS, ">"),
                typed("140", SYSTOLIC[0].1),
            ),
            condition(
                "c2",
                PATH_POSITION,
                plain_choice(CODED_OPERATORS, "="),
                Operand::Coded(positions),
            ),
        ],
        groups: vec![inner],
    };
    view! {
        <div id="query">
            {titled_panel(
                tr(Msg::ClQuery),
                (),
                view! {
                    <form method="post" action=format!("/{brand}/clinical") class="fui-panel-body">
                        {condition_groups(outer)}
                    </form>
                },
            )}
        </div>
    }
    .into_any()
}

/// The answer as its table.
fn answer_table() -> AnyView {
    let [time, systolic, diastolic] = COLUMNS;
    let columns = [
        Column::text("time", time).mono().role(Role::Title),
        Column::text("systolic", systolic).mono().numeric(),
        Column::text("diastolic", diastolic)
            .mono()
            .numeric()
            .role(Role::Quiet),
    ];
    let rows = ROWS
        .iter()
        .map(|row| {
            let [_, high_column, low_column] = &columns;
            view! {
                <tr>
                    {title_cell(None, view! { <code>{row.at}</code> }, None)}
                    {cell(high_column, row.systolic.to_string())}
                    {cell(low_column, row.diastolic.to_string())}
                </tr>
            }
        })
        .collect_view()
        .into_any();
    let caption = tr(Msg::ClResultsTitle);
    data_table(DataTable {
        caption: &caption,
        columns: &columns,
        sorting: ferrohealth_ui::table::view::Sorting::default(),
        body: Body::Rows(rows),
    })
    .into_any()
}

/// The day and the short month of `row`, such as "21 Oct".
fn day_label(row: &AnswerRow) -> String {
    let msg = match row.month {
        8 => Msg::ClAugust,
        9 => Msg::ClSeptember,
        _ => Msg::ClOctober,
    };
    tr_with(msg, &[("day", u32::from(row.day).into())])
}

/// The answer as the base line chart: one series per numeric column over
/// the time column, the oldest first.
fn answer_chart() -> AnyView {
    let [_, systolic, diastolic] = COLUMNS;
    let oldest_first = ROWS.iter().rev().collect::<Vec<_>>();
    let points = |pick: fn(&AnswerRow) -> f64| {
        (0_u32..)
            .zip(&oldest_first)
            .map(|(index, row)| (f64::from(index), pick(row)))
            .collect::<Vec<_>>()
    };
    let last = oldest_first.len().saturating_sub(1);
    let ticks = [0, last.div_euclid(2), last]
        .into_iter()
        .filter_map(|index| {
            let row = oldest_first.get(index)?;
            let at = u32::try_from(index).ok()?;
            Some((f64::from(at), day_label(row)))
        })
        .collect();
    let chart = LineChart {
        id: "clinical-answer",
        label: tr(Msg::ClChartLabel),
        summary: tr(Msg::ClChartSummary),
        series: vec![
            Series {
                name: systolic.to_owned(),
                points: points(|row| row.systolic),
                palette: ACCENT,
            },
            Series {
                name: diastolic.to_owned(),
                points: points(|row| row.diastolic),
                palette: INFO,
            },
        ],
        x_ticks: ticks,
        x_labels: (0_u32..)
            .zip(&oldest_first)
            .map(|(index, row)| (f64::from(index), day_label(row)))
            .collect(),
        y_range: None,
    };
    line_chart(&chart, ChartWords::of_page())
}

/// One answer as a table, its switch in the URL, and as a chart, its switch
/// a form the console answers from its session, then an answer with no row.
fn results(brand: &str) -> AnyView {
    let count = u64::try_from(ROWS.len()).unwrap_or(u64::MAX);
    let page = format!("/{brand}/clinical");
    let table = results_pane(ResultsPane {
        title: tr(Msg::ClResultsTitle),
        count,
        numeric: true,
        shown: Shown::Table,
        switch: Switch::Url {
            table: format!("{page}#results"),
            chart: format!("{page}#results-chart"),
        },
        csv: Export::Link(format!("{page}#results")),
        json: Export::Link(format!("{page}#results")),
        window: tr(Msg::ClResultsWindow),
        body: answer_table(),
    });
    let chart = results_pane(ResultsPane {
        title: tr(Msg::ClResultsTitle),
        count,
        numeric: true,
        shown: Shown::Chart,
        switch: Switch::Session {
            action: page.clone(),
            back: format!("{page}#results-chart"),
        },
        csv: Export::Link(format!("{page}#results-chart")),
        json: Export::Link(format!("{page}#results-chart")),
        window: tr(Msg::ClResultsWindow),
        body: answer_chart(),
    });
    let empty = results_pane(ResultsPane {
        title: tr(Msg::ClResultsTitle),
        count: 0,
        numeric: true,
        shown: Shown::Table,
        switch: Switch::Url {
            table: format!("{page}#results-empty"),
            chart: format!("{page}#results-empty"),
        },
        csv: Export::Link(format!("{page}#results-empty")),
        json: Export::Link(format!("{page}#results-empty")),
        window: tr(Msg::ClResultsEmptyWindow),
        body: ().into_any(),
    });
    view! {
        <div id="results">{panel_sections_card(tr(Msg::ClResultsTable), table)}</div>
        <div id="results-chart">{panel_sections_card(tr(Msg::ClResultsChart), chart)}</div>
        <div id="results-empty">{panel_sections_card(tr(Msg::ClResultsEmpty), empty)}</div>
    }
    .into_any()
}

/// The scopes an app holds: an identity and a launch context, three
/// narrow (one constrained), one broad, one malformed and one the grammar
/// takes and the CDR refuses.
fn scopes() -> AnyView {
    let mut grants = SCOPES
        .iter()
        .map(|(raw, resource, broad)| ScopeGrant::read(*raw, |_| ((*resource).to_owned(), *broad)))
        .collect::<Vec<_>>();
    grants.push(ScopeGrant {
        raw: String::from(FOREIGN),
        grant: Grant::Refused(tr(Msg::ClNotOpenehr).into()),
    });
    view! {
        <div id="scopes">
            {titled_panel(tr(Msg::ClScopes), (), scope_grants(tr(Msg::ClScopesHeld), grants))}
        </div>
    }
    .into_any()
}

/// The clinical records page.
#[component]
pub fn ClinicalPage() -> impl IntoView {
    let brand = crate::app::route_brand().map_or_else(String::new, |brand| brand.key().to_owned());
    let root = if brand.is_empty() {
        String::new()
    } else {
        format!("/{brand}")
    };
    let header = page_heading(
        Header {
            title: tr(Msg::ClinicalRecords).into(),
            subtitle: Some(tr(Msg::ClSubtitle).into()),
            subtitle_view: None,
            crumbs: vec![Crumb::new(tr(Msg::Gallery), root)],
            mono: false,
            badge: None,
        },
        ActionGroup::none(),
    )
    .into_any();
    let parts = (
        subjects(&brand),
        histories(&brand),
        composition(),
        builder(&brand),
        results(&brand),
        scopes(),
    );
    view! {
        {super::title(tr(Msg::ClinicalRecords))}
        {header}
        {stack(
            view! {
                {parts.0}
                {parts.1}
                {parts.2}
                {parts.3}
                {parts.4}
                {parts.5}
            },
        )}
    }
}
