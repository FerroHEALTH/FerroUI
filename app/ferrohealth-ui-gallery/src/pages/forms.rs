// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page: the fields, the checkbox row, the highlighted editor with
//! diagnostics that link to its lines, and the filter over a list.

use crate::app::DRAFT_ROUTE;
use crate::data::{
    CONTEXT_MAPPING, DRAFT_FINDINGS, DRAFT_MAPPING, DRAFT_OPENED, DRAFT_PATH, DRAFT_TEXT,
    FINDING_CODES, MAPPING_PATH, WHOLE_FINDING,
};
use crate::locale::{Msg, Words, message, tr, tr_with};
use ferrohealth_ui::calendar::{Bounds, Day, Preset, Quick, Weekday};
use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::i18n::Phrase;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::island::{Combobox, DatePicker, DateRangePicker};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::code::Gravity;
use ferrohealth_ui::view::diagnostics::{
    DiagnosticsWords, FileFindings, Finding, Position, diagnostics_by_file,
};
use ferrohealth_ui::view::form::{
    ERROR, FIELD_FLOW, FORM, Field, INPUT, Inner, SELECT, SWITCH, TEXTAREA, action_field, check,
    check_field, chip_choices, columns, described_by, described_with_unit, error_id, field,
    field_flow, help_id, keyed_check_field, keyed_field,
};
use ferrohealth_ui::view::layout::{FieldRow, field_row};
use ferrohealth_ui::view::notice::{Announce, notice};
use ferrohealth_ui::view::segmented::choice_reveal;
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{
    Crumb, Header, Mode, SectionHead, card, editor_heading, grid, grid_start, page_header,
    panel_section, panel_sections_card, stack, well,
};
use ferrohealth_ui::widget::combobox::{ComboOption, ComboboxWords, KeyedCombobox, keyed_combobox};
use ferrohealth_ui::widget::date::{
    DateField, DateRange, DateWords, QuickChoice, RangeEnd, RangePreset,
};
use ferrohealth_ui::widget::editor::{
    Editor, EditorWords, LineMark, MarkKind, Opened, Panel, code_editor, editor_diagnostics,
    editor_panel, editor_split, line_id,
};
use ferrohealth_ui::widget::filter::{keeps, table_filter};
use ferrohealth_ui::widget::pick_or_type::{PickOrType, Picker, pick_or_type};
use ferrohealth_ui::widget::upload::{Upload, UploadWords, upload_field};
use leptos::prelude::*;

/// A query form: the nodes as checkbox chips, an option as a switch, and its
/// actions at its foot.
fn federated() -> AnyView {
    let nodes = ["node-a", "node-b", "node-c", "node-d"]
        .into_iter()
        .map(|node| (String::from(node), format!("synthetic-{node}")))
        .collect();
    let checked = [String::from("node-a"), String::from("node-c")];
    let partial = view! {
        <input
            id="query-partial"
            name="partial"
            type="checkbox"
            role="switch"
            class=SWITCH
            checked=true
            aria-describedby=help_id("query-partial")
        />
    };
    card(
        tr(Msg::FederatedQuery),
        view! {
            <form method="post" action="#" class=FORM>
                {chip_choices("nodes", tr(Msg::AskTheseNodes), nodes, &checked)}
                {check_field(
                    "query-partial",
                    tr(Msg::AcceptPartialAnswer),
                    tr(Msg::WhenNodeDoesNotAnswer),
                    partial,
                )}
                <div class="fui-cluster">
                    <button type="button" class=Variant::Primary.class()>
                        {label(Some(Glyph::PLAY), tr(Msg::RunQuery))}
                    </button>
                    <button type="button" class=Variant::Secondary.class()>
                        {label(Some(Glyph::X), tr(Msg::Clear))}
                    </button>
                </div>
            </form>
        },
    )
    .into_any()
}

/// A field whose label is made at run time, one whose refusal shows after
/// its help, a list of values whose labels only a screen reader reads, and
/// the button that adds a value with the line its own refusal shows in.
fn shapes() -> AnyView {
    let member = "synthetic-node-a";
    let base = "shape-base";
    let base_input = view! {
        <input
            id=base
            name="base"
            type="url"
            class=INPUT
            value="node-a.example.org"
            aria-invalid="true"
            aria-describedby=described_by(base, true, true)
        />
    };
    let base_field = Field::new(base, tr_with(Msg::BaseUrlOf, &[("member", member.into())]))
        .help(tr(Msg::WhereGatewayReachesMember))
        .after(view! {
            <p id=error_id(base) class=ERROR aria-live="polite">
                {tr(Msg::AddressMustAbsoluteStartingHttps)}
            </p>
        })
        .view(base_input);
    let rows = ["urn:example:alpha", "urn:example:beta"]
        .into_iter()
        .zip(1_u32..)
        .map(|(value, place)| {
            let id = format!("shape-value-{place}");
            let input = view! {
                <input id=id.clone() name="values" type="text" class=format!("{INPUT} fui-mono") value=value />
            };
            field_row(
                FieldRow::WithActions,
                view! {
                    {Field::new(id, tr(Msg::Value)).label_hidden().view(input)}
                    <button type="button" class=Variant::Ghost.class() aria-label=tr_with(Msg::RemoveValue, &[("value", value.into())])>
                        {label(Some(Glyph::TRASH), tr(Msg::Remove))}
                    </button>
                },
            )
        })
        .collect_view();
    let add = "shape-add";
    card(
        tr(Msg::FieldShapes),
        view! {
            <form method="post" action="#" class=FORM>
                {base_field}
                <fieldset class="fui-fieldset">
                    <legend class="fui-label">{tr(Msg::AllowedAudiences)}</legend>
                    <div class="fui-stack">{rows}</div>
                    {action_field(
                        view! {
                            <button type="button" id=add class=Variant::Secondary.class() aria-describedby=error_id(add)>
                                {label(Some(Glyph::PLUS), tr(Msg::AddValue))}
                            </button>
                        },
                        view! {
                            <p id=error_id(add) class=ERROR aria-live="polite">
                                {tr(Msg::TenValuesMost)}
                            </p>
                        },
                    )}
                </fieldset>
            </form>
        },
    )
    .into_any()
}

/// The option of a pick-or-type select that asks for the typed input.
const ANOTHER: &str = "another";

/// One value picked from a short list in a select, or from a long one in a
/// combobox, or typed behind the last option, each control with its own
/// label, and a button back to the list.
fn pick_or_typed() -> AnyView {
    let words = Words::of(&[
        Msg::OneServerResolves,
        Msg::AnotherVersion,
        Msg::Version,
        Msg::VersionServerHoldsAnotherOne,
        Msg::ChooseFromList,
        Msg::Loinc,
        Msg::SnomedCt,
        Msg::AnotherCanonical,
        Msg::CodeSystem,
        Msg::ChooseOneServerHolds,
        Msg::CodeSystemServerHoldsCanonical,
    ]);
    card(
        tr(Msg::PickType),
        view! { <crate::islands::PickOrTyped words combo=ComboboxWords::of_page() /> },
    )
    .into_any()
}

/// The interactive part of [`pick_or_typed`], hydrated as the island
/// [`crate::islands::PickOrTyped`].
pub(crate) fn pick_or_typed_body(words: &Words, combo: ComboboxWords) -> AnyView {
    let version_typing = RwSignal::new(false);
    let select = view! {
        <select
            id="pick-version-choice"
            class=SELECT
            aria-describedby=help_id("pick-version")
            prop:value=move || if version_typing.get() { ANOTHER } else { "" }
            on:change=move |event| version_typing.set(event_target_value(&event) == ANOTHER)
        >
            <option value="">{words.get(Msg::OneServerResolves)}</option>
            <option value="2.81">"2.81"</option>
            <option value="2.80">"2.80"</option>
            <option value=ANOTHER>{words.get(Msg::AnotherVersion)}</option>
        </select>
    };
    let version = pick_or_type(PickOrType {
        label: words.get(Msg::Version),
        picker: Picker::Control {
            id: String::from("pick-version-choice"),
            control: select.into_any(),
        },
        typed_id: String::from("pick-version"),
        typed: view! {
            <input id="pick-version" name="version" type="text" class=INPUT placeholder="2.79" aria-describedby=help_id("pick-version") />
        }
        .into_any(),
        typing: version_typing.into(),
        after: Some(
            view! {
                <p id=help_id("pick-version") class="fui-help">
                    {words.get(Msg::VersionServerHoldsAnotherOne)}
                </p>
                <button
                    type="button"
                    id="pick-version-back"
                    class=Variant::Ghost.small()
                    hidden=move || !version_typing.get()
                    on:click=move |_| version_typing.set(false)
                >
                    {label(Some(Glyph::ARROW_LEFT), words.get(Msg::ChooseFromList))}
                </button>
            }
            .into_any(),
        ),
    });
    let picked = RwSignal::new(None::<String>);
    let canonical_typing = Signal::derive(move || picked.read().as_deref() == Some(ANOTHER));
    let offered = vec![
        ComboOption::new("http://loinc.org", words.get(Msg::Loinc)).detail("http://loinc.org"),
        ComboOption::new("http://snomed.info/sct", words.get(Msg::SnomedCt))
            .detail("http://snomed.info/sct"),
        ComboOption::new(ANOTHER, words.get(Msg::AnotherCanonical)),
    ];
    let canonical = pick_or_type(PickOrType {
        label: words.get(Msg::CodeSystem),
        picker: Picker::Labelled(keyed_combobox(KeyedCombobox {
            id: String::from("pick-canonical-choice"),
            name: String::from("picked"),
            label: words.get(Msg::CodeSystem),
            placeholder: words.get(Msg::ChooseOneServerHolds),
            options: offered,
            selected: picked,
            described_by: Some(help_id("pick-canonical")),
            words: combo,
        })),
        typed_id: String::from("pick-canonical"),
        typed: view! {
            <input id="pick-canonical" name="canonical" type="url" class=INPUT placeholder="http://example.org/fhir/CodeSystem/synthetic" aria-describedby=help_id("pick-canonical") />
        }
        .into_any(),
        typing: canonical_typing,
        after: Some(
            view! {
                <p id=help_id("pick-canonical") class="fui-help">
                    {words.get(Msg::CodeSystemServerHoldsCanonical)}
                </p>
            }
            .into_any(),
        ),
    });
    view! { <form method="post" action="#" class=FORM>{version} {canonical}</form> }.into_any()
}

/// The id of the editor's text area.
pub const DRAFT: &str = "gallery-draft";

/// The fields, the select, the text area and the checkbox row.
fn fields() -> AnyView {
    let reason = view! {
        <input
            id="form-reason"
            name="reason"
            type="text"
            class=INPUT
            maxlength="200"
            aria-describedby=help_id("form-reason")
        />
    };
    let format = view! {
        <select id="form-format" name="format" class=SELECT>
            <option value="canonical">{tr(Msg::Canonical)}</option>
            <option value="flat">{tr(Msg::Flat)}</option>
        </select>
    };
    let bundle =
        view! { <textarea id="form-bundle" name="bundle" rows="4" class=TEXTAREA></textarea> };
    let until = view! {
        <DatePicker field=date_field("form-until", "until", tr(Msg::KeepDraftUntil), "", Bounds { earliest: today().step(1), latest: None }) />
    };
    let confirm = view! { <input id="form-confirm" name="confirmed" type="checkbox" /> };
    card(
        tr(Msg::Fields),
        view! {
            <form method="post" action="#" class=FORM>
                {field("form-reason", tr(Msg::WhyYouSaveOneLine), Some(tr(Msg::TwoHundredCharactersMost).clone()), reason)}
                {field("form-format", tr(Msg::CompositionFormat), None, format)}
                {field("form-bundle", tr(Msg::BundleJson), None, bundle)}
                {until}
                {check(
                    "form-confirm",
                    tr(Msg::SaveDraftReloadSetBridge),
                    confirm,
                )}
                <button type="button" class=Variant::Primary.class()>
                    {label(Some(Glyph::SAVE), tr(Msg::Save))}
                </button>
            </form>
        },
    )
    .into_any()
}

/// Fields in one row that share its width, a select and a date-time input
/// beside text inputs, and the button variants enabled and disabled.
fn inline() -> AnyView {
    let ehr = view! { <input id="row-ehr" name="ehr" type="text" class=INPUT placeholder=tr(Msg::EhrIdUuidBlankLets) /> };
    let subject = view! { <input id="row-subject" name="subject" type="text" class=INPUT placeholder=tr(Msg::ExternalSubjectId) /> };
    let namespace = view! { <input id="row-namespace" name="namespace" type="text" class=INPUT placeholder=tr(Msg::Namespace) /> };
    let path = view! { <input id="row-path" name="path" type="text" class=INPUT placeholder="/context/start_time/value" /> };
    let version = view! {
        <select id="row-version" name="version" class=SELECT>
            <option value="latest">{tr(Msg::Latest)}</option>
            <option value="1">{tr(Msg::Version1)}</option>
        </select>
    };
    let at = view! {
        <DatePicker field=DateField {
            with_time: true,
            words: DateWords::of_page(true),
            ..date_field("row-at", "at", tr(Msg::TimeTravelUtc), "", Bounds { earliest: None, latest: Some(today()) })
        } />
    };
    card(
        tr(Msg::FieldsRow),
        view! {
            <div class=FORM>
                {field_flow(view! {
                    {field("row-ehr", tr(Msg::EhrIdOptional), None, ehr)}
                    {field("row-subject", tr(Msg::SubjectIdOptional), None, subject)}
                    {field("row-namespace", tr(Msg::SubjectNamespaceOptional), None, namespace)}
                    <button type="button" class=Variant::Primary.class()>
                        {tr(Msg::Create)}
                    </button>
                })}
                {field_flow(view! {
                    {field("row-version", tr(Msg::Version), None, version)}
                    {at}
                    {field("row-path", tr(Msg::TargetPathOptional), None, path)}
                    <button type="button" class=Variant::Secondary.class()>
                        {tr(Msg::Time)}
                    </button>
                })}
                <div class="fui-cluster">
                    <button type="button" class=Variant::Primary.class()>{tr(Msg::Primary)}</button>
                    <button type="button" class=Variant::Secondary.class()>{tr(Msg::Secondary)}</button>
                    <button type="button" class=Variant::Danger.class()>{tr(Msg::Danger)}</button>
                    <button type="button" class=Variant::Ghost.class()>{tr(Msg::Ghost)}</button>
                    <button type="button" class=Variant::Secondary.small()>{tr(Msg::Small)}</button>
                </div>
                <div class="fui-cluster">
                    <button type="button" class=Variant::Primary.class() disabled=true>{tr(Msg::PrimaryDisabled)}</button>
                    <button type="button" class=Variant::Secondary.class() disabled=true>{tr(Msg::SecondaryDisabled)}</button>
                    <button type="button" class=Variant::Danger.class() disabled=true>{tr(Msg::DangerDisabled)}</button>
                </div>
            </div>
        },
    )
    .into_any()
}

/// A field and a switch drawn once per clause, each id made from the
/// clause's key, so every label names its own control and every help line
/// its own.
fn per_row() -> AnyView {
    let clauses = [
        ("include-1", tr(Msg::Include)),
        ("exclude-2", tr(Msg::Exclude)),
    ];
    let rows = clauses
        .into_iter()
        .map(|(key, verb)| {
            let search = format!("compose-search-{key}");
            let search_help = help_id(&search);
            let inactive = format!("compose-inactive-{key}");
            let inactive_help = help_id(&inactive);
            view! {
                <fieldset class="fui-fieldset">
                    <legend>{verb} {tr(Msg::Codes)}</legend>
                    {field_flow(view! {
                        {keyed_field(
                            search.clone(),
                            tr(Msg::FindCodeCodeSystem),
                            Some(tr(Msg::TwoLettersMore).clone()),
                            view! {
                                <input id=search name=format!("search-{key}") type="search" class=INPUT placeholder=tr(Msg::BloodPressure) aria-describedby=search_help />
                            },
                        )}
                        <button type="button" class=Variant::Secondary.class()>{tr(Msg::Add)}</button>
                    })}
                    {keyed_check_field(
                        inactive.clone(),
                        tr(Msg::IncludeInactiveCodes),
                        tr(Msg::CodesSystemRetiredStillMatch),
                        view! {
                            <input type="checkbox" role="switch" id=inactive name=format!("inactive-{key}") class=SWITCH aria-describedby=inactive_help />
                        },
                    )}
                </fieldset>
            }
        })
        .collect_view();
    card(
        tr(Msg::FieldsPerRow),
        view! { <div class=FORM>{rows}</div> },
    )
    .into_any()
}

/// One combobox per clause, the library's island,
/// its id made from the clause's place, each described by its own help
/// line.
fn clause_pickers() -> AnyView {
    (1_u8..=2)
        .map(|place| {
            let help = format!("clause-{place}-system-help");
            view! {
                <div class="fui-field">
                    {clause_picker(place)}
                    <p id=help class="fui-help">
                        {tr(Msg::SystemClauseDrawsCodesFrom)}
                    </p>
                </div>
            }
        })
        .collect_view()
        .into_any()
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

/// The words of a code system combobox's island: every system's, the
/// label's and the placeholder's.
fn system_words() -> Words {
    let msgs: Vec<Msg> = SYSTEMS
        .iter()
        .map(|(_, msg, _)| *msg)
        .chain([Msg::CodeSystem, Msg::ChooseCodeSystem])
        .collect();
    Words::of(&msgs)
}

/// The options of [`SYSTEMS`] in `words`.
fn system_options(words: &Words) -> Vec<ComboOption> {
    SYSTEMS
        .iter()
        .map(|(value, msg, url)| ComboOption::new(*value, words.get(*msg)).detail(*url))
        .collect()
}

/// The combobox of the clause at `place`, the library's island.
fn clause_picker(place: u8) -> AnyView {
    let id = format!("clause-{place}-system");
    let words = system_words();
    view! {
        <Combobox
            id=id.clone()
            name=format!("clause{place}")
            label=words.fill(Msg::ClauseCodeSystem, &[("place", &place.to_string())])
            placeholder=words.get(Msg::ChooseCodeSystem)
            options=system_options(&words)
            selected=None
            described_by=Some(format!("{id}-help"))
            words=ComboboxWords::of_page()
        />
    }
    .into_any()
}

/// The combobox of the code system, SNOMED CT chosen, the library's island.
fn system_picker() -> AnyView {
    let words = system_words();
    view! {
        <Combobox
            id=String::from("choose-system")
            name=String::from("system")
            label=words.get(Msg::CodeSystem)
            placeholder=words.get(Msg::ChooseCodeSystem)
            options=system_options(&words)
            selected=Some(String::from("snomed"))
            described_by=None
            words=ComboboxWords::of_page()
        />
    }
    .into_any()
}

/// A long list to choose from, filtered as the reader types, beside a group
/// with its heading, a checkbox on the controls' line, and a disabled save
/// that names its reason.
fn choosing() -> AnyView {
    let clauses = clause_pickers();
    let code = view! { <input id="choose-code" name="code" type="text" class=INPUT placeholder="22298006" /> };
    let active = view! { <input id="choose-active" name="active" type="checkbox" checked=true /> };
    card(
        tr(Msg::Combobox),
        view! {
            <div class=FORM>
                {system_picker()}
                <fieldset class="fui-fieldset">
                    <legend class="fui-sr-only">{tr(Msg::Clauses)}</legend>
                    {clauses}
                </fieldset>
                <fieldset class="fui-fieldset">
                    <legend>{tr(Msg::TheCode)}</legend>
                    {field_flow(view! {
                        {field("choose-code", tr(Msg::Code), None, code)}
                        {check("choose-active", tr(Msg::ActiveConceptsOnly), active)}
                    })}
                </fieldset>
                <div class="fui-cluster">
                    <button type="button" class=Variant::Primary.class() disabled=true aria-describedby="choose-save-why">
                        {label(Some(Glyph::SAVE), tr(Msg::SaveValueSet))}
                    </button>
                    <button type="button" class=Variant::Danger.small()>
                        {label(Some(Glyph::TRASH), tr(Msg::RemoveClause))}
                    </button>
                </div>
                <p id="choose-save-why" class="fui-help">
                    {tr(Msg::CanonicalClauseWhatSaveNeeds)}
                </p>
            </div>
        },
    )
    .into_any()
}

/// Related fields under a legend, in pairs, a disabled select and search
/// field inside the well, and a choice drawn as segments.
fn grouped() -> AnyView {
    let pair = columns(view! {
        {field("member-name", tr(Msg::Member), None, view! { <input id="member-name" type="text" class=INPUT value="synthetic-node-a" /> })}
        {field("member-base", tr(Msg::BaseUrl), None, view! { <input id="member-base" type="text" class=INPUT value="https://node-a.example.org/" /> })}
    });
    card(
        tr(Msg::GroupedFields),
        view! {
            <form method="post" action="#" class=FORM>
                <fieldset class="fui-fieldset fui-well">
                    <legend>{tr(Msg::FederationMember)}</legend>
                    {pair}
                    {field_flow(view! {
                        {field("member-system", tr(Msg::CodeSystem), None, view! {
                            <select id="member-system" name="system" class=SELECT disabled=true>
                                <option value="">{tr(Msg::ChooseMemberFirst)}</option>
                            </select>
                        })}
                        {field("member-code", tr(Msg::FindCodeCodeSystem), None, view! {
                            <input id="member-code" name="code" type="search" class=INPUT disabled=true placeholder=tr(Msg::ChooseCodeSystem) />
                        })}
                        <button type="button" class=Variant::Secondary.class() disabled=true>
                            {tr(Msg::Search)}
                        </button>
                    })}
                </fieldset>
                {choice_reveal(
                    "secret",
                    tr(Msg::ClientSecret),
                    &[("keep", tr(Msg::Keep)), ("replace", tr(Msg::Replace)), ("remove", tr(Msg::Remove))],
                    "keep",
                    vec![
                        ().into_any(),
                        field(
                            "member-secret",
                            tr(Msg::NewSecret),
                            None,
                            view! { <input id="member-secret" name="secret-value" type="password" class=INPUT autocomplete="new-password" /> },
                        )
                        .into_any(),
                        notice(
                            Tone::Warn,
                            Announce::Status,
                            None,
                            tr(Msg::MemberSignsNoSecretOnce),
                        )
                        .into_any(),
                    ],
                )}
            </form>
        },
    )
    .into_any()
}

/// A grant whose chosen kind reveals its settings, one of which is a secret
/// revealed by a choice of its own: each reveal answers to its own choice.
fn nested_reveal() -> AnyView {
    let secret = choice_reveal(
        "grant-secret",
        tr(Msg::ClientSecret),
        &[("keep", tr(Msg::Keep)), ("replace", tr(Msg::Replace))],
        "keep",
        vec![
            ().into_any(),
            field(
                "grant-secret-value",
                tr(Msg::NewSecret),
                None,
                view! { <input id="grant-secret-value" type="password" class=INPUT autocomplete="new-password" /> },
            )
            .into_any(),
        ],
    );
    let client = view! {
        <div class="fui-stack">
            {field("grant-client-id", tr(Msg::ClientId), None, view! { <input id="grant-client-id" type="text" class=INPUT value="synthetic-client" /> })}
            {secret}
        </div>
    }
    .into_any();
    let token = field(
        "grant-token-value",
        tr(Msg::TokenLabel),
        None,
        view! { <input id="grant-token-value" type="password" class=INPUT autocomplete="off" /> },
    )
    .into_any();
    card(
        tr(Msg::RevealInsideReveal),
        view! {
            <form method="post" action="#" class=FORM>
                {choice_reveal(
                    "grant",
                    tr(Msg::Grant),
                    &[("client", tr(Msg::ClientCredentials)), ("token", tr(Msg::Token))],
                    "client",
                    vec![client, token],
                )}
            </form>
        },
    )
    .into_any()
}

/// A button whose words are longer than a phone's column, which break
/// between words rather than push the page sideways, beside a short one.
fn long_label() -> AnyView {
    card(
        tr(Msg::LongButtonLabel),
        view! {
            <div class="fui-cluster">
                <button type="button" id="long-label" class=Variant::Primary.class()>
                    {label(Some(Glyph::DOWNLOAD), tr(Msg::DownloadWholeConfigurationToml))}
                </button>
                <button type="button" class=Variant::Secondary.class()>
                    {label(Some(Glyph::X), tr(Msg::Clear))}
                </button>
            </div>
        },
    )
    .into_any()
}

/// The upload field: a file dropped or chosen, read into the text area.
fn upload() -> AnyView {
    let words = Words::of(&[Msg::OperationalTemplate, Msg::ChooseOptFile]);
    let upload = UploadWords::of_page(UPLOAD_ACCEPT, UPLOAD_MAX_BYTES);
    card(
        tr(Msg::Upload),
        view! { <crate::islands::UploadField words upload /> },
    )
    .into_any()
}

/// The extensions the upload field takes.
const UPLOAD_ACCEPT: &[&str] = &[".opt", ".xml"];

/// The largest file the upload field reads.
const UPLOAD_MAX_BYTES: u64 = 2 * 1024 * 1024;

/// The interactive part of [`upload`], hydrated as the island
/// [`crate::islands::UploadField`].
pub(crate) fn upload_body(words: &Words, upload: UploadWords) -> AnyView {
    let source = RwSignal::new(String::new());
    upload_field(
        Upload {
            id: "gallery-upload",
            label: words.get(Msg::OperationalTemplate).into(),
            choose: words.get(Msg::ChooseOptFile).into(),
            accept: UPLOAD_ACCEPT,
            max_bytes: UPLOAD_MAX_BYTES,
            placeholder: "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".into(),
            words: upload,
        },
        source,
        Signal::stored(false),
        Signal::stored(None),
    )
    .into_any()
}

/// The highlighted editor over the synthetic YAML, with two diagnostics
/// marking and linking to their lines, and the bytes it holds.
fn editor() -> AnyView {
    card(
        tr(Msg::HighlightedEditor),
        view! {
            <crate::islands::DraftEditor words=Words::of(&[
                Msg::CommentAfterValueKeptBut,
                Msg::TemplateGalleryTemplateV1Not,
                Msg::File,
            ])
            .counted(Msg::SizeLines, &["bytes", "lines"])
            .open(Msg::IssueLine, &["line"]) />
        },
    )
    .into_any()
}

/// The interactive part of [`editor`], hydrated as the island
/// [`crate::islands::DraftEditor`].
pub(crate) fn editor_body(words: Words) -> AnyView {
    let text = RwSignal::new(String::from(crate::data::YAML));
    let ready = RwSignal::new(false);
    Effect::new(move |_| ready.set(true));
    let marks = Signal::stored(vec![
        LineMark {
            line: 4,
            kind: MarkKind::Warning,
        },
        LineMark {
            line: 13,
            kind: MarkKind::Error,
        },
    ]);
    let field = code_editor(
        Editor {
            id: DRAFT,
            language: Language::Yaml,
            text: text.into(),
            marks,
            editable: ready.into(),
        },
        move |typed| text.set(typed),
    );
    let words = StoredValue::new(words);
    let size = move || {
        text.with(|text| {
            let bytes = u64::try_from(text.len()).unwrap_or(u64::MAX);
            let lines = u64::try_from(text.split('\n').count()).unwrap_or(u64::MAX);
            words.with_value(|words| {
                words.count(Msg::SizeLines, &[("bytes", bytes), ("lines", lines)])
            })
        })
    };
    let diagnostics = [
        (
            4,
            Tone::Warn,
            "warning",
            words.with_value(|words| words.get(Msg::CommentAfterValueKeptBut)),
        ),
        (
            13,
            Tone::Bad,
            "error",
            words.with_value(|words| words.get(Msg::TemplateGalleryTemplateV1Not)),
        ),
    ]
    .into_iter()
    .map(|(line, tone, word, text)| {
        view! {
            <li>
                {badge(tone, word)}
                <span class="fui-issue-head">
                    <a href=format!("#{}", line_id(DRAFT, line))>{words.with_value(|words| words.fill(Msg::IssueLine, &[("line", &line.to_string())]))}</a>
                    ":"
                </span>
                <span class="fui-issue-text">{text}</span>
            </li>
        }
    })
    .collect_view();
    view! {
        {Field::new(DRAFT, words.with_value(|words| words.get(Msg::File)))
            .after(view! {
                <p class="fui-help" role="status">
                    {size}
                </p>
            })
            .view(field)}
        <ul class="fui-issues">{diagnostics}</ul>
    }
    .into_any()
}

/// The id of the code editor's text area.
pub const MAPPING_DRAFT: &str = "gallery-mapping";

/// The id of the new mapping's text area.
pub const NEW_MAPPING: &str = "gallery-new-mapping";

/// The code editor (B2) beside its diagnostics (B3), in a form that posts
/// its draft.
fn mapping_editor() -> AnyView {
    let words = Words::of(&[
        Msg::Check,
        Msg::Diagnostics,
        Msg::NoteKeptNeverRead,
        Msg::RetriesMustBeNumber,
        Msg::Utf8Lf,
        Msg::YamlFhirconnectContext,
    ])
    .open(Msg::DraftOf, &["name"]);
    let editor = EditorWords::of_page();
    let diagnostics = DiagnosticsWords::of_page();
    view! { <crate::islands::MappingEditor words editor diagnostics /> }.into_any()
}

/// The interactive part of [`mapping_editor`], hydrated as the island
/// [`crate::islands::MappingEditor`]: the draft opened with a warning and
/// an error, and the list beside it in step with the cursor.
pub(crate) fn mapping_editor_body(
    words: &Words,
    editor: EditorWords,
    diagnostics: &DiagnosticsWords,
) -> AnyView {
    let text = RwSignal::new(String::from(DRAFT_TEXT));
    let ready = RwSignal::new(false);
    Effect::new(move |_| ready.set(true));
    let caret = RwSignal::new(Position::default());
    let [(note, note_line, note_column), (retries, line, column)] = DRAFT_FINDINGS;
    let marks = Signal::stored(vec![
        LineMark {
            line: note_line,
            kind: MarkKind::Warning,
        },
        LineMark {
            line,
            kind: MarkKind::Error,
        },
    ]);
    let field = Editor {
        id: MAPPING_DRAFT,
        language: Language::Yaml,
        text: text.into(),
        marks,
        editable: ready.into(),
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
        findings: vec![
            found(
                Gravity::Warning,
                note,
                (note_line, note_column),
                words.get(Msg::NoteKeptNeverRead),
            ),
            found(
                Gravity::Error,
                retries,
                (line, column),
                words.get(Msg::RetriesMustBeNumber),
            ),
        ],
    };
    let check = view! {
        <button type="submit" class=Variant::Secondary.small()>
            {label(Some(Glyph::LIST_CHECKS), words.get(Msg::Check))}
        </button>
    };
    let panel = editor_panel(
        Panel {
            editor: field,
            label: words.fill(Msg::DraftOf, &[("name", DRAFT_PATH)]),
            lead: words.get(Msg::YamlFhirconnectContext),
            facts: vec![words.get(Msg::Utf8Lf)],
            opened: Opened::Text(String::from(DRAFT_OPENED)),
            caret,
            tools: Some(check.into_any()),
            end: None,
            words: editor,
        },
        move |typed| text.set(typed),
    );
    let title = words.get(Msg::Diagnostics);
    let list = editor_diagnostics(&title, &findings, diagnostics, field, caret);
    let side = panel_sections_card(
        title.clone(),
        panel_section(
            Some(SectionHead {
                title,
                count: Some(2),
                actions: None,
            }),
            list,
        ),
    );
    editor_split(
        view! { <form method="post" action=DRAFT_ROUTE>{panel}</form> }.into_any(),
        side,
    )
}

/// The code editor of a new file, empty, in a panel section (B10).
fn new_mapping() -> AnyView {
    let words = Words::of(&[
        Msg::NewMappingFile,
        Msg::Utf8Lf,
        Msg::YamlFhirconnectContext,
    ]);
    let editor = EditorWords::of_page();
    panel_sections_card(
        tr(Msg::NewMapping),
        panel_section(
            Some(SectionHead {
                title: tr(Msg::NewMapping),
                count: None,
                actions: None,
            }),
            view! { <crate::islands::NewMappingEditor words editor /> },
        ),
    )
}

/// The interactive part of [`new_mapping`], hydrated as the island
/// [`crate::islands::NewMappingEditor`].
pub(crate) fn new_mapping_body(words: &Words, editor: EditorWords) -> AnyView {
    let text = RwSignal::new(String::new());
    let ready = RwSignal::new(false);
    Effect::new(move |_| ready.set(true));
    editor_panel(
        Panel {
            editor: Editor {
                id: NEW_MAPPING,
                language: Language::Yaml,
                text: text.into(),
                marks: Signal::stored(Vec::new()),
                editable: ready.into(),
            },
            label: words.get(Msg::NewMappingFile),
            lead: words.get(Msg::YamlFhirconnectContext),
            facts: vec![words.get(Msg::Utf8Lf)],
            opened: Opened::New,
            caret: RwSignal::new(Position::default()),
            tools: None,
            end: None,
            words: editor,
        },
        move |typed| text.set(typed),
    )
}

/// The diagnostics list of two files, errors first, the current row on the
/// draft's error: the code page's mapping file, and the draft the code
/// editor holds.
fn diagnostics_of_files() -> AnyView {
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
            findings: vec![
                found(
                    Gravity::Warning,
                    code_start,
                    Some((20, 3)),
                    tr(Msg::FindingStartTwice),
                    CONTEXT_MAPPING,
                ),
                found(
                    Gravity::Error,
                    code_retries,
                    Some((18, 12)),
                    tr(Msg::FindingRetries),
                    CONTEXT_MAPPING,
                ),
            ],
        },
        FileFindings {
            file: String::from(DRAFT_PATH),
            href: None,
            lines: format!("#{MAPPING_DRAFT}-L"),
            current: Some(line),
            findings: vec![
                found(
                    Gravity::Warning,
                    note,
                    Some((note_line, note_column)),
                    tr(Msg::NoteKeptNeverRead),
                    DRAFT_MAPPING,
                ),
                found(
                    Gravity::Error,
                    retries,
                    Some((line, column)),
                    tr(Msg::RetriesMustBeNumber),
                    DRAFT_MAPPING,
                ),
                found(
                    Gravity::Error,
                    WHOLE_FINDING,
                    None,
                    tr(Msg::ProfileUnknown),
                    DRAFT_MAPPING,
                ),
            ],
        },
    ];
    let title = tr(Msg::DiagnosticsByFile);
    panel_sections_card(
        title.clone(),
        panel_section(
            Some(SectionHead {
                title: tr(Msg::Diagnostics),
                count: Some(5),
                actions: None,
            }),
            diagnostics_by_file(&title, &files),
        ),
    )
}

/// Waits most of a second in the browser, so a reload keeps its old view on
/// screen long enough to type into; on the server it waits not at all.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(
        clippy::unused_async,
        reason = "the pause awaits only in the browser half; the server half has nothing to wait for"
    )
)]
async fn pause() {
    #[cfg(target_arch = "wasm32")]
    {
        let promise = web_sys::js_sys::Promise::new(&mut |resolve, _reject| {
            let Some(window) = web_sys::window() else {
                return;
            };
            if window
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 800)
                .is_err()
            {
                leptos::logging::warn!("the browser refused the pause's timer");
            }
        });
        if wasm_bindgen_futures::JsFuture::from(promise).await.is_err() {
            leptos::logging::warn!("the pause's timer failed");
        }
    }
}

/// An editor drawn inside a `Transition` whose resource reloads: the old
/// editor stays mounted while the new run's owner replaces the old one, so
/// the editor must hold no reactive value of its own. Its text, marks and
/// editable state are the page's.
fn reloading_editor() -> AnyView {
    card(
        tr(Msg::EditorReloads),
        view! {
            <crate::islands::ReloadingEditor words=Words::of(&[Msg::ReadAgain])
                .counted(Msg::ReadTimesAgain, &["count"]) />
        },
    )
    .into_any()
}

/// The interactive part of [`reloading_editor`], hydrated as the island
/// [`crate::islands::ReloadingEditor`].
pub(crate) fn reloading_editor_body(words: Words) -> AnyView {
    let again = words.get(Msg::ReadAgain);
    let words = StoredValue::new(words);
    let reloads = RwSignal::new(0_u32);
    let text = RwSignal::new(String::from("{\n  \"synthetic\": true\n}\n"));
    let marks = Signal::stored(Vec::<LineMark>::new());
    let ready = RwSignal::new(false);
    Effect::new(move |_| ready.set(true));
    // A local resource: the browser's timer is not Send, and the screen
    // this stands for reads in the browser after hydration.
    let read = LocalResource::new(move || {
        let count = reloads.get();
        async move {
            pause().await;
            count
        }
    });
    let drawn = move || {
        Suspend::new(async move {
            let count = read.await;
            view! {
                <p id="reloaded" class="fui-help">{words.with_value(|words| words.count(Msg::ReadTimesAgain, &[("count", u64::from(count))]))}</p>
                {code_editor(
                    Editor {
                        id: "reloading-draft",
                        language: Language::Json,
                        text: text.into(),
                        marks,
                        editable: ready.into(),
                    },
                    move |typed| text.set(typed),
                )}
            }
        })
    };
    view! {
        <div class="fui-cluster">
            <button
                id="reload-draft"
                type="button"
                class=Variant::Secondary.class()
                on:click=move |_| reloads.update(|count| *count = count.saturating_add(1))
            >
                {again}
            </button>
        </div>
        <Transition fallback=|| ()>{drawn}</Transition>
    }
    .into_any()
}

/// The filter over a list of synthetic file names.
fn filter() -> AnyView {
    let words =
        Words::of(&[Msg::FilterFiles, Msg::Files]).open(Msg::FilesOnPage, &["kept", "total"]);
    card(
        tr(Msg::Filter),
        view! { <crate::islands::FileFilter words /> },
    )
    .into_any()
}

/// The interactive part of [`filter`], hydrated as the island
/// [`crate::islands::FileFilter`].
pub(crate) fn filter_body(words: &Words) -> AnyView {
    let typed = RwSignal::new(String::new());
    let names: Vec<String> = crate::data::rows()
        .into_iter()
        .take(12)
        .map(|row| row.name)
        .collect();
    let total = names.len();
    let names = StoredValue::new(names);
    let kept = Memo::new(move |_| {
        let words = typed.get();
        names.with_value(|names| {
            names
                .iter()
                .filter(|name| keeps(&words, &[name.as_str()]))
                .cloned()
                .collect::<Vec<_>>()
        })
    });
    let on_page = words.get(Msg::FilesOnPage);
    let total = total.to_string();
    let count = Signal::derive(move || {
        Phrase::new(on_page.clone()).fill(&[
            ("kept", &kept.with(Vec::len).to_string()),
            ("total", &total),
        ])
    });
    view! {
        {table_filter("gallery-filter", words.get(Msg::FilterFiles), typed, count)}
        <ul class="fui-divided" aria-label=words.get(Msg::Files)>
            <For each=move || kept.get() key=|name| name.clone() let:name>
                <li>
                    <code>{name}</code>
                </li>
            </For>
        </ul>
    }
    .into_any()
}

/// The header of an editor that creates a file and of one that edits a
/// file: the title says which, and the trail ends with the list a new file
/// joins or with the file edited.
fn editor_headers() -> AnyView {
    let header = |mode: Mode| {
        let mut crumbs = vec![Crumb::new(tr(Msg::MappingFilesCrumb), String::from("#"))];
        if let Mode::Editing(name) = &mode {
            crumbs.push(Crumb::new(name.clone(), String::from("#")));
        }
        well(editor_heading(
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
        tr(Msg::EditorHeaders),
        stack(view! {
            <div data-editor-mode="new">{header(Mode::Creating)}</div>
            <div data-editor-mode="edit">
                {header(Mode::Editing(String::from("synthetic-007.model.yml")))}
            </div>
        }),
    )
    .into_any()
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
fn date_field(id: &str, name: &str, label: String, value: &str, bounds: Bounds) -> DateField {
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
        words: DateWords::of_page(false),
    }
}

/// The date fields: an audit's From that forbids the future, a deferral's
/// date with its quick choices, and a history filter's range.
fn dates() -> AnyView {
    let past = Bounds {
        earliest: None,
        latest: Some(today()),
    };
    let from = DateField {
        help: Some(tr(Msg::DaysAfterTodayNotOffered)),
        ..date_field("audit-from", "from", tr(Msg::FromDay), "2026-10-14", past)
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
        words: tr(words),
    })
    .collect();
    let defer = DateField {
        quick,
        chosen: Some(Phrase::new(tr_with(
            Msg::BackInQueueOn,
            &[("day", Phrase::open("day"))],
        ))),
        ..date_field(
            "defer-until",
            "until",
            tr(Msg::DeferUntil),
            "2026-10-28",
            Bounds {
                earliest: today().step(1),
                latest: None,
            },
        )
    };
    let range = DateRange {
        id: String::from("audit-range"),
        label: tr(Msg::ActionsBetween),
        from: RangeEnd {
            id: String::from("audit-range-from"),
            name: String::from("from"),
            label: tr(Msg::FromDay),
            value: String::from("2026-10-15"),
        },
        to: RangeEnd {
            id: String::from("audit-range-to"),
            name: String::from("to"),
            label: tr(Msg::ToDay),
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
        .map(RangePreset::of_page)
        .collect(),
        words: DateWords::of_page(false),
    };
    card(
        tr(Msg::DateFields),
        view! {
            <div class=FORM>
                <DatePicker field=from />
                <DatePicker field=defer />
                <form method="get" action="#date-range" id="date-range" class=FIELD_FLOW>
                    <DateRangePicker range=range />
                    <button type="submit" class=Variant::Secondary.class()>
                        {tr(Msg::ShowActions)}
                    </button>
                </form>
            </div>
        },
    )
    .into_any()
}

/// Fields with a part inside their edge: a unit, a prefix, a refused value
/// and a disabled one (R10).
fn edge_parts() -> AnyView {
    let timeout = view! {
        <input id="edge-timeout" name="timeout" class=INPUT inputmode="numeric" value="1200" aria-describedby=described_with_unit("edge-timeout", false, false) />
    };
    let base = view! {
        <input id="edge-base" name="base" class=INPUT spellcheck="false" value="cdr-a.example.org" />
    };
    let limit = view! {
        <input id="edge-limit" name="limit" class=INPUT inputmode="numeric" value="fifty" aria-invalid="true" aria-describedby=described_with_unit("edge-limit", false, true) />
    };
    let retention = view! {
        <input id="edge-retention" name="retention" class=INPUT inputmode="numeric" value="30" disabled aria-describedby=described_with_unit("edge-retention", true, false) />
    };
    card(
        tr(Msg::FieldEdgeParts),
        view! {
            <div class=FORM>
                {Field::new("edge-timeout", tr(Msg::Timeout))
                    .trailing(Inner::Unit(tr(Msg::Milliseconds)))
                    .view(timeout)}
                {Field::new("edge-base", tr(Msg::BaseAddress))
                    .leading(Inner::Text(String::from("https://")))
                    .view(base)}
                {Field::new("edge-limit", tr(Msg::RowLimit))
                    .trailing(Inner::Unit(tr(Msg::RowsUnit)))
                    .after(view! { <p id=error_id("edge-limit") class=ERROR>{tr(Msg::TypeWholeNumber)}</p> })
                    .view(limit)}
                {Field::new("edge-retention", tr(Msg::Retention))
                    .help(tr(Msg::SetByDeployment))
                    .trailing(Inner::Unit(tr(Msg::DaysUnit)))
                    .view(retention)}
            </div>
        },
    )
    .into_any()
}

/// The forms page.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn FormsPage() -> impl IntoView {
    let header = page_header(
        tr(Msg::FormsEditor),
        Some(tr(Msg::FieldsGroupedFieldsUploadField)),
        ActionGroup::none(),
    )
    .into_any();
    let pair = grid(view! {
        {fields()}
        {stack(view! {
            {grouped()}
            {federated()}
        })}
    })
    .into_any();
    let sections = (
        pair,
        upload(),
        editor(),
        filter(),
        inline(),
        choosing(),
        per_row(),
        grid_start(view! {
            {shapes()}
            {pick_or_typed()}
        })
        .into_any(),
    );
    // Cards of unlike length side by side, each at its own height.
    let unlike = grid_start(view! {
        {long_label()}
        {nested_reveal()}
    })
    .into_any();
    let headers = editor_headers();
    let dated = grid_start(view! {
        {dates()}
        {edge_parts()}
    })
    .into_any();
    let reloading = reloading_editor();
    let mapping = mapping_editor();
    let fresh = new_mapping();
    let by_file = diagnostics_of_files();
    let long = super::dock::long_form();
    let dock = super::dock::dock();
    view! {
        {super::title(tr(Msg::FormsEditor))}
        {header}
        {stack(view! {
            {sections.0}
            {dated}
            {headers}
            {sections.4}
            {sections.6}
            {sections.5}
            {unlike}
            {sections.7}
            {sections.1}
            {sections.2}
            {mapping}
            {fresh}
            {by_file}
            {reloading}
            {sections.3}
            {super::records::identifiers_section()}
            {long}
        })}
        {dock}
    }
}
