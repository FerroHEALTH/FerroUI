# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The synthetic deployment every FerroTERM mockup page draws: one server at
terminology.example.org with four FHIR roots, its code systems, value sets
and concept maps, the writes it recorded, and the people who use it.

Synthetic or public only: the concepts shown are from a synthetic code
system (codes SF-nnnn) and from public ones (ISO 3166-1, UCUM, BCP 47, HL7
terminology). SNOMED CT and LOINC appear as listed rows, never with content.
The conformance and benchmark figures are the ones the FerroTERM repository
commits (conformance/tx-ecosystem, bench/bars.json, bench/records)."""

VERSION = "0.1.8"
BASE = "https://terminology.example.org"
FHIR_BASE = BASE + "/r4"
LOCAL = BASE + "/fhir"
CONSOLE = "https://console.terminology.example.org"
IDP = "idp.example.org"

SF = LOCAL + "/CodeSystem/synthetic-findings"
CARE = LOCAL + "/CodeSystem/synthetic-care-settings"
PRIO = LOCAL + "/CodeSystem/synthetic-order-priorities"

# (key, name, canonical, versions, source, content, hierarchy, subsumption, status, concepts, page)
# source: written (through the API, editable), index (a built index), builtin
# (the four registry systems) or directory (a FERROTERM_CODESYSTEMS directory).
CODE_SYSTEMS = [
    ("sf", "Synthetic clinical findings", SF, ["3"], "written", "complete", True, True, "active", 14, "code-system.html"),
    ("care", "Synthetic care settings", CARE, ["1"], "written", "complete", False, False, "draft", 6, "code-system.html"),
    ("prio", "Synthetic order priorities", PRIO, ["2"], "written", "complete", False, False, "active", 4, "code-system.html"),
    ("sct", "SNOMED CT, Netherlands edition", "http://snomed.info/sct", ["20260930", "20260331"], "index", "complete", True, True, "active", None, "code-system-index.html"),
    ("loinc", "LOINC", "http://loinc.org", ["2.83"], "index", "complete", True, True, "active", None, "code-system-index.html"),
    ("icd10", "ICD-10, Dutch translation", "http://hl7.org/fhir/sid/icd-10-nl", ["2026"], "index", "complete", True, True, "active", None, "code-system-index.html"),
    ("iso3166", "ISO 3166-1 country codes", "urn:iso:std:iso:3166", ["48"], "builtin", "complete", False, False, "active", 249, "code-system-builtin.html"),
    ("ucum", "Unified Code for Units of Measure (UCUM)", "http://unitsofmeasure.org", ["2.2"], "builtin", "not-present", False, False, "active", None, "code-system-builtin.html"),
    ("bcp47", "IETF BCP 47 language tags", "urn:ietf:bcp:47", ["2026-08-08"], "builtin", "not-present", False, False, "active", None, "code-system-builtin.html"),
    ("bcp13", "IETF BCP 13 media types", "urn:ietf:bcp:13", [], "builtin", "not-present", False, False, "active", None, "code-system-builtin.html"),
    ("gender", "AdministrativeGender", "http://hl7.org/fhir/administrative-gender", ["4.0.1"], "directory", "complete", False, False, "active", 4, "code-system-builtin.html"),
    ("clinical", "Condition Clinical Status Codes", "http://terminology.hl7.org/CodeSystem/condition-clinical", ["3.0.0"], "directory", "complete", True, True, "active", 6, "code-system-builtin.html"),
    ("obscat", "Observation Category Codes", "http://terminology.hl7.org/CodeSystem/observation-category", ["2.0.0"], "directory", "complete", False, False, "active", 9, "code-system-builtin.html"),
    ("national", "Synthetic national findings", "https://nts.example.org/fhir/CodeSystem/synthetic-national-findings", ["2026-10"], "directory", "complete", True, True, "active", 220, "code-system-builtin.html"),
    ("actcode", "ActCode", "http://terminology.hl7.org/CodeSystem/v3-ActCode", ["9.0.0"], "directory", "complete", True, True, "active", None, "code-system-builtin.html"),
]

SOURCE_WORD = {"written": "Stored on this server", "index": "Offline index", "builtin": "Built-in standard", "directory": "Resource directory"}
SOURCE_TIP = {
    "written": "Written through the FHIR API and kept in the server's resource store; it can be edited and has a history",
    "index": "Served from an index built offline; read-only",
    "builtin": "Ships inside the server; read-only",
    "directory": "Read from a resource directory when the server starts; read-only",
}

# The synthetic hierarchy: (code, display, parent, inactive, Dutch display).
FINDINGS = [
    ("SF-1000", "Synthetic finding", None, False, "Synthetische bevinding"),
    ("SF-1100", "Finding of the skin", "SF-1000", False, "Bevinding van de huid"),
    ("SF-1110", "Rash", "SF-1100", False, "Uitslag"),
    ("SF-1120", "Itch", "SF-1100", False, "Jeuk"),
    ("SF-1200", "Finding of the chest", "SF-1000", False, "Bevinding van de borst"),
    ("SF-1210", "Cough", "SF-1200", False, "Hoest"),
    ("SF-1211", "Dry cough", "SF-1210", False, "Droge hoest"),
    ("SF-1212", "Productive cough", "SF-1210", False, "Productieve hoest"),
    ("SF-1220", "Wheeze", "SF-1200", False, "Piepende ademhaling"),
    ("SF-1300", "General finding", "SF-1000", False, "Algemene bevinding"),
    ("SF-1310", "Fever", "SF-1300", False, "Koorts"),
    ("SF-1320", "Fatigue", "SF-1300", False, "Vermoeidheid"),
    ("SF-1390", "Malaise, old code", "SF-1300", True, "Malaise, oude code"),
    ("SF-1400", "Finding of the abdomen", "SF-1000", False, "Bevinding van de buik"),
]

# (key, title, canonical, version, source, status, page, summary of what it selects)
VALUE_SETS = [
    ("resp", "Synthetic respiratory findings", LOCAL + "/ValueSet/synthetic-respiratory", "2", "written", "active", "value-set.html", "is-a SF-1200, less SF-1220"),
    ("skin", "Synthetic skin findings", LOCAL + "/ValueSet/synthetic-skin", "1", "written", "active", "value-set.html", "is-a SF-1100"),
    ("sfall", "Every synthetic finding", LOCAL + "/ValueSet/synthetic-findings-all", "3", "written", "active", "value-set.html", "the whole code system"),
    ("care", "Synthetic care settings", LOCAL + "/ValueSet/synthetic-care-settings", "1", "written", "draft", "value-set.html", "the whole code system"),
    ("benelux", "Benelux countries", LOCAL + "/ValueSet/benelux-countries", "1", "written", "draft", "value-set.html", "3 codes of ISO 3166-1"),
    ("weight", "Units for body weight", LOCAL + "/ValueSet/body-weight-units", "1", "written", "active", "value-set.html", "3 UCUM codes"),
    ("prio", "Synthetic order priorities", LOCAL + "/ValueSet/synthetic-order-priorities", "2", "written", "active", "value-set.html", "the whole code system"),
    ("referral", "Referral reasons", LOCAL + "/ValueSet/referral-reasons", "4", "written", "active", "value-set-finding.html", "6 codes of Synthetic national findings"),
    ("gender", "AdministrativeGender", "http://hl7.org/fhir/ValueSet/administrative-gender", "4.0.1", "directory", "active", "value-set-loaded.html", "the whole code system"),
    ("clinical", "Condition Clinical Status Codes", "http://hl7.org/fhir/ValueSet/condition-clinical", "4.0.1", "directory", "active", "value-set-loaded.html", "the whole code system"),
    ("obscat", "Observation Category Codes", "http://hl7.org/fhir/ValueSet/observation-category", "4.0.1", "directory", "active", "value-set-loaded.html", "the whole code system"),
    ("langs", "All Languages", "http://hl7.org/fhir/ValueSet/all-languages", "4.0.1", "directory", "active", "value-set-loaded.html", "every BCP 47 tag"),
    ("mime", "MimeType", "http://hl7.org/fhir/ValueSet/mimetypes", "4.0.1", "directory", "active", "value-set-loaded.html", "every BCP 13 type"),
]

# (key, title, canonical, version, source, status, from, to, mappings, page)
CONCEPT_MAPS = [
    ("legacy", "Legacy findings to synthetic findings", LOCAL + "/ConceptMap/legacy-findings", "2", "written", "active", "Legacy findings", "Synthetic clinical findings", 7, "concept-map.html"),
    ("act", "Care settings to ActCode", LOCAL + "/ConceptMap/care-settings-actcode", "1", "written", "draft", "Synthetic care settings", "ActCode", 6, "concept-map.html"),
    ("gender", "v2 map for AdministrativeGender", "http://hl7.org/fhir/ConceptMap/cm-administrative-gender-v2", "4.0.1", "directory", "draft", "AdministrativeGender", "v2 table 0001", 5, "concept-map-loaded.html"),
    ("clinical", "Condition clinical status to synthetic problem status", LOCAL + "/ConceptMap/clinical-to-problem-status", "1", "written", "active", "Condition Clinical Status Codes", "Synthetic problem status", 6, "concept-map.html"),
]

LEGACY = "https://terminology.example.org/fhir/CodeSystem/legacy-findings"

# Legacy findings to synthetic findings: (source, source display, relationship, target, target display, comment)
LEGACY_MAPPINGS = [
    ("LF-01", "Cough, any", "equivalent", "SF-1210", "Cough", ""),
    ("LF-02", "Cough, dry", "equivalent", "SF-1211", "Dry cough", ""),
    ("LF-03", "Cough with sputum", "equivalent", "SF-1212", "Productive cough", ""),
    ("LF-04", "Chest complaint", "narrower", "SF-1220", "Wheeze", "Only the wheeze part of the old code survives."),
    ("LF-05", "Skin complaint", "wider", "SF-1100", "Finding of the skin", ""),
    ("LF-06", "Raised temperature", "equivalent", "SF-1310", "Fever", ""),
    ("LF-07", "Feeling unwell", "unmatched", None, None, "No synthetic code carries this meaning."),
]

# The R4 ConceptMap equivalence codes (http://hl7.org/fhir/concept-map-equivalence),
# each read as the target's relation to the source, in plain words.
REL_WORD = {
    "relatedto": "related to",
    "equivalent": "equivalent",
    "equal": "equal",
    "wider": "the target is wider",
    "subsumes": "the target subsumes it",
    "narrower": "the target is narrower",
    "specializes": "the target specialises it",
    "inexact": "inexact",
    "unmatched": "maps to nothing",
    "disjoint": "disjoint",
}

# The writes the server recorded, newest first, from each type's _history:
# (type, title, version, interaction, hms or date, page)
WRITES = [
    ("ValueSet", "Synthetic respiratory findings", "2", "update", "09:29:02", "value-set-history.html"),
    ("CodeSystem", "Synthetic clinical findings", "3", "update", "08:41:40", "code-system-history.html"),
    ("ConceptMap", "Care settings to ActCode", "1", "create", "06:12:15", "concept-map-history.html"),
    ("ValueSet", "Benelux countries", "1", "create", "2026-10-20T16:05", "value-set-history.html"),
    ("ValueSet", "Units for body weight", "1", "create", "2026-10-19T11:30", "value-set-history.html"),
    ("CodeSystem", "Synthetic care settings", "1", "create", "2026-10-16T14:02", "code-system-history.html"),
    ("ConceptMap", "Legacy findings to synthetic findings", "2", "update", "2026-10-14T10:18", "concept-map-history.html"),
]

# The versions of Synthetic clinical findings (version, when, interaction, source system)
SF_HISTORY = [
    ("3", "2026-10-21T08:41", "update", None),
    ("2", "2026-10-09T15:20", "update", None),
    ("1", "2026-09-30T10:02", "create", None),
]

# The four roots and what each declares (from the FerroTERM console's About
# screen over the e2e fixture, with this deployment's base).
OPERATIONS = [
    ("$versions", ["declared"] * 4),
    ("$cache-control", ["declared"] * 4),
    ("$closure", ["declared", "declared", "declared", None]),
    ("CodeSystem/$lookup", ["type", "type", "type, instance", "type, instance"]),
    ("CodeSystem/$validate-code", ["type, instance"] * 4),
    ("CodeSystem/$subsumes", ["type, instance"] * 4),
    ("ValueSet/$expand", ["type, instance"] * 4),
    ("ValueSet/$validate-code", ["type, instance"] * 4),
    ("ConceptMap/$translate", ["type, instance"] * 4),
]

# The HL7 terminology ecosystem suite, from conformance/tx-ecosystem/README.md
# (validator 6.10.4, suite tests v1.9.3): (mode, surface, passed, ran, needs)
ECOSYSTEM = [
    ("general", "/r4", 635, 670, None),
    ("general", "/r4b", 628, 670, None),
    ("general", "/r5", 637, 670, None),
    ("mimetypes", "/r4b", 35, 37, None),
    ("icd-11", "/r4b", 44, 52, "the three ICD-11 files"),
    ("tx.fhir.org", "/r4b", 57, 227, "a LOINC release"),
    ("snomed", "/r4b", 2, 170, "a SNOMED CT edition"),
]

# bench/bars.json: (bench, bar in µs, measured in µs)
BARS = [
    ("operations/lookup", 25, 3), ("operations/validate_code", 25, 4), ("operations/subsumes", 10, 1), ("operations/translate", 100, 16),
    ("expand/isa_root_page_10", 400, 66), ("http/lookup", 100, 13), ("http/validate_code", 50, 7), ("http/expand_page_10", 750, 129), ("http/translate", 400, 120),
]
