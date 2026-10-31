// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page: the fields, the checkbox row, the highlighted editor with
//! diagnostics that link to its lines, and the filter over a list.
//!
//! The page takes one post of its own: the note of the action answered on
//! its page, which it draws again with the count of its words or the
//! refusal, the note kept in its field and every field the form marks
//! secret left out (#485, #499).

use ferrohealth_ui::calendar::{Bounds, Day, Preset, Quick, Weekday};
use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::{Arg, Phrase};
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::code::Gravity;
use ferrohealth_ui::view::combobox::{ComboOption, ComboboxWords, KeyedCombobox, keyed_combobox};
use ferrohealth_ui::view::date::{
    DateField, DateRange, DateWords, QuickChoice, RangeEnd, RangePreset, date_field, date_range,
};
use ferrohealth_ui::view::diagnostics::{
    DiagnosticsWords, FileFindings, Finding, Position, diagnostics_by_file,
};
use ferrohealth_ui::view::editor::{
    Editor, EditorWords, LineMark, MarkKind, Opened, Panel, code_editor, editor_diagnostics,
    editor_panel, editor_split, line_id,
};
use ferrohealth_ui::view::filter::{ROW_TEXT, TableFilter, table_filter};
use ferrohealth_ui::view::form::{
    ERROR, FIELD_FLOW, FORM, Field, INPUT, Inner, Posted, SELECT, SWITCH, TEXTAREA, action_field,
    check, check_field, chip_choices, columns, described_by, described_with_unit, error_id, field,
    field_flow, help_id, keyed_check_field, keyed_field, masked_input,
};
use ferrohealth_ui::view::layout::{FieldRow, field_row};
use ferrohealth_ui::view::masked::HoldWords;
use ferrohealth_ui::view::notice::{Announce, notice_text};
use ferrohealth_ui::view::pick_or_type::{PickOrType, Picker, pick_or_type};
use ferrohealth_ui::view::repeat::{Fields, ROW, Repeat, RepeatAdd, RepeatRow, RowEnd, repeat};
use ferrohealth_ui::view::segmented::choice_reveal;
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{
    Crumb, Header, Mode, SectionHead, card, editor_heading, grid, grid_start, page_header,
    panel_section, panel_sections_card, stack, well,
};
use ferrohealth_ui::view::template::list::HeadingCount;
use ferrohealth_ui::view::upload::{Sending, Upload, UploadWords, upload_field};

use crate::app::{DRAFT_ROUTE, Page};
use crate::data::{
    CLEAN_FILE, CONTEXT_MAPPING, DRAFT_FINDINGS, DRAFT_MAPPING, DRAFT_OPENED, DRAFT_PATH,
    DRAFT_TEXT, FINDING_CODES, LONG_FINDING, MAPPING_PATH, WHOLE_FINDING,
};
use crate::locale::{Msg, message};
use crate::markup::{h, t};
use crate::visit::Visit;

mod answer;

/// A query form: the nodes as checkbox chips, an option as a switch, and its
/// actions at its foot.
fn federated(v: &Visit) -> Html {
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
fn shapes(v: &Visit) -> Html {
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
fn pick_or_typed(v: &Visit) -> Html {
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
pub const DRAFT: &str = "gallery-draft";

/// The fields, the select, the text area and the checkbox row.
fn fields(v: &Visit) -> Html {
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
fn button(variant: &str, words: &str, disabled: bool) -> Html {
    h(format!(
        "<button type=\"button\" class=\"{variant}\"{}>{}</button>",
        if disabled { " disabled" } else { "" },
        t(words)
    ))
}

/// Fields in one row that share its width, a select and a date-time input
/// beside text inputs, and the button variants enabled and disabled.
fn inline(v: &Visit) -> Html {
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
fn per_row(v: &Visit) -> Html {
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
    })
}

/// A long list to choose from, filtered as the reader types, beside a group
/// with its heading, a checkbox on the controls' line, and a disabled save
/// that names its reason.
fn choosing(v: &Visit) -> Html {
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
fn grouped(v: &Visit) -> Html {
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
fn nested_reveal(v: &Visit) -> Html {
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
fn long_label(v: &Visit) -> Html {
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

/// The extensions the upload field takes.
const UPLOAD_ACCEPT: &[&str] = &[".opt", ".xml"];

/// The largest file the upload field reads.
const UPLOAD_MAX_BYTES: u64 = 2 * 1024 * 1024;

/// The upload field: a file dropped or chosen, read into the text area.
fn upload(v: &Visit) -> Html {
    let field = upload_field(
        &Upload {
            id: "gallery-upload",
            label: v.tr(Msg::OperationalTemplate).into(),
            choose: v.tr(Msg::ChooseOptFile).into(),
            accept: UPLOAD_ACCEPT,
            max_bytes: UPLOAD_MAX_BYTES,
            placeholder: "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".into(),
            words: UploadWords::of(&v.locale, UPLOAD_ACCEPT, UPLOAD_MAX_BYTES),
        },
        Sending::default(),
    );
    card(v.tr(Msg::Upload), field)
}

/// The highlighted editor over the synthetic YAML, with two diagnostics
/// marking and linking to their lines, and the bytes it holds.
fn editor(v: &Visit) -> Html {
    let text = crate::data::YAML;
    let marks = vec![
        LineMark {
            line: 4,
            kind: MarkKind::Warning,
        },
        LineMark {
            line: 13,
            kind: MarkKind::Error,
        },
    ];
    let field = code_editor(&Editor {
        id: DRAFT.into(),
        language: Language::Yaml,
        text: String::from(text),
        marks,
        editable: true,
    });
    let bytes = u32::try_from(text.len()).unwrap_or(u32::MAX);
    let lines = u32::try_from(text.matches('\n').count())
        .unwrap_or(u32::MAX)
        .saturating_add(1);
    let size = v.tr_with(
        Msg::SizeLines,
        &[("bytes", Arg::from(bytes)), ("lines", Arg::from(lines))],
    );
    let diagnostics: Html = [
        (4, Tone::Warn, "warning", v.tr(Msg::CommentAfterValueKeptBut)),
        (
            13,
            Tone::Bad,
            "error",
            v.tr(Msg::TemplateGalleryTemplateV1Not),
        ),
    ]
    .into_iter()
    .map(|(line, tone, word, text)| {
        h(format!(
            "<li>{}<span class=\"fui-issue-head\"><a href=\"#{}\">{}</a>:</span><span class=\"fui-issue-text\">{}</span></li>",
            badge(tone, word),
            line_id(DRAFT, line),
            t(v.tr_with(Msg::IssueLine, &[("line", Arg::from(line))])),
            t(text)
        ))
    })
    .collect();
    card(
        v.tr(Msg::HighlightedEditor),
        Html::join([
            Field::new(DRAFT, v.tr(Msg::File))
                .after(h(format!(
                    "<p class=\"fui-help\" role=\"status\">{}</p>",
                    t(size)
                )))
                .view(field),
            h(format!("<ul class=\"fui-issues\">{diagnostics}</ul>")),
        ]),
    )
}

/// The id of the code editor's text area.
pub const MAPPING_DRAFT: &str = "gallery-mapping";

/// The id of the new mapping's text area.
pub const NEW_MAPPING: &str = "gallery-new-mapping";

/// The code editor (B2) beside its diagnostics (B3), in a form that posts
/// its draft: the draft opened with a warning and an error, and the list
/// beside it in step with the cursor.
fn mapping_editor(v: &Visit) -> Html {
    let [(note, note_line, note_column), (retries, line, column)] = DRAFT_FINDINGS;
    let marks = vec![
        LineMark {
            line: note_line,
            kind: MarkKind::Warning,
        },
        LineMark {
            line,
            kind: MarkKind::Error,
        },
    ];
    let field = Editor {
        id: MAPPING_DRAFT.into(),
        language: Language::Yaml,
        text: String::from(DRAFT_TEXT),
        marks,
        editable: true,
    };
    let found = |gravity, code: &str, (line, column), message| Finding {
        gravity,
        code: String::from(code),
        at: Some(Position { line, column }),
        message,
        mapping: Some(String::from(DRAFT_MAPPING)),
    };
    let findings = FileFindings {
        file: String::from(DRAFT_PATH),
        href: None,
        lines: format!("#{MAPPING_DRAFT}-L"),
        current: None,
        all: None,
        findings: vec![
            found(
                Gravity::Warning,
                note,
                (note_line, note_column),
                v.tr(Msg::NoteKeptNeverRead),
            ),
            found(
                Gravity::Error,
                retries,
                (line, column),
                v.tr(Msg::RetriesMustBeNumber),
            ),
        ],
    };
    let check = h(format!(
        "<button type=\"submit\" class=\"{}\">{}</button>",
        Variant::Secondary.small(),
        label(Some(Glyph::LIST_CHECKS), v.tr(Msg::Check))
    ));
    let caret = Position::default();
    let title = v.tr(Msg::Diagnostics);
    let list = editor_diagnostics(
        &title,
        &findings,
        &DiagnosticsWords::of(&v.locale),
        &field,
        caret,
    );
    let panel = editor_panel(Panel {
        editor: field,
        label: v.tr_with(Msg::DraftOf, &[("name", DRAFT_PATH.into())]),
        lead: v.tr(Msg::YamlFhirconnectContext),
        facts: vec![v.tr(Msg::Utf8Lf)],
        opened: Opened::Text(String::from(DRAFT_OPENED)),
        caret,
        tools: Some(check),
        end: None,
        words: EditorWords::of(&v.locale),
    });
    let side = panel_sections_card(
        title.clone(),
        panel_section(
            &v.locale,
            Some(SectionHead {
                title,
                count: Some(HeadingCount::All(2)),
                actions: None,
            }),
            list,
        ),
    );
    editor_split(
        h(format!(
            "<form method=\"post\" action=\"{DRAFT_ROUTE}\">{panel}</form>"
        )),
        side,
    )
}

/// The code editor of a new file, empty, in a panel section (B10).
fn new_mapping(v: &Visit) -> Html {
    let panel = editor_panel(Panel {
        editor: Editor {
            id: NEW_MAPPING.into(),
            language: Language::Yaml,
            text: String::new(),
            marks: Vec::new(),
            editable: true,
        },
        label: v.tr(Msg::NewMappingFile),
        lead: v.tr(Msg::YamlFhirconnectContext),
        facts: vec![v.tr(Msg::Utf8Lf)],
        opened: Opened::New,
        caret: Position::default(),
        tools: None,
        end: None,
        words: EditorWords::of(&v.locale),
    });
    panel_sections_card(
        v.tr(Msg::NewMapping),
        panel_section(
            &v.locale,
            Some(SectionHead {
                title: v.tr(Msg::NewMapping),
                count: None,
                actions: None,
            }),
            panel,
        ),
    )
}

/// The diagnostics list of two files, errors first, the current row on the
/// draft's error: the code page's mapping file, and the draft the code
/// editor holds.
fn diagnostics_of_files(v: &Visit) -> Html {
    let [(note, note_line, note_column), (retries, line, column)] = DRAFT_FINDINGS;
    let [code_retries, code_start] = FINDING_CODES;
    let found = |gravity, code: &str, at: Option<(u32, u32)>, message, mapping: &str| Finding {
        gravity,
        code: String::from(code),
        at: at.map(|(line, column)| Position { line, column }),
        message,
        mapping: Some(String::from(mapping)),
    };
    let files = [
        FileFindings {
            file: String::from(MAPPING_PATH),
            href: Some(String::from("code")),
            lines: String::from("code#file-L"),
            current: None,
            all: None,
            findings: vec![
                found(
                    Gravity::Warning,
                    code_start,
                    Some((20, 3)),
                    v.tr(Msg::FindingStartTwice),
                    CONTEXT_MAPPING,
                ),
                found(
                    Gravity::Error,
                    code_retries,
                    Some((18, 12)),
                    v.tr(Msg::FindingRetries),
                    CONTEXT_MAPPING,
                ),
            ],
        },
        FileFindings {
            file: String::from(DRAFT_PATH),
            href: None,
            lines: format!("#{MAPPING_DRAFT}-L"),
            current: Some(line),
            all: None,
            findings: vec![
                found(
                    Gravity::Warning,
                    note,
                    Some((note_line, note_column)),
                    v.tr(Msg::NoteKeptNeverRead),
                    DRAFT_MAPPING,
                ),
                found(
                    Gravity::Error,
                    retries,
                    Some((line, column)),
                    v.tr(Msg::RetriesMustBeNumber),
                    DRAFT_MAPPING,
                ),
                found(
                    Gravity::Error,
                    WHOLE_FINDING,
                    None,
                    v.tr(Msg::ProfileUnknown),
                    DRAFT_MAPPING,
                ),
            ],
        },
    ];
    let title = v.tr(Msg::DiagnosticsByFile);
    panel_sections_card(
        title.clone(),
        panel_section(
            &v.locale,
            Some(SectionHead {
                title: v.tr(Msg::Diagnostics),
                count: Some(HeadingCount::All(5)),
                actions: None,
            }),
            diagnostics_by_file(&v.locale, &title, &files),
        ),
    )
}

/// The diagnostics list at real size: a finding whose message and mapping
/// run past a phone's width, under a long path, and a file the check found
/// clean, which says so.
fn diagnostics_real_size(v: &Visit) -> Html {
    let [path, code, lines, mapping] = LONG_FINDING;
    let [clean, clean_lines] = CLEAN_FILE;
    let files = [
        FileFindings {
            file: String::from(path),
            href: Some(String::from("code")),
            lines: String::from(lines),
            current: None,
            all: None,
            findings: vec![Finding {
                gravity: Gravity::Error,
                code: String::from(code),
                at: Some(Position {
                    line: 1204,
                    column: 37,
                }),
                message: v.tr(Msg::PathNamesNoNode),
                mapping: Some(String::from(mapping)),
            }],
        },
        FileFindings {
            file: String::from(clean),
            href: None,
            lines: String::from(clean_lines),
            current: None,
            all: None,
            findings: Vec::new(),
        },
    ];
    let title = v.tr(Msg::DiagnosticsRealSize);
    panel_sections_card(
        title.clone(),
        panel_section(
            &v.locale,
            Some(SectionHead {
                title: title.clone(),
                count: Some(HeadingCount::All(1)),
                actions: None,
            }),
            diagnostics_by_file(&v.locale, &title, &files),
        ),
    )
}

/// The filter over a list of synthetic file names.
fn filter(v: &Visit) -> Html {
    let names: Vec<String> = crate::data::rows()
        .into_iter()
        .take(12)
        .map(|row| row.name)
        .collect();
    let total = u32::try_from(names.len()).unwrap_or(u32::MAX);
    let items: Html = names
        .iter()
        .map(|name| {
            h(format!(
                "<li {ROW_TEXT}=\"{0}\"><code>{0}</code></li>",
                t(name)
            ))
        })
        .collect();
    let filter = table_filter(TableFilter {
        id: "gallery-filter",
        label: v.tr(Msg::FilterFiles),
        rows: String::from("gallery-filter-rows"),
        count: v.tr_with(
            Msg::FilesOnPage,
            &[("kept", Arg::from(total)), ("total", Arg::from(total))],
        ),
        phrase: Phrase::new(v.tr_with(
            Msg::FilesOnPage,
            &[
                ("kept", Phrase::open("kept")),
                ("total", Phrase::open("total")),
            ],
        )),
        words: String::new(),
    });
    card(
        v.tr(Msg::Filter),
        h(format!(
            "{filter}<ul id=\"gallery-filter-rows\" class=\"fui-divided\" aria-label=\"{}\">{items}</ul>",
            t(v.tr(Msg::Files))
        )),
    )
}

/// The header of an editor that creates a file and of one that edits a
/// file: the title says which, and the trail ends with the list a new file
/// joins or with the file edited.
fn editor_headers(v: &Visit) -> Html {
    let header = |mode: Mode| {
        let mut crumbs = vec![Crumb::new(v.tr(Msg::MappingFilesCrumb), String::from("#"))];
        if let Mode::Editing(name) = &mode {
            crumbs.push(Crumb::new(name.clone(), String::from("#")));
        }
        well(editor_heading(
            &v.locale,
            message(Msg::FileEditor),
            &mode,
            Header {
                crumbs,
                ..Header::default()
            },
            ActionGroup::none(),
        ))
    };
    card(
        v.tr(Msg::EditorHeaders),
        stack(h(format!(
            "<div data-editor-mode=\"new\">{}</div><div data-editor-mode=\"edit\">{}</div>",
            header(Mode::Creating),
            header(Mode::Editing(String::from("synthetic-007.model.yml")))
        ))),
    )
}

/// The day the gallery takes as today, so every capture draws the same
/// month.
#[expect(
    clippy::expect_used,
    reason = "2026-10-21 is a day of the calendar, by inspection"
)]
fn today() -> Day {
    Day::new(2026, 10, 21).expect("the gallery's today should be a day")
}

/// A date field `id` posting as `name`, labelled `label`, holding `value`
/// and allowing `bounds`, its week from Monday.
fn date(v: &Visit, id: &str, name: &str, label: String, value: &str, bounds: Bounds) -> DateField {
    DateField {
        id: id.to_owned(),
        name: name.to_owned(),
        label,
        value: value.to_owned(),
        help: None,
        error: None,
        today: today(),
        bounds,
        first_day: Weekday::Monday,
        with_time: false,
        quick: Vec::new(),
        chosen: None,
        words: DateWords::of(&v.locale, false),
    }
}

/// The date fields: an audit's From that forbids the future, a deferral's
/// date with its quick choices, and a history filter's range.
fn dates(v: &Visit) -> Html {
    let past = Bounds {
        earliest: None,
        latest: Some(today()),
    };
    let from = DateField {
        help: Some(v.tr(Msg::DaysAfterTodayNotOffered)),
        ..date(
            v,
            "audit-from",
            "from",
            v.tr(Msg::FromDay),
            "2026-10-14",
            past,
        )
    };
    let quick = [
        (Quick::Days(1), Msg::Tomorrow),
        (Quick::Days(3), Msg::InThreeDays),
        (Quick::Next(Weekday::Monday), Msg::NextMonday),
        (Quick::Days(7), Msg::InAWeek),
    ]
    .into_iter()
    .map(|(quick, words)| QuickChoice {
        quick,
        words: v.tr(words),
    })
    .collect();
    let defer = DateField {
        quick,
        chosen: Some(Phrase::new(
            v.tr_with(Msg::BackInQueueOn, &[("day", Phrase::open("day"))]),
        )),
        ..date(
            v,
            "defer-until",
            "until",
            v.tr(Msg::DeferUntil),
            "2026-10-28",
            Bounds {
                earliest: today().step(1),
                latest: None,
            },
        )
    };
    let range = DateRange {
        id: String::from("audit-range"),
        label: v.tr(Msg::ActionsBetween),
        from: RangeEnd {
            id: String::from("audit-range-from"),
            name: String::from("from"),
            label: v.tr(Msg::FromDay),
            value: String::from("2026-10-15"),
        },
        to: RangeEnd {
            id: String::from("audit-range-to"),
            name: String::from("to"),
            label: v.tr(Msg::ToDay),
            value: String::from("2026-10-21"),
        },
        today: today(),
        bounds: past,
        first_day: Weekday::Monday,
        presets: [
            Preset::Today,
            Preset::LastDays(7),
            Preset::LastDays(30),
            Preset::ThisMonth,
        ]
        .into_iter()
        .map(|preset| RangePreset::of(&v.locale, preset))
        .collect(),
        words: DateWords::of(&v.locale, false).counted(&v.locale),
    };
    card(
        v.tr(Msg::DateFields),
        h(format!(
            "<div class=\"{FORM}\">{}{}<form method=\"get\" action=\"#date-range\" id=\"date-range\" class=\"{FIELD_FLOW}\">{}<button type=\"submit\" class=\"{}\">{}</button></form></div>",
            date_field(from),
            date_field(defer),
            date_range(range),
            Variant::Secondary.class(),
            t(v.tr(Msg::ShowActions)),
        )),
    )
}

/// Fields with a part inside their edge: a unit, a prefix, a refused value,
/// a disabled one and a masked identifier with its hold-to-show eye (R10).
fn edge_parts(v: &Visit) -> Html {
    let timeout = h(format!(
        "<input id=\"edge-timeout\" name=\"timeout\" class=\"{INPUT}\" inputmode=\"numeric\" value=\"1200\" aria-describedby=\"{}\">",
        described_with_unit("edge-timeout", false, false)
    ));
    let base = h(format!(
        "<input id=\"edge-base\" name=\"base\" class=\"{INPUT}\" spellcheck=\"false\" value=\"cdr-a.example.org\">"
    ));
    let limit = h(format!(
        "<input id=\"edge-limit\" name=\"limit\" class=\"{INPUT}\" inputmode=\"numeric\" value=\"fifty\" aria-invalid=\"true\" aria-describedby=\"{}\">",
        described_with_unit("edge-limit", false, true)
    ));
    let retention = h(format!(
        "<input id=\"edge-retention\" name=\"retention\" class=\"{INPUT}\" inputmode=\"numeric\" value=\"30\" disabled aria-describedby=\"{}\">",
        described_with_unit("edge-retention", true, false)
    ));
    card(
        v.tr(Msg::FieldEdgeParts),
        h(format!(
            "<div class=\"{FORM}\">{}{}{}{}{}</div>",
            Field::new("edge-timeout", v.tr(Msg::Timeout))
                .trailing(Inner::Unit(v.tr(Msg::Milliseconds)))
                .view(timeout),
            Field::new("edge-base", v.tr(Msg::BaseAddress))
                .leading(Inner::Text(String::from("https://")))
                .view(base),
            Field::new("edge-limit", v.tr(Msg::RowLimit))
                .trailing(Inner::Unit(v.tr(Msg::RowsUnit)))
                .after(h(format!(
                    "<p id=\"{}\" class=\"{ERROR}\">{}</p>",
                    error_id("edge-limit"),
                    t(v.tr(Msg::TypeWholeNumber))
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
    let input = h(format!(
        "<input id=\"{id}\" name=\"endpoint[{endpoint}][profile][{key}]\" class=\"{INPUT}\" spellcheck=\"false\" value=\"{}\">",
        t(url)
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
    let fields = h(format!(
        "<div class=\"{FIELD_FLOW}\">{}{}</div>{}",
        Field::new(id.clone(), v.tr(Msg::EpId)).view(h(format!(
            "<input id=\"{id}\" name=\"endpoint[{key}][id]\" class=\"{INPUT}\" spellcheck=\"false\" value=\"{}\">",
            t(name)
        ))),
        Field::new(base_id.clone(), v.tr(Msg::EpBase)).view(h(format!(
            "<input id=\"{base_id}\" name=\"endpoint[{key}][base]\" class=\"{INPUT}\" spellcheck=\"false\" value=\"{}\">",
            t(base)
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
fn endpoints(v: &Visit) -> Html {
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
        h(format!(
            "<form method=\"post\" action=\"#\" class=\"{FORM}\">{list}</form>"
        )),
    )
}

/// The forms page, with the answer to `posted` when the page was posted to.
fn drawn(v: &Visit, posted: Option<&Posted>) -> Page {
    let header = page_header(
        &v.locale,
        v.tr(Msg::FormsEditor),
        Some(v.tr(Msg::FieldsGroupedFieldsUploadField)),
        ActionGroup::none(),
    );
    let pair = grid(Html::join([
        fields(v),
        stack(Html::join([grouped(v), federated(v)])),
    ]));
    // Cards of unlike length side by side, each at its own height.
    let unlike = grid_start(Html::join([long_label(v), nested_reveal(v)]));
    let dated = grid_start(Html::join([dates(v), edge_parts(v)]));
    let shaped = grid_start(Html::join([shapes(v), pick_or_typed(v)]));
    let body = stack(Html::join([
        pair,
        dated,
        endpoints(v),
        editor_headers(v),
        inline(v),
        per_row(v),
        choosing(v),
        unlike,
        grid_start(Html::join([
            answer::answered_action(v, posted),
            answer::answered_in_place(v, posted),
        ])),
        shaped,
        upload(v),
        editor(v),
        mapping_editor(v),
        new_mapping(v),
        diagnostics_of_files(v),
        diagnostics_real_size(v),
        filter(v),
        super::records::identifiers_section(v),
        super::dock::long_form(v),
    ]));
    let dock = super::dock::dock(v).unwrap_or_default();
    Page::shell(v.tr(Msg::FormsEditor), Html::join([header, body, dock]))
}

/// The forms page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    drawn(v, None)
}

/// The forms page drawn again with the answer to the form body `body`.
#[must_use]
pub fn posted(v: &Visit, body: &str) -> Page {
    drawn(v, Some(&Posted::read(body)))
}
