// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page: the fields, the checkbox row, the highlighted editor with
//! diagnostics that link to its lines, and the filter over a list.

use crate::locale::{Msg, message, tr, tr_with};
use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::form::{
    ERROR, FORM, Field, INPUT, MEDIUM, SELECT, SWITCH, TEXTAREA, action_field, check, check_field,
    chip_choices, columns, described_by, error_id, field, field_flow, help_id, keyed_check_field,
    keyed_field,
};
use ferrohealth_ui::view::layout::action_bar;
use ferrohealth_ui::view::layout::{FieldRow, field_row};
use ferrohealth_ui::view::notice::{Announce, notice};
use ferrohealth_ui::view::segmented::choice_reveal;
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{
    Crumb, Header, Mode, card, editor_heading, grid, grid_start, page_header, stack, well,
};
use ferrohealth_ui::widget::combobox::{
    ComboOption, Combobox, KeyedCombobox, combobox, keyed_combobox,
};
use ferrohealth_ui::widget::editor::{Editor, LineMark, MarkKind, code_editor, line_id};
use ferrohealth_ui::widget::filter::{keeps, table_filter};
use ferrohealth_ui::widget::pick_or_type::{PickOrType, Picker, pick_or_type};
use ferrohealth_ui::widget::upload::{Upload, upload_field};
use leptos::prelude::*;

/// A query form: the nodes as checkbox chips, an option as a switch, and the
/// actions in the bar at the card's foot.
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
                {action_bar(view! {
                    <button type="button" class=Variant::Primary.class()>
                        {label(Some(Glyph::PLAY), tr(Msg::RunQuery))}
                    </button>
                    <button type="button" class=Variant::Secondary.class()>
                        {label(Some(Glyph::X), tr(Msg::Clear))}
                    </button>
                })}
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
    let version_typing = RwSignal::new(false);
    let select = view! {
        <select
            id="pick-version-choice"
            class=SELECT
            aria-describedby=help_id("pick-version")
            prop:value=move || if version_typing.get() { ANOTHER } else { "" }
            on:change=move |event| version_typing.set(event_target_value(&event) == ANOTHER)
        >
            <option value="">{tr(Msg::OneServerResolves)}</option>
            <option value="2.81">"2.81"</option>
            <option value="2.80">"2.80"</option>
            <option value=ANOTHER>{tr(Msg::AnotherVersion)}</option>
        </select>
    };
    let version = pick_or_type(PickOrType {
        label: tr(Msg::Version),
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
                    {tr(Msg::VersionServerHoldsAnotherOne)}
                </p>
                <button
                    type="button"
                    id="pick-version-back"
                    class=Variant::Ghost.small()
                    hidden=move || !version_typing.get()
                    on:click=move |_| version_typing.set(false)
                >
                    {label(Some(Glyph::ARROW_LEFT), tr(Msg::ChooseFromList))}
                </button>
            }
            .into_any(),
        ),
    });
    let canonical_typing = RwSignal::new(false);
    let picked = RwSignal::new(None::<String>);
    Effect::new(move |_| {
        if picked.get().as_deref() == Some(ANOTHER) {
            canonical_typing.set(true);
        }
    });
    let offered = vec![
        ComboOption::new("http://loinc.org", tr(Msg::Loinc)).detail("http://loinc.org"),
        ComboOption::new("http://snomed.info/sct", tr(Msg::SnomedCt))
            .detail("http://snomed.info/sct"),
        ComboOption::new(ANOTHER, tr(Msg::AnotherCanonical)),
    ];
    let canonical = pick_or_type(PickOrType {
        label: tr(Msg::CodeSystem),
        picker: Picker::Labelled(keyed_combobox(KeyedCombobox {
            id: String::from("pick-canonical-choice"),
            name: String::from("picked"),
            label: tr(Msg::CodeSystem),
            placeholder: tr(Msg::ChooseOneServerHolds),
            options: offered,
            selected: picked,
            described_by: Some(help_id("pick-canonical")),
        })),
        typed_id: String::from("pick-canonical"),
        typed: view! {
            <input id="pick-canonical" name="canonical" type="url" class=INPUT placeholder="http://example.org/fhir/CodeSystem/synthetic" aria-describedby=help_id("pick-canonical") />
        }
        .into_any(),
        typing: canonical_typing.into(),
        after: Some(
            view! {
                <p id=help_id("pick-canonical") class="fui-help">
                    {tr(Msg::CodeSystemServerHoldsCanonical)}
                </p>
            }
            .into_any(),
        ),
    });
    card(
        tr(Msg::PickType),
        view! { <form method="post" action="#" class=FORM>{version} {canonical}</form> },
    )
    .into_any()
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
    let until = view! { <input id="form-until" name="until" type="date" class=format!("{INPUT} {MEDIUM}") /> };
    let confirm = view! { <input id="form-confirm" name="confirmed" type="checkbox" /> };
    card(
        tr(Msg::Fields),
        view! {
            <form method="post" action="#" class=FORM>
                {field("form-reason", tr(Msg::WhyYouSaveOneLine), Some(tr(Msg::TwoHundredCharactersMost).clone()), reason)}
                {field("form-format", tr(Msg::CompositionFormat), None, format)}
                {field("form-bundle", tr(Msg::BundleJson), None, bundle)}
                {field("form-until", tr(Msg::KeepDraftUntil), None, until)}
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
    let at = view! { <input id="row-at" name="at" type="datetime-local" class=INPUT /> };
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
                    {field("row-at", tr(Msg::TimeTravelUtc), None, at)}
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

/// One combobox per clause, its id made from the clause's place, each
/// described by its own help line.
fn clause_pickers(systems: &[(&'static str, String, &'static str)]) -> AnyView {
    (1..=2)
        .map(|place| {
            let id = format!("clause-{place}-system");
            let help = format!("{id}-help");
            let offered = systems
                .iter()
                .map(|(value, label, url)| ComboOption::new(*value, label.clone()).detail(*url))
                .collect();
            view! {
                <div class="fui-field">
                    {keyed_combobox(KeyedCombobox {
                        id: id.clone(),
                        name: format!("clause{place}"),
                        label: tr_with(Msg::ClauseCodeSystem, &[("place", place.into())]),
                        placeholder: tr(Msg::ChooseCodeSystem),
                        options: offered,
                        selected: RwSignal::new(None),
                        described_by: Some(help.clone()),
                    })}
                    <p id=help class="fui-help">
                        {tr(Msg::SystemClauseDrawsCodesFrom)}
                    </p>
                </div>
            }
        })
        .collect_view()
        .into_any()
}

/// A long list to choose from, filtered as the reader types, beside a group
/// with its heading, a checkbox on the controls' line, and a disabled save
/// that names its reason.
fn choosing() -> AnyView {
    let systems = [
        ("loinc", tr(Msg::Loinc), "http://loinc.org"),
        ("snomed", tr(Msg::SnomedCt), "http://snomed.info/sct"),
        ("ucum", tr(Msg::Ucum), "http://unitsofmeasure.org"),
        ("icd10", tr(Msg::Icd10), "http://hl7.org/fhir/sid/icd-10"),
        ("atc", tr(Msg::Atc), "http://www.whocc.no/atc"),
        (
            "rxnorm",
            tr(Msg::Rxnorm),
            "http://www.nlm.nih.gov/research/umls/rxnorm",
        ),
        ("iso3166", tr(Msg::Iso3166Countries), "urn:iso:std:iso:3166"),
        ("bcp47", tr(Msg::Bcp47Languages), "urn:ietf:bcp:47"),
        ("mime", tr(Msg::MediaTypes), "urn:ietf:bcp:13"),
        (
            "gender",
            tr(Msg::AdministrativeGender),
            "http://hl7.org/fhir/administrative-gender",
        ),
        (
            "status",
            tr(Msg::ObservationStatus),
            "http://hl7.org/fhir/observation-status",
        ),
        (
            "synthetic",
            tr(Msg::SyntheticGalleryCodes),
            "http://example.org/fhir/CodeSystem/gallery-synthetic",
        ),
    ];
    let options = systems
        .iter()
        .map(|(value, label, url)| ComboOption::new(*value, label.clone()).detail(*url))
        .collect();
    let chosen = RwSignal::new(Some(String::from("snomed")));
    let clauses = clause_pickers(&systems);
    let code = view! { <input id="choose-code" name="code" type="text" class=INPUT placeholder="22298006" /> };
    let active = view! { <input id="choose-active" name="active" type="checkbox" checked=true /> };
    card(
        tr(Msg::Combobox),
        view! {
            <div class=FORM>
                {combobox(Combobox {
                    id: "choose-system",
                    name: "system",
                    label: tr(Msg::CodeSystem).into(),
                    placeholder: tr(Msg::ChooseCodeSystem).into(),
                    options,
                    selected: chosen,
                })}
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

/// A long form whose actions stay in the bar at the window's foot: a
/// control the keyboard reaches is scrolled clear of the bar.
fn long_form() -> AnyView {
    let fields = (1..=14_u32)
        .map(|index| {
            let id = format!("long-field-{index}");
            let input_id = id.clone();
            keyed_field(
                id,
                tr_with(Msg::SyntheticSetting, &[("index", index.into())]),
                None,
                view! { <input id=input_id type="text" class=INPUT value=format!("value-{index}") /> },
            )
        })
        .collect_view();
    card(
        tr(Msg::LongForm),
        view! {
            <form method="post" action="#" class=FORM id="long-form">
                {fields}
                {action_bar(view! {
                    <button type="button" class=Variant::Primary.class()>
                        {label(Some(Glyph::CHECK), tr(Msg::SaveSettings))}
                    </button>
                })}
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
    let source = RwSignal::new(String::new());
    card(
        tr(Msg::Upload),
        upload_field(
            Upload {
                id: "gallery-upload",
                label: tr(Msg::OperationalTemplate).into(),
                choose: tr(Msg::ChooseOptFile).into(),
                accept: &[".opt", ".xml"],
                max_bytes: 2 * 1024 * 1024,
                placeholder: "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".into(),
            },
            source,
            Signal::derive(|| false),
            Signal::derive(|| None),
        ),
    )
    .into_any()
}

/// The highlighted editor over the synthetic YAML, with two diagnostics
/// marking and linking to their lines, and the bytes it holds.
fn editor() -> AnyView {
    let text = RwSignal::new(String::from(crate::data::YAML));
    let ready = RwSignal::new(false);
    Effect::new(move |_| ready.set(true));
    let marks = Signal::derive(|| {
        vec![
            LineMark {
                line: 4,
                kind: MarkKind::Warning,
            },
            LineMark {
                line: 13,
                kind: MarkKind::Error,
            },
        ]
    });
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
    let size = move || {
        text.with(|text| {
            tr_with(
                Msg::SizeLines,
                &[
                    ("bytes", text.len().into()),
                    ("lines", text.split('\n').count().into()),
                ],
            )
        })
    };
    let diagnostics = [
        (
            4,
            Tone::Warn,
            "warning",
            tr(Msg::CommentAfterValueKeptBut),
        ),
        (
            13,
            Tone::Bad,
            "error",
            tr(Msg::TemplateGalleryTemplateV1Not),
        ),
    ]
    .into_iter()
    .map(|(line, tone, word, text)| {
        view! {
            <li>
                {badge(tone, word)}
                <span class="fui-issue-head">
                    <a href=format!("#{}", line_id(DRAFT, line))>{tr_with(Msg::IssueLine, &[("line", line.into())])}</a>
                    ":"
                </span>
                <span class="fui-issue-text">{text}</span>
            </li>
        }
    })
    .collect_view();
    card(
        tr(Msg::HighlightedEditor),
        view! {
            {Field::new(DRAFT, tr(Msg::File))
                .after(view! {
                    <p class="fui-help" role="status">
                        {size}
                    </p>
                })
                .view(field)}
            <ul class="fui-issues">{diagnostics}</ul>
        },
    )
    .into_any()
}

/// The filter over a list of synthetic file names.
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
                <p id="reloaded" class="fui-help">{tr_with(Msg::ReadTimesAgain, &[("count", count.into())])}</p>
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
    card(
        tr(Msg::EditorReloads),
        view! {
            <div class="fui-cluster">
                <button
                    id="reload-draft"
                    type="button"
                    class=Variant::Secondary.class()
                    on:click=move |_| reloads.update(|count| *count = count.saturating_add(1))
                >
                    {tr(Msg::ReadAgain)}
                </button>
            </div>
            <Transition fallback=|| ()>{drawn}</Transition>
        },
    )
    .into_any()
}

fn filter() -> AnyView {
    let words = RwSignal::new(String::new());
    let names: Vec<String> = crate::data::rows()
        .into_iter()
        .take(12)
        .map(|row| row.name)
        .collect();
    let total = names.len();
    let names = StoredValue::new(names);
    let kept = Memo::new(move |_| {
        let words = words.get();
        names.with_value(|names| {
            names
                .iter()
                .filter(|name| keeps(&words, &[name.as_str()]))
                .cloned()
                .collect::<Vec<_>>()
        })
    });
    let count = Signal::derive(move || {
        tr_with(
            Msg::FilesOnPage,
            &[
                ("kept", kept.with(Vec::len).into()),
                ("total", total.into()),
            ],
        )
    });
    card(
        tr(Msg::Filter),
        view! {
            {table_filter("gallery-filter", tr(Msg::FilterFiles), words, count)}
            <ul class="fui-divided" aria-label=tr(Msg::Files)>
                <For each=move || kept.get() key=|name| name.clone() let:name>
                    <li>
                        <code>{name}</code>
                    </li>
                </For>
            </ul>
        },
    )
    .into_any()
}

/// The header of an editor that creates a file and of one that edits a
/// file: the title and the trail's last crumb say which.
fn editor_headers() -> AnyView {
    let crumbs = || vec![Crumb::new(tr(Msg::MappingFilesCrumb), String::from("#"))];
    let header = |mode: Mode| {
        well(editor_heading(
            message(Msg::FileEditor),
            &mode,
            Header {
                crumbs: crumbs(),
                ..Header::default()
            },
            (),
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
        (),
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
    let reloading = reloading_editor();
    let long = long_form();
    view! {
        {super::title(tr(Msg::FormsEditor))}
        {header}
        {stack(view! {
            {sections.0}
            {headers}
            {sections.4}
            {sections.6}
            {sections.5}
            {unlike}
            {sections.7}
            {sections.1}
            {sections.2}
            {reloading}
            {sections.3}
            {long}
        })}
    }
}
