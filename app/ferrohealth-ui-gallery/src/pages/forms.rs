// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page: the fields, the checkbox row, the highlighted editor with
//! diagnostics that link to its lines, and the filter over a list.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::form::{FORM, INPUT, SELECT, TEXTAREA, check, field, help_id};
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{card, page_header, stack};
use ferrohealth_ui::widget::editor::{Editor, LineMark, MarkKind, code_editor, line_id};
use ferrohealth_ui::widget::filter::{keeps, table_filter};
use leptos::prelude::*;

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
    let confirm = view! { <input id="form-confirm" name="confirmed" type="checkbox" /> };
    card(
        "Fields",
        view! {
            <form method="post" action="#" class=FORM>
                {field("form-reason", "Why you save it, one line", Some("Two hundred characters at most."), reason)}
                {field("form-format", "Composition format", None, format)}
                {field("form-bundle", "Bundle, JSON", None, bundle)}
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
                {badge(tone, word)} " "
                <a href=format!("#{}", line_id(DRAFT, line))>{format!("line {line}")}</a>
                ": " {text}
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
            <ul class="fui-issues">
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
            "Fields, the checkbox row, the highlighted editor and the filter.",
        )),
        (),
    )
    .into_any();
    let sections = (fields(), editor(), filter());
    view! {
        {super::title("Forms and editor")}
        {header}
        {stack(view! {
            {sections.0}
            {sections.1}
            {sections.2}
        })}
    }
}
