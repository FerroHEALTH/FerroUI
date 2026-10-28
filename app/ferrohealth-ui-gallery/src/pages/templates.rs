// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The page templates with the reference content of `design/templates/`.
//!
//! They are T1 short, long, empty and under tabs, T2, T3 from a file and managed,
//! the About page, and the error page as a 403 and a 404 in the shell and a
//! 500 in the centred layout. Each is a page of its own, so the screenshots
//! show it whole.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::filter::{FilterChoice, FilterMenu, MenuForm, filter_menu};
use ferrohealth_ui::table::model::{Paging, Window};
use ferrohealth_ui::table::pager::{Listing, link_pager};
use ferrohealth_ui::table::view::{Body, Column, DataTable, Role, Sorting, data_table};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::{Variant, label, link_button};
use ferrohealth_ui::view::dialog::{Confirmation, Explanation, danger_dialog, info_dialog};
use ferrohealth_ui::view::error_page::{Detail, ErrorPage, Status, Way, error_page};
use ferrohealth_ui::view::shell::centred;
use ferrohealth_ui::view::status::{badge, pill};
use ferrohealth_ui::view::surface::{Crumb, Header, card, facts_row, grid_start};
use ferrohealth_ui::view::tabs::{Tab, TabRow};
use ferrohealth_ui::view::template::about::{
    AboutPage, Link, Manufacturer, SelfDescription, about_page,
};
use ferrohealth_ui::view::template::detail::{DetailPage, Stepper, detail_page, stepper};
use ferrohealth_ui::view::template::list::{
    ActiveFilters, FilterChip, HeadingCount, ListCard, ListPage, list_empty, list_page,
};
use ferrohealth_ui::view::template::settings::{
    How, Managed, Setting, SettingsPage, Source, ValueSource, revision_line, settings_page,
};
use ferrohealth_ui::widget::action_menu::{Act, Action, Item, Menu};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::data::{
    CONFIG_FILE, EXAMPLE_SERVER, FAULT_TIME, LIBRARY, LOCATION_ID, Location, OPERATOR_SCOPE,
    REQUEST_ID, SELF_DESCRIPTION, SUPPORT, SUPPORT_HREF, library_release, locations,
    many_locations,
};
use crate::locale::{Msg, tr, tr_with};

/// The variants the template pages show, as their routes name them.
pub const VARIANTS: [&str; 10] = [
    "list",
    "list-many",
    "list-empty",
    "list-tabs",
    "detail",
    "settings",
    "settings-managed",
    "about",
    "forbidden",
    "not-found",
];

/// The root of the brand's pages, such as `/ferrobridge`.
fn root() -> String {
    crate::app::route_brand().map_or_else(String::new, |brand| format!("/{}", brand.key()))
}

/// The path of the template `variant` in the brand in force.
fn at(variant: &str) -> String {
    format!("{}/templates/{variant}", root())
}

/// The intro every list template carries.
fn list_header() -> Header {
    Header {
        title: tr(Msg::Locations).into(),
        subtitle: Some(tr(Msg::TemplateExample).into()),
        ..Header::default()
    }
}

/// The list's one primary action.
fn new_location() -> ActionGroup {
    ActionGroup::primary(Action::new(
        Some(Glyph::PLUS),
        tr(Msg::NewLocation),
        Act::Link(at("list")),
    ))
}

/// The state cell of a location in `state`.
fn state_pill(state: &str) -> AnyView {
    match state {
        "warn" => pill(Tone::Warn, &tr(Msg::StateDegraded)).into_any(),
        "muted" => pill(Tone::Neutral, &tr(Msg::StateExcluded)).into_any(),
        _ => pill(Tone::Ok, &tr(Msg::StateReachable)).into_any(),
    }
}

/// The row of `location`, its name the link to its page.
fn location_row(location: &Location) -> impl IntoView + use<> {
    let class = (location.state == "muted").then_some("is-excluded");
    view! {
        <tr class=class>
            <th scope="row" data-role="title">
                <a class="fui-row-link" href=at("detail")>
                    {location.name.clone()}
                </a>
            </th>
            <td data-label=tr(Msg::ColumnId) data-role="wide">
                <code>{location.id.clone()}</code>
            </td>
            <td class="fui-num" data-label=tr(Msg::ColumnUnits) data-role="meta-q">
                {location.units}
            </td>
            <td data-label=tr(Msg::ColumnState) data-role="meta-2">{state_pill(location.state)}</td>
        </tr>
    }
}

/// The columns of the locations table.
fn columns() -> [Column; 4] {
    let column = |key: &'static str, msg: Msg, numeric: bool| Column {
        key: key.into(),
        label: tr(msg).into(),
        numeric,
        sortable: false,
        mono: false,
        title: None,
        role: Role::default(),
    };
    [
        column("location", Msg::ColumnLocation, false),
        column("id", Msg::ColumnId, false),
        column("units", Msg::ColumnUnits, true),
        column("state", Msg::ColumnState, false),
    ]
}

/// The table of `rows`.
fn table(rows: &[Location]) -> AnyView {
    let body = rows.iter().map(location_row).collect_view().into_any();
    data_table(DataTable {
        caption: &tr(Msg::Locations),
        columns: &columns(),
        sorting: Sorting::default(),
        body: Body::Rows(body),
    })
    .into_any()
}

/// The card of the short list, with its tabs when `tabs`.
fn short_card(tabs: bool) -> ListCard {
    let rows = locations();
    let tabs = tabs.then(|| TabRow {
        label: tr(Msg::Locations),
        tabs: vec![
            Tab::new("all", at("list-tabs"), tr(Msg::Locations))
                .count(5, tr_with(Msg::LocationsCount, &[("count", 5.into())])),
            Tab::new("units", at("list-tabs"), tr(Msg::AllUnits)),
        ],
        current: String::from("all"),
    });
    ListCard {
        title: tr(Msg::Locations),
        count: Some(HeadingCount::All(5)),
        order: None,
        tabs,
        actions: None,
        body: table(&rows),
        pager: None,
    }
}

/// T1 with five items: no summary strip, no toolbar.
fn list() -> AnyView {
    list_page(ListPage {
        header: list_header(),
        actions: new_location(),
        total: Some(5),
        summary: None,
        toolbar: None,
        filters: ActiveFilters::default(),
        card: short_card(false),
    })
}

/// T1 with five items under binder tabs.
fn list_tabs() -> AnyView {
    list_page(ListPage {
        header: list_header(),
        actions: new_location(),
        total: Some(5),
        summary: None,
        toolbar: None,
        filters: ActiveFilters::default(),
        card: short_card(true),
    })
}

/// T1 past ten items: the summary strip, the toolbar, a filter set and the
/// pager.
fn list_many() -> AnyView {
    let rows = many_locations();
    let window = Window::over(30, Paging { page: 1, size: 25 });
    let shown = window.slice(&rows).to_vec();
    let base = at("list-many");
    let summary = view! {
        <div class="fui-chip-checks fui-summary-chips" role="group" aria-label=tr(Msg::Show)>
            <a href=base.clone() class="fui-chip-check" aria-current="true">
                {tr_with(Msg::LocationsCount, &[("count", 30_u32.into())])}
            </a>
            <a href=base.clone() class="fui-chip-check">
                {tr_with(Msg::DegradedCount, &[("count", 1_u32.into())])}
            </a>
        </div>
    }
    .into_any();
    let toolbar = view! {
        <form class="fui-list-toolbar" method="get" action=base.clone()>
            <div class="fui-list-toolbar-start">
                <div class="fui-list-find">
                    <span class="fui-search-glyph">
                        {ferrohealth_ui::icon::icon(Glyph::SEARCH, "16")}
                    </span>
                    <input
                        class="fui-input"
                        type="search"
                        id="l-find"
                        name="q"
                        placeholder=tr(Msg::FindLocation)
                        aria-label=tr(Msg::FindLocation)
                    />
                </div>
                {filter_menu(FilterMenu {
                    id: "l-state",
                    name: "state",
                    glyph: Glyph::LIST_FILTER,
                    label: tr(Msg::State).into(),
                    title: tr(Msg::StatesShow),
                    choices: vec![
                        FilterChoice::new("reachable", tr(Msg::StateReachable), 28, true),
                        FilterChoice::new("degraded", tr(Msg::StateDegraded), 1, false),
                        FilterChoice::new("excluded", tr(Msg::StateExcluded), 1, false),
                    ],
                    note: None,
                    form: MenuForm::Enclosing,
                })}
            </div>
        </form>
    }
    .into_any();
    let page_href = {
        let base = base.clone();
        move |paging: Paging| format!("{base}?page={}&size={}", paging.page, paging.size)
    };
    list_page(ListPage {
        header: list_header(),
        actions: new_location(),
        total: Some(30),
        summary: Some(summary),
        toolbar: Some(toolbar),
        filters: ActiveFilters {
            chips: vec![FilterChip {
                label: tr(Msg::FilterReachable),
                remove: base.clone(),
            }],
            clear: base,
            shown: Some(28),
        },
        card: ListCard {
            title: tr(Msg::Locations),
            count: Some(HeadingCount::All(30)),
            order: None,
            tabs: None,
            actions: None,
            body: table(&shown),
            pager: Some(link_pager(
                Listing::new(tr(Msg::Locations)),
                window,
                &page_href,
            )),
        },
    })
}

/// T1 with no item: the primary stays, and one quiet line offers the same.
fn list_empty_page() -> AnyView {
    list_page(ListPage {
        header: list_header(),
        actions: new_location(),
        total: Some(0),
        summary: None,
        toolbar: None,
        filters: ActiveFilters::default(),
        card: ListCard {
            title: tr(Msg::Locations),
            count: Some(HeadingCount::All(0)),
            order: None,
            tabs: None,
            actions: None,
            body: list_empty(
                tr(Msg::NoLocation),
                Some((tr(Msg::AddFirstLocation), at("list"))),
            ),
            pager: None,
        },
    })
}

/// T2: the trail, the title with its state, one primary, binder tabs.
fn detail() -> AnyView {
    let tabs = TabRow {
        label: tr(Msg::LocatieNoord),
        tabs: [
            ("overview", Msg::Overview),
            ("units", Msg::Units),
            ("audit", Msg::Audit),
            ("settings", Msg::SettingsTab),
        ]
        .into_iter()
        .map(|(key, msg)| Tab::new(key, at("detail"), tr(msg)))
        .collect(),
        current: String::from("overview"),
    };
    let facts = facts_row(vec![
        (
            tr(Msg::ColumnId),
            view! { <code>{LOCATION_ID}</code> }.into_any(),
        ),
        (
            tr(Msg::Standing),
            pill(Tone::Ok, &tr(Msg::Active)).into_any(),
        ),
        (tr(Msg::ColumnUnits), "3".into_any()),
        (tr(Msg::Added), "2026-10-16".into_any()),
    ]);
    let steps = stepper(Stepper {
        label: tr(Msg::Locations),
        position: tr(Msg::LocationOf),
        what: tr(Msg::LocationWord),
        previous: None,
        next: Some(at("detail")),
    });
    let body = grid_start(view! {
        {card(tr(Msg::LocationWordTitle), facts)}
        {card(tr(Msg::QueueSteps), steps)}
    })
    .into_any();
    detail_page(DetailPage {
        header: Header {
            title: tr(Msg::LocatieNoord).into(),
            subtitle: Some(tr(Msg::LocationSubtitle).into()),
            crumbs: vec![Crumb::new(tr(Msg::Locations), at("list"))],
            badge: Some(pill(Tone::Ok, &tr(Msg::StateReachable)).into_any()),
            ..Header::default()
        },
        actions: location_actions(),
        stepper: None,
        tabs: Some(tabs),
        body: view! {
            {body}
            {delete_location()}
        }
        .into_any(),
    })
}

/// The detail page's title row (F14): Add unit, and More with two plain
/// items, one disabled with its reason, and the destructive delete last.
fn location_actions() -> ActionGroup {
    ActionGroup::primary(Action::new(
        Some(Glyph::PLUS),
        tr(Msg::AddUnit),
        Act::Link(at("detail")),
    ))
    .more(
        Menu::new(
            "location-more",
            tr(Msg::MoreActionsLocation),
            vec![
                Item::new(Action::new(
                    Some(Glyph::COPY),
                    tr(Msg::CopyLocationId),
                    Act::Link(at("detail")),
                )),
                Item::new(Action::new(
                    Some(Glyph::DOWNLOAD),
                    tr(Msg::ExportJson),
                    Act::Link(at("detail")),
                )),
                Item::new(Action::new(
                    Some(Glyph::LAYERS),
                    tr(Msg::MergeLocation),
                    Act::Link(at("detail")),
                ))
                .disabled(tr(Msg::MergeReason)),
            ],
        )
        .destructive(vec![Item::new(Action::new(
            Some(Glyph::TRASH),
            tr(Msg::DeleteLocation),
            Act::Dialog(String::from("delete-location")),
        ))]),
    )
}

/// The confirmation the detail page's delete opens; its confirm leads back
/// to the list, as a console's would post and redirect.
fn delete_location() -> AnyView {
    danger_dialog(
        Confirmation {
            id: "delete-location".into(),
            title: tr(Msg::DeleteTheLocation).into(),
            message: tr(Msg::DeleteLocationBody).into(),
        },
        link_button(
            at("list"),
            Variant::Destructive,
            Some(Glyph::TRASH),
            tr(Msg::Delete),
        ),
    )
    .into_any()
}

/// The settings both T3 pages show, Mode at the next start; read from a
/// file (`file`), Timeout comes from the environment, Retries from the
/// command line and Best effort is the default.
fn settings(file: bool) -> Vec<Setting> {
    let row = |label: Msg,
               description: Msg,
               value: String,
               key: &str,
               next_start: bool,
               source: Option<ValueSource>| Setting {
        label: tr(label),
        description: tr(description),
        value: value.into_any(),
        keys: vec![key.to_owned()],
        next_start,
        source: source.filter(|_| file),
    };
    vec![
        row(
            Msg::Timeout,
            Msg::TimeoutDesc,
            tr(Msg::TimeoutValue),
            "limits.timeout_ms",
            false,
            Some(ValueSource::Environment),
        ),
        row(
            Msg::Retries,
            Msg::RetriesDesc,
            String::from("2"),
            "limits.retries",
            false,
            Some(ValueSource::CommandLine),
        ),
        row(
            Msg::ModeSetting,
            Msg::ModeDesc,
            tr(Msg::Nearest),
            "limits.mode",
            true,
            None,
        ),
        row(
            Msg::BestEffort,
            Msg::BestEffortDesc,
            tr(Msg::Offered),
            "limits.best_effort",
            false,
            Some(ValueSource::Default),
        ),
    ]
}

/// The binder tabs of the settings page `variant`, General holding a
/// draft when `draft`.
fn settings_tabs(variant: &str, draft: bool) -> TabRow {
    TabRow {
        label: tr(Msg::SettingsSections),
        tabs: [
            ("general", Msg::General),
            ("limits", Msg::Limits),
            ("access", Msg::AccessLog),
            ("history", Msg::History),
        ]
        .into_iter()
        .map(|(key, msg)| {
            let tab = Tab::new(key, at(variant), tr(msg));
            if draft && key == "general" {
                tab.draft()
            } else {
                tab
            }
        })
        .collect(),
        current: String::from("general"),
    }
}

/// The header of the settings pages.
fn settings_header() -> Header {
    Header {
        title: tr(Msg::SettingsTab).into(),
        subtitle: Some(tr(Msg::SettingsIntro).into()),
        ..Header::default()
    }
}

/// T3 read from a file: one quiet line, read-only, no badge, whose How
/// configuration works opens the informational dialog.
fn settings_file() -> AnyView {
    let page = settings_page(SettingsPage {
        header: settings_header(),
        actions: ActionGroup::none(),
        source: Source::File {
            file: String::from(CONFIG_FILE),
            how: How::Dialog("how-configuration".into()),
        },
        tabs: Some(settings_tabs("settings", false)),
        settings: settings(true),
        replica: None,
    });
    let how = info_dialog(
        Explanation {
            id: "how-configuration".into(),
            title: tr(Msg::HowConfigurationWorks).into(),
        },
        view! {
            <p>{tr(Msg::HowConfigReads)}</p>
            <p class="fui-small fui-muted">{tr(Msg::HowConfigChange)}</p>
        },
    );
    view! {
        {page}
        {how}
    }
    .into_any()
}

/// T3 managed: the mode badge, the revision line, the intro and Edit.
fn settings_managed() -> AnyView {
    settings_page(SettingsPage {
        header: settings_header(),
        actions: ActionGroup::more_only(Menu::new(
            "settings-more",
            tr(Msg::MoreActionsSettings),
            vec![Item::new(Action::new(
                Some(Glyph::INFO),
                tr(Msg::HowSettingsWork),
                Act::Link(at("settings-managed")),
            ))],
        )),
        source: Source::Managed(Box::new(Managed {
            badge: badge(Tone::Neutral, &tr(Msg::ManagedBadge)).into_any(),
            revision: Some(revision_line(tr(Msg::RevisionLine).into_any())),
            intro: tr(Msg::SettingsPanelIntro),
            actions: Some(
                view! {
                    <a href=at("settings-managed") class=Variant::Secondary.small()>
                        {label(Some(Glyph::PENCIL), tr(Msg::EditWord))}
                    </a>
                }
                .into_any(),
            ),
        })),
        tabs: Some(settings_tabs("settings-managed", true)),
        settings: settings(false),
        replica: None,
    })
}

/// The About page.
fn about() -> AnyView {
    about_page(AboutPage {
        header: Header {
            title: tr(Msg::AboutTitle).into(),
            subtitle: Some(tr(Msg::AboutIntro).into()),
            ..Header::default()
        },
        release: crate::app::footer().release(),
        versions: vec![
            (tr(Msg::Server), EXAMPLE_SERVER.into_any()),
            (
                String::from(LIBRARY),
                library_release(crate::app::VERSION).into_any(),
            ),
        ],
        manufacturer: Manufacturer {
            licence: tr(Msg::LicenceName),
            support: Link {
                label: String::from(SUPPORT),
                href: String::from(SUPPORT_HREF),
            },
            documentation: Some(Link {
                label: tr(Msg::TheBook),
                href: String::from("https://ferrobridge.eu/docs/"),
            }),
        },
        cards: Vec::new(),
        self_description: Some(SelfDescription {
            title: tr(Msg::SelfDescription),
            line: tr(Msg::SelfDescriptionLine),
            language: Language::Json,
            text: String::from(SELF_DESCRIPTION),
            copy_id: String::from("about-copy"),
            download: at("about"),
        }),
        replica: None,
    })
}

/// The overview, one way on from an error.
fn to_overview() -> Way {
    Way {
        label: tr(Msg::Overview),
        href: root(),
        glyph: None,
    }
}

/// The 403 page.
fn forbidden() -> AnyView {
    let brand = crate::app::route_brand().map_or("ferrobridge", |brand| brand.key());
    error_page(ErrorPage {
        status: Status::Forbidden,
        title: tr(Msg::ForbiddenTitle),
        body: tr(Msg::ForbiddenBody).into_any(),
        hint: Some(
            view! {
                {tr(Msg::ScopeIs)}
                " "
                <code>{OPERATOR_SCOPE}</code>
            }
            .into_any(),
        ),
        ways: vec![
            Way {
                label: tr(Msg::SignInAgain),
                href: format!("/sign-in/{brand}/basic"),
                glyph: Some(Glyph::LOG_IN),
            },
            to_overview(),
        ],
        detail: None,
    })
}

/// The 404 page.
fn not_found() -> AnyView {
    error_page(ErrorPage {
        status: Status::NotFound,
        title: tr(Msg::NotFoundTitle),
        body: tr(Msg::NotFoundTemplateBody).into_any(),
        hint: None,
        ways: vec![
            Way {
                label: tr(Msg::Locations),
                href: at("list"),
                glyph: Some(Glyph::LAYERS),
            },
            to_overview(),
        ],
        detail: None,
    })
}

/// A template page, the one the route's `:variant` names.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn TemplatePage() -> impl IntoView {
    let params = use_params_map();
    let variant = params.with_untracked(|params| params.get("variant").unwrap_or_default());
    let (title, page) = match variant.as_str() {
        "list" => (tr(Msg::Locations), list()),
        "list-many" => (tr(Msg::Locations), list_many()),
        "list-empty" => (tr(Msg::Locations), list_empty_page()),
        "list-tabs" => (tr(Msg::Locations), list_tabs()),
        "detail" => (tr(Msg::LocatieNoord), detail()),
        "settings" => (tr(Msg::SettingsTab), settings_file()),
        "settings-managed" => (tr(Msg::SettingsTab), settings_managed()),
        "about" => (tr(Msg::AboutTitle), about()),
        "forbidden" => (tr(Msg::ForbiddenTitle), forbidden()),
        _ => (tr(Msg::NotFoundTitle), not_found()),
    };
    view! {
        {super::title(title)}
        {page}
    }
}

/// The 500 page, in the centred layout a signed-out visitor sees.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn FaultPage() -> impl IntoView {
    let params = use_params_map();
    let brand = params.with_untracked(|params| params.get("brand").unwrap_or_default());
    let page = error_page(ErrorPage {
        status: Status::Fault,
        title: tr(Msg::FaultTitle),
        body: tr(Msg::FaultBody).into_any(),
        hint: None,
        ways: vec![Way {
            label: tr(Msg::TryAgain),
            href: format!("/fault/{brand}"),
            glyph: Some(Glyph::REFRESH_CW),
        }],
        detail: Some(Detail {
            time: String::from(FAULT_TIME),
            request_id: String::from(REQUEST_ID),
        }),
    });
    let theme = ferrohealth_ui::style::Brand::parse(&brand)
        .map(|brand| view! { <leptos_meta::Stylesheet id="brand-theme" href=crate::app::theme_path(brand) /> });
    view! {
        {super::title(tr(Msg::FaultTitle))}
        {theme}
        {centred(page, crate::app::footer())}
    }
}
