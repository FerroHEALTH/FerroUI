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
    /// Whether it compiled.
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
