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
    field, field_flow, fieldset, help_id,
};
use ferrohealth_ui::view::layout::action_bar;
use ferrohealth_ui::view::notice::{Announce, notice};
use ferrohealth_ui::view::segmented::choice_reveal;
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{card, grid, page_header, stack};
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
                {fieldset("A federation member", pair)}
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
    let sections = (pair, upload(), editor(), filter(), inline());
    view! {
        {super::title("Forms and editor")}
        {header}
        {stack(view! {
            {sections.0}
            {sections.4}
            {sections.1}
            {sections.2}
            {sections.3}
        })}
    }
}
