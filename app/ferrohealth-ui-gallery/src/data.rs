// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The synthetic rows and texts the gallery shows.
//!
//! They are invented for it and the same on every render, so the server pass
//! and the modules agree and every screenshot is reproducible. Nothing here is
//! clinical data.

/// One synthetic row of the gallery's tables: a mapping file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The file's name.
    pub name: String,
    /// What it is.
    pub kind: &'static str,
    /// Its size in lines.
    pub lines: u32,
    /// Whether it compiled, or whether the set leaves it out (`excluded`).
    pub state: &'static str,
    /// When it last changed, as a server writes an instant: RFC 3339 with
    /// microseconds and an offset.
    pub changed: String,
}

/// The number of synthetic rows.
pub const ROWS: u32 = 137;

/// The synthetic rows, in their natural order.
#[must_use]
pub fn rows() -> Vec<Row> {
    (1..=ROWS)
        .map(|index| {
            let kind = match index % 3 {
                0 => "context",
                1 => "model",
                _ => "extension",
            };
            let state = if index % 11 == 0 {
                "refused"
            } else if index % 13 == 0 {
                "excluded"
            } else if index % 7 == 0 {
                "warning"
            } else {
                "compiled"
            };
            Row {
                name: format!("synthetic-{index:03}.{kind}.yml"),
                kind,
                lines: index.wrapping_mul(37) % 400 + 12,
                state,
                changed: changed(index),
            }
        })
        .collect()
}

/// The synthetic instant the row `index` last changed, most in UTC and every
/// fourth two hours ahead of it, so the reading form moves it back.
fn changed(index: u32) -> String {
    let day = index % 28 + 1;
    let hour = index * 7 % 24;
    let minute = index * 13 % 60;
    let second = index * 29 % 60;
    let micros = index * 39_437 % 1_000_000;
    let offset = if index.is_multiple_of(4) {
        "+02:00"
    } else {
        "+00:00"
    };
    format!("2026-09-{day:02}T{hour:02}:{minute:02}:{second:02}.{micros:06}{offset}")
}

/// The keys of the columns the gallery's table sorts by.
pub const SORTABLE: [&str; 2] = ["name", "lines"];

/// `rows` sorted by the column `column`, descending when `descending`;
/// the sort is stable.
#[must_use]
pub fn sorted(mut rows: Vec<Row>, column: &str, descending: bool) -> Vec<Row> {
    match column {
        "lines" => rows.sort_by_key(|row| row.lines),
        _ => rows.sort_by(|a, b| a.name.cmp(&b.name)),
    }
    if descending {
        rows.reverse();
    }
    rows
}

/// A synthetic `FHIRconnect` context, the YAML the editor and the code panel
/// show.
pub const YAML: &str = "grammar: FHIRConnect/v1.0.0
type: context
metadata:
  name: gallery.context   # a synthetic mapping
  version: 1.0.0
spec:
  system: FHIR
  version: R4
context:
  profile:
    url: \"http://example.org/fhir/StructureDefinition/gallery\"
  template:
    id: \"gallery.template.v1\"
  archetypes:
    - \"gallery_archetype\"
  start: &start \"gallery_archetype\"
  retries: 3
  enabled: true
  fallback: *start
";

/// The path of the synthetic mapping file the code viewer shows.
pub const MAPPING_PATH: &str = "fhirconnect/gallery.context.yml";

/// The synthetic mapping file the code viewer shows: the context with one
/// line long enough to scroll sideways.
pub const MAPPING: &str = "grammar: FHIRConnect/v1.0.0
type: context
metadata:
  name: gallery.context   # a synthetic mapping
  version: 1.0.0
  description: \"A synthetic context whose one long line runs past the right edge of the viewer, so the box scrolls sideways under its cue and never wraps inside a token.\"
spec:
  system: FHIR
  version: R4
context:
  profile:
    url: \"http://example.org/fhir/StructureDefinition/gallery\"
  template:
    id: \"gallery.template.v1\"
  archetypes:
    - \"gallery_archetype\"
  start: &start \"gallery_archetype\"
  retries: 3
  enabled: true
  fallback: *start
";

/// The short digest of [`MAPPING`], as a viewer's head shows it.
pub const MAPPING_DIGEST: &str = "SHA-256 5712e3f7…f2e0e3fa";

/// The language tag of a mapping file.
pub const FHIRCONNECT: &str = "FHIRconnect";

/// The codes of the two synthetic findings on [`MAPPING`].
pub const FINDING_CODES: [&str; 2] = ["FC-201", "FC-114"];

/// The path of the synthetic draft the code editor holds.
pub const DRAFT_PATH: &str = "fhirconnect/gallery-weight.context.yml";

/// The synthetic draft as it was opened: a note that holds markup, which a
/// form posts as typed, and one line long enough to scroll sideways.
pub const DRAFT_OPENED: &str = "grammar: FHIRConnect/v1.0.0
type: context
metadata:
  name: gallery-weight.context
  version: 1.0.0
  note: \"Kept as typed: </textarea> & <b> stay text.\"
spec:
  system: FHIR
  version: R4
context:
  profile:
    url: \"http://example.org/fhir/StructureDefinition/gallery-weight-with-a-canonical-long-enough-to-run-past-the-editor\"
  template:
    id: \"gallery.template.v1\"
  start: \"gallery_archetype\"
  retries: 3
";

/// The synthetic draft the code editor holds: [`DRAFT_OPENED`] with its
/// retries changed into words, which a check refuses.
pub const DRAFT_TEXT: &str = "grammar: FHIRConnect/v1.0.0
type: context
metadata:
  name: gallery-weight.context
  version: 1.0.0
  note: \"Kept as typed: </textarea> & <b> stay text.\"
spec:
  system: FHIR
  version: R4
context:
  profile:
    url: \"http://example.org/fhir/StructureDefinition/gallery-weight-with-a-canonical-long-enough-to-run-past-the-editor\"
  template:
    id: \"gallery.template.v1\"
  start: \"gallery_archetype\"
  retries: three
";

/// The draft's findings: its code, line and column.
pub const DRAFT_FINDINGS: [(&str, u32, u32); 2] = [("FC-114", 6, 3), ("FC-201", 16, 12)];

/// The mapping the draft's findings belong to.
pub const DRAFT_MAPPING: &str = "gallery_weight";

/// The code of a finding about the whole of the draft.
pub const WHOLE_FINDING: &str = "FC-100";

/// The findings at real size: a template whose path runs past a phone,
/// the code of its one finding with the link to that finding's line, and
/// the long name of the mapping it belongs to.
pub const LONG_FINDING: [&str; 4] = [
    "openehr/gallery-vital-signs-encounter-with-a-name-long-enough-to-run-past-a-phone.v1.opt",
    "FC-301",
    "#template-L",
    "gallery_body_temperature_rectal_with_a_mapping_name_long_enough_to_break",
];

/// A file the check found clean, with the link to its lines.
pub const CLEAN_FILE: [&str; 2] = ["fhirconnect/gallery-clean.context.yml", "#clean-L"];

/// The mapping the findings on [`MAPPING`] belong to.
pub const CONTEXT_MAPPING: &str = "gallery_context";

/// The file names the four language viewers carry in their heads.
pub const FILE_NAMES: [&str; 4] = [
    "fhirconnect/gallery.context.yml",
    "fhir/Observation/gallery-1.json",
    "openehr/gallery.composition.xml",
    "aql/gallery.query.aql",
];

/// A synthetic FHIR resource, the JSON the code panel shows.
pub const JSON: &str = r#"{
  "resourceType": "Observation",
  "id": "gallery-1",
  "status": "final",
  "code": {
    "text": "A synthetic measurement"
  },
  "valueQuantity": {
    "value": 72.5,
    "unit": "beats/min"
  },
  "issued": null,
  "derived": false
}"#;

/// A synthetic canonical XML fragment, the XML the code panel shows.
pub const XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!-- a synthetic composition fragment -->
<composition xmlns="http://schemas.openehr.org/v1" archetype_node_id="openEHR-EHR-COMPOSITION.gallery.v1">
  <name>
    <value>Gallery</value>
  </name>
  <language terminology_id="ISO_639-1" code_string="en"/>
</composition>"#;

/// A synthetic AQL query, the AQL the code panel shows.
pub const AQL: &str =
    "SELECT c/uid/value, o/data[at0001]/events[at0006]/data/items[at0004]/value/magnitude AS rate
FROM EHR e[ehr_id/value = $ehr_id] CONTAINS COMPOSITION c[openEHR-EHR-COMPOSITION.gallery.v1]
  CONTAINS OBSERVATION o[openEHR-EHR-OBSERVATION.gallery.v1]
-- the newest first
WHERE c/context/start_time > '2026-01-01' AND rate >= 40
ORDER BY c/context/start_time DESC
LIMIT 25";

/// The keys that press an operation's Run.
pub const RUN_KEYS: &str = "Ctrl Enter";

/// The operation a transform pair's Run calls, its tooltip.
pub const RUN_OPERATION: &str = "$tofhir";

/// The format tags of a transform pair's input and answer.
pub const PAIR_TAGS: [&str; 2] = ["FLAT", "FHIR R4"];

/// A synthetic composition in the FLAT format, a transform pair's input.
pub const FLAT: &str = r#"{
  "ctx/language": "en",
  "ctx/territory": "NL",
  "ctx/composer_name": "Synthetic Clinician",
  "vitals/body_weight/any_event:0/time": "2026-10-20T08:15:00+02:00",
  "vitals/body_weight/any_event:0/weight|magnitude": 72.4,
  "vitals/body_weight/any_event:0/weight|unit": "kg"
}"#;

/// The request a transform pair's answer came from.
pub const PAIR_REQUEST: &str =
    "POST https://bridge.example.org/fhir/$tofhir?templateId=synthetic.vital_signs.v1&format=FLAT";

/// The EHR id of an options row's id field: a whole uuid.
pub const EHR_ID: &str = "7d44b88c-4199-4bad-97dc-d78268e01398";

/// The templates an options row offers.
pub const TEMPLATES: [&str; 2] = ["synthetic.vital_signs.v1", "synthetic.lab_report.v1"];

/// A synthetic HL7 v2 message, one segment per line, the message viewer's
/// input.
pub const HL7_MESSAGE: &str = "MSH|^~\\&|GALLERY|SYNTHETIC|FERROBRIDGE|SYNTHETIC|20261020081500||ORU^R01^ORU_R01|MSG-0001|P|2.5.1
PID|1||SYN-0042^^^GALLERY^MR||Synthetic^Example||19800101|U
OBR|1||ORD-0001|29463-7^Body weight^LN|||20261020081500
OBX|1|NM|29463-7^Body weight^LN||72.4|kg^kilogram^UCUM|||||F";

/// The version tag of the OMOP row preview.
pub const OMOP_VERSION: &str = "OMOP CDM v5.4";

/// The version tag of [`HL7_MESSAGE`].
pub const HL7_VERSION: &str = "v2.5.1";

/// One parsed segment: its id, its name, and its fields by position, name
/// and value.
pub type Hl7Parsed = (
    &'static str,
    &'static str,
    &'static [(&'static str, &'static str, &'static str)],
);

/// [`HL7_MESSAGE`] as read, segment by segment.
pub const HL7_SEGMENTS: [Hl7Parsed; 4] = [
    (
        "MSH",
        "Message header",
        &[
            ("MSH-3", "Sending application", "GALLERY"),
            ("MSH-4", "Sending facility", "SYNTHETIC"),
            ("MSH-7", "Date/time of message", "20261020081500"),
            ("MSH-9", "Message type", "ORU^R01^ORU_R01"),
            ("MSH-10", "Message control id", "MSG-0001"),
            ("MSH-12", "Version id", "2.5.1"),
        ],
    ),
    (
        "PID",
        "Patient identification",
        &[
            ("PID-3", "Patient identifier list", "SYN-0042^^^GALLERY^MR"),
            ("PID-5", "Patient name", "Synthetic^Example"),
            ("PID-8", "Administrative sex", "U"),
        ],
    ),
    (
        "OBR",
        "Observation request",
        &[
            ("OBR-3", "Filler order number", "ORD-0001"),
            (
                "OBR-4",
                "Universal service identifier",
                "29463-7^Body weight^LN",
            ),
            ("OBR-7", "Observation date/time", "20261020081500"),
        ],
    ),
    (
        "OBX",
        "Observation/result",
        &[
            ("OBX-2", "Value type", "NM"),
            ("OBX-3", "Observation identifier", "29463-7^Body weight^LN"),
            ("OBX-5", "Observation value", "72.4"),
            ("OBX-6", "Units", "kg^kilogram^UCUM"),
            ("OBX-11", "Observation result status", "F"),
        ],
    ),
];

/// One previewed table: its name, the mapping that writes it, and its rows,
/// each its columns and values, `None` for a concept that did not resolve.
pub type OmopTable = (
    &'static str,
    &'static str,
    &'static [&'static [(&'static str, Option<&'static str>)]],
);

/// The rows the synthetic composition becomes, table by table.
pub const OMOP_TABLES: [OmopTable; 2] = [
    (
        "MEASUREMENT",
        "synthetic_measurement",
        &[
            &[
                ("measurement_concept_id", Some("3025315 · Body weight")),
                ("value_as_number", Some("72.4")),
                ("unit_concept_id", Some("9529 · kilogram")),
                ("measurement_date", Some("2026-10-20")),
                (
                    "person_source_value",
                    Some("0199c2a4-77e1-7c3a-9b1e-5d0f4c2e8a17"),
                ),
            ],
            &[
                ("measurement_concept_id", Some("3027018 · Heart rate")),
                ("value_as_number", Some("64")),
                ("unit_concept_id", Some("8541 · per minute")),
                ("measurement_date", Some("2026-10-20")),
                (
                    "person_source_value",
                    Some("0199c2a4-77e1-7c3a-9b1e-5d0f4c2e8a17"),
                ),
            ],
        ],
    ),
    (
        "OBSERVATION",
        "synthetic_observation",
        &[
            &[
                ("observation_concept_id", None),
                ("value_as_concept_id", None),
                ("observation_date", Some("2026-10-20")),
            ],
            &[
                (
                    "observation_concept_id",
                    Some("4275495 · Synthetic finding"),
                ),
                ("value_as_concept_id", None),
                ("observation_date", Some("2026-10-20")),
            ],
        ],
    ),
];

#[cfg(test)]
mod tests {
    use super::{ROWS, rows, sorted};

    #[test]
    fn the_rows_are_the_same_on_every_call_and_sort_both_ways() {
        assert_eq!(rows(), rows(), "deterministic");
        assert_eq!(Some(ROWS), u32::try_from(rows().len()).ok());
        let by_lines = sorted(rows(), "lines", false);
        assert!(
            by_lines
                .windows(2)
                .all(|pair| pair[0].lines <= pair[1].lines),
            "ascending"
        );
        let down = sorted(rows(), "name", true);
        assert_eq!(
            Some("synthetic-137.extension.yml"),
            down.first().map(|row| row.name.as_str())
        );
    }
}

/// One synthetic location of the page templates' list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// Its name.
    pub name: String,
    /// Its id, drawn as code.
    pub id: String,
    /// How many units it holds.
    pub units: u32,
    /// Its state: `ok`, `warn` or `muted`.
    pub state: &'static str,
}

/// The five locations of the short list, as the reference template draws
/// them.
#[must_use]
pub fn locations() -> Vec<Location> {
    [
        ("Locatie Noord", "loc-n", 3, "ok"),
        ("Locatie Oost", "loc-o", 1, "warn"),
        ("Locatie Zuid", "loc-z", 2, "ok"),
        ("Locatie West", "loc-w", 1, "ok"),
        ("Locatie Midden", "loc-m", 1, "muted"),
    ]
    .into_iter()
    .map(|(name, id, units, state)| Location {
        name: name.to_owned(),
        id: id.to_owned(),
        units,
        state,
    })
    .collect()
}

/// The thirty locations of the long list, its first page of 25 shown.
#[must_use]
pub fn many_locations() -> Vec<Location> {
    (1..=30)
        .map(|index: u32| Location {
            name: format!("Locatie {index:02}"),
            id: format!("loc-{index:02}"),
            units: 1 + index % 3,
            state: match index {
                4 => "warn",
                9 => "muted",
                _ => "ok",
            },
        })
        .collect()
}

/// The self-description the About template shows.
pub const SELF_DESCRIPTION: &str = r#"{
  "name": "example",
  "version": "0.1.0",
  "limits": { "timeout_ms": 2500, "retries": 2 },
  "best_effort": true
}
"#;

/// The synthetic request id of the fault page's detail.
pub const REQUEST_ID: &str = "0199c2a4-77e1-7c3a-9b1e-5d0f4c2e8a17";

/// The synthetic time of the fault page's detail.
pub const FAULT_TIME: &str = "2026-10-21 09:42:11 UTC";

/// The server row of the About template's Versions card.
pub const EXAMPLE_SERVER: &str = "Example 0.1.0";

/// The library row's name on the About template.
pub const LIBRARY: &str = "FerroUI";

/// The library and its version, as the About template's library row says.
#[must_use]
pub fn library_release(version: &str) -> String {
    format!("ferrohealth-ui {version}")
}

/// The maker's support address, as the About template shows it.
pub const SUPPORT: &str = "info@cadasto.com";

/// The link to the maker's support address.
pub const SUPPORT_HREF: &str = "mailto:info@cadasto.com";

/// The file a file-only configuration is read from.
pub const CONFIG_FILE: &str = "example.toml";

/// The secret every set state of the configuration page's secret fields
/// reads from.
pub const SECRET_FILE: &str = "/run/secrets/gallery/node-b.secret";

/// The secret of the configuration page's unreadable secret field.
pub const BROKEN_FILE: &str = "/run/secrets/gallery/node-c-client.pem";

/// The path the configuration page's file choice holds.
pub const NEW_SECRET_FILE: &str = "/run/secrets/gallery/node-a-client-2026-11.pem";

/// The read-only secrets of a console in file mode: each setting's
/// configuration key and the file it is read from, when it names one.
pub const FILE_SECRETS: [(&str, Option<&str>); 4] = [
    ("db.url_file", Some("/run/secrets/gallery/db-url")),
    ("events.url", None),
    ("auth.oidc.client_secret_file", None),
    ("session.secret_file", Some("/run/secrets/gallery/session")),
];

/// The deadline of the configuration page's revision on trial, the trial's
/// length and the seconds left as the page is drawn.
pub const TRIAL: (&str, u32, u32) = ("10:14:42", 120, 102);

/// The mapping files of the provenance table: the file, the template it
/// fills and whether a provisioned directory owns it.
pub const MAPPING_FILES: [(&str, &str, bool); 3] = [
    (
        "observation.yml",
        "openEHR-EHR-OBSERVATION.blood_pressure.v2",
        true,
    ),
    (
        "vital-signs.yml",
        "openEHR-EHR-COMPOSITION.encounter.v1",
        true,
    ),
    (
        "allergy-draft.yml",
        "openEHR-EHR-EVALUATION.adverse_reaction_risk.v2",
        false,
    ),
];

/// The operator scope the 403 page names.
pub const OPERATOR_SCOPE: &str = "__example.operator";

/// The id of the location the detail template shows.
pub const LOCATION_ID: &str = "loc-n";

/// What the top bar's search holds while it runs.
pub const SEARCHED_BUSY: &str = "synthetic-007";

/// The gallery's sample query: nine mapping files hold it, more than the
/// panel shows of one kind.
pub const SEARCHED: &str = "synthetic-00";

/// A query the search finds nothing for.
pub const SEARCHED_NONE: &str = "zz-unknown";

/// The operational template the search finds, and its version.
pub const TEMPLATE: (&str, &str) = ("synthetic-vital-signs", "v1.0.2");

/// The path of a new mapping file the forms page's docked bar names.
pub const NEW_PATH: &str = "fhirconnect/synthetic-night.context.yml";

/// The synthetic members the overlays page's row menus remove.
pub const MEMBERS: [&str; 2] = ["node-b", "node-c"];

/// The synthetic set whose name the overlays page's More alone menu copies.
pub const SET_NAME: &str = "synthetic-terminology-set";
