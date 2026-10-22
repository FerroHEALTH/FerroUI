// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The operations page.
//!
//! It holds the load report in each outcome with and without its action,
//! the transform pair before the first run, running, answered and refused,
//! a dense options row with its verdict and request, the OMOP row preview
//! with an unresolved concept, and an HL7 v2 message raw and parsed.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::island::CopyButton;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::code::{Code, code_viewer};
use ferrohealth_ui::view::form::{Field, INPUT, SELECT, field};
use ferrohealth_ui::view::hl7v2::{Hl7Field, Hl7Segment, hl7v2_raw, hl7v2_segments};
use ferrohealth_ui::view::load_report::{Arrival, Count, LoadReport, Outcome, load_report};
use ferrohealth_ui::view::refusal::{Issue, Refusal, Severity, refusal};
use ferrohealth_ui::view::row_preview::{RowField, RowTable, RowValue, row_preview};
use ferrohealth_ui::view::surface::{
    Crumb, Header, SectionHead, page_heading, panel_body, panel_section, stack, titled_panel,
};
use ferrohealth_ui::view::transform::{
    Answer, AnswerState, Density, Request, Run, RunState, TransformPane, field_checks, request,
    run_button, transform_options, transform_pair,
};
use ferrohealth_ui::widget::copy::{CopySource, CopyWords};
use ferrohealth_ui::widget::editor::{Editor, code_editor};
use leptos::prelude::*;

use crate::data::{
    EHR_ID, FLAT, HL7_MESSAGE, HL7_SEGMENTS, HL7_VERSION, JSON, OMOP_TABLES, OMOP_VERSION,
    PAIR_REQUEST, PAIR_TAGS, RUN_KEYS, RUN_OPERATION, TEMPLATES,
};
use crate::locale::{Msg, tr, tr_with};

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
    /// The key the island of its input takes.
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

/// The input editor of the pair `key`, hydrated as the island
/// [`crate::islands::PairInput`], its text area named by `label`.
pub(crate) fn pair_input(key: &str, label: String) -> AnyView {
    let (id, text) = match key {
        "running" => ("pair-running-input", String::from(FLAT)),
        "answered" => ("pair-answered-input", String::from(FLAT)),
        "refused" => ("pair-refused-input", FLAT.replace("\"kg\"", "\"lb\"")),
        "rows" => ("pair-rows-input", String::from(FLAT)),
        _ => ("pair-waiting-input", String::from(FLAT)),
    };
    let text = RwSignal::new(text);
    let ready = RwSignal::new(false);
    Effect::new(move |_| ready.set(true));
    let editor = code_editor(
        Editor {
            id,
            language: Language::Json,
            text: text.into(),
            marks: Signal::stored(Vec::new()),
            editable: ready.into(),
        },
        move |typed| text.set(typed),
    );
    view! {
        <label for=id class="fui-sr-only">
            {label}
        </label>
        {editor}
    }
    .into_any()
}

/// The input pane holding the editor of the pair `key`.
fn input_pane(key: &'static str) -> TransformPane {
    let [flat, _] = PAIR_TAGS;
    TransformPane {
        title: tr(Msg::OpComposition),
        tags: vec![String::from(flat)],
        state: None,
        actions: Some(
            view! {
                <button type="reset" class=Variant::Ghost.small()>
                    {label(Some(Glyph::X), tr(Msg::OpClear))}
                </button>
            }
            .into_any(),
        ),
        body: view! {
            <crate::islands::PairInput
                pair=String::from(key)
                label=tr(Msg::OpCompositionInput)
            />
        }
        .into_any(),
    }
}

/// The template select with the id `id`.
fn template_field(id: &'static str) -> AnyView {
    let options = TEMPLATES
        .iter()
        .map(|template| view! { <option value=*template>{*template}</option> })
        .collect_view();
    field(
        id,
        tr(Msg::OpTemplate),
        None,
        view! {
            <select id=id name="template" class=SELECT>
                {options}
            </select>
        },
    )
    .into_any()
}

/// The format select with the id `id`.
fn format_field(id: &'static str) -> AnyView {
    let [flat, _] = PAIR_TAGS;
    field(
        id,
        tr(Msg::OpFormat),
        None,
        view! {
            <select id=id name="format" class=SELECT>
                <option value="canonical">{tr(Msg::OpCanonical)}</option>
                <option value="flat" selected>
                    {flat}
                </option>
            </select>
        },
    )
    .into_any()
}

/// Run with the id `id`, saying `run` and `busy`, in `state`.
fn run(id: &str, run: Msg, busy: Msg, state: RunState) -> AnyView {
    run_button(Run {
        id: id.to_owned(),
        label: tr(run),
        busy: tr(busy),
        glyph: Glyph::PLAY,
        shortcut: Some(String::from(RUN_KEYS)),
        operation: Some(tr_with(
            Msg::OpRuns,
            &[("name", String::from(RUN_OPERATION).into())],
        )),
        state,
    })
}

/// The answer pane of the pair `pair`.
fn answer_pane(pair: Pair) -> TransformPane {
    let [_, fhir] = PAIR_TAGS;
    let (state, actions, answer) = match pair {
        Pair::Waiting => (None, None, Answer::Waiting(tr(Msg::OpConvertIdle))),
        Pair::Running => (None, None, Answer::Running(tr(Msg::OpConvertingFhir))),
        Pair::Answered => (
            Some(AnswerState {
                outcome: Outcome::Ok,
                word: tr(Msg::OpConverted),
                detail: Some(tr(Msg::OpAnsweredDetail)),
            }),
            Some(
                view! {
                    <CopyButton
                        id=String::from("pair-answer-copy")
                        label=tr(Msg::OpCopy)
                        source=CopySource::Text(String::from(JSON))
                        words=CopyWords::of_page()
                    />
                }
                .into_any(),
            ),
            Answer::Answered(code_viewer(
                None,
                &Code {
                    label: tr(Msg::OpBundleJson),
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
                word: tr(Msg::OpRefused),
                detail: Some(tr(Msg::OpRefusedDetail)),
            }),
            None,
            Answer::Refused(
                refusal(&Refusal {
                    status: Some(422),
                    summary: tr(Msg::OpRefusedSummary),
                    issues: vec![
                        Issue::new(Severity::Error, "processing").text(tr(Msg::OpRefusedUnit)),
                    ],
                    request: Some(String::from(PAIR_REQUEST)),
                    evidence: None,
                })
                .into_any(),
            ),
        ),
    };
    TransformPane {
        title: tr(Msg::OpBundle),
        tags: vec![String::from(fhir)],
        state,
        actions,
        body: answer.view(),
    }
}

/// The pair `pair` in a panel of its own: the options and the two panes
/// in one form.
fn pair_panel(pair: Pair) -> AnyView {
    let state = if pair == Pair::Running {
        RunState::Running
    } else {
        RunState::Ready
    };
    let id = format!("pair-{}-run", pair.key());
    let options = transform_options(
        Density::Plain,
        view! { {format_field_for(pair)} {template_for(pair)} },
        run(&id, Msg::OpConvert, Msg::OpConverting, state),
    );
    let pair_view = transform_pair(input_pane(pair.key()), answer_pane(pair));
    titled_panel(
        tr(pair.title()),
        (),
        view! {
            <form method="post" action="#">
                {options}
                {pair_view}
            </form>
        },
    )
    .into_any()
}

/// The format field of the options row of `pair`.
fn format_field_for(pair: Pair) -> AnyView {
    format_field(match pair {
        Pair::Waiting => "pair-waiting-format",
        Pair::Running => "pair-running-format",
        Pair::Answered => "pair-answered-format",
        Pair::Refused => "pair-refused-format",
    })
}

/// The template field of the options row of `pair`.
fn template_for(pair: Pair) -> AnyView {
    template_field(match pair {
        Pair::Waiting => "pair-waiting-template",
        Pair::Running => "pair-running-template",
        Pair::Answered => "pair-answered-template",
        Pair::Refused => "pair-refused-template",
    })
}

/// A count of a load report.
fn count(value: &str, noun: Msg) -> Count {
    Count {
        value: value.to_owned(),
        noun: tr(noun),
    }
}

/// The load report in each outcome, with its action and then without.
fn load_reports(root: &str) -> AnyView {
    let mappings = format!("{root}/tables");
    let reports = [true, false]
        .into_iter()
        .flat_map(|with_action| {
            let mappings = mappings.clone();
            Outcome::ALL.into_iter().map(move |outcome| {
                let (title, counts, action) = match outcome {
                    Outcome::Ok => (
                        Msg::OpLoaded,
                        vec![count("12", Msg::OpMappings), count("0", Msg::OpErrors)],
                        Msg::OpOpenMappings,
                    ),
                    Outcome::Warn => (
                        Msg::OpLoadedWarnings,
                        vec![count("0", Msg::OpErrors), count("3", Msg::OpWarnings)],
                        Msg::OpShowWarnings,
                    ),
                    Outcome::Bad => (
                        Msg::OpReloadRefused,
                        vec![count("2", Msg::OpErrors), count("1", Msg::OpWarning)],
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
                    title: tr(title).into_any(),
                    counts,
                    action: with_action.then(|| {
                        view! {
                            <a href=mappings.clone() class=Variant::Secondary.small()>
                                {tr(action)}
                            </a>
                        }
                        .into_any()
                    }),
                })
            })
        })
        .collect_view();
    titled_panel(tr(Msg::OpLoadReport), (), panel_body(stack(reports))).into_any()
}

/// The dense options row: three fields and a group of check boxes, with
/// the verdict and the request under it.
fn dense() -> AnyView {
    let patient = field(
        "dense-patient",
        tr(Msg::OpPatient),
        None,
        view! { <input id="dense-patient" name="patient" class=INPUT value="SYN-0042" /> },
    );
    let checks = field_checks(view! {
        <div class="fui-check">
            <input id="dense-active" name="active" type="checkbox" checked />
            <label for="dense-active">{tr(Msg::OpActiveOnly)}</label>
        </div>
        <div class="fui-check">
            <input id="dense-designations" name="designations" type="checkbox" />
            <label for="dense-designations">{tr(Msg::OpIncludeDesignations)}</label>
        </div>
    });
    let options = transform_options(
        Density::Dense,
        view! {
            {template_field("dense-template")}
            {format_field("dense-format")}
            {patient}
            {checks}
        },
        run(
            "dense-run",
            Msg::OpConvert,
            Msg::OpConverting,
            RunState::Ready,
        ),
    );
    let verdict = panel_section(
        None,
        load_report(LoadReport {
            outcome: Outcome::Ok,
            arrival: Arrival::WithPage,
            title: tr(Msg::OpConvertedOne).into_any(),
            counts: Vec::new(),
            action: None,
        }),
    );
    let raw = code_viewer(
        None,
        &Code {
            label: tr(Msg::OpBundleSent),
            language: Language::Json,
            text: String::from(JSON),
            ids: None,
            current: None,
            diagnostics: Vec::new(),
        },
    );
    let asked = panel_section(
        Some(SectionHead {
            title: tr(Msg::OpRequest),
            count: None,
            actions: None,
        }),
        request(Request {
            text: String::from(PAIR_REQUEST),
            copy: Some(String::from("dense-request-copy")),
            raw: Some((tr(Msg::OpBundleSent), raw)),
        }),
    );
    titled_panel(
        tr(Msg::OpDense),
        (),
        view! {
            <form method="post" action="#">
                {options}
                {verdict}
                {asked}
            </form>
        },
    )
    .into_any()
}

/// The OMOP row preview of the synthetic composition, answered with three
/// unresolved concepts.
fn rows() -> AnyView {
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
        title: tr(Msg::OpRows),
        tags: vec![String::from(OMOP_VERSION)],
        state: Some(AnswerState {
            outcome: Outcome::Warn,
            word: tr(Msg::OpRowsZero),
            detail: Some(tr(Msg::OpRowsDetail)),
        }),
        actions: None,
        body: row_preview(tables),
    };
    let ehr = Field::new("rows-ehr", tr(Msg::OpEhrId))
        .holds_id()
        .view(view! {
            <input
                id="rows-ehr"
                name="ehr"
                class=format!("{INPUT} fui-mono")
                value=EHR_ID
                spellcheck="false"
            />
        });
    let options = transform_options(
        Density::Plain,
        view! {
            {template_field("rows-template")}
            {ehr}
        },
        run(
            "rows-run",
            Msg::OpPreview,
            Msg::OpPreviewing,
            RunState::Ready,
        ),
    );
    let pair = transform_pair(input_pane("rows"), answer);
    titled_panel(
        tr(Msg::OpOmopRows),
        (),
        view! {
            <form method="post" action="#">
                {options}
                {pair}
            </form>
        },
    )
    .into_any()
}

/// The synthetic HL7 v2 message raw beside the Bundle it maps to, and as
/// read, segment by segment.
fn message() -> AnyView {
    let [_, fhir] = PAIR_TAGS;
    let input = TransformPane {
        title: tr(Msg::OpMessage),
        tags: vec![String::from(HL7_VERSION)],
        state: None,
        actions: None,
        body: hl7v2_raw(tr(Msg::OpMessageLines), HL7_MESSAGE),
    };
    let answer = TransformPane {
        title: tr(Msg::OpBundle),
        tags: vec![String::from(fhir)],
        state: Some(AnswerState {
            outcome: Outcome::Ok,
            word: tr(Msg::OpWouldAnswer),
            detail: Some(tr(Msg::OpWouldAnswerDetail)),
        }),
        actions: None,
        body: code_viewer(
            None,
            &Code {
                label: tr(Msg::OpBundleJson),
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
        Some(SectionHead {
            title: tr(Msg::OpMessageRead),
            count: Some(count),
            actions: None,
        }),
        hl7v2_segments(tr(Msg::OpMessageReadList), segments),
    );
    let charset = field(
        "hl7-charset",
        tr(Msg::OpCharacterSet),
        None,
        view! {
            <select id="hl7-charset" name="charset" class=SELECT>
                <option value="ascii">{tr(Msg::OpCharacterSetAscii)}</option>
            </select>
        },
    );
    let options = transform_options(
        Density::Plain,
        charset,
        run("hl7-run", Msg::OpMap, Msg::OpMapping, RunState::Ready),
    );
    let pair = transform_pair(input, answer);
    titled_panel(
        tr(Msg::OpHl7Message),
        (),
        view! {
            <form method="post" action="#">
                {options}
                {pair}
                {read}
            </form>
        },
    )
    .into_any()
}

/// The operations page.
#[component]
pub fn OperationsPage() -> impl IntoView {
    let root =
        crate::app::route_brand().map_or_else(String::new, |brand| format!("/{}", brand.key()));
    let header = page_heading(
        Header {
            title: tr(Msg::Operations),
            subtitle: Some(tr(Msg::OpSubtitle)),
            crumbs: vec![Crumb::new(tr(Msg::Gallery), root.clone())],
            mono: false,
            badge: None,
        },
        ActionGroup::none(),
    )
    .into_any();
    let reports = load_reports(&root);
    let pairs = [Pair::Waiting, Pair::Running, Pair::Answered, Pair::Refused]
        .into_iter()
        .map(pair_panel)
        .collect_view()
        .into_any();
    let sections = (reports, pairs, dense(), rows(), message());
    view! {
        {super::title(tr(Msg::Operations))}
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
