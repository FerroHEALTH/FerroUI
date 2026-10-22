# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The one synthetic data source of the FerroPIX mockups. Each patient,
pair, domain, source, pseudonymisation domain, revision and audit act is
defined once here, and every page derives what it shows from it.

No real person: every nine-digit number fails the BSN eleven-proof, every
address is a Voorbeeld- or Teststraat in the 9999 range."""
import datetime as _dt

TODAY = _dt.date(2026, 10, 21)
NOW = "09:42:11"


def bsn_valid(digits):
    weights = [9, 8, 7, 6, 5, 4, 3, 2, -1]
    return sum(w * int(d) for w, d in zip(weights, digits)) % 11 == 0


# ---- request ids: each names one event, once ----

_RIDS = set()


def rid(value):
    assert value not in _RIDS, f"request id {value} reused"
    _RIDS.add(value)
    return value


UNREACHABLE_RID = rid("01HZ8X3K7Q4M")

# ---- matching: the running thresholds and weights (revision 7) ----

THRESH = [("possible_at", "Possible from", "0.60"), ("review_at", "Probable from", "0.80"), ("link_at", "Certain from", "0.95")]
# FHIR R4 Patient match-grade: "certain" is a match considered automatically,
# so a certain pair is linked without review and never queued.
LINK_CERTAIN = True

WEIGHTS = [
    # key, label, comparator in words, agree, disagree
    ("family", "Family name", "Spelled alike: whole from 0.92 alike, half from 0.85", 5.1, -3.1),
    ("given", "Given names", "Spelled alike; an initial that agrees counts half", 4.2, -2.4),
    ("born", "Birth date", "Exact; two digits swapped or one digit off counts half", 6.0, -5.0),
    ("gender", "Gender", "Exact; absent on either side adds nothing", 0.9, -2.0),
    ("postal", "Postal code", "Exact; the first four characters count half", 2.6, -0.8),
    ("street", "Street", "Words in common", 2.2, -0.4),
    ("phone", "Telephone", "Exact after normalising the number", 3.9, -1.2),
]
WEIGHT = {k: (a, d) for k, _, _, a, d in WEIGHTS}
FIELD_LABEL = {k: l for k, l, _, _, _ in WEIGHTS}


def weight(field, kind):
    agree, disagree = WEIGHT[field]
    return {"agree": agree, "half": round(agree / 2, 1), "differ": disagree, "absent": 0.0}[kind]


def score_of(total):
    """The score from the total weight: 0.60 near +3.9, 0.80 near +8.4,
    0.95 near +15.6."""
    return round(1 / (1 + 2 ** (-(total - 2) / 3.2)), 2)


def grade_of(score):
    if score >= 0.95:
        return "certain"
    if score >= 0.80:
        return "probable"
    if score >= 0.60:
        return "possible"
    return None


# ---- people ----
# identifiers: (domain, tail, last fed, linked by, linked-by note)

PERSONS = {
    "mohr": {
        "name": "Mohr, Alice", "born": "1958-01-30", "gender": "female",
        "id": "0199c2f1-5a8e-7c3e-9b41-00000000a001", "short": "0199c2f1⁠…⁠a001", "page": "patient-identifiers",
        "identifiers": [
            ("BSN", "123", "2026-10-20T09:41", "feed", "brp-feed · created this person"),
            ("Hospital MRN", "994", "2026-10-19T14:03", "deterministic", "the BSN agreed"),
            ("Radiology RIS", "077", "2026-06-30T11:20", "merge", "from Mohr, A."),
        ],
    },
    "maiden": {
        "name": "Maiden, Alice", "born": "1958-01-30", "gender": None,
        "id": "0199c2f1-5a8e-79a1-9b41-00000000b002", "short": "0199c2f1⁠…⁠b002", "page": "patient-maiden",
        "identifiers": [("Laboratory", "550", "2026-09-02T08:15", "feed", "lab-lis · created this person")],
    },
    "mohr-a": {
        "name": "Mohr, A.", "born": "1958-01-30", "gender": "female",
        "id": "0199b7d0-1c2e-7a10-8f33-00000000c003", "short": "0199b7d0⁠…⁠c003", "page": None,
        "identifiers": [],
    },
    "alissa": {
        "name": "Mohr, Alissa", "born": "1958-01-30", "gender": "female",
        "id": "0199b5e2-4d1f-7b22-9c10-00000000d008", "short": "0199b5e2⁠…⁠d008", "page": "patient-alissa",
        "identifiers": [("Hospital MRN", "394", "2026-07-11T16:20", "feed", "his-prod · created this person")],
    },
    "petrova": {
        "name": "Petrova, Irina", "born": "1963-02-17", "gender": "female",
        "id": "0199a9b1-7e30-7c44-8d21-00000000e007", "short": "0199a9b1⁠…⁠e007", "page": "patient-petrova",
        "identifiers": [
            ("BSN", "481", "2026-10-21T09:38", "feed", "brp-feed · created this person"),
            ("Laboratory", "207", "2026-10-21T09:38", "deterministic", "the BSN agreed"),
        ],
    },
}

# Pseudonyms of Mohr, Alice, one per pseudonymisation domain.
MOHR_PSEUDONYMS = {"ferroehr-nl": ("q7v", "2026-09-02"), "research-2026-q3": ("8kd", "2026-07-01")}

# ---- the review queue: probable and possible pairs only ----
# Each field: (verdict mark, reading, weight kind). A pair's score is its
# total weight on the running weights.


def _f(mark, reading, kind):
    return (mark, reading, kind)


A, D, H, X = "agree", "differ", "half", "absent"

PAIRS = [
    {
        "key": "mohr", "slug": "compare", "found_days": 12, "wait": "12 d",
        "a": {"person": "mohr", "domain": "BSN", "tail": "123", "updated": "2026-10-20T09:41",
              "values": {"family": "Mohr", "given": "Alice", "born": "1958-01-30", "gender": "female", "postal": "9999 AB", "street": "Voorbeeldstraat 12", "phone": "4421"}},
        "b": {"person": "maiden", "domain": "Laboratory", "tail": "550", "updated": "2026-09-02T08:15",
              "values": {"family": "Maiden", "given": "Alice", "born": "1958-01-30", "gender": None, "postal": "9999 AB", "street": "Voorbeeldstraat 21", "phone": "4421"}},
        "fields": {"family": _f(D, "0.71 alike in spelling", "differ"), "given": _f(A, "exact", "agree"), "born": _f(A, "exact", "agree"),
                   "gender": _f(X, "absent in B", "absent"), "postal": _f(A, "exact", "agree"), "street": _f(D, "half the words", "differ"),
                   "phone": _f(A, "exact", "agree")},
        "paired_by": "postal code and birth date",
    },
    {
        "key": "demir", "slug": "compare-demir", "found_days": 8, "wait": "8 d",
        "a": {"name": "Demir, Ayşe", "domain": "Hospital MRN", "tail": "418", "updated": "2026-10-13T10:02",
              "values": {"family": "Demir", "given": "Ayşe", "born": "1990-03-08", "gender": "female", "postal": "9998 BC", "street": "Teststraat 4", "phone": "1180"}},
        "b": {"name": "Demir, Ayse", "domain": "Laboratory", "tail": "662", "updated": "2026-10-12T16:40",
              "values": {"family": "Demir", "given": "Ayse", "born": "1990-03-08", "gender": None, "postal": "9998 BD", "street": "Teststraat 40", "phone": None}},
        "fields": {"family": _f(A, "exact", "agree"), "given": _f(A, "the same, ignoring the accent", "agree"), "born": _f(A, "exact", "agree"),
                   "gender": _f(X, "absent in B", "absent"), "postal": _f(D, "the first four characters agree", "differ"), "street": _f(D, "a different house number", "differ"),
                   "phone": _f(X, "absent in B", "absent")},
        "paired_by": "birth year and the family name's sound",
    },
    {
        "key": "visser", "slug": "compare-visser", "found_days": 1, "wait": "1 d",
        "a": {"name": "Visser, Noor", "domain": "BSN", "tail": "605", "updated": "2026-10-20T08:12",
              "values": {"family": "Visser", "given": "Noor", "born": "1984-05-21", "gender": "female", "postal": "9997 EF", "street": "Voorbeeldlaan 3", "phone": "7702"}},
        "b": {"name": "Visser, N.", "domain": "Radiology RIS", "tail": "318", "updated": "2026-10-18T15:30",
              "values": {"family": "Visser", "given": "N.", "born": "1984-05-21", "gender": None, "postal": "9997 EF", "street": "Voorbeeldlaan 30", "phone": "7720"}},
        "fields": {"family": _f(A, "exact", "agree"), "given": _f(D, "the initial agrees: half", "half"), "born": _f(A, "exact", "agree"),
                   "gender": _f(X, "absent in B", "absent"), "postal": _f(A, "exact", "agree"), "street": _f(D, "a different house number", "differ"),
                   "phone": _f(D, "two digits differ", "differ")},
        "paired_by": "postal code and birth date",
    },
    {
        "key": "jansen", "slug": "compare-jansen", "found_days": 9, "wait": "9 d",
        "a": {"name": "Jansen, Piet", "domain": "Hospital MRN", "tail": "231", "updated": "2026-10-12T10:20",
              "values": {"family": "Jansen", "given": "Piet", "born": "1971-06-02", "gender": "male", "postal": "9997 CD", "street": "Teststraat 8", "phone": None}},
        "b": {"name": "Janssen, P.", "domain": "Hospital MRN", "tail": "877", "updated": "2026-10-11T15:02",
              "values": {"family": "Janssen", "given": "P.", "born": "1971-06-02", "gender": None, "postal": "9996 CD", "street": "Teststraat 80", "phone": None}},
        "fields": {"family": _f(A, "0.96 alike in spelling: whole", "agree"), "given": _f(D, "the initial agrees: half", "half"), "born": _f(A, "exact", "agree"),
                   "gender": _f(X, "absent in B", "absent"), "postal": _f(D, "a different postal code", "differ"), "street": _f(D, "a different house number", "differ"),
                   "phone": _f(X, "absent in both", "absent")},
        "paired_by": "birth year and the family name's sound",
    },
    {
        "key": "testman", "slug": "compare-testman", "found_days": 2, "wait": "2 d",
        "a": {"name": "Testman, Bram", "domain": "Hospital MRN", "tail": "509", "updated": "2026-10-19T09:15",
              "values": {"family": "Testman", "given": "Bram", "born": "1949-09-23", "gender": "male", "postal": "9995 GH", "street": "Voorbeeldplein 7", "phone": "3311"}},
        "b": {"name": "Testman, Abraham", "domain": "Radiology RIS", "tail": "144", "updated": "2026-10-18T11:47",
              "values": {"family": "Testman", "given": "Abraham", "born": "1949-09-23", "gender": "male", "postal": "9995 GH", "street": "Voorbeeldplein 70", "phone": "3317"}},
        "fields": {"family": _f(A, "exact", "agree"), "given": _f(D, "0.62 alike in spelling", "differ"), "born": _f(A, "exact", "agree"),
                   "gender": _f(A, "exact", "agree"), "postal": _f(A, "exact", "agree"), "street": _f(D, "a different house number", "differ"),
                   "phone": _f(D, "one digit differs", "differ")},
        "paired_by": "postal code and birth date",
    },
    {
        "key": "smit", "slug": "compare-smit", "found_days": 0.25, "wait": "6 h",
        "a": {"name": "Smit, Daan", "domain": "Hospital MRN", "tail": "726", "updated": "2026-10-21T03:20",
              "values": {"family": "Smit", "given": "Daan", "born": "2015-08-09", "gender": "male", "postal": "9994 JK", "street": "Teststeeg 2", "phone": "5050"}},
        "b": {"name": "Smit, D.", "domain": "Radiology RIS", "tail": "933", "updated": "2026-10-18T09:10",
              "values": {"family": "Smit", "given": "D.", "born": "2015-08-09", "gender": None, "postal": "9993 JK", "street": "Teststeeg 20", "phone": "5005"}},
        "fields": {"family": _f(A, "exact", "agree"), "given": _f(D, "the initial agrees: half", "half"), "born": _f(A, "exact", "agree"),
                   "gender": _f(X, "absent in B", "absent"), "postal": _f(D, "a different postal code", "differ"), "street": _f(D, "a different house number", "differ"),
                   "phone": _f(D, "two digits swapped", "differ")},
        "paired_by": "birth year and the family name's sound",
    },
    {
        "key": "devries", "slug": "compare-devries", "found_days": 14, "wait": "14 d",
        "a": {"name": "De Vries, Anna", "domain": "Hospital MRN", "tail": "352", "updated": "2026-10-07T13:05",
              "values": {"family": "De Vries", "given": "Anna", "born": "1990-07-14", "gender": "female", "postal": "9992 LM", "street": "Voorbeeldkade 9", "phone": None}},
        "b": {"name": "Vries, Anna", "domain": "Laboratory", "tail": "819", "updated": "2026-10-06T08:44",
              "values": {"family": "Vries", "given": "Anna", "born": "1990-07-14", "gender": None, "postal": "9991 LM", "street": "Voorbeeldkade 19", "phone": None}},
        "fields": {"family": _f(D, "0.80 alike in spelling", "differ"), "given": _f(A, "exact", "agree"), "born": _f(A, "exact", "agree"),
                   "gender": _f(X, "absent in B", "absent"), "postal": _f(D, "a different postal code", "differ"), "street": _f(D, "a different house number", "differ"),
                   "phone": _f(X, "absent in both", "absent")},
        "paired_by": "postal code and birth date",
    },
    {
        "key": "voorbeeld", "slug": "compare-voorbeeld", "found_days": 7, "wait": "7 d",
        "a": {"name": "Voorbeeld, Carla", "domain": "Laboratory", "tail": "470", "updated": "2026-10-14T07:55",
              "values": {"family": "Voorbeeld", "given": "Carla", "born": "1977-04-30", "gender": "female", "postal": "9990 NP", "street": "Teststraat 15", "phone": "8801"}},
        "b": {"name": "Voorbeeld, C.", "domain": "Hospital MRN", "tail": "188", "updated": "2026-10-13T19:22",
              "values": {"family": "Voorbeeld", "given": "C.", "born": "1977-04-03", "gender": None, "postal": "9990 NR", "street": "Teststraat 51", "phone": "8810"}},
        "fields": {"family": _f(A, "exact", "agree"), "given": _f(D, "the initial agrees: half", "half"), "born": _f(D, "two digits swapped: half", "half"),
                   "gender": _f(X, "absent in B", "absent"), "postal": _f(D, "the first four characters agree", "differ"), "street": _f(D, "a different house number", "differ"),
                   "phone": _f(D, "two digits swapped", "differ")},
        "paired_by": "birth year and the family name's sound",
    },
    {
        "key": "kok", "slug": "compare-kok", "found_days": 4, "wait": "4 d",
        "a": {"name": "Kok, Ruben", "domain": "BSN", "tail": "866", "updated": "2026-10-17T12:30",
              "values": {"family": "Kok", "given": "Ruben", "born": "1966-11-02", "gender": "male", "postal": "9989 ST", "street": "Voorbeeldweg 11", "phone": "6620"}},
        "b": {"name": "Kok, R.", "domain": "Laboratory", "tail": "291", "updated": "2026-10-16T10:05",
              "values": {"family": "Kok", "given": "R.", "born": "1969-11-02", "gender": "male", "postal": "9989 ST", "street": "Voorbeeldweg 11", "phone": "6602"}},
        "fields": {"family": _f(A, "exact", "agree"), "given": _f(D, "the initial agrees: half", "half"), "born": _f(D, "a different year", "differ"),
                   "gender": _f(A, "exact", "agree"), "postal": _f(A, "exact", "agree"), "street": _f(A, "exact", "agree"),
                   "phone": _f(D, "two digits swapped", "differ")},
        "paired_by": "postal code and the family name's sound",
    },
    {
        "key": "nguyen", "slug": "compare-nguyen", "found_days": 3, "wait": "3 d",
        "a": {"name": "Nguyen, Linh", "domain": "BSN", "tail": "940", "updated": "2026-10-18T14:10",
              "values": {"family": "Nguyen", "given": "Linh", "born": "2002-12-01", "gender": "female", "postal": "9988 UV", "street": "Testlaan 6", "phone": "2290"}},
        "b": {"name": "Nguyen, L.", "domain": "Hospital MRN", "tail": "615", "updated": "2026-10-17T09:48",
              "values": {"family": "Nguyen", "given": "L.", "born": "2002-12-10", "gender": None, "postal": "9987 UV", "street": "Testlaan 60", "phone": "2209"}},
        "fields": {"family": _f(A, "exact", "agree"), "given": _f(D, "the initial agrees: half", "half"), "born": _f(D, "two digits swapped: half", "half"),
                   "gender": _f(X, "absent in B", "absent"), "postal": _f(D, "a different postal code", "differ"), "street": _f(D, "a different house number", "differ"),
                   "phone": _f(D, "two digits swapped", "differ")},
        "paired_by": "birth year and the family name's sound",
    },
]

PAIRS.append({
    "key": "hendriks", "slug": "compare-hendriks", "found_days": 2, "wait": "2 d",
    "a": {"name": "Hendriks, Mila", "domain": "Laboratory", "tail": "537", "updated": "2026-10-19T08:30",
          "values": {"family": "Hendriks", "given": "Mila", "born": "1999-02-11", "gender": "female", "postal": "9986 WX", "street": "Teststraat 33", "phone": "4108"}},
    "b": {"name": "Hendrikx, Mirte", "domain": "Radiology RIS", "tail": "402", "updated": "2026-10-18T13:15",
          "values": {"family": "Hendrikx", "given": "Mirte", "born": "1999-11-02", "gender": None, "postal": "9985 WX", "street": "Teststraat 3", "phone": None}},
    "fields": {"family": _f(D, "0.93 alike in spelling: whole", "agree"), "given": _f(D, "0.70 alike in spelling", "differ"), "born": _f(D, "day and month swapped: half", "half"),
               "gender": _f(X, "absent in B", "absent"), "postal": _f(D, "a different postal code", "differ"), "street": _f(D, "a different house number", "differ"),
               "phone": _f(X, "absent in B", "absent")},
    "paired_by": "birth year and the family name's sound",
})

DEFERRED_PAIR = {
    "key": "bakker", "slug": None, "found_days": 10, "wait": "back 2026-10-28", "back": "2026-10-28",
    "a": {"name": "Bakker, Joost", "domain": "Hospital MRN", "tail": "742", "updated": "2026-10-10T11:00", "values": {}},
    "b": {"name": "Bakker, J.", "domain": "Laboratory", "tail": "356", "updated": "2026-10-09T15:40", "values": {}},
    "fields": {"family": _f(A, "exact", "agree"), "given": _f(D, "the initial agrees: half", "half"), "born": _f(A, "exact", "agree"),
               "gender": _f(X, "absent in B", "absent"), "postal": _f(D, "a different postal code", "differ"), "street": _f(D, "a different house number", "differ"),
               "phone": _f(D, "one digit differs", "differ")},
    "paired_by": "postal code and birth date",
}

GRADE_RANK = {"certain": 0, "probable": 1, "possible": 2}
FIELD_ORDER = ["family", "given", "born", "gender", "postal", "street", "phone"]


def _finish(p):
    p["rows"] = [(k, FIELD_LABEL[k], p["fields"][k][1], weight(k, p["fields"][k][2])) for k in FIELD_ORDER]
    p["total"] = round(sum(w for _, _, _, w in p["rows"]), 1)
    p["score"] = score_of(p["total"])
    p["grade"] = grade_of(p["score"])
    for side in ("a", "b"):
        r = p[side]
        if "person" in r:
            r["name"] = PERSONS[r["person"]]["name"]
    p["names"] = (p["a"]["name"], p["b"]["name"])
    return p


for _p in PAIRS + [DEFERRED_PAIR]:
    _finish(_p)
    assert _p["grade"] in ("probable", "possible"), (_p["key"], _p["score"])

PAIRS.sort(key=lambda p: (GRADE_RANK[p["grade"]], -p["score"], -p["found_days"]))
PAIR = {p["key"]: p for p in PAIRS}

PAGE1 = PAIRS[:10]
QUEUE_TOTAL = 38
GRADE_COUNTS = {"probable": 22, "possible": 16}
assert sum(GRADE_COUNTS.values()) == QUEUE_TOTAL
# Pages 2 to 4 hold pairs that wait at most 7 days, so page 1 holds every
# pair that waits longer.
LONG_WAITS = [p for p in PAIRS if p["found_days"] > 7]
OLDEST = sorted(PAIRS, key=lambda p: -p["found_days"])[:5]


def position(key):
    return [p["key"] for p in PAIRS].index(key) + 1


# ---- the dashboard: feeds per hour today, 00:00 to 09:00 ----

FEEDS_FHIR = [40, 25, 18, 15, 20, 45, 95, 160, 205, 168]
FEEDS_V2 = [12, 8, 6, 5, 7, 20, 48, 85, 100, 80]
FED_FHIR = sum(FEEDS_FHIR)
FED_V2 = sum(FEEDS_V2)
FED_TODAY = FED_FHIR + FED_V2
AUTO_LINKED = 1071
QUEUE_DEPTH = [52, 49, 47, 51, 55, 58, 54, 50, 46, 44, 43, 47, 49, 52, 48, 45, 41, 39, 42, 44, 40, 38, 37, 41, 43, 40, 39, 36, 38, 38]
assert QUEUE_DEPTH[-1] == QUEUE_TOTAL

# ---- sources and identifier domains ----
# today: records fed today; v2 senders split the v2 series.

SOURCES = [
    {"name": "brp-feed", "channel": "FHIR", "how": "ITI-104 · client credentials at FerroHEALTH ID", "last": "2026-10-21T09:41", "today": FED_FHIR, "state": ("ok", "active")},
    {"name": "his-prod", "channel": "HL7 v2", "how": "MLLP · HIS^ZKH from 10.20.0.0/24", "last": "2026-10-21T09:40", "today": 275, "state": ("ok", "active")},
    {"name": "lab-lis", "channel": "HL7 v2", "how": "MLLP · LIS^LAB from 10.20.4.12", "last": "2026-10-21T08:15", "today": 96, "state": ("ok", "active")},
    {"name": "ris-prod", "channel": "FHIR", "how": "ITI-104 · client credentials at FerroHEALTH ID", "last": "2026-10-18T17:02", "today": 0, "state": ("warn", "quiet 3 days")},
    {"name": "brp-feed-2", "channel": "FHIR", "how": "ITI-104 · client credentials at FerroHEALTH ID", "last": None, "today": 0, "state": ("neutral", "registered, feeds nothing")},
]
assert sum(s["today"] for s in SOURCES if s["channel"] == "HL7 v2") == FED_V2
SOURCE = {s["name"]: s for s in SOURCES}

DOMAINS = [
    {"name": "BSN", "slug": "domain", "system": "http://fhir.nl/fhir/NamingSystem/bsn", "kind": "authority", "sensitive": True, "source": "brp-feed",
     "records": "412,318", "last": "2026-10-21T09:41", "state": ("ok", "active"), "created": "2025-11-01",
     "about": "the Dutch citizen service number, fed by the BRP.",
     "per_day": [1420, 1380, 1310, 1290, 1350, 1440, 980, 760, 1400, 1370, 1330, 1300, 1360, 1450, 990, 770, 1410, 1390, 1340, 1310, 1370, 1460, 1000, 780, 1420, 1400, 1350, 1320, 1380, 1470],
     "outcomes": [("Accepted", 777), ("Queued for review", 12), ("Refused", 2)], "refused_note": "both without a birth date"},
    {"name": "Hospital MRN", "slug": "domain-mrn", "system": "urn:oid:1.3.6.1.4.1.21367.2017.2.5.99", "kind": "local", "sensitive": False, "source": "his-prod",
     "records": "388,104", "last": "2026-10-21T09:40", "state": ("ok", "active"), "created": "2025-11-01",
     "about": "the hospital information system's medical record numbers.",
     "per_day": [d // 3 for d in (1420, 1380, 1310, 1290, 1350, 1440, 980, 760, 1400, 1370, 1330, 1300, 1360, 1450, 990, 770, 1410, 1390, 1340, 1310, 1370, 1460, 1000, 780, 1420, 1400, 1350, 1320, 1380, 1470)],
     "outcomes": [("Accepted", 266), ("Queued for review", 8), ("Refused", 1)], "refused_note": "without a birth date"},
    {"name": "Laboratory", "slug": "domain-lab", "system": "urn:oid:1.3.6.1.4.1.21367.2017.2.5.100", "kind": "local", "sensitive": False, "source": "lab-lis",
     "records": "96,220", "last": "2026-10-21T08:15", "state": ("ok", "active"), "created": "2025-11-01",
     "about": "the laboratory information system's patient numbers.",
     "per_day": [d // 12 for d in (1420, 1380, 1310, 1290, 1350, 1440, 980, 760, 1400, 1370, 1330, 1300, 1360, 1450, 990, 770, 1410, 1390, 1340, 1310, 1370, 1460, 1000, 780, 1420, 1400, 1350, 1320, 1380, 1470)],
     "outcomes": [("Accepted", 93), ("Queued for review", 3), ("Refused", 0)], "refused_note": ""},
    {"name": "Radiology RIS", "slug": "domain-ris", "system": "urn:oid:1.3.6.1.4.1.21367.2017.2.5.101", "kind": "local", "sensitive": False, "source": "ris-prod",
     "records": "41,007", "last": "2026-10-18T17:02", "state": ("ok", "active"), "created": "2025-11-01",
     "about": "the radiology information system's patient numbers.",
     "per_day": [140, 132, 128, 120, 135, 141, 60, 44, 138, 131, 126, 122, 133, 144, 58, 41, 139, 130, 127, 121, 134, 143, 61, 45, 137, 129, 118, 96, 0, 0],
     "outcomes": [], "refused_note": ""},
    {"name": "Legacy ZIS", "slug": "domain-legacy", "created": "2019-03-01", "about": "the numbers of the hospital system replaced in 2024.", "per_day": [0] * 30, "outcomes": [], "refused_note": "", "system": "urn:oid:1.3.6.1.4.1.21367.2017.2.5.42", "kind": "local", "sensitive": False, "source": None,
     "records": "12,330", "last": "2024-02-01T00:00", "state": ("neutral", "retired")},
]
DOMAINS.sort(key=lambda x: x["name"])
DOMAIN = {d["name"]: d for d in DOMAINS}
assert sum(n for _, n in DOMAIN["BSN"]["outcomes"]) == FED_FHIR
assert sum(n for _, n in DOMAIN["Hospital MRN"]["outcomes"]) == SOURCE["his-prod"]["today"]
assert sum(n for _, n in DOMAIN["Laboratory"]["outcomes"]) == SOURCE["lab-lis"]["today"]
DOMAIN = {d["name"]: d for d in DOMAINS}

# The last feeds of a domain: (date-time, channel, ITI-104 message, outcome, request id). Never a value.
DOMAIN_FEEDS = {
    "BSN": [
        ("2026-10-21T09:41:58", "FHIR", "Add or revise", ("ok", "accepted"), rid("01HZ8K7B2D4F")),
        ("2026-10-21T09:41:12", "FHIR", "Add or revise", ("ok", "accepted"), rid("01HZ8K6T9W1A")),
        ("2026-10-21T09:40:47", "FHIR", "Resolve duplicate", ("ok", "accepted"), rid("01HZ8K6E2E5T")),
        ("2026-10-21T09:39:30", "FHIR", "Add or revise", ("warn", "queued for review"), rid("01HZ8K5577N3")),
        ("2026-10-21T09:38:05", "FHIR", "Add or revise", ("ok", "accepted"), rid("01HZ8K4C8DKQ")),
        ("2026-10-21T09:36:51", "FHIR", "Add or revise", ("bad", "refused: no birth date"), rid("01HZ8K30B6FJ")),
    ],
    "Hospital MRN": [
        ("2026-10-21T09:40:22", "HL7 v2", "ADT^A08", ("ok", "accepted"), rid("01HZ8K5Y1H8M")),
        ("2026-10-21T09:39:01", "HL7 v2", "ADT^A28", ("ok", "accepted"), rid("01HZ8K4V6J2N")),
        ("2026-10-21T09:35:44", "HL7 v2", "ADT^A31", ("warn", "queued for review"), rid("01HZ8K2R4K6P")),
    ],
    "Laboratory": [
        ("2026-10-21T08:15:02", "HL7 v2", "ADT^A08", ("ok", "accepted"), rid("01HZ8F2A9L3Q")),
        ("2026-10-21T08:14:40", "HL7 v2", "ADT^A28", ("ok", "accepted"), rid("01HZ8F1Z3M5R")),
        ("2026-10-21T08:02:11", "HL7 v2", "ADT^A40", ("warn", "queued for review"), rid("01HZ8EP6N7S1")),
    ],
    "Legacy ZIS": [],
    "Radiology RIS": [
        ("2026-10-18T17:02:40", "FHIR", "Add or revise", ("ok", "accepted"), rid("01HZ2QW1M7XA")),
        ("2026-10-18T16:58:12", "FHIR", "Add or revise", ("ok", "accepted"), rid("01HZ2QT8P2LB")),
        ("2026-10-18T16:41:05", "FHIR", "Remove", ("ok", "accepted"), rid("01HZ2QA3R9KC")),
    ],
}

# ---- pseudonymisation domains ----

PSEUDO_DOMAINS = [
    ("ferroehr-nl", "Research export to FerroEHR", "Keyed hash (HMAC-SHA-256)", "231,440", "2025-11-12", ("ok", "active")),
    ("research-2026-q3", "Cohort study, pseudonymised", "Keyed hash (HMAC-SHA-256)", "18,902", "2026-07-01", ("ok", "active")),
    ("dashboard-kpi", "Hospital KPI dashboard", "Random, stored", "402,110", "2025-12-03", ("ok", "active")),
    ("pilot-2025", "Pilot export, closed", "Random, stored", "4,018", "2025-06-10", ("neutral", "retired")),
]
# The pseudonym resolved on the resolve page belongs to Petrova, Irina.
RESOLVE = {"domain": "research-2026-q3", "pseudonym": "c41e-7rw", "person": "petrova", "reason": "Research ticket RES-204: withdrawal of consent"}

# ---- configuration revisions (console mode) ----
# Revision 7 runs. Your thresholds draft becomes revision 8, the added
# blocking key revision 9, and restoring revision 7 from History revision 10.

REVISIONS = [
    {"n": "7", "how": "console", "by": "Demo Steward", "at": "2026-10-14T11:05", "reason": "Raise the probable grade after the September audit",
     "changes": [("Probable from", "0.78", "0.80", "live")]},
    {"n": "6", "how": "console", "by": "Demo Reviewer", "at": "2026-10-13T10:30", "reason": "Weight the telephone less after the pilot",
     "changes": [("Telephone, agree", "+4.4", "+3.9", "live")]},
    {"n": "5", "how": "restore", "by": "Demo Steward", "at": "2026-09-30T15:40", "reason": "Restore revision 3: too many weak pairs", "changes": []},
    {"n": "4", "how": "console", "by": "Demo Reviewer", "at": "2026-09-30T15:12", "reason": "Weight the telephone higher", "changes": []},
    {"n": "3", "how": "console", "by": "Demo Steward", "at": "2026-09-22T10:05", "reason": "Lower the possible grade for the pilot", "changes": []},
    {"n": "2", "how": "console", "by": "Demo Reviewer", "at": "2026-09-15T14:40", "reason": "Weight the postal code higher", "changes": []},
    {"n": "1", "how": "start", "by": "ferropix.toml", "at": "2026-09-01T06:00", "reason": "The file's settings, read when console mode was switched on", "changes": []},
]
REV8 = {"n": "8", "how": "console", "by": "Demo Steward", "at": "2026-10-21T09:42", "reason": "Fewer weak candidates after the October review",
        "changes": [("Probable from", "0.80", "0.82", "live"), ("Possible from", "0.60", "0.65", "live")]}
REV9 = {"n": "9", "how": "console", "by": "Demo Steward", "at": "2026-10-21T09:42", "reason": "Pair records by the given name's sound as well",
        "changes": [("Blocking key: the given name's sound and birth date", "off", "on", "restart")]}

# ---- the audit trail, today ----
# (time, actor, (tone, kind), patient key or a name, reason, request id)

ACTS = [
    ("09:41:55", "Demo Steward", ("warn", "reveal"), "mohr", "Phone call with the laboratory about sample LAB-…550", rid("01HZ8K6M2C3D")),
    ("09:40:30", "Demo Steward", ("neutral", "view"), "mohr", None, rid("01HZ8K5A8E5F")),
    ("09:38:02", "system", ("info", "auto-link"), "petrova", "Certain: the BSN agreed", rid("01HZ8K4ZZG7H")),
    ("09:31:44", "Demo Steward", ("neutral", "defer"), "Bakker, Joost", "Awaiting the laboratory's call back about the birth date", rid("01HZ8JX01J9K")),
    ("09:28:09", "Demo Steward", ("bad", "not a match"), "Kuipers, Sanne", "Different birth date confirmed with the registration desk", rid("01HZ8JT4RL2M")),
    ("09:20:40", "Demo Steward", ("ok", "link"), "De Boer, Kees", "Same telephone and address; the old record held a typo", rid("01HZ8JK2PA5N")),
    ("09:12:51", "Demo Steward", ("warn", "resolve"), "petrova", "Research ticket RES-198: a data quality query", rid("01HZ8J8Q0N4P")),
    ("08:55:17", "Demo Steward", ("neutral", "export"), None, "Weekly audit extract for the DPO", rid("01HZ8HV2WQ6R")),
    ("08:30:00", "Demo Steward", ("neutral", "sign-in"), None, None, rid("01HZ8H0A1S8T")),
]

# Request ids used by single pages.
RID_HISTORY_VERSION = rid("01HZ6A3P1Q2R")
RID_EXPORT = rid("01HZ8K8T5U2V")
RID_RESOLVE = rid("01HZ8K9R3W6X")
RID_REVEAL_REFUSED = rid("01HZ8K9C1Y7Z")
LAB_MESSAGES = [
    ("08:15:02", "ADT^A08", ("ok", "accepted"), "AA"), ("08:14:40", "ADT^A28", ("ok", "accepted"), "AA"),
    ("08:02:11", "ADT^A40", ("warn", "queued for review"), "AA"), ("07:58:30", "ADT^A31", ("bad", "refused: an unknown domain"), "AR"),
]
