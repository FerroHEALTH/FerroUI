// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The operations page.
//!
//! It holds the load report in each outcome with and without its action,
//! the transform pair before the first run, running, answered and refused,
//! a dense options row with its verdict and request, the OMOP row preview
//! with an unresolved concept, and an HL7 v2 message raw and parsed.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::code::{Code, code_viewer};
use ferrohealth_ui::view::copy::copy_button;
use ferrohealth_ui::view::editor::{Editor, code_editor};
use ferrohealth_ui::view::form::{Field, INPUT, SELECT, field};
use ferrohealth_ui::view::hl7v2::{Hl7Field, Hl7Segment, hl7v2_raw, hl7v2_segments};
use ferrohealth_ui::view::load_report::{Arrival, Count, LoadReport, Outcome, load_report};
use ferrohealth_ui::view::refusal::{Issue, Refusal, Severity, refusal};
use ferrohealth_ui::view::row_preview::{RowField, RowTable, RowValue, row_preview};
use ferrohealth_ui::view::surface::{
    Crumb, Header, SectionHead, page_heading, panel_body, panel_section, stack, titled_panel,
};
use ferrohealth_ui::view::template::list::HeadingCount;
use ferrohealth_ui::view::transform::{
    Answer, AnswerState, Density, Request, Run, RunState, TransformPane, field_checks, request,
    run_button, transform_options, transform_pair,
};

use crate::app::Page;
use crate::data::{
    EHR_ID, FLAT, HL7_MESSAGE, HL7_SEGMENTS, HL7_VERSION, JSON, OMOP_TABLES, OMOP_VERSION,
    PAIR_REQUEST, PAIR_TAGS, RUN_KEYS, RUN_OPERATION, TEMPLATES,
};
use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

/// The state of a transform pair the page shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pair {
    /// Before the first run.
    Waiting,
    /// A run in progress.
    Running,
    /// The answer.
    Answered,
    /// The refusal.
    Refused,
}

impl Pair {
    /// The key its input's ids are made from.
    const fn key(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Running => "running",
            Self::Answered => "answered",
            Self::Refused => "refused",
        }
    }

    /// The panel's title.
    const fn title(self) -> Msg {
        match self {
            Self::Waiting => Msg::OpBeforeFirstRun,
            Self::Running => Msg::OpRunning,
            Self::Answered => Msg::OpAnswered,
            Self::Refused => Msg::OpRefused,
        }
    }
}

/// The input editor of the pair `key`, its text area named by `label`.
fn pair_input(key: &str, label: &str) -> Html {
    let (id, text) = match key {
        "running" => ("pair-running-input", String::from(FLAT)),
        "answered" => ("pair-answered-input", String::from(FLAT)),
        "refused" => ("pair-refused-input", FLAT.replace("\"kg\"", "\"lb\"")),
        "rows" => ("pair-rows-input", String::from(FLAT)),
        _ => ("pair-waiting-input", String::from(FLAT)),
    };
    let editor = code_editor(&Editor {
        id: id.into(),
        language: Language::Json,
        text,
        marks: Vec::new(),
        editable: true,
    });
    h(format!(
        "<label for=\"{id}\" class=\"fui-sr-only\">{}</label>{editor}",
        t(label)
    ))
}

/// The input pane holding the editor of the pair `key`.
fn input_pane(v: &Visit, key: &'static str) -> TransformPane {
    let [flat, _] = PAIR_TAGS;
    TransformPane {
        title: v.tr(Msg::OpComposition),
        tags: vec![String::from(flat)],
        state: None,
        actions: Some(h(format!(
            "<button type=\"reset\" class=\"{}\">{}</button>",
            Variant::Ghost.small(),
            label(Some(Glyph::X), v.tr(Msg::OpClear))
        ))),
        body: pair_input(key, &v.tr(Msg::OpCompositionInput)),
    }
}

/// A template whose id is wider than a phone, offered by the dense row.
const LONG_TEMPLATE: &str = "synthetic.laboratory_test_result_with_a_reference_range_panel.v1";

/// The template select with the id `id`, offering `more` after the
/// gallery's templates.
fn template_field(v: &Visit, id: &'static str, more: &[&str]) -> Html {
    let options: Html = TEMPLATES
        .iter()
        .chain(more)
        .map(|template| h(format!("<option value=\"{template}\">{template}</option>")))
        .collect();
    field(
        id,
        v.tr(Msg::OpTemplate),
        None,
        h(format!(
            "<select id=\"{id}\" name=\"template\" class=\"{SELECT}\">{options}</select>"
        )),
    )
}

/// The format select with the id `id`.
fn format_field(v: &Visit, id: &'static str) -> Html {
    let [flat, _] = PAIR_TAGS;
    field(
        id,
        v.tr(Msg::OpFormat),
        None,
        h(format!(
            "<select id=\"{id}\" name=\"format\" class=\"{SELECT}\"><option value=\"canonical\">{}</option><option value=\"flat\" selected>{flat}</option></select>",
            t(v.tr(Msg::OpCanonical))
        )),
    )
}

/// Run with the id `id`, saying `run` and `busy`, in `state`.
fn run(v: &Visit, id: &str, run: Msg, busy: Msg, state: RunState) -> Html {
    run_button(Run {
        id: id.to_owned(),
        label: v.tr(run),
        busy: v.tr(busy),
        glyph: Glyph::PLAY,
        shortcut: Some(String::from(RUN_KEYS)),
        operation: Some(v.tr_with(Msg::OpRuns, &[("name", String::from(RUN_OPERATION).into())])),
        state,
    })
}

/// The answer pane of the pair `pair`.
fn answer_pane(v: &Visit, pair: Pair) -> TransformPane {
    let [_, fhir] = PAIR_TAGS;
    let (state, actions, answer) = match pair {
        Pair::Waiting => (None, None, Answer::Waiting(v.tr(Msg::OpConvertIdle))),
        Pair::Running => (None, None, Answer::Running(v.tr(Msg::OpConvertingFhir))),
        Pair::Answered => (
            Some(AnswerState {
                outcome: Outcome::Ok,
                word: v.tr(Msg::OpConverted),
                detail: Some(v.tr(Msg::OpAnsweredDetail)),
            }),
            Some(copy_button(
                &v.locale,
                "pair-answer-copy",
                v.tr(Msg::OpCopy),
                JSON,
            )),
            Answer::Answered(code_viewer(
                &v.locale,
                None,
                &Code {
                    label: v.tr(Msg::OpBundleJson),
                    language: Language::Json,
                    text: String::from(JSON),
                    ids: None,
                    current: None,
                    diagnostics: Vec::new(),
                },
            )),
        ),
        Pair::Refused => (
            Some(AnswerState {
                outcome: Outcome::Bad,
                word: v.tr(Msg::OpRefused),
                detail: Some(v.tr(Msg::OpRefusedDetail)),
            }),
            None,
            Answer::Refused(refusal(
                &v.locale,
                &Refusal {
                    status: Some(422),
                    summary: v.tr(Msg::OpRefusedSummary),
                    issues: vec![
                        Issue::new(Severity::Error, "processing").text(v.tr(Msg::OpRefusedUnit)),
                    ],
                    request: Some(String::from(PAIR_REQUEST)),
                    evidence: None,
                },
            )),
        ),
    };
    TransformPane {
        title: v.tr(Msg::OpBundle),
        tags: vec![String::from(fhir)],
        state,
        actions,
        body: answer.view(),
    }
}

/// The pair `pair` in a panel of its own: the options and the two panes
/// in one form.
fn pair_panel(v: &Visit, pair: Pair) -> Html {
    let state = if pair == Pair::Running {
        RunState::Running
    } else {
        RunState::Ready
    };
    let id = format!("pair-{}-run", pair.key());
    let options = transform_options(
        Density::Plain,
        Html::join([format_field_for(v, pair), template_for(v, pair)]),
        run(v, &id, Msg::OpConvert, Msg::OpConverting, state),
    );
    let pair_view = transform_pair(input_pane(v, pair.key()), answer_pane(v, pair));
    titled_panel(
        v.tr(pair.title()),
        Html::empty(),
        h(format!(
            "<form method=\"post\" action=\"#\">{options}{pair_view}</form>"
        )),
    )
}

/// The format field of the options row of `pair`.
fn format_field_for(v: &Visit, pair: Pair) -> Html {
    format_field(
        v,
        match pair {
            Pair::Waiting => "pair-waiting-format",
            Pair::Running => "pair-running-format",
            Pair::Answered => "pair-answered-format",
            Pair::Refused => "pair-refused-format",
        },
    )
}

/// The template field of the options row of `pair`.
fn template_for(v: &Visit, pair: Pair) -> Html {
    template_field(
        v,
        match pair {
            Pair::Waiting => "pair-waiting-template",
            Pair::Running => "pair-running-template",
            Pair::Answered => "pair-answered-template",
            Pair::Refused => "pair-refused-template",
        },
        &[],
    )
}

/// A count of a load report.
fn count(v: &Visit, value: &str, noun: Msg) -> Count {
    Count {
        value: value.to_owned(),
        noun: v.tr(noun),
    }
}

/// The load report in each outcome, with its action and then without.
fn load_reports(v: &Visit, root: &str) -> Html {
    let mappings = format!("{root}/tables");
    let reports = [true, false]
        .into_iter()
        .flat_map(|with_action| {
            let mappings = mappings.clone();
            Outcome::ALL.into_iter().map(move |outcome| {
                let (title, counts, action) = match outcome {
                    Outcome::Ok => (
                        Msg::OpLoaded,
                        vec![
                            count(v, "12", Msg::OpMappings),
                            count(v, "0", Msg::OpErrors),
                        ],
                        Msg::OpOpenMappings,
                    ),
                    Outcome::Warn => (
                        Msg::OpLoadedWarnings,
                        vec![count(v, "0", Msg::OpErrors), count(v, "3", Msg::OpWarnings)],
                        Msg::OpShowWarnings,
                    ),
                    Outcome::Bad => (
                        Msg::OpReloadRefused,
                        vec![count(v, "2", Msg::OpErrors), count(v, "1", Msg::OpWarning)],
                        Msg::OpOpenDiagnostics,
                    ),
                };
                load_report(LoadReport {
                    outcome,
                    arrival: if with_action {
                        Arrival::WithPage
                    } else {
                        Arrival::AfterAction
                    },
                    title: t(v.tr(title)),
                    counts,
                    action: with_action.then(|| {
                        h(format!(
                            "<a href=\"{}\" class=\"{}\">{}</a>",
                            t(&mappings),
                            Variant::Secondary.small(),
                            t(v.tr(action))
                        ))
                    }),
                })
            })
        })
        .collect();
    titled_panel(
        v.tr(Msg::OpLoadReport),
        Html::empty(),
        panel_body(stack(reports)),
    )
}

/// The dense options row: three fields and a group of check boxes, with
/// the verdict and the request under it.
fn dense(v: &Visit) -> Html {
    let patient = field(
        "dense-patient",
        v.tr(Msg::OpPatient),
        None,
        h(format!(
            "<input id=\"dense-patient\" name=\"patient\" class=\"{INPUT}\" value=\"SYN-0042\">"
        )),
    );
    let checks = field_checks(h(format!(
        "<div class=\"fui-check\"><input id=\"dense-active\" name=\"active\" type=\"checkbox\" checked><label for=\"dense-active\">{}</label></div><div class=\"fui-check\"><input id=\"dense-designations\" name=\"designations\" type=\"checkbox\"><label for=\"dense-designations\">{}</label></div>",
        t(v.tr(Msg::OpActiveOnly)),
        t(v.tr(Msg::OpIncludeDesignations))
    )));
    let options = transform_options(
        Density::Dense,
        Html::join([
            template_field(v, "dense-template", &[LONG_TEMPLATE]),
            format_field(v, "dense-format"),
            patient,
            checks,
        ]),
        run(
            v,
            "dense-run",
            Msg::OpConvert,
            Msg::OpConverting,
            RunState::Ready,
        ),
    );
    let verdict = panel_section(
        &v.locale,
        None,
        load_report(LoadReport {
            outcome: Outcome::Ok,
            arrival: Arrival::WithPage,
            title: t(v.tr(Msg::OpConvertedOne)),
            counts: Vec::new(),
            action: None,
        }),
    );
    let raw = code_viewer(
        &v.locale,
        None,
        &Code {
            label: v.tr(Msg::OpBundleSent),
            language: Language::Json,
            text: String::from(JSON),
            ids: None,
            current: None,
            diagnostics: Vec::new(),
        },
    );
    let asked = panel_section(
        &v.locale,
        Some(SectionHead {
            title: v.tr(Msg::OpRequest),
            count: None,
            actions: None,
        }),
        request(
            &v.locale,
            Request {
                text: String::from(PAIR_REQUEST),
                copy: Some(String::from("dense-request-copy")),
                raw: Some((v.tr(Msg::OpBundleSent), raw)),
            },
        ),
    );
    titled_panel(
        v.tr(Msg::OpDense),
        Html::empty(),
        h(format!(
            "<form method=\"post\" action=\"#\">{options}{verdict}{asked}</form>"
        )),
    )
}

/// The OMOP row preview of the synthetic composition, answered with three
/// unresolved concepts.
fn rows(v: &Visit) -> Html {
    let tables = OMOP_TABLES
        .iter()
        .map(|(name, mapping, rows)| RowTable {
            name: String::from(*name),
            detail: Some(String::from(*mapping)),
            rows: rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|(column, value)| RowField {
                            column: String::from(*column),
                            value: value.map_or(RowValue::ConceptZero, |value| {
                                RowValue::Value(String::from(value))
                            }),
                        })
                        .collect()
                })
                .collect(),
        })
        .collect();
    let answer = TransformPane {
        title: v.tr(Msg::OpRows),
        tags: vec![String::from(OMOP_VERSION)],
        state: Some(AnswerState {
            outcome: Outcome::Warn,
            word: v.tr(Msg::OpRowsZero),
            detail: Some(v.tr(Msg::OpRowsDetail)),
        }),
        actions: None,
        body: row_preview(&v.locale, tables),
    };
    let ehr = Field::new("rows-ehr", v.tr(Msg::OpEhrId))
        .holds_id()
        .view(h(format!(
            "<input id=\"rows-ehr\" name=\"ehr\" class=\"{INPUT} fui-mono\" value=\"{EHR_ID}\" spellcheck=\"false\">"
        )));
    let options = transform_options(
        Density::Plain,
        Html::join([template_field(v, "rows-template", &[]), ehr]),
        run(
            v,
            "rows-run",
            Msg::OpPreview,
            Msg::OpPreviewing,
            RunState::Ready,
        ),
    );
    let pair = transform_pair(input_pane(v, "rows"), answer);
    titled_panel(
        v.tr(Msg::OpOmopRows),
        Html::empty(),
        h(format!(
            "<form method=\"post\" action=\"#\">{options}{pair}</form>"
        )),
    )
}

/// The synthetic HL7 v2 message raw beside the Bundle it maps to, and as
/// read, segment by segment.
fn message(v: &Visit) -> Html {
    let [_, fhir] = PAIR_TAGS;
    let input = TransformPane {
        title: v.tr(Msg::OpMessage),
        tags: vec![String::from(HL7_VERSION)],
        state: None,
        actions: None,
        body: hl7v2_raw(v.tr(Msg::OpMessageLines), HL7_MESSAGE),
    };
    let answer = TransformPane {
        title: v.tr(Msg::OpBundle),
        tags: vec![String::from(fhir)],
        state: Some(AnswerState {
            outcome: Outcome::Ok,
            word: v.tr(Msg::OpWouldAnswer),
            detail: Some(v.tr(Msg::OpWouldAnswerDetail)),
        }),
        actions: None,
        body: code_viewer(
            &v.locale,
            None,
            &Code {
                label: v.tr(Msg::OpBundleJson),
                language: Language::Json,
                text: String::from(JSON),
                ids: None,
                current: None,
                diagnostics: Vec::new(),
            },
        ),
    };
    let segments = HL7_SEGMENTS
        .iter()
        .map(|(id, name, fields)| Hl7Segment {
            id: String::from(*id),
            name: String::from(*name),
            fields: fields
                .iter()
                .map(|(position, name, value)| Hl7Field {
                    position: String::from(*position),
                    name: String::from(*name),
                    value: String::from(*value),
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    let count = u64::try_from(segments.len()).unwrap_or(u64::MAX);
    let read = panel_section(
        &v.locale,
        Some(SectionHead {
            title: v.tr(Msg::OpMessageRead),
            count: Some(HeadingCount::All(count)),
            actions: None,
        }),
        hl7v2_segments(v.tr(Msg::OpMessageReadList), segments),
    );
    let charset = field(
        "hl7-charset",
        v.tr(Msg::OpCharacterSet),
        None,
        h(format!(
            "<select id=\"hl7-charset\" name=\"charset\" class=\"{SELECT}\"><option value=\"ascii\">{}</option></select>",
            t(v.tr(Msg::OpCharacterSetAscii))
        )),
    );
    let options = transform_options(
        Density::Plain,
        charset,
        run(v, "hl7-run", Msg::OpMap, Msg::OpMapping, RunState::Ready),
    );
    let pair = transform_pair(input, answer);
    titled_panel(
        v.tr(Msg::OpHl7Message),
        Html::empty(),
        h(format!(
            "<form method=\"post\" action=\"#\">{options}{pair}{read}</form>"
        )),
    )
}

/// The operations page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let root = format!("/{}", v.brand.key());
    let header = page_heading(
        &v.locale,
        Header {
            title: v.tr(Msg::Operations).into(),
            subtitle: Some(v.tr(Msg::OpSubtitle).into()),
            subtitle_view: None,
            crumbs: vec![Crumb::new(v.tr(Msg::Gallery), root.clone())],
            mono: false,
            badge: None,
        },
        ActionGroup::none(),
    );
    let pairs: Html = [Pair::Waiting, Pair::Running, Pair::Answered, Pair::Refused]
        .into_iter()
        .map(|pair| pair_panel(v, pair))
        .collect();
    let body = stack(Html::join([
        load_reports(v, &root),
        pairs,
        dense(v),
        rows(v),
        message(v),
    ]));
    Page::shell(v.tr(Msg::Operations), Html::join([header, body]))
}
