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
use ferrohealth_ui::html::Html;
use ferrohealth_ui::table::view::{Body, Column, DataTable, Role, cell, data_table, title_cell};
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::chart::{ACCENT, INFO, LineChart, Series};
use ferrohealth_ui::view::chart::{ChartWords, line_chart};
use ferrohealth_ui::view::coding::{Coding, Layout, coding};
use ferrohealth_ui::view::conditions::{
    Choice, Condition, ConditionGroup, Match, Operand, condition_groups,
};
use ferrohealth_ui::view::date::{DateField, DateWords, date_field};
use ferrohealth_ui::view::form::{TEXTAREA, field};
use ferrohealth_ui::view::identifier::{
    Domain, Identifier, IdentifierWords, KEEP, Masked, Reveal, Revealed, identifier,
    identifier_static,
};
use ferrohealth_ui::view::reading::{Element, Level, Reading, Section, Value, reading};
use ferrohealth_ui::view::results::{Export, ResultsPane, SHOWN, Shown, Switch, results_pane};
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

use crate::app::Page;
use crate::clinical_data::{
    AnswerRow, BLOOD_PRESSURE, BODY_TEMPERATURE, CALIBRATED, CALIBRATION, CLINICIAN,
    CODED_OPERATORS, COLUMNS, COMPOSER, Coded, DELETION, DEVICE, DEVICE_NAME, DIASTOLIC, FOREIGN,
    NOW, PATH_POSITION, PATH_RATE, PATH_SYSTOLIC, PATH_TEMPERATURE, POSITION, POSITIONS, PULSE,
    QUANTITY_OPERATORS, QUERY, Quantity, RATE, REGULARITY, ROWS, SCOPES, SETTING, START_TIME,
    SUBJECT_NAMESPACE, SUBJECT_VALUE, SYSTOLIC, TEMPERATURE, VERSIONS, VersionRow,
};
use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

/// The seconds a revealed subject shows.
const REVEAL_SECONDS: u32 = 30;

/// The subject's identifier, masked as the console's server masks it.
fn masked_subject() -> Masked {
    Masked::new(Domain::new(SUBJECT_NAMESPACE), SUBJECT_VALUE, KEEP)
}

/// The interactive subject identifier: Reveal leads to the page with the
/// subject's `reveal`, which draws the value as the console's audited round
/// trip would.
fn subject_identifier(v: &Visit, brand: &str, revealed: bool) -> Html {
    let state = if revealed {
        Reveal::Shown(Revealed::new(SUBJECT_VALUE.to_owned(), REVEAL_SECONDS))
    } else {
        Reveal::Masked
    };
    identifier(Identifier {
        id: String::from("subject-identifier"),
        masked: masked_subject(),
        show_domain: true,
        state,
        fallback: format!("/{brand}/clinical?reveal=subject#subject"),
        words: IdentifierWords::of(&v.locale),
    })
}

/// The subject line in its three states: masked with its Reveal, with no
/// external subject, and with a cleared state.
fn subjects(v: &Visit, brand: &str) -> Html {
    let masked = subject_line(
        &v.locale,
        SubjectLine {
            subject: Subject::External(subject_identifier(
                v,
                brand,
                v.param("reveal") == Some("subject"),
            )),
            queryable: true,
            modifiable: true,
        },
    );
    let none = subject_line(
        &v.locale,
        SubjectLine {
            subject: Subject::None,
            queryable: true,
            modifiable: true,
        },
    );
    let cleared = subject_line(
        &v.locale,
        SubjectLine {
            subject: Subject::External(identifier_static(&v.locale, &masked_subject(), true)),
            queryable: false,
            modifiable: true,
        },
    );
    let section = |title: Msg, body: Html| {
        panel_section(
            &v.locale,
            Some(SectionHead {
                title: v.tr(title),
                count: None,
                actions: None,
            }),
            h(format!("<div class=\"fui-panel-body\">{body}</div>")),
        )
    };
    h(format!(
        "<div id=\"subject\">{}</div>",
        panel_sections_card(
            v.tr(Msg::ClSubject),
            Html::join([
                section(Msg::ClSubjectMasked, masked),
                section(Msg::ClSubjectNone, none),
                section(Msg::ClSubjectCleared, cleared),
            ]),
        )
    ))
}

/// One row of the history.
fn version(v: &Visit, row: &VersionRow, standing: Standing, read: bool, change: Msg) -> Version {
    let href = (standing != Standing::Deleted).then(|| format!("#version-{}", row.number));
    let contribution = row.contribution;
    Version {
        number: row.number.to_owned(),
        standing,
        href,
        change: v.tr(change),
        committer: String::from(if standing == Standing::Deleted {
            CLINICIAN
        } else {
            COMPOSER
        }),
        when: relative(&v.locale, row.at, NOW),
        contribution: h(format!(
            "<a href=\"#contribution\"><code>{contribution}</code></a>"
        )),
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
fn at_field(v: &Visit) -> Html {
    let field = DateField {
        id: String::from("history-at"),
        name: String::from("at"),
        label: at_time_label(&v.locale).text().to_owned(),
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
        words: DateWords::of(&v.locale, true),
    };
    date_field(field)
}

/// The version history with version 2 open, and a deleted composition's.
fn histories(v: &Visit, brand: &str) -> Html {
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
            version(v, row, standing, row.number == "2", change_of(row.number))
        })
        .collect();
    let count = u32::try_from(VERSIONS.len()).unwrap_or(u32::MAX);
    let open = version_history(
        &v.locale,
        VersionHistory {
            caption: v.tr(Msg::ClVersionsTitle),
            versions: rows,
            at_time: Some(AtTime {
                action: format!("/{brand}/clinical"),
                field: at_field(v),
                opens: v.tr(Msg::ClVersionsOpens),
            }),
        },
    );
    let deleted_rows = std::iter::once(version(
        v,
        &DELETION,
        Standing::Deleted,
        false,
        Msg::ClChangeDeletion,
    ))
    .chain(
        VERSIONS
            .iter()
            .map(|row| version(v, row, Standing::Older, false, change_of(row.number))),
    )
    .collect();
    let deleted = version_history(
        &v.locale,
        VersionHistory {
            caption: v.tr(Msg::ClVersionsDeleted),
            versions: deleted_rows,
            at_time: None,
        },
    );
    h(format!(
        "<div id=\"versions\">{}</div>",
        panel_sections_card(
            v.tr(Msg::ClVersions),
            Html::join([
                panel_section(
                    &v.locale,
                    None,
                    older_version(&v.locale, "2", count, latest)
                ),
                panel_section(
                    &v.locale,
                    Some(SectionHead {
                        title: v.tr(Msg::ClVersionsTitle),
                        count: Some(HeadingCount::All(u64::from(count))),
                        actions: None,
                    }),
                    open,
                ),
                panel_section(
                    &v.locale,
                    Some(SectionHead {
                        title: v.tr(Msg::ClVersionsDeleted),
                        count: Some(HeadingCount::All(u64::from(count) + 1)),
                        actions: None,
                    }),
                    deleted,
                ),
            ]),
        )
    ))
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
fn coded(v: &Visit, (display, code, terminology): Coded) -> Value {
    Value::Coded(coding(
        &v.locale,
        Coding::new(code, display).system(terminology),
        Layout::Inline,
    ))
}

/// An element `label` holding `value`.
fn element(v: &Visit, label: Msg, value: Value) -> Element {
    Element {
        label: v.tr(label),
        value,
    }
}

/// The composition read with nested clusters.
fn composition(v: &Visit) -> Html {
    let context = Section {
        title: v.tr(Msg::ClContext),
        node: None,
        elements: vec![
            element(
                v,
                Msg::ClStartTime,
                Value::Other(timestamp(&v.locale, START_TIME)),
            ),
            element(v, Msg::ClSetting, coded(v, SETTING)),
            element(v, Msg::ClComposer, Value::Text(String::from(COMPOSER))),
        ],
        sections: Vec::new(),
    };
    let [systolic, _] = SYSTOLIC;
    let [diastolic, _] = DIASTOLIC;
    let calibration = Section {
        title: v.tr(Msg::ClCalibration),
        node: Some(String::from(CALIBRATION)),
        elements: vec![element(
            v,
            Msg::ClLastCalibrated,
            Value::Other(timestamp(&v.locale, CALIBRATED)),
        )],
        sections: Vec::new(),
    };
    let device = Section {
        title: v.tr(Msg::ClDevice),
        node: Some(String::from(DEVICE)),
        elements: vec![element(
            v,
            Msg::ClDeviceName,
            Value::Text(String::from(DEVICE_NAME)),
        )],
        sections: vec![calibration],
    };
    let blood = Section {
        title: v.tr(Msg::ClBloodPressure),
        node: Some(String::from(BLOOD_PRESSURE)),
        elements: vec![
            element(v, Msg::ClSystolic, quantity(systolic)),
            element(v, Msg::ClDiastolic, quantity(diastolic)),
            element(v, Msg::ClPosition, coded(v, POSITION)),
            element(v, Msg::ClCuffSize, Value::Null(v.tr(Msg::ClNoInformation))),
        ],
        sections: vec![device],
    };
    let pulse = Section {
        title: v.tr(Msg::ClPulse),
        node: Some(String::from(PULSE)),
        elements: vec![
            element(v, Msg::ClRate, quantity(RATE)),
            element(v, Msg::ClRegularity, coded(v, REGULARITY)),
        ],
        sections: Vec::new(),
    };
    let temperature = Section {
        title: v.tr(Msg::ClBodyTemperature),
        node: Some(String::from(TEMPERATURE)),
        elements: vec![element(v, Msg::ClTemperature, quantity(BODY_TEMPERATURE))],
        sections: Vec::new(),
    };
    let folded = h(format!(
        "{} <a href=\"#reading\">{}</a>",
        t(v.tr(Msg::ClFolded)),
        t(v.tr(Msg::ClShowDocument))
    ));
    h(format!(
        "<div id=\"reading\">{}</div>",
        titled_panel(
            v.tr(Msg::ClReading),
            Html::empty(),
            reading(
                &v.locale,
                Reading {
                    sections: vec![context, blood, pulse, temperature],
                    level: Level::Three,
                    folded: Some(folded),
                }
            ),
        )
    ))
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
fn builder(v: &Visit, brand: &str) -> Html {
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
    h(format!(
        "<div id=\"query\">{}</div>",
        titled_panel(
            v.tr(Msg::ClQuery),
            Html::empty(),
            h(format!(
                "<form method=\"post\" action=\"/{brand}/clinical\" class=\"fui-panel-body\">{}</form>",
                condition_groups(&v.locale, outer)
            )),
        )
    ))
}

/// The answer as its table.
fn answer_table(v: &Visit) -> Html {
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
            h(format!(
                "<tr>{}{}{}</tr>",
                title_cell(None, h(format!("<code>{}</code>", row.at)), None),
                cell(high_column, t(row.systolic.to_string())),
                cell(low_column, t(row.diastolic.to_string())),
            ))
        })
        .collect();
    let caption = v.tr(Msg::ClResultsTitle);
    data_table(DataTable {
        caption: &caption,
        columns: &columns,
        sorting: ferrohealth_ui::table::view::Sorting::default(),
        body: Body::Rows(rows),
    })
}

/// The day and the short month of `row`, such as "21 Oct".
fn day_label(v: &Visit, row: &AnswerRow) -> String {
    let msg = match row.month {
        8 => Msg::ClAugust,
        9 => Msg::ClSeptember,
        _ => Msg::ClOctober,
    };
    v.tr_with(msg, &[("day", u32::from(row.day).into())])
}

/// The answer as the base line chart: one series per numeric column over
/// the time column, the oldest first.
fn answer_chart(v: &Visit) -> Html {
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
            Some((f64::from(at), day_label(v, row)))
        })
        .collect();
    let chart = LineChart {
        id: "clinical-answer",
        label: v.tr(Msg::ClChartLabel),
        summary: v.tr(Msg::ClChartSummary),
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
            .map(|(index, row)| (f64::from(index), day_label(v, row)))
            .collect(),
        y_range: None,
    };
    line_chart(&chart, ChartWords::of(&v.locale))
}

/// One answer as a table, its switch in the URL, and as a chart, its switch
/// a form the console answers from its session, then an answer with no row,
/// and one riding the query's own form ([`riding`]).
fn results(v: &Visit, brand: &str) -> Html {
    let count = u64::try_from(ROWS.len()).unwrap_or(u64::MAX);
    let page = format!("/{brand}/clinical");
    let table = results_pane(
        &v.locale,
        ResultsPane {
            title: v.tr(Msg::ClResultsTitle),
            count,
            numeric: true,
            shown: Shown::Table,
            switch: Switch::Url {
                table: format!("{page}#results"),
                chart: format!("{page}#results-chart"),
            },
            csv: Export::Link(format!("{page}#results")),
            json: Export::Link(format!("{page}#results")),
            window: v.tr(Msg::ClResultsWindow),
            body: answer_table(v),
        },
    );
    let chart = results_pane(
        &v.locale,
        ResultsPane {
            title: v.tr(Msg::ClResultsTitle),
            count,
            numeric: true,
            shown: Shown::Chart,
            switch: Switch::Session {
                action: page.clone(),
                back: format!("{page}#results-chart"),
            },
            csv: Export::Link(format!("{page}#results-chart")),
            json: Export::Link(format!("{page}#results-chart")),
            window: v.tr(Msg::ClResultsWindow),
            body: answer_chart(v),
        },
    );
    let empty = results_pane(
        &v.locale,
        ResultsPane {
            title: v.tr(Msg::ClResultsTitle),
            count: 0,
            numeric: true,
            shown: Shown::Table,
            switch: Switch::Url {
                table: format!("{page}#results-empty"),
                chart: format!("{page}#results-empty"),
            },
            csv: Export::Link(format!("{page}#results-empty")),
            json: Export::Link(format!("{page}#results-empty")),
            window: v.tr(Msg::ClResultsEmptyWindow),
            body: Html::empty(),
        },
    );
    h(format!(
        "<div id=\"results\">{}</div><div id=\"results-chart\">{}</div><div id=\"results-empty\">{}</div>{}",
        panel_sections_card(v.tr(Msg::ClResultsTable), table),
        panel_sections_card(v.tr(Msg::ClResultsChart), chart),
        panel_sections_card(v.tr(Msg::ClResultsEmpty), empty),
        riding(v, &page, count)
    ))
}

/// An answer inside the query's own form, as a console whose run is a form
/// post draws it: the switch submits the form, and Export's items submit it
/// to [`crate::app::EXPORT_ROUTE`] by `formaction`, the query riding along.
fn riding(v: &Visit, page: &str, count: u64) -> Html {
    let chart = v.param(SHOWN) == Some(Shown::Chart.key());
    let (shown, body) = if chart {
        (Shown::Chart, answer_chart(v))
    } else {
        (Shown::Table, answer_table(v))
    };
    let answer = results_pane(
        &v.locale,
        ResultsPane {
            title: v.tr(Msg::ClResultsTitle),
            count,
            numeric: true,
            shown,
            switch: Switch::Form { form: None },
            csv: Export::Submit {
                action: format!("{}/csv", crate::app::EXPORT_ROUTE),
                form: None,
            },
            json: Export::Submit {
                action: format!("{}/json", crate::app::EXPORT_ROUTE),
                form: None,
            },
            window: v.tr(Msg::ClResultsWindow),
            body,
        },
    );
    let query = panel_section(
        &v.locale,
        None,
        field(
            "results-query-text",
            v.tr(Msg::ClResultsQuery),
            None,
            h(format!(
                "<textarea id=\"results-query-text\" name=\"query\" class=\"{TEXTAREA} fui-mono\" rows=\"2\" spellcheck=\"false\">{}</textarea>",
                t(QUERY)
            )),
        ),
    );
    h(format!(
        "<form id=\"results-form\" method=\"get\" action=\"{page}#results-form\">{}</form>",
        panel_sections_card(v.tr(Msg::ClResultsForm), Html::join([query, answer]))
    ))
}

/// The scopes an app holds: two identities and a launch context, three
/// narrow (one constrained), one broad, one malformed and one the grammar
/// takes and the CDR refuses.
fn scopes(v: &Visit) -> Html {
    let mut grants = SCOPES
        .iter()
        .map(|(raw, resource, broad)| {
            ScopeGrant::read(&v.locale, *raw, |_| ((*resource).to_owned(), *broad))
        })
        .collect::<Vec<_>>();
    grants.push(ScopeGrant {
        raw: String::from(FOREIGN),
        grant: Grant::Refused(v.tr(Msg::ClNotOpenehr).into()),
    });
    h(format!(
        "<div id=\"scopes\">{}</div>",
        titled_panel(
            v.tr(Msg::ClScopes),
            Html::empty(),
            scope_grants(&v.locale, &v.tr(Msg::ClScopesHeld), grants)
        )
    ))
}

/// The clinical records page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let brand = v.brand.key();
    let header = page_heading(
        &v.locale,
        Header {
            title: v.tr(Msg::ClinicalRecords).into(),
            subtitle: Some(v.tr(Msg::ClSubtitle).into()),
            subtitle_view: None,
            crumbs: vec![Crumb::new(v.tr(Msg::Gallery), format!("/{brand}"))],
            mono: false,
            badge: None,
        },
        ActionGroup::none(),
    );
    let body = stack(Html::join([
        subjects(v, brand),
        histories(v, brand),
        composition(v),
        builder(v, brand),
        results(v, brand),
        scopes(v),
    ]));
    Page::shell(v.tr(Msg::ClinicalRecords), Html::join([header, body]))
}
