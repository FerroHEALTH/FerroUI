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
    t = tpl.settings_tabs("Server sections", tabs_list, tab, f'<div class="fui-settings-panel">{intro_}{panel_html}</div>', many=True)
    return shell("Server", "server", h + f'<div class="fui-stack">{t}</div>', overlay=overlay, sandbox=sandbox)


def overview_body(sandbox=False):
    status = facts([("Status", state("ok", "Up") + ' <span class="fui-small fui-muted">answered 13 s ago</span>'), ("Version", f"FerroEHR {VERSION}"),
                    ("Deployment", ("Sandbox · 3 separations open" if sandbox else "Production")), ("openEHR REST", "ITS-REST 1.1.0")])
    manifest = facts([("Solution", "FerroEHR, by Cadasto B.V."), ("Conformance profile", "STANDARD"),
                      ("API groups mounted", "EHR, Query, Definition, Demographic, Admin"), ("Read from", "<code>OPTIONS /ferroehr/<wbr>rest/<wbr>openehr/<wbr>v1</code> · in full on <a href=\"about.html\">About</a>")])
    build = facts([("Git commit", "<code>7c1e9d2</code>"), ("Built", when("2026-10-19T06:12")), ("rustc", "1.99.1"), ("PostgreSQL target", "18.6"),
                   ("Spec profile", "<code>openehr-2026</code>: RM 1.2.0, BASE 1.3.0, TERM 3.1.0, AM 1.4.0 and 2.4.0"),
                   ("Audit", "On · fails closed · local store · kept 7 years"), ("Support until", "2027-10")])
    parts = [section("Status", status)]
    if sandbox:
        gaps = table("Open separations", th("Separation") + th("What it means"), [
            f'<tr><th scope="row" data-role="title">{state("warn", "Shared database credential")}</th>{td("What it means", "Schema changes and the running CDR use one PostgreSQL role.", role="sub")}</tr>',
            f'<tr><th scope="row" data-role="title">{state("warn", "Shared cluster")}</th>{td("What it means", "The clinical, party and audit databases share one PostgreSQL cluster.", role="sub")}</tr>',
            f'<tr><th scope="row" data-role="title">{state("warn", "Migrations on the runtime credential")}</th>{td("What it means", "The CDR prepares its own schema at start.", role="sub")}</tr>'])
        parts.append(section("Open separations", intro("A sandbox has not made these separations and must not hold real patient data.") + gaps, count=3))
    parts += [section("Conformance manifest", manifest), section("Build", build)]
    return "".join(parts)


def page_server():
    return server_page("overview", overview_body(), "From the CDR's status document, its conformance manifest and its build information.")


def page_server_sandbox():
    return server_page("overview", overview_body(True), "From the CDR's status document, its conformance manifest and its build information.", sandbox=True)


def page_server_health():
    rows = [("Database", "ok", "Up", "PostgreSQL 18.6 answered in 4 ms"), ("Migrations", "ok", "Up", "41 applied, none pending"),
            ("Audit store", "ok", "Up", "Chain verified 09:30"), ("Event publisher", "warn", "Degraded", "Reconnecting to the broker, attempt 3"),
            ("Terminology server", "muted", "Off", "Not configured")]
    t = table("Dependency indicators", th("Dependency") + th("State") + th("Detail"),
              [f'<tr><th scope="row" data-role="title">{n}</th>{td("State", state(tone, w), role="sub")}{td("Detail", d, role="meta-q")}</tr>' for n, tone, w, d in rows])
    note = section("", '<p class="fui-small fui-muted">The top bar\'s pill says whether the API answers; these say whether what it depends on is healthy.</p>')
    return server_page("health", section("Dependencies", t, count=5) + note, "From the management surface's health endpoint, with each indicator's detail.")


METRICS = ["aql_queries_total", "compositions_committed_total", "db_pool_connections", "http_request_duration_seconds", "http_requests_in_flight", "audit_events_total"]


def page_server_metrics():
    from common import stat_card
    tiles = (f'<div class="fui-grid fui-grid-stats">{stat_card("activity", "3", "Requests in flight")}{stat_card("file-text", "184", "Compositions committed")}'
             f'{stat_card("square-terminal", "1,206", "AQL queries")}{stat_card("database", "6 of 20", "Database connections")}</div>')
    pick = (f'<div class="fui-transform-options">{field("m-pick", "Metric", select("m-pick", [(m, m) for m in METRICS], "db_pool_connections"))}'
            f'<div class="fui-transform-run">{button("secondary", "Show", "chart-line")}</div></div>')
    samples = table("Samples of db_pool_connections", th("Labels") + th("Value", num=True), [
        f'<tr><th scope="row" data-role="title"><code>state="in_use"</code></th>{td("Value", "6", "fui-num", role="sub")}</tr>',
        f'<tr><th scope="row" data-role="title"><code>state="idle"</code></th>{td("Value", "4", "fui-num", role="sub")}</tr>',
        f'<tr><th scope="row" data-role="title"><code>state="max"</code></th>{td("Value", "20", "fui-num", role="sub")}</tr>'])
    body = section("Since the last start", tiles) + section("The metric registry", pick + samples, count=f"{len(METRICS)} metrics")
    return server_page("metrics", body, "The numbers Prometheus scrapes, read from the management surface; for alerting, scrape it instead.")


def logging_body():
    f = facts([("In effect", "<code>ferroehr=info,sqlx=warn</code>"), ("At boot", "<code>ferroehr=info,sqlx=warn</code>, from <code>log.filter</code>")])
    form = (f'<div class="fui-transform-options">{field("lf", "New filter", input_("lf", "ferroehr=debug,sqlx=warn", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34)))}'
            f'<div class="fui-transform-run">{link_button("primary", "Apply…", "server-logging-apply.html", "check")}{button("secondary", "Reset to the boot filter", "rotate-ccw")}</div></div>')
    return section("Filter", f) + section("Change it", intro("Applies to this instance only, until it restarts; set log.filter for a lasting change.") + form)


def page_server_logging(overlay=None):
    return server_page("logging", logging_body(), "The live log filter of the instance that answered.", overlay=overlay)


def page_server_logging_apply():
    d = dialog("lg", "check", "Apply the filter ferroehr=debug,sqlx=warn?",
               "<p>Logging changes at once for every request this instance serves, until it restarts. Other instances behind the load balancer keep theirs.</p>",
               button("secondary", "Cancel", keys="Esc") + button("primary", "Apply the filter", "check"))
    return page_server_logging(d)


def page_server_api():
    fams = [("all", "The complete surface"), ("ehr", "openEHR: EHR"), ("query", "openEHR: Query"), ("definition", "openEHR: Definition"), ("demographic", "openEHR: Demographic"),
            ("admin", "openEHR: Admin"), ("management", "FerroEHR: Status & Management"), ("terminology", "FerroEHR: Terminology"), ("relationships", "FerroEHR: Party Relationships"),
            ("events", "FerroEHR: Event Subscriptions"), ("audit", "FerroEHR: Audit"), ("smart", "FerroEHR: SMART Discovery")]
    pick = f'<div class="fui-transform-options">{field("api-f", "API family", select("api-f", fams, "ehr"))}<div class="fui-transform-run">{button("secondary", "Download OpenAPI", "download")}</div></div>'
    eps = [("GET", "/ehr/{ehr_id}", "Get an EHR by id"), ("POST", "/ehr", "Create an EHR"), ("PUT", "/ehr/{ehr_id}", "Create an EHR with an id"),
           ("GET", "/ehr/{ehr_id}/ehr_status", "Get the EHR status"), ("PUT", "/ehr/{ehr_id}/ehr_status", "Update the EHR status"),
           ("POST", "/ehr/{ehr_id}/composition", "Create a composition"), ("GET", "/ehr/{ehr_id}/composition/{uid_based_id}", "Get a composition"),
           ("PUT", "/ehr/{ehr_id}/composition/{versioned_object_uid}", "Update a composition"), ("DELETE", "/ehr/{ehr_id}/composition/{preceding_version_uid}", "Delete a composition")]
    t = table("Endpoints of openEHR: EHR", th("Method") + th("Path") + th("Summary"),
              [f'<tr><th scope="row" data-role="title"><code>{m}</code></th>{td("Path", "<code>" + e(p).replace("/", "/<wbr>") + "</code>", role="sub")}{td("Summary", s, role="meta-q")}</tr>' for m, p, s in eps])
    return server_page("api", section("", pick) + section("Compositions and EHR status", t + '<p class="fui-small fui-muted">The family\'s other groups (EHR, directory, contribution, versions) follow in the same table.</p>', count=9),
                       "The OpenAPI documents the CDR serves for itself, by API family.")


SMART = """{
  "issuer": "https://idp.example.org/realms/ferroehr",
  "authorization_endpoint": "https://idp.example.org/realms/ferroehr/protocol/openid-connect/auth",
  "token_endpoint": "https://idp.example.org/realms/ferroehr/protocol/openid-connect/token",
  "capabilities": [ "launch-standalone", "client-confidential-symmetric", "permission-v2" ],
  "scopes_supported": [ "openid", "fhirUser", "patient/COMPOSITION.rs", "user/COMPOSITION.cruds", "system/*.rs" ],
  "code_challenge_methods_supported": [ "S256" ]
}"""


def page_server_smart():
    return server_page("smart", section("", codeview("/.well-known/smart-configuration", SMART, what="the discovery document")),
                       "The SMART discovery document; its scopes are what the CDR supports, not what you hold (Your access says that).")


def page_server_no_management():
    tl = [t for t in SERVER_TABS if t[0] in ("overview", "api", "smart")]
    note = section("", '<p class="fui-small fui-muted">The CDR serves no management surface, so Health, Metrics and Logging are not offered; <code>[management] enabled</code> turns it on.</p>')
    return server_page("overview", overview_body() + note, "From the CDR's status document, its conformance manifest and its build information.", tabs_list=tl)


# ---- Audit log ----

def audit_heading():
    return page_heading("Audit log", subtitle="The CDR's security audit trail: who accessed what, and with what outcome.")


def audit_rows(items=AUDIT):
    out = []
    for t, ev, who, pat, tone, word in items:
        p = masked(pat, static=True) if pat else '<span class="mk-dash">none</span>'
        out.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="audit-record.html">{e(ev)}</a></th>'
                   + td("When", ago(f"2026-10-21T{t}", seconds=True), role="sub") + td("Principal", e(who), role="meta-q")
                   + td("Patient", p, role="meta-2") + td("Outcome", state(tone, word), role="meta-1") + "</tr>")
    return table("Audit records", th("Event") + th("When", key="w", sort="descending") + th("Principal") + th("Patient") + th("Outcome"), out)


def audit_toolbar(value="", window=None, outcome=None):
    return tpl.toolbar("Principal or action", value, [tpl.filter_button("When", window), tpl.filter_button("Outcome", outcome), tpl.filter_button("Patient")])


def audit_page(body=None, tb=None, chips=(), foot=None):
    from common import pager
    c = tpl.card(body if body is not None else audit_rows(), foot=pager("Audit pages", 4812, 10) if foot is None else foot, label="Audit records", title="Audit records")
    return shell("Audit log", "audit", audit_heading() + tpl.list_body("", tb or audit_toolbar(), tpl.active(chips), c))


def page_audit():
    return audit_page()


def page_audit_filtered():
    rows = [a for a in AUDIT if a[4] != "ok"]
    return audit_page(audit_rows(rows), audit_toolbar(window="Today", outcome="Failures"), ["When: today", "Outcome: minor or serious failure"], foot="")


AE = """{
  "resourceType": "AuditEvent",
  "type": { "system": "http://dicom.nema.org/resources/ontology/DCM", "code": "110110", "display": "Patient Record" },
  "action": "R",
  "recorded": "2026-10-21T09:41:58.204+02:00",
  "outcome": "0",
  "agent": [ { "who": { "display": "Demo Clinician" }, "requestor": true, "network": { "address": "10.20.4.17", "type": "2" } } ],
  "source": { "observer": { "display": "cdr.example.org" } },
  "entity": [
    { "what": { "identifier": { "system": "demo.hospital.mrn", "value": "••••••537" } }, "role": { "code": "1", "display": "Patient" } },
    { "what": { "reference": "composition/0199f1d0-33aa-7b20-8c4e-2a9e51b07c11::cdr.example.org::3" } }
  ]
}"""


def page_audit_record():
    f = facts([("Event", "Read composition"), ("When", when("2026-10-21T09:41:58", seconds=True)), ("Principal", "Demo Clinician, from 10.20.4.17"),
               ("Patient", masked("537", "demo.hospital.mrn")), ("What", '<a href="composition.html">Vital signs, version 3</a>'), ("Outcome", state("ok", "Success"))])
    h = page_heading("Read composition", crumbs=[("Audit log", "audit.html")], subtitle=f"Recorded {when('2026-10-21T09:41:58', seconds=True)} · ITI-81 AuditEvent")
    body = sections(section("The event", f), section("As stored", intro("The patient identifier is masked here as in the list; Reveal shows it.") + codeview("AuditEvent", AE, what="the record")))
    return shell("Audit record", "audit", h + f'<div class="fui-stack">{body}</div>')


def page_audit_no_match():
    return audit_page(empty_line("No audit record matches these filters."), audit_toolbar("svc-etl", "Today", "Failures"), ["When: today", "Outcome: minor or serious failure", "Principal: svc-etl"], foot="")


def page_audit_store_off():
    body = (f'<section class="fui-card fui-panel"><div class="fui-panel-body fui-stack"><p><strong>The CDR keeps no local audit store.</strong></p>'
            '<p class="fui-small fui-muted">Its audit records go only to the forwarding target it is configured with, so there is nothing to read here. <code>[audit] local_store</code> in <code>ferroehr.toml</code> turns it on.</p>'
            f'<div class="fui-cluster">{link_button("secondary", "Configuration: Records", "configuration-records.html", "file-text")}</div></div></section>')
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


def sub_rows(extra_row=False):
    out = []
    items = ([("all-changes", None, None, None, True, "2026-10-21T09:42:05")] if extra_row else []) + list(SUBSCRIPTIONS)
    for name, kind, change, tmpl, on, created in items:
        sentence = {"vital-signs-feed": "Every committed composition under Vital signs encounter.v1", "ehr-status-changes": "Every committed EHR status",
                    "deletions": "Every deletion, of any kind", "all-changes": "Every committed version of any kind"}[name]
        cls = "" if on else ' class="is-excluded"'
        out.append(f'<tr{cls}><th scope="row" data-role="title"><a class="fui-row-link" href="subscription-edit.html"><code>{e(name)}</code></a><span class="fui-row-sub">{sentence}</span></th>'
                   + td("State", state("ok", "Enabled") if on else state("muted", "Disabled"), role="sub") + td("Created", ago(created), role="meta-q")
                   + td("", sub_menu(name, on), "fui-row-menu", role="menu") + "</tr>")
    return table("Subscriptions", th("Subscription") + th("State") + th("Created") + th('<span class="fui-sr-only">Actions</span>'), out)


def subs_page(overlay=None, toast_html="", extra_row=False):
    h = page_heading("Subscriptions", actions=link_button("primary", "New subscription", "subscription-new.html", "plus"),
                     subtitle="Get a message on the event queue for each change that matches.")
    c = tpl.card(sub_rows(extra_row), label="Subscriptions", title="Subscriptions", count=4 if extra_row else 3)
    return shell("Subscriptions", "subscriptions", h + tpl.list_body("", "", "", c), overlay=overlay, toast_html=toast_html)


def page_subscriptions():
    return subs_page()


KIND_OPTS = [("", "Any kind"), ("COMPOSITION", "Composition"), ("EHR_STATUS", "EHR status"), ("FOLDER", "Folder")]
CHANGE_OPTS = [("", "Any change"), ("249", "Creation"), ("251", "Modification"), ("523", "Deletion")]
TMPL_OPTS = [("", "Any template"), ("Vital signs encounter.v1", "Vital signs encounter.v1"), ("Laboratory result report.v1", "Laboratory result report.v1")]


def sub_summary(kind="", change="", tmpl=""):
    k = dict(KIND_OPTS)[kind].lower() if kind else "any kind"
    k = {"composition": "composition", "ehr status": "EHR status", "folder": "folder"}.get(k, k)
    c = dict(CHANGE_OPTS)[change].lower() if change else "any change"
    t = tmpl or "any template"
    if kind:
        return f"Every committed {k}, {c}, {t}." if kind == "COMPOSITION" else f"Every committed {k}, {c}."
    return f"Every committed version of any kind, {c}, {t}."


def sub_form(name="all-changes", edit=False, invalid=None, kind="", change="", tmpl=""):
    queue = f"ferroehr.events.{name}" if name and not invalid else None
    if edit:
        nm = field("sn", "Name", input_("sn", name, extra='class="fui-input fui-mono" readonly'),
                   help_=f"Queue <code>ferroehr.events.{name}</code> · fixed: a new name would be a different queue.", cls="fui-span-2")
    else:
        help_ = None if invalid else (f"Lowercase letters, digits and dashes. Queue <code>ferroehr.events.{name}</code> · can\'t be changed later." if queue
                                      else "Lowercase letters, digits and dashes; it names the queue and can\'t be changed later.")
        nm = field("sn", "Name", input_("sn", name, "for example vital-signs-feed", extra='class="fui-input fui-mono" spellcheck="false"' + (' aria-invalid="true"' if invalid else "")),
                   help_=help_, error=invalid, cls="fui-span-2")
    nm = nm.replace('<label for="sn" class="fui-label">Name</label>', '<label for="sn" class="fui-sr-only">Name</label>', 1)
    name_s = section("Name", '<div class="fui-form-grid">' + nm + "</div>")
    sel = ('<div class="fui-form-grid">' + field("sk", "Kind", select("sk", KIND_OPTS, kind), cls="fui-span-2")
           + field("sc", "Change type", select("sc", CHANGE_OPTS, change), cls="fui-span-2")
           + field("st", "Template", select("st", TMPL_OPTS, tmpl), cls="fui-span-2") + "</div>")
    what = section("What it selects", sel + load_report("neutral", sub_summary(kind, change, tmpl), glyph="list-filter"))
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
                      sub_dock("Fix the name: lowercase letters, digits and dashes only.", "Create subscription", disabled="Fix the name first", glyph="circle-x"))


def page_subscription_name_taken():
    return edit_shell("New subscription", "subscriptions", [("Subscriptions", "subscriptions.html")], SUB_INTRO,
                      sub_form("vital-signs-feed", invalid="The CDR already has a subscription called vital-signs-feed (409); choose another name.", kind="COMPOSITION", tmpl="Vital signs encounter.v1"),
                      sub_dock("Not created: the name is taken.", "Create subscription", glyph="circle-x"))


def page_subscription_created(no_broker=False):
    if no_broker:
        t = toast("warn", "triangle-alert", "Created, not delivering yet", "all-changes is stored, but the CDR is not connected to a broker; messages start once it is.",
                  action=link_button("secondary", "Open", "subscription-edit.html", small=True))
    else:
        t = toast("ok", "circle-check", "Subscription created", "all-changes · queue ferroehr.events.all-changes.", action=link_button("secondary", "Open", "subscription-edit.html", small=True))
    return subs_page(toast_html=toasts(t), extra_row=True)


def page_subscription_created_no_broker():
    return page_subscription_created(True)


def page_subscription_edit():
    return edit_shell("Edit", "subscriptions", [("Subscriptions", "subscriptions.html"), ("vital-signs-feed", "subscriptions.html")], "Saving replaces every predicate.",
                      sub_form("vital-signs-feed", edit=True, kind="COMPOSITION", tmpl="Vital signs encounter.v1"),
                      sub_dock(sub_summary("COMPOSITION", "", "Vital signs encounter.v1"), "Save"))


def page_subscription_delete():
    d = confirm_delete("del", "Delete the subscription vital-signs-feed?", "Removes the subscription for good; the CDR stops binding its queue. The queue stays on the broker with what it holds, so reap it there.", True, "Delete the subscription")
    return subs_page(d)


# ---- Configuration (file mode) ----

LINE = 'Set in <code>ferroehr.toml</code> · read at start · <a href="configuration-dialog.html">How configuration works</a>'
VLINE = 'Set in <code>ferroehr-viewer.toml</code> · read at start · <a href="configuration-dialog.html">How configuration works</a>'


def key_hint(k, show=False):
    return f'<code class="mk-key-hint{" is-shown" if show else ""}" title="Copy the key">{e(k)}{icon("copy", "12")}</code>'


def srow(label, desc, value, key, hint=False):
    return (f'<div class="fui-setting-row{" mk-hover" if hint else ""}"><div class="fui-setting-text"><span class="fui-setting-label">{label}{key_hint(key, hint)}</span>'
            f'<p class="fui-setting-desc">{desc}</p></div><div class="fui-setting-control"><span class="fui-setting-value">{value}</span></div></div>')


def secret(ref):
    return f'<span class="fui-secret-now">{icon("lock", "12")}set, read from <code>{e(ref).replace("/", "/<wbr>")}</code></span>'


def redacted():
    return f'<span class="fui-secret-now">{icon("lock", "12")}set, not shown</span>'


def on(v=True):
    return badge("ok", "On") if v else badge("neutral", "Off")


def config_rows(hint=False):
    return {
        "deployment": srow("Deployment profile", "Production makes the separations a real deployment needs; a sandbox says what it has not made.", "Production", "deployment_profile", hint)
        + srow("Spec profile", "The openEHR specification versions the CDR implements.", "<code>openehr-2026</code>", "spec_profile")
        + srow("Accepted gaps", "Separations a production deployment declares it accepts, each by its code.", '<span class="mk-dash">none</span>', "deployment_accepts")
        + srow("System id", "The id the CDR writes into every version it commits.", "<code>cdr.example.org</code>", "server.system_id"),
        "server": srow("Listener", "The socket address the REST API binds.", "<code>0.0.0.0:8080</code>", "server.bind", hint)
        + srow("Base path", "Where the openEHR REST API lives; the viewer's cdr.base_path mirrors it.", "<code>/ferroehr/<wbr>rest/<wbr>openehr/<wbr>v1</code>", "server.base_path")
        + srow("Requests at once", "How many requests the CDR serves before it answers 503.", "512", "server.max_in_flight")
        + srow("Database", "The PostgreSQL URL of the running CDR.", secret("/run/secrets/ferroehr-db-url"), "db.url_file")
        + srow("Connections", "The pool's size.", "4 to 20", "db.max_connections")
        + srow("Statement timeout", "The longest one statement may run.", "30 s", "db.statement_timeout_ms")
        + srow("Migrations", "Whether the CDR prepares its schema at start.", "Checked, never applied: a separate role migrates", "db.migrate"),
        "access": srow("Authentication", "Bearer tokens from the issuers below; Basic for the accounts in authz.basic_roles.", "<code>https://idp.example.org/<wbr>realms/<wbr>ferroehr</code>", "auth.issuers")
        + srow("Role-based access", "The CDR checks every request against the caller's role.", on(), "authz.enabled")
        + srow("Admin role", "The role that may use the Admin API and read the audit trail.", "<code>admin</code>", "authz.admin_role")
        + srow("User role", "The role for reading and writing clinical content.", "<code>user</code>", "authz.user_role")
        + srow("Admin API", "Physical deletes of EHRs, templates and stored queries; off by default.", on(), "admin.enabled")
        + srow("SMART on openEHR", "SMART App Launch with scopes.", on(), "smart.enabled"),
        "records": srow("Version signing", "Whether each committed version is signed.", on(False), "signing.enabled")
        + srow("Audit", "Every access is recorded.", on(), "audit.enabled")
        + srow("Audit failure", "A request whose audit record cannot be written is refused.", "Fails closed", "audit.fail_mode")
        + srow("Local audit store", "Keeps the records the Audit log reads.", on(), "audit.local_store")
        + srow("Audit retention", "How long a record is kept.", "7 years", "audit.retention_years")
        + srow("Privacy scan", "Refuses a composition that carries an identifier outside the subject namespaces.", "Refuse", "privacy.mode")
        + srow("Subject namespaces", "The namespaces a subject id may come from.", "<code>demo.hospital.mrn</code>", "privacy.subject_namespaces"),
        "integrations": srow("Query timeout", "The longest an AQL query may run.", "20 s", "query.timeout_ms")
        + srow("Result rows", "The most rows one query may return.", "10,000", "query.max_result_rows")
        + srow("Event publishing", "Publishes committed versions to the broker.", on(), "events.enabled")
        + srow("Broker", "The AMQP URL.", secret("/run/secrets/ferroehr-amqp-url"), "events.url_file")
        + srow("Subscription API", "Serves Subscriptions in the viewer.", on(), "events.admin_api")
        + srow("Terminology API", "Serves Terminology in the viewer.", on(), "terminology.api_enabled")
        + srow("External terminology", "A FHIR terminology server the lookups go to.", on(False), "terminology.external.enabled"),
        "operations": srow("Log filter", "The filter at boot; Server › Logging changes it until a restart.", "<code>ferroehr=info,sqlx=warn</code>", "log.filter", hint)
        + srow("Log format", "How each line is written.", "JSON", "log.format")
        + srow("Telemetry", "Where traces and metrics are sent.", redacted(), "telemetry.otlp_endpoint")
        + srow("Management surface", "Health, metrics, logging and configuration for the operator.", on(), "management.enabled")
        + srow("Management endpoints", "Each one's access level.", "info, metrics, env, loggers: admin only", "management.endpoints")
        + srow("Usage report", "Sends FerroPULSE a daily installation report with no patient data.", on(), "usage_report.enabled"),
        "viewer": srow("CDR", "Where the viewer reaches the CDR.", "<code>http://ferroehr:8080</code>", "cdr.base_url", hint)
        + srow("CDR base path", "Mirrors the CDR's server.base_path.", "<code>/ferroehr/<wbr>rest/<wbr>openehr/<wbr>v1</code>", "cdr.base_path")
        + srow("Management", "The CDR's management surface, base path included.", "<code>http://ferroehr:9100/<wbr>management</code>", "cdr.management_base_url")
        + srow("Basic sign-in", "Offers the username and password form, checked against the CDR.", on(), "auth.basic_enabled")
        + srow("OIDC sign-in", "Offers sign-in at the identity provider.", "<code>https://idp.example.org/<wbr>realms/<wbr>ferroehr</code>", "auth.oidc.issuer")
        + srow("OIDC client secret", "Read from a file; never shown.", secret("/run/secrets/viewer-oidc"), "auth.oidc.client_secret_file")
        + srow("Session idle time", "A session ends after this long without a request.", "60 min", "session.idle_minutes")
        + srow("Session key", "Seals the session cookie; every replica holds the same.", secret("/run/secrets/viewer-session"), "session.secret_file")
        + srow("Language", "Used when the browser names none the viewer ships.", "English (en)", "ui.locale"),
    }


CONFIG_TABS = [("deployment", "Deployment", "configuration.html"), ("server", "Server and database", "configuration-server.html"), ("access", "Access", "configuration-access.html"),
               ("records", "Records", "configuration-records.html"), ("integrations", "Integrations", "configuration-integrations.html"),
               ("operations", "Operations", "configuration-operations.html"), ("viewer", "Viewer", "configuration-viewer.html")]


def mode_dialog():
    body = ('<p>FerroEHR runs in <strong>file mode</strong>: the CDR reads <code>ferroehr.toml</code>, then <code>FERROEHR_</code> variables and <code>--set</code>, once at start; the viewer reads <code>ferroehr-viewer.toml</code> and <code>FERROEHR_VIEWER__</code> the same way.</p>'
            '<div class="fui-modal-body"><div class="fui-ownership">'
            f'<div><h3>{icon("file-text", "16")} Where you see it</h3><p class="fui-small fui-muted">Here, read-only: the CDR\'s settings as it serves them, redacted, and the viewer\'s own.</p></div>'
            f'<div><h3>{icon("refresh-cw", "16")} How you change it</h3><p class="fui-small fui-muted">Change the file, review it, and restart the CDR or the viewer.</p></div></div>'
            '<p class="fui-small fui-muted mk-mode-other">The CDR\'s PostgreSQL holds clinical data and never configuration, so there is no console mode.</p></div>')
    return dialog("mode", "info", "How configuration works", body, button("secondary", "Close", "x", keys="Esc", attrs="autofocus"), wide=True)


def config_page(tab, overlay=None, hint=False):
    rows = config_rows(hint)[tab]
    body = tpl.settings_panel(rows, VLINE if tab == "viewer" else LINE)
    tabs_html = tpl.settings_tabs("Configuration sections", CONFIG_TABS, tab, body, many=True)
    help_ = link_button("ghost", "How configuration works", "configuration-dialog.html", "info", attrs='aria-haspopup="dialog"')
    h = page_heading("Configuration", actions=help_, subtitle="The CDR's settings as it serves them, redacted, and the viewer's own.")
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
        "keyboard": setting_row("Shortcut hints", "The key caps after a label. Phones never show them.", sel("hints", [("wide", "Wide screens only (default)"), ("always", "Always"), ("never", "Never")], "wide"), id_="hints")
        + setting_row("All shortcuts", "Every shortcut of the screen you are on, in one list.", link_button("secondary", "Open the shortcut legend", "shortcuts.html", "keyboard", small=True, keys="?")),
    }


def preferences_page(tab="appearance", theme="light", saved=None, overlay=None, lock_dark=False):
    inner = tpl.settings_panel(pref_rows(theme, saved)[tab])
    reset = f'<div class="fui-settings-foot"><p class="fui-small fui-muted">Each preference is its own cookie in this browser, read when each page is drawn.</p>{link_button("ghost", "Reset to defaults…", "preferences-reset.html", "rotate-ccw", small=True)}</div>'
    content = (page_heading("Your preferences", subtitle="Only for you, in this browser.")
               + f'<div class="fui-stack">{tpl.settings_tabs("Preference sections", PREF_TABS, tab, inner)}{reset}</div>')
    html = shell("Your preferences", "", content, overlay=overlay)
    if lock_dark:
        html = html.replace('<html lang="en"', '<html lang="en" data-lock-theme="dark"', 1)
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
               "<p>Theme, density, motion, the language, the first day of the week, the reveal time and the hints go back to their defaults: their cookies in this browser are cleared. Your session and the CDR are not touched.</p>",
               button("secondary", "Cancel", keys="Esc") + button("primary", "Reset to defaults", "rotate-ccw"))
    return preferences_page(overlay=d)


def page_preferences_dark():
    return preferences_page(theme="dark", saved="theme", lock_dark=True)


# ---- Your access ----

def access_page(preview="", preview_items=None, basic=False):
    if basic:
        who = facts([("Signed in as", f"<strong>ferroehr</strong>"), ("How", "Basic: the viewer replays this CDR account on every request"), ("Role", "<code>admin</code>, from the CDR's authz.basic_roles")])
        sc = section("This session's scopes", '<p class="fui-small">A Basic session carries no SMART scopes; the CDR checks the account\'s role.</p>')
    else:
        who = facts([("Signed in as", f"<strong>{OPERATOR}</strong>"), ("How", "OIDC at idp.example.org"), ("Role", "<code>admin</code>, from the token's roles claim"),
                     ("Session", f"since {when('2026-10-21T08:30')}, ends after 60 minutes without a request")])
        sc = section("This session's scopes", scopes([scope("user/COMPOSITION.cruds", "User", "Compositions, any template", "cruds"),
                                                      scope("user/EHR_STATUS.rus", "User", "EHR statuses", "rus"),
                                                      scope("system/*.rs", "System", "Every resource", "rs", broad=True)]), count=3)
    prev_items = preview_items if preview_items is not None else ""
    pv = (f'<div class="fui-transform-options">{field("sp", "Scope or claim", input_("sp", preview, "patient/COMPOSITION.rs", extra="class=" + chr(34) + "fui-input fui-mono" + chr(34) + " spellcheck=" + chr(34) + "false" + chr(34)))}'
          f'<div class="fui-transform-run">{button("secondary", "Read it", "search")}</div></div>' + (scopes(prev_items, "Previewed scopes") if prev_items else quiet("Paste a scope to read it here.")))
    body = sections(section("Who you are", who), sc,
                    section("Read a scope", intro("Parsed with the grammar the CDR enforces; a scope narrows access and never grants it.") + pv), card=True)
    h = page_heading("Your access", subtitle="What this session may do, and who says so; the CDR stays the one that decides.")
    return shell("Your access", "", h + f'<div class="fui-stack">{body}</div>')


def page_access():
    return access_page()


def page_access_preview():
    items = [scope("patient/COMPOSITION.rs", "Patient compartment", "Compositions", "rs"),
             scope("patient/composition.read", "", "", "", refused="Not a SMART v2 scope: the resource is upper case (COMPOSITION) and the operations are letters from cruds, so .rs, not .read.")]
    return access_page("patient/COMPOSITION.rs patient/composition.read", items)


def page_access_basic():
    return access_page(basic=True).replace('<span class="fui-operator-name">Demo Operator</span>', '<span class="fui-operator-name">ferroehr</span>')


# ---- sign-in ----

def sign_in_card(notice_html="", state_=None):
    if state_ == "redirecting":
        oidc = '<button type="button" class="fui-button fui-button-primary" aria-busy="true" disabled><span class="fui-spinner-ring" aria-hidden="true"></span>Taking you to idp.example.org…</button>'
    else:
        oidc = f'<a href="sign-in-redirecting.html" rel="external" class="fui-button fui-button-primary">{icon("log-in", "16")}Sign in at idp.example.org</a>'
    basic = (f'<form class="fui-stack" method="post" action="dashboard.html">{field("u", "Username", input_("u", "", extra="autocomplete=" + chr(34) + "username" + chr(34)))}'
             f'{field("pw", "Password", input_("pw", "", type_="password", extra="autocomplete=" + chr(34) + "current-password" + chr(34)))}'
             f'{button("secondary", "Sign in with a CDR account", "key-round", type_="submit")}</form>')
    dep = ('<div class="fui-sign-in-notice fui-tone-info" role="note">Evaluation deployment: synthetic data only. Ask the operator for an account.</div>')
    return (
        f'<div class="fui-card fui-sign-in"><div class="fui-sign-in-lockup">{lockup()}</div>'
        '<h1 class="mk-sign-in-title">Sign in</h1>'
        + notice_html + dep + oidc + '<p class="fui-small fui-muted mk-sign-in-line">or</p>' + basic
        + '<p class="fui-small fui-muted mk-sign-in-line">The browser keeps only a sealed session cookie; your password and tokens stay on the viewer\'s server.</p>'
        '<div class="fui-sign-in-links"><a href="#" rel="external noopener">API reference</a><a href="#" rel="external noopener">The FerroEHR book</a></div></div>'
        '<p class="fui-sign-in-caption">The viewer of the FerroEHR clinical data repository.</p>'
    )


def sign_in_page(**kw):
    return centred("Sign in", f'<div class="fui-stack mk-sign-in-stack">{sign_in_card(**kw)}</div>')


def n(tone, role, text):
    return f'<div role="{role}" class="fui-sign-in-notice fui-tone-{tone}">{text}</div>'


def page_sign_in():
    return sign_in_page()


def page_sign_in_redirecting():
    return sign_in_page(state_="redirecting", notice_html=n("info", "status", "Opening idp.example.org; you come back to the page you were on."))


def page_sign_in_refused():
    return sign_in_page(notice_html=n("bad", "alert", "Not signed in: the CDR refused that username or password."))


def page_sign_in_expired():
    return sign_in_page(notice_html=n("warn", "status", "Your session ended. Sign in again to return to the page you were on."))


def page_sign_in_signed_out():
    return sign_in_page(notice_html=n("ok", "status", "You are signed out."))


def page_sign_in_cdr_unreachable():
    return sign_in_page(notice_html=n("warn", "status", "The CDR does not answer, so this page offers what the viewer is configured for; a sign-in will say if the CDR is still down."))
