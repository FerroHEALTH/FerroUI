// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Your preferences, as a console draws it.
//!
//! The library's three tabs read the gallery's cookies, with FerroPIX's
//! reveal settings in Appearance as a console's own rows, and in Language
//! and region FerroTERM's typed concept language and page size, then the
//! dates and the time zone.
//!
//! The page is server HTML; each preference posts to the gallery's
//! preference route, which keeps it in its cookie and returns with
//! `?saved=` naming it, so the row says Saved.

use ferrohealth_ui::calendar::Weekday;
use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::{Arg, Locale};
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::view::preferences::{
    Post, PreferenceRow, Section, SectionRoutes, SelectPreference, SwitchPreference, Typed,
    TypedPreference, preference_row, preferences_page, select_preference, switch_preference,
    typed_preference,
};

use crate::app::{PREFERENCE_ROUTE, Page, RESET_ROUTE, own_of};
use crate::locale::Msg;
use crate::visit::Visit;

/// The name of the brand's accent, which its theme cards name.
fn accent(v: &Visit, brand: Brand) -> String {
    v.tr(match brand {
        Brand::Bridge => Msg::AccentIndigo,
        Brand::Fed => Msg::AccentAzure,
        Brand::Ehr | Brand::Term => Msg::AccentTeal,
        Brand::Pix => Msg::AccentPlum,
    })
}

/// FerroPIX's own rows in Appearance: how long a revealed identifier
/// stays, and whether leaving the page hides it.
fn reveal_rows(v: &Visit, cookies: &str, post: &Post, saved: Option<&str>) -> Vec<Html> {
    let reveal_saved = saved == Some("reveal");
    let hide_saved = saved == Some("hide");
    let reveal = select_preference(
        &v.locale,
        SelectPreference {
            name: String::from("reveal"),
            choices: ["10", "15", "30"]
                .into_iter()
                .map(|seconds| {
                    let count = seconds.parse::<f64>().unwrap_or_default();
                    (
                        String::from(seconds),
                        v.tr_with(Msg::RevealSeconds, &[("count", Arg::Number(count))])
                            .into(),
                    )
                })
                .collect(),
            current: String::from(own_of(cookies, "reveal").unwrap_or("15")),
            post: post.clone(),
            saved: reveal_saved,
        },
    );
    let hide = switch_preference(
        &v.locale,
        SwitchPreference {
            name: String::from("hide"),
            label: v.tr(Msg::HideOnLeaveSwitch),
            on: own_of(cookies, "hide") != Some("off"),
            post: post.clone(),
            saved: hide_saved,
        },
    );
    vec![
        preference_row(
            &v.locale,
            PreferenceRow {
                label: v.tr(Msg::RevealFor).into(),
                description: v.tr(Msg::RevealForDesc).into(),
                control_id: Some(String::from("reveal")),
                control: reveal,
                saved: reveal_saved,
            },
        ),
        preference_row(
            &v.locale,
            PreferenceRow {
                label: v.tr(Msg::HideOnLeave).into(),
                description: v.tr(Msg::HideOnLeaveDesc).into(),
                control_id: Some(String::from("hide")),
                control: hide,
                saved: hide_saved,
            },
        ),
    ]
}

/// FerroTERM's own rows in Language and region, drawn for every brand:
/// the language concepts are asked for in, a text, and the rows a listing
/// shows, a number.
fn typed_rows(v: &Visit, cookies: &str, post: &Post, saved: Option<&str>) -> Vec<Html> {
    let row = |name: &str, label: Msg, description: Msg, kind: Typed, value: &str, hint| {
        let saved = saved == Some(name);
        let control = typed_preference(
            &v.locale,
            TypedPreference {
                name: String::from(name),
                kind,
                value: String::from(value),
                placeholder: hint,
                post: post.clone(),
                saved,
            },
        );
        preference_row(
            &v.locale,
            PreferenceRow {
                label: v.tr(label).into(),
                description: v.tr(description).into(),
                control_id: Some(String::from(name)),
                control,
                saved,
            },
        )
    };
    vec![
        row(
            "concepts",
            Msg::ConceptLanguage,
            Msg::ConceptLanguageDesc,
            Typed::Text {
                min_length: Some(2),
                max_length: Some(35),
            },
            own_of(cookies, "concepts").unwrap_or_default(),
            Some(v.tr(Msg::ConceptLanguageHint)),
        ),
        row(
            "size",
            Msg::RowsPerPage,
            Msg::RowsPerPageDesc,
            Typed::Number {
                min: Some(10),
                max: Some(200),
                step: None,
            },
            own_of(cookies, "size").unwrap_or("25"),
            None,
        ),
    ]
}

/// The read-only facts of Language and region: how dates are written and
/// the server's time zone.
fn region_rows(v: &Visit) -> Vec<Html> {
    [
        (Msg::Dates, Msg::DatesDesc, Msg::DatesExample),
        (Msg::TimeZone, Msg::TimeZoneDesc, Msg::TimeZoneValue),
    ]
    .into_iter()
    .map(|(label, description, value)| {
        preference_row(
            &v.locale,
            PreferenceRow {
                label: v.tr(label).into(),
                description: v.tr(description).into(),
                control_id: None,
                control: Html::raw(format!(
                    "<span class=\"fui-setting-value\">{}</span>",
                    Html::text(v.tr(value))
                )),
                saved: false,
            },
        )
    })
    .collect()
}

/// Your preferences, on the tab the path names.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let Some(section) = v
        .segment(2)
        .map_or(Some(Section::Appearance), Section::parse)
    else {
        return crate::app::not_found(v);
    };
    let brand = v.brand;
    let root = format!("/{}/preferences", brand.key());
    let routes = SectionRoutes {
        appearance: root.clone(),
        language: format!("{root}/language"),
        keyboard: format!("{root}/keyboard"),
    };
    let saved = v.param("saved").map(str::to_owned);
    let post = Post {
        action: String::from(PREFERENCE_ROUTE),
        back: routes.of(section).to_owned(),
    };
    let own = match (section, brand) {
        (Section::Appearance, Brand::Pix) => reveal_rows(v, &v.cookies, &post, saved.as_deref()),
        (Section::Language, _) => {
            let mut rows = typed_rows(v, &v.cookies, &post, saved.as_deref());
            rows.extend(region_rows(v));
            rows
        }
        _ => Vec::new(),
    };
    let page = preferences_page(
        &v.locale,
        ferrohealth_ui::view::preferences::PreferencesPage {
            section,
            routes,
            reading: brand == Brand::Ehr,
            preferences: v.preferences.clone(),
            action: String::from(PREFERENCE_ROUTE),
            saved,
            accent: accent(v, brand),
            languages: crate::locale::choices(),
            browser: Locale::english(),
            week_from_language: Weekday::Monday,
            shortcuts: Some(format!("/{}/overlays", brand.key())),
            reset: String::from(RESET_ROUTE),
            own,
        },
    );
    Page::shell(v.tr(Msg::YourPreferences), page)
}
