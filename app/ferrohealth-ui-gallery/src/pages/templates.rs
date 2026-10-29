// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The page templates with the reference content of `design/templates/`.
//!
//! They are T1 short, long, empty and under tabs, T2, T3 from a file and managed,
//! the About page, and the error page as a 403 and a 404 in the shell and a
//! 500 in the centred layout. Each is a page of its own, so the screenshots
//! show it whole.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::filter::{FilterChoice, FilterMenu, MenuForm, filter_menu};
use ferrohealth_ui::table::model::{Paging, Window};
use ferrohealth_ui::table::pager::{Listing, link_pager};
use ferrohealth_ui::table::view::{Body, Column, DataTable, Role, Sorting, data_table};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::action_menu::{Act, Action, Item, Menu};
use ferrohealth_ui::view::button::{Variant, label, link_button};
use ferrohealth_ui::view::dialog::{Confirmation, Explanation, danger_dialog, info_dialog};
use ferrohealth_ui::view::error_page::{Detail, ErrorPage, Status, Way, error_page};
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

use crate::app::Page;
use crate::data::{
    CONFIG_FILE, EXAMPLE_SERVER, FAULT_TIME, LIBRARY, LOCATION_ID, Location, OPERATOR_SCOPE,
    REQUEST_ID, SELF_DESCRIPTION, SUPPORT, SUPPORT_HREF, library_release, locations,
    many_locations,
};
use crate::locale::Msg;
use crate::visit::Visit;

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
fn root(v: &Visit) -> String {
    format!("/{}", v.brand.key())
}

/// The path of the template `variant` in the brand in force.
fn at(v: &Visit, variant: &str) -> String {
    format!("{}/templates/{variant}", root(v))
}

/// The intro every list template carries.
fn list_header(v: &Visit) -> Header {
    Header {
        title: v.tr(Msg::Locations).into(),
        subtitle: Some(v.tr(Msg::TemplateExample).into()),
        ..Header::default()
    }
}

/// The list's one primary action.
fn new_location(v: &Visit) -> ActionGroup {
    ActionGroup::primary(Action::new(
        Some(Glyph::PLUS),
        v.tr(Msg::NewLocation),
        Act::Link(at(v, "list")),
    ))
}

/// The state cell of a location in `state`.
fn state_pill(v: &Visit, state: &str) -> Html {
    match state {
        "warn" => pill(Tone::Warn, &v.tr(Msg::StateDegraded)),
        "muted" => pill(Tone::Neutral, &v.tr(Msg::StateExcluded)),
        _ => pill(Tone::Ok, &v.tr(Msg::StateReachable)),
    }
}

/// The row of `location`, its name the link to its page.
fn location_row(v: &Visit, location: &Location) -> Html {
    let class = if location.state == "muted" {
        " class=\"is-excluded\""
    } else {
        ""
    };
    Html::raw(format!(
        "<tr{class}><th scope=\"row\" data-role=\"title\"><a class=\"fui-row-link\" href=\"{}\">{}</a></th><td data-label=\"{}\" data-role=\"wide\"><code>{}</code></td><td class=\"fui-num\" data-label=\"{}\" data-role=\"meta-q\">{}</td><td data-label=\"{}\" data-role=\"meta-2\">{}</td></tr>",
        Html::text(at(v, "detail")),
        Html::text(&location.name),
        Html::text(v.tr(Msg::ColumnId)),
        Html::text(&location.id),
        Html::text(v.tr(Msg::ColumnUnits)),
        location.units,
        Html::text(v.tr(Msg::ColumnState)),
        state_pill(v, location.state)
    ))
}

/// The columns of the locations table.
fn columns(v: &Visit) -> [Column; 4] {
    let column = |key: &'static str, msg: Msg, numeric: bool| Column {
        key: key.into(),
        label: v.tr(msg).into(),
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
fn table(v: &Visit, rows: &[Location]) -> Html {
    let body = rows.iter().map(|row| location_row(v, row)).collect();
    data_table(DataTable {
        caption: &v.tr(Msg::Locations),
        columns: &columns(v),
        sorting: Sorting::default(),
        body: Body::Rows(body),
    })
}

/// The card of the short list, with its tabs when `tabs`.
fn short_card(v: &Visit, tabs: bool) -> ListCard {
    let rows = locations();
    let tabs = tabs.then(|| TabRow {
        label: v.tr(Msg::Locations),
        tabs: vec![
            Tab::new("all", at(v, "list-tabs"), v.tr(Msg::Locations))
                .count(5, v.tr_with(Msg::LocationsCount, &[("count", 5.into())])),
            Tab::new("units", at(v, "list-tabs"), v.tr(Msg::AllUnits)),
        ],
        current: String::from("all"),
    });
    ListCard {
        title: v.tr(Msg::Locations),
        count: Some(HeadingCount::All(5)),
        order: None,
        tabs,
        actions: None,
        body: table(v, &rows),
        pager: None,
    }
}

/// T1 with five items: no summary strip, no toolbar.
fn list(v: &Visit) -> Html {
    list_page(
        &v.locale,
        ListPage {
            header: list_header(v),
            actions: new_location(v),
            total: Some(5),
            summary: None,
            toolbar: None,
            filters: ActiveFilters::default(),
            card: short_card(v, false),
        },
    )
}

/// T1 with five items under binder tabs.
fn list_tabs(v: &Visit) -> Html {
    list_page(
        &v.locale,
        ListPage {
            header: list_header(v),
            actions: new_location(v),
            total: Some(5),
            summary: None,
            toolbar: None,
            filters: ActiveFilters::default(),
            card: short_card(v, true),
        },
    )
}

/// T1 past ten items: the summary strip, the toolbar, a filter set and the
/// pager.
fn list_many(v: &Visit) -> Html {
    let rows = many_locations();
    let window = Window::over(30, Paging { page: 1, size: 25 });
    let shown = window.slice(&rows).to_vec();
    let base = at(v, "list-many");
    let summary = Html::raw(format!(
        "<div class=\"fui-chip-checks fui-summary-chips\" role=\"group\" aria-label=\"{}\"><a href=\"{base}\" class=\"fui-chip-check\" aria-current=\"true\">{}</a><a href=\"{base}\" class=\"fui-chip-check\">{}</a></div>",
        Html::text(v.tr(Msg::Show)),
        Html::text(v.tr_with(Msg::LocationsCount, &[("count", 30_u32.into())])),
        Html::text(v.tr_with(Msg::DegradedCount, &[("count", 1_u32.into())])),
    ));
    let menu = filter_menu(
        &v.locale,
        FilterMenu {
            id: "l-state",
            name: "state",
            glyph: Glyph::LIST_FILTER,
            label: v.tr(Msg::State).into(),
            title: v.tr(Msg::StatesShow),
            choices: vec![
                FilterChoice::new("reachable", v.tr(Msg::StateReachable), 28, true),
                FilterChoice::new("degraded", v.tr(Msg::StateDegraded), 1, false),
                FilterChoice::new("excluded", v.tr(Msg::StateExcluded), 1, false),
            ],
            note: None,
            form: MenuForm::Enclosing,
        },
    );
    let find = Html::text(v.tr(Msg::FindLocation));
    let toolbar = Html::raw(format!(
        "<form class=\"fui-list-toolbar\" method=\"get\" action=\"{base}\"><div class=\"fui-list-toolbar-start\"><div class=\"fui-list-find\"><span class=\"fui-search-glyph\">{}</span><input class=\"fui-input\" type=\"search\" id=\"l-find\" name=\"q\" placeholder=\"{find}\" aria-label=\"{find}\"></div>{menu}</div></form>",
        ferrohealth_ui::icon::icon(Glyph::SEARCH, "16"),
    ));
    let page_href = {
        let base = base.clone();
        move |paging: Paging| format!("{base}?page={}&size={}", paging.page, paging.size)
    };
    list_page(
        &v.locale,
        ListPage {
            header: list_header(v),
            actions: new_location(v),
            total: Some(30),
            summary: Some(summary),
            toolbar: Some(toolbar),
            filters: ActiveFilters {
                chips: vec![FilterChip {
                    label: v.tr(Msg::FilterReachable),
                    remove: base.clone(),
                }],
                clear: base,
                shown: Some(28),
            },
            card: ListCard {
                title: v.tr(Msg::Locations),
                count: Some(HeadingCount::All(30)),
                order: None,
                tabs: None,
                actions: None,
                body: table(v, &shown),
                pager: Some(link_pager(
                    &v.locale,
                    Listing::new(v.tr(Msg::Locations)),
                    window,
                    &page_href,
                )),
            },
        },
    )
}

/// T1 with no item: the primary stays, and one quiet line offers the same.
fn list_empty_page(v: &Visit) -> Html {
    list_page(
        &v.locale,
        ListPage {
            header: list_header(v),
            actions: new_location(v),
            total: Some(0),
            summary: None,
            toolbar: None,
            filters: ActiveFilters::default(),
            card: ListCard {
                title: v.tr(Msg::Locations),
                count: Some(HeadingCount::All(0)),
                order: None,
                tabs: None,
                actions: None,
                body: list_empty(
                    v.tr(Msg::NoLocation),
                    Some((v.tr(Msg::AddFirstLocation), at(v, "list"))),
                ),
                pager: None,
            },
        },
    )
}

/// T2: the trail, the title with its state, one primary, binder tabs.
fn detail(v: &Visit) -> Html {
    let tabs = TabRow {
        label: v.tr(Msg::LocatieNoord),
        tabs: [
            ("overview", Msg::Overview),
            ("units", Msg::Units),
            ("audit", Msg::Audit),
            ("settings", Msg::SettingsTab),
        ]
        .into_iter()
        .map(|(key, msg)| Tab::new(key, at(v, "detail"), v.tr(msg)))
        .collect(),
        current: String::from("overview"),
    };
    let facts = facts_row(vec![
        (
            v.tr(Msg::ColumnId),
            Html::raw(format!("<code>{LOCATION_ID}</code>")),
        ),
        (v.tr(Msg::Standing), pill(Tone::Ok, &v.tr(Msg::Active))),
        (v.tr(Msg::ColumnUnits), Html::text("3")),
        (v.tr(Msg::Added), Html::text("2026-10-16")),
    ]);
    let steps = stepper(
        &v.locale,
        Stepper {
            label: v.tr(Msg::Locations),
            position: v.tr(Msg::LocationOf),
            what: v.tr(Msg::LocationWord),
            previous: None,
            next: Some(at(v, "detail")),
        },
    );
    let body = grid_start(Html::join([
        card(v.tr(Msg::LocationWordTitle), facts),
        card(v.tr(Msg::QueueSteps), steps),
    ]));
    detail_page(
        &v.locale,
        DetailPage {
            header: Header {
                title: v.tr(Msg::LocatieNoord).into(),
                subtitle: Some(v.tr(Msg::LocationSubtitle).into()),
                crumbs: vec![Crumb::new(v.tr(Msg::Locations), at(v, "list"))],
                badge: Some(pill(Tone::Ok, &v.tr(Msg::StateReachable))),
                ..Header::default()
            },
            actions: location_actions(v),
            stepper: None,
            tabs: Some(tabs),
            body: Html::join([body, delete_location(v)]),
        },
    )
}

/// The detail page's title row (F14): Add unit, and More with two plain
/// items, one disabled with its reason, and the destructive delete last.
fn location_actions(v: &Visit) -> ActionGroup {
    ActionGroup::primary(Action::new(
        Some(Glyph::PLUS),
        v.tr(Msg::AddUnit),
        Act::Link(at(v, "detail")),
    ))
    .more(
        Menu::new(
            "location-more",
            v.tr(Msg::MoreActionsLocation),
            vec![
                Item::new(Action::new(
                    Some(Glyph::COPY),
                    v.tr(Msg::CopyLocationId),
                    Act::Link(at(v, "detail")),
                )),
                Item::new(Action::new(
                    Some(Glyph::DOWNLOAD),
                    v.tr(Msg::ExportJson),
                    Act::Link(at(v, "detail")),
                )),
                Item::new(Action::new(
                    Some(Glyph::LAYERS),
                    v.tr(Msg::MergeLocation),
                    Act::Link(at(v, "detail")),
                ))
                .disabled(v.tr(Msg::MergeReason)),
            ],
        )
        .destructive(vec![Item::new(Action::new(
            Some(Glyph::TRASH),
            v.tr(Msg::DeleteLocation),
            Act::Dialog(String::from("delete-location")),
        ))]),
    )
}

/// The confirmation the detail page's delete opens; its confirm leads back
/// to the list, as a console's would post and redirect.
fn delete_location(v: &Visit) -> Html {
    danger_dialog(
        &v.locale,
        Confirmation {
            id: "delete-location".into(),
            title: v.tr(Msg::DeleteTheLocation).into(),
            message: v.tr(Msg::DeleteLocationBody).into(),
        },
        link_button(
            at(v, "list"),
            Variant::Destructive,
            Some(Glyph::TRASH),
            v.tr(Msg::Delete),
        ),
    )
}

/// The settings both T3 pages show, Mode at the next start; read from a
/// file (`file`), Timeout comes from the environment, Retries from the
/// command line and Best effort is the default.
fn settings(v: &Visit, file: bool) -> Vec<Setting> {
    let row = |label: Msg,
               description: Msg,
               value: String,
               key: &str,
               next_start: bool,
               source: Option<ValueSource>| Setting {
        label: v.tr(label),
        description: v.tr(description),
        value: Html::text(value),
        keys: vec![key.to_owned()],
        next_start,
        source: source.filter(|_| file),
    };
    vec![
        row(
            Msg::Timeout,
            Msg::TimeoutDesc,
            v.tr(Msg::TimeoutValue),
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
            v.tr(Msg::Nearest),
            "limits.mode",
            true,
            None,
        ),
        row(
            Msg::BestEffort,
            Msg::BestEffortDesc,
            v.tr(Msg::Offered),
            "limits.best_effort",
            false,
            Some(ValueSource::Default),
        ),
    ]
}

/// The binder tabs of the settings page `variant`, General holding a
/// draft when `draft`.
fn settings_tabs(v: &Visit, variant: &str, draft: bool) -> TabRow {
    TabRow {
        label: v.tr(Msg::SettingsSections),
        tabs: [
            ("general", Msg::General),
            ("limits", Msg::Limits),
            ("access", Msg::AccessLog),
            ("history", Msg::History),
        ]
        .into_iter()
        .map(|(key, msg)| {
            let tab = Tab::new(key, at(v, variant), v.tr(msg));
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
fn settings_header(v: &Visit) -> Header {
    Header {
        title: v.tr(Msg::SettingsTab).into(),
        subtitle: Some(v.tr(Msg::SettingsIntro).into()),
        ..Header::default()
    }
}

/// T3 read from a file: one quiet line, read-only, no badge, whose How
/// configuration works opens the informational dialog.
fn settings_file(v: &Visit) -> Html {
    let page = settings_page(
        &v.locale,
        SettingsPage {
            header: settings_header(v),
            actions: ActionGroup::none(),
            source: Source::File {
                file: String::from(CONFIG_FILE),
                how: How::Dialog("how-configuration".into()),
            },
            tabs: Some(settings_tabs(v, "settings", false)),
            settings: settings(v, true),
            replica: None,
        },
    );
    let how = info_dialog(
        &v.locale,
        Explanation {
            id: "how-configuration".into(),
            title: v.tr(Msg::HowConfigurationWorks).into(),
        },
        Html::raw(format!(
            "<p>{}</p><p class=\"fui-small fui-muted\">{}</p>",
            Html::text(v.tr(Msg::HowConfigReads)),
            Html::text(v.tr(Msg::HowConfigChange))
        )),
    );
    Html::join([page, how])
}

/// T3 managed: the mode badge, the revision line, the intro and Edit.
fn settings_managed(v: &Visit) -> Html {
    settings_page(
        &v.locale,
        SettingsPage {
            header: settings_header(v),
            actions: ActionGroup::more_only(Menu::new(
                "settings-more",
                v.tr(Msg::MoreActionsSettings),
                vec![Item::new(Action::new(
                    Some(Glyph::INFO),
                    v.tr(Msg::HowSettingsWork),
                    Act::Link(at(v, "settings-managed")),
                ))],
            )),
            source: Source::Managed(Box::new(Managed {
                badge: badge(Tone::Neutral, v.tr(Msg::ManagedBadge)),
                revision: Some(revision_line(Html::text(v.tr(Msg::RevisionLine)))),
                intro: v.tr(Msg::SettingsPanelIntro),
                actions: Some(Html::raw(format!(
                    "<a href=\"{}\" class=\"{}\">{}</a>",
                    Html::text(at(v, "settings-managed")),
                    Variant::Secondary.small(),
                    label(Some(Glyph::PENCIL), v.tr(Msg::EditWord))
                ))),
            })),
            tabs: Some(settings_tabs(v, "settings-managed", true)),
            settings: settings(v, false),
            replica: None,
        },
    )
}

/// The About page.
fn about(v: &Visit) -> Html {
    about_page(
        &v.locale,
        AboutPage {
            header: Header {
                title: v.tr(Msg::AboutTitle).into(),
                subtitle: Some(v.tr(Msg::AboutIntro).into()),
                ..Header::default()
            },
            release: crate::app::footer(v).release(),
            versions: vec![
                (v.tr(Msg::Server), Html::text(EXAMPLE_SERVER)),
                (
                    String::from(LIBRARY),
                    Html::text(library_release(crate::app::VERSION)),
                ),
            ],
            manufacturer: Manufacturer {
                licence: v.tr(Msg::LicenceName),
                support: Link {
                    label: String::from(SUPPORT),
                    href: String::from(SUPPORT_HREF),
                },
                documentation: Some(Link {
                    label: v.tr(Msg::TheBook),
                    href: String::from("https://ferrobridge.eu/docs/"),
                }),
            },
            cards: Vec::new(),
            self_description: Some(SelfDescription {
                title: v.tr(Msg::SelfDescription),
                line: v.tr(Msg::SelfDescriptionLine),
                language: Language::Json,
                text: String::from(SELF_DESCRIPTION),
                copy_id: String::from("about-copy"),
                download: at(v, "about"),
            }),
            replica: None,
        },
    )
}

/// The overview, one way on from an error.
fn to_overview(v: &Visit) -> Way {
    Way {
        label: v.tr(Msg::Overview),
        href: root(v),
        glyph: None,
    }
}

/// The 403 page.
fn forbidden(v: &Visit) -> Html {
    let brand = v.brand.key();
    error_page(
        &v.locale,
        ErrorPage {
            status: Status::Forbidden,
            title: v.tr(Msg::ForbiddenTitle),
            body: Html::text(v.tr(Msg::ForbiddenBody)),
            hint: Some(Html::raw(format!(
                "{} <code>{OPERATOR_SCOPE}</code>",
                Html::text(v.tr(Msg::ScopeIs))
            ))),
            ways: vec![
                Way {
                    label: v.tr(Msg::SignInAgain),
                    href: format!("/sign-in/{brand}/basic"),
                    glyph: Some(Glyph::LOG_IN),
                },
                to_overview(v),
            ],
            detail: None,
        },
    )
}

/// The 404 page.
fn not_found(v: &Visit) -> Html {
    error_page(
        &v.locale,
        ErrorPage {
            status: Status::NotFound,
            title: v.tr(Msg::NotFoundTitle),
            body: Html::text(v.tr(Msg::NotFoundTemplateBody)),
            hint: None,
            ways: vec![
                Way {
                    label: v.tr(Msg::Locations),
                    href: at(v, "list"),
                    glyph: Some(Glyph::LAYERS),
                },
                to_overview(v),
            ],
            detail: None,
        },
    )
}

/// A template page, the one the path's variant names.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let variant = v.segment(2).unwrap_or_default();
    let (title, page) = match variant {
        "list" => (v.tr(Msg::Locations), list(v)),
        "list-many" => (v.tr(Msg::Locations), list_many(v)),
        "list-empty" => (v.tr(Msg::Locations), list_empty_page(v)),
        "list-tabs" => (v.tr(Msg::Locations), list_tabs(v)),
        "detail" => (v.tr(Msg::LocatieNoord), detail(v)),
        "settings" => (v.tr(Msg::SettingsTab), settings_file(v)),
        "settings-managed" => (v.tr(Msg::SettingsTab), settings_managed(v)),
        "about" => (v.tr(Msg::AboutTitle), about(v)),
        "forbidden" => (v.tr(Msg::ForbiddenTitle), forbidden(v)),
        _ => (v.tr(Msg::NotFoundTitle), not_found(v)),
    };
    Page::shell(title, page)
}

/// The 500 page, in the centred layout a signed-out visitor sees.
#[must_use]
pub fn fault(v: &Visit) -> Page {
    let brand = v.segment(1).unwrap_or_default();
    let page = error_page(
        &v.locale,
        ErrorPage {
            status: Status::Fault,
            title: v.tr(Msg::FaultTitle),
            body: Html::text(v.tr(Msg::FaultBody)),
            hint: None,
            ways: vec![Way {
                label: v.tr(Msg::TryAgain),
                href: format!("/fault/{brand}"),
                glyph: Some(Glyph::REFRESH_CW),
            }],
            detail: Some(Detail {
                time: String::from(FAULT_TIME),
                request_id: String::from(REQUEST_ID),
            }),
        },
    );
    Page::centred(v.tr(Msg::FaultTitle), page)
}
