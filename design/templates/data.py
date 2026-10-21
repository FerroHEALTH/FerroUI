# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The synthetic federation every FerroFED mockup page draws: no real
organisation, person, endpoint or certificate. Organisations are invented
Dutch place names (Voorbeeldstad, Proefdorp, Testmeer); every host is under
example.org; every OID is under the 2.999 example arc; every ehr_id is a
made-up UUID; the operators are Demo Operator and Demo Reviewer. Every run
of nine digits on a page must fail the BSN eleven-proof (build.py checks)."""

FEDERATION = "demo-regio-noord"
VERSION = "0.0.12"
SPEC = "1.0.0"

# (node, organisation id, organisation name, system_id, product, version,
#  endpoints [(id, kind, url)], membership, health, median ms, last seen)
MEMBERS = [
    ("node-a", "org-zvs", "Ziekenhuis Voorbeeldstad", "cdr-a.example.org", "FerroEHR", "4.3.3",
     [("node-a-pub", "query", "https://cdr-a.example.org/openehr"), ("node-a-fhir", "fhir", "https://cdr-a.example.org/fhir")],
     "active", "up", 38, "2026-10-21T09:42"),
    ("node-b", "org-tko", "Testkliniek Oost", "cdr-b.example.org", "EHRbase", "2.36.0",
     [("node-b-pub", "query", "https://cdr-b.example.org/ehrbase/rest/openehr")],
     "active", "failing", 2410, "2026-10-21T09:41"),
    ("node-c", "org-zgp", "Zorggroep Proefdorp", "cdr-c.example.org", "FerroEHR", "4.3.3",
     [("node-c-pub", "query", "https://cdr-c.example.org/openehr")],
     "active", "up", 61, "2026-10-21T09:42"),
    ("node-d", "org-rdw", "Revalidatiecentrum Demo-West", "cdr-d.example.org", "EHRbase", "2.36.0",
     [("node-d-pub", "query", "https://cdr-d.example.org/ehrbase/rest/openehr")],
     "active", "up", 112, "2026-10-21T09:42"),
    ("node-e", "org-htm", "Huisartsenpost Testmeer", "cdr-e.example.org", "FerroEHR", "4.3.2",
     [("node-e-pub", "query", "https://cdr-e.example.org/openehr")],
     "suspended", "unknown", None, None),
]

DEPENDENCIES = [
    ("resolver", "PIXm Manager", "pix.example.org", "up", "14 ms"),
    ("demographics", "PDQm Supplier", "pdq.example.org", "up", "22 ms"),
    ("localizer", "NVI", "nvi.example.org", "up", "31 ms"),
    ("consent", "Mitz", "mitz.example.org", "failing", "503 since 09:38"),
    ("audit", "Audit record repository", "arr.example.org:6514", "up", "spool 0 of 100,000"),
]

# Open integrity incidents: (kind, detected, ehr_id, claimants, description)
EHR_A = "7c1e4a2b-0d3f-4e5a-9b61-2f8d0c5a1e07"
EHR_B = "c2a9f0d4-71b8-4c2e-8a3f-5d6e9b0a4c11"
EHR_C = "0f4d8e2a-6b1c-4a7e-b39d-8e2f1a6c0d55"
INCIDENTS_OPEN = [
    ("EhrIdCollision", "2026-10-21T09:12", EHR_A, ["node-b", "node-c"], "Two nodes claim one ehr_id; requests for it are refused with 409 until it is cleared."),
    ("LearnedCreatingSystemConflict", "2026-10-21T08:47", None, ["node-a", "node-d"], "A creating_system_id learned from node-d is already registered to node-a; node-a keeps it."),
]
INCIDENTS_RESOLVED = [
    ("EhrIdCollision", "2026-10-20T16:05", EHR_B, ["node-a", "node-c"], "Cleared at gateway-0 and gateway-1 by Demo Operator at 2026-10-20 16:40."),
    ("IndexInsertCollision", "2026-10-19T11:20", EHR_C, ["node-d"], "The second insert was refused; the first owner kept it."),
]
INCIDENT_KINDS = [
    ("EhrIdCollision", 3), ("IndexInsertCollision", 1), ("LearnedCreatingSystemConflict", 1),
    ("PrescriptionIdentifierCollision", 0), ("RegisteredCreatingSystemConflict", 0),
]

# The creating_system_id routing table: (creating_system_id, source, node, endpoint)
ROUTES = [
    ("cdr-a.example.org", "node", "node-a", "node-a-pub"),
    ("cdr-b.example.org", "node", "node-b", "node-b-pub"),
    ("cdr-c.example.org", "node", "node-c", "node-c-pub"),
    ("cdr-d.example.org", "node", "node-d", "node-d-pub"),
    ("legacy-zvs.example.org", "registered", "node-a", "node-a-pub"),
    ("lab-zvs.example.org", "learned", "node-a", "node-a-pub"),
    ("ris-tko.example.org", "learned", "node-b", "node-b-pub"),
    ("gp-htm.example.org", "registered", "node-e", "node-e-pub"),
    ("rehab-west.example.org", "learned", "node-d", "node-d-pub"),
    ("pharmacy-zgp.example.org", "learned", "node-c", "node-c-pub"),
]

EEHRXF = [
    "patient-summary-advance-directives", "patient-summary-allergies-and-intolerances", "patient-summary-care-plans",
    "patient-summary-immunisations", "patient-summary-medical-devices-and-implants", "patient-summary-medication-summary",
    "patient-summary-observation-results", "patient-summary-pregnancy-history", "patient-summary-problems",
    "patient-summary-procedures", "patient-summary-social-history",
]

# Stored queries of the store: (qualified name, [versions newest first], stored)
STORED = [
    ("org.demo-regio::vital_signs", ["1.1.0", "1.0.0"], "2026-10-14T10:20"),
    ("org.demo-regio::problem_list", ["2.0.0", "1.2.1", "1.2.0"], "2026-10-09T14:02"),
    ("org.demo-regio::lab_results_recent", ["1.0.3", "1.0.2"], "2026-10-02T08:31"),
    ("org.demo-regio::medication_active", ["1.4.0"], "2026-09-28T16:45"),
    ("org.demo-regio::allergies", ["1.0.0"], "2026-09-21T11:10"),
    ("org.demo-regio::encounters_last_year", ["0.9.0"], "2026-09-15T09:00"),
]

AQL_VITALS = (
    "SELECT\n"
    "  c/context/start_time/value AS recorded,\n"
    "  o/data[at0002]/events[at0003]/data[at0001]/items[at0004]/value/magnitude AS systolic\n"
    "FROM EHR e\n"
    "  CONTAINS COMPOSITION c\n"
    "  CONTAINS OBSERVATION o[openEHR-EHR-OBSERVATION.blood_pressure.v2]\n"
    "WHERE e/ehr_status/subject/external_ref/id/value = $patient\n"
    "  AND e/ehr_status/subject/external_ref/namespace = $namespace\n"
    "ORDER BY c/context/start_time/value DESC"
)

# Revisions, newest first: (n, origin, by, at, reason)
REVISIONS = [
    ("7", "operator", "Demo Operator", "2026-10-19T14:12", "Raise the per-node timeout for node-b's slower link"),
    ("6", "operator", "Demo Reviewer", "2026-10-16T10:05", "Add Revalidatiecentrum Demo-West and its node"),
    ("5", "rollback", "Demo Operator", "2026-10-15T17:30", "Roll back revision 4: the NVI rejected the new client key"),
    ("4", "operator", "Demo Operator", "2026-10-15T17:02", "Rotate the NVI client key"),
    ("3", "import", "Demo Reviewer", "2026-10-12T09:48", "Budgets and organisations as merged in the config repository"),
    ("2", "operator", "Demo Operator", "2026-10-08T13:15", "Turn on the Mitz pre-filter for the pilot"),
    ("1", "file", "ferrofed.toml", "2026-10-06T08:00", "Read at the first start in console mode"),
]

# A synthetic patient identifier for the query console: a value in the
# example namespace, never a BSN.
PATIENT_PARAM = "ffd-demo-0001"
PATIENT_NS = "urn:oid:2.999.1.1"


# ---- organisations, their nodes, and each node's endpoints: FerroFED's
# own model ([[organisation]], [[node]], [[endpoint]], the Federation Tier
# terms). An organisation comes first, then its nodes, then their endpoints.

TONE = {"up": "ok", "failing": "warn", "down": "bad", "unknown": "neutral", "excluded": "muted"}
STATE_WORD = {"ok": "reachable", "warn": "degraded", "bad": "unreachable", "neutral": "not probed", "muted": "excluded"}
SLUG = {"node-a": "zvs", "node-b": "tko", "node-c": "zgp", "node-d": "rdw", "node-e": "htm"}
CONN = {"query": "openehr-rest-query", "fhir": "hl7-fhir-rest"}


def worst(tones):
    for t in ("bad", "warn", "neutral", "ok", "muted"):
        if t in tones:
            return t
    return "neutral"


def make_node(m):
    node, oid, oname, sid, prod, ver, eps, mem, h, ms, seen = m
    endpoints = []
    for ep, kind, url in eps:
        tone = "muted" if mem != "active" else TONE[h]
        lat = ms if kind == "query" else (22 if tone == "ok" else None)
        endpoints.append({"id": ep, "kind": kind, "conn": CONN[kind], "url": url, "tone": tone, "p50": lat if tone not in ("muted", "neutral") else None,
                          "probe": "09:41:58" if tone not in ("muted", "neutral") else None})
    return {"id": node, "sid": sid, "product": f"{prod} {ver}", "endpoints": endpoints, "tone": worst([x["tone"] for x in endpoints])}


ORGS = [{"slug": SLUG[m[0]], "id": m[1], "name": m[2], "standing": m[7], "nodes": [make_node(m)]} for m in MEMBERS]
# Ziekenhuis Voorbeeldstad runs a second CDR for its laboratory.
ORGS[0]["nodes"].append({"id": "node-a-lab", "sid": "cdr-a-lab.example.org", "product": "EHRbase 2.36.0", "tone": "ok",
                         "endpoints": [{"id": "node-a-lab-pub", "kind": "query", "conn": "openehr-rest-query", "url": "https://cdr-a-lab.example.org/ehrbase/rest/openehr",
                                        "tone": "ok", "p50": 74, "probe": "09:41:58"}]})

PLACES = ["Voorbeeldstad", "Proefdorp", "Testmeer", "Demo-West", "Beta-Testerveen", "Oefenburg", "Schetsdam", "Modelveen",
          "Ontwerpwolde", "Pilotstad", "Steekproefdorp", "Concepthaven", "Fictiefveen", "Mockumer", "Proefwijk"]
KINDS = ["Ziekenhuis", "Zorggroep", "Huisartsenpost", "Revalidatiecentrum", "Verpleeghuis", "Kliniek", "GGZ-centrum", "Trombosedienst"]


def large_orgs(n=30):
    out = []
    for i in range(n):
        name = f"{KINDS[i % len(KINDS)]} {PLACES[(i * 7) % len(PLACES)]}" + (f" {['Noord', 'Zuid', 'Oost'][i // 15]}" if i >= 15 else "")
        k = 1 + (i * 5) % 3
        tones = ["ok"] * k
        if i == 4:
            tones[0] = "warn"
        if i == 9:
            tones[-1] = "bad"
        if i == 17:
            tones = ["neutral"]
        if i == 23:
            tones = ["muted"]
        out.append({"name": name, "id": f"org-{i + 1:02d}", "n": len(tones), "worst": worst(tones), "tones": tones,
                    "probe": None if tones in (["neutral"], ["muted"]) else f"09:4{i % 2}:{(i * 13) % 60:02d}"})
    return out
