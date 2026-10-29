// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The overview: the stat cards, the buttons, the status marks and notices,
//! the facts, the tabs and segmented controls, the refusal, the empty and
//! loading states, and every icon.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::Locale;
use ferrohealth_ui::icon::{Glyph, icon};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::action_menu::{Act, Action, Item, Menu};
use ferrohealth_ui::view::button::{Variant, label, link_button};
use ferrohealth_ui::view::empty::{Busy, busy, empty_quiet, empty_state, skeleton, spinner};
use ferrohealth_ui::view::layout::divided_list;
use ferrohealth_ui::view::notice::{Announce, notice, notice_text};
use ferrohealth_ui::view::refusal::{Issue, Refusal, Severity, fault, refusal};
use ferrohealth_ui::view::segmented::{
    SegmentedForm, choice_group, segmented_form, segmented_links,
};
use ferrohealth_ui::view::state::{Mark, state};
use ferrohealth_ui::view::status::{badge, banner, pill, status};
use ferrohealth_ui::view::surface::{
    Stat, card, card_action, chip, cluster, definitions, disclosure, facts_row, grid, link_card,
    link_card_glyph, or_none, page_header, stack, stat_card, stat_card_marked, stat_grid,
    titled_panel, well,
};
use ferrohealth_ui::view::tabs::{Tab, binder, keyboard_tabs, tab_panels};
use ferrohealth_ui::view::tag::{Tag, tag};
use ferrohealth_ui::view::time::timestamp;

use crate::app::Page;
use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

/// The path of the page `page` of the visit's brand.
pub(crate) fn brand_page(v: &Visit, page: &str) -> String {
    format!("/{}/{page}", v.brand.key())
}

/// A panel that previews the newest files, its header action leading to
/// all of them.
fn preview(v: &Visit) -> Html {
    let files = [
        "synthetic-001.model.yml",
        "synthetic-002.extension.yml",
        "synthetic-003.context.yml",
    ]
    .into_iter()
    .map(|file| h(format!("<code>{file}</code>")))
    .collect();
    titled_panel(
        v.tr(Msg::RecentFiles),
        card_action(brand_page(v, "tables"), v.tr(Msg::AllFiles)),
        divided_list(None, files),
    )
}

/// The title row's actions: New file, and More with a link and a plain
/// form post, each of which works with no script.
fn overview_actions(v: &Visit) -> ActionGroup {
    let back = brand_page(v, "").trim_end_matches('/').to_owned();
    ActionGroup::primary(Action::new(
        Some(Glyph::PLUS),
        v.tr(Msg::NewFile),
        Act::Link(brand_page(v, "forms")),
    ))
    .more(Menu::new(
        "overview-more",
        v.tr(Msg::MoreActionsSet),
        vec![
            Item::new(Action::new(
                Some(Glyph::FILE_CODE),
                v.tr(Msg::OpenTheDocument),
                Act::Link(brand_page(v, "code")),
            )),
            Item::new(Action::new(
                Some(Glyph::SLIDERS_HORIZONTAL),
                v.tr(Msg::SwitchToCompact),
                Act::Post {
                    action: String::from(crate::app::PREFERENCE_ROUTE),
                    fields: vec![
                        (String::from("density"), String::from("compact")),
                        (String::from("back"), back),
                    ],
                },
            ))
            .note(v.tr(Msg::PostsTheDensity)),
        ],
    ))
}

/// The stat cards: three that lead to a page, two with a mark after their
/// label, whose glyphs and figures stay in line with the others.
fn stats(v: &Visit) -> Html {
    let plain: Html = [
        (Glyph::FILES, "137", v.tr(Msg::MappingFiles), Some("tables")),
        (
            Glyph::CIRCLE_CHECK,
            "112",
            v.tr(Msg::Compiled),
            Some("tables"),
        ),
        (Glyph::TRIANGLE_ALERT, "13", v.tr(Msg::WithAWarning), None),
    ]
    .into_iter()
    .map(|(glyph, value, label, page)| {
        stat_card(Stat {
            glyph,
            value: value.to_owned(),
            label: label.into(),
            href: page.map(|page| brand_page(v, page)),
        })
    })
    .collect();
    let marked = stat_card_marked(
        Stat {
            glyph: Glyph::NETWORK,
            value: String::from("4"),
            label: v.tr(Msg::Members).into(),
            href: Some(brand_page(v, "tables")),
        },
        badge(Tone::Bad, v.tr(Msg::N1Failing)),
    );
    let pending = stat_card_marked(
        Stat {
            glyph: Glyph::SERVER,
            value: String::from("2"),
            label: v.tr(Msg::RunningRevisionEveryFederationNode).into(),
            href: None,
        },
        pill(Tone::Warn, &v.tr(Msg::Revision3Pending)),
    );
    stat_grid(Html::join([plain, marked, pending]))
}

/// The buttons, each variant in both sizes, disabled, as a link, and an
/// icon button.
fn buttons(v: &Visit) -> Html {
    let full = Variant::ALL
        .into_iter()
        .map(|variant| {
            h(format!(
                "<button type=\"button\" class=\"{}\" data-variant=\"{}\">{}</button>",
                variant.class(),
                t(variant.name(&Locale::english()).text()),
                label(Some(Glyph::PLAY), variant.name(&v.locale))
            ))
        })
        .collect();
    let small: Html = Variant::ALL
        .into_iter()
        .map(|variant| {
            h(format!(
                "<button type=\"button\" class=\"{}\">{}</button>",
                variant.small(),
                variant.name(&v.locale)
            ))
        })
        .collect();
    let link = link_button(
        String::from("#buttons"),
        Variant::Secondary,
        Some(Glyph::ARROW_RIGHT),
        v.tr(Msg::LinkButton),
    );
    card(
        v.tr(Msg::Buttons),
        Html::join([
            cluster(full),
            cluster(h(format!(
                "{small}<button type=\"button\" class=\"{}\" disabled>{}</button>{link}<button type=\"button\" class=\"fui-icon-button\" aria-label=\"{}\">{}</button>",
                Variant::Primary.class(),
                label(Some(Glyph::SAVE), v.tr(Msg::Disabled)),
                t(v.tr(Msg::Settings)),
                icon(Glyph::SETTINGS, "16")
            ))),
        ]),
    )
}

/// The status marks: pills, badges, chips, tags, status indicators and
/// the F12 state marks.
fn marks_card(v: &Visit) -> Html {
    let pills = Tone::ALL
        .into_iter()
        .map(|tone| pill(tone, tone.name()))
        .collect();
    let badges = Tone::ALL
        .into_iter()
        .map(|tone| badge(tone, tone.name()))
        .collect();
    let chips = ["/ehr", "/definition", "/query", "/admin"]
        .into_iter()
        .map(chip)
        .collect();
    let tags = Html::join([
        tag(v.tr(Msg::FhirR4)),
        Tag::new(v.tr(Msg::FhirConnect)).code("fhirconnect").view(),
        Tag::new(v.tr(Msg::ReadOperation)).code("r").view(),
        Tag::new("1.1.0").mono().view(),
        Tag::new(v.tr(Msg::Directory))
            .locked(v.tr(Msg::FromDirectory))
            .view(),
    ]);
    let statuses = Html::join([
        status(
            Tone::Ok,
            Glyph::CIRCLE_CHECK,
            &v.tr(Msg::Serving),
            Some(v.tr(Msg::R4)),
        ),
        status(
            Tone::Warn,
            Glyph::TRIANGLE_ALERT,
            &v.tr(Msg::Degraded),
            Some(v.tr(Msg::N13LanesDown)),
        ),
        status(Tone::Bad, Glyph::SERVER_CRASH, &v.tr(Msg::Down), None),
    ]);
    let states = Html::join([
        state(Mark::Ok, v.tr(Msg::MarkReachable), None),
        state(
            Mark::Warn,
            v.tr(Msg::Degraded),
            Some(v.tr(Msg::MarkLatency)),
        ),
        state(Mark::Bad, v.tr(Msg::MarkUnreachable), None),
        state(Mark::Neutral, v.tr(Msg::MarkNotChecked), None),
        state(Mark::Off, v.tr(Msg::MarkExcluded), None),
    ]);
    card(
        v.tr(Msg::Status),
        Html::join([
            cluster(pills),
            cluster(badges),
            cluster(chips),
            cluster(tags),
            cluster(statuses),
            cluster(states),
        ]),
    )
}

/// A notice in every tone.
fn notices_card(v: &Visit) -> Html {
    let deleted = h(format!(
        "{}<a href=\"#marks\">{}</a>",
        t(v.tr(Msg::SyntheticVersionDeleted)),
        t(v.tr(Msg::BackFiles))
    ));
    card(
        v.tr(Msg::Notices),
        stack(Html::join([
            notice_text(
                Tone::Warn,
                Announce::Status,
                v.tr(Msg::SmartConfigurationReturnedButNot),
            ),
            notice_text(
                Tone::Info,
                Announce::Status,
                v.tr(Msg::SmartNotEnabledSyntheticServer),
            ),
            notice_text(
                Tone::Ok,
                Announce::Status,
                v.tr(Msg::SetCompiledReplacedRunningOne),
            ),
            notice(Tone::Neutral, Announce::Status, None, deleted),
            notice_text(Tone::Bad, Announce::Alert, v.tr(Msg::SaveRefusedIssues)),
        ])),
    )
}

/// A banner whose sentence holds a URL longer than a phone is wide, which
/// wraps inside the banner.
fn banners_card(v: &Visit) -> Html {
    let body = v.tr_with(
        Msg::BannerSandboxBody,
        &[(
            "value",
            "https://sandbox.demo-regio.example.org/fhir/r4/metadata?_format=application/fhir+json"
                .into(),
        )],
    );
    card(
        v.tr(Msg::Banners),
        banner(
            Tone::Info,
            Glyph::INFO,
            v.tr(Msg::BannerSandbox),
            &body,
            None,
        ),
    )
}

/// The facts across a row, one of them a timestamp.
fn facts_across(v: &Visit) -> Html {
    card(
        v.tr(Msg::FactsAcrossRow),
        facts_row(vec![
            (v.tr(Msg::Concept), t(v.tr(Msg::SyntheticEvaluation))),
            (v.tr(Msg::Version), t("2.3")),
            (v.tr(Msg::DefaultLanguage), t("en")),
            (v.tr(Msg::Languages), t("en")),
            (
                v.tr(Msg::LastChanged),
                timestamp(&v.locale, "2026-10-19T01:18:37+02:00"),
            ),
            (
                v.tr(Msg::Uid),
                h(format!(
                    "<code>{}</code>",
                    "711d7d49-0000-4a6a-a6b4-synthetic"
                )),
            ),
        ]),
    )
}

/// The definition grids, a well and a disclosure.
fn facts(v: &Visit) -> Html {
    let row = facts_across(v);
    let grid_card = card(
        v.tr(Msg::DefinitionGrid),
        definitions(vec![
            (v.tr(Msg::ProductColumn), t(v.tr(Msg::FerrohealthUi))),
            (v.tr(Msg::Version), t("0.1.7")),
            (
                v.tr(Msg::Commit),
                h(format!("<code>{}</code>", "0123456789ab")),
            ),
            (v.tr(Msg::Profile), or_none(&v.locale, String::new())),
            (v.tr(Msg::Licence), t(v.tr(Msg::BusinessSourceLicense11))),
            (
                v.tr(Msg::Built),
                timestamp(&v.locale, "2026-10-18T23:18:37.039437+00:00"),
            ),
            (v.tr(Msg::Locale), t("en")),
            (
                v.tr(Msg::Target),
                h(format!(
                    "<code>{}</code>",
                    t(v.tr(Msg::Wasm32UnknownUnknown))
                )),
            ),
            (
                v.tr(Msg::VersionUid),
                h(String::from(
                    "<code>01a15050-e772-7541-b5d5-0f5ec26781d6::synthetic.local::1</code>",
                )),
            ),
            (
                v.tr(Msg::Subject),
                t(v.tr(Msg::SelfNoExternalSubjectReference)),
            ),
        ]),
    );
    let muted = |msg| h(format!("<p class=\"fui-muted\">{}</p>", t(v.tr(msg))));
    let operation = link_card_glyph(
        brand_page(v, "layouts"),
        Glyph::PLAY,
        v.tr(Msg::ExpandValueSet),
        muted(Msg::RunExpandOverValueSet),
    );
    let path = card(
        "fhirconnect/ferrohealth_synthetic_journey_diagnose.context.yml",
        muted(Msg::CardTitledFileSPath),
    );
    let listed = card(
        v.tr(Msg::BulletList),
        h(format!(
            "<ul class=\"fui-bullets\"><li>{}</li><li>{}</li></ul>",
            t(v.tr(Msg::ValidateCodeValueSetsServer)),
            t(v.tr(Msg::TofhirHttpFhirconnectOrgFhir))
        )),
    );
    let folded = card(
        v.tr(Msg::DisclosureWell),
        Html::join([
            disclosure(
                v.tr(Msg::WhatBridgeAnsweredJson),
                muted(Msg::RawDocumentFoldedUntilAsked),
            ),
            well(h(format!(
                "<p class=\"fui-small\">{}</p>",
                t(v.tr(Msg::WellSetsPassageApartInside))
            ))),
        ]),
    );
    let linked = link_card(
        brand_page(v, "tables"),
        v.tr(Msg::StagedChanges),
        muted(Msg::ThreeChangesWaitAppliedRunning),
    );
    let pair = grid(Html::join([
        grid_card,
        stack(Html::join([folded, linked, operation, path, listed])),
    ]));
    stack(Html::join([row, pair]))
}

/// The panel of a binder: one sentence.
fn binder_panel(words: &str) -> Html {
    h(format!("<p>{}</p>", t(words)))
}

/// The binder tabs: a file's views with a count, a record's seven sections
/// that a phone shows as a select, preferences with a draft and the
/// Keyboard tab, and the keyboard tabs that switch in place.
fn tab_sets(v: &Visit) -> Html {
    let views = binder(
        &v.locale,
        v.tr(Msg::ViewsFile),
        vec![
            Tab::new("source", "#tabs", v.tr(Msg::Source)),
            Tab::new("diagnostics", "#tabs-diagnostics", v.tr(Msg::Diagnostics))
                .count(2, v.tr_with(Msg::FindingsCount, &[("count", 2.into())])),
            Tab::new("history", "#tabs-history", v.tr(Msg::History)),
        ],
        "source",
        binder_panel(&v.tr(Msg::BinderPanelJoined)),
    );
    let record = binder(
        &v.locale,
        v.tr(Msg::Record),
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
        .map(|(key, msg)| Tab::new(key, format!("?tab={key}"), v.tr(msg)))
        .collect(),
        "status",
        binder_panel(&v.tr(Msg::BinderManyTabsSelect)),
    );
    let preferences = binder(
        &v.locale,
        v.tr(Msg::YourPreferences),
        vec![
            Tab::new("appearance", "?prefs=appearance", v.tr(Msg::Appearance)),
            Tab::new("keyboard", "?prefs=keyboard", v.tr(Msg::Keyboard)),
            Tab::new(
                "notifications",
                "?prefs=notifications",
                v.tr(Msg::Notifications),
            )
            .draft(),
        ],
        "appearance",
        binder_panel(&v.tr(Msg::BinderPreferencesDraft)),
    );
    let panels = tab_panels(
        "answer",
        vec![
            binder_panel(&v.tr(Msg::RowsAnswerTable)),
            binder_panel(&v.tr(Msg::AnswerServerSent)),
            binder_panel(&v.tr(Msg::WhatConsoleAsked)),
        ],
    );
    let keyboard = keyboard_tabs(
        &v.tr(Msg::Answer),
        "answer",
        &[v.tr(Msg::Table), v.tr(Msg::Raw), v.tr(Msg::Request)],
        panels,
    );
    card(
        v.tr(Msg::Tabs),
        stack(Html::join([
            views,
            record,
            preferences,
            h(format!(
                "<p class=\"fui-muted\">{}</p>",
                t(v.tr(Msg::KeyboardTabsLeftRightHome))
            )),
            keyboard,
        ])),
    )
}

/// The segmented controls: radio segments, as links, and as a form that
/// posts its choice.
fn segments(v: &Visit) -> Html {
    let formats = [
        ("json", v.tr(Msg::Json)),
        ("xml", v.tr(Msg::Xml)),
        ("flat", v.tr(Msg::FlatUpper)),
        ("structured", v.tr(Msg::Structured)),
    ];
    let chosen = choice_group("format", v.tr(Msg::Format), &formats, "json");
    let as_links = segmented_links(
        v.tr(Msg::Detail),
        vec![
            (String::from("#segments-minimal"), v.tr(Msg::Minimal)),
            (String::from("#segments-full"), v.tr(Msg::Full)),
        ],
        0,
    );
    let posted = segmented_form(SegmentedForm {
        name: "fhir",
        label: v.tr(Msg::FhirVersion).into(),
        prefix: Some((Glyph::LAYERS, v.tr(Msg::Fhir).into())),
        choices: [v.tr(Msg::R4), v.tr(Msg::R4b), v.tr(Msg::R5), v.tr(Msg::R6)]
            .into_iter()
            .map(|version| (version.to_lowercase(), version.into()))
            .collect(),
        current: String::from("r4"),
        action: String::from("#segments"),
        back: String::from("/"),
    });
    let shown = v.tr_with(Msg::ShownAs, &[("format", "json".to_uppercase().into())]);
    card(
        v.tr(Msg::SegmentedControls),
        Html::join([
            cluster(Html::join([chosen, as_links, posted])),
            h(format!("<p class=\"fui-muted\">{}</p>", t(shown))),
        ]),
    )
}

/// A refusal with its issues and a fault.
fn refusals(v: &Visit) -> Html {
    let answered = Refusal {
        status: Some(422),
        summary: v.tr(Msg::BridgeRefusedRequest),
        issues: vec![
            Issue::new(Severity::Error, "processing")
                .text(v.tr(Msg::NoProgramMapsTemplateGallery))
                .place(v.tr(Msg::BundleEntry0Resource))
                .details(vec![v.tr(Msg::UnknownTypeCode)]),
            Issue::new(Severity::Warning, "informational"),
            Issue::new(Severity::Information, "informational"),
        ],
        request: Some(v.tr(Msg::PostHttpsBridgeExampleOrg)),
        evidence: None,
    };
    let unreadable = Refusal {
        status: Some(502),
        summary: v.tr(Msg::TerminologyServerAnsweredNoReadable),
        request: Some(v.tr(Msg::GetHttpsTermExampleOrg)),
        evidence: Some(String::from(
            "<html><head><title>502 Bad Gateway</title></head>\n<body>upstream timed out</body></html>",
        )),
        ..Refusal::default()
    };
    let silent = Refusal {
        summary: v.tr(Msg::CdrDidNotAnswerWithin),
        ..Refusal::default()
    };
    card(
        v.tr(Msg::RefusalFault),
        stack(Html::join([
            refusal(&v.locale, &answered),
            refusal(&v.locale, &unreadable),
            refusal(&v.locale, &silent),
            fault(v.tr(Msg::ConsoleCannotReachBridge)),
        ])),
    )
}

/// The empty states, the spinner and the skeleton.
fn states(v: &Visit) -> Html {
    let action = link_button(
        String::from("#states"),
        Variant::Primary,
        Some(Glyph::PLUS),
        v.tr(Msg::NewFile),
    );
    let empty = card(
        v.tr(Msg::EmptyState),
        empty_state(
            Glyph::INBOX,
            v.tr(Msg::NoMappingFile),
            v.tr(Msg::BridgeServesNoMappingFile),
            action,
        ),
    );
    let quiet = card(
        v.tr(Msg::QuietEmptySection),
        empty_quiet(
            Glyph::SEARCH_X,
            v.tr(Msg::NoMatch),
            v.tr(Msg::NoFileMatchesFilter),
        ),
    );
    let loading = card(
        v.tr(Msg::Loading),
        stack(Html::join([
            busy(v.tr(Msg::ReadingCapabilityStatement), Busy::Section),
            spinner(v.tr(Msg::AskingBridge)),
            h(format!(
                "<p class=\"fui-cluster\"><button type=\"button\" class=\"{}\" disabled>{}</button>{}</p>",
                Variant::Primary.class(),
                label(Some(Glyph::LOG_IN), v.tr(Msg::SignIn)),
                busy(v.tr(Msg::Signing), Busy::Control)
            )),
            skeleton(4, v.tr(Msg::LoadingFiles)),
        ])),
    );
    stack(Html::join([grid(Html::join([empty, quiet])), loading]))
}

/// Every icon the library vendors, each with its name.
fn icons(v: &Visit) -> Html {
    let all = Glyph::ALL
        .into_iter()
        .map(|(name, glyph)| {
            h(format!(
                "<span class=\"fui-badge fui-tone-neutral\">{}{name}</span>",
                icon(glyph, "16")
            ))
        })
        .collect();
    card(v.tr(Msg::Icons), cluster(all))
}

/// The overview page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let header = page_header(
        &v.locale,
        v.tr(Msg::Overview),
        Some(v.tr(Msg::EveryStaticViewLibraryBrand)),
        overview_actions(v),
    );
    let body = stack(Html::join([
        stack(Html::join([stats(v), preview(v)])),
        buttons(v),
        stack(Html::join([
            marks_card(v),
            notices_card(v),
            banners_card(v),
        ])),
        facts(v),
        tab_sets(v),
        segments(v),
        refusals(v),
        states(v),
        icons(v),
    ]));
    Page::shell(v.tr(Msg::Overview), Html::join([header, body]))
}
