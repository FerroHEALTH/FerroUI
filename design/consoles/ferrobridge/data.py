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
    ("omocl/synthetic_person.omocl.yml", "PERSON", "openEHR-DEMOGRAPHIC-PERSON.person.v2", "provisioned"),
    ("omocl/synthetic_measurement.omocl.yml", "MEASUREMENT", "openEHR-EHR-OBSERVATION.blood_pressure.v2", "provisioned"),
    ("omocl/synthetic_condition.omocl.yml", "CONDITION_OCCURRENCE", "openEHR-EHR-EVALUATION.problem_diagnosis.v1", "provisioned"),
    ("omocl/synthetic_drug.omocl.yml", "DRUG_EXPOSURE", "openEHR-EHR-INSTRUCTION.medication_order.v3", "console"),
]

# The operational templates: (id, source, file or None, archetypes)
TEMPLATES = [
    ("synthetic.vital_signs.v1", "directory", "templates/synthetic_vital_signs.opt", 4),
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

# The running load's diagnostics: (severity, code, file, line, column, message, mapping)
DIAGNOSTICS = [
    ("warning", "unidirectional-value", "fhirconnect/models/lab_analyte.model.yml", 42, 9,
     "The mapping writes Observation.interpretation from openEHR only, so a $toopenehr drops it.", "lab_analyte › interpretation"),
    ("warning", "search-unbound-parameter", "fhirconnect/synthetic_body_weight.context.yml", 18, 5,
     "The context names the search parameter date, which no projection binds, so the facade does not offer it.", None),
    ("warning", "display-without-terminology", "fhirconnect/models/body_weight.model.yml", 27, 11,
     "The coded value is written with its code as display, because the bridge reaches no terminology server for it.", "body_weight › method"),
]

# A draft the bridge refuses: the same shape.
DRAFT_ERRORS = [
    ("error", "missing-key", "fhirconnect/synthetic_body_weight.context.yml", 3, 1,
     "metadata.version is required (FHIRconnect v1.0.0 header schema).", None),
    ("warning", "search-unbound-parameter", "fhirconnect/synthetic_body_weight.context.yml", 17, 5,
     "The context names the search parameter date, which no projection binds, so the facade does not offer it.", None),
    ("error", "unknown-model", "fhirconnect/synthetic_body_weight.context.yml", 25, 11,
     "The context names the model body_weight_detail, which no file of the set defines.", None),
]

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
  template:
    id: "synthetic.vital_signs.v1"
  archetypes:
    - "openEHR-EHR-OBSERVATION.body_weight.v2"
  start: "openEHR-EHR-OBSERVATION.body_weight.v2"
search:
  parameters:
    - date
    - code
mappings:
  - name: body_weight
    with:
      fhir: "$resource"
      openehr: "$archetype"
  - name: subject
    with:
      fhir: "subject"
      openehr: "$composition/context"
"""

# The draft with the two errors DRAFT_ERRORS names: no version, an unknown model.
DRAFT_YAML = CONTEXT_YAML.replace("  version: 1.2.0\n", "").replace("  - name: subject\n", "  - name: body_weight_detail\n", 1)

MODEL_YAML = """grammar: FHIRConnect/v1.0.0
type: model
metadata:
  name: body_weight
  version: 1.0.0
spec:
  system: FHIR
  version: R4
  openEHRConfig:
    archetype: "openEHR-EHR-OBSERVATION.body_weight.v2"
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
        "meta": { "profile": [ "…/StructureDefinition/synthetic-body-weight" ] },
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
  "meta": { "profile": [ "…/StructureDefinition/synthetic-body-weight" ] },
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
                               ("MSH-7", "Date/time of message", "20261021094211"), ("MSH-9", "Message type", "ORU^R01^ORU_R01"),
                               ("MSH-10", "Message control id", "MSG-2026-1021-0007"), ("MSH-12", "Version id", "2.5.1"), ("MSH-18", "Character set", "UNICODE UTF-8")]),
    ("PID", "Patient identification", [("PID-3", "Patient identifier list", "SYN-0042^^^DEMO-HOSP^MR"), ("PID-5", "Patient name", "TEST^SYNTHETIC^^^^^L"),
                                       ("PID-7", "Date/time of birth", "19800101"), ("PID-8", "Administrative sex", "U")]),
    ("OBR", "Observation request", [("OBR-2", "Placer order number", "ORD-77^DEMO-LAB"), ("OBR-4", "Universal service id", "29463-7^Body weight^LN"),
                                    ("OBR-7", "Observation date/time", "20261021091500")]),
    ("OBX", "Observation result", [("OBX-2", "Value type", "NM"), ("OBX-3", "Observation identifier", "29463-7^Body weight^LN"),
                                   ("OBX-5", "Observation value", "72.4"), ("OBX-6", "Units", "kg^kilogram^UCUM"), ("OBX-11", "Result status", "F")]),
]

# The ETL run report (planned #612): the RunReport the runner prints today.
RUN = {
    "id": "etl-2026-10-21-0400",
    "read": 1284, "committed": 1240, "skipped": 31, "refused": 13,
    "with_visit": 1187, "without_visit": 49, "ambiguous": 4,
    "rows": [("MEASUREMENT", 2210), ("CONDITION_OCCURRENCE", 412), ("DRUG_EXPOSURE", 388), ("PERSON", 318), ("FACT_RELATIONSHIP", 41)],
    "concept_zero": 17,
    "derived": [("VISIT_OCCURRENCE", 702), ("OBSERVATION_PERIOD", 318), ("CONDITION_ERA", 377), ("DRUG_ERA", 351)],
    "persons": (318, 12),
    "deletions": {"checked": 26, "extant": 21, "removed": 4, "kept": 0, "unknown": 1, "superseded": 0},
}

RUNS = [
    ("etl-2026-10-21-0400", "2026-10-21T04:00", "2026-10-21T04:07", "warn", 1284, 13),
    ("etl-2026-10-20-0400", "2026-10-20T04:00", "2026-10-20T04:06", "ok", 1190, 0),
    ("etl-2026-10-19-0400", "2026-10-19T04:00", "2026-10-19T04:01", "bad", 0, 0),
    ("etl-2026-10-18-0400", "2026-10-18T04:00", "2026-10-18T04:06", "warn", 1173, 2),
    ("etl-2026-10-17-0400", "2026-10-17T04:00", "2026-10-17T04:05", "ok", 1102, 0),
]

REFUSALS = [
    ("1f6e0c2a-3b4d-4e5f-8a9b-0c1d2e3f4a50", "3", "synthetic_measurement", "MEASUREMENT", "unit_concept_id",
     "the unit mm[Hg] resolves to no standard concept in the loaded vocabulary"),
    ("5a2b7c90-1d3e-4f6a-b8c7-d9e0f1a2b3c4", "1", "synthetic_condition", "CONDITION_OCCURRENCE", "condition_start_date",
     "the onset is absent and the mapping gives no alternative"),
    ("9c8d7e6f-5a4b-4c3d-9e2f-1a0b9c8d7e6f", "2", "synthetic_drug", "DRUG_EXPOSURE", "drug_concept_id",
     "the code DEMO-MED-12 is in no vocabulary the aliases name"),
]

# The configuration revisions (console mode, proposed): (n, origin, by, at, reason, where)
REVISIONS = [
    ("5", "operator", "Demo Operator", "2026-10-19T14:12", "Raise the CDR timeout for the night ETL", "Connections › openEHR CDR"),
    ("4", "operator", "Demo Reviewer", "2026-10-16T10:30", "Accept the regional laboratory as an HL7 v2 sender", "HL7 v2"),
    ("3", "rollback", "Demo Operator", "2026-10-15T17:30", "Roll back revision 2: the identifier search matched no person", "FHIR facade"),
    ("2", "operator", "Demo Operator", "2026-10-15T16:05", "Search patients by identifier through the demographic AQL", "FHIR facade"),
    ("1", "file", "ferrobridge.toml", "2026-10-13T06:00", "Read at the first start in console mode", "every section"),
]
