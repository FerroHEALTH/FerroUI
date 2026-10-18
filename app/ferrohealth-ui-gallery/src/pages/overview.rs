// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The overview: the stat cards, the buttons, the status marks and notices,
//! the facts, the tabs and segmented controls, the refusal, the empty and
//! loading states, and every icon.

use ferrohealth_ui::icon::{Glyph, icon};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::{Variant, label, link_button};
use ferrohealth_ui::view::empty::{Busy, busy, empty_quiet, empty_state, skeleton, spinner};
use ferrohealth_ui::view::form::field;
use ferrohealth_ui::view::notice::{Announce, notice};
use ferrohealth_ui::view::refusal::{Issue, Refusal, Severity, fault, refusal};
use ferrohealth_ui::view::segmented::{SegmentedForm, segmented_form, segmented_links};
use ferrohealth_ui::view::status::{badge, pill, status};
use ferrohealth_ui::view::surface::{
    Stat, card, chip, cluster, definitions, disclosure, facts_row, grid, or_none, page_header,
    stack, stat_card, stat_card_marked, stat_grid, well,
};
use ferrohealth_ui::view::tabs::{KeyedTab, Tab, keyed_tabs, link_tabs};
use ferrohealth_ui::view::theme_switch::{density_switch, theme_switch};
use ferrohealth_ui::widget::segmented::segmented;
use ferrohealth_ui::widget::tabs::{Panel, tabs};
use leptos::prelude::*;

/// The formats the segmented control offers.
const FORMATS: &[(&str, &str)] = &[
    ("json", "JSON"),
    ("xml", "XML"),
    ("flat", "FLAT"),
    ("structured", "STRUCTURED"),
];

/// The path of the page `page` of the brand the route names.
fn brand_page(page: &str) -> Option<String> {
    crate::app::route_brand().map(|brand| format!("/{}/{page}", brand.key()))
}

/// The stat cards: three that lead to a page, one with a mark after its
/// label.
fn stats() -> AnyView {
    let plain = [
        (Glyph::FILES, "137", "Mapping files", Some("tables")),
        (Glyph::CIRCLE_CHECK, "112", "Compiled", Some("tables")),
        (Glyph::TRIANGLE_ALERT, "13", "With a warning", None),
    ]
    .into_iter()
    .map(|(glyph, value, label, page)| {
        stat_card(Stat {
            glyph,
            value: value.to_owned(),
            label: label.into(),
            href: page.and_then(brand_page),
        })
    })
    .collect_view();
    let marked = stat_card_marked(
        Stat {
            glyph: Glyph::NETWORK,
            value: String::from("4"),
            label: String::from("Members").into(),
            href: brand_page("tables"),
        },
        badge(Tone::Bad, "1 failing"),
    );
    stat_grid(view! {
        {plain}
        {marked}
    })
    .into_any()
}

/// The buttons, each variant in both sizes, disabled, as a link, and an
/// icon button.
fn buttons() -> AnyView {
    let full = Variant::ALL
        .into_iter()
        .map(|variant| {
            view! {
                <button type="button" class=variant.class() data-variant=variant.name()>
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
    let link = link_button(
        String::from("#buttons"),
        Variant::Secondary,
        Some(Glyph::ARROW_RIGHT),
        "A link as a button",
    );
    card(
        "Buttons",
        view! {
            {cluster(full)}
            {cluster(
                view! {
                    {small}
                    <button type="button" class=Variant::Primary.class() disabled=true>
                        {label(Some(Glyph::SAVE), "Disabled")}
                    </button>
                    {link}
                    <button type="button" class="fui-icon-button" aria-label="Settings">
                        {icon(Glyph::SETTINGS, "16")}
                    </button>
                },
            )}
        },
    )
    .into_any()
}

/// The status marks: pills, badges, chips and status indicators.
fn marks_card() -> AnyView {
    let pills = Tone::ALL
        .into_iter()
        .map(|tone| pill(tone, tone.name()))
        .collect_view();
    let badges = Tone::ALL
        .into_iter()
        .map(|tone| badge(tone, tone.name()))
        .collect_view();
    let chips = ["/ehr", "/definition", "/query", "/admin"]
        .into_iter()
        .map(chip)
        .collect_view();
    let statuses = view! {
        {status(Tone::Ok, Glyph::CIRCLE_CHECK, "Serving", Some(String::from("R4")))}
        {status(Tone::Warn, Glyph::TRIANGLE_ALERT, "Degraded", Some(String::from("1 of 3 lanes down")))}
        {status(Tone::Bad, Glyph::SERVER_CRASH, "Down", None)}
    };
    card(
        "Status",
        view! {
            {cluster(pills)}
            {cluster(badges)}
            {cluster(chips)}
            {cluster(statuses)}
        },
    )
    .into_any()
}

/// A notice in every tone.
fn notices_card() -> AnyView {
    let deleted = view! {
        "This synthetic version is deleted. "
        <a href="#marks">"Back to the files"</a>
    };
    card(
        "Notices",
        stack(view! {
            {notice(Tone::Warn, Announce::Status, None, "SMART configuration returned, but it is not valid JSON.")}
            {notice(Tone::Info, Announce::Status, None, "SMART is not enabled on this synthetic server.")}
            {notice(Tone::Ok, Announce::Status, None, "The set compiled and replaced the running one.")}
            {notice(Tone::Neutral, Announce::Status, None, deleted)}
            {notice(
                Tone::Bad,
                Announce::Alert,
                None,
                "The save was refused:\nline 4: a key is missing\nline 13: the template is not loaded",
            )}
        }),
    )
    .into_any()
}

/// The definition grids, a well and a disclosure.
fn facts() -> AnyView {
    let row = card(
        "Facts across a row",
        facts_row(vec![
            ("Concept", "Synthetic evaluation".into_any()),
            ("Version", "2.3".into_any()),
            ("Default language", "en".into_any()),
            ("Languages", "en".into_any()),
            (
                "Uid",
                view! { <code>"711d7d49-0000-4a6a-a6b4-synthetic"</code> }.into_any(),
            ),
        ]),
    )
    .into_any();
    let grid_card = card(
        "Definition grid",
        definitions(vec![
            ("Product", "FerroHEALTH UI".into_any()),
            ("Version", "0.1.2".into_any()),
            ("Commit", view! { <code>"0123456789ab"</code> }.into_any()),
            ("Profile", or_none(String::new())),
            ("Licence", "Business Source License 1.1".into_any()),
            ("Built", "2026-10-18".into_any()),
            ("Locale", "en".into_any()),
            (
                "Target",
                view! { <code>"wasm32-unknown-unknown"</code> }.into_any(),
            ),
            (
                "Version uid",
                view! { <code>"01a15050-e772-7541-b5d5-0f5ec26781d6::synthetic.local::1"</code> }
                    .into_any(),
            ),
            (
                "Subject",
                "self, with no external subject reference, so the record names no person outside the server"
                    .into_any(),
            ),
        ]),
    )
    .into_any();
    let operation = ferrohealth_ui::view::surface::link_card_glyph(
        brand_page("layouts").unwrap_or_default(),
        Glyph::PLAY,
        "Expand a value set",
        view! { <p class="fui-muted">"Run $expand over a value set this server holds."</p> },
    );
    let path = card(
        "fhirconnect/ferrohealth_synthetic_journey_diagnose.context.yml",
        view! { <p class="fui-muted">"A card titled with a file's path."</p> },
    )
    .into_any();
    let listed = card(
        "A bullet list",
        view! {
            <ul class="fui-bullets">
                <li>"$validate-code on the value sets the server loaded"</li>
                <li>"$tofhir http://fhirconnect.org/fhir/OperationDefinition/ToFhirSynthetic"</li>
            </ul>
        },
    )
    .into_any();
    let folded = card(
        "Disclosure and well",
        view! {
            {disclosure(
                "What the bridge answered, as JSON",
                view! { <p class="fui-muted">"The raw document, folded until it is asked for."</p> },
            )}
            {well(
                view! {
                    <p class="fui-small">
                        "A well sets a passage apart inside a card, such as a note on what the figures mean."
                    </p>
                },
            )}
        },
    )
    .into_any();
    let linked = ferrohealth_ui::view::surface::link_card(
        brand_page("tables").unwrap_or_default(),
        "Staged changes",
        view! { <p class="fui-muted">"Three changes wait to be applied to the running set."</p> },
    );
    let pair = grid(view! {
        {grid_card}
        {stack(view! {
            {folded}
            {linked}
            {operation}
            {path}
            {listed}
        })}
    })
    .into_any();
    stack(view! {
        {row}
        {pair}
    })
    .into_any()
}

/// The link tabs, the keyed tabs and the keyboard tabs.
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
    )
    .into_any();
    let keyed = keyed_tabs(
        "Record",
        vec![
            KeyedTab {
                key: "compositions",
                href: String::from("?tab=compositions"),
                label: "Compositions",
            },
            KeyedTab {
                key: "status",
                href: String::from("?tab=status"),
                label: "Status",
            },
            KeyedTab {
                key: "contributions",
                href: String::from("?tab=contributions"),
                label: "Contributions",
            },
            KeyedTab {
                key: "directory",
                href: String::from("?tab=directory"),
                label: "Directory",
            },
            KeyedTab {
                key: "history",
                href: String::from("?tab=history"),
                label: "Status history",
            },
            KeyedTab {
                key: "commit",
                href: String::from("?tab=commit"),
                label: "Commit",
            },
            KeyedTab {
                key: "tags",
                href: String::from("?tab=tags"),
                label: "Tags",
            },
        ],
        "status",
    )
    .into_any();
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
    )
    .into_any();
    card(
        "Tabs",
        view! {
            {links}
            {keyed}
            <p class="fui-muted">"Keyboard tabs: Left, Right, Home and End move between them."</p>
            {keyboard}
        },
    )
    .into_any()
}

/// The segmented controls: bound to state, and as links.
fn segments() -> AnyView {
    let format = RwSignal::new("json");
    let chosen = segmented("Format", FORMATS, format.into(), move |key| format.set(key)).into_any();
    let as_links = segmented_links(
        "Detail",
        vec![
            (String::from("#segments-minimal"), "Minimal"),
            (String::from("#segments-full"), "Full"),
        ],
        0,
    )
    .into_any();
    let posted = segmented_form(SegmentedForm {
        name: "fhir",
        label: "FHIR version",
        prefix: Some((Glyph::LAYERS, "FHIR")),
        choices: ["R4", "R4B", "R5", "R6"]
            .into_iter()
            .map(|version| (version.to_lowercase(), String::from(version)))
            .collect(),
        current: String::from("r4"),
        action: String::from("#segments"),
        back: String::from("/"),
    });
    card(
        "Segmented controls",
        view! {
            {cluster(
                view! {
                    {chosen}
                    {as_links}
                    {posted}
                },
            )}
            <p class="fui-muted">
                {move || format!("The document is shown as {}.", format.get().to_uppercase())}
            </p>
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
            Issue::new(Severity::Error, "processing")
                .text("No program maps the template gallery.template.v1.")
                .place("Bundle.entry[0].resource")
                .details(vec![String::from("MSG_UNKNOWN_TYPE")]),
            Issue::new(Severity::Warning, "informational"),
            Issue::new(Severity::Information, "informational"),
        ],
        request: Some(String::from(
            "POST https://bridge.example.org/fhir/Bundle?_format=json",
        )),
        evidence: None,
    };
    let unreadable = Refusal {
        status: Some(502),
        summary: String::from("the terminology server answered with no readable outcome."),
        request: Some(String::from(
            "GET https://term.example.org/fhir/ValueSet/$expand?url=http://example.org/vs",
        )),
        evidence: Some(String::from(
            "<html><head><title>502 Bad Gateway</title></head>\n<body>upstream timed out</body></html>",
        )),
        ..Refusal::default()
    };
    let silent = Refusal {
        summary: String::from("the CDR did not answer within 30 seconds."),
        ..Refusal::default()
    };
    card(
        "Refusal and fault",
        stack(view! {
            {refusal(&answered)}
            {refusal(&unreadable)}
            {refusal(&silent)}
            {fault("The console cannot reach the bridge.")}
        }),
    )
    .into_any()
}

/// The empty states, the spinner and the skeleton.
fn states() -> AnyView {
    let action = link_button(
        String::from("#states"),
        Variant::Primary,
        Some(Glyph::PLUS),
        "New file",
    );
    let empty = card(
        "Empty state",
        empty_state(
            Glyph::INBOX,
            "No mapping file",
            "The bridge serves no mapping file yet.",
            action,
        ),
    )
    .into_any();
    let quiet = card(
        "A quiet empty section",
        empty_quiet(Glyph::SEARCH_X, "No match", "No file matches the filter."),
    )
    .into_any();
    let loading = card(
        "Loading",
        stack(view! {
            {busy("Reading the capability statement", Busy::Section)}
            {spinner("Asking the bridge…")}
            <p class="fui-cluster">
                <button type="button" class=Variant::Primary.class() disabled=true>
                    {label(Some(Glyph::LOG_IN), "Sign in")}
                </button>
                {busy("Signing in", Busy::Control)}
            </p>
            {skeleton(4, "Loading the files.")}
        }),
    )
    .into_any();
    stack(view! {
        {grid(view! {
            {empty}
            {quiet}
        })}
        {loading}
    })
    .into_any()
}

/// The theme and density switches, each a plain form that posts.
fn settings() -> AnyView {
    let theme = use_context::<crate::app::InForce>()
        .map_or(ferrohealth_ui::theme::Theme::System, |in_force| in_force.0);
    let density = use_context::<crate::app::DensityInForce>()
        .map_or_else(ferrohealth_ui::theme::Density::default, |in_force| {
            in_force.0
        });
    let back = brand_page("").unwrap_or_else(|| String::from("/"));
    let back = back.trim_end_matches('/').to_owned();
    card(
        "Settings",
        ferrohealth_ui::view::form::columns(view! {
            {field(
                "settings-theme",
                "Theme",
                None,
                theme_switch(theme, String::from(crate::app::THEME_ROUTE), back.clone()),
            )}
            {field(
                "settings-density",
                "Density",
                None,
                density_switch(density, String::from(crate::app::DENSITY_ROUTE), back),
            )}
        }),
    )
    .into_any()
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
    card("Icons", cluster(all)).into_any()
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
    let marks = stack(view! {
        {marks_card()}
        {notices_card()}
    })
    .into_any();
    let sections = (
        stats(),
        buttons(),
        marks,
        facts(),
        tab_sets(),
        segments(),
        settings(),
        refusals(),
        states(),
        icons(),
    );
    view! {
        {super::title("Overview")}
        {header}
        {stack(
            view! {
                {sections.0}
                {sections.1}
                {sections.2}
                {sections.3}
                {sections.4}
                {sections.5}
                {sections.6}
                {sections.7}
                {sections.8}
                {sections.9}
            },
        )}
    }
}
