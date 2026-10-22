# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The synthetic bridge every FerroBRIDGE mockup page draws: no real person,
template, organisation or host. Templates are synthetic.* ids, hosts are
under example.org, patients are Patient/synthetic-NNNN, the operator is
Demo Operator and the second operator Demo Reviewer. Every run of nine digits
on a page must fail the BSN eleven-proof (build.py checks)."""

VERSION = "0.0.12"
COMMIT = "6af1258"
BRIDGE = "https://bridge.demo-regio.example.org"

# The pin matrix the bridge reports in GET /health/info.
PINS = [("FHIRconnect", "v1.0.0"), ("FHIR", "R4 (4.0.1)"), ("OMOCL", "v1.0.0"), ("OMOP CDM", "v5.4"),
        ("openEHR ITS-REST", "1.1.0"), ("openehr-* crates", "0.0.87"), ("fhir-types", "0.1.127")]

# The FHIRconnect contexts the running load compiled:
# (key, context file, resource type, profile, template, models, search, owner, warnings)
CONTEXTS = [
    ("vital_signs", "fhirconnect/synthetic_vital_signs.context.yml", "Observation", "synthetic-vital-signs", "synthetic.vital_signs.v1",
     ["blood_pressure", "body_temperature", "pulse"], "fhirconnect/synthetic_vital_signs.search.yml", "provisioned", 0),
    ("body_weight", "fhirconnect/synthetic_body_weight.context.yml", "Observation", "synthetic-body-weight", "synthetic.vital_signs.v1",
     ["body_weight"], None, "console", 2),
    ("lab_result", "fhirconnect/synthetic_lab_result.context.yml", "Observation", "synthetic-lab-result", "synthetic.lab_report.v1",
     ["lab_analyte", "specimen"], None, "provisioned", 1),
    ("diagnosis", "fhirconnect/synthetic_diagnosis.context.yml", "Condition", "synthetic-diagnosis", "synthetic.diagnosis.v1",
     ["problem_diagnosis"], "fhirconnect/synthetic_diagnosis.search.yml", "provisioned", 0),
    ("medication", "fhirconnect/synthetic_medication.context.yml", "MedicationStatement", "synthetic-medication", "synthetic.medication_summary.v1",
     ["medication_statement"], None, "provisioned", 0),
    ("allergy", "fhirconnect/synthetic_allergy.context.yml", "AllergyIntolerance", "synthetic-allergy", "synthetic.allergy_list.v1",
     ["adverse_reaction"], None, "console", 0),
    ("encounter", "fhirconnect/synthetic_encounter.context.yml", "Encounter", "synthetic-encounter", "synthetic.encounter.v1",
     ["encounter"], None, "provisioned", 0),
    ("procedure", "fhirconnect/synthetic_procedure.context.yml", "Procedure", "synthetic-procedure", "synthetic.procedure.v1",
     ["procedure"], None, "provisioned", 0),
    ("problem_list", "fhirconnect/synthetic_problem_list.context.yml", "Condition", "synthetic-problem-list", "synthetic.diagnosis.v1",
     ["problem_diagnosis"], None, "provisioned", 0),
]

PROFILE_BASE = "http://example.org/fhir/StructureDefinition/"

# The OMOCL files: (file, table, archetype, owner)
OMOCL = [
    ("omocl/synthetic_person.omocl.yml", "PERSON", "openEHR-EHR-ADMIN_ENTRY.person_data.v0", "provisioned"),
    ("omocl/synthetic_measurement.omocl.yml", "MEASUREMENT", "openEHR-EHR-OBSERVATION.blood_pressure.v2", "provisioned"),
    ("omocl/synthetic_condition.omocl.yml", "CONDITION_OCCURRENCE", "openEHR-EHR-EVALUATION.problem_diagnosis.v1", "provisioned"),
    ("omocl/synthetic_drug.omocl.yml", "DRUG_EXPOSURE", "openEHR-EHR-INSTRUCTION.medication_order.v3", "console"),
]

# The operational templates: (id, source, file or None, archetypes)
TEMPLATES = [
    ("synthetic.vital_signs.v1", "directory", "templates/synthetic_vital_signs.opt", 5),
    ("synthetic.lab_report.v1", "cdr", None, 3),
    ("synthetic.diagnosis.v1", "directory", "templates/synthetic_diagnosis.opt", 2),
    ("synthetic.medication_summary.v1", "cdr", None, 3),
    ("synthetic.allergy_list.v1", "directory", "templates/synthetic_allergy_list.opt", 2),
    ("synthetic.encounter.v1", "cdr", None, 2),
    ("synthetic.procedure.v1", "cdr", None, 2),
]

MODEL_FILES = ["blood_pressure", "body_temperature", "pulse", "body_weight", "lab_analyte", "specimen", "problem_diagnosis",
               "medication_statement", "adverse_reaction", "encounter", "procedure"]
EXTENSION_FILES = [("synthetic_nl_problem", "problem_diagnosis")]
SEARCH_FILES = ["fhirconnect/synthetic_vital_signs.search.yml", "fhirconnect/synthetic_diagnosis.search.yml"]

CONTEXT_YAML = """grammar: FHIRConnect/v1.0.0
type: context
metadata:
  name: synthetic_body_weight.context
  version: 1.2.0
spec:
  system: FHIR
  version: R4
context:
  profile:
    url: "http://example.org/fhir/StructureDefinition/synthetic-body-weight"
    version: "1.2.0"
  template:
    id: "synthetic.vital_signs.v1"
  archetypes:
    - "OBSERVATION.body_weight.v2"
    - "CLUSTER.device.v1"
  start: "OBSERVATION.body_weight.v2"
"""

MODEL_YAML = """grammar: FHIRConnect/v1.0.0
type: model
metadata:
  name: OBSERVATION.body_weight.v2
  version: 1.0.0
spec:
  system: FHIR
  version: R4
  openEhrConfig:
    archetype: openEHR-EHR-OBSERVATION.body_weight.v2
  fhirConfig:
    structureDefinition: http://hl7.org/fhir/StructureDefinition/Observation
mappings:
  - name: weight
    with:
      fhir: "$resource.valueQuantity"
      openehr: "$archetype/data[at0002]/events[at0003]/data[at0001]/items[at0004]"
    type: QUANTITY
  - name: time
    with:
      fhir: "$resource.effectiveDateTime"
      openehr: "$archetype/data[at0002]/events[at0003]/time"
    type: DATETIME
  - name: method
    with:
      fhir: "$resource.method"
      openehr: "$archetype/protocol[at0015]/items[at0020]"
    type: CODEABLECONCEPT
"""


def line_of(text, needle, nth=1):
    """The 1-based line of the nth line holding `needle`, and the column it starts at."""
    seen = 0
    for i, line in enumerate(text.split("\n"), start=1):
        if needle in line:
            seen += 1
            if seen == nth:
                return i, line.index(needle) + 1
    raise ValueError(needle)


def diag(sev, code, path, text, needle, msg, mapping=None):
    """A diagnostic on the real line and column of `needle` in `text`."""
    ln, col = line_of(text, needle)
    return (sev, code, path, ln, col, msg, mapping)


CONTEXT_PATH = "fhirconnect/synthetic_body_weight.context.yml"
MODEL_PATH = "fhirconnect/models/body_weight.model.yml"
UNMAPPED = "The context lists CLUSTER.device.v1, which no model of the set maps, so the bridge carries none of its data."

# The running load's diagnostics: (severity, code, file, line, column, message, mapping)
DIAGNOSTICS = [
    ("warning", "unidirectional-value", "fhirconnect/models/lab_analyte.model.yml", 42, 9,
     "The mapping writes Observation.interpretation from openEHR only, so a conversion to openEHR leaves it out.", "lab_analyte › interpretation"),
    diag("warning", "unmapped-archetype", CONTEXT_PATH, CONTEXT_YAML, '"CLUSTER.device.v1"', UNMAPPED),
    diag("warning", "display-without-terminology", MODEL_PATH, MODEL_YAML, '"$resource.method"',
         "The coded value is written with its code as display, because the bridge reaches no terminology server for it.", "body_weight › method"),
]

# The draft the editor opens and checks clean: the profile's next version.
DRAFT_OK = CONTEXT_YAML.replace("  version: 1.2.0", "  version: 1.3.0").replace('    version: "1.2.0"', '    version: "1.3.0"')
CHECK_WARN = [diag("warning", "unmapped-archetype", CONTEXT_PATH, DRAFT_OK, '"CLUSTER.device.v1"', UNMAPPED)]

# A draft the bridge refuses: no metadata.version, and an extension no file defines.
DRAFT_YAML = DRAFT_OK.replace("  version: 1.3.0\n", "", 1).replace('  start: ', '  extensions:\n    - "body_weight_detail"\n  start: ', 1)
DRAFT_ERRORS = [
    diag("error", "missing-key", CONTEXT_PATH, DRAFT_YAML, "metadata:",
         "metadata.version is required (FHIRconnect contextual-mapping schema)."),
    diag("warning", "unmapped-archetype", CONTEXT_PATH, DRAFT_YAML, '"CLUSTER.device.v1"', UNMAPPED),
    diag("error", "unknown-extension", CONTEXT_PATH, DRAFT_YAML, '"body_weight_detail"',
         "The context names the extension body_weight_detail, which no file of the set defines."),
]

# The confirmed draft (DRAFT_OK), refused at the save because another file changed after the check.
REFUSED_ERRORS = [
    diag("error", "unmapped-start", CONTEXT_PATH, DRAFT_OK, 'start: "OBSERVATION',
         "No model of the set maps the start archetype OBSERVATION.body_weight.v2: fhirconnect/models/body_weight.model.yml changed after your check."),
] + CHECK_WARN

# Synthetic compositions and resources for the Playground. No real person.
PATIENT_REF = "Patient/synthetic-0042"
COMPOSITION_FLAT = """{
  "vitals/language|code": "en",
  "vitals/territory|code": "NL",
  "vitals/composer|name": "Demo Clinician",
  "vitals/body_weight/any_event:0/weight|magnitude": 72.4,
  "vitals/body_weight/any_event:0/weight|unit": "kg",
  "vitals/body_weight/any_event:0/time": "2026-10-20T08:15:00+02:00"
}"""

BUNDLE_JSON = """{
  "resourceType": "Bundle",
  "type": "collection",
  "entry": [
    {
      "resource": {
        "resourceType": "Observation",
        "meta": { "profile": [ "http://example.org/fhir/StructureDefinition/synthetic-body-weight" ] },
        "status": "final",
        "code": {
          "coding": [
            { "system": "http://loinc.org", "code": "29463-7" }
          ]
        },
        "subject": { "reference": "Patient/synthetic-0042" },
        "effectiveDateTime": "2026-10-20T08:15:00+02:00",
        "valueQuantity": {
          "value": 72.4,
          "unit": "kg",
          "system": "http://unitsofmeasure.org",
          "code": "kg"
        }
      }
    }
  ]
}"""

OBSERVATION_JSON = """{
  "resourceType": "Observation",
  "meta": { "profile": [ "http://example.org/fhir/StructureDefinition/synthetic-body-weight" ] },
  "status": "final",
  "code": {
    "coding": [ { "system": "http://loinc.org", "code": "29463-7" } ]
  },
  "subject": { "reference": "Patient/synthetic-0042" },
  "effectiveDateTime": "2026-10-20T08:15:00+02:00",
  "valueQuantity": { "value": 72.4, "unit": "kg" }
}"""

HL7_SEGMENTS = [
    ("MSH", "Message header", [("MSH-3", "Sending application", "DEMO-LAB"), ("MSH-4", "Sending facility", "DEMO-HOSP"),
                               ("MSH-7", "Date/time of message", "20261021094211+0200"), ("MSH-9", "Message type", "ORU^R01^ORU_R01"),
                               ("MSH-10", "Message control ID", "MSG-2026-1021-0007"), ("MSH-12", "Version ID", "2.5.1"), ("MSH-18", "Character set", "empty, so ASCII (the default)")]),
    ("PID", "Patient identification", [("PID-3", "Patient identifier list", "SYN-0042^^^DEMO-HOSP^MR"), ("PID-5", "Patient name", "TEST^SYNTHETIC^^^^^L"),
                                       ("PID-7", "Date/time of birth", "19800101"), ("PID-8", "Administrative sex", "U")]),
    ("OBR", "Observation request", [("OBR-2", "Placer order number", "ORD-77^DEMO-LAB"), ("OBR-4", "Universal service ID", "29463-7^Body weight^LN"),
                                    ("OBR-7", "Observation date/time", "20261021091500+0200"), ("OBR-25", "Result status", "F")]),
    ("OBX", "Observation result", [("OBX-2", "Value type", "NM"), ("OBX-3", "Observation identifier", "29463-7^Body weight^LN"),
                                   ("OBX-5", "Observation value", "72.4"), ("OBX-6", "Units", "kg^kilogram^UCUM"), ("OBX-11", "Result status", "F")]),
]

# The ETL run report (planned #612): the RunReport the runner prints today.
RUN = {
    "id": "0f9e3c2a-6b1d-4e7f-a2c8-5d4b3a2f1e60", "start": "2026-10-21T04:00",
    "read": 1284, "committed": 1240, "skipped": 41, "refused": 3,
    "with_visit": 1187, "without_visit": 49, "ambiguous": 4,
    "rows": [("MEASUREMENT", 2210), ("CONDITION_OCCURRENCE", 412), ("DRUG_EXPOSURE", 388), ("FACT_RELATIONSHIP", 41), ("PERSON", 23)],
    "concept_zero": {"MEASUREMENT": 9, "CONDITION_OCCURRENCE": 3, "DRUG_EXPOSURE": 5},
    "derived": [("VISIT_OCCURRENCE", 702), ("OBSERVATION_PERIOD", 318), ("CONDITION_ERA", 377), ("DRUG_ERA", 351)],
    "persons": (23, 12),
    "deletions": {"checked": 26, "extant": 21, "removed": 4, "kept": 0, "unknown": 1, "superseded": 0},
}

# (run id, started, took, tone, compositions read, refused)
RUNS = [
    ("0f9e3c2a-6b1d-4e7f-a2c8-5d4b3a2f1e60", "2026-10-21T04:00", "7 min", "warn", 1284, 3),
    ("8c7b6a59-4e3d-4c2b-91a0-f9e8d7c6b5a4", "2026-10-20T04:00", "6 min", "ok", 1190, 0),
    ("3a2b1c0d-9e8f-4a7b-86c5-d4e3f2a1b0c9", "2026-10-19T04:00", "4 s", "bad", 0, 0),
    ("e5d4c3b2-a1f0-4e9d-8c7b-6a5f4e3d2c1b", "2026-10-18T04:00", "6 min", "warn", 1173, 2),
    ("7f6e5d4c-3b2a-4190-8f7e-6d5c4b3a2918", "2026-10-17T04:00", "5 min", "ok", 1102, 0),
]

REFUSALS = [
    ("1f6e0c2a-3b4d-4e5f-8a9b-0c1d2e3f4a50", "3", "synthetic_measurement", "MEASUREMENT", "unit_concept_id",
     "the unit mm[Hg] has no standard concept in the loaded vocabulary"),
    ("5a2b7c90-1d3e-4f6a-b8c7-d9e0f1a2b3c4", "1", "synthetic_condition", "CONDITION_OCCURRENCE", "condition_start_date",
     "the onset is absent and the mapping gives no alternative"),
    ("9c8d7e6f-5a4b-4c3d-9e2f-1a0b9c8d7e6f", "2", "synthetic_drug", "DRUG_EXPOSURE", "drug_concept_id",
     "the code DEMO-MED-12 is in none of the vocabularies the mapping lists"),
]

# The configuration revisions (console mode, proposed): (n, origin, by, at, reason, where)
REVISIONS = [
    ("5", "operator", "Demo Operator", "2026-10-19T14:12", "Raise the CDR timeout for the night ETL", "Connections › openEHR CDR"),
    ("4", "operator", "Demo Reviewer", "2026-10-16T10:30", "Accept the regional laboratory as an HL7 v2 sender", "HL7 v2"),
    ("3", "rollback", "Demo Operator", "2026-10-15T17:30", "Roll back revision 2: the identifier search matched no person", "FHIR facade"),
    ("2", "operator", "Demo Operator", "2026-10-15T16:05", "Search patients by identifier through the demographic AQL", "FHIR facade"),
    ("1", "file", "ferrobridge.toml", "2026-10-13T06:00", "Read at the first start in console mode", "every section"),
]

# The message Bundle the HL7 v2 face maps HL7_RAW to, after the v2-to-FHIR guide's ORU_R01 message map
# (hl7.fhir.uv.v2mappings 1.0.0): the MessageHeader first, then each resource in the order the run made it.
HL7_RAW = ["MSH|^~\\&|DEMO-LAB|DEMO-HOSP|FERROBRIDGE|DEMO-REGION|20261021094211+0200||ORU^R01^ORU_R01|MSG-2026-1021-0007|P|2.5.1",
           "PID|1||SYN-0042^^^DEMO-HOSP^MR||TEST^SYNTHETIC^^^^^L||19800101|U",
           "OBR|1|ORD-77^DEMO-LAB|FIL-77^DEMO-LAB|29463-7^Body weight^LN|||20261021091500+0200||||||||||||||||||F",
           "OBX|1|NM|29463-7^Body weight^LN||72.4|kg^kilogram^UCUM|||||F"]

HL7_BUNDLE = """{
  "resourceType": "Bundle",
  "type": "message",
  "timestamp": "2026-10-21T09:42:11+02:00",
  "entry": [
    { "fullUrl": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e01",
      "resource": { "resourceType": "MessageHeader",
        "eventCoding": { "system": "http://terminology.hl7.org/CodeSystem/v2-0003", "code": "R01" },
        "source": { "name": "DEMO-LAB", "endpoint": "DEMO-LAB" },
        "destination": [ { "name": "FERROBRIDGE", "endpoint": "FERROBRIDGE",
          "target": { "reference": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e03" },
          "receiver": { "reference": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e04" } } ],
        "sender": { "reference": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e02" } } },
    { "fullUrl": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e02",
      "resource": { "resourceType": "Organization", "name": "DEMO-HOSP" } },
    { "fullUrl": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e03",
      "resource": { "resourceType": "Device", "deviceName": [ { "name": "FERROBRIDGE", "type": "user-friendly-name" } ] } },
    { "fullUrl": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e04",
      "resource": { "resourceType": "Organization", "name": "DEMO-REGION" } },
    { "fullUrl": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e05",
      "resource": { "resourceType": "Provenance",
        "target": [ { "reference": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e01" } ],
        "recorded": "2026-10-21T09:42:11+02:00",
        "agent": [ { "who": { "reference": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e02" } } ] } },
    { "fullUrl": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e06",
      "resource": { "resourceType": "Patient",
        "identifier": [ { "value": "SYN-0042", "type": { "coding": [ { "code": "MR" } ] } } ],
        "name": [ { "family": "TEST", "given": [ "SYNTHETIC" ], "use": "official" } ],
        "birthDate": "1980-01-01", "gender": "unknown" } },
    { "fullUrl": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e07",
      "resource": { "resourceType": "DiagnosticReport", "status": "final",
        "identifier": [ { "value": "ORD-77" }, { "value": "FIL-77" } ],
        "code": { "coding": [ { "system": "http://loinc.org", "code": "29463-7", "display": "Body weight" } ] },
        "subject": { "reference": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e06" },
        "effectiveDateTime": "2026-10-21T09:15:00+02:00",
        "result": [ { "reference": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e09" } ] } },
    { "fullUrl": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e08",
      "resource": { "resourceType": "Specimen",
        "accessionIdentifier": { "value": "ORD-77" },
        "collection": { "collectedDateTime": "2026-10-21T09:15:00+02:00" } } },
    { "fullUrl": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e09",
      "resource": { "resourceType": "Observation", "status": "final",
        "code": { "coding": [ { "system": "http://loinc.org", "code": "29463-7", "display": "Body weight" } ] },
        "subject": { "reference": "urn:uuid:4f0c2d1e-8a3b-4c5d-9e6f-0a1b2c3d4e06" },
        "effectiveDateTime": "2026-10-21T09:15:00+02:00",
        "valueQuantity": { "value": 72.4, "unit": "kilogram", "system": "http://unitsofmeasure.org", "code": "kg" } } }
  ]
}"""
