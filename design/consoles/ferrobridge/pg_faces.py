# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The bridge's faces (FHIR facade, HL7 v2, OMOP ETL) and its upstreams
(Connections), each a T2 page. A tab panel holds B10 sections, never cards
in a card; the last tab is Settings, a read-only view of ferrobridge.toml in
T3 rows. A face has no primary action of its own, so its title row holds
More alone. B7, the ETL run report, is drawn for the planned screens."""
from common import (
    action_group, button, copy_button, definitions, e, empty_state, facts_row, icon, link_button, menu_item, page_heading, pager, pill, refusal,
    shell, table, tabbed, tabs, td, th, when,
)
import tpl
from bridge import json_panel, planned, section, sections, state
from data import REFUSALS, RUN, RUNS
from settings import on, secret_now, settings_tab, srow


def more_only(items, label, open_=False):
    return action_group("", items, open_=open_, label=label)


# ---- FHIR facade ----

FACADE_TABS = [("overview", "Overview", "facade.html"), ("capability", "Capability", "facade-capability.html", 6), ("security", "Security", "facade-security.html"), ("settings", "Settings", "facade-settings.html")]


def facade_heading(off=False, more_open=False):
    ag = more_only([menu_item("Copy the base URL", "copy"), menu_item("Open the CapabilityStatement", "file-code", desc="/fhir/metadata as JSON"),
                    menu_item("Open SMART discovery", "file-code", desc="The SMART configuration as JSON")], "More actions for the FHIR facade", more_open)
    b = state("muted", "off") if off else state("ok", "on · listening")
    sub = "Off, so the bridge serves no FHIR route." if off else "Serves FHIR R4 over the CDR at <code>/fhir</code>."
    return page_heading("FHIR facade", actions=ag, subtitle=sub, badge=b)


def facade_page(tab, panel, off=False, more_open=False):
    t = tabs("FHIR facade", FACADE_TABS, tab)
    return shell("FHIR facade", "facade", facade_heading(off, more_open) + f'<div class="fui-stack">{tabbed(t, panel)}</div>')


def page_facade(more_open=False):
    facts = facts_row([("Base", "<code>https://bridge.demo-regio.example.org/fhir</code>"), ("Listener", f'<code>0.0.0.0:8080</code> {pill("ok", "Up")}'),
                       ("Contexts", '<a href="mappings.html">9</a>'), ("Resource types", '<a href="facade-capability.html">6</a>'), ("FHIR", "R4 (4.0.1)")])
    serves = definitions([("Interactions", "read, vread, search-type, create, update, delete, over the CDR"), ("Search", "the AQL projections a search file declares beside a context"),
                          ("References", "followed 2 levels deep, from 1 listed base")])
    ops = definitions([("Lane", state("ok", "on")), ("Operations", "<code>$tofhir</code>, <code>$toopenehr</code>, <code>$validate</code>; none of them commits"),
                       ("Try them", '<a href="playground.html">in the Playground</a>')])
    ident = definitions([("Identity store", f'<code>identity.redb</code> {state("ok", "up")}'), ("New EHRs", "only into an EHR the CDR already holds"),
                         ("Commits", "language <code>en</code>, territory <code>NL</code>")])
    return facade_page("overview", sections(section("Facade", facts), section("What it serves", serves), section("Operations", ops), section("Identity", ident)), more_open=more_open)


def page_facade_more():
    return page_facade(more_open=True)


CAP = [("Observation", 3, "read, vread, search-type, create, update, delete", "code, date, patient", "$validate"),
       ("Condition", 2, "read, vread, search-type, create, update", "code, patient", "$validate"),
       ("MedicationStatement", 1, "read, search-type, create", "patient", "$validate"),
       ("AllergyIntolerance", 1, "read, search-type, create, update", "patient", "$validate"),
       ("Encounter", 1, "read, search-type", "patient", "–"),
       ("Procedure", 1, "read, search-type, create", "patient", "–")]


def page_facade_capability():
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{"file-program.html" if rt == "Observation" else "file-provisioned.html"}">{rt}</a></th>'
            + td("Profiles", str(n), "fui-num", role="meta-q") + td("Interactions", e(i), role="wide") + td("Search", e(s_), role="sub") + td("Operations", f"<code>{o}</code>" if o != "–" else '<span class="mk-dash">–</span>', role="meta-2") + "</tr>"
            for rt, n, i, s_, o in CAP]
    t = table("Resource types", th("Resource type", "r", "ascending") + th("Profiles", num=True) + th("Interactions") + th("Search parameters") + th("Operations"), rows)
    intro = '<p class="fui-small fui-muted">From the CapabilityStatement at <code>/fhir/metadata</code>. A resource type opens the program that serves it.</p>'
    return facade_page("capability", sections(section("", intro + t)))


DISCOVERY = """{
  "issuer": "https://idp.demo-regio.example.org/realms/bridge",
  "authorization_endpoint": "https://idp.demo-regio.example.org/realms/bridge/protocol/openid-connect/auth",
  "token_endpoint": "https://idp.demo-regio.example.org/realms/bridge/protocol/openid-connect/token",
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
            + srow("New EHRs", "Write only into an EHR the CDR holds, or create one for an unknown subject.", "only existing EHRs", "facade.ehr_policy")
            + srow("Identity store", "The file the identity map is kept in.", "<code>identity.redb</code>", "facade.identity_store")
            + srow("Language and territory", "What every commit carries, ISO 639-1 and ISO 3166-1.", "en · NL", "facade.composition_language")
            + srow("Search page", "Results per page, and the most a search answers.", "20 · at most 100", "facade.search_count")
            + srow("Reference bases", "How many listed FHIR bases an absolute reference may be fetched from, and how many levels deep.", "1 base · 2 levels", "facade.references.fetch_bases")
            + srow("Identifier search", "Resolves patient.identifier through the demographic AQL.", on(False), "facade.identifier_search.enabled")
            + srow("Operations", "Mounts $tofhir, $toopenehr and $validate.", on(), "operations.enabled"))


def page_facade_settings(hint=False):
    return facade_page("settings", settings_tab(facade_rows(hint)))


def page_facade_settings_hint():
    return page_facade_settings(hint=True)


def page_facade_off():
    body = empty_state("flame", "The FHIR facade is off",
                       "FHIR clients reach nothing. The mapping set still compiles, and the Playground runs when the operations lane is on.",
                       link_button("secondary", "Its settings", "facade-settings.html", "settings"))
    return facade_page("overview", body, off=True)


# ---- HL7 v2 ----

HL7_TABS = [("overview", "Overview", "hl7v2.html"), ("settings", "Settings", "hl7v2-settings.html")]


def hl7_page(tab, panel, off=False, tabs_list=None):
    ag = more_only([menu_item("Copy the listener address", "copy")], "More actions for the HL7 v2 face")
    b = state("muted", "off") if off else state("ok", "on · listening")
    sub = "Off, so no MLLP listener runs." if off else "Takes HL7 v2 messages over MLLP and commits each as one transaction."
    h = page_heading("HL7 v2", actions=ag, subtitle=sub, badge=b)
    return shell("HL7 v2", "hl7v2", h + f'<div class="fui-stack">{tabbed(tabs("HL7 v2", tabs_list or HL7_TABS, tab), panel)}</div>')


def page_hl7v2():
    facts = facts_row([("Listener", f'<code>0.0.0.0:2575</code> {pill("ok", "Up")}'), ("Character set", "ASCII if MSH-18 is empty"), ("Largest frame", "1 MiB"),
                       ("Unmapped entries", "skipped and counted")])
    senders = [f'<tr><th scope="row">{n}</th>' + td("Kind", k) + "</tr>" for n, k in (("DEMO-HOSP", "namespace id"), ("DEMO-LAB", "namespace id"))]
    st = table("Accepted senders", th("MSH-4") + th("Kind"), senders)
    guide = definitions([("Guide", "<code>hl7.fhir.uv.v2mappings</code> 1.0.0, and 1 supplement"), ("Acknowledgment", "original mode, from the commit's outcome"),
                         ("ConceptMaps", "read from the guide's directory; a read-only list is planned (#613)")])
    return hl7_page("overview", sections(section("Listener", facts), section("Accepted senders", st, 2), section("Mapping", guide)))


def page_hl7v2_conceptmaps():
    rows = (("HL7 Table 0001 to administrative gender", "HL70001", "AdministrativeGender"), ("HL7 Table 0078 to observation interpretation", "HL70078", "ObservationInterpretation"),
            ("HL7 Table 0085 to observation status", "HL70085", "ObservationStatus"), ("Demo-regio local units", "DEMO-UNITS", "UCUM"))
    body = [f'<tr><th scope="row">{e(n)}</th>' + td("Source", f"<code>{s_}</code>") + td("Target", t_) + td("Messages served", "–", "fui-num") + "</tr>" for n, s_, t_ in rows]
    t = table("ConceptMaps", th("ConceptMap") + th("Source") + th("Target") + th("Messages served", num=True), body)
    note = planned(613, "The ConceptMap directory, read-only: each map, its source and target, and how many messages it served.")
    tl = [("overview", "Overview", "hl7v2.html"), ("maps", 'ConceptMaps <span class="fui-badge fui-tone-neutral">Planned</span>', "hl7v2-conceptmaps.html"), ("settings", "Settings", "hl7v2-settings.html")]
    return hl7_page("maps", sections(section("", note + t)), tabs_list=tl)


def hl7_rows():
    return (srow("HL7 v2 face", "Runs the MLLP listener.", on(), "hl7v2.enabled")
            + srow("Listener", "The socket address the listener binds.", "<code>0.0.0.0:2575</code>", "hl7v2.listen")
            + srow("Character set", "What a message with an empty MSH-18 is read in.", "ASCII", "hl7v2.default_charset")
            + srow("ConceptMap directory", "The guide's package directory.", "<code>/etc/ferrobridge/v2mappings</code>", "hl7v2.concept_maps")
            + srow("Unmapped entries", "Skip and count an entry no program maps, or refuse the message.", "skip and count", "hl7v2.unmapped_entries")
            + srow("Timeouts", "Between messages, and for one frame to arrive whole.", "300 s · 30 s", "hl7v2.idle_timeout_ms")
            + srow("Largest frame", "The largest message one frame may carry.", "1 MiB", "hl7v2.frame_limit_bytes")
            + srow("Accepted senders", "The sending facilities (MSH-4) accepted: DEMO-HOSP and DEMO-LAB.", "2 senders", "hl7v2.senders"))


def page_hl7v2_settings():
    return hl7_page("settings", settings_tab(hl7_rows()))


def page_hl7v2_off():
    body = empty_state("message-square", "The HL7 v2 face is off", "A sender's connection is refused.", link_button("secondary", "Its settings", "hl7v2-settings.html", "settings"))
    return hl7_page("overview", body, off=True)


# ---- OMOP ETL ----

OMOP_TABS = [("overview", "Overview", "omop.html"), ("settings", "Settings", "omop-settings.html")]
OMOP_PLANNED_TABS = [("overview", "Overview", "omop.html"), ("runs", 'Runs <span class="fui-badge fui-tone-neutral">Planned</span>', "omop-runs.html"), ("settings", "Settings", "omop-settings.html")]


def omop_page(tab, panel, tabs_list=None):
    ag = more_only([menu_item("Copy the run command", "copy", desc="ferrobridge etl run")], "More actions for the OMOP ETL")
    h = page_heading("OMOP ETL", actions=ag, subtitle="A batch that maps compositions from the CDR into the OMOP CDM database.", badge=state("ok", "configured"))
    return shell("OMOP ETL", "omop", h + f'<div class="fui-stack">{tabbed(tabs("OMOP ETL", tabs_list or OMOP_TABS, tab), panel)}</div>')


def page_omop():
    flow = definitions([("Reads", 'compositions from the <a href="connection.html">openEHR CDR</a>, over AQL'), ("Maps", 'with the <a href="mappings-omocl.html">4 OMOCL files</a>'),
                        ("Writes", 'the <a href="connection-cdm.html">OMOP CDM database</a>, schema <code>cdm</code>'), ("Deleted compositions", "their rows are removed")])
    runs = definitions([("How it runs", "<code>ferrobridge etl run</code>, as its own process, started by your scheduler"),
                        ("Run reports", "printed by each run; keeping and listing them here is planned (#612)")])
    return omop_page("overview", sections(section("What a run does", flow), section("Runs", runs)))


def omop_rows():
    return (srow("Composition query", "The AQL a run reads, 6 lines; it aliases ehr_id, version_uid and the composition.", "<code>synthetic compositions</code>", "etl.aql")
            + srow("Page size", "How many rows each page of a query fetches.", "100", "etl.page_size")
            + srow("Deleted compositions", "Remove their rows, or list them in the report and keep them.", "remove", "etl.deletions")
            + srow("Type concepts", "The record and observation period provenance the rows carry.", "32817 · 32817", "etl.type_concept_id")
            + srow("Vocabulary aliases", "Terminology ids looked up under another OHDSI vocabulary: LOINC, SNOMED, Gender.", "3 aliases", "etl.vocabulary_aliases")
            + srow("Visit query", "The AQL that derives each visit, 14 lines.", "<code>synthetic visits</code>", "etl.visits.aql")
            + srow("Visit concepts", "The concept and the type of every visit.", "9202 · 32817", "etl.visits.visit_concept_id"))


def page_omop_settings():
    return omop_page("settings", settings_tab(omop_rows()))


# ---- OMOP ETL, planned #612: the runs and B7, one run's report ----

def run_state(t, refused):
    if t == "bad":
        return state("bad", "stopped")
    if refused:
        return state("warn", f"{refused} refused")
    return state("ok", "complete")


def page_omop_runs():
    rows = []
    for rid, start, end, t, read, refused in RUNS:
        href = "omop-run-failed.html" if t == "bad" else "omop-run.html"
        took = "1 min" if t == "bad" else f"{int(end[-2:]) - int(start[-2:])} min"
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{href}"><code>{rid}</code></a></th>' + td("Started", when(start), role="sub")
                    + td("Took", took, "fui-num", role="meta-q") + td("Read", f"{read:,}", "fui-num", role="wide") + td("State", run_state(t, refused), role="meta-2") + "</tr>")
    t = table("Runs", th("Run", "r", "descending") + th("Started") + th("Took", num=True) + th("Compositions read", num=True) + th("State"), rows)
    note = planned(612, "Each run's report kept in the bridge schema and listed here, read-only.")
    return omop_page("runs", sections(section("", note + t)), OMOP_PLANNED_TABS)


def page_omop_empty():
    body = empty_state("table-2", "No run yet", "Run <code>ferrobridge cdm init</code> and <code>ferrobridge vocab load</code> once, then <code>ferrobridge etl run</code>.")
    return omop_page("runs", sections(section("", planned(612, "Run reports, kept and listed.") + body)), OMOP_PLANNED_TABS)


def funnel(r):
    tot = r["read"]
    parts = [("ok", "Committed", r["committed"]), ("neutral", "Skipped, same version", r["skipped"]), ("bad", "Refused whole", r["refused"])]
    bar = "".join(f'<span class="fui-funnel-part fui-tone-{t}" style="width: {n / tot * 100:.2f}%" title="{w}: {n:,}"></span>' for t, w, n in parts)
    legend = f'<div><dt><span class="fui-funnel-swatch is-total" aria-hidden="true"></span>Read</dt><dd>{tot:,}</dd></div>' + "".join(
        f'<div><dt><span class="fui-funnel-swatch fui-tone-{t}" aria-hidden="true"></span>{w}</dt><dd>{n:,}</dd></div>' for t, w, n in parts)
    return (f'<div class="fui-funnel"><div class="fui-funnel-bar" role="img" aria-label="Of {tot:,} compositions read: {r["committed"]:,} committed, {r["skipped"]:,} skipped, {r["refused"]} refused">{bar}</div>'
            f'<dl class="fui-funnel-legend">{legend}</dl></div>')


def barlist(rows):
    mx = max(n for _, n in rows)
    items = "".join(f'<li><span class="fui-barlist-name">{t}</span><span class="fui-barlist-track" aria-hidden="true"><span class="fui-barlist-fill" style="width: {n / mx * 100:.1f}%"></span></span>'
                    f'<span class="fui-barlist-num">{n:,}</span></li>' for t, n in rows)
    return f'<ul class="fui-barlist">{items}</ul>'


def run_heading(rid="etl-2026-10-21-0400", badge_html=None, sub=None):
    ag = action_group(button("primary", "Download the report", "download"), [menu_item("Copy the run id", "copy")], label=f"More actions for {rid}")
    s = sub or f'Started {when("2026-10-21T04:00")}, took 7 min.'
    return page_heading(rid, actions=ag, subtitle=s, crumbs=[("OMOP ETL", "omop-runs.html")], badge=badge_html or state("warn", "13 refused"), mono=True)


def page_omop_run(tab="summary"):
    r = RUN
    t = tabs("Run sections", [("summary", "Summary", "omop-run.html"), ("refusals", "Refusals", "omop-run-refusals.html", r["refused"])], tab)
    if tab == "summary":
        visits = facts_row([("With a visit", f'{r["with_visit"]:,}'), ("Without a visit", str(r["without_visit"])), ("Several visits, none chosen", str(r["ambiguous"]))])
        rows = (barlist(r["rows"]) + f'<p class="fui-small"><span class="fui-concept-zero">{icon("triangle-alert", "12")}{r["concept_zero"]} concept 0 assignments</span></p>')
        der = barlist(r["derived"]) + f'<p class="fui-small fui-muted">{r["persons"][0]} PERSON rows, {r["persons"][1]} of them placeholders.</p>'
        d = r["deletions"]
        dele = definitions([("Checked with the CDR", str(d["checked"])), ("Still held", str(d["extant"])), ("Deleted, rows removed", f'{d["removed"]} · 9 rows'),
                            ("Unknown to the CDR, kept", f'{d["unknown"]}')])
        panel = sections(section("", planned(612, "The report each run prints today, kept and drawn. It names compositions by id and counts rows; never a clinical value.")),
                         section("Compositions", f'<div class="fui-stack">{funnel(r)}{visits}</div>'), section("Rows written", rows, f'{sum(n for _, n in r["rows"]):,}'),
                         section("Derived tables, after the run", der), section("Deletions", dele))
    else:
        body = [f'<tr><th scope="row" data-role="title"><code>{c[:8]}…</code></th>' + td("Mapping", f'<a href="file-omocl.html">{m}</a>', role="sub")
                + td("Column", f"<code>{tb}.<wbr>{col}</code>", role="wide") + td("Why", e(why), role="meta-q") + "</tr>" for c, v, m, tb, col, why in REFUSALS]
        tt = table("Refused records", th("Composition") + th("Mapping") + th("Column") + th("Why"), body)
        panel = sections(section("", '<p class="fui-small fui-muted">Every refused record; each is read again by the next run.</p>' + tt + f'<div class="fui-list-foot">{pager("Refused records", 13, 3, 1, sizes=(3, 25, 50))}</div>'))
    return shell(r["id"], "omop", run_heading() + f'<div class="fui-stack">{tabbed(t, panel)}</div>')


def page_omop_run_refusals():
    return page_omop_run("refusals")


def page_omop_run_failed():
    rf = refusal("The run stopped before it called an upstream: two OMOCL files define one mapping.", request="ferrobridge etl run",
                 issues=[("bad", "error", "mapping-clash", 'a console file and a provisioned file both define synthetic_drug. <a href="file-omocl.html">Open the console file</a>, remove one, and run again.')])
    note = planned(612, "Kept and drawn from the runner's report.")
    body = f'<div class="fui-stack">{note}{rf}{definitions([("Rows written", "0"), ("Watermarks", "unchanged: the next run reads from the same point")])}</div>'
    return shell("etl-2026-10-19-0400", "omop", run_heading("etl-2026-10-19-0400", state("bad", "stopped"), f'Started {when("2026-10-19T04:00")}, stopped after 4 s.') + body)


# ---- Connections ----

CONN = [("cdr", "openEHR CDR", "connection.html", "<code>cdr.demo-regio.example.org</code>", "OAuth 2.0 client credentials", "facade, operations, HL7 v2, ETL"),
        ("term", "Terminology server", "connection-terminology.html", "<code>term.demo-regio.example.org</code>", "OAuth 2.0 client credentials", "facade, operations, HL7 v2"),
        ("cdm", "OMOP CDM database", "connection-cdm.html", "<code>cdm.demo-regio.example.org</code>", "password, from a file", "ETL")]


def page_connections(attention=False):
    rows = []
    for k, n, href, addr, cred, used in CONN:
        st = state("bad", "unreachable") if attention and k == "term" else state("ok", "up")
        rows.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{n}</a></th>' + td("Host", addr, role="sub") + td("Credentials", cred, role="wide")
                    + td("Used by", used, role="wide") + td("State", st, role="meta-2") + "</tr>")
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
    ag = more_only([menu_item("Copy the address", "copy")], f"More actions for {title}")
    h = page_heading(title, actions=ag, subtitle=sub, badge=badge_html, crumbs=[("Connections", "connections.html")])
    return shell(title, "connections", h + f'<div class="fui-stack">{tabbed(t, panel)}</div>')


def page_connection():
    facts = facts_row([("Base", "<code>https://cdr.demo-regio.example.org/openehr/v1</code>"), ("API", "openEHR ITS-REST 1.1.0"), ("Product", "FerroEHR 4.3.3")])
    ind = definitions([("Readiness", state("ok", "up")), ("Sign-in", "OAuth 2.0 client credentials, renewed once on a 401"), ("Retries", "up to 3 attempts")])
    used = definitions([("FHIR facade", "every read and write"), ("Operations", "the templates the programs name"), ("HL7 v2", "each message's transaction"), ("OMOP ETL", "the composition and visit queries")])
    return conn_page("cdr", "overview", sections(section("Connection", facts), section("State", ind), section("Used by", used)),
                     "openEHR CDR", "The CDR the bridge reads and writes.", state("ok", "up"))


def cdr_rows():
    return (srow("Base URL", "The openEHR REST API root.", "<code>https://cdr.demo-regio.example.org/openehr/v1</code>", "cdr.base_url")
            + srow("Timeout", "How long one call may take, connection included.", "30 s", "cdr.timeout_ms")
            + srow("Retry", "Attempts at most, between 200 ms and 5 s apart.", "3 attempts", "cdr.retry.max_attempts")
            + srow("Demographic API", "Reaches the ITS-REST 1.1.0 Demographic API, marked DEVELOPMENT.", on(False), "cdr.demographic")
            + srow("Token endpoint", "Where the client credentials are exchanged for a token.", "<code>idp.demo-regio.example.org</code>", "cdr.credentials.oauth.token_endpoint")
            + srow("Client id", "The bridge's client at the authorization server.", "<code>ferrobridge</code>", "cdr.credentials.oauth.client_id")
            + srow("Client secret", "Write-only, read from a file.", secret_now("/run/secrets/cdr.secret"), "cdr.credentials.oauth.client_secret_file"))


def page_connection_settings():
    return conn_page("cdr", "settings", settings_tab(cdr_rows()), "openEHR CDR", "The CDR the bridge reads and writes.", state("ok", "up"))


def page_connection_terminology():
    rf = refusal("The terminology server does not answer.", request="GET /fhir/metadata",
                 issues=[("bad", "down", "terminology", "term.demo-regio.example.org answered 503 to 3 attempts, the last at 09:41:58.")])
    eff = definitions([("Coded values", "refused, because display fallback is off"), ("FHIR facade", "a create or update with a coded value answers 422"),
                       ("HL7 v2", "a message with a table value answers AE"), ("Unaffected", "reads, and the ETL")])
    facts = facts_row([("Base", "<code>https://term.demo-regio.example.org/fhir</code>"), ("FHIR", "R4"), ("Product", "FerroTERM")])
    return conn_page("term", "overview", sections(section("", rf), section("What it does to the bridge", eff), section("Connection", facts)),
                     "Terminology server", "The FHIR terminology server the bridge looks codes up in.", state("bad", "unreachable"))


def page_connection_cdm():
    facts = facts_row([("Host", "<code>cdm.demo-regio.example.org:5432</code>"), ("Database", "<code>ohdsi</code>"), ("TLS", "verified, full"), ("Schemas", "<code>cdm</code> · <code>ferrobridge</code>")])
    ind = definitions([("Readiness", state("ok", "up")), ("Vocabulary", "loaded, 10 tables"), ("New persons", "a placeholder row on first sight")])
    used = definitions([("OMOP ETL", "writes the CDM tables, its side table and watermarks"), ("cdm init", "creates the schemas once")])
    return conn_page("cdm", "overview", sections(section("Connection", facts), section("State", ind), section("Used by", used)),
                     "OMOP CDM database", "The PostgreSQL database the ETL writes.", state("ok", "up"))


def term_rows():
    return (srow("Base URL", "The FHIR service base URL.", "<code>https://term.demo-regio.example.org/fhir</code>", "terminology.base_url")
            + srow("FHIR version", "The release the server answers in.", "R4", "terminology.wire_version")
            + srow("Timeout", "How long one call may take, connection included.", "30 s", "terminology.timeout_ms")
            + srow("Display fallback", "For an unknown code, write the code as its display instead of refusing the resource.", on(False), "terminology.display_fallback")
            + srow("Client secret", "Write-only, read from a file.", secret_now("/run/secrets/term.secret"), "terminology.credentials.oauth.client_secret_file"))


def page_connection_terminology_settings():
    return conn_page("term", "settings", settings_tab(term_rows()), "Terminology server", "The FHIR terminology server the bridge looks codes up in.", state("bad", "unreachable"))


def cdm_rows():
    return (srow("Connection URL", "Write-only, read from a file; it carries the password.", secret_now("/run/secrets/cdm-url"), "cdm.url_file")
            + srow("TLS", "The certificate is checked in full against the CA in /etc/ferrobridge/cdm-ca.pem.", "verify-full", "cdm.tls_ca_file")
            + srow("Schemas", "Where the CDM tables live, and the bridge's side table and watermarks.", "<code>cdm</code> · <code>ferrobridge</code>", "cdm.schema")
            + srow("New persons", "For a person no mapping writes: a placeholder row, or refuse the row.", "placeholder", "cdm.person_policy"))


def page_connection_cdm_settings():
    return conn_page("cdm", "settings", settings_tab(cdm_rows()), "OMOP CDM database", "The PostgreSQL database the ETL writes.", state("ok", "up"))
