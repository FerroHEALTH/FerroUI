# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The bridge's faces (FHIR facade, HL7 v2, OMOP ETL) and its upstreams
(Connections), each a T2 page. A tab panel holds B10 sections, never cards
in a card; the last tab is Settings, a read-only view of ferrobridge.toml in
T3 rows. A face has no primary action of its own, so its title row holds
More alone. B7, the ETL run report, is drawn for the planned screens."""
from common import (
    action_group, badge, button, copy_button, definitions, disclosure, e, empty_state, facts_row, icon, link_button, menu_item, notice, page_heading, pager, pill, refusal,
    shell, table, tabbed, tabs, td, th, when,
)
import tpl
from bridge import json_panel, planned, quiet, section, sections, state
from data import REFUSALS, RUN, RUNS
from settings import on, secret_now, settings_tab, srow


def more_only(items, label, open_=False):
    return action_group("", items, open_=open_, label=label)


# ---- FHIR facade ----

FACADE_TABS = [("overview", "Overview", "facade.html"), ("capability", "Capability", "facade-capability.html", 6), ("security", "Security", "facade-security.html"), ("settings", "Settings", "facade-settings.html")]
FACADE_OFF_TABS = [("overview", "Overview", "facade.html"), ("settings", "Settings", "facade-settings.html")]


def facade_heading(off=False, more_open=False):
    ag = "" if off else more_only([menu_item("Copy the base URL", "copy"), menu_item("Open the CapabilityStatement", "file-code", desc="/fhir/metadata as JSON"),
                                   menu_item("Open SMART discovery", "file-code", desc="The SMART configuration as JSON")], "More actions for the FHIR facade", more_open)
    b = badge("neutral", "Off") if off else state("ok", "on · listening")
    sub = "Off, so the bridge serves no FHIR route." if off else "Serves FHIR R4 over the CDR at <code>/fhir</code>."
    return page_heading("FHIR facade", actions=ag, subtitle=sub, badge=b)


def facade_page(tab, panel, off=False, more_open=False):
    t = tabs("FHIR facade", FACADE_OFF_TABS if off else FACADE_TABS, tab)
    return shell("FHIR facade", "facade", facade_heading(off, more_open) + f'<div class="fui-stack">{tabbed(t, panel)}</div>')


def page_facade(more_open=False):
    facts = facts_row([("Base", "<code>https://bridge.demo-regio.example.org/fhir</code>"), ("Listener", f'<code>0.0.0.0:8080</code> {pill("ok", "Up")}'),
                       ("Contexts", '<a href="mappings.html">9</a>'), ("Resource types", '<a href="facade-capability.html">6</a>'), ("FHIR", "R4 (4.0.1)")])
    serves = definitions([("Interactions", "read, vread, create and update on every resource type, over the CDR; search-type where a search file answers the type"),
                          ("Validation", "<code>$validate</code> on every resource type"), ("Search", "the AQL projections a search file declares beside a context"),
                          ("References", "followed 2 levels deep, from 1 listed base")])
    ops = definitions([("Lane", state("ok", "on")), ("Operations", "<code>$tofhir</code> and <code>$toopenehr</code>; neither commits"),
                       ("Try them", '<a href="playground.html">in the Playground</a>')])
    ident = definitions([("Identity store", f'<code>identity.redb</code> {state("ok", "up")}'), ("EHRs", "writes only into existing ones"),
                         ("Language and territory", "English · Netherlands (en, NL)")])
    return facade_page("overview", sections(section("Facade", facts), section("What it serves", serves), section("Operations", ops), section("Identity", ident)), more_open=more_open)


def page_facade_more():
    return page_facade(more_open=True)


# NOTE: capability.rs: every mapped type takes create, read, vread and update with $validate; search-type only where a search file answers it.
BASE_I = "read, vread, create, update"
CAP = [("Observation", 3, BASE_I + ", search-type", "_id, code, date, patient", "$validate"),
       ("Condition", 2, BASE_I + ", search-type", "_id, code, patient", "$validate"),
       ("MedicationStatement", 1, BASE_I, "–", "$validate"),
       ("AllergyIntolerance", 1, BASE_I, "–", "$validate"),
       ("Encounter", 1, BASE_I, "–", "$validate"),
       ("Procedure", 1, BASE_I, "–", "$validate")]


def page_facade_capability():
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{"file-program.html" if rt == "Observation" else "file-provisioned.html"}">{rt}</a></th>'
            + td("Profiles", str(n), "fui-num", role="meta-q", unit="profiles" if n != 1 else "profile") + td("Interactions", e(i), role="wide") + td("Search", e(s_) if s_ != "–" else '<span class="mk-dash">–</span>', role="sub") + td("Operations", f"<code>{o}</code>" if o != "–" else '<span class="mk-dash">–</span>', role="meta-2") + "</tr>"
            for rt, n, i, s_, o in CAP]
    t = table("Resource types", th("Resource type", "r", "ascending") + th("Profiles", num=True) + th("Interactions") + th("Search parameters") + th("Operations"), rows)
    intro = '<p class="fui-small fui-muted">From the CapabilityStatement at <code>/fhir/metadata</code>. A resource type opens the program that serves it.</p>'
    return facade_page("capability", sections(section("", intro + t)))


DISCOVERY = """{
  "issuer": "https://idp.demo-regio.example.org/realms/bridge",
  "authorization_endpoint": "https://idp.demo-regio.example.org/realms/bridge/protocol/openid-connect/auth",
  "token_endpoint": "https://idp.demo-regio.example.org/realms/bridge/protocol/openid-connect/token",
  "grant_types_supported": [ "authorization_code", "client_credentials" ],
  "code_challenge_methods_supported": [ "S256" ],
  "capabilities": [
    "permission-v1",
    "permission-patient",
    "permission-user",
    "launch-ehr"
  ],
  "scopes_supported": [ "patient/Observation.rs", "patient/Condition.rs", "system/*.rs" ]
}"""


def page_facade_security():
    facts = definitions([("Issuers", "2"), ("Audience", "<code>https://bridge.demo-regio.example.org/fhir</code>"), ("Token profile", "RFC 9068"),
                         ("Capabilities", " ".join(f'<span class="fui-tag">{c}</span>' for c in ("permission-v1", "permission-patient", "permission-user", "launch-ehr")))])
    note = '<p class="fui-small fui-muted">Who may call is set in <a href="configuration-core.html">Configuration › Server and access</a>.</p>'
    doc = section("SMART discovery", f'<p class="fui-small fui-muted">At <code>/fhir/<wbr>.well-known/<wbr>smart-configuration</code>.</p>{json_panel(DISCOVERY, "The discovery document as JSON")}',
                  actions=copy_button("Copy", "the discovery document"))
    return facade_page("security", sections(section("What the facade declares", facts + note), doc))


def facade_rows(hint=False):
    return (srow("Facade", "Mounts the FHIR routes under /fhir.", on(), "facade.enabled", hint=hint)
            + srow("Base URL", "The FHIR base a client reaches; Location headers are written under it.", "<code>https://bridge.demo-regio.example.org/fhir</code>", "facade.base_url")
            + srow("EHRs", "Write only into an EHR the CDR holds, or create one for an unknown subject.", "Only existing ones", "facade.ehr_policy")
            + srow("Identity store", "The file the identity map is kept in.", "<code>identity.redb</code>", "facade.identity_store")
            + srow("Language and territory", "What every commit carries, ISO 639-1 and ISO 3166-1.", "English · Netherlands (en, NL)", "facade.composition_language",
                   keys=("facade.composition_language", "facade.composition_territory"))
            + srow("Search page", "Results per page, and the most a search answers.", "20 · at most 100", "facade.search_count", keys=("facade.search_count", "facade.search_count_max"))
            + srow("Reference bases", "The FHIR bases an absolute reference may be fetched from, and how many levels deep.", "<code>https://ehr.demo-region.example.org/fhir</code> · 2 levels",
                   "facade.references.fetch_bases", keys=("facade.references.fetch_bases", "facade.references.max_depth"))
            + srow("Identifier search", "Resolves patient.identifier through the demographic AQL.", on(False), "facade.identifier_search.enabled")
            + srow("Operations", "Mounts $tofhir, $toopenehr and $validate.", on(), "operations.enabled"))


def page_facade_settings(hint=False):
    return facade_page("settings", settings_tab(facade_rows(hint)))


def page_facade_settings_hint():
    return page_facade_settings(hint=True)


def page_facade_off():
    body = quiet('FHIR clients reach nothing; the mapping set still compiles, and the Playground runs while conversions are on. <a href="facade-settings.html">See its settings</a>')
    return facade_page("overview", body, off=True)


# ---- HL7 v2 ----

HL7_TABS = [("overview", "Overview", "hl7v2.html"), ("settings", "Settings", "hl7v2-settings.html")]


def hl7_page(tab, panel, off=False, tabs_list=None):
    b = badge("neutral", "Off") if off else state("ok", "on · listening")
    sub = "Off, so no MLLP listener runs." if off else "Takes HL7 v2 messages over MLLP and commits each as one transaction."
    h = page_heading("HL7 v2", subtitle=sub, badge=b)
    return shell("HL7 v2", "hl7v2", h + f'<div class="fui-stack">{tabbed(tabs("HL7 v2", tabs_list or HL7_TABS, tab), panel)}</div>')


def page_hl7v2():
    facts = facts_row([("Address", f'<code>0.0.0.0:2575</code> {pill("ok", "Up")} {copy_button("Copy", "the address")}'), ("Character set", "ASCII if MSH-18 is empty"), ("Largest frame", "1 MiB"),
                       ("Unmapped entries", "skipped and counted")])
    senders = [f'<tr><th scope="row">{n}</th></tr>' for n in ("DEMO-HOSP", "DEMO-LAB")]
    st = table("Accepted senders", th("Sending facility (MSH-4)"), senders)
    guide = definitions([("Guide", "<code>hl7.fhir.uv.v2mappings</code> 1.0.0"), ("Supplements", "<code>/etc/ferrobridge/v2mappings-local</code>"),
                         ("Acknowledgment", "original mode, from the commit's outcome"), ("ConceptMaps", "read from the guide's directory")])
    return hl7_page("overview", sections(section("Listener", facts), section("Accepted senders", st, 2), section("Mapping", guide)))


def page_hl7v2_conceptmaps():
    rows = (("HL7 Table 0001 to administrative gender", "HL70001", "AdministrativeGender"), ("HL7 Table 0078 to observation interpretation", "HL70078", "ObservationInterpretation"),
            ("HL7 Table 0085 to observation status", "HL70085", "ObservationStatus"), ("Demo region local units", "DEMO-UNITS", "UCUM"))
    body = [f'<tr><th scope="row">{e(n)}</th>' + td("Source", f"<code>{s_}</code>") + td("Target", t_) + "</tr>" for n, s_, t_ in rows]
    t = table("ConceptMaps", th("ConceptMap") + th("Source") + th("Target"), body)
    note = planned(613, "The ConceptMap directory, read-only: each map with its source and target.")
    tl = [("overview", "Overview", "hl7v2.html"), ("maps", 'ConceptMaps <span class="fui-badge fui-tone-neutral">Planned</span>', "hl7v2-conceptmaps.html"), ("settings", "Settings", "hl7v2-settings.html")]
    return hl7_page("maps", sections(section("", note + t)), tabs_list=tl)


def hl7_rows():
    return (srow("HL7 v2 face", "Runs the MLLP listener.", on(), "hl7v2.enabled")
            + srow("Address", "The socket address the listener binds.", "<code>0.0.0.0:2575</code>", "hl7v2.listen")
            + srow("Character set", "What a message with an empty MSH-18 is read in.", "ASCII", "hl7v2.default_charset")
            + srow("ConceptMap directory", "The guide's package directory.", "<code>/etc/ferrobridge/v2mappings</code>", "hl7v2.concept_maps")
            + srow("Supplements", "Directories of local ConceptMaps read after the guide's: 1 supplement.", "<code>/etc/ferrobridge/v2mappings-local</code>", "hl7v2.supplements")
            + srow("EHRs", "Write only into an EHR the CDR holds, or create one; unset, the facade's choice applies.", "As the FHIR facade: only existing ones", "hl7v2.ehr_policy")
            + srow("Profiles", "A profile per resource type the face claims in meta.profile.", '<span class="mk-dash">none</span>', "hl7v2.profiles")
            + srow("Unmapped entries", "Skip and count an entry no program maps, or refuse the message.", "skip and count", "hl7v2.unmapped_entries")
            + srow("Timeouts", "Between messages, and for one frame to arrive whole.", "300 s · 30 s", "hl7v2.idle_timeout_ms", keys=("hl7v2.idle_timeout_ms", "hl7v2.frame_timeout_ms"))
            + srow("Largest frame", "The largest message one frame may carry.", "1 MiB", "hl7v2.frame_limit_bytes")
            + srow("Accepted senders", "The sending facilities (MSH-4) accepted: DEMO-HOSP and DEMO-LAB.", "2 senders", "hl7v2.senders"))


def page_hl7v2_settings():
    return hl7_page("settings", settings_tab(hl7_rows()))


def page_hl7v2_off():
    body = quiet('A sender\'s connection is refused. <a href="hl7v2-settings.html">See its settings</a>')
    return hl7_page("overview", body, off=True)


# ---- OMOP ETL ----

OMOP_TABS = [("overview", "Overview", "omop.html"), ("settings", "Settings", "omop-settings.html")]
OMOP_PLANNED_TABS = [("overview", "Overview", "omop.html"), ("runs", 'Runs <span class="fui-badge fui-tone-neutral">Planned</span>', "omop-runs.html"), ("settings", "Settings", "omop-settings.html")]


def omop_page(tab, panel, tabs_list=None):
    h = page_heading("OMOP ETL", subtitle="A batch that maps compositions from the CDR into the OMOP CDM database.", badge=badge("neutral", "Configured"))
    return shell("OMOP ETL", "omop", h + f'<div class="fui-stack">{tabbed(tabs("OMOP ETL", tabs_list or OMOP_TABS, tab), panel)}</div>')


def page_omop():
    flow = definitions([("Reads", 'compositions from the <a href="connection.html">openEHR CDR</a>, over AQL'), ("Maps", 'with the <a href="mappings-omocl.html">4 OMOCL files</a>'),
                        ("Writes", 'the <a href="connection-cdm.html">OMOP CDM database</a>, schema <code>cdm</code>'), ("Deleted compositions", "their rows are removed")])
    runs = definitions([("How it runs", f'<code>ferrobridge etl run</code> {copy_button("Copy", "the run command")}, as its own process, started by your scheduler'),
                        ("Run reports", "printed by each run")])
    return omop_page("overview", sections(section("What a run does", flow), section("Runs", runs)))


AQL = """SELECT e/ehr_id/value AS ehr_id,
       c/uid/value AS version_uid,
       c AS composition
FROM EHR e CONTAINS COMPOSITION c
WHERE c/archetype_details/template_id/value MATCHES {'synthetic.vital_signs.v1', 'synthetic.diagnosis.v1'}
ORDER BY c/context/start_time/value"""

VISIT_AQL = """SELECT e/ehr_id/value AS ehr_id,
       c/uid/value AS version_uid,
       c/context/start_time/value AS visit_start,
       c/context/end_time/value AS visit_end
FROM EHR e CONTAINS COMPOSITION c[openEHR-EHR-COMPOSITION.encounter.v1]
WHERE c/context/start_time/value >= $since
ORDER BY c/context/start_time/value"""


def omop_rows():
    return (srow("Composition query", "The AQL a run reads; it aliases ehr_id, version_uid and the composition.", disclosure("Show the AQL, 6 lines", f'<pre class="fui-code-panel mk-aql"><code>{e(AQL)}</code></pre>'), "etl.aql")
            + srow("Page size", "How many rows each page of a query fetches.", "100", "etl.page_size")
            + srow("Deleted compositions", "Remove their rows, or list them in the report and keep them.", "remove", "etl.deletions")
            + srow("Type concepts", "The provenance of the records and of the observation periods.", 'EHR <span class="fui-muted">(32817)</span> · EHR <span class="fui-muted">(32817)</span>', "etl.type_concept_id",
                   keys=("etl.type_concept_id", "etl.observation_period_type_concept_id"))
            + srow("Vocabulary aliases", "Terminology ids looked up under another OHDSI vocabulary: LOINC, SNOMED, Gender.", "3 aliases", "etl.vocabulary_aliases")
            + srow("Visit query", "The AQL that derives each visit.", disclosure("Show the AQL, 7 lines", f'<pre class="fui-code-panel mk-aql"><code>{e(VISIT_AQL)}</code></pre>'), "etl.visits.aql")
            + srow("Visit concepts", "The concept and the type of every visit.", 'Outpatient visit <span class="fui-muted">(9202)</span> · EHR <span class="fui-muted">(32817)</span>', "etl.visits.visit_concept_id",
                   keys=("etl.visits.visit_concept_id", "etl.visits.visit_type_concept_id")))


def page_omop_settings():
    return omop_page("settings", settings_tab(omop_rows()))


# ---- OMOP ETL, planned #612: the runs and B7, one run's report ----

def run_title(start):
    d, t = start.split("T")
    day = int(d[-2:])
    return f"Run of {day} October 2026, {t}"


def run_state(t, refused):
    if t == "bad":
        return state("bad", "stopped")
    if refused:
        return state("warn", f"{refused} refused")
    return state("ok", "complete")


def uuid_hint(rid):
    return f'<span class="fui-small fui-muted"><code>{rid}</code></span> {copy_button("Copy", "the run id")}'


def page_omop_runs():
    rows = []
    for rid, start, took, t, read, refused in RUNS:
        href = "omop-run-failed.html" if t == "bad" else "omop-run.html"
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{href}" title="Run id {rid}">{run_title(start)}</a></th>' + td("Started", when(start), role="sub")
                    + td("Took", took, "fui-num", role="meta-q") + td("Compositions read", f"{read:,}", "fui-num", role="wide", unit="read") + td("State", run_state(t, refused), role="meta-2") + "</tr>")
    t = table("Runs", th("Run", "r", "descending") + th("Started") + th("Took", num=True) + th("Compositions read", num=True) + th("State"), rows)
    note = planned(612, "Each run's report kept in the bridge schema and listed here, read-only.")
    return omop_page("runs", sections(section("", note + t)), OMOP_PLANNED_TABS)


def page_omop_empty():
    body = quiet("No run yet. Run <code>ferrobridge cdm init</code> and load the OHDSI vocabulary once, then <code>ferrobridge etl run</code>.")
    return omop_page("runs", sections(section("", planned(612, "Run reports, kept and listed.") + body)), OMOP_PLANNED_TABS)


def funnel(r):
    tot = r["read"]
    parts = [("ok", "Committed", r["committed"]), ("neutral", "Skipped, same version", r["skipped"]), ("bad", "Refused whole", r["refused"])]
    bar = "".join(f'<span class="fui-funnel-part fui-tone-{t}" style="width: {n / tot * 100:.2f}%" title="{w}: {n:,}"></span>' for t, w, n in parts)
    legend = "".join(f'<div><dt><span class="fui-funnel-swatch fui-tone-{t}" aria-hidden="true"></span>{w}</dt><dd>{n:,}</dd></div>' for t, w, n in parts)
    return (f'<div class="fui-funnel"><p><strong>{tot:,}</strong> compositions read</p>'
            f'<div class="fui-funnel-bar" role="img" aria-label="Of {tot:,} compositions read: {r["committed"]:,} committed, {r["skipped"]:,} skipped, {r["refused"]} refused">{bar}</div>'
            f'<dl class="fui-funnel-legend">{legend}</dl></div>')


def barlist(rows, notes=None):
    mx = max(n for _, n in rows)
    notes = notes or {}
    items = "".join(f'<li><span class="fui-barlist-name">{t}{notes.get(t, "")}</span><span class="fui-barlist-track" aria-hidden="true"><span class="fui-barlist-fill" style="width: {n / mx * 100:.1f}%"></span></span>'
                    f'<span class="fui-barlist-num">{n:,}</span></li>' for t, n in rows)
    return f'<ul class="fui-barlist">{items}</ul>'


def run_heading(rid=RUN["id"], start="2026-10-21T04:00", badge_html=None, sub=None):
    ag = action_group(button("primary", "Download the report", "download"), [menu_item("Copy the run id", "copy", desc=rid)], label=f"More actions for the {run_title(start).lower()}")
    s = sub or f'Started {when(start)}, took 7 min · {uuid_hint(rid)}'
    return page_heading(run_title(start), actions=ag, subtitle=s, crumbs=[("OMOP ETL", "omop-runs.html")], badge=badge_html or state("warn", f'{RUN["refused"]} refused'))


def page_omop_run(tab="summary"):
    r = RUN
    t = tabs("Run sections", [("summary", "Summary", "omop-run.html"), ("refusals", "Refusals", "omop-run-refusals.html", r["refused"])], tab)
    if tab == "summary":
        visits = facts_row([("With a visit", f'{r["with_visit"]:,}'), ("Without a visit", str(r["without_visit"])), ("Several visits, none chosen", str(r["ambiguous"]))])
        cz = {k: f'<span class="fui-concept-zero">{icon("triangle-alert", "12")}{n} with no standard concept</span>' for k, n in r["concept_zero"].items()}
        cz["PERSON"] = f'<span class="fui-small fui-muted"> · {r["persons"][1]} placeholders</span>'
        total_cz = sum(r["concept_zero"].values())
        rows = (barlist(r["rows"], cz) + f'<p class="fui-small fui-muted">{total_cz} values with no standard concept in all (concept 0).</p>')
        der = barlist(r["derived"])
        d = r["deletions"]
        dele = definitions([("Checked with the CDR", str(d["checked"])), ("Still held", str(d["extant"])), ("Deleted, rows removed", f'{d["removed"]} · 9 rows'),
                            ("Unknown to the CDR, kept", f'{d["unknown"]}')])
        panel = sections(section("", planned(612, "The report each run prints today, kept and drawn. It names compositions by id and counts rows; never a clinical value.")),
                         section("Compositions", f'<div class="fui-stack">{funnel(r)}{visits}</div>'), section("Rows written", rows, f'{sum(n for _, n in r["rows"]):,}'),
                         section("Derived tables, after the run", der), section("Deletions", dele))
    else:
        body = [f'<tr><th scope="row" data-role="title"><code title="{c}">{c[:8]}…</code> {copy_button("Copy", "the composition id")}</th>' + td("Mapping", f'<a href="file-omocl.html">{m}</a>', role="sub")
                + td("Column", f"<code>{tb}.<wbr>{col}</code>", role="wide") + td("Why", e(why), role="meta-q") + "</tr>" for c, v, m, tb, col, why in REFUSALS]
        tt = table("Refused records", th("Composition") + th("Mapping") + th("Column") + th("Why"), body)
        panel = sections(section("", planned(612, "Every refused record of a kept run report; each is read again by the next run.") + tt))
    return shell(run_title(r["start"]), "omop", run_heading() + f'<div class="fui-stack">{tabbed(t, panel)}</div>')


def page_omop_run_refusals():
    return page_omop_run("refusals")


def page_omop_run_failed():
    rid = RUNS[2][0]
    rf = refusal("The run stopped before it read anything from the CDR: two OMOCL files define one mapping.", request="ferrobridge etl run",
                 issues=[("bad", "error", "duplicate-mapping-name", 'a console file and a provisioned file both define synthetic_drug. <a href="file-omocl.html">Open the console file</a>, remove one, and run again.')])
    note = planned(612, "Kept and drawn from the runner's report.")
    body = f'<div class="fui-stack">{note}{rf}{definitions([("Rows written", "0"), ("Watermarks", "unchanged: the next run reads from the same point")])}</div>'
    return shell(run_title("2026-10-19T04:00"), "omop", run_heading(rid, "2026-10-19T04:00", state("bad", "stopped"), f'Started {when("2026-10-19T04:00")}, stopped after 4 s · {uuid_hint(rid)}') + body)


# ---- Connections ----

CONN = [("cdr", "openEHR CDR", "connection.html", "<code>cdr.demo-regio.example.org</code>", "OAuth 2.0 client credentials", "FHIR facade, Playground, HL7 v2, OMOP ETL"),
        ("term", "Terminology server", "connection-terminology.html", "<code>term.demo-regio.example.org</code>", "OAuth 2.0 client credentials", "FHIR facade, Playground, HL7 v2"),
        ("cdm", "OMOP CDM database", "connection-cdm.html", "<code>cdm.demo-regio.example.org</code>", "Password, from a file", "OMOP ETL")]


def page_connections(attention=False):
    rows = []
    for k, n, href, addr, cred, used in CONN:
        st = state("warn", "degraded") if attention and k == "term" else state("ok", "up")
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{n}</a></th>' + td("Host", addr, role="sub") + td("Credentials", cred, role="meta-q")
                    + td("Used by", used, role="sub") + td("State", st, role="meta-2") + "</tr>")
    t = table("Connections", th("Connection") + th("Host") + th("Credentials") + th("Used by") + th("State", title="From the bridge's readiness indicators"), rows)
    c = tpl.card(t, label="Connections", title="Connections", count="3")
    h = page_heading("Connections", subtitle="The upstreams the bridge reaches.")
    return shell("Connections", "connections", h + tpl.list_body("", "", "", c))


def page_connections_attention():
    return page_connections(True)


CONN_HREF = {"cdr": "connection", "term": "connection-terminology", "cdm": "connection-cdm"}


def conn_page(key, tab, panel, title, sub, badge_html):
    href = CONN_HREF[key]
    t = tabs(title, [("overview", "Overview", f"{href}.html"), ("settings", "Settings", f"{href}-settings.html")], tab)
    h = page_heading(title, subtitle=sub, badge=badge_html, crumbs=[("Connections", "connections.html")])
    return shell(title, "connections", h + f'<div class="fui-stack">{tabbed(t, panel)}</div>')


def page_connection():
    facts = facts_row([("Base", f'<code>https://cdr.demo-regio.example.org/openehr/v1</code> {copy_button("Copy", "the base URL")}'), ("API", "openEHR ITS-REST 1.1.0"), ("Product", "FerroEHR 4.3.3")])
    ind = definitions([("Readiness", f'answered {when("2026-10-21T09:42:07", seconds=True)}'), ("Sign-in", "OAuth 2.0 client credentials, renewed once on a 401"), ("Retries", "up to 3 attempts")])
    used = definitions([("FHIR facade", "every read and write"), ("Playground", "the templates the programs name"), ("HL7 v2", "each message's transaction"), ("OMOP ETL", "the composition and visit queries")])
    return conn_page("cdr", "overview", sections(section("Connection", facts), section("State", ind), section("Used by", used)),
                     "openEHR CDR", "The CDR the bridge reads and writes.", state("ok", "up"))


TOKEN = "https://idp.demo-regio.example.org/realms/bridge/protocol/openid-connect/token"


def cdr_rows():
    return (srow("Base URL", "The openEHR REST API root.", "<code>https://cdr.demo-regio.example.org/openehr/v1</code>", "cdr.base_url")
            + srow("Timeout", "How long one call may take, connection included.", "30 s", "cdr.timeout_ms")
            + srow("Retry", "Attempts at most, between 200 ms and 5 s apart.", "3 attempts", "cdr.retry.max_attempts",
                   keys=("cdr.retry.max_attempts", "cdr.retry.initial_backoff_ms", "cdr.retry.max_backoff_ms"))
            + srow("Demographic API", "Reaches the ITS-REST 1.1.0 Demographic API, marked DEVELOPMENT.", on(False), "cdr.demographic")
            + srow("Token endpoint", "Where the client credentials are exchanged for a token.", f'<code>{TOKEN}</code>', "cdr.credentials.oauth.token_endpoint")
            + srow("Client ID", "The bridge's client at the authorization server.", "<code>ferrobridge</code>", "cdr.credentials.oauth.client_id")
            + srow("Client secret", "Read from a file; never shown.", secret_now("/run/secrets/cdr.secret"), "cdr.credentials.oauth.client_secret_file"))


def page_connection_settings():
    return conn_page("cdr", "settings", settings_tab(cdr_rows()), "openEHR CDR", "The CDR the bridge reads and writes.", state("ok", "up"))


def page_connection_terminology():
    rf = notice("warn", "triangle-alert", "The terminology server answers 503 (3 attempts, the last at 09:41:58).")
    eff = definitions([("Coded values", "refused, because display fallback is off"), ("FHIR facade", "a create or update with a coded value answers 422"),
                       ("Playground", "a conversion with a coded value is refused"),
                       ("HL7 v2", "a message with a table value answers AE"), ("Unaffected", "reads, and the ETL")])
    facts = facts_row([("Base", f'<code>https://term.demo-regio.example.org/fhir</code> {copy_button("Copy", "the base URL")}'), ("FHIR", "R4"), ("Product", "FerroTERM")])
    return conn_page("term", "overview", sections(section("", rf), section("What it does to the bridge", eff), section("Connection", facts)),
                     "Terminology server", "The FHIR terminology server the bridge looks codes up in.", state("warn", "degraded"))


def page_connection_cdm():
    facts = facts_row([("Host", f'<code>cdm.demo-regio.example.org:5432</code> {copy_button("Copy", "the host")}'), ("Database", "<code>ohdsi</code>"), ("TLS", "Verified (verify-full)"), ("Schemas", "<code>cdm</code> · <code>ferrobridge</code>")])
    ind = definitions([("Readiness", f'answered {when("2026-10-21T09:42:05", seconds=True)}'), ("Vocabulary", "loaded, 10 tables"), ("New persons", "Unknown persons get a placeholder row")])
    used = definitions([("OMOP ETL", "writes the CDM tables, its side table and watermarks"), ("<code>ferrobridge cdm init</code>", "creates the schemas once")])
    return conn_page("cdm", "overview", sections(section("Connection", facts), section("State", ind), section("Used by", used)),
                     "OMOP CDM database", "The PostgreSQL database the ETL writes.", state("ok", "up"))


def term_rows():
    return (srow("Base URL", "The FHIR service base URL.", "<code>https://term.demo-regio.example.org/fhir</code>", "terminology.base_url")
            + srow("FHIR version", "The release the server answers in.", "R4", "terminology.wire_version")
            + srow("Timeout", "How long one call may take, connection included.", "30 s", "terminology.timeout_ms")
            + srow("Retry", "Attempts at most, between 200 ms and 5 s apart.", "3 attempts", "terminology.retry.max_attempts",
                   keys=("terminology.retry.max_attempts", "terminology.retry.initial_backoff_ms", "terminology.retry.max_backoff_ms"))
            + srow("Display fallback", "For an unknown code, write the code as its display instead of refusing the resource.", on(False), "terminology.display_fallback")
            + srow("Token endpoint", "Where the client credentials are exchanged for a token.", f'<code>{TOKEN}</code>', "terminology.credentials.oauth.token_endpoint")
            + srow("Client ID", "The bridge's client at the authorization server.", "<code>ferrobridge</code>", "terminology.credentials.oauth.client_id")
            + srow("Client secret", "Read from a file; never shown.", secret_now("/run/secrets/term.secret"), "terminology.credentials.oauth.client_secret_file"))


def page_connection_terminology_settings():
    return conn_page("term", "settings", settings_tab(term_rows()), "Terminology server", "The FHIR terminology server the bridge looks codes up in.", state("warn", "degraded"))


def cdm_rows():
    return (srow("Connection URL", "Read from a file; never shown. It carries the password.", secret_now("/run/secrets/cdm-url"), "cdm.url_file")
            + srow("TLS", 'The server certificate, checked against the CA in /etc/ferrobridge/cdm-ca.pem. <span class="fui-muted"><code>sslmode=verify-full</code> in the URL</span>',
                   "Checked in full against the CA", "cdm.tls_ca_file")
            + srow("Schemas", "Where the CDM tables live, and the bridge's side table and watermarks.", "<code>cdm</code> · <code>ferrobridge</code>", "cdm.schema", keys=("cdm.schema", "cdm.bridge_schema"))
            + srow("New persons", 'For a person no mapping writes: create a placeholder row, or refuse the row. <span class="fui-muted"><code>create_on_first_sight</code></span>',
                   "Create a placeholder row", "cdm.person_policy"))


def page_connection_cdm_settings():
    return conn_page("cdm", "settings", settings_tab(cdm_rows()), "OMOP CDM database", "The PostgreSQL database the ETL writes.", state("ok", "up"))
