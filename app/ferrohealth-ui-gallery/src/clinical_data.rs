// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The synthetic record the clinical record parts draw
//! (`pages::clinical`), as data: a subject, a composition's versions and
//! content, a query's paths and rows, and the scopes an app holds.
//!
//! Every value is invented; none is patient data.

/// The subject's namespace, the identifier's domain.
pub const SUBJECT_NAMESPACE: &str = "demo.hospital.mrn";

/// The subject's whole identifier, which the gallery's pretend server alone
/// reads to answer a reveal.
pub const SUBJECT_VALUE: &str = "P-20481-537";

/// The gallery's clock, which the version history's relative times are
/// read against.
pub const NOW: &str = "2026-10-21T09:15:00Z";

/// One version of the synthetic composition: its number, when it was
/// committed, its change and its contribution.
#[derive(Debug)]
pub struct VersionRow {
    /// The version's number.
    pub number: &'static str,
    /// When it was committed, as an RFC 3339 instant.
    pub at: &'static str,
    /// Its contribution's short uid.
    pub contribution: &'static str,
}

/// The composition's versions, the latest first.
pub const VERSIONS: &[VersionRow] = &[
    VersionRow {
        number: "3",
        at: "2026-10-21T09:03:00Z",
        contribution: "0199f1d0…4f16",
    },
    VersionRow {
        number: "2",
        at: "2026-10-21T08:51:00Z",
        contribution: "0199f1c4…9e30",
    },
    VersionRow {
        number: "1",
        at: "2026-10-21T08:44:00Z",
        contribution: "0199f1b2…8a29",
    },
];

/// The deletion of the composition, as a fourth version.
pub const DELETION: VersionRow = VersionRow {
    number: "4",
    at: "2026-10-21T09:12:00Z",
    contribution: "0199f22a…61c4",
};

/// The archetype ids of the composition's entries and clusters.
pub const BLOOD_PRESSURE: &str = "openEHR-EHR-OBSERVATION.blood_pressure.v2";
/// The device cluster inside the blood pressure.
pub const DEVICE: &str = "openEHR-EHR-CLUSTER.device.v1";
/// The calibration cluster inside the device.
pub const CALIBRATION: &str = "openEHR-EHR-CLUSTER.device_calibration.v0";
/// The pulse.
pub const PULSE: &str = "openEHR-EHR-OBSERVATION.pulse.v2";
/// The body temperature.
pub const TEMPERATURE: &str = "openEHR-EHR-OBSERVATION.body_temperature.v2";

/// A quantity's magnitude, its unit as read, and its UCUM code.
pub type Quantity = (&'static str, &'static str, &'static str);

/// The systolic pressure of the latest version and of version 2.
pub const SYSTOLIC: [Quantity; 2] = [("128", "mm Hg", "mm[Hg]"), ("141", "mm Hg", "mm[Hg]")];
/// The diastolic pressure of the latest version and of version 2.
pub const DIASTOLIC: [Quantity; 2] = [("82", "mm Hg", "mm[Hg]"), ("88", "mm Hg", "mm[Hg]")];
/// The pulse rate.
pub const RATE: Quantity = ("72", "/min", "/min");
/// The body temperature.
pub const BODY_TEMPERATURE: Quantity = ("37.1", "°C", "Cel");

/// A coded value: its display, its code and its terminology.
pub type Coded = (&'static str, &'static str, &'static str);

/// The position the pressure was taken in.
pub const POSITION: Coded = ("Sitting", "at1001", "local");
/// The pulse's regularity.
pub const REGULARITY: Coded = ("Regular", "at0006", "local");
/// The composition's setting.
pub const SETTING: Coded = ("other care", "238", "openehr");

/// The composition's start time.
pub const START_TIME: &str = "2026-10-21T07:50:00Z";
/// The device's name.
pub const DEVICE_NAME: &str = "Automatic upper-arm cuff";
/// The day the device was last calibrated.
pub const CALIBRATED: &str = "2026-09-30";
/// The composer.
pub const COMPOSER: &str = "Demo Nurse";
/// The clinician who deleted the composition.
pub const CLINICIAN: &str = "Demo Clinician";

/// A path the builder offers: its words and its archetype path.
pub type PathChoice = (&'static str, &'static str);

/// The systolic pressure.
pub const PATH_SYSTOLIC: PathChoice = (
    "Blood pressure › Systolic",
    "…/items[at0004]/value/magnitude",
);
/// The position.
pub const PATH_POSITION: PathChoice = ("Blood pressure › Position", "…/items[at0008]/value");
/// The pulse rate.
pub const PATH_RATE: PathChoice = ("Pulse › Rate", "…/items[at0004]/value/magnitude");
/// The body temperature.
pub const PATH_TEMPERATURE: PathChoice = (
    "Body temperature › Temperature",
    "…/items[at0004]/value/magnitude",
);

/// The operators of a quantity.
pub const QUANTITY_OPERATORS: &[&str] = &[">", "≥", "=", "<", "≤", "≠"];
/// The operators of a coded value.
pub const CODED_OPERATORS: &[&str] = &["=", "≠"];

/// The positions' value set: each code and its words.
pub const POSITIONS: &[(&str, &str)] = &[
    ("at1000", "Standing · at1000"),
    ("at1001", "Sitting · at1001"),
    ("at1002", "Reclining · at1002"),
    ("at1003", "Lying · at1003"),
];

/// The answer's column aliases.
pub const COLUMNS: [&str; 3] = ["time", "systolic", "diastolic"];

/// One row of the answer: its time, its systolic and diastolic pressure,
/// and the day and month of its time, which the chart's axis names.
#[derive(Debug)]
pub struct AnswerRow {
    /// The time, as the CDR returns it.
    pub at: &'static str,
    /// The systolic pressure.
    pub systolic: f64,
    /// The diastolic pressure.
    pub diastolic: f64,
    /// The day of the month.
    pub day: u8,
    /// The month, 1 to 12.
    pub month: u8,
}

/// The answer's rows, the latest first.
pub const ROWS: &[AnswerRow] = &[
    AnswerRow {
        at: "2026-10-21T07:50:00+02:00",
        systolic: 128.0,
        diastolic: 82.0,
        day: 21,
        month: 10,
    },
    AnswerRow {
        at: "2026-10-07T09:05:00+02:00",
        systolic: 141.0,
        diastolic: 88.0,
        day: 7,
        month: 10,
    },
    AnswerRow {
        at: "2026-09-23T08:51:00+02:00",
        systolic: 136.0,
        diastolic: 85.0,
        day: 23,
        month: 9,
    },
    AnswerRow {
        at: "2026-09-09T10:12:00+02:00",
        systolic: 144.0,
        diastolic: 91.0,
        day: 9,
        month: 9,
    },
    AnswerRow {
        at: "2026-08-26T09:40:00+02:00",
        systolic: 139.0,
        diastolic: 87.0,
        day: 26,
        month: 8,
    },
    AnswerRow {
        at: "2026-08-11T16:30:00+02:00",
        systolic: 151.0,
        diastolic: 94.0,
        day: 11,
        month: 8,
    },
];

/// The scopes an app holds: each raw, its resource in words, and whether
/// it is broad.
///
/// Two ask for no resource, one is narrowed by a search constraint, and
/// the last breaks the grammar, its permissions out of order.
pub const SCOPES: &[(&str, &str, bool)] = &[
    ("openid", "", false),
    ("launch/patient", "", false),
    (
        "patient/composition-*.rs",
        "Compositions of any template",
        false,
    ),
    (
        "patient/composition-*.rs?archetype_id=openEHR-EHR-OBSERVATION.blood_pressure.v2",
        "Compositions of any template",
        false,
    ),
    (
        "user/composition-*.cruds",
        "Compositions of any template",
        false,
    ),
    ("system/aql-*.rs", "Every stored and ad-hoc query", true),
    ("patient/composition-*.sr", "", false),
];

/// A scope the grammar takes and the CDR does not: a FHIR resource type.
pub const FOREIGN: &str = "patient/Observation.read";
