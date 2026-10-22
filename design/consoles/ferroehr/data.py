# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The synthetic deployment every FerroEHR mockup page draws: one CDR at
cdr.example.org, the viewer in front of it, its EHRs, compositions,
templates, stored queries, parties, audit records and subscriptions.

Synthetic or public only. No real person appears: subjects are synthetic
hospital numbers (P-nnnnn-nnn) in the namespace demo.hospital.mrn, people
are Demo Operator, Demo Clinician, Demo Nurse and Demo Person nnnn.
Archetype ids are public CKM ids. Every nine-digit run fails the BSN
eleven-proof; the generator checks every page."""

VERSION = "4.3.10"
CDR = "https://cdr.example.org"
REST = "/ferroehr/rest/openehr/v1"
VIEWER = "https://viewer.example.org"
SYSTEM_ID = "cdr.example.org"

# ---- EHRs ----
E1 = "0199c2a4-7e1b-4c3e-9a51-4f2d8b6e1a01"
E2 = "0199c2a1-02d4-4f61-b0c3-6e1d2f9a4b17"

# (id, subject tail or None, created iso, compositions)
EHRS = [
    (E2, "112", "2026-10-21T09:31", 0),
    ("0199c29e-55a0-4d12-8e40-0b7c3a1f6d29", "208", "2026-10-21T08:02", 3),
    ("0199c28b-1f3c-4a88-9d21-3c5e7b0a2e44", "761", "2026-10-20T16:47", 12),
    ("0199c27f-9b62-4c05-a1f4-7d2e8c6b5a90", "054", "2026-10-20T11:20", 5),
    ("0199c26a-4e08-41b3-b6d7-2a9f0e4c8d13", None, "2026-10-19T14:05", 1),
    ("0199c25c-07d9-4e44-8a3b-5f1c6d2e9b78", "390", "2026-10-18T09:58", 21),
    ("0199c241-c3a5-4f9e-92c0-8b4d1e7a3f05", "618", "2026-10-17T13:36", 7),
    ("0199c22d-6a17-40c8-b5e2-1d9a3c8f4e66", "925", "2026-10-15T10:11", 2),
    ("0199c210-8d4f-4b31-a7c6-4e2b9d0f1a82", "473", "2026-10-14T15:44", 16),
    (E1, "537", "2026-03-02T10:14", 8),
]

# ---- compositions of E1 ----
# (key, name, template, start iso, composer, versions, uid)
COMPS = [
    ("vitals", "Vital signs", "Vital signs encounter.v1", "2026-10-21T07:50", "Demo Nurse", 3, "0199f1d0-33aa-7b20-8c4e-2a9e51b07c11"),
    ("lab", "Laboratory result report", "Laboratory result report.v1", "2026-10-18T13:02", "Demo Clinician", 1, "0199e4b7-91c2-7d05-b3a8-6f0d2c4e8a37"),
    ("problems", "Problem list", "Problem list.v1", "2026-10-14T10:40", "Demo Clinician", 4, "0199d6a3-2e5f-7a19-9c71-0b8e4d3f2c90"),
    ("meds", "Medication list", "Medication list.v1", "2026-10-14T10:38", "Demo Clinician", 2, "0199d6a2-7c40-7b88-a2e6-3d1f9c0b5e14"),
    ("vitals2", "Vital signs", "Vital signs encounter.v1", "2026-10-07T09:05", "Demo Nurse", 1, "0199c9e1-4b0a-7f23-8e5d-7a2c6b1d0f48"),
    ("allergy", "Adverse reaction list", "Adverse reaction list.v1", "2026-09-30T15:22", "Demo Clinician", 1, "0199bc08-5d71-7c46-b9f0-2e4a8d3c1b65"),
    ("vitals3", "Vital signs", "Vital signs encounter.v1", "2026-09-23T08:51", "Demo Nurse", 1, "0199ae6f-0a93-7e12-a4c8-9b5d2f7e3a01"),
    ("discharge", "Discharge summary", "Discharge summary.v1", "2026-08-11T16:30", "Demo Clinician", 2, "01995f22-c8e4-7a07-8d3b-1f6c0e9a2b53"),
]
C1 = COMPS[0][6]

# ---- templates (ADL 1.4) ----
# (id, concept, root archetype, compositions, uploaded iso)
TEMPLATES = [
    ("Vital signs encounter.v1", "Vital signs encounter", "openEHR-EHR-COMPOSITION.encounter.v1", 1284, "2026-02-11T09:00"),
    ("Laboratory result report.v1", "Laboratory result report", "openEHR-EHR-COMPOSITION.report-result.v1", 642, "2026-02-11T09:02"),
    ("Problem list.v1", "Problem list", "openEHR-EHR-COMPOSITION.problem_list.v2", 311, "2026-02-11T09:03"),
    ("Medication list.v1", "Medication list", "openEHR-EHR-COMPOSITION.medication_list.v1", 297, "2026-02-11T09:05"),
    ("Adverse reaction list.v1", "Adverse reaction list", "openEHR-EHR-COMPOSITION.adverse_reaction_list.v1", 188, "2026-02-12T14:20"),
    ("Discharge summary.v1", "Discharge summary", "openEHR-EHR-COMPOSITION.transfer_summary.v1", 96, "2026-03-04T11:31"),
    ("Care plan.v1", "Care plan", "openEHR-EHR-COMPOSITION.care_plan.v0", 41, "2026-04-22T10:08"),
    ("Referral.v1", "Referral", "openEHR-EHR-COMPOSITION.request.v1", 37, "2026-05-06T13:44"),
    ("Diagnostic imaging report.v1", "Diagnostic imaging report", "openEHR-EHR-COMPOSITION.report.v1", 22, "2026-06-18T08:15"),
    ("Growth chart.v1", "Growth chart", "openEHR-EHR-COMPOSITION.encounter.v1", 9, "2026-08-27T15:02"),
    ("Self-monitoring.v0", "Self-monitoring", "openEHR-EHR-COMPOSITION.self_monitoring.v0", 3, "2026-10-02T09:40"),
    ("Minimal evaluation.v1", "Minimal evaluation", "openEHR-EHR-COMPOSITION.encounter.v1", 0, "2026-10-20T17:12"),
]
EHR_COUNT = 2418
COMP_COUNT = sum(t[3] for t in TEMPLATES)

# ADL 2 artefacts: (hrid, concept, root, compositions)
ADL2 = [
    ("openEHR-EHR-COMPOSITION.vital_signs_encounter.v1.1.0", "Vital signs encounter", "openEHR-EHR-COMPOSITION.encounter.v1", 0, "2026-10-12T11:20"),
    ("openEHR-EHR-COMPOSITION.vital_signs_encounter.v1.0.0", "Vital signs encounter", "openEHR-EHR-COMPOSITION.encounter.v1", 0, "2026-09-01T10:05"),
    ("openEHR-EHR-COMPOSITION.lab_order.v0.4.0", "Laboratory order", "openEHR-EHR-COMPOSITION.request.v1", 0, "2026-07-15T14:48"),
]

# ---- stored queries ----
# (namespace or None, name, version, parameters, matches when run)
QUERIES = [
    ("org.example.cardio", "bp_latest", "1.1.0", ["ehr_id"]),
    ("org.example.cardio", "bp_latest", "1.0.0", ["ehr_id"]),
    ("org.example.cardio", "hypertensive_ehrs", "1.0.0", ["systolic"]),
    ("org.example.quality", "allergy_review", "2.0.0", []),
    ("org.example.quality", "hba1c_overdue", "1.0.0", ["months"]),
    ("org.example.research", "cohort_diabetes", "1.0.0", []),
    (None, "compositions_by_template", "1.0.0", ["template_id"]),
    (None, "recent_vitals", "1.0.0", []),
]
# The dashboard's namespace rows: (namespace label, queries, summed matches or None when one failed)
NAMESPACES = [("org.example.cardio", 2, 1342), ("org.example.quality", 2, 57), ("org.example.research", 1, 214), ("unqualified", 2, 3120)]

BP_AQL = """SELECT
  c/context/start_time/value AS time,
  o/data[at0001]/events[at0006]/data[at0003]/items[at0004]/value/magnitude AS systolic,
  o/data[at0001]/events[at0006]/data[at0003]/items[at0005]/value/magnitude AS diastolic
FROM EHR e
  CONTAINS COMPOSITION c[openEHR-EHR-COMPOSITION.encounter.v1]
  CONTAINS OBSERVATION o[openEHR-EHR-OBSERVATION.blood_pressure.v2]
WHERE e/ehr_id/value = $ehr_id
ORDER BY c/context/start_time/value DESC"""

# ---- contributions of E1: (uid, committed iso, committer, change, versions) ----
CONTRIBS = [
    # (uid, committed, committer, change, versions); 18 contributions make the EHR's 20 versions:
    # compositions 15, status 2, directory 3.
    ("0199f1da-0c55-7e21-b7a4-1d3f6c9e2a80", "2026-10-21T08:20", "Demo Nurse", "Modification", 1),
    ("0199f1d0-3412-7c9a-a0e1-5b7d2c8e4f16", "2026-10-21T08:14", "Demo Nurse", "Modification", 1),
    ("0199f1c4-8d02-7b55-9e17-4a6c0f2b9e30", "2026-10-21T08:02", "Demo Nurse", "Modification", 1),
    ("0199f1b2-77e0-7a3d-9f42-0c6e1b5d8a29", "2026-10-21T07:55", "Demo Nurse", "Creation", 1),
    ("0199eb10-6c2f-7a51-9d04-3e8b1f7d7d42", "2026-10-20T15:02", "Demo Operator", "Modification", 1),
    ("0199e4b7-9a03-7e61-b2c9-4d8f0a3e7c52", "2026-10-18T13:02", "Demo Clinician", "Creation", 1),
    ("0199dbe2-41f6-7a08-8c35-6e9b2d0f4a71", "2026-10-16T11:30", "Demo Clinician", "Modification", 2),
    ("0199d7f0-2a9c-7d43-b6e1-0f8a5c3d7b26", "2026-10-15T09:10", "Demo Clinician", "Modification", 1),
    ("0199d6a4-7b13-7f62-a0d8-3c5e9b1f6a44", "2026-10-14T10:45", "Demo Clinician", "Modification", 1),
    ("0199d6a3-5f21-7c84-9b06-2d7e0a4c8f19", "2026-10-14T10:41", "Demo Clinician", "Modification", 1),
    ("0199d6a3-31b8-7f04-8c5a-2e9d6b1f0a73", "2026-10-14T10:40", "Demo Clinician", "Creation", 2),
    ("0199c9e1-5c27-7d90-a6b3-8f1e4c2d9b05", "2026-10-07T09:05", "Demo Nurse", "Creation", 1),
    ("0199bc08-6e15-7b72-9d48-3a0c7f5e1b86", "2026-09-30T15:22", "Demo Clinician", "Creation", 1),
    ("0199ae6f-1b88-7c3f-b5d0-6e2a9c4f8d17", "2026-09-23T08:51", "Demo Nurse", "Creation", 1),
    ("01996446-3e70-7a19-8f52-1b4d7c0e9a35", "2026-08-12T09:00", "Demo Clinician", "Modification", 1),
    ("01995f22-d915-7e8b-a3c6-0d7b2e1f5a94", "2026-08-11T16:30", "Demo Clinician", "Creation", 1),
    ("01994a22-6c48-7d01-9a7e-5f2b8c1d3e60", "2026-03-02T10:20", "Demo Operator", "Creation", 1),
    ("01994a1f-0e93-7b26-8d41-7a3c9e5f0b12", "2026-03-02T10:14", "Demo Operator", "Creation", 1),
]

# ---- parties ----
P1 = "8a3e51c2-6d04-4f7b-9e18-2c5a0b7d3f61"
ORG1 = "3f9b0d47-21ac-4e85-b6d3-7a1c9e5f2b08"
REL1 = "c71e4a90-5b38-4d26-8f0e-9d2b6a1c4e75"

AUDIT = [
    # (time, event, principal, patient tail or None, outcome tone, outcome word, record id)
    ("09:41:58", "Read composition", "Demo Clinician", "537", "ok", "Success"),
    ("09:41:12", "Run stored query", "svc-reporting", None, "ok", "Success"),
    ("09:39:40", "Reveal subject id", "Demo Operator", "537", "ok", "Success"),
    ("09:37:05", "Commit composition", "Demo Nurse", "208", "ok", "Success"),
    ("09:35:51", "Read EHR status", "Demo Clinician", "761", "ok", "Success"),
    ("09:33:20", "Delete template", "Demo Operator", None, "warn", "Minor failure"),
    ("09:31:02", "Create EHR", "Demo Operator", "112", "ok", "Success"),
    ("09:30:44", "Read composition", "unknown", "390", "bad", "Serious failure"),
    ("09:28:13", "Run AQL", "Demo Operator", None, "ok", "Success"),
    ("09:25:37", "Update EHR status", "Demo Clinician", "054", "ok", "Success"),
]

SUBSCRIPTIONS = [
    # (name, kind, change type, template, enabled, created iso)
    ("vital-signs-feed", "COMPOSITION", None, "Vital signs encounter.v1", True, "2026-09-02T10:00"),
    ("ehr-status-changes", "EHR_STATUS", None, None, True, "2026-09-02T10:05"),
    ("deletions", None, "523", None, False, "2026-10-01T14:12"),
]
