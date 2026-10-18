// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page: the fields, the checkbox row, the highlighted editor with
//! diagnostics that link to its lines, and the filter over a list.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::form::{
    FORM, INPUT, MEDIUM, SELECT, SWITCH, TEXTAREA, check, check_field, chip_choices, columns,
    field, field_flow, help_id,
};
use ferrohealth_ui::view::layout::action_bar;
use ferrohealth_ui::view::notice::{Announce, notice};
use ferrohealth_ui::view::segmented::choice_reveal;
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{card, grid, grid_start, page_header, stack};
use ferrohealth_ui::widget::combobox::{ComboOption, Combobox, combobox};
use ferrohealth_ui::widget::editor::{Editor, LineMark, MarkKind, code_editor, line_id};
use ferrohealth_ui::widget::filter::{keeps, table_filter};
use ferrohealth_ui::widget::upload::{Upload, upload_field};
use leptos::prelude::*;

/// A query form: the nodes as checkbox chips, an option as a switch, and the
/// actions in the bar at the card's foot.
fn federated() -> AnyView {
    let nodes = ["node-a", "node-b", "node-c", "node-d"]
        .into_iter()
        .map(|node| (String::from(node), format!("synthetic-{node}")))
        .collect();
    let checked = [String::from("node-a"), String::from("node-c")];
    let partial = view! {
        <input
            id="query-partial"
            name="partial"
            type="checkbox"
            role="switch"
            class=SWITCH
            checked=true
            aria-describedby=help_id("query-partial")
        />
    };
    card(
        "A federated query",
        view! {
            <form method="post" action="#" class=FORM>
                {chip_choices("nodes", "Ask these nodes", nodes, &checked)}
                {check_field(
                    "query-partial",
                    "Accept a partial answer",
                    "When a node does not answer, the rows of the others still show.",
                    partial,
                )}
                {action_bar(view! {
                    <button type="button" class=Variant::Primary.class()>
                        {label(Some(Glyph::PLAY), "Run the query")}
                    </button>
                    <button type="button" class=Variant::Secondary.class()>
                        {label(Some(Glyph::X), "Clear")}
                    </button>
                })}
            </form>
        },
    )
    .into_any()
}

/// The id of the editor's text area.
pub const DRAFT: &str = "gallery-draft";

/// The fields, the select, the text area and the checkbox row.
fn fields() -> AnyView {
    let reason = view! {
        <input
            id="form-reason"
            name="reason"
            type="text"
            class=INPUT
            maxlength="200"
            aria-describedby=help_id("form-reason")
        />
    };
    let format = view! {
        <select id="form-format" name="format" class=SELECT>
            <option value="canonical">"canonical"</option>
            <option value="flat">"flat"</option>
        </select>
    };
    let bundle =
        view! { <textarea id="form-bundle" name="bundle" rows="4" class=TEXTAREA></textarea> };
    let until = view! { <input id="form-until" name="until" type="date" class=format!("{INPUT} {MEDIUM}") /> };
    let confirm = view! { <input id="form-confirm" name="confirmed" type="checkbox" /> };
    card(
        "Fields",
        view! {
            <form method="post" action="#" class=FORM>
                {field("form-reason", "Why you save it, one line", Some("Two hundred characters at most."), reason)}
                {field("form-format", "Composition format", None, format)}
                {field("form-bundle", "Bundle, JSON", None, bundle)}
                {field("form-until", "Keep the draft until", None, until)}
                {check(
                    "form-confirm",
                    "Save this draft and reload the set. The bridge refuses a draft that does not compile.",
                    confirm,
                )}
                <button type="button" class=Variant::Primary.class()>
                    {label(Some(Glyph::SAVE), "Save")}
                </button>
            </form>
        },
    )
    .into_any()
}

/// Fields in one row that share its width, a select and a date-time input
/// beside text inputs, and the button variants enabled and disabled.
fn inline() -> AnyView {
    let ehr = view! { <input id="row-ehr" name="ehr" type="text" class=INPUT placeholder="ehr_id (UUID), blank lets the server mint one" /> };
    let subject = view! { <input id="row-subject" name="subject" type="text" class=INPUT placeholder="external subject id" /> };
    let namespace = view! { <input id="row-namespace" name="namespace" type="text" class=INPUT placeholder="namespace" /> };
    let path = view! { <input id="row-path" name="path" type="text" class=INPUT placeholder="/context/start_time/value" /> };
    let version = view! {
        <select id="row-version" name="version" class=SELECT>
            <option value="latest">"Latest"</option>
            <option value="1">"Version 1"</option>
        </select>
    };
    let at = view! { <input id="row-at" name="at" type="datetime-local" class=INPUT /> };
    card(
        "Fields in a row",
        view! {
            <div class=FORM>
                {field_flow(view! {
                    {field("row-ehr", "EHR id (optional)", None, ehr)}
                    {field("row-subject", "Subject id (optional)", None, subject)}
                    {field("row-namespace", "Subject namespace (optional)", None, namespace)}
                    <button type="button" class=Variant::Primary.class()>
                        "Create"
                    </button>
                })}
                {field_flow(view! {
                    {field("row-version", "Version", None, version)}
                    {field("row-at", "Time travel (UTC)", None, at)}
                    {field("row-path", "Target path (optional)", None, path)}
                    <button type="button" class=Variant::Secondary.class()>
                        "At time"
                    </button>
                })}
                <div class="fui-cluster">
                    <button type="button" class=Variant::Primary.class()>"Primary"</button>
                    <button type="button" class=Variant::Secondary.class()>"Secondary"</button>
                    <button type="button" class=Variant::Danger.class()>"Danger"</button>
                    <button type="button" class=Variant::Ghost.class()>"Ghost"</button>
                    <button type="button" class=Variant::Secondary.small()>"Small"</button>
                </div>
                <div class="fui-cluster">
                    <button type="button" class=Variant::Primary.class() disabled=true>"Primary, disabled"</button>
                    <button type="button" class=Variant::Secondary.class() disabled=true>"Secondary, disabled"</button>
                    <button type="button" class=Variant::Danger.class() disabled=true>"Danger, disabled"</button>
                </div>
            </div>
        },
    )
    .into_any()
}

/// A long list to choose from, filtered as the reader types, beside a group
/// with its heading, a checkbox on the controls' line, and a disabled save
/// that names its reason.
fn choosing() -> AnyView {
    let systems = [
        ("loinc", "LOINC", "http://loinc.org"),
        ("snomed", "SNOMED CT", "http://snomed.info/sct"),
        ("ucum", "UCUM", "http://unitsofmeasure.org"),
        ("icd10", "ICD-10", "http://hl7.org/fhir/sid/icd-10"),
        ("atc", "ATC", "http://www.whocc.no/atc"),
        (
            "rxnorm",
            "RxNorm",
            "http://www.nlm.nih.gov/research/umls/rxnorm",
        ),
        ("iso3166", "ISO 3166 countries", "urn:iso:std:iso:3166"),
        ("bcp47", "BCP 47 languages", "urn:ietf:bcp:47"),
        ("mime", "Media types", "urn:ietf:bcp:13"),
        (
            "gender",
            "Administrative gender",
            "http://hl7.org/fhir/administrative-gender",
        ),
        (
            "status",
            "Observation status",
            "http://hl7.org/fhir/observation-status",
        ),
        (
            "synthetic",
            "Synthetic gallery codes",
            "http://example.org/fhir/CodeSystem/gallery-synthetic",
        ),
    ];
    let options = systems
        .iter()
        .map(|(value, label, url)| ComboOption::new(*value, *label).detail(*url))
        .collect();
    let chosen = RwSignal::new(Some(String::from("snomed")));
    let code = view! { <input id="choose-code" name="code" type="text" class=INPUT placeholder="22298006" /> };
    let active = view! { <input id="choose-active" name="active" type="checkbox" checked=true /> };
    card(
        "Combobox",
        view! {
            <div class=FORM>
                {combobox(Combobox {
                    id: "choose-system",
                    name: "system",
                    label: "Code system",
                    placeholder: "Choose a code system",
                    options,
                    selected: chosen,
                })}
                <fieldset class="fui-fieldset">
                    <legend>"The code"</legend>
                    {field_flow(view! {
                        {field("choose-code", "Code", None, code)}
                        {check("choose-active", "Active concepts only", active)}
                    })}
                </fieldset>
                <div class="fui-cluster">
                    <button type="button" class=Variant::Primary.class() disabled=true aria-describedby="choose-save-why">
                        {label(Some(Glyph::SAVE), "Save this value set")}
                    </button>
                    <button type="button" class=Variant::Danger.small()>
                        {label(Some(Glyph::TRASH), "Remove this clause")}
                    </button>
                </div>
                <p id="choose-save-why" class="fui-help">
                    "A canonical and a clause are what a save needs."
                </p>
            </div>
        },
    )
    .into_any()
}

/// Related fields under a legend, in pairs, and a choice drawn as
/// segments.
fn grouped() -> AnyView {
    let pair = columns(view! {
        {field("member-name", "Member", None, view! { <input id="member-name" type="text" class=INPUT value="synthetic-node-a" /> })}
        {field("member-base", "Base URL", None, view! { <input id="member-base" type="text" class=INPUT value="https://node-a.example.org/" /> })}
    });
    card(
        "Grouped fields",
        view! {
            <form method="post" action="#" class=FORM>
                <fieldset class="fui-fieldset fui-well">
                    <legend>"A federation member"</legend>
                    {pair}
                </fieldset>
                {choice_reveal(
                    "secret",
                    "Its client secret",
                    &[("keep", "Keep"), ("replace", "Replace"), ("remove", "Remove")],
                    "keep",
                    vec![
                        ().into_any(),
                        field(
                            "member-secret",
                            "The new secret",
                            None,
                            view! { <input id="member-secret" name="secret-value" type="password" class=INPUT autocomplete="new-password" /> },
                        )
                        .into_any(),
                        notice(
                            Tone::Warn,
                            Announce::Status,
                            None,
                            "The member signs in with no secret once this is saved.",
                        )
                        .into_any(),
                    ],
                )}
            </form>
        },
    )
    .into_any()
}

/// A grant whose chosen kind reveals its settings, one of which is a secret
/// revealed by a choice of its own: each reveal answers to its own choice.
fn nested_reveal() -> AnyView {
    let secret = choice_reveal(
        "grant-secret",
        "Its client secret",
        &[("keep", "Keep"), ("replace", "Replace")],
        "keep",
        vec![
            ().into_any(),
            field(
                "grant-secret-value",
                "The new secret",
                None,
                view! { <input id="grant-secret-value" type="password" class=INPUT autocomplete="new-password" /> },
            )
            .into_any(),
        ],
    );
    let client = view! {
        <div class="fui-stack">
            {field("grant-client-id", "Client id", None, view! { <input id="grant-client-id" type="text" class=INPUT value="synthetic-client" /> })}
            {secret}
        </div>
    }
    .into_any();
    let token = field(
        "grant-token-value",
        "Token",
        None,
        view! { <input id="grant-token-value" type="password" class=INPUT autocomplete="off" /> },
    )
    .into_any();
    card(
        "A reveal inside a reveal",
        view! {
            <form method="post" action="#" class=FORM>
                {choice_reveal(
                    "grant",
                    "The grant",
                    &[("client", "Client credentials"), ("token", "A token")],
                    "client",
                    vec![client, token],
                )}
            </form>
        },
    )
    .into_any()
}

/// A long form whose actions stay in the bar at the window's foot: a
/// control the keyboard reaches is scrolled clear of the bar.
fn long_form() -> AnyView {
    let fields = (1..=14_u32)
        .map(|index| {
            let id = format!("long-field-{index}");
            let input_id = id.clone();
            view! {
                <div class="fui-field">
                    <label for=id class="fui-label">{format!("Synthetic setting {index}")}</label>
                    <input id=input_id type="text" class=INPUT value=format!("value-{index}") />
                </div>
            }
        })
        .collect_view();
    card(
        "A long form",
        view! {
            <form method="post" action="#" class=FORM id="long-form">
                {fields}
                {action_bar(view! {
                    <button type="button" class=Variant::Primary.class()>
                        {label(Some(Glyph::CHECK), "Save the settings")}
                    </button>
                })}
            </form>
        },
    )
    .into_any()
}

/// A button whose words are longer than a phone's column, which break
/// between words rather than push the page sideways, beside a short one.
fn long_label() -> AnyView {
    card(
        "A long button label",
        view! {
            <div class="fui-cluster">
                <button type="button" id="long-label" class=Variant::Primary.class()>
                    {label(Some(Glyph::DOWNLOAD), "Download the whole configuration as TOML")}
                </button>
                <button type="button" class=Variant::Secondary.class()>
                    {label(Some(Glyph::X), "Clear")}
                </button>
            </div>
        },
    )
    .into_any()
}

/// The upload field: a file dropped or chosen, read into the text area.
fn upload() -> AnyView {
    let source = RwSignal::new(String::new());
    card(
        "Upload",
        upload_field(
            Upload {
                id: "gallery-upload",
                label: "Operational template",
                choose: "Choose OPT file",
                accept: &[".opt", ".xml"],
                max_bytes: 2 * 1024 * 1024,
                placeholder: "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
            },
            source,
            Signal::derive(|| false),
            Signal::derive(|| None),
        ),
    )
    .into_any()
}

/// The highlighted editor over the synthetic YAML, with two diagnostics
/// marking and linking to their lines, and the bytes it holds.
fn editor() -> AnyView {
    let text = RwSignal::new(String::from(crate::data::YAML));
    let ready = RwSignal::new(false);
    Effect::new(move |_| ready.set(true));
    let marks = Signal::derive(|| {
        vec![
            LineMark {
                line: 4,
                kind: MarkKind::Warning,
            },
            LineMark {
                line: 13,
                kind: MarkKind::Error,
            },
        ]
    });
    let field = code_editor(
        Editor {
            id: DRAFT,
            language: Language::Yaml,
            text: text.into(),
            marks,
            editable: ready.into(),
        },
        move |typed| text.set(typed),
    );
    let size = move || {
        text.with(|text| {
            format!(
                "{} bytes in {} lines.",
                text.len(),
                text.split('\n').count()
            )
        })
    };
    let diagnostics = [
        (
            4,
            Tone::Warn,
            "warning",
            "a comment after a value is kept but not read",
        ),
        (
            13,
            Tone::Bad,
            "error",
            "the template gallery.template.v1 is not loaded",
        ),
    ]
    .into_iter()
    .map(|(line, tone, word, text)| {
        view! {
            <li>
                {badge(tone, word)}
                <span class="fui-issue-head">
                    <a href=format!("#{}", line_id(DRAFT, line))>{format!("line {line}")}</a>
                    ":"
                </span>
                <span class="fui-issue-text">{text}</span>
            </li>
        }
    })
    .collect_view();
    card(
        "Highlighted editor",
        view! {
            <div class="fui-field">
                <label for=DRAFT class="fui-label">
                    "The file"
                </label>
                {field}
                <p class="fui-help" role="status">
                    {size}
                </p>
            </div>
            <ul class="fui-issues">{diagnostics}</ul>
        },
    )
    .into_any()
}

/// The filter over a list of synthetic file names.
fn filter() -> AnyView {
    let words = RwSignal::new(String::new());
    let names: Vec<String> = crate::data::rows()
        .into_iter()
        .take(12)
        .map(|row| row.name)
        .collect();
    let total = names.len();
    let names = StoredValue::new(names);
    let kept = Memo::new(move |_| {
        let words = words.get();
        names.with_value(|names| {
            names
                .iter()
                .filter(|name| keeps(&words, &[name.as_str()]))
                .cloned()
                .collect::<Vec<_>>()
        })
    });
    let count =
        Signal::derive(move || format!("{} of {total} files on this page.", kept.with(Vec::len)));
    card(
        "Filter",
        view! {
            {table_filter("gallery-filter", "Filter the files", words, count)}
            <ul class="fui-divided" aria-label="Files">
                <For each=move || kept.get() key=|name| name.clone() let:name>
                    <li>
                        <code>{name}</code>
                    </li>
                </For>
            </ul>
        },
    )
    .into_any()
}

/// The forms page.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn FormsPage() -> impl IntoView {
    let header = page_header(
        "Forms and editor",
        Some(String::from(
            "Fields, grouped fields, the upload field, the highlighted editor and the filter.",
        )),
        (),
    )
    .into_any();
    let pair = grid(view! {
        {fields()}
        {stack(view! {
            {grouped()}
            {federated()}
        })}
    })
    .into_any();
    let sections = (pair, upload(), editor(), filter(), inline(), choosing());
    // Cards of unlike length side by side, each at its own height.
    let unlike = grid_start(view! {
        {long_label()}
        {nested_reveal()}
    })
    .into_any();
    let long = long_form();
    view! {
        {super::title("Forms and editor")}
        {header}
        {stack(view! {
            {sections.0}
            {sections.4}
            {sections.5}
            {unlike}
            {sections.1}
            {sections.2}
            {sections.3}
            {long}
        })}
    }
}
