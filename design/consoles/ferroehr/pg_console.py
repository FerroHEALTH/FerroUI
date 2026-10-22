# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The Console group and the reader's own pages: Server (T3 reference over
the CDR's status, manifest, management surface, OpenAPI and SMART), the
Audit log, Subscriptions, Configuration (T3, file mode only), Your
preferences (cookies), Your access, and the sign-in states."""
import tpl
from common import ago
from common import (
    toast, toasts,
    CLINICIAN, MENU_SEP, OPERATOR, action_group, badge, button, card, centred, check, e, field, icon, icon_button, input_, link_button, lockup,
    menu, menu_item, notice, page_heading, refusal, select, shell, table, td, th, when,
)
from data import AUDIT, E1, SUBSCRIPTIONS, VERSION
from ehr import (
    load_report,
    codeview, confirm_delete, dialog, empty_line, facts, intro, json_html, masked, plural, quiet, scope, scopes, section, sections, short, state,
)
from parts import choice_cards, date_input, density_preview, setting_row, theme_preview
from pg_ehr import edit_dock, edit_shell, msg

# ---- Server ----

SERVER_TABS = [("overview", "Overview", "server.html"), ("health", "Health", "server-health.html"), ("metrics", "Metrics", "server-metrics.html"),
               ("logging", "Logging", "server-logging.html"), ("api", "API", "server-api.html"), ("smart", "SMART", "server-smart.html")]


def server_page(tab, panel_html, line, tabs_list=SERVER_TABS, overlay=None, sandbox=False):
    h = page_heading("Server", subtitle="The CDR at <code>cdr.example.org</code>: what it is, how it runs, and what it serves.")
    intro_ = f'<div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">{line}</p></div>'
    t = tpl.settings_tabs("Server sections", tabs_list, tab, f'<div class="fui-settings-panel">{intro_}{panel_html}</div>', many=len(tabs_list) > 5)
    return shell("Server", "server", h + f'<div class="fui-stack">{t}</div>', overlay=overlay, sandbox=sandbox)


def overview_body(sandbox=False):
    status = facts([("State", state("ok", "Up") + ' <span class="fui-small fui-muted">answered 13 s ago</span>'),
                    ("Deployment", ("Sandbox" if sandbox else "Production")),
                    ("Versions and conformance", '<a href="about.html">About</a>')])
    build = facts([("Git commit", "<code>7c1e9d2</code>"), ("Built", when("2026-10-19T06:12")), ("rustc", "1.99.1"), ("PostgreSQL target", "18.6"),
                   ("Audit", "On · fails closed · local store · kept 7 years"), ("Support until", "2027-10")])
    parts = [section("Status", status)]
    if sandbox:
        gaps = table("Production safeguards that are off", th("Safeguard") + th("What it means"), [
            f'<tr><th scope="row" data-role="title">{state("warn", "Shared database credential")}</th>{td("What it means", "Schema changes and the running CDR use one PostgreSQL role.", role="sub")}</tr>',
            f'<tr><th scope="row" data-role="title">{state("warn", "Shared cluster")}</th>{td("What it means", "The clinical, party and audit databases share one PostgreSQL cluster.", role="sub")}</tr>',
            f'<tr><th scope="row" data-role="title">{state("warn", "Migrations on the runtime credential")}</th>{td("What it means", "The CDR prepares its own schema at start.", role="sub")}</tr>'])
        parts.append(section("Production safeguards that are off", intro("A sandbox must not hold real patient data.") + gaps, count=3))
    parts += [section("Build", build)]
    return "".join(parts)


def page_server():
    return server_page("overview", overview_body(), "How the CDR was built and deployed.")


def page_server_sandbox():
    return server_page("overview", overview_body(True), "How the CDR was built and deployed.", sandbox=True)


def page_server_health():
    rows = [("Database", "ok", "Up", "PostgreSQL 18.6 answered in 4\u00a0ms"), ("Migrations", "ok", "Up", "41 applied, none pending"),
            ("Audit store", "ok", "Up", "Chain verified " + ago("2026-10-21T09:30")), ("Event publisher", "warn", "Degraded", "Reconnecting to the broker, attempt 3"),
            ("Terminology server", "muted", "Off", "Not configured")]
    t = table("Dependency indicators", th("Dependency") + th("State") + th("Detail"),
              [f'<tr><th scope="row" data-role="title">{n}</th>{td("State", state(tone, w), role="sub")}{td("Detail", d, role="meta-q")}</tr>' for n, tone, w, d in rows])
    return server_page("health", section("Dependencies", t, count=5), "What the CDR depends on, and whether each part is healthy.")


METRICS = ["aql_queries_total", "compositions_committed_total", "db_pool_connections", "http_request_duration_seconds", "http_requests_in_flight", "audit_events_total"]


METRIC_NAME = {"aql_queries_total": "AQL queries", "compositions_committed_total": "Compositions committed", "db_pool_connections": "Database connections",
               "http_request_duration_seconds": "Request duration", "http_requests_in_flight": "Requests in flight", "audit_events_total": "Audit events"}


def page_server_metrics():
    from common import stat_card
    now = f'<div class="fui-grid fui-grid-stats fui-stat-row">{stat_card("activity", "3", "Requests in flight")}{stat_card("database", "6 of 20", "Database connections")}</div>'
    since = f'<div class="fui-grid fui-grid-stats fui-stat-row">{stat_card("file-text", "184", "Compositions committed")}{stat_card("square-terminal", "1,206", "AQL queries")}</div>'
    names = sorted([(METRIC_NAME[m], m) for m in METRICS])
    pick = (f'<div class="fui-transform-options">{field("m-pick", "Metric", select("m-pick", [(m, f"{n} · {m}") for n, m in names], "db_pool_connections"), cls="is-id")}'
            f'<div class="fui-transform-run">{button("secondary", "Show", "chart-line")}</div></div>')
    samples = table("Samples of db_pool_connections", th("Connections") + th("Value", num=True), [
        f'<tr><th scope="row" data-role="title"><span title="state=&quot;in_use&quot;">In use</span></th>{td("Value", "6", "fui-num", role="sub")}</tr>',
        f'<tr><th scope="row" data-role="title"><span title="state=&quot;idle&quot;">Idle</span></th>{td("Value", "4", "fui-num", role="sub")}</tr>',
        f'<tr><th scope="row" data-role="title"><span title="state=&quot;max&quot;">Maximum</span></th>{td("Value", "20", "fui-num", role="sub")}</tr>'])
    body = section("Now", now) + section("Since the last start", since) + section("The metric registry", pick + samples, count=f"{len(METRICS)} metrics")
    return server_page("metrics", body, "Counters since the last start.")


def logging_body():
    f = facts([("In effect", "<code>ferroehr=info,sqlx=warn</code>"), ("At boot", '<code title="Set by log.filter">ferroehr=info,sqlx=warn</code>')])
    reset = '<button type="button" class="fui-button fui-button-secondary" disabled title="The filter in effect is the boot filter">' + icon("rotate-ccw", "16") + "Reset to the boot filter</button>"
    form = (f'<div class="fui-transform-options">{field("lf", "New filter", input_("lf", "ferroehr=debug,sqlx=warn", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34)), cls="is-id")}'
            f'<div class="fui-transform-run fui-cluster">{link_button("primary", "Apply…", "server-logging-apply.html", "check")}{reset}</div></div>')
    return section("Filter", f) + section("Change the log filter", intro("Applies to this instance until it restarts.") + form)


def page_server_logging(overlay=None):
    return server_page("logging", logging_body(), "The live log filter of the instance that answered.", overlay=overlay)


def page_server_logging_apply():
    d = dialog("lg", "check", "Apply the new log filter?",
               '<p><code>ferroehr=debug,sqlx=warn</code></p><p>Until this instance restarts.</p>',
               button("secondary", "Cancel", keys="Esc") + button("primary", "Apply the filter", "check"))
    return page_server_logging(d)


def page_server_api():
    fams = [("all", "The complete surface"), ("ehr", "openEHR: EHR"), ("query", "openEHR: Query"), ("definition", "openEHR: Definition"), ("demographic", "openEHR: Demographic (in development)"),
            ("admin", "openEHR: Admin (in development)"), ("management", "FerroEHR: Status & Management"), ("terminology", "FerroEHR: Terminology"), ("relationships", "FerroEHR: Party Relationships"),
            ("events", "FerroEHR: Event Subscriptions"), ("audit", "FerroEHR: Audit"), ("smart", "FerroEHR: SMART Discovery")]
    pick = f'<div class="fui-transform-options">{field("api-f", "API family", select("api-f", fams, "ehr"))}<div class="fui-transform-run">{button("secondary", "Download OpenAPI", "download")}</div></div>'
    groups = [("EHR", [("POST", "/ehr", "Create an EHR"), ("GET", "/ehr", "Get an EHR by subject id"), ("PUT", "/ehr/{ehr_id}", "Create an EHR with an id"), ("GET", "/ehr/{ehr_id}", "Get an EHR by id")]),
              ("EHR status", [("GET", "/ehr/{ehr_id}/ehr_status/{version_uid}", "Get the EHR status by version id"), ("GET", "/ehr/{ehr_id}/ehr_status", "Get the EHR status at a time"),
                              ("PUT", "/ehr/{ehr_id}/ehr_status", "Update the EHR status"), ("GET", "/ehr/{ehr_id}/versioned_ehr_status", "Get the versioned EHR status"),
                              ("GET", "/ehr/{ehr_id}/versioned_ehr_status/revision_history", "Get its revision history"), ("GET", "/ehr/{ehr_id}/versioned_ehr_status/version", "Get its version at a time"),
                              ("GET", "/ehr/{ehr_id}/versioned_ehr_status/version/{version_uid}", "Get its version by id")]),
              ("Composition", [("POST", "/ehr/{ehr_id}/composition", "Create a composition"), ("GET", "/ehr/{ehr_id}/composition/{uid_based_id}", "Get a composition"),
                               ("PUT", "/ehr/{ehr_id}/composition/{uid_based_id}", "Update a composition"), ("DELETE", "/ehr/{ehr_id}/composition/{uid_based_id}", "Delete a composition"),
                               ("GET", "/ehr/{ehr_id}/versioned_composition/{versioned_object_uid}", "Get the versioned composition"),
                               ("GET", "/ehr/{ehr_id}/versioned_composition/{versioned_object_uid}/revision_history", "Get its revision history"),
                               ("GET", "/ehr/{ehr_id}/versioned_composition/{versioned_object_uid}/version", "Get its version at a time"),
                               ("GET", "/ehr/{ehr_id}/versioned_composition/{versioned_object_uid}/version/{version_uid}", "Get its version by id")]),
              ("Directory", [("POST", "/ehr/{ehr_id}/directory", "Create the directory"), ("PUT", "/ehr/{ehr_id}/directory", "Update the directory"), ("DELETE", "/ehr/{ehr_id}/directory", "Delete the directory"),
                             ("GET", "/ehr/{ehr_id}/directory", "Get a folder at a time"), ("GET", "/ehr/{ehr_id}/directory/{version_uid}", "Get a folder in a version")]),
              ("Contribution", [("POST", "/ehr/{ehr_id}/contribution", "Create a contribution"), ("GET", "/ehr/{ehr_id}/contribution/{contribution_uid}", "Get a contribution by id")]),
              ("Tags", [("GET", "/ehr/{ehr_id}/tags", "Get the EHR's tags"), ("GET", "/ehr/{ehr_id}/composition/{uid_based_id}/tags", "Get a composition's tags"),
                        ("PUT", "/ehr/{ehr_id}/composition/{uid_based_id}/tags", "Update a composition's tags"), ("DELETE", "/ehr/{ehr_id}/composition/{uid_based_id}/tags/{key}", "Delete a composition tag"),
                        ("GET", "/ehr/{ehr_id}/ehr_status/{uid_based_id}/tags", "Get the EHR status's tags"), ("PUT", "/ehr/{ehr_id}/ehr_status/{uid_based_id}/tags", "Update the EHR status's tags"),
                        ("DELETE", "/ehr/{ehr_id}/ehr_status/{uid_based_id}/tags/{key}", "Delete an EHR status tag")])]
    rows = []
    for g, eps in groups:
        rows.append(f'<tr class="fui-group-row"><th scope="rowgroup" colspan="2">{g} <span class="fui-small fui-muted">· {plural(len(eps), "endpoint")}</span></th></tr>')
        rows += [f'<tr><th scope="row" data-role="title"><code class="mk-method">{m}</code> <code class="fui-path">{e(p).replace("/", "/<wbr>")}</code></th>{td("Summary", sm, role="sub")}</tr>' for m, p, sm in eps]
    total = sum(len(x[1]) for x in groups)
    t = table("Endpoints of openEHR: EHR", th("Method and path") + th("Summary"), rows)
    return server_page("api", section("", pick) + section("Endpoints of openEHR: EHR", t, count=total),
                       "The OpenAPI documents the CDR serves for itself, by API family.")


SMART = """{
  "issuer": "https://idp.example.org/realms/ferroehr",
  "authorization_endpoint": "https://idp.example.org/realms/ferroehr/protocol/openid-connect/auth",
  "token_endpoint": "https://idp.example.org/realms/ferroehr/protocol/openid-connect/token",
  "grant_types_supported": [ "authorization_code", "client_credentials" ],
  "capabilities": [ "launch-standalone", "client-confidential-symmetric", "permission-v2" ],
  "scopes_supported": [ "openid", "fhirUser", "patient/composition-*.rs", "user/composition-*.cruds", "user/template-*.rs", "system/aql-*.rs" ],
  "code_challenge_methods_supported": [ "S256" ]
}"""


def page_server_smart():
    rows = scopes([scope("patient/composition-*.rs", "Patient", "Compositions of any template", "rs"), scope("user/composition-*.cruds", "User", "Compositions of any template", "cruds"),
                   scope("user/template-*.rs", "User", "Templates", "rs"), scope("system/aql-*.rs", "System", "Every stored and ad-hoc query", "rs", broad=True)], "Scopes the CDR supports")
    other = facts([("Also", "<code>openid</code>, <code>fhirUser</code>"), ("Grant types", "authorization_code, client_credentials"), ("PKCE", "S256")])
    return server_page("smart", section("Scopes it supports", rows, count=4) + section("Sign-in", other) + section("The document", codeview("/.well-known/smart-configuration", SMART, what="the discovery document")),
                       'The scopes the CDR supports; <a href="access.html">Your access</a> shows the ones you hold.')


def page_server_no_management():
    tl = [t for t in SERVER_TABS if t[0] in ("overview", "api", "smart")]
    note = section("", f'<p class="fui-small fui-muted">The CDR has no management endpoints turned on, so Health, Metrics and Logging are not offered.</p><p class="fui-small fui-muted">Set in ferroehr.toml · <a href="configuration-operations.html">Operations settings</a></p>')
    return server_page("overview", overview_body() + note, "What the CDR is and how it was built.", tabs_list=tl)


# ---- Audit log ----

def audit_heading():
    return page_heading("Audit log", subtitle="The CDR's security audit log: who accessed what, and with what outcome.")


def audit_rows(items=AUDIT):
    out = []
    for t, ev, who, pat, tone, word in items:
        p = masked(pat, static=True) if pat else '<span class="mk-dash">No patient</span>'
        out.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="audit-record.html">{e(ev)}</a></th>'
                   + td("When", ago(f"2026-10-21T{t}", seconds=True), role="sub") + td("Principal", '<span class="fui-muted">Unknown</span>' if who == "unknown" else e(who), role="meta-q")
                   + td("Patient", p, role="meta-2") + td("Outcome", state(tone, word), role="meta-1") + "</tr>")
    return table("Audit records", th("Event") + th("When", key="w", sort="descending") + th("Principal") + th("Patient") + th("Outcome"), out)


def audit_toolbar(value="", window=None, outcome=None):
    return tpl.toolbar("Principal or action", value, [tpl.filter_button("When", window), tpl.filter_button("Outcome", outcome), tpl.filter_button("Patient")])


def audit_page(body=None, tb=None, chips=(), foot=None, count=4812):
    from common import pager
    c = tpl.card(body if body is not None else audit_rows(), foot=pager("Audit pages", 4812, 10) if foot is None else foot, label="Audit records", title="Audit records", count=f"{count:,}")
    return shell("Audit log", "audit", audit_heading() + tpl.list_body("", tb or audit_toolbar(), tpl.active(chips), c))


def page_audit():
    return audit_page()


def page_audit_filtered():
    rows = [a for a in AUDIT if a[4] != "ok"]
    return audit_page(audit_rows(rows), audit_toolbar(window="today", outcome="failures"), ["When: today", "Outcome: failures"], foot="", count=len(rows))


AE = """{
  "resourceType": "AuditEvent",
  "type": { "system": "http://dicom.nema.org/resources/ontology/DCM", "code": "110110", "display": "Patient Record" },
  "action": "R",
  "recorded": "2026-10-21T09:41:58.204+02:00",
  "outcome": "0",
  "agent": [ { "who": { "display": "Demo Clinician" }, "requestor": true, "network": { "address": "10.20.4.17", "type": "2" } } ],
  "source": { "observer": { "display": "cdr.example.org" } },
  "entity": [
    { "what": { "identifier": { "system": "demo.hospital.mrn", "value": "••••••537" } }, "role": { "system": "http://terminology.hl7.org/CodeSystem/object-role", "code": "1", "display": "Patient" } },
    { "what": { "identifier": { "system": "urn:openehr:cdr.example.org:composition", "value": "0199f1d0-33aa-7b20-8c4e-2a9e51b07c11::cdr.example.org::3" } }, "role": { "system": "http://terminology.hl7.org/CodeSystem/object-role", "code": "4", "display": "Domain Resource" } }
  ]
}"""


def page_audit_record():
    f = facts([("When", when("2026-10-21T09:41:58", seconds=True, absolute=True)), ("Principal", "Demo Clinician, from 10.20.4.17"),
               ("Patient", masked("537", "demo.hospital.mrn")), ("What", '<a href="composition.html">Vital signs, version 3</a>'), ("Outcome", state("ok", "Success"))])
    h = page_heading("Read composition", crumbs=[("Audit log", "audit.html")], subtitle='Stored as a <span title="IHE ITI-81">FHIR AuditEvent</span>')
    body = sections(section("The event", f), section("As stored", intro("The patient identifier is masked here as in the list; Reveal shows it.") + codeview("AuditEvent", AE, what="the record")))
    return shell("Audit record", "audit", h + f'<div class="fui-stack">{body}</div>')


def page_audit_no_match():
    return audit_page(empty_line("No audit record matches these filters."), audit_toolbar("svc-etl", "today", "failures"), ["When: today", "Outcome: failures", "Search: svc-etl"], foot="", count=0)


def page_audit_store_off():
    body = ('<p>The CDR keeps no local audit store; its records go only to the forwarding target, so there is nothing to read here.</p>'
            '<p class="fui-small fui-muted">Set in ferroehr.toml · <a href="configuration-records.html">Records settings</a></p>')
    return shell("Audit log", "audit", audit_heading() + f'<div class="fui-stack">{body}</div>')


# ---- Subscriptions ----

def sub_sentence(kind, change, tmpl):
    parts = []
    parts.append({"COMPOSITION": "compositions", "EHR_STATUS": "EHR statuses", None: "versions of any kind"}[kind])
    if change:
        parts.append("deleted" if change == "523" else f"with change type {change}")
    if tmpl:
        parts.append(f"under {tmpl}")
    s = " ".join(parts)
    return ("Every committed version of " + s if not change else "Every committed version of " + s.replace(" deleted", "") + ", when deleted") if kind is None and change else f"Every committed version of {s}"


def sub_menu(name, enabled, open_=False):
    items = [menu_item("Edit", "pencil", href="subscription-edit.html"), menu_item("Disable" if enabled else "Enable", "ban" if enabled else "check"), MENU_SEP,
             menu_item("Delete…", "trash-2", danger=True, href="subscription-delete.html")]
    m = menu(items, label=f"Actions for {name}", sheet=True) if open_ else ""
    return (f'<span class="fui-anchor"><button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" '
            f'aria-label="Actions for {e(name)}" title="Actions">{icon("ellipsis", "16")}</button>{m}</span>')


def sub_rows(extra_row=False, no_broker=False):
    out = []
    items = ([("all-changes", None, None, None, True, "2026-10-21T09:42:05")] if extra_row else []) + list(SUBSCRIPTIONS)
    for name, kind, change, tmpl, on, created in items:
        sentence = sub_summary(kind or "", change or "", tmpl or "").rstrip(".")
        cls = "" if on else ' class="is-excluded"'
        out.append(f'<tr{cls}><th scope="row" data-role="title"><a class="fui-row-link" href="subscription-edit.html"><code>{e(name)}</code></a><span class="fui-row-sub">{sentence}</span></th>'
                   + td("State", (state("warn", "Waiting for the broker") if no_broker else state("ok", "Enabled")) if on else state("muted", "Disabled"), role="sub") + td("Created", ago(created), role="meta-q")
                   + td("", sub_menu(name, on), "fui-row-menu", role="menu") + "</tr>")
    return table("Subscriptions", th("Subscription") + th("State") + th("Created") + th('<span class="fui-sr-only">Actions</span>'), out)


def subs_page(overlay=None, toast_html="", extra_row=False, no_broker=False):
    h = page_heading("Subscriptions", actions=link_button("primary", "New subscription", "subscription-new.html", "plus"),
                     subtitle="Get a message on the event queue for each change that matches.")
    c = tpl.card(sub_rows(extra_row, no_broker), label="Subscriptions", title="Subscriptions", count=4 if extra_row else 3)
    top = notice("warn", "triangle-alert", "The CDR is not connected to a message broker, so no subscription delivers; messages start once it is.") if no_broker else ""
    return shell("Subscriptions", "subscriptions", h + tpl.list_body(top, "", "", c), overlay=overlay, toast_html=toast_html)


def page_subscriptions():
    return subs_page()


KIND_OPTS = [("", "Any kind"), ("COMPOSITION", "Composition"), ("EHR_STATUS", "EHR status"), ("FOLDER", "Folder")]
CHANGE_OPTS = [("", "Any change"), ("249", "Creation"), ("251", "Modification"), ("523", "Deletion")]
TMPL_OPTS = [("", "Any template"), ("Vital signs encounter.v1", "Vital signs encounter.v1"), ("Laboratory result report.v1", "Laboratory result report.v1")]


def sub_summary(kind="", change="", tmpl=""):
    """One sentence for what a subscription selects, the same in its list row and its dock."""
    what = {"": "version of any kind", "COMPOSITION": "composition", "EHR_STATUS": "EHR status", "FOLDER": "folder"}[kind]
    if change == "523":
        line = "Every deletion" + (f" of a {what}" if kind else ", of any kind")
    else:
        line = f"Every committed {what}" + (f", {dict(CHANGE_OPTS)[change].lower()} only" if change else "")
    if tmpl:
        line += f" under <code>{e(tmpl)}</code>"
    return line + "."


def sub_form(name="all-changes", edit=False, invalid=None, kind="", change="", tmpl=""):
    queue = f"ferroehr.events.{name}" if name and not invalid else None
    if edit:
        name_s = section("Name", f'<p><code>{e(name)}</code></p>' + facts([("Queue", f"<code>ferroehr.events.{e(name)}</code>")])
                         + '<p class="fui-help">The name cannot change: a new name would be a different queue.</p>')
    else:
        help_ = None if invalid else (f"Lowercase letters, digits and dashes. The queue is <code>ferroehr.events.{name}</code>. The name cannot change later." if queue
                                      else "Lowercase letters, digits and dashes. The name cannot change later.")
        nm = field("sn", "Name", input_("sn", name, "for example vital-signs-feed", extra='class="fui-input fui-mono" spellcheck="false"' + (' aria-invalid="true"' if invalid else "")),
                   help_=help_, error=invalid, cls="fui-span-2")
        nm = nm.replace('<label for="sn" class="fui-label">Name</label>', '<label for="sn" class="fui-sr-only">Name</label>', 1)
        name_s = section("Name", '<div class="fui-form-grid">' + nm + "</div>")
    sel = ('<div class="fui-form-grid">' + field("sk", "Kind", select("sk", KIND_OPTS, kind), cls="fui-span-2")
           + field("sc", "Change type", select("sc", CHANGE_OPTS, change), cls="fui-span-2")
           + field("st", "Template", select("st", TMPL_OPTS, tmpl), cls="fui-span-2") + "</div>")
    what = section("What it selects", sel)
    deliver = section("Delivery", f'<div class="fui-setting-rows">{setting_row("Deliver messages", "When off, the subscription is kept but sends nothing.", input_("se", "", type_="checkbox", extra="checked aria-label=" + chr(34) + "Deliver messages" + chr(34), cls="fui-switch"), id_="se")}</div>')
    return sections(name_s, what, deliver, card=True)


SUB_INTRO = "Get a message on the event queue for each change that matches."


def sub_dock(text, label, disabled=None, glyph="list-filter"):
    return edit_dock(msg(glyph, text), label, "subscriptions.html", disabled=disabled)


def page_subscription_new():
    return edit_shell("New subscription", "subscriptions", [("Subscriptions", "subscriptions.html")], SUB_INTRO,
                      sub_form(), sub_dock(sub_summary(), "Create subscription"))


def page_subscription_new_invalid():
    return edit_shell("New subscription", "subscriptions", [("Subscriptions", "subscriptions.html")], SUB_INTRO,
                      sub_form("All changes", invalid="Use lowercase letters, digits and dashes only: “All changes” has a capital and a space."),
                      sub_dock(sub_summary(), "Create subscription", disabled="Fix the name first"))


def page_subscription_name_taken():
    return edit_shell("New subscription", "subscriptions", [("Subscriptions", "subscriptions.html")], SUB_INTRO,
                      sub_form("vital-signs-feed", invalid="The CDR already has a subscription called vital-signs-feed. Choose another name, for example vital-signs-feed-2.", kind="COMPOSITION", tmpl="Vital signs encounter.v1"),
                      sub_dock(sub_summary("COMPOSITION", "", "Vital signs encounter.v1"), "Create subscription", disabled="Change the name first"))


def page_subscription_created(no_broker=False):
    t = toast("ok", "circle-check", "Subscription created", "<code>all-changes</code> · queue <code>ferroehr.events.all-changes</code>.", action=link_button("secondary", "Edit", "subscription-edit.html", small=True))
    return subs_page(toast_html=toasts(t), extra_row=True, no_broker=no_broker)


def page_subscription_created_no_broker():
    return page_subscription_created(True)


def page_subscription_edit():
    more = action_group("", [menu_item("Copy the queue name", "copy"), MENU_SEP, menu_item("Delete the subscription…", "trash-2", danger=True, href="subscription-delete.html")], label="More actions for the subscription")
    return edit_shell("Edit vital-signs-feed", "subscriptions", [("Subscriptions", "subscriptions.html")], "Saving replaces what this subscription selects.",
                      sub_form("vital-signs-feed", edit=True, kind="COMPOSITION", tmpl="Vital signs encounter.v1"),
                      sub_dock(sub_summary("COMPOSITION", "", "Vital signs encounter.v1"), "Save"), actions=more)


def page_subscription_delete():
    d = confirm_delete("del", "Delete the subscription vital-signs-feed?", "Removes the subscription for good. The queue and its messages stay on the broker; remove them there.", True, "Delete the subscription")
    return subs_page(d)


# ---- Configuration (file mode) ----

LINE = 'Set in <code>ferroehr.toml</code> · read at start · <a href="configuration-dialog.html">How configuration works</a>'
VLINE = 'Set in <code>ferroehr-viewer.toml</code> · read at start · <a href="configuration-dialog.html">How configuration works</a>'


def key_hint(k, show=False):
    return f'<button type="button" class="fui-key-hint{" is-shown" if show else ""}" aria-label="Copy the key {e(k)}" title="Copy the key"><code>{e(k)}</code>{icon("copy", "12")}</button>'


def srow(label, desc, value, key, hint=False):
    return (f'<div class="fui-setting-row{" mk-hover" if hint else ""}"><div class="fui-setting-text"><span class="fui-setting-label">{label}{key_hint(key, hint)}</span>'
            f'<p class="fui-setting-desc">{desc}</p></div><div class="fui-setting-control"><span class="fui-setting-value">{value}</span></div></div>')


def secret(ref):
    return f'<span class="fui-secret-now">{icon("lock", "12")}set, read from <code>{e(ref)}</code></span>'


def redacted():
    return f'<span class="fui-secret-now">{icon("lock", "12")}set, not shown</span>'


def on(v=True):
    return badge("ok", "On") if v else badge("neutral", "Off")


def config_rows(hint=False):
    return {
        "deployment": srow("Deployment profile", "Production keeps every production safeguard on; a sandbox says which are off.", "Production", "deployment_profile", hint)
        + srow("Spec profile", "The openEHR specification versions the CDR implements.", "<code>openehr-2026</code>", "spec_profile")
        + srow("Safeguards accepted as off", "Production safeguards this deployment runs without, each by its code.", '<span class="mk-dash">none</span>', "deployment_accepts")
        + srow("System id", "The id the CDR writes into every version it commits.", "<code>cdr.example.org</code>", "server.system_id"),
        "server": srow("Listener", "The socket address the REST API binds.", "<code>0.0.0.0:8080</code>", "server.bind", hint)
        + srow("Base path", "Where the openEHR REST API lives.", "<code>/ferroehr/rest/openehr/v1</code>", "server.base_path")
        + srow("Requests at once", "How many requests the CDR serves at once before it turns new ones away (503).", "512", "server.max_in_flight")
        + srow("Database", "The PostgreSQL URL of the running CDR.", secret("/run/secrets/ferroehr-db-url"), "db.url_file")
        + srow("Connections", "The most connections the pool opens.", "20", "db.max_connections")
        + srow("Statement timeout", "The longest one statement may run.", "30 s", "db.statement_timeout_ms")
        + srow("Query timeout", "The longest an AQL query may run.", "20 s", "query.timeout_ms")
        + srow("Result rows", "The most rows one query may return.", "10,000", "query.max_result_rows")
        + srow("Migrations", "Whether the CDR prepares its schema at start.", "Checked, never applied: a separate role migrates", "db.migrate"),
        "access": srow("Authentication", "Bearer tokens from this issuer, and Basic sign-in for the CDR's own accounts.", "<code>https://idp.example.org/realms/ferroehr</code>", "auth.issuers")
        + srow("Role-based access", "The CDR checks every request against the caller's role.", on(), "authz.enabled")
        + srow("Admin role", "The role that may use the Admin API and read the audit log.", "<code>admin</code>", "authz.admin_role")
        + srow("User role", "The role for reading and writing clinical content.", "<code>user</code>", "authz.user_role")
        + srow("Admin API", "Physical deletes of EHRs, templates and stored queries.", on(), "admin.enabled")
        + srow("SMART on openEHR", "SMART App Launch with scopes.", on(), "smart.enabled"),
        "records": srow("Version signing", "Whether each committed version is signed.", on(False), "signing.enabled")
        + srow("Audit", "Every access is recorded.", on(), "audit.enabled")
        + srow("Audit failure", "A request whose audit record cannot be written is refused.", "Fails closed", "audit.fail_mode")
        + srow("Local audit store", "Keeps the records the Audit log reads.", on(), "audit.local_store")
        + srow("Audit retention", "How long a record is kept.", "7 years", "audit.retention_years")
        + srow("Privacy scan", "Refuses a composition that carries an identifier outside the subject namespaces.", on(), "privacy.mode")
        + srow("Subject namespaces", "The namespaces a subject id may come from.", "<code>demo.hospital.mrn</code>", "privacy.subject_namespaces"),
        "integrations": srow("Event publishing", "Publishes committed versions to the broker.", on(), "events.enabled")
        + srow("Broker", "The AMQP URL.", secret("/run/secrets/ferroehr-amqp-url"), "events.url_file")
        + srow("Subscription API", "The API that creates and lists event subscriptions.", on(), "events.admin_api")
        + srow("Terminology API", "The API that looks up codes and expands value sets.", on(), "terminology.api_enabled")
        + srow("External terminology", "A FHIR terminology server the lookups go to.", on(False), "terminology.external.enabled"),
        "operations": srow("Log filter", 'The filter at boot; <a href="server-logging.html">Server › Logging</a> changes it until a restart.', "<code>ferroehr=info,sqlx=warn</code>", "log.filter", hint)
        + srow("Log format", "How each line is written.", "JSON", "log.format")
        + srow("Telemetry", "Where traces and metrics are sent.", redacted(), "telemetry.otlp_endpoint")
        + srow("Management surface", "Health, metrics, logging and configuration for the operator.", on(), "management.enabled")
        + srow("Management endpoints", "Who may read the build information, metrics, settings and log levels.", '<span title="info, metrics, env, loggers">Admin only</span>', "management.endpoints")
        + srow("Usage report", "Sends FerroPULSE a daily installation report with no patient data.", on(), "usage_report.enabled"),
        "viewer": srow("CDR address", "The address the viewer calls for every CDR request.", "<code>http://ferroehr:8080</code>", "cdr.base_url", hint)
        + srow("CDR base path", "Must match the base path the CDR serves its API under.", "<code>/ferroehr/rest/openehr/v1</code>", "cdr.base_path")
        + srow("Management address", "Where the viewer reads health, metrics and logging.", "<code>http://ferroehr:9100/management</code>", "cdr.management_base_url")
        + srow("Basic sign-in", "Offers the username and password form; the CDR checks the account.", on(), "auth.basic_enabled")
        + srow("OIDC sign-in", "The identity provider that signs people in.", "<code>https://idp.example.org/realms/ferroehr</code>", "auth.oidc.issuer")
        + srow("OIDC client secret", "Proves the viewer to the identity provider; read from a file and never shown.", secret("/run/secrets/viewer-oidc"), "auth.oidc.client_secret_file")
        + srow("Session idle time", "A session ends after this long without a request.", "60 min", "session.idle_minutes")
        + srow("Session key", "Seals the session cookie; every replica holds the same key.", secret("/run/secrets/viewer-session"), "session.secret_file")
        + srow("Language", "Used when the browser asks for a language the viewer does not have.", "English (en)", "ui.locale"),
    }


CONFIG_TABS = [("deployment", "Deployment", "configuration.html"), ("server", "Server and database", "configuration-server.html"), ("access", "Access", "configuration-access.html"),
               ("records", "Records", "configuration-records.html"), ("integrations", "Integrations", "configuration-integrations.html"),
               ("operations", "Operations", "configuration-operations.html"), ("viewer", "Viewer", "configuration-viewer.html")]


def mode_dialog():
    body = ('<p>The CDR reads <code>ferroehr.toml</code> once at start, and the viewer reads <code>ferroehr-viewer.toml</code>; an environment variable overrides a key in either.</p>'
            '<div class="fui-modal-body"><div class="fui-ownership">'
            f'<div><h3>{icon("file-text", "16")} Where you see it</h3><p class="fui-small fui-muted">Here, read-only: the CDR\'s settings as it serves them, redacted, and the viewer\'s own.</p></div>'
            f'<div><h3>{icon("refresh-cw", "16")} How you change it</h3><p class="fui-small fui-muted">Change the file, review it, and restart the CDR or the viewer.</p></div></div>'
            '<p class="fui-small fui-muted mk-mode-other">Nothing here can be changed from the viewer: the database holds clinical data, never settings.</p></div>')
    return dialog("mode", "info", "How configuration works", body, button("secondary", "Close", "x", keys="Esc", attrs="autofocus"), wide=True)


def config_page(tab, overlay=None, hint=False):
    rows = config_rows(hint)[tab]
    body = tpl.settings_panel(rows, VLINE if tab == "viewer" else LINE)
    tabs_html = tpl.settings_tabs("Configuration sections", CONFIG_TABS, tab, body, many=True)
    h = page_heading("Configuration", subtitle="The CDR's settings as it serves them, redacted, and the viewer's own.")
    return shell("Configuration", "configuration", h + f'<div class="fui-stack">{tabs_html}</div>', overlay=overlay)


def page_configuration():
    return config_page("deployment")


def page_configuration_server():
    return config_page("server")


def page_configuration_access():
    return config_page("access")


def page_configuration_records():
    return config_page("records")


def page_configuration_integrations():
    return config_page("integrations")


def page_configuration_operations():
    return config_page("operations")


def page_configuration_viewer():
    return config_page("viewer")


def page_configuration_dialog():
    return config_page("deployment", overlay=mode_dialog())


def page_configuration_key_hint():
    return config_page("deployment", hint=True)


# ---- Your preferences ----

PREF_TABS = [("appearance", "Appearance", "preferences.html"), ("reading", "Reading", "preferences-reading.html"), ("keyboard", "Keyboard", "preferences-keyboard.html")]


def pref_rows(theme="light", saved=None):
    sel = lambda id_, opts, cur: select(id_, opts, cur, cls="fui-select fui-setting-select")
    return {
        "appearance": setting_row("Theme", "Light, dark, or the one your system uses. Applies at once.",
                                  choice_cards("theme", "Theme", [("light", "Light", "Teal on white"), ("dark", "Dark", "Teal on ink"), ("system", "System", "Follows your device")], theme, theme_preview), saved=saved == "theme")
        + setting_row("Density", "Compact fits more rows in every list; comfortable, the default, gives rows and forms room.",
                      choice_cards("density", "Density", [("comfortable", "Comfortable", "More room"), ("compact", "Compact", "More rows")], "comfortable", density_preview))
        + setting_row("Reduce motion", "Fewer animations in menus, panels and trees.", sel("motion", [("system", "Follow the system"), ("always", "Always reduce")], "system"), id_="motion"),
        "reading": setting_row("Language", "English is the one language the viewer ships today.", sel("lang", [("browser", "Your browser's (English)"), ("en", "English")], "browser"), id_="lang")
        + setting_row("First day of the week", "Used by every date picker.", sel("week", [("monday", "Monday"), ("sunday", "Sunday")], "monday"), id_="week")
        + setting_row("Show a revealed identifier for", "How long a subject id stays shown after you reveal it; the viewer caps it at 2 minutes.",
                      sel("rev", [("15", "15 seconds"), ("30", "30 seconds"), ("60", "1 minute"), ("120", "2 minutes")], "60" if saved == "reveal" else "30"), id_="rev", saved=saved == "reveal"),
        "keyboard": setting_row("Shortcut hints", "The key caps after a label. Phones never show them.", sel("hints", [("wide", "On wide screens (default)"), ("never", "Never")], "wide"), id_="hints")
        + setting_row("All shortcuts", "Every shortcut of the screen you are on, in one list.", link_button("secondary", "Open the shortcut legend", "shortcuts.html", "keyboard", small=True, keys="?")),
    }


def preferences_page(tab="appearance", theme="light", saved=None, overlay=None, lock_dark=False):
    inner = tpl.settings_panel(pref_rows(theme, saved)[tab])
    reset = f'<div class="fui-settings-foot"><p class="fui-small fui-muted">Each preference is its own cookie in this browser, read when each page is drawn.</p>{link_button("ghost", "Reset to defaults…", "preferences-reset.html", "rotate-ccw", small=True)}</div>'
    content = (page_heading("Your preferences", subtitle="Only for you, in this browser.")
               + f'<div class="fui-stack">{tpl.settings_tabs("Preference sections", PREF_TABS, tab, inner)}{reset}</div>')
    html = shell("Your preferences", "", content, overlay=overlay)
    lock = "dark" if lock_dark else theme
    if lock in ("light", "dark"):
        html = html.replace('<html lang="en"', f'<html lang="en" data-lock-theme="{lock}"', 1)
    return html


def page_preferences():
    return preferences_page()


def page_preferences_reading():
    return preferences_page("reading")


def page_preferences_saved():
    return preferences_page("reading", saved="reveal")


def page_preferences_keyboard():
    return preferences_page("keyboard")


def page_preferences_reset():
    d = dialog("reset", "rotate-ccw", "Reset your preferences?",
               "<p>All your preferences go back to their defaults.</p>",
               button("secondary", "Cancel", keys="Esc") + button("primary", "Reset to defaults", "rotate-ccw"))
    return preferences_page(overlay=d)


def page_preferences_dark():
    return preferences_page(theme="dark", saved="theme", lock_dark=True)


# ---- Your access ----

def access_page(preview="", preview_items=None, basic=False):
    if basic:
        who = facts([("Signed in as", "<strong>ferroehr</strong>"), ("How", "Basic sign-in with a CDR account"), ("Role", '<span title="authz.basic_roles">admin, from the account\'s role</span>'),
                     ("Session", f"since {when('2026-10-21T08:30')}, ends after 60 minutes without a request")])
        sc = section("Scopes", quiet("A Basic session carries no SMART scopes; the CDR checks the account's role."), count="none")
    else:
        who = facts([("Signed in as", f"<strong>{OPERATOR}</strong>"), ("How", "OIDC at idp.example.org"), ("Role", "<code>admin</code>, from the token's roles claim"),
                     ("Session", f"since {when('2026-10-21T08:30')}, ends after 60 minutes without a request")])
        sc = section("Scopes", scopes([scope("user/composition-*.cruds", "User", "Compositions of any template", "cruds"),
                                       scope("user/template-*.rs", "User", "Templates", "rs"),
                                       scope("system/aql-*.rs", "System", "Every stored and ad-hoc query", "rs", broad=True)]), count=3)
    prev_items = preview_items if preview_items is not None else ""
    pv = (f'<div class="fui-transform-options">{field("sp", "Scope or claim", input_("sp", preview, "patient/composition-*.rs", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34) + " spellcheck=" + chr(34) + "false" + chr(34)), cls="is-id")}'
          f'<div class="fui-transform-run">{button("secondary", "Read it", "search")}</div></div>' + (scopes(prev_items, "Previewed scopes") if prev_items else quiet("Paste a scope to read it here.")))
    body = sections(section("Who you are", who), sc,
                    section("Read a scope", intro("Paste a scope to see what it allows.") + pv), card=True)
    h = page_heading("Your access", subtitle="What this session may do.")
    return shell("Your access", "", h + f'<div class="fui-stack">{body}</div>')


def page_access():
    return access_page()


def page_access_preview():
    items = [scope("patient/composition-*.rs", "Patient", "Compositions of any template", "rs"),
             scope("patient/COMPOSITION.read", "", "", "", refused="Not a SMART on openEHR scope. Use composition-&lt;template&gt;, for example composition-*.rs.")]
    return access_page("patient/composition-*.rs patient/COMPOSITION.read", items)


def page_access_basic():
    return access_page(basic=True).replace('<span class="fui-operator-name">Demo Operator</span>', '<span class="fui-operator-name">ferroehr</span>')


# ---- sign-in ----

def sign_in_card(notice_html="", state_=None, form_error=None):
    off = " disabled" if state_ == "redirecting" else ""
    if state_ == "redirecting":
        oidc = '<button type="button" class="fui-button fui-button-primary" aria-busy="true" disabled><span class="fui-spinner-ring" aria-hidden="true"></span>Taking you to idp.example.org…</button>'
    else:
        oidc = f'<a href="sign-in-redirecting.html" rel="external" class="fui-button fui-button-primary">{icon("log-in", "16")}Sign in at idp.example.org</a>'
    err = f'<div role="alert" class="fui-sign-in-notice fui-tone-bad">{form_error}</div>' if form_error else ""
    uval = "ferroehr" if form_error else ""
    basic = (f'<form class="fui-stack" method="post" action="dashboard.html">{err}{field("u", "Username", input_("u", uval, extra="autocomplete=" + chr(34) + "username" + chr(34) + off))}'
             f'{field("pw", "Password", input_("pw", "", type_="password", extra="autocomplete=" + chr(34) + "current-password" + chr(34) + off + (" aria-invalid=" + chr(34) + "true" + chr(34) if form_error else "")))}'
             f'{button("secondary", "Sign in with a CDR account", "key-round", type_="submit", attrs=off.strip())}</form>')
    dep = '<p class="fui-small fui-muted mk-sign-in-line">Evaluation deployment: synthetic data only. Ask the operator for an account.</p>'
    mark = lockup().replace('<a href="dashboard.html" class="fui-lockup">', '<span class="fui-lockup">', 1).replace("</span></a>", "</span></span>", 1)
    return (
        f'<div class="fui-card fui-sign-in"><div class="fui-sign-in-lockup">{mark}</div>'
        '<h1 class="mk-sign-in-title">Sign in</h1>'
        + notice_html + oidc + '<p class="fui-small fui-muted mk-sign-in-line">or</p>' + basic + dep
        + '<div class="fui-sign-in-links"><a href="https://ferroehr.eu/docs/latest/" rel="external noopener">The FerroEHR book</a></div></div>'
        '<p class="fui-sign-in-caption">The viewer of the FerroEHR clinical data repository.</p>'
    )


def sign_in_page(**kw):
    return centred("Sign in", f'<div class="fui-stack mk-sign-in-stack">{sign_in_card(**kw)}</div>')


def n(tone, role, text):
    return f'<div role="{role}" class="fui-sign-in-notice fui-tone-{tone}">{text}</div>'


def page_sign_in():
    return sign_in_page()


def page_sign_in_redirecting():
    return sign_in_page(state_="redirecting")


def page_sign_in_refused():
    return sign_in_page(form_error="Not signed in: the CDR refused that username or password.")


def page_sign_in_expired():
    return sign_in_page(notice_html=n("warn", "status", "Your session ended. Sign in again to return to the page you were on."))


def page_sign_in_signed_out():
    return sign_in_page(notice_html=n("ok", "status", "You are signed out."))


def page_sign_in_cdr_unreachable():
    return sign_in_page(notice_html=n("warn", "status", f'The CDR did not answer at {when("2026-10-21T09:42:11", seconds=True)}. You can still try to sign in. {button("secondary", "Try again", "refresh-cw", small=True)}'))
