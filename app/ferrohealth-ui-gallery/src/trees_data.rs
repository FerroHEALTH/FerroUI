// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The synthetic code systems, directory and template the trees page
//! (`pages::trees`) draws, as data. Every code, uid and path is invented.

use crate::locale::Msg;

/// One synthetic concept: its code, its display and whether it is
/// inactive.
pub type Concept = (&'static str, Msg, bool);

/// The synthetic findings of the deep browser, by code.
pub const FINDINGS: [Concept; 15] = [
    ("SF-1000", Msg::TreesFinding, false),
    ("SF-1100", Msg::TreesSkin, false),
    ("SF-1110", Msg::TreesRash, false),
    ("SF-1200", Msg::TreesChest, false),
    ("SF-1210", Msg::TreesCough, false),
    ("SF-1211", Msg::TreesDryCough, false),
    ("SF-1212", Msg::TreesProductiveCough, false),
    ("SF-1220", Msg::TreesChestPain, false),
    ("SF-1300", Msg::TreesGeneral, false),
    ("SF-1310", Msg::TreesFatigue, false),
    ("SF-1320", Msg::TreesFever, false),
    ("SF-1390", Msg::TreesMalaise, true),
    ("SF-1400", Msg::TreesAbdomen, false),
    ("SF-1410", Msg::TreesAbdominalPain, false),
    ("SF-1420", Msg::TreesNausea, false),
];

/// The parent of each finding below the top.
pub const PARENTS: [(&str, &str); 14] = [
    ("SF-1100", "SF-1000"),
    ("SF-1110", "SF-1100"),
    ("SF-1200", "SF-1000"),
    ("SF-1210", "SF-1200"),
    ("SF-1211", "SF-1210"),
    ("SF-1212", "SF-1210"),
    ("SF-1220", "SF-1200"),
    ("SF-1300", "SF-1000"),
    ("SF-1310", "SF-1300"),
    ("SF-1320", "SF-1300"),
    ("SF-1390", "SF-1300"),
    ("SF-1400", "SF-1000"),
    ("SF-1410", "SF-1400"),
    ("SF-1420", "SF-1400"),
];

/// The top finding.
pub const FINDINGS_TOP: &str = "SF-1000";
/// The findings open when the address names none: the walk to the
/// selected one, and the branch with the inactive concept.
pub const FINDINGS_OPEN: [&str; 4] = ["SF-1000", "SF-1200", "SF-1210", "SF-1300"];
/// The finding selected when the address names none.
pub const FINDINGS_SELECTED: &str = "SF-1211";
/// The findings the search for cough matches.
pub const FINDINGS_MATCHED: [&str; 3] = ["SF-1210", "SF-1211", "SF-1212"];
/// The version of the findings read.
pub const FINDINGS_VERSION: &str = "3";
/// The findings' code system.
pub const FINDINGS_SYSTEM: &str = "http://example.org/synthetic-findings";

/// The top national finding.
pub const NATIONAL_TOP: &str = "NF-100";
/// The national finding with a long level.
pub const NATIONAL_CHEST: &str = "NF-200";
/// The national finding being read.
pub const NATIONAL_SKIN: &str = "NF-300";
/// The version of the national findings read.
pub const NATIONAL_VERSION: &str = "2026-10";
/// How many children the long level has, how many it shows, and how many
/// the next page reads.
pub const NATIONAL_LEVEL: (u32, u32, u32) = (40, 25, 25);

/// The code of the national finding's child `index`.
#[must_use]
pub fn reason_code(index: u32) -> String {
    format!("NF-2{index:02}")
}

/// The flat code system's matches: a code and its display.
pub const COUNTRIES: [(&str, Msg); 2] =
    [("NL", Msg::TreesNetherlands), ("BQ", Msg::TreesCaribbean)];
/// The version of the flat code system read.
pub const COUNTRIES_VERSION: &str = "CLDR 48";

/// The directory's references: a key, a name and a short uid.
pub const REFERENCES: [(&str, Msg, &str); 3] = [
    ("vitals", Msg::TreesVitalSigns, "0199f1d0…7c11"),
    ("laboratory", Msg::TreesLaboratory, "0199e4b7…8a37"),
    ("summary", Msg::TreesDischarge, "01995f22…2b53"),
];

/// One node of the synthetic template: its key, its parent's key, its
/// name and its RM type.
pub type Node = (&'static str, Option<&'static str>, Msg, &'static str);

/// The synthetic template's nodes, each after its parent.
pub const TEMPLATE: [Node; 14] = [
    ("encounter", None, Msg::TreesEncounter, "COMPOSITION"),
    (
        "context",
        Some("encounter"),
        Msg::TreesContext,
        "EVENT_CONTEXT",
    ),
    (
        "setting",
        Some("context"),
        Msg::TreesSetting,
        "DV_CODED_TEXT",
    ),
    (
        "blood-pressure",
        Some("encounter"),
        Msg::TreesBloodPressure,
        "OBSERVATION",
    ),
    (
        "systolic",
        Some("blood-pressure"),
        Msg::TreesSystolic,
        "DV_QUANTITY",
    ),
    (
        "diastolic",
        Some("blood-pressure"),
        Msg::TreesDiastolic,
        "DV_QUANTITY",
    ),
    (
        "position",
        Some("blood-pressure"),
        Msg::TreesPosition,
        "DV_CODED_TEXT",
    ),
    (
        "cuff",
        Some("blood-pressure"),
        Msg::TreesCuff,
        "DV_CODED_TEXT",
    ),
    (
        "device",
        Some("blood-pressure"),
        Msg::TreesDevice,
        "CLUSTER",
    ),
    (
        "device-name",
        Some("device"),
        Msg::TreesDeviceName,
        "DV_TEXT",
    ),
    ("pulse", Some("encounter"), Msg::TreesPulse, "OBSERVATION"),
    ("rate", Some("pulse"), Msg::TreesRate, "DV_QUANTITY"),
    (
        "temperature",
        Some("encounter"),
        Msg::TreesTemperature,
        "OBSERVATION",
    ),
    (
        "degrees",
        Some("temperature"),
        Msg::TreesBodyTemperature,
        "DV_QUANTITY",
    ),
];

/// The template's nodes open as the page arrives.
pub const TEMPLATE_OPEN: [&str; 2] = ["encounter", "blood-pressure"];
/// The template node selected: a coded one.
pub const TEMPLATE_SELECTED: &str = "position";
/// The selected node's archetype path.
pub const POSITION_PATH: &str = "/content[openEHR-EHR-OBSERVATION.blood_pressure.v2]/data[at0001]/events[at0006]/state[at0007]/items[at0008]/value";
/// The selected node's FLAT path.
pub const POSITION_FLAT: &str = "vital_signs_encounter/blood_pressure/any_event:0/position";
/// The selected node's RM type.
pub const POSITION_TYPE: &str = "DV_CODED_TEXT";
/// The selected node's occurrences.
pub const POSITION_OCCURRENCES: &str = "0..1";
/// The selected node's value set: a display and its local code.
pub const POSITION_CODES: [(Msg, &str); 5] = [
    (Msg::TreesStanding, "at1000"),
    (Msg::TreesSitting, "at1001"),
    (Msg::TreesReclining, "at1002"),
    (Msg::TreesLying, "at1003"),
    (Msg::TreesLyingTilt, "at1014"),
];
/// The template's root archetype.
pub const ROOT_ARCHETYPE: &str = "openEHR-EHR-COMPOSITION.encounter.v1";
