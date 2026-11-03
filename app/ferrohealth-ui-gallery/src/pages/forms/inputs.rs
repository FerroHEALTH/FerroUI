// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page's fields: the federated search, the field shapes, pick or type, the inline and per-row forms.

use ferrohealth_ui::calendar::Bounds;
use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::combobox::{ComboOption, ComboboxWords, KeyedCombobox, keyed_combobox};
use ferrohealth_ui::view::date::{DateField, DateWords, date_field};
use ferrohealth_ui::view::form::{
    ERROR, FORM, Field, INPUT, SELECT, SWITCH, TEXTAREA, action_field, check, check_field,
    chip_choices, described_by, error_id, field, field_flow, help_id, keyed_check_field,
    keyed_field,
};
use ferrohealth_ui::view::layout::{FieldRow, field_row};
use ferrohealth_ui::view::pick_or_type::{PickOrType, Picker, pick_or_type};
use ferrohealth_ui::view::surface::card;

use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

use super::dates::{date, today};

/// A query form: the nodes as checkbox chips, an option as a switch, and its
/// actions at its foot.
pub(super) fn federated(v: &Visit) -> Html {
    let nodes = ["node-a", "node-b", "node-c", "node-d"]
        .into_iter()
        .map(|node| (String::from(node), format!("synthetic-{node}")))
        .collect();
    let checked = [String::from("node-a"), String::from("node-c")];
    let partial = h(format!(
        "<input id=\"query-partial\" name=\"partial\" type=\"checkbox\" role=\"switch\" class=\"{SWITCH}\" checked aria-describedby=\"{}\">",
        help_id("query-partial")
    ));
    card(
        v.tr(Msg::FederatedQuery),
        h(format!(
            "<form method=\"post\" action=\"#\" class=\"{FORM}\">{}{}<div class=\"fui-cluster\"><button type=\"button\" class=\"{}\">{}</button><button type=\"button\" class=\"{}\">{}</button></div></form>",
            chip_choices("nodes", v.tr(Msg::AskTheseNodes), nodes, &checked),
            check_field(
                "query-partial",
                v.tr(Msg::AcceptPartialAnswer),
                v.tr(Msg::WhenNodeDoesNotAnswer),
                partial,
            ),
            Variant::Primary.class(),
            label(Some(Glyph::PLAY), v.tr(Msg::RunQuery)),
            Variant::Secondary.class(),
            label(Some(Glyph::X), v.tr(Msg::Clear)),
        )),
    )
}

/// A field whose label is made at run time, one whose refusal shows after
/// its help, a list of values whose labels only a screen reader reads, and
/// the button that adds a value with the line its own refusal shows in.
pub(super) fn shapes(v: &Visit) -> Html {
    let member = "synthetic-node-a";
    let base = "shape-base";
    let base_input = h(format!(
        "<input id=\"{base}\" name=\"base\" type=\"url\" class=\"{INPUT}\" value=\"node-a.example.org\" aria-invalid=\"true\" aria-describedby=\"{}\">",
        described_by(base, true, true).unwrap_or_default()
    ));
    let base_field = Field::new(
        base,
        v.tr_with(Msg::BaseUrlOf, &[("member", member.into())]),
    )
    .help(v.tr(Msg::WhereGatewayReachesMember))
    .after(h(format!(
        "<p id=\"{}\" class=\"{ERROR}\" aria-live=\"polite\">{}</p>",
        error_id(base),
        t(v.tr(Msg::AddressMustAbsoluteStartingHttps))
    )))
    .view(base_input);
    let rows: Html = ["urn:example:alpha", "urn:example:beta"]
        .into_iter()
        .zip(1_u32..)
        .map(|(value, place)| {
            let id = format!("shape-value-{place}");
            let input = h(format!(
                "<input id=\"{id}\" name=\"values\" type=\"text\" class=\"{INPUT} fui-mono\" value=\"{value}\">"
            ));
            field_row(
                FieldRow::WithActions,
                Html::join([
                    Field::new(id, v.tr(Msg::Value)).label_hidden().view(input),
                    h(format!(
                        "<button type=\"button\" class=\"{}\" aria-label=\"{}\">{}</button>",
                        Variant::Ghost.class(),
                        t(v.tr_with(Msg::RemoveValue, &[("value", value.into())])),
                        label(Some(Glyph::TRASH), v.tr(Msg::Remove))
                    )),
                ]),
            )
        })
        .collect();
    let add = "shape-add";
    let adding = action_field(
        h(format!(
            "<button type=\"button\" id=\"{add}\" class=\"{}\" aria-describedby=\"{}\">{}</button>",
            Variant::Secondary.class(),
            error_id(add),
            label(Some(Glyph::PLUS), v.tr(Msg::AddValue))
        )),
        h(format!(
            "<p id=\"{}\" class=\"{ERROR}\" aria-live=\"polite\">{}</p>",
            error_id(add),
            t(v.tr(Msg::TenValuesMost))
        )),
    );
    card(
        v.tr(Msg::FieldShapes),
        h(format!(
            "<form method=\"post\" action=\"#\" class=\"{FORM}\">{base_field}<fieldset class=\"fui-fieldset\"><legend class=\"fui-label\">{}</legend><div class=\"fui-stack\">{rows}</div>{adding}</fieldset></form>",
            t(v.tr(Msg::AllowedAudiences))
        )),
    )
}

/// The option of a pick-or-type select that asks for the typed input.
const ANOTHER: &str = "another";

/// One value picked from a short list in a select, or from a long one in a
/// combobox, or typed behind the last option, each control with its own
/// label, and each with a button back to the list.
pub(super) fn pick_or_typed(v: &Visit) -> Html {
    let select = h(format!(
        "<select id=\"pick-version-choice\" class=\"{SELECT}\" aria-describedby=\"{help}\"><option value=\"\">{}</option><option value=\"2.81\">2.81</option><option value=\"2.80\">2.80</option><option value=\"{ANOTHER}\" data-other>{}</option></select>",
        t(v.tr(Msg::OneServerResolves)),
        t(v.tr(Msg::AnotherVersion)),
        help = help_id("pick-version"),
    ));
    let version = pick_or_type(PickOrType {
        label: v.tr(Msg::Version),
        picker: Picker::Control {
            id: String::from("pick-version-choice"),
            control: select,
        },
        typed_id: String::from("pick-version"),
        typed: h(format!(
            "<input id=\"pick-version\" name=\"version\" type=\"text\" class=\"{INPUT}\" placeholder=\"2.79\" aria-describedby=\"{}\">",
            help_id("pick-version")
        )),
        typing: false,
        after: Some(h(format!(
            "<p id=\"{}\" class=\"fui-help\">{}</p><button type=\"button\" id=\"pick-version-back\" class=\"{}\" data-fui-pick-back hidden>{}</button>",
            help_id("pick-version"),
            t(v.tr(Msg::VersionServerHoldsAnotherOne)),
            Variant::Ghost.small(),
            label(Some(Glyph::ARROW_LEFT), v.tr(Msg::ChooseFromList))
        ))),
    });
    let offered = vec![
        ComboOption::new("http://loinc.org", v.tr(Msg::Loinc)).detail("http://loinc.org"),
        ComboOption::new("http://snomed.info/sct", v.tr(Msg::SnomedCt))
            .detail("http://snomed.info/sct"),
        ComboOption::new(ANOTHER, v.tr(Msg::AnotherCanonical)).other(),
    ];
    let canonical = pick_or_type(PickOrType {
        label: v.tr(Msg::CodeSystem),
        picker: Picker::Labelled(keyed_combobox(KeyedCombobox {
            id: String::from("pick-canonical-choice"),
            name: String::from("picked"),
            label: v.tr(Msg::CodeSystem),
            placeholder: v.tr(Msg::ChooseOneServerHolds),
            options: offered,
            selected: None,
            described_by: Some(help_id("pick-canonical")),
            words: ComboboxWords::of(&v.locale),
            remote: None,
        })),
        typed_id: String::from("pick-canonical"),
        typed: h(format!(
            "<input id=\"pick-canonical\" name=\"canonical\" type=\"url\" class=\"{INPUT}\" placeholder=\"{}\" aria-describedby=\"{}\">",
            "http://example.org/fhir/CodeSystem/synthetic",
            help_id("pick-canonical")
        )),
        typing: false,
        after: Some(h(format!(
            "<p id=\"{}\" class=\"fui-help\">{}</p><button type=\"button\" id=\"pick-canonical-back\" class=\"{}\" data-fui-pick-back hidden>{}</button>",
            help_id("pick-canonical"),
            t(v.tr(Msg::CodeSystemServerHoldsCanonical)),
            Variant::Ghost.small(),
            label(Some(Glyph::ARROW_LEFT), v.tr(Msg::ChooseFromList))
        ))),
    });
    card(
        v.tr(Msg::PickType),
        h(format!(
            "<form method=\"post\" action=\"#\" class=\"{FORM}\">{version} {canonical}</form>"
        )),
    )
}

/// The id of the editor's text area.
pub(super) const DRAFT: &str = "gallery-draft";

/// The fields, the select, the text area and the checkbox row.
pub(super) fn fields(v: &Visit) -> Html {
    let reason = h(format!(
        "<input id=\"form-reason\" name=\"reason\" type=\"text\" class=\"{INPUT}\" maxlength=\"200\" aria-describedby=\"{}\">",
        help_id("form-reason")
    ));
    let format = h(format!(
        "<select id=\"form-format\" name=\"format\" class=\"{SELECT}\"><option value=\"canonical\">{}</option><option value=\"flat\">{}</option></select>",
        t(v.tr(Msg::Canonical)),
        t(v.tr(Msg::Flat))
    ));
    let bundle = h(format!(
        "<textarea id=\"form-bundle\" name=\"bundle\" rows=\"4\" class=\"{TEXTAREA}\"></textarea>"
    ));
    let until = date_field(date(
        v,
        "form-until",
        "until",
        v.tr(Msg::KeepDraftUntil),
        "",
        Bounds {
            earliest: today().step(1),
            latest: None,
        },
    ));
    let confirm = h(String::from(
        "<input id=\"form-confirm\" name=\"confirmed\" type=\"checkbox\">",
    ));
    card(
        v.tr(Msg::Fields),
        h(format!(
            "<form method=\"post\" action=\"#\" class=\"{FORM}\">{}{}{}{until}{}<button type=\"button\" class=\"{}\">{}</button></form>",
            field(
                "form-reason",
                v.tr(Msg::WhyYouSaveOneLine),
                Some(v.tr(Msg::TwoHundredCharactersMost)),
                reason
            ),
            field("form-format", v.tr(Msg::CompositionFormat), None, format),
            field("form-bundle", v.tr(Msg::BundleJson), None, bundle),
            check("form-confirm", v.tr(Msg::SaveDraftReloadSetBridge), confirm),
            Variant::Primary.class(),
            label(Some(Glyph::SAVE), v.tr(Msg::Save)),
        )),
    )
}

/// A button of `variant` saying `words`, disabled when `disabled`.
pub(super) fn button(variant: &str, words: &str, disabled: bool) -> Html {
    h(format!(
        "<button type=\"button\" class=\"{variant}\"{}>{}</button>",
        if disabled { " disabled" } else { "" },
        t(words)
    ))
}

/// Fields in one row that share its width, a select and a date-time input
/// beside text inputs, and the button variants enabled and disabled.
pub(super) fn inline(v: &Visit) -> Html {
    let text = |id: &str, name: &str, placeholder: &str| {
        h(format!(
            "<input id=\"{id}\" name=\"{name}\" type=\"text\" class=\"{INPUT}\" placeholder=\"{}\">",
            t(placeholder)
        ))
    };
    let ehr = text("row-ehr", "ehr", &v.tr(Msg::EhrIdUuidBlankLets));
    let subject = text("row-subject", "subject", &v.tr(Msg::ExternalSubjectId));
    let namespace = text("row-namespace", "namespace", &v.tr(Msg::Namespace));
    let path = text("row-path", "path", "/context/start_time/value");
    let version = h(format!(
        "<select id=\"row-version\" name=\"version\" class=\"{SELECT}\"><option value=\"latest\">{}</option><option value=\"1\">{}</option></select>",
        t(v.tr(Msg::Latest)),
        t(v.tr(Msg::Version1))
    ));
    let at = date_field(DateField {
        with_time: true,
        words: DateWords::of(&v.locale, true),
        ..date(
            v,
            "row-at",
            "at",
            v.tr(Msg::TimeTravelUtc),
            "",
            Bounds {
                earliest: None,
                latest: Some(today()),
            },
        )
    });
    let first = field_flow(Html::join([
        field("row-ehr", v.tr(Msg::EhrIdOptional), None, ehr),
        field("row-subject", v.tr(Msg::SubjectIdOptional), None, subject),
        field(
            "row-namespace",
            v.tr(Msg::SubjectNamespaceOptional),
            None,
            namespace,
        ),
        button(Variant::Primary.class(), &v.tr(Msg::Create), false),
    ]));
    let second = field_flow(Html::join([
        field("row-version", v.tr(Msg::Version), None, version),
        at,
        field("row-path", v.tr(Msg::TargetPathOptional), None, path),
        button(Variant::Secondary.class(), &v.tr(Msg::Time), false),
    ]));
    let variants = Html::join([
        button(Variant::Primary.class(), &v.tr(Msg::Primary), false),
        button(Variant::Secondary.class(), &v.tr(Msg::Secondary), false),
        button(Variant::Danger.class(), &v.tr(Msg::Danger), false),
        button(Variant::Ghost.class(), &v.tr(Msg::Ghost), false),
        button(Variant::Secondary.small(), &v.tr(Msg::Small), false),
    ]);
    let disabled = Html::join([
        button(Variant::Primary.class(), &v.tr(Msg::PrimaryDisabled), true),
        button(
            Variant::Secondary.class(),
            &v.tr(Msg::SecondaryDisabled),
            true,
        ),
        button(Variant::Danger.class(), &v.tr(Msg::DangerDisabled), true),
    ]);
    card(
        v.tr(Msg::FieldsRow),
        h(format!(
            "<div class=\"{FORM}\">{first}{second}<div class=\"fui-cluster\">{variants}</div><div class=\"fui-cluster\">{disabled}</div></div>"
        )),
    )
}

/// A field and a switch drawn once per clause, each id made from the
/// clause's key, so every label names its own control and every help line
/// its own.
pub(super) fn per_row(v: &Visit) -> Html {
    let clauses = [
        ("include-1", v.tr(Msg::Include)),
        ("exclude-2", v.tr(Msg::Exclude)),
    ];
    let rows: Html = clauses
        .into_iter()
        .map(|(key, verb)| {
            let search = format!("compose-search-{key}");
            let search_help = help_id(&search);
            let inactive = format!("compose-inactive-{key}");
            let inactive_help = help_id(&inactive);
            let find = keyed_field(
                search.clone(),
                v.tr(Msg::FindCodeCodeSystem),
                Some(v.tr(Msg::TwoLettersMore)),
                h(format!(
                    "<input id=\"{search}\" name=\"search-{key}\" type=\"search\" class=\"{INPUT}\" placeholder=\"{}\" aria-describedby=\"{search_help}\">",
                    t(v.tr(Msg::BloodPressure))
                )),
            );
            let switch = keyed_check_field(
                inactive.clone(),
                v.tr(Msg::IncludeInactiveCodes),
                v.tr(Msg::CodesSystemRetiredStillMatch),
                h(format!(
                    "<input type=\"checkbox\" role=\"switch\" id=\"{inactive}\" name=\"inactive-{key}\" class=\"{SWITCH}\" aria-describedby=\"{inactive_help}\">"
                )),
            );
            h(format!(
                "<fieldset class=\"fui-fieldset\"><legend>{} {}</legend>{}{switch}</fieldset>",
                t(verb),
                t(v.tr(Msg::Codes)),
                field_flow(Html::join([
                    find,
                    button(Variant::Secondary.class(), &v.tr(Msg::Add), false)
                ]))
            ))
        })
        .collect();
    card(
        v.tr(Msg::FieldsPerRow),
        h(format!("<div class=\"{FORM}\">{rows}</div>")),
    )
}
