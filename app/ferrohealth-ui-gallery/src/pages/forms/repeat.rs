// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page's repeated rows: the edge parts and the endpoints with
//! their profiles.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::Arg;
use ferrohealth_ui::view::form::{
    ERROR, FIELD_FLOW, FORM, Field, INPUT, Inner, described_by, described_with_unit, error_id,
    masked_input,
};
use ferrohealth_ui::view::masked::HoldWords;
use ferrohealth_ui::view::repeat::{Fields, ROW, Repeat, RepeatAdd, RepeatRow, RowEnd, repeat};
use ferrohealth_ui::view::surface::card;

use crate::locale::Msg;
use crate::visit::Visit;

/// Fields with a part inside their edge: a unit, a prefix, a refused value,
/// a disabled one and a masked identifier with its hold-to-show eye (R10).
pub(super) fn edge_parts(v: &Visit) -> Html {
    let timeout = Html::raw(format!(
        "<input id=\"edge-timeout\" name=\"timeout\" class=\"{INPUT}\" inputmode=\"numeric\" value=\"1200\" aria-describedby=\"{}\">",
        described_with_unit("edge-timeout", false, false)
    ));
    let base = Html::raw(format!(
        "<input id=\"edge-base\" name=\"base\" class=\"{INPUT}\" spellcheck=\"false\" value=\"cdr-a.example.org\">"
    ));
    let limit = Html::raw(format!(
        "<input id=\"edge-limit\" name=\"limit\" class=\"{INPUT}\" inputmode=\"numeric\" value=\"fifty\" aria-invalid=\"true\" aria-describedby=\"{}\">",
        described_with_unit("edge-limit", false, true)
    ));
    let retention = Html::raw(format!(
        "<input id=\"edge-retention\" name=\"retention\" class=\"{INPUT}\" inputmode=\"numeric\" value=\"30\" disabled aria-describedby=\"{}\">",
        described_with_unit("edge-retention", true, false)
    ));
    card(
        v.tr(Msg::FieldEdgeParts),
        Html::raw(format!(
            "<div class=\"{FORM}\">{}{}{}{}{}</div>",
            Field::new("edge-timeout", v.tr(Msg::Timeout))
                .trailing(Inner::Unit(v.tr(Msg::Milliseconds)))
                .view(timeout),
            Field::new("edge-base", v.tr(Msg::BaseAddress))
                .leading(Inner::Text(String::from("https://")))
                .view(base),
            Field::new("edge-limit", v.tr(Msg::RowLimit))
                .trailing(Inner::Unit(v.tr(Msg::RowsUnit)))
                .after(Html::raw(format!(
                    "<p id=\"{}\" class=\"{ERROR}\">{}</p>",
                    error_id("edge-limit"),
                    Html::text(v.tr(Msg::TypeWholeNumber))
                )))
                .view(limit),
            Field::new("edge-retention", v.tr(Msg::Retention))
                .help(v.tr(Msg::SetByDeployment))
                .trailing(Inner::Unit(v.tr(Msg::DaysUnit)))
                .view(retention),
            Field::new("edge-identifier", v.tr(Msg::MaskedIdentifier))
                .help(v.tr(Msg::MaskedIdentifierHelp))
                .trailing(Inner::HoldToShow(
                    HoldWords::of(&v.locale).named(v.tr(Msg::ShowIdentifierWhileHeld)),
                ))
                .view(masked_input(
                    "edge-identifier",
                    "identifier",
                    described_by("edge-identifier", true, false)
                )),
        )),
    )
}

/// A member's endpoints, each as an F7 sub-card row, and in each its
/// profiles, a list of one-field rows.
const ENDPOINTS: [(&str, &str, &[&str]); 2] = [
    (
        "node-c-pub",
        "https://node-c.example.org/query",
        &["https://profiles.example.org/patient-summary"],
    ),
    (
        "node-c-fhir",
        "https://node-c.example.org/fhir",
        &[
            "https://profiles.example.org/patient-summary",
            "https://profiles.example.org/lab-result",
        ],
    ),
];

/// The profile row `key` of the endpoint row `endpoint`, holding `url`.
fn profile_row(v: &Visit, endpoint: &str, key: &str, url: &str) -> RepeatRow {
    let id = format!("ep-{endpoint}-profile-{key}");
    let input = Html::raw(format!(
        "<input id=\"{id}\" name=\"endpoint[{endpoint}][profile][{key}]\" class=\"{INPUT}\" spellcheck=\"false\" value=\"{}\">",
        Html::text(url)
    ));
    RepeatRow {
        key: key.to_owned(),
        heading: None,
        fields: Field::new(id, v.tr(Msg::EpProfile))
            .label_hidden()
            .view(input),
        end: RowEnd::Remove {
            label: v.tr_with(Msg::EpRemoveProfile, &[("value", url.to_owned().into())]),
            name: String::from("remove-profile"),
            value: format!("{endpoint}/{key}"),
        },
        after: None,
    }
}

/// The key token of a profile's blank row, its own beside the endpoints'
/// [`ROW`], so a copied endpoint's profiles add in place.
const PROFILE: &str = "__profile__";

/// The profiles of the endpoint row `endpoint`, named `name`, with the blank
/// row Add copies.
fn profiles(v: &Visit, endpoint: &str, name: &str, urls: &[&str]) -> Html {
    let rows = urls
        .iter()
        .zip(1_u32..)
        .map(|(url, count)| profile_row(v, endpoint, &format!("p{count}"), url))
        .collect();
    repeat(Repeat {
        id: format!("ep-{endpoint}-profiles"),
        label: v.tr_with(Msg::EpProfiles, &[("name", name.to_owned().into())]),
        fields: Fields::One,
        head: vec![v.tr(Msg::EpProfile)],
        rows,
        token: PROFILE,
        blank: Some(profile_row(v, endpoint, PROFILE, "")),
        add: Some(RepeatAdd {
            label: v.tr(Msg::EpAddProfile),
            name: String::from("add-profile"),
            value: endpoint.to_owned(),
        }),
        empty: v.tr(Msg::EpProfilesNone),
        form: None,
    })
}

/// The endpoint row `key` named `name` at `base`, with its profiles.
fn endpoint_row(v: &Visit, key: &str, name: &str, base: &str, urls: &[&str]) -> RepeatRow {
    let id = format!("ep-{key}-id");
    let base_id = format!("ep-{key}-base");
    let named = [("name", Arg::from(name.to_owned()))];
    let fields = Html::raw(format!(
        "<div class=\"{FIELD_FLOW}\">{}{}</div>{}",
        Field::new(id.clone(), v.tr(Msg::EpId)).view(Html::raw(format!(
            "<input id=\"{id}\" name=\"endpoint[{key}][id]\" class=\"{INPUT}\" spellcheck=\"false\" value=\"{}\">",
            Html::text(name)
        ))),
        Field::new(base_id.clone(), v.tr(Msg::EpBase)).view(Html::raw(format!(
            "<input id=\"{base_id}\" name=\"endpoint[{key}][base]\" class=\"{INPUT}\" spellcheck=\"false\" value=\"{}\">",
            Html::text(base)
        ))),
        profiles(v, key, name, urls)
    ));
    RepeatRow {
        key: key.to_owned(),
        heading: Some(v.tr_with(Msg::EpHeading, &named)),
        fields,
        end: RowEnd::Remove {
            label: v.tr_with(Msg::EpRemove, &named),
            name: String::from("remove-endpoint"),
            value: key.to_owned(),
        },
        after: None,
    }
}

/// A member's endpoints: F7 sub-card rows, each with a list of one-field
/// rows of its own.
pub(super) fn endpoints(v: &Visit) -> Html {
    let rows = ENDPOINTS
        .iter()
        .zip(1_u32..)
        .map(|((name, base, urls), count)| endpoint_row(v, &format!("e{count}"), name, base, urls))
        .collect();
    let list = repeat(Repeat {
        id: String::from("member-endpoints"),
        label: v.tr(Msg::EpTitle),
        fields: Fields::Card,
        head: Vec::new(),
        rows,
        token: ROW,
        blank: Some(endpoint_row(v, ROW, "", "", &[])),
        add: Some(RepeatAdd {
            label: v.tr(Msg::EpAdd),
            name: String::from("add-endpoint"),
            value: String::from("member"),
        }),
        empty: v.tr(Msg::EpNone),
        form: None,
    });
    card(
        v.tr(Msg::EpTitle),
        Html::raw(format!(
            "<form method=\"post\" action=\"#\" class=\"{FORM}\">{list}</form>"
        )),
    )
}
