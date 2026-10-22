// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The synthetic rows and texts the gallery shows.
//!
//! They are invented for it and the same on every render, so the server pass
//! and hydration agree and every screenshot is reproducible. Nothing here is
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

/// The operator scope the 403 page names.
pub const OPERATOR_SCOPE: &str = "__example.operator";

/// The id of the location the detail template shows.
pub const LOCATION_ID: &str = "loc-n";

/// What the top bar's search was given in each of its states the gallery
/// shows: while it runs, for an answer, and for none.
pub const SEARCHED_BUSY: &str = "synthetic-007";

/// The query the top bar's answer answers.
pub const SEARCHED: &str = "synthetic-00";

/// The query the top bar's search finds nothing for.
pub const SEARCHED_NONE: &str = "zz-unknown";

/// The operational template the top bar's answer finds, and its version.
pub const TEMPLATE: (&str, &str) = ("synthetic-vital-signs", "v1.0.2");

/// The path of a new mapping file the forms page's docked bar names.
pub const NEW_PATH: &str = "fhirconnect/synthetic-night.context.yml";
