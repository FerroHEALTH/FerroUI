// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The overview: every static view of the library, and the keyboard tabs,
//! the dialog and the toasts.

use ferrohealth_ui::icon::{Glyph, icon};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::{Variant, label, link_button};
use ferrohealth_ui::view::dialog::{Confirmation, confirm_dialog, confirm_trigger};
use ferrohealth_ui::view::empty::{empty_state, skeleton};
use ferrohealth_ui::view::refusal::{Issue, Refusal, Severity, fault, refusal};
use ferrohealth_ui::view::status::{badge, pill, status};
use ferrohealth_ui::view::surface::{
    Stat, card, definitions, disclosure, grid, or_none, page_header, stack, stat_card,
};
use ferrohealth_ui::view::tabs::{Tab, link_tabs};
use ferrohealth_ui::widget::tabs::{Panel, tabs};
use ferrohealth_ui::widget::toast::use_toasts;
use leptos::prelude::*;

/// The stat cards.
fn stats() -> AnyView {
    let cards = [
        (Glyph::FILES, "137", "Mapping files", Some("tables")),
        (Glyph::CIRCLE_CHECK, "112", "Compiled", None),
        (Glyph::TRIANGLE_ALERT, "13", "With a warning", None),
        (Glyph::CIRCLE_X, "12", "Refused", None),
    ]
    .into_iter()
    .map(|(glyph, value, label, href)| {
        stat_card(Stat {
            glyph,
            value: value.to_owned(),
            label,
            href: href.and_then(|page| {
                crate::app::route_brand().map(|brand| format!("/{}/{page}", brand.key()))
            }),
        })
    })
    .collect_view();
    grid(cards).into_any()
}

/// The buttons, each variant in both sizes, disabled, and as a link.
fn buttons() -> AnyView {
    let full = Variant::ALL
        .into_iter()
        .map(|variant| {
            view! {
                <button type="button" class=variant.class()>
                    {label(Some(Glyph::PLAY), variant.name())}
                </button>
            }
        })
        .collect_view();
    let small = Variant::ALL
        .into_iter()
        .map(|variant| {
            view! {
                <button type="button" class=variant.small()>
                    {variant.name()}
                </button>
            }
        })
        .collect_view();
    card(
        "Buttons",
        view! {
            <div class="fui-actions">{full}</div>
            <div class="fui-actions">
                {small}
                <button type="button" class=Variant::Primary.class() disabled=true>
                    {label(Some(Glyph::SAVE), "Disabled")}
                </button>
                {link_button(String::from("#buttons"), Variant::Secondary, Some(Glyph::ARROW_RIGHT), "A link as a button")}
            </div>
        },
    )
    .into_any()
}

/// The status marks: a pill, a badge and a status indicator in every tone.
fn marks() -> AnyView {
    let pills = Tone::ALL
        .into_iter()
        .map(|tone| pill(tone, tone.name()))
        .collect_view();
    let badges = Tone::ALL
        .into_iter()
        .map(|tone| badge(tone, tone.name()))
        .collect_view();
    card(
        "Status",
        view! {
            <div class="fui-actions">{pills}</div>
            <div class="fui-actions">{badges}</div>
            <div class="fui-actions">
                {status(Tone::Ok, Glyph::CIRCLE_CHECK, "Serving", Some(String::from("R4")))}
                {status(Tone::Warn, Glyph::TRIANGLE_ALERT, "Degraded", Some(String::from("1 of 3 lanes down")))}
                {status(Tone::Bad, Glyph::SERVER_CRASH, "Down", None)}
            </div>
        },
    )
    .into_any()
}

/// The definition grid and a disclosure.
fn facts() -> AnyView {
    let grid_card = card(
        "Definition grid",
        definitions(vec![
            ("Product", "FerroHEALTH UI".into_any()),
            ("Version", "0.1.0".into_any()),
            ("Commit", view! { <code>"0123456789ab"</code> }.into_any()),
            ("Profile", or_none(String::new())),
            ("Licence", "Business Source License 1.1".into_any()),
        ]),
    );
    let folded = card(
        "Disclosure",
        disclosure(
            "What the bridge answered, as JSON",
            view! { <p class="fui-muted">"The raw document, folded until it is asked for."</p> },
        ),
    );
    grid(view! {
        {grid_card}
        {folded}
    })
    .into_any()
}

/// The link tabs and the keyboard tabs.
fn tab_sets() -> AnyView {
    let links = link_tabs(
        "Views of a file",
        &[
            Tab {
                href: "#tabs",
                label: "Source",
            },
            Tab {
                href: "#tabs-diagnostics",
                label: "Diagnostics",
            },
            Tab {
                href: "#tabs-history",
                label: "History",
            },
        ],
        "#tabs",
    );
    let keyboard = tabs(
        "Answer",
        "answer",
        vec![
            Panel {
                label: "Table",
                body: view! { <p>"The rows of the answer, as a table."</p> }.into_any(),
            },
            Panel {
                label: "Raw",
                body: view! { <p>"The answer as the server sent it."</p> }.into_any(),
            },
            Panel {
                label: "Request",
                body: view! { <p>"What the console asked."</p> }.into_any(),
            },
        ],
    );
    card(
        "Tabs",
        view! {
            {links}
            <p class="fui-muted">"Keyboard tabs: Left, Right, Home and End move between them."</p>
            {keyboard}
        },
    )
    .into_any()
}

/// A refusal with its issues and a fault.
fn refusals() -> AnyView {
    let answered = Refusal {
        status: Some(422),
        summary: String::from("the bridge refused the request."),
        issues: vec![
            Issue {
                severity: Severity::Error,
                code: String::from("processing"),
                text: Some(String::from(
                    "No program maps the template gallery.template.v1.",
                )),
                place: String::from("Bundle.entry[0].resource"),
            },
            Issue {
                severity: Severity::Warning,
                code: String::from("informational"),
                text: None,
                place: String::new(),
            },
            Issue {
                severity: Severity::Information,
                code: String::from("informational"),
                text: None,
                place: String::new(),
            },
        ],
    };
    card(
        "Refusal and fault",
        view! {
            {refusal(&answered)}
            {fault("The console cannot reach the bridge.")}
        },
    )
    .into_any()
}

/// The empty state and the skeleton.
fn states() -> AnyView {
    grid(view! {
        {card("Empty state", empty_state(Glyph::INBOX, "No mapping file", "The bridge serves no mapping file yet.", link_button(String::from("#states"), Variant::Primary, Some(Glyph::PLUS), "New file")))}
        {card("Loading", skeleton(4, "Loading the files."))}
    })
    .into_any()
}

/// The confirmation dialog, whose confirm raises a toast, and the toasts.
fn dialog_and_toasts() -> AnyView {
    let toasts = use_toasts();
    let confirm = move |_| {
        if let Some(toasts) = toasts {
            toasts.push(Tone::Ok, "The synthetic file was deleted.");
        }
        hide("delete-file");
    };
    let raise = move |tone: Tone, text: &'static str| {
        move |_| {
            if let Some(toasts) = toasts {
                toasts.push(tone, text);
            }
        }
    };
    card(
        "Dialog and toasts",
        view! {
            <div class="fui-actions">
                {confirm_trigger("delete-file", Variant::Danger, Glyph::TRASH, "Delete the file")}
                <button type="button" class=Variant::Secondary.class() on:click=raise(Tone::Info, "The set reloaded.")>
                    {label(Some(Glyph::BELL), "Raise a toast")}
                </button>
                <button type="button" class=Variant::Secondary.class() on:click=raise(Tone::Bad, "The save was refused: 412.")>
                    {label(Some(Glyph::CIRCLE_X), "Raise a refusal")}
                </button>
            </div>
            {confirm_dialog(
                Confirmation {
                    id: "delete-file",
                    title: "Delete the file?",
                    message: "The synthetic file goes for good; the running set reloads without it.",
                },
                view! {
                    <button type="button" class=Variant::Danger.class() on:click=confirm>
                        {icon(Glyph::TRASH, "16")}
                        "Delete"
                    </button>
                },
            )}
        },
    )
    .into_any()
}

/// Closes the popover `id`.
fn hide(id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let element = document()
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<leptos::web_sys::HtmlElement>().ok());
        if let Some(element) = element
            && element.hide_popover().is_err()
        {
            leptos::logging::warn!("the browser refused to close the dialog");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = id;
}

/// Every icon the library vendors, each with its name.
fn icons() -> AnyView {
    let all = Glyph::ALL
        .into_iter()
        .map(|(name, glyph)| {
            view! {
                <span class="fui-badge fui-tone-neutral">
                    {icon(glyph, "16")}
                    {name}
                </span>
            }
        })
        .collect_view();
    card("Icons", view! { <div class="fui-actions">{all}</div> }).into_any()
}

/// The overview page.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn OverviewPage() -> impl IntoView {
    let header = page_header(
        "Overview",
        Some(String::from(
            "Every static view of the library, in this brand's theme.",
        )),
        view! {
            <button type="button" class=Variant::Secondary.class()>
                {label(Some(Glyph::REFRESH_CW), "Reload")}
            </button>
            <button type="button" class=Variant::Primary.class()>
                {label(Some(Glyph::PLUS), "New file")}
            </button>
        },
    )
    .into_any();
    let sections = (
        stats(),
        buttons(),
        marks(),
        facts(),
        tab_sets(),
        refusals(),
        states(),
        dialog_and_toasts(),
        icons(),
    );
    view! {
        {super::title("Overview")}
        {header}
        {stack(view! {
            {sections.0}
            {sections.1}
            {sections.2}
            {sections.3}
            {sections.4}
            {sections.5}
            {sections.6}
            {sections.7}
            {sections.8}
        })}
    }
}
