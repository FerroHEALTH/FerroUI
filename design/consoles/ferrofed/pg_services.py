# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Services: the outside services the gateway asks, each with one home. The
binding sections ([pixm], [pdqm], [pmir], [nl_gf], [xcpd]) and the audit,
terminology and directory dependencies. A service's own connection is set
here; its per-node entries live on each node's Settings; the consent
disclosure and the emergency rule live in Configuration › Consent."""
from common import (
    action_group, MENU_SEP, button, definitions, dock, dock_field, e, field, health_dot, hint, icon,
    input_, link_button, menu, menu_item, notice, page_heading, panel_head, rel, section, sections, select, shell,
    state, table, tabbed, tabs, td, th, url_code, when,
)
import data
from data import MITZ_SINCE, SERVICE_GROUPS, SERVICES, SET_UP, service_counts, word
from parts import skeleton_lines
from pg_orgs import sparkline
import routes
import tpl

GROUPS = SERVICE_GROUPS
SETUP_LABEL = {"xcpd": "New service", "mcsd": "New service"}
APPROVAL_WORD = {"identity-services": "Identity services", "localization": "Identity services and localization", "consent": "Identity services, localization and consent"}
APPROVAL_CODE = {"identity-services": "identity-services", "localization": "identity-services, localization", "consent": "identity-services, localization, consent"}
MITZ_PROBES = [176, 182, 170, 188, 179, 175, 181, 184, 172, 178, 185, 180, 177, 183, 179, 174, 186, 181, 178, 182, 176, 180, 183, 179, 175, 181, 178, 180, 182, 180]
MITZ_FAIL = tuple(range(19, 30))


def host(url):
    return url.split("//", 1)[1].split("/", 1)[0] if url else None


def svc(key):
    return next(s for s in SERVICES if s[1] == key)


def svc_menu(key, name, open_=False, probe_hover=False):
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="Actions for {e(name)}" title="More actions">{icon("ellipsis", "16")}</button>')
    m = menu([menu_item("Probe now", "activity", hover=probe_hover), menu_item("Open", "arrow-right", href=routes.svc_href(key), keys="Enter"),
              menu_item("Edit the connection", "pencil", href="service-connection-edit.html" if key == "mitz" else routes.svc_href(key, "connection")),
              MENU_SEP, menu_item("Stop asking this service…", "ban", danger=True)], label=f"Actions for {name}", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def policy_line(edit=False):
    if edit:
        return ('<div class="mk-group-policy mk-group-policy-edit fui-columns">'
                + field("lo-fail", f'When the localizer fails{hint("federation.localization.on_failure")}', select("lo-fail", [("closed", "Ask no node"), ("ask-all", "Ask every node")], "closed"),
                        help_="Ask no node: the query fails and says why (§14.1).")
                + field("lo-timeout", f'Localizer budget{hint("federation.localization.timeout_ms")}', '<div class="fui-input-group">' + input_("lo-timeout", "1200", extra='inputmode="numeric" class="fui-input fui-config-changed"') + '<span class="fui-input-suffix">ms</span></div>',
                        help_="was 1,500 ms · a part of the overall budget, with Mitz and PDQm")
                + "</div>")
    return f'<span class="mk-group-policy">When the localizer fails: ask no node, after 1,500 ms {link_button("ghost", "Edit", "services-localization-edit.html", "pencil", small=True)}</span>'


def svc_rows(only_attention=False, menu_open=None, loc_edit=False):
    order = {"bad": 0, "warn": 1, "neutral": 2, "ok": 3, "unset": 4}
    out = []
    for g in GROUPS:
        items = [x for x in SERVICES if x[0] == g]
        if only_attention:
            items = [x for x in items if x[6] in ("bad", "warn", "neutral")]
            if not items:
                continue
        items.sort(key=lambda x: order[x[6]])
        pol = policy_line(loc_edit) if g == "Localization" and not only_attention else ""
        out.append(f'<tr class="mk-group-row"><th scope="rowgroup" colspan="5"><span class="mk-group-name">{g}</span>{pol}</th></tr>')
        for gg, k, n, role, sec, url, t, p, appr, enable in items:
            if t == "unset":
                out.append(f'<tr class="is-unset"><th scope="row" data-role="title"><span class="mk-svc-name">{e(n)}</span></th>'
                           + td("Host", f'<span class="fui-small fui-muted">{e(enable)}</span>', role="sub") + td("Health", '<span class="fui-small fui-muted">Not set up</span>', role="meta-2")
                           + td("Last probe", "", role="wide") + td("", link_button("secondary", "Set it up", "service-new.html", "plus", small=True), "fui-row-menu", role="menu") + "</tr>")
                continue
            tip = {"warn": "Degraded · 503 Service Unavailable", "neutral": "Not probed yet: asked only when the FHIR face writes a summary", "ok": f"Reachable · median {p} ms"}[t]
            health = f'<span title="{e(tip)}">{state(t, "503" if t == "warn" else None)}</span>'
            last = ("probed " + rel("09:41:52")) if t != "neutral" else '<span class="mk-dash">never probed</span>'
            out.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link mk-svc-name" href="{routes.svc_href(k)}">{e(n)}</a><span class="mk-svc-role">{e(role)}</span></th>'
                       + td("Host", f'<code title="{e(url)}">{e(host(url))}</code>', role="sub") + td("Health", health, role="meta-2")
                       + td("Last probe", last, role="meta-q") + td("", svc_menu(k, n, menu_open == k), "fui-row-menu", role="menu") + "</tr>")
    return out


def services_page(toast_html="", overlay=None, loc_edit=False, attention=False, menu_open=None, dock_html=""):
    rows = svc_rows(attention, menu_open, loc_edit)
    c = service_counts()
    att = c.get("warn", 0) + c.get("neutral", 0)
    chips = tpl.summary([(f"{len(SERVICES)} services", None, not attention), (f"{c.get('warn', 0)} degraded", "warn", False), (f"{c.get('neutral', 0)} not probed", "neutral", False),
                         (f"{len(SERVICES) - len(SET_UP)} not set up", None, False), (f"Needs attention · {att}", None, attention)])
    t = tpl.card(table("Services", th("Service") + th("Host") + th("Health") + th("Last probe") + th('<span class="fui-sr-only">Actions</span>'), rows, scroll_label="Services"),
                 label="Services", count=f"{len(SET_UP)} set up, {len(SERVICES) - len(SET_UP)} not set up" if not attention else f"{att} need attention").replace("fui-list-card", "fui-list-card mk-svc-list")
    act = "" if loc_edit else link_button("primary", "New service", "services-setup.html", "plus")
    head = page_heading("Services", actions=act, subtitle="The outside services the gateway asks for identity, localization, consent, audit and terminology.")
    content = head + tpl.list_body(chips, "", tpl.active(["Needs attention"], shown=(att, len(SET_UP))) if attention else "", t)
    return shell("Services", "services", content, toast_html=toast_html, overlay=overlay, dock_html=dock_html)


def page_services():
    return services_page()


def page_services_attention():
    return services_page(attention=True)


def page_services_menu():
    return services_page(menu_open="term")


def page_services_setup():
    opts = "".join(
        f'<label class="fui-choice-card"><input type="radio" class="fui-sr-only" name="svc-new"{" checked" if k == "xcpd" else ""} />'
        f'<span class="fui-choice-card-text"><span class="fui-choice-card-title">{n}</span><span class="fui-choice-card-sub">{e(enable[0].upper() + enable[1:])} · {e(g)}</span></span>'
        f'<span class="fui-choice-card-check" aria-hidden="true">{icon("check", "12")}</span></label>'
        for g, k, n, role, sec, url, t, p, appr, enable in SERVICES if t == "unset")
    dialog = ('<div role="dialog" aria-modal="true" aria-labelledby="su-title" class="fui-dialog fui-modal mk-open" open>'
              f'<h2 id="su-title" class="fui-dialog-title">{icon("plug", "18")} New service</h2>'
              '<p>Only the services this gateway does not ask yet are offered.</p>'
              f'<fieldset class="fui-choice-cards mk-svc-choices"><legend class="fui-sr-only">Service</legend>{opts}</fieldset>'
              f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{link_button("primary", "Continue", "service-new.html", "arrow-right")}</div></div>')
    return services_page(overlay=dialog)


def page_services_localization_edit():
    d = dock(dock_field("lo-reason", "Reason", "The pilot's queries need an answer sooner; give the NVI 1.2 s", "Required: kept with the revision", keys="r"),
             link_button("secondary", "Discard", "services.html", "x") + link_button("primary", "Review the changes", "config-review.html", "git-compare"))
    return services_page(loc_edit=True, dock_html=d)


# ---- one service (T2) ----

def service_page(key, tab, panel, dock_html="", top="", drafts=()):
    g, k, n, role, sec, url, t, p, appr, enable = svc(key)
    own = key == "mitz"
    edit_href = "service-connection-edit.html" if own else routes.svc_href(key, "connection")
    ag = "" if drafts else action_group(link_button("primary", "Edit the connection", edit_href, "pencil"), [menu_item("Probe now", "activity"), MENU_SEP, menu_item("Stop asking this service…", "ban", danger=True)], label=f"More actions for {n}")
    head = page_heading(e(n), actions=ag, badge=state(t, "503" if t == "warn" else None),
                        subtitle=f"{e(g)} · {e(role[0].upper() + role[1:])}{hint(sec)}", crumbs=[("Services", "services.html")])
    items = [(tk, {"overview": "Overview", "connection": "Connection", "requests": "Requests", "audit": "Audit"}[tk] + ('<span class="fui-tab-draft">draft</span>' if tk in drafts else ""), routes.svc_href(key, tk))
             for tk in routes.SVC_TABS]
    content = head + f'<div class="fui-stack">{top}{tabbed(tabs(n, items, tab), panel)}</div>'
    return shell(n, "services", content, dock_html=dock_html)


def mitz_overview():
    hc = section("Health", f'<p class="fui-cluster">{state("warn", "503")}<span class="fui-small fui-muted">since {MITZ_SINCE} · last probe {rel("09:41:52")}, every 60 s</span></p>'
                 + sparkline(MITZ_PROBES, MITZ_FAIL) + '<p class="fui-spark-legend"><span>30 min ago</span><span>ms · a dot: a 503</span><span>now</span></p>'
                 + '<p class="fui-small">While Mitz answers 503 the pre-filter has no answer. Each query records the consent decision as unavailable and asks every node, which decides itself (§13.2.1).</p>')
    errs = section("Recent errors", f'<ul class="mk-plain-list fui-small"><li>{rel("09:41:52")} · 503 Service Unavailable · a consent question</li>'
                   f'<li>{rel("09:40:52")} · 503 Service Unavailable · a probe</li><li>{rel("09:31:04")} · 503 Service Unavailable · the first one</li></ul>')
    mapped = section("Who it serves", definitions([("Nodes mapped", '5 of 6 · <a href="node-settings.html">node-a-lab is not mapped</a>'),
                                                   ("Consent policy", 'Exclusions disclosed, and in an emergency the pre-filter applies · <a href="configuration-consent.html">Configuration › Consent</a>')]))
    return sections(hc, errs, mapped)


def page_service():
    top = notice("warn", "triangle-alert", f"<strong>Degraded.</strong> Mitz answers 503 since {MITZ_SINCE}. Queries go on without its answer; the access record says so for each one.")
    return service_page("mitz", "overview", mitz_overview(), top=top)


def connection_view():
    d = definitions([("URL", url_code("https://mitz.example.org/geslotenautorisatievraag")), ("Purpose of use", f"Treatment{hint('TREAT')}"),
                     ("Data categories", "<code>GGC002</code>"), ("BSN namespaces", f"<code>urn:oid:2.999.1</code>{hint('nl_gf.mitz.namespaces')}"),
                     ("Timeout", "1,000 ms, one round of questions, within the query's overall budget"),
                     ("Client certificate", f'<span class="fui-secret-now">{icon("lock", "12")}set, read from <code>/run/secrets/ferrofed/mitz-client.pem</code></span>'),
                     ("Password", f'<span class="fui-secret-now">{icon("lock", "12")}set, read from <code>/run/secrets/ferrofed/mitz-password</code></span>'),
                     ("Approval", f'{APPROVAL_WORD["consent"]}{hint(APPROVAL_CODE["consent"])}: a second operator approves a change, which then runs on trial for 2 minutes')])
    elsewhere = definitions([("Each node's care provider", '5 of 6 nodes mapped, on each node\'s Settings · <a href="node-settings.html">node-a-lab is not</a>'),
                             ("Consent exclusions", 'Disclosed · <a href="configuration-consent.html">Configuration › Consent</a>'),
                             ("In an emergency", 'The pre-filter applies · <a href="configuration-consent.html">Configuration › Consent</a>')])
    return sections(panel_head("How the gateway reaches Mitz.", link_button("secondary", "Edit", "service-connection-edit.html", "pencil", small=True, keys="e")) + section("Connection", d), section("Set elsewhere", elsewhere))


def connection_edit():
    from pg_config import secret_field
    fields = ('<div class="fui-columns">'
              + field("mz-url", "URL", input_("mz-url", "https://mitz.example.org/geslotenautorisatievraag", extra='spellcheck="false" class="fui-input fui-mono"'))
              + field("mz-purpose", "Purpose of use", select("mz-purpose", [("TREAT", "Treatment (TREAT)"), ("COC", "Continuity of care (COC)")], "TREAT"))
              + field("mz-timeout", "Timeout", '<div class="fui-input-group">' + input_("mz-timeout", "1500", extra='inputmode="numeric" class="fui-input fui-config-changed"') + '<span class="fui-input-suffix">ms</span></div>', help_="was 1,000 ms · with PDQm and the localizer, below the overall budget")
              + field("mz-cats", "Data categories", input_("mz-cats", "GGC002", extra='spellcheck="false" class="fui-input fui-mono"'), help_="The Mitz categories asked about, comma-separated.")
              + field("mz-ns", "BSN namespaces", input_("mz-ns", "urn:oid:2.999.1", extra='spellcheck="false" class="fui-input fui-mono"'), help_="nl_gf.mitz.namespaces")
              + "</div>")
    secrets = secret_field("Client certificate", "keep", now="/run/secrets/ferrofed/mitz-client.pem", idx=20, key="client_identity_file") + secret_field("Password", "keep", now="/run/secrets/ferrofed/mitz-password", idx=21, key="credentials.password_file")
    return sections(panel_head("A draft of this tab, based on revision 7: 1 change. A second operator approves it, then it runs on trial for 2 minutes after the apply.", '')
                    + section("Connection", fields) + section("Client certificate and password", secrets))


def page_service_connection():
    return service_page("mitz", "connection", connection_view())


def page_service_connection_edit():
    d = dock(dock_field("mz-reason", "Reason", "Mitz answers slower in the pilot; give it 1.5 s", "Required: kept with the revision", keys="r"),
             link_button("secondary", "Discard", "service-connection.html", "x") + link_button("primary", "Review the changes", "config-review.html", "git-compare"))
    return service_page("mitz", "connection", connection_edit(), dock_html=d, drafts=("connection",))


def mitz_requests():
    rows = [("09:41:52", "a consent question for 3 nodes", "503", 180, "Unavailable: Mitz did not answer", "01JAF3M0R2KD"),
            ("09:37:10", "a consent question for 4 nodes", "503", 176, "Unavailable: Mitz did not answer", "01JAF3KG7VNA"),
            ("09:29:44", "a consent question for 4 nodes", "200", 212, "2 permitted, 2 denied", "01JAF3K1T9PE"),
            ("09:24:31", "a consent question for 2 nodes", "200", 188, "2 permitted", "01JAF3JX4QRB")]
    body = [f'<tr><th scope="row">{rel(t)}</th>' + td("Question", e(q)) + td("Status", f'<span class="fui-pill fui-tone-{"ok" if st == "200" else "bad"}">{st}</span>') + td("Latency", f"{lat:,} ms", "fui-num")
            + td("Consent decision", e(a)) + td("Request id", f"<code>{rid}</code>") + "</tr>" for t, q, st, lat, a, rid in rows]
    tbl = table("Questions to Mitz", th("Time", "t", "descending") + th("Question") + th("Status") + th("Latency", num=True) + th("Consent decision") + th("Request id"), body)
    tools = tpl.toolbar("A request id", "", [tpl.filter_button("Status")])
    return sections(f'<div class="fui-panel-section"><p class="fui-small fui-muted">Patient identifiers and values are not shown here; the access record keeps them.</p>{tools}</div>' + tbl)


def page_service_requests():
    return service_page("mitz", "requests", mitz_requests())


def mitz_audit():
    rows = [("2026-10-08T13:25", "Demo Operator", "Confirmation", "confirmed revision 2 after its trial", None),
            ("2026-10-08T13:22", "Demo Reviewer", "Approval", "approved revision 2", None),
            ("2026-10-08T13:15", "Demo Operator", "Change", "turned on the Mitz pre-filter · revision 2", "For the pilot")]
    body = [f'<tr><th scope="row">{rel(t)}</th>' + td("By", e(by), "mk-nowrap") + td("Kind", e(k)) + td("What", e(w)) + td("Reason", e(r) if r else '<span class="mk-dash">–</span>') + "</tr>" for t, by, k, w, r in rows]
    return sections(f'<div class="fui-panel-section"><p class="fui-small fui-muted">Every change to Mitz, with its approval and confirmation, newest first.</p></div>'
                    + table("Audit", th("When", "t", "descending") + th("By") + th("Kind") + th("What") + th("Reason"), body))


def page_service_audit():
    return service_page("mitz", "audit", mitz_audit())


def page_service_new():
    from pg_config import secret_field
    fields = ('<div class="fui-columns">'
              + field("xc-sender", "The gateway's device OID", input_("xc-sender", "2.999.40.1", extra='class="fui-input fui-mono"'))
              + field("xc-home", "Home community", input_("xc-home", "2.999.40", extra='class="fui-input fui-mono"'))
              + field("xc-url", "Responding gateway", input_("xc-url", "https://xcpd.region.example.org/RespondingGateway", extra='spellcheck="false" class="fui-input fui-mono"'))
              + field("xc-device", "Its device OID", input_("xc-device", "2.999.50.1", extra='class="fui-input fui-mono"'))
              + field("xc-audit", "Audit messages", select("xc-audit", [("log", "To the gateway's log"), ("repository", "To the audit record repository")], "repository"))
              + "</div>")
    body = sections(
        f'<div class="fui-panel-section"><p class="fui-small fui-muted">A draft based on revision 7. A second operator approves it ({APPROVAL_WORD["localization"].lower()}); each node\'s community is then set on the node.</p>'
        + field("xc-kind", "Which service", select("xc-kind", [("xcpd", "XCPD"), ("mcsd", "Care services directory")], "xcpd"), help_="Only the services not set up yet are offered.") + "</div>"
        + section("Connection", fields) + section("Client certificate", secret_field("Client certificate", "file", now="", now_state="unset", path="/run/secrets/ferrofed/xcpd-client.pem", idx=30, key="client_identity_file")))
    content = page_heading("New service", subtitle="A service the gateway does not ask yet.", crumbs=[("Services", "services.html")]) + f'<div class="fui-stack">{body}</div>'
    d = dock(dock_field("xc-reason", "Reason", "The region's XCPD community goes live on 1 November", "Required: kept with the revision", keys="r"),
             link_button("secondary", "Discard", "services.html", "x") + link_button("primary", "Review the changes", "config-review.html", "git-compare"))
    return shell("New service", "services", content, dock_html=d)


# ---- route-only pages: every other service's tabs ----

def generic_overview(key):
    g, k, n, role, sec, url, t, p, appr, enable = svc(key)
    if t == "neutral":
        line = "Not probed yet: the gateway asks it only when the FHIR face writes a summary."
        h = f'<p>{state(t)} <span class="fui-small fui-muted">{line}</span></p>'
    else:
        h = (f'<p class="fui-cluster">{state(t)}<span class="fui-small fui-muted">last probe {rel("09:41:52")}, every 60 s · median {p} ms</span></p>'
             + sparkline([p + (i * 7) % 5 for i in range(30)]) + '<p class="fui-spark-legend"><span>30 min ago</span><span>response time, ms</span><span>now</span></p>')
    return sections(section("Health", h))


def generic_connection(key):
    g, k, n, role, sec, url, t, p, appr, enable = svc(key)
    rows = [("URL", url_code(url)), ("Section", f"<code>{e(sec)}</code>")]
    if appr:
        rows.append(("Approval", APPROVAL_WORD[appr] + hint(APPROVAL_CODE[appr])))
    return sections(section("Connection", definitions(rows)))


def generic_requests(key):
    from common import quiet_line
    return quiet_line(f"No request to {svc(key)[2]} in the last 24 hours that the access record keeps.")


def generic_audit(key):
    r = data.REVISIONS[0]
    body = [f'<tr><th scope="row">{rel(r[3])}</th>' + td("By", e(r[2])) + td("Kind", "Change") + td("What", f"set up from the file · revision 1") + td("Reason", e(r[4])) + "</tr>"]
    return sections(table("Audit", th("When", "t", "descending") + th("By") + th("Kind") + th("What") + th("Reason"), body))


def route_pages():
    out = {}
    for s in SET_UP:
        k = s[1]
        if k == "mitz":
            continue
        out[routes.slug("svc", k, "overview")] = lambda k=k: service_page(k, "overview", generic_overview(k))
        out[routes.slug("svc", k, "connection")] = lambda k=k: service_page(k, "connection", generic_connection(k))
        out[routes.slug("svc", k, "requests")] = lambda k=k: service_page(k, "requests", generic_requests(k))
        out[routes.slug("svc", k, "audit")] = lambda k=k: service_page(k, "audit", generic_audit(k))
    return out
