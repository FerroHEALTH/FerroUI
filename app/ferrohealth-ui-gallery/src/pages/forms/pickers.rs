// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page's pickers: the clause and system pickers, the choices, the
//! grouped fields, the nested reveal and the long label.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::Arg;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::combobox::{ComboOption, ComboboxWords, KeyedCombobox, keyed_combobox};
use ferrohealth_ui::view::form::{FORM, INPUT, SELECT, check, columns, field, field_flow};
use ferrohealth_ui::view::notice::{Announce, notice_text};
use ferrohealth_ui::view::segmented::choice_reveal;
use ferrohealth_ui::view::surface::card;

use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

use super::inputs::button;

/// One combobox per clause, its id made from the clause's place, each
/// described by its own help line.
fn clause_pickers(v: &Visit) -> Html {
    (1_u8..=2)
        .map(|place| {
            h(format!(
                "<div class=\"fui-field\">{}<p id=\"clause-{place}-system-help\" class=\"fui-help\">{}</p></div>",
                clause_picker(v, place),
                t(v.tr(Msg::SystemClauseDrawsCodesFrom))
            ))
        })
        .collect()
}

/// The code systems the comboboxes offer: each value, its message and its
/// canonical.
const SYSTEMS: [(&str, Msg, &str); 12] = [
    ("loinc", Msg::Loinc, "http://loinc.org"),
    ("snomed", Msg::SnomedCt, "http://snomed.info/sct"),
    ("ucum", Msg::Ucum, "http://unitsofmeasure.org"),
    ("icd10", Msg::Icd10, "http://hl7.org/fhir/sid/icd-10"),
    ("atc", Msg::Atc, "http://www.whocc.no/atc"),
    (
        "rxnorm",
        Msg::Rxnorm,
        "http://www.nlm.nih.gov/research/umls/rxnorm",
    ),
    ("iso3166", Msg::Iso3166Countries, "urn:iso:std:iso:3166"),
    ("bcp47", Msg::Bcp47Languages, "urn:ietf:bcp:47"),
    ("mime", Msg::MediaTypes, "urn:ietf:bcp:13"),
    (
        "gender",
        Msg::AdministrativeGender,
        "http://hl7.org/fhir/administrative-gender",
    ),
    (
        "status",
        Msg::ObservationStatus,
        "http://hl7.org/fhir/observation-status",
    ),
    (
        "synthetic",
        Msg::SyntheticGalleryCodes,
        "http://example.org/fhir/CodeSystem/gallery-synthetic",
    ),
];

/// The options of [`SYSTEMS`].
fn system_options(v: &Visit) -> Vec<ComboOption> {
    SYSTEMS
        .iter()
        .map(|(value, msg, url)| ComboOption::new(*value, v.tr(*msg)).detail(*url))
        .collect()
}

/// The combobox of the clause at `place`.
fn clause_picker(v: &Visit, place: u8) -> Html {
    let id = format!("clause-{place}-system");
    keyed_combobox(KeyedCombobox {
        described_by: Some(format!("{id}-help")),
        name: format!("clause{place}"),
        label: v.tr_with(
            Msg::ClauseCodeSystem,
            &[("place", Arg::from(u32::from(place)))],
        ),
        placeholder: v.tr(Msg::ChooseCodeSystem),
        options: system_options(v),
        selected: None,
        words: ComboboxWords::of(&v.locale),
        id,
        remote: None,
    })
}

/// The combobox of the code system, SNOMED CT chosen.
fn system_picker(v: &Visit) -> Html {
    keyed_combobox(KeyedCombobox {
        id: String::from("choose-system"),
        name: String::from("system"),
        label: v.tr(Msg::CodeSystem),
        placeholder: v.tr(Msg::ChooseCodeSystem),
        options: system_options(v),
        selected: Some(String::from("snomed")),
        described_by: None,
        words: ComboboxWords::of(&v.locale),
        remote: None,
    })
}

/// A long list to choose from, filtered as the reader types, beside a group
/// with its heading, a checkbox on the controls' line, and a disabled save
/// that names its reason.
pub(super) fn choosing(v: &Visit) -> Html {
    let code = h(format!(
        "<input id=\"choose-code\" name=\"code\" type=\"text\" class=\"{INPUT}\" placeholder=\"22298006\">"
    ));
    let active = h(String::from(
        "<input id=\"choose-active\" name=\"active\" type=\"checkbox\" checked>",
    ));
    card(
        v.tr(Msg::Combobox),
        h(format!(
            "<div class=\"{FORM}\">{}<fieldset class=\"fui-fieldset\"><legend class=\"fui-sr-only\">{}</legend>{}</fieldset><fieldset class=\"fui-fieldset\"><legend>{}</legend>{}</fieldset><div class=\"fui-cluster\"><button type=\"button\" class=\"{}\" disabled aria-describedby=\"choose-save-why\">{}</button><button type=\"button\" class=\"{}\">{}</button></div><p id=\"choose-save-why\" class=\"fui-help\">{}</p></div>",
            system_picker(v),
            t(v.tr(Msg::Clauses)),
            clause_pickers(v),
            t(v.tr(Msg::TheCode)),
            field_flow(Html::join([
                field("choose-code", v.tr(Msg::Code), None, code),
                check("choose-active", v.tr(Msg::ActiveConceptsOnly), active),
            ])),
            Variant::Primary.class(),
            label(Some(Glyph::SAVE), v.tr(Msg::SaveValueSet)),
            Variant::Danger.small(),
            label(Some(Glyph::TRASH), v.tr(Msg::RemoveClause)),
            t(v.tr(Msg::CanonicalClauseWhatSaveNeeds)),
        )),
    )
}

/// Related fields under a legend, in pairs, a disabled select and search
/// field inside the well, and a choice drawn as segments.
pub(super) fn grouped(v: &Visit) -> Html {
    let pair = columns(Html::join([
        field(
            "member-name",
            v.tr(Msg::Member),
            None,
            h(format!(
                "<input id=\"member-name\" type=\"text\" class=\"{INPUT}\" value=\"synthetic-node-a\">"
            )),
        ),
        field(
            "member-base",
            v.tr(Msg::BaseUrl),
            None,
            h(format!(
                "<input id=\"member-base\" type=\"text\" class=\"{INPUT}\" value=\"https://node-a.example.org/\">"
            )),
        ),
    ]));
    let flow = field_flow(Html::join([
        field(
            "member-system",
            v.tr(Msg::CodeSystem),
            None,
            h(format!(
                "<select id=\"member-system\" name=\"system\" class=\"{SELECT}\" disabled><option value=\"\">{}</option></select>",
                t(v.tr(Msg::ChooseMemberFirst))
            )),
        ),
        field(
            "member-code",
            v.tr(Msg::FindCodeCodeSystem),
            None,
            h(format!(
                "<input id=\"member-code\" name=\"code\" type=\"search\" class=\"{INPUT}\" disabled placeholder=\"{}\">",
                t(v.tr(Msg::ChooseCodeSystem))
            )),
        ),
        button(Variant::Secondary.class(), &v.tr(Msg::Search), true),
    ]));
    let reveal = choice_reveal(
        "secret",
        v.tr(Msg::ClientSecret),
        &[
            ("keep", v.tr(Msg::Keep)),
            ("replace", v.tr(Msg::Replace)),
            ("remove", v.tr(Msg::Remove)),
        ],
        "keep",
        vec![
            Html::empty(),
            field(
                "member-secret",
                v.tr(Msg::NewSecret),
                None,
                h(format!(
                    "<input id=\"member-secret\" name=\"secret-value\" type=\"password\" class=\"{INPUT}\" autocomplete=\"new-password\">"
                )),
            ),
            notice_text(
                Tone::Warn,
                Announce::Status,
                v.tr(Msg::MemberSignsNoSecretOnce),
            ),
        ],
    );
    card(
        v.tr(Msg::GroupedFields),
        h(format!(
            "<form method=\"post\" action=\"#\" class=\"{FORM}\"><fieldset class=\"fui-fieldset fui-well\"><legend>{}</legend>{pair}{flow}</fieldset>{reveal}</form>",
            t(v.tr(Msg::FederationMember))
        )),
    )
}

/// A grant whose chosen kind reveals its settings, one of which is a secret
/// revealed by a choice of its own: each reveal answers to its own choice.
pub(super) fn nested_reveal(v: &Visit) -> Html {
    let secret = choice_reveal(
        "grant-secret",
        v.tr(Msg::ClientSecret),
        &[("keep", v.tr(Msg::Keep)), ("replace", v.tr(Msg::Replace))],
        "keep",
        vec![
            Html::empty(),
            field(
                "grant-secret-value",
                v.tr(Msg::NewSecret),
                None,
                h(format!(
                    "<input id=\"grant-secret-value\" type=\"password\" class=\"{INPUT}\" autocomplete=\"new-password\">"
                )),
            ),
        ],
    );
    let client = h(format!(
        "<div class=\"fui-stack\">{}{secret}</div>",
        field(
            "grant-client-id",
            v.tr(Msg::ClientId),
            None,
            h(format!(
                "<input id=\"grant-client-id\" type=\"text\" class=\"{INPUT}\" value=\"synthetic-client\">"
            )),
        )
    ));
    let token = field(
        "grant-token-value",
        v.tr(Msg::TokenLabel),
        None,
        h(format!(
            "<input id=\"grant-token-value\" type=\"password\" class=\"{INPUT}\" autocomplete=\"off\">"
        )),
    );
    card(
        v.tr(Msg::RevealInsideReveal),
        h(format!(
            "<form method=\"post\" action=\"#\" class=\"{FORM}\">{}</form>",
            choice_reveal(
                "grant",
                v.tr(Msg::Grant),
                &[
                    ("client", v.tr(Msg::ClientCredentials)),
                    ("token", v.tr(Msg::Token))
                ],
                "client",
                vec![client, token],
            )
        )),
    )
}

/// A button whose words are longer than a phone's column, which break
/// between words rather than push the page sideways, beside a short one.
pub(super) fn long_label(v: &Visit) -> Html {
    card(
        v.tr(Msg::LongButtonLabel),
        h(format!(
            "<div class=\"fui-cluster\"><button type=\"button\" id=\"long-label\" class=\"{}\">{}</button><button type=\"button\" class=\"{}\">{}</button></div>",
            Variant::Primary.class(),
            label(
                Some(Glyph::DOWNLOAD),
                v.tr(Msg::DownloadWholeConfigurationToml)
            ),
            Variant::Secondary.class(),
            label(Some(Glyph::X), v.tr(Msg::Clear)),
        )),
    )
}
