// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The synthetic code systems, value sets, roots and concept map the
//! terminology page draws.
//!
//! They are invented for the gallery, in the shape a FHIR terminology
//! server answers with, and the same on every render. A display is data the
//! server sends, so it stays here beside its code; the page's own words are
//! in its catalog. Nothing here is clinical data.

/// The synthetic findings code system's name.
pub const FINDINGS: &str = "Synthetic clinical findings";

/// The legacy code system's name.
pub const LEGACY: &str = "Legacy findings";

/// The finding of the chest.
pub const CHEST: &str = "SF-1200";

/// Cough.
pub const COUGH: &str = "SF-1210";

/// Wheeze.
pub const WHEEZE: &str = "SF-1220";

/// Fever.
pub const FEVER: &str = "SF-1310";

/// An inactive finding.
pub const MALAISE: &str = "SF-1390";

/// A finding the table holds no display for.
pub const ABDOMEN: &str = "SF-1400";

/// The root the synthetic server serves as a ballot, which did not answer.
pub const BALLOT_ROOT: &str = "R6";

/// The legacy cough.
pub const LEGACY_COUGH: &str = "LF-01";

/// The legacy chest complaint.
pub const LEGACY_CHEST: &str = "LF-04";

/// The synthetic findings, by code: the code and its display.
pub const CODES: &[(&str, &str)] = &[
    ("SF-1200", "Finding of the chest"),
    ("SF-1210", "Cough"),
    ("SF-1220", "Wheeze"),
    ("SF-1310", "Fever"),
    ("SF-1390", "Malaise, old code"),
];

/// The display of `code`, or the code itself when the table has none.
#[must_use]
pub fn display(code: &str) -> String {
    CODES
        .iter()
        .find(|(each, _)| *each == code)
        .map_or_else(|| code.to_owned(), |(_, display)| (*display).to_owned())
}

/// A canonical long enough to wrap on a phone.
pub const LONG_CANONICAL: &str = "https://terminology.example.org/fhir/ValueSet/synthetic-respiratory-findings-for-the-ward-screens";

/// A synthetic `urn:uuid:` canonical, with no slash to break after.
pub const URN_CANONICAL: &str = "urn:uuid:3f1c2a9e-7b4d-4e8a-9c21-5d6b0e4f8a17";

/// A synthetic composition's versioned uid as a code, too long for one
/// line on a phone.
pub const VERSIONED_UID: &str = "8849182c-82ad-4088-a07f-48ead4180515::ferroehr.example.org::12";

/// The value sets of the coding card's table: the title and the canonical.
pub const VALUE_SETS: &[(&str, &str)] = &[
    (
        "Synthetic respiratory findings",
        "https://terminology.example.org/fhir/ValueSet/synthetic-respiratory",
    ),
    (
        "Synthetic skin findings",
        "https://terminology.example.org/fhir/ValueSet/synthetic-skin",
    ),
];

/// The FHIR roots the synthetic server answers on: the name, the release
/// and whether it answered.
pub const ROOTS: &[(&str, &str, bool)] = &[
    ("R4", "FHIR 4.0.1", true),
    ("R4B", "FHIR 4.3.0", true),
    ("R5", "FHIR 5.0.0", true),
    ("R6", "FHIR 6.0.0-ballot5", false),
];

/// The root the synthetic server reads through by default.
pub const DEFAULT_ROOT: &str = "R4";

/// The software each root names.
pub const SOFTWARE: &str = "FerroTERM 0.9.0";

/// The base address of each root, by its name in lower case.
#[must_use]
pub fn base(root: &str) -> String {
    format!("/{}", root.to_lowercase())
}

/// The code systems each root declares.
pub const CODE_SYSTEMS: &str = "15";

/// The security service each root declares.
pub const SECURITY: &str = "SMART-on-FHIR";

/// One mapping of the synthetic concept map: the source, the R4
/// equivalence, the target and the comment.
pub type Mapped = (
    &'static str,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
);

/// The legacy codes and their displays.
pub const LEGACY_CODES: &[(&str, &str)] = &[
    ("LF-01", "Cough, any"),
    ("LF-04", "Chest complaint"),
    ("LF-06", "Raised temperature"),
    ("LF-07", "Feeling unwell"),
];

/// The display of the legacy `code`, or the code itself.
#[must_use]
pub fn legacy_display(code: &str) -> String {
    LEGACY_CODES
        .iter()
        .find(|(each, _)| *each == code)
        .map_or_else(|| code.to_owned(), |(_, display)| (*display).to_owned())
}

/// The mappings of the synthetic concept map.
pub const MAPPINGS: &[Mapped] = &[
    ("LF-01", "equivalent", Some("SF-1210"), None),
    (
        "LF-04",
        "narrower",
        Some("SF-1220"),
        Some("Only the wheeze part of the old code survives."),
    ),
    ("LF-06", "equivalent", Some("SF-1310"), None),
    (
        "LF-07",
        "unmatched",
        None,
        Some("No synthetic code carries this meaning."),
    ),
];
