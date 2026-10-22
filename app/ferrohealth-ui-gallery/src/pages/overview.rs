// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The overview: the stat cards, the buttons, the status marks and notices,
//! the facts, the tabs and segmented controls, the refusal, the empty and
//! loading states, and every icon.

use crate::locale::{Msg, Words, tr, tr_with};
use ferrohealth_ui::i18n::Phrase;
use ferrohealth_ui::icon::{Glyph, icon};
use ferrohealth_ui::island::KeyboardTabs;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::{Variant, label, link_button};
use ferrohealth_ui::view::empty::{Busy, busy, empty_quiet, empty_state, skeleton, spinner};
use ferrohealth_ui::view::layout::divided_list;
use ferrohealth_ui::view::notice::{Announce, notice};
use ferrohealth_ui::view::refusal::{Issue, Refusal, Severity, fault, refusal};
use ferrohealth_ui::view::segmented::{SegmentedForm, segmented_form, segmented_links};
use ferrohealth_ui::view::status::{badge, pill, status};
use ferrohealth_ui::view::surface::{
    Stat, card, card_action, chip, cluster, definitions, disclosure, facts_row, grid, or_none,
    page_header, stack, stat_card, stat_card_marked, stat_grid, titled_panel, well,
};
use ferrohealth_ui::view::tabs::{Tab, binder};
use ferrohealth_ui::view::tag::{Tag, tag};
use ferrohealth_ui::view::time::timestamp;
use ferrohealth_ui::widget::action_menu::{Act, Action, Item, Menu};
use ferrohealth_ui::widget::segmented::segmented;
use ferrohealth_ui::widget::tabs::tab_panels;
use leptos::prelude::*;

/// A panel that previews the newest files, its header action leading to
/// all of them.
fn preview() -> AnyView {
    let all = brand_page("tables").unwrap_or_else(|| String::from("/"));
    let files = [
        "synthetic-001.model.yml",
        "synthetic-002.extension.yml",
        "synthetic-003.context.yml",
    ]
    .into_iter()
    .map(|file| view! { <code>{file}</code> }.into_any())
    .collect();
    titled_panel(
        tr(Msg::RecentFiles),
        card_action(all, tr(Msg::AllFiles)),
        divided_list(None, files),
    )
    .into_any()
}

/// The path of the page `page` of the brand the route names.
pub(crate) fn brand_page(page: &str) -> Option<String> {
    crate::app::route_brand().map(|brand| format!("/{}/{page}", brand.key()))
}

/// The title row's actions: New file, and More with a link and a plain
/// form post, each of which works with no WebAssembly.
fn overview_actions() -> ActionGroup {
    let back = brand_page("").unwrap_or_else(|| String::from("/"));
    let back = back.trim_end_matches('/').to_owned();
    ActionGroup::primary(Action::new(
        Some(Glyph::PLUS),
        tr(Msg::NewFile),
        Act::Link(brand_page("forms").unwrap_or_else(|| String::from("/"))),
    ))
    .more(Menu::new(
        "overview-more",
        tr(Msg::MoreActionsSet),
        vec![
            Item::new(Action::new(
                Some(Glyph::FILE_CODE),
                tr(Msg::OpenTheDocument),
                Act::Link(brand_page("code").unwrap_or_else(|| String::from("/"))),
            )),
            Item::new(Action::new(
                Some(Glyph::SLIDERS_HORIZONTAL),
                tr(Msg::SwitchToCompact),
                Act::Post {
                    action: String::from(crate::app::PREFERENCE_ROUTE),
                    fields: vec![
                        (String::from("density"), String::from("compact")),
                        (String::from("back"), back),
                    ],
                },
            ))
            .note(tr(Msg::PostsTheDensity)),
        ],
    ))
}

/// The stat cards: three that lead to a page, two with a mark after their
/// label, whose glyphs and figures stay in line with the others.
fn stats() -> AnyView {
    let plain = [
        (Glyph::FILES, "137", tr(Msg::MappingFiles), Some("tables")),
        (
            Glyph::CIRCLE_CHECK,
            "112",
            tr(Msg::Compiled),
            Some("tables"),
        ),
        (Glyph::TRIANGLE_ALERT, "13", tr(Msg::WithAWarning), None),
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
            label: tr(Msg::Members).into(),
            href: brand_page("tables"),
        },
        badge(Tone::Bad, &tr(Msg::N1Failing)),
    );
    let pending = stat_card_marked(
        Stat {
            glyph: Glyph::SERVER,
            value: String::from("2"),
            label: tr(Msg::RunningRevisionEveryFederationNode).into(),
            href: None,
        },
        pill(Tone::Warn, &tr(Msg::Revision3Pending)),
    );
    stat_grid(view! {
        {plain}
        {marked}
        {pending}
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
        tr(Msg::LinkButton),
    );
    card(
        tr(Msg::Buttons),
        view! {
            {cluster(full)}
            {cluster(
                view! {
                    {small}
                    <button type="button" class=Variant::Primary.class() disabled=true>
                        {label(Some(Glyph::SAVE), tr(Msg::Disabled))}
                    </button>
                    {link}
                    <button type="button" class="fui-icon-button" aria-label=tr(Msg::Settings)>
                        {icon(Glyph::SETTINGS, "16")}
                    </button>
                },
            )}
        },
    )
    .into_any()
}

/// The status marks: pills, badges, chips, tags and status indicators.
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
    let tags = view! {
        {tag(tr(Msg::FhirR4))}
        {Tag::new(tr(Msg::FhirConnect)).code("fhirconnect").view()}
        {Tag::new(tr(Msg::ReadOperation)).code("r").view()}
        {Tag::new("1.1.0").mono().view()}
        {Tag::new(tr(Msg::Directory)).locked().title(tr(Msg::FromDirectory)).view()}
    };
    let statuses = view! {
        {status(Tone::Ok, Glyph::CIRCLE_CHECK, &tr(Msg::Serving), Some(tr(Msg::R4)))}
        {status(Tone::Warn, Glyph::TRIANGLE_ALERT, &tr(Msg::Degraded), Some(tr(Msg::N13LanesDown)))}
        {status(Tone::Bad, Glyph::SERVER_CRASH, &tr(Msg::Down), None)}
    };
    card(
        tr(Msg::Status),
        view! {
            {cluster(pills)}
            {cluster(badges)}
            {cluster(chips)}
            {cluster(tags)}
            {cluster(statuses)}
        },
    )
    .into_any()
}

/// A notice in every tone.
fn notices_card() -> AnyView {
    let deleted = view! {
        {tr(Msg::SyntheticVersionDeleted)}
        <a href="#marks">{tr(Msg::BackFiles)}</a>
    };
    card(
        tr(Msg::Notices),
        stack(view! {
            {notice(Tone::Warn, Announce::Status, None, tr(Msg::SmartConfigurationReturnedButNot))}
            {notice(Tone::Info, Announce::Status, None, tr(Msg::SmartNotEnabledSyntheticServer))}
            {notice(Tone::Ok, Announce::Status, None, tr(Msg::SetCompiledReplacedRunningOne))}
            {notice(Tone::Neutral, Announce::Status, None, deleted)}
            {notice(
                Tone::Bad,
                Announce::Alert,
                None,
                tr(Msg::SaveRefusedIssues),
            )}
        }),
    )
    .into_any()
}

/// The facts across a row, one of them a timestamp.
fn facts_across() -> AnyView {
    card(
        tr(Msg::FactsAcrossRow),
        facts_row(vec![
            (tr(Msg::Concept), tr(Msg::SyntheticEvaluation).into_any()),
            (tr(Msg::Version), "2.3".into_any()),
            (tr(Msg::DefaultLanguage), "en".into_any()),
            (tr(Msg::Languages), "en".into_any()),
            (
                tr(Msg::LastChanged),
                timestamp("2026-10-19T01:18:37+02:00").into_any(),
            ),
            (
                tr(Msg::Uid),
                view! { <code>"711d7d49-0000-4a6a-a6b4-synthetic"</code> }.into_any(),
            ),
        ]),
    )
    .into_any()
}

/// The definition grids, a well and a disclosure.
fn facts() -> AnyView {
    let row = facts_across();
    let grid_card =
        card(
            tr(Msg::DefinitionGrid),
            definitions(vec![
            (tr(Msg::ProductColumn), tr(Msg::FerrohealthUi).into_any()),
            (tr(Msg::Version), "0.1.7".into_any()),
            (tr(Msg::Commit), view! { <code>"0123456789ab"</code> }.into_any()),
            (tr(Msg::Profile), or_none(String::new())),
            (tr(Msg::Licence), tr(Msg::BusinessSourceLicense11).into_any()),
            (tr(Msg::Built), timestamp("2026-10-18T23:18:37.039437+00:00").into_any()),
            (tr(Msg::Locale), "en".into_any()),
            (
                tr(Msg::Target),
                view! { <code>{tr(Msg::Wasm32UnknownUnknown)}</code> }.into_any(),
            ),
            (
                tr(Msg::VersionUid),
                view! { <code>"01a15050-e772-7541-b5d5-0f5ec26781d6::synthetic.local::1"</code> }
                    .into_any(),
            ),
            (
                tr(Msg::Subject),
                {tr(Msg::SelfNoExternalSubjectReference)}
                    .into_any(),
            ),
        ]),
        )
        .into_any();
    let operation = ferrohealth_ui::view::surface::link_card_glyph(
        brand_page("layouts").unwrap_or_default(),
        Glyph::PLAY,
        tr(Msg::ExpandValueSet),
        view! { <p class="fui-muted">{tr(Msg::RunExpandOverValueSet)}</p> },
    );
    let path = card(
        "fhirconnect/ferrohealth_synthetic_journey_diagnose.context.yml",
        view! { <p class="fui-muted">{tr(Msg::CardTitledFileSPath)}</p> },
    )
    .into_any();
    let listed = card(
        tr(Msg::BulletList),
        view! {
            <ul class="fui-bullets">
                <li>{tr(Msg::ValidateCodeValueSetsServer)}</li>
                <li>{tr(Msg::TofhirHttpFhirconnectOrgFhir)}</li>
            </ul>
        },
    )
    .into_any();
    let folded = card(
        tr(Msg::DisclosureWell),
        view! {
            {disclosure(
                tr(Msg::WhatBridgeAnsweredJson),
                view! { <p class="fui-muted">{tr(Msg::RawDocumentFoldedUntilAsked)}</p> },
            )}
            {well(
                view! {
                    <p class="fui-small">
                        {tr(Msg::WellSetsPassageApartInside)}
                    </p>
                },
            )}
        },
    )
    .into_any();
    let linked = ferrohealth_ui::view::surface::link_card(
        brand_page("tables").unwrap_or_default(),
        tr(Msg::StagedChanges),
        view! { <p class="fui-muted">{tr(Msg::ThreeChangesWaitAppliedRunning)}</p> },
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

/// The panel of a binder: one sentence.
fn binder_panel(words: String) -> AnyView {
    view! { <p>{words}</p> }.into_any()
}

/// The binder tabs: a file's views with a count, a record's seven sections
/// that a phone shows as a select, preferences with a draft and the
/// Keyboard tab, and the keyboard tabs that switch in place.
fn tab_sets() -> AnyView {
    let views = binder(
        tr(Msg::ViewsFile),
        vec![
            Tab::new("source", "#tabs", tr(Msg::Source)),
            Tab::new("diagnostics", "#tabs-diagnostics", tr(Msg::Diagnostics))
                .count(2, tr_with(Msg::FindingsCount, &[("count", 2.into())])),
            Tab::new("history", "#tabs-history", tr(Msg::History)),
        ],
        "source",
        binder_panel(tr(Msg::BinderPanelJoined)),
    );
    let record = binder(
        tr(Msg::Record),
        [
            ("compositions", Msg::Compositions),
            ("status", Msg::Status),
            ("contributions", Msg::Contributions),
            ("directory", Msg::Directory),
            ("history", Msg::StatusHistory),
            ("commit", Msg::Commit),
            ("tags", Msg::Tags),
        ]
        .into_iter()
        .map(|(key, msg)| Tab::new(key, format!("?tab={key}"), tr(msg)))
        .collect(),
        "status",
        binder_panel(tr(Msg::BinderManyTabsSelect)),
    );
    let preferences = binder(
        tr(Msg::YourPreferences),
        vec![
            Tab::new("appearance", "?prefs=appearance", tr(Msg::Appearance)),
            Tab::new("keyboard", "?prefs=keyboard", tr(Msg::Keyboard)),
            Tab::new(
                "notifications",
                "?prefs=notifications",
                tr(Msg::Notifications),
            )
            .draft(),
        ],
        "appearance",
        binder_panel(tr(Msg::BinderPreferencesDraft)),
    );
    let panels = tab_panels(
        "answer",
        vec![
            view! { <p>{tr(Msg::RowsAnswerTable)}</p> }.into_any(),
            view! { <p>{tr(Msg::AnswerServerSent)}</p> }.into_any(),
            view! { <p>{tr(Msg::WhatConsoleAsked)}</p> }.into_any(),
        ],
    );
    let keyboard = view! {
        <KeyboardTabs
            label=tr(Msg::Answer)
            prefix=String::from("answer")
            tabs=vec![tr(Msg::Table), tr(Msg::Raw), tr(Msg::Request)]
        >
            {panels}
        </KeyboardTabs>
    }
    .into_any();
    card(
        tr(Msg::Tabs),
        stack(view! {
            {views}
            {record}
            {preferences}
            <p class="fui-muted">{tr(Msg::KeyboardTabsLeftRightHome)}</p>
            {keyboard}
        }),
    )
    .into_any()
}

/// The segmented control bound to state at the head of the cluster of
/// `others`, in `words`, and the line that follows it.
pub(crate) fn format_segments(words: &Words, others: AnyView) -> AnyView {
    let format = RwSignal::new("json");
    let formats = [
        ("json", words.get(Msg::Json)),
        ("xml", words.get(Msg::Xml)),
        ("flat", words.get(Msg::FlatUpper)),
        ("structured", words.get(Msg::Structured)),
    ];
    let chosen = segmented(
        words.get(Msg::Format),
        &formats,
        format.into(),
        move |key| {
            format.set(key);
        },
    )
    .into_any();
    let shown = words.get(Msg::ShownAs);
    view! {
        {cluster(view! {
            {chosen}
            {others}
        })}
        <p class="fui-muted">
            {move || Phrase::new(shown.clone()).fill(&[("format", &format.get().to_uppercase())])}
        </p>
    }
    .into_any()
}

/// The segmented controls: bound to state, and as links.
fn segments() -> AnyView {
    let as_links = segmented_links(
        tr(Msg::Detail),
        vec![
            (String::from("#segments-minimal"), tr(Msg::Minimal)),
            (String::from("#segments-full"), tr(Msg::Full)),
        ],
        0,
    )
    .into_any();
    let posted = segmented_form(SegmentedForm {
        name: "fhir",
        label: tr(Msg::FhirVersion).into(),
        prefix: Some((Glyph::LAYERS, tr(Msg::Fhir).into())),
        choices: [tr(Msg::R4), tr(Msg::R4b), tr(Msg::R5), tr(Msg::R6)]
            .into_iter()
            .map(|version| (version.to_lowercase(), version))
            .collect(),
        current: String::from("r4"),
        action: String::from("#segments"),
        back: String::from("/"),
    });
    card(
        tr(Msg::SegmentedControls),
        view! {
            <crate::islands::FormatSegments words=Words::of(&[
                Msg::Json,
                Msg::Xml,
                Msg::FlatUpper,
                Msg::Structured,
                Msg::Format,
            ]).open(Msg::ShownAs, &["format"])>
                {as_links}
                {posted}
            </crate::islands::FormatSegments>
        },
    )
    .into_any()
}

/// A refusal with its issues and a fault.
fn refusals() -> AnyView {
    let answered = Refusal {
        status: Some(422),
        summary: tr(Msg::BridgeRefusedRequest),
        issues: vec![
            Issue::new(Severity::Error, "processing")
                .text(tr(Msg::NoProgramMapsTemplateGallery))
                .place(tr(Msg::BundleEntry0Resource))
                .details(vec![tr(Msg::UnknownTypeCode)]),
            Issue::new(Severity::Warning, "informational"),
            Issue::new(Severity::Information, "informational"),
        ],
        request: Some(tr(Msg::PostHttpsBridgeExampleOrg)),
        evidence: None,
    };
    let unreadable = Refusal {
        status: Some(502),
        summary: tr(Msg::TerminologyServerAnsweredNoReadable),
        request: Some(tr(Msg::GetHttpsTermExampleOrg)),
        evidence: Some(String::from(
            "<html><head><title>502 Bad Gateway</title></head>\n<body>upstream timed out</body></html>",
        )),
        ..Refusal::default()
    };
    let silent = Refusal {
        summary: tr(Msg::CdrDidNotAnswerWithin),
        ..Refusal::default()
    };
    card(
        tr(Msg::RefusalFault),
        stack(view! {
            {refusal(&answered)}
            {refusal(&unreadable)}
            {refusal(&silent)}
            {fault(tr(Msg::ConsoleCannotReachBridge))}
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
        tr(Msg::NewFile),
    );
    let empty = card(
        tr(Msg::EmptyState),
        empty_state(
            Glyph::INBOX,
            tr(Msg::NoMappingFile),
            tr(Msg::BridgeServesNoMappingFile),
            action,
        ),
    )
    .into_any();
    let quiet = card(
        tr(Msg::QuietEmptySection),
        empty_quiet(
            Glyph::SEARCH_X,
            tr(Msg::NoMatch),
            tr(Msg::NoFileMatchesFilter),
        ),
    )
    .into_any();
    let loading = card(
        tr(Msg::Loading),
        stack(view! {
            {busy(tr(Msg::ReadingCapabilityStatement), Busy::Section)}
            {spinner(tr(Msg::AskingBridge))}
            <p class="fui-cluster">
                <button type="button" class=Variant::Primary.class() disabled=true>
                    {label(Some(Glyph::LOG_IN), tr(Msg::SignIn))}
                </button>
                {busy(tr(Msg::Signing), Busy::Control)}
            </p>
            {skeleton(4, tr(Msg::LoadingFiles))}
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
    card(tr(Msg::Icons), cluster(all)).into_any()
}

/// The overview page.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn OverviewPage() -> impl IntoView {
    let header = page_header(
        tr(Msg::Overview),
        Some(tr(Msg::EveryStaticViewLibraryBrand)),
        overview_actions(),
    )
    .into_any();
    let marks = stack(view! {
        {marks_card()}
        {notices_card()}
    })
    .into_any();
    let leading = stack(view! {
        {stats()}
        {preview()}
    })
    .into_any();
    let sections = (
        leading,
        buttons(),
        marks,
        facts(),
        tab_sets(),
        segments(),
        refusals(),
        states(),
        icons(),
    );
    view! {
        {super::title(tr(Msg::Overview))}
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
            },
        )}
    }
}
