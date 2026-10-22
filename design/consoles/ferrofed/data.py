# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The one synthetic federation every FerroFED mockup page draws: no real
organisation, person, endpoint or certificate. Organisations are invented
Dutch place names (Voorbeeldstad, Proefdorp, Testmeer); every host is under
example.org; every OID is under the 2.999 example arc; every ehr_id is a
made-up UUID; the operators are Demo Operator and Demo Reviewer. Every run
of nine digits on a page must fail the BSN eleven-proof (build.py checks).

Every page derives its organisations, nodes, endpoints, services, routes,
incidents, stored queries and revisions from this module, so a count or a
state reads the same on every page."""

FEDERATION = "demo-regio-noord"
VERSION = "0.0.12"
SPEC = "1.0.0"
SPEC_URL = "https://syntaric.github.io/openehr-federation-spec/federation-aql/1.0/"
BOOK_URL = "https://ferrofed.eu/docs/"
SUPPORT_MAIL = "info@cadasto.com"
GATEWAY_URL = "https://gateway.demo-regio.example.org"
JWKS_URI = GATEWAY_URL + "/.well-known/jwks.json"

# ---- the state vocabulary (family rule 12) ----

# The state as a mark says it, capitalised; prose lower-cases it.
STATE_WORD = {"ok": "Reachable", "warn": "Degraded", "bad": "Unreachable", "neutral": "Not probed", "muted": "Excluded"}
STATE_ORDER = {"bad": 0, "warn": 1, "neutral": 2, "ok": 3, "muted": 4}


def word(tone):
    return STATE_WORD[tone].lower()


def worst(tones):
    for t in ("bad", "warn", "neutral", "ok", "muted"):
        if t in tones:
            return t
    return "neutral"


# The connection types of an endpoint: the plain name first, the registry's
# code as a quiet hint.
CONN = {"query": "openehr-rest-query", "fhir": "hl7-fhir-rest"}
CONN_WORD = {"query": "openEHR Query API", "fhir": "FHIR REST"}

NOW = "09:42:11"
LAST_PROBE = "09:41:58"

# ---- organisations, their nodes and each node's endpoints ----
# An organisation manages at least one endpoint (registry §8.1), so it is
# created together with its first node and that node's first endpoint.

ORGS = [
    {"slug": "zvs", "id": "org-zvs", "name": "Ziekenhuis Voorbeeldstad", "standing": "active", "added": "1", "nodes": [
        {"id": "node-a", "sid": "cdr-a.example.org", "product": "FerroEHR 4.3.3", "added": "1", "tone": "ok", "p50": 38, "endpoints": [
            {"id": "node-a-pub", "kind": "query", "url": "https://cdr-a.example.org/openehr", "tone": "ok", "p50": 38, "cred": ("OAuth 2.0", "console")},
            {"id": "node-a-fhir", "kind": "fhir", "url": "https://cdr-a.example.org/fhir", "tone": "ok", "p50": 22, "cred": None},
        ]},
        {"id": "node-a-lab", "sid": "cdr-a-lab.example.org", "product": "EHRbase 2.36.0", "added": "6", "tone": "ok", "p50": 74, "endpoints": [
            {"id": "node-a-lab-pub", "kind": "query", "url": "https://cdr-a-lab.example.org/ehrbase/rest/openehr", "tone": "ok", "p50": 74, "cred": None},
        ]},
    ]},
    {"slug": "tko", "id": "org-tko", "name": "Testkliniek Oost", "standing": "active", "added": "1", "nodes": [
        # node-b is degraded for one cause: its endpoint answers 503 since 09:38.
        {"id": "node-b", "sid": "cdr-b.example.org", "product": "EHRbase 2.36.0", "added": "1", "tone": "warn", "p50": 388, "cause": "503", "since": "09:38", "endpoints": [
            {"id": "node-b-pub", "kind": "query", "url": "https://cdr-b.example.org/ehrbase/rest/openehr", "tone": "warn", "p50": 388, "cred": ("Basic", "file")},
        ]},
    ]},
    {"slug": "zgp", "id": "org-zgp", "name": "Zorggroep Proefdorp", "standing": "active", "added": "1", "nodes": [
        {"id": "node-c", "sid": "cdr-c.example.org", "product": "FerroEHR 4.3.3", "added": "1", "tone": "ok", "p50": 61, "endpoints": [
            {"id": "node-c-pub", "kind": "query", "url": "https://cdr-c.example.org/openehr", "tone": "ok", "p50": 61, "cred": ("OAuth 2.0", "console")},
        ]},
    ]},
    {"slug": "rdw", "id": "org-rdw", "name": "Revalidatiecentrum Demo-West", "standing": "active", "added": "6", "nodes": [
        {"id": "node-d", "sid": "cdr-d.example.org", "product": "EHRbase 2.36.0", "added": "6", "tone": "ok", "p50": 112, "endpoints": [
            {"id": "node-d-pub", "kind": "query", "url": "https://cdr-d.example.org/ehrbase/rest/openehr", "tone": "ok", "p50": 112, "cred": ("Nuts", "console")},
        ]},
    ]},
    {"slug": "htm", "id": "org-htm", "name": "Huisartsenpost Testmeer", "standing": "excluded", "added": "1", "nodes": [
        {"id": "node-e", "sid": "cdr-e.example.org", "product": "FerroEHR 4.3.2", "added": "1", "tone": "muted", "p50": None, "endpoints": [
            {"id": "node-e-pub", "kind": "query", "url": "https://cdr-e.example.org/openehr", "tone": "muted", "p50": None, "cred": ("OAuth 2.0", "console")},
        ]},
    ]},
]
EXCLUDED_WHY = "CDR maintenance, 21 to 23 October"

# The organisation the revision fixture adds (revision 8): created with its
# first node and that node's first endpoint, never empty.
ZBT = {"slug": "zbt", "id": "org-zbt", "name": "Zorgcentrum Beta-Testerveen", "standing": "active", "added": "8", "nodes": [
    {"id": "node-f", "sid": "cdr-f.example.org", "product": "FerroEHR 4.3.3", "added": "8", "tone": "neutral", "p50": None, "endpoints": [
        {"id": "node-f-pub", "kind": "query", "url": "https://cdr-f.example.org/openehr", "tone": "neutral", "p50": None, "cred": None},
    ]},
]}

for _o in ORGS + [ZBT]:
    for _n in _o["nodes"]:
        _n["org"] = _o
        for _ep in _n["endpoints"]:
            _ep["node"] = _n
            _ep["conn"] = CONN[_ep["kind"]]
            _ep["probe"] = LAST_PROBE if _ep["tone"] not in ("muted", "neutral") else None
        _n["probe"] = LAST_PROBE if _n["tone"] not in ("muted", "neutral") else None


def nodes(include_zbt=False):
    return [n for o in ORGS + ([ZBT] if include_zbt else []) for n in o["nodes"]]


def endpoints(kind=None, include_zbt=False):
    return [ep for n in nodes(include_zbt) for ep in n["endpoints"] if kind is None or ep["kind"] == kind]


def node(nid):
    return next(n for n in nodes(True) if n["id"] == nid)


def org(slug):
    return next(o for o in ORGS + [ZBT] if o["slug"] == slug)


def endpoint(eid):
    return next(ep for ep in endpoints(include_zbt=True) if ep["id"] == eid)


def org_tone(o):
    if o["standing"] != "active":
        return "muted"
    live = [n["tone"] for n in o["nodes"] if n["tone"] != "muted"]
    return worst(live) if live else "muted"


N_ORGS = len(ORGS)
N_NODES = len(nodes())
N_ENDPOINTS = len(endpoints())
# The OPTIONS document lists the openEHR Query API endpoints only: a native
# FHIR endpoint is never one (registry, Native FHIR endpoints).
QUERY_ENDPOINTS = endpoints("query")
IN_SERVICE = [ep for ep in QUERY_ENDPOINTS if ep["tone"] != "muted"]


def node_counts():
    c = {}
    for n in nodes():
        c[n["tone"]] = c.get(n["tone"], 0) + 1
    return c


def org_counts():
    c = {}
    for o in ORGS:
        t = org_tone(o)
        c[t] = c.get(t, 0) + 1
    return c


# ---- the services the gateway asks ----
# (group, key, name, role in plain words, section, URL, tone, p50, approval kind, what it would enable when not set)
SERVICES = [
    ("Identity", "pixm", "PIXm resolver", "finds each node's EHR for a patient", "[pixm]", "https://pix.example.org/fhir/", "ok", 14, "identity-services", None),
    ("Identity", "pdqm", "PDQm demographics", "finds the master identity first", "[pdqm]", "https://pdq.example.org/fhir/", "ok", 22, "identity-services", None),
    ("Identity", "pmir", "PMIR identity feed", "drops stale bindings on a merge", "[pmir]", "https://pmir.example.org/fhir", "ok", 26, "identity-services", None),
    ("Localization", "nvi", "NVI", "finds which nodes hold the patient", "[nl_gf.nvi]", "https://nvi.example.org/fhir", "ok", 31, "localization", None),
    ("Localization", "xcpd", "XCPD", "", "[xcpd]", None, "unset", None, "localization", "would localize patients in an IHE community"),
    # Mitz is degraded for one cause: it answers 503 since 09:31.
    ("Consent", "mitz", "Mitz", "asks consent before a node is asked", "[nl_gf.mitz]", "https://mitz.example.org/geslotenautorisatievraag", "warn", 180, "consent", None),
    ("Records and terminology", "arr", "Audit record repository", "keeps the ATNA audit records", "[audit]", "tls://arr.example.org:6514", "ok", 9, None, None),
    ("Records and terminology", "term", "Terminology server", "translates codes on the FHIR face", "[eehrxf.terminology]", "https://term.demo-regio.example.org/fhir", "neutral", None, None, None),
    ("Directory", "mcsd", "Care services directory", "", "[registry.mcsd]", None, "unset", None, None, "would read organisations and nodes from an mCSD directory"),
]
SERVICE_GROUPS = ["Identity", "Localization", "Consent", "Records and terminology", "Directory"]
MITZ_SINCE = "09:31"
SET_UP = [s for s in SERVICES if s[6] != "unset"]


def service_counts():
    c = {}
    for s in SET_UP:
        c[s[6]] = c.get(s[6], 0) + 1
    return c


def status_counts():
    """The topology's Status menu counts one population: every node and
    every service card, each once."""
    c = {t: 0 for t in ("ok", "warn", "bad", "neutral", "muted")}
    for n in nodes():
        c[n["tone"]] += 1
    for s in SET_UP:
        c[s[6]] += 1
    return c


# ---- integrity incidents ----
EHR_A = "7c1e4a2b-0d3f-4e5a-9b61-2f8d0c5a1e07"
EHR_B = "c2a9f0d4-71b8-4c2e-8a3f-5d6e9b0a4c11"
EHR_C = "0f4d8e2a-6b1c-4a7e-b39d-8e2f1a6c0d55"
# Plain words first, the log's kind as a quiet hint.
INCIDENT_WORD = {
    "EhrIdCollision": "Two nodes claim one EHR id",
    "IndexInsertCollision": "A second claim on an indexed EHR id",
    "LearnedCreatingSystemConflict": "One creating system seen at two nodes",
    "PrescriptionIdentifierCollision": "Two nodes hold one prescription",
    "RegisteredCreatingSystemConflict": "A learned route against the registry",
}
# (kind, detected, ehr_id, nodes, description, resolution)
INCIDENTS_OPEN = [
    ("EhrIdCollision", "2026-10-21T09:12", EHR_A, ["node-b", "node-c"],
     "Requests for this EHR id that name no node are refused (409) until it is cleared.", None),
    ("LearnedCreatingSystemConflict", "2026-10-21T08:47", None, ["node-a", "node-d"],
     "ris-west.example.org answered from both nodes, so its learned route is withdrawn and neither node is routed on.", None),
]
INCIDENTS_RESOLVED = [
    ("EhrIdCollision", "2026-10-20T16:05", EHR_B, ["node-a", "node-c"], "Cleared at gateway-0 and gateway-1.", "Demo Operator, 2026-10-20 16:40"),
    ("IndexInsertCollision", "2026-10-19T11:20", EHR_C, ["node-d"], "The second claim was refused and the first owner kept it.", "Resolved by the gateway; no clear needed"),
]
# Counts since the replica started, as ferrofed_integrity_incidents_total counts them.
INCIDENT_KINDS = [
    ("EhrIdCollision", 2), ("IndexInsertCollision", 1), ("LearnedCreatingSystemConflict", 1),
    ("PrescriptionIdentifierCollision", 0), ("RegisteredCreatingSystemConflict", 0),
]

# ---- the creating_system_id routes ----
# (creating_system_id, ground, node, endpoint); a node's own system_id is its
# route, and every route goes to an openEHR Query API endpoint.
ROUTES = [(n["sid"], "node", n["id"], n["endpoints"][0]["id"]) for n in nodes()] + [
    ("legacy-zvs.example.org", "registered", "node-a", "node-a-pub"),
    ("his-legacy.zvs.example.org", "registered", "node-a", "node-a-pub"),
    ("lab-zvs.example.org", "learned", "node-a", "node-a-pub"),
    ("ris-tko.example.org", "learned", "node-b", "node-b-pub"),
    ("gp-htm.example.org", "registered", "node-e", "node-e-pub"),
    ("rehab-west.example.org", "learned", "node-d", "node-d-pub"),
    ("pharmacy-zgp.example.org", "learned", "node-c", "node-c-pub"),
] + [("ward-%02d.zvs.example.org" % i, "learned", "node-a", "node-a-pub") for i in range(1, 16)]


def routes_of(nid):
    order = {"node": 0, "registered": 1, "learned": 2}
    return sorted([r for r in ROUTES if r[2] == nid], key=lambda r: (order[r[1]], r[0]))


EEHRXF = [
    "patient-summary-advance-directives", "patient-summary-allergies-and-intolerances", "patient-summary-care-plans",
    "patient-summary-immunisations", "patient-summary-medical-devices-and-implants", "patient-summary-medication-summary",
    "patient-summary-observation-results", "patient-summary-pregnancy-history", "patient-summary-problems",
    "patient-summary-procedures", "patient-summary-social-history",
]

# Stored queries of the store, A to Z: (qualified name, [versions newest first], newest stored)
STORED = [
    ("org.demo-regio::allergies", ["1.0.0"], "2026-09-21T11:10"),
    ("org.demo-regio::encounters_last_year", ["0.9.0"], "2026-09-15T09:00"),
    ("org.demo-regio::lab_results_recent", ["1.0.3", "1.0.2"], "2026-10-02T08:31"),
    ("org.demo-regio::medication_active", ["1.4.0"], "2026-09-28T16:45"),
    ("org.demo-regio::problem_list", ["2.0.0", "1.2.1", "1.2.0"], "2026-10-09T14:02"),
    ("org.demo-regio::vital_signs", ["1.1.0", "1.0.0"], "2026-10-14T10:20"),
]
N_STORED = len(STORED) + len(EEHRXF)

AQL_VITALS = (
    "SELECT\n"
    "  c/context/start_time/value AS recorded,\n"
    "  o/data[at0001]/events[at0006]/data[at0003]/items[at0004]/value/magnitude AS systolic\n"
    "FROM EHR e\n"
    "  CONTAINS COMPOSITION c\n"
    "  CONTAINS OBSERVATION o[openEHR-EHR-OBSERVATION.blood_pressure.v2]\n"
    "WHERE e/ehr_status/subject/external_ref/id/value = $patient\n"
    "  AND e/ehr_status/subject/external_ref/namespace = $namespace\n"
    "ORDER BY c/context/start_time/value DESC"
)

# ---- the revision fixture: one sequence for the whole configuration flow ----
# (n, origin, by, at, reason, what it changes)
# 1 to 7 ran before today; revision 7 runs at 09:42 on every resting page.
# Today: 8 adds Zorgcentrum Beta-Testerveen (you, saved 09:51, applied 09:58);
# 9 is your Federation behaviour change, rebased after a 409 because 8 was
# applied while you edited (applied 10:01, both settings wait for the next
# start); 10 sets the credentials of node-a-lab-pub (saved 10:02, approved
# 10:09 by Demo Reviewer, applied 10:12:42 on trial, confirmed 10:13:58).
# History and import are read at 10:20, with 10 running. The
# not-confirmed branch (24.12) has the gateway save 9's content as 11.
REVISIONS = [
    ("1", "file", "ferrofed.toml", "2026-10-06T08:00", "Read at the first start with config.ui = \"managed\"", "4 organisations, their nodes and endpoints, the budgets and the services"),
    ("2", "operator", "Demo Operator", "2026-10-08T13:15", "Turn on the Mitz pre-filter for the pilot", "the Mitz pre-filter"),
    ("3", "import", "Demo Reviewer", "2026-10-12T09:48", "Budgets as merged in the config repository", "the overall timeout, 4,000 ms"),
    ("4", "operator", "Demo Operator", "2026-10-15T17:02", "Rotate the NVI client key", "the NVI client key"),
    ("5", "rollback", "Demo Operator", "2026-10-15T17:30", "Roll back revision 4: the NVI rejected the new client key", "the NVI client key, back to revision 3's"),
    ("6", "operator", "Demo Reviewer", "2026-10-16T10:05", "Add Revalidatiecentrum Demo-West, and the laboratory CDR of Ziekenhuis Voorbeeldstad", "Revalidatiecentrum Demo-West with node-d; node-a-lab"),
    ("7", "operator", "Demo Operator", "2026-10-19T14:12", "Raise the per-node timeout for the pilot's larger queries", "the per-node timeout, 2,000 to 2,500 ms"),
]
FUTURE = [
    ("8", "operator", "Demo Operator", "2026-10-21T09:51", "Zorgcentrum Beta-Testerveen joins the federation in November", "Zorgcentrum Beta-Testerveen with node-f"),
    ("9", "operator", "Demo Operator", "2026-10-21T10:00", "The pilot's larger queries need 3 s per node", "the per-node timeout 3,000 ms and the overall 5,000 ms"),
    ("10", "operator", "Demo Operator", "2026-10-21T10:02", "Credentials for the laboratory CDR, client registered at its authorization server", "the credentials of node-a-lab-pub"),
]
RUNNING = "7"
RUNNING_AT = "2026-10-19T14:12"
RUNNING_BY = "Demo Operator"
REASON_8 = FUTURE[0][4]
REASON_9 = FUTURE[1][4]
REASON_10 = FUTURE[2][4]

# A synthetic patient identifier for the query console: a value in the
# example namespace, never a BSN.
PATIENT_PARAM = "ffd-demo-0001"
PATIENT_NS = "urn:oid:2.999.1.1"

# ---- a large federation, for the pager, the filters and the big map ----

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


def large_counts(items):
    c = {}
    for o in items:
        c[o["worst"]] = c.get(o["worst"], 0) + 1
    return c
