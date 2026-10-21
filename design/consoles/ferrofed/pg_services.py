# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Services: the outside services the gateway asks, each with one home. The
binding sections ([pixm], [pdqm], [pmir], [nl_gf.nvi], [xcpd], [nl_gf.mitz])
and the audit, terminology and directory dependencies. A service's own
connection is set here; its per-node entries live on each node's Settings;
the consent and localization policies live in Configuration."""
from common import (
    action_group, MENU_SEP, badge, button, card, definitions, dock, dock_field, e, empty_state, facts_row, field, health_dot, icon,
    input_, link_button, menu, menu_item, notice, page_heading, pill, select, shell, table, tabbed, tabs, td, textarea,
    th, titled_panel, toast, toasts, when,
)
from data import STATE_WORD
from parts import effect, revision_strip
from pg_orgs import requests_table, sparkline, state, BAD_PROBES, OK_PROBES

# (group, key, name, role in plain words, section, URL, tone, p50, approval kind, what it would enable when not set)
SERVICES = [
    ("Identity", "pixm", "PIXm resolver", "finds each node's EHR for a patient", "[pixm]", "https://pix.example.org/fhir/", "ok", 14, "identity-services", None),
    ("Identity", "pdqm", "PDQm demographics", "finds the master identity first", "[pdqm]", "https://pdq.example.org/fhir/", "ok", 22, "identity-services", None),
    ("Identity", "pmir", "PMIR identity feed", "drops stale bindings on a merge", "[pmir]", "https://pmir.example.org/fhir", "ok", 26, "identity-services", None),
    ("Localization", "nvi", "NVI", "finds which nodes hold the patient", "[nl_gf.nvi]", "https://nvi.example.org/fhir", "ok", 31, "localization", None),
    ("Localization", "xcpd", "XCPD", "", "[xcpd]", None, "unset", None, "localization", "would localize patients in an IHE community"),
    ("Consent", "mitz", "Mitz", "asks consent before a node is asked", "[nl_gf.mitz]", "https://mitz.example.org/geslotenautorisatievraag", "warn", 1000, "consent", None),
    ("Records and terminology", "arr", "Audit record repository", "keeps the ATNA audit records", "[audit] · [xcpd.audit_repository]", "tls://arr.example.org:6514", "ok", 9, None, None),
    ("Records and terminology", "term", "Terminology server", "translates codes on the FHIR face", "[eehrxf.terminology]", "https://term.demo-regio.example.org/fhir", "neutral", None, None, None),
    ("Directory", "mcsd", "Care services directory", "", "[registry.mcsd]", None, "unset", None, None, "would read organisations and nodes from an mCSD directory"),
]
HREF = {"mitz": "service.html"}
GROUPS = ["Identity", "Localization", "Consent", "Records and terminology", "Directory"]
SETUP_LABEL = {"xcpd": "Set up XCPD", "mcsd": "Set up the directory"}


def host(url):
    return url.split("//", 1)[1].split("/", 1)[0] if url else None


def svc_menu(name, open_=False, probe_hover=False):
    trigger = (f'<button type="button" class="fui-icon-button" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="Actions for {e(name)}" title="More actions">{icon("ellipsis", "16")}</button>')
    m = menu([menu_item("Probe now", "activity", hover=probe_hover), menu_item("Open", "arrow-right", href="service.html", keys="Enter"),
              menu_item("Edit the connection", "pencil", href="service-connection-edit.html")], label=f"Actions for {name}", sheet=True) if open_ else ""
    return f'<span class="fui-anchor">{trigger}{m}</span>'


def svc_rows(only_attention=False, menu_open=None):
    order = {"bad": 0, "warn": 1, "neutral": 2, "ok": 3, "unset": 4}
    out = []
    for g in GROUPS:
        items = [x for x in SERVICES if x[0] == g]
        if only_attention:
            items = [x for x in items if x[6] in ("bad", "warn", "neutral")]
            if not items:
                continue
        items.sort(key=lambda x: order[x[6]])
        policy = ""
        if g == "Localization" and not only_attention:
            policy = f'<span class="mk-group-policy">Policy: fail closed after 5 s {link_button("ghost", "Edit", "services-localization-edit.html", "pencil", small=True)}</span>'
        out.append(f'<tr class="mk-group-row"><th scope="rowgroup" colspan="5"><span class="mk-group-name">{g}</span>{policy}</th></tr>')
        for gg, k, n, role, sec, url, t, p, appr, enable in items:
            if t == "unset":
                out.append(f'<tr class="is-unset"><th scope="row" data-role="title"><span class="mk-svc-name">{e(n)}</span><span class="mk-svc-role">{e(enable)}</span></th>'
                           + td("Host", '<span class="mk-dash">–</span>', role="wide") + td("Health", '<span class="fui-small fui-muted">Not set up</span>', role="meta-2")
                           + td("Last probe", "", role="wide") + td("", link_button("secondary", SETUP_LABEL[k], "service-new.html", "plus", small=True), "fui-row-menu", role="menu") + "</tr>")
                continue
            tip = {"warn": "degraded · 503 Service Unavailable", "neutral": "not probed yet: asked only when the FHIR face writes a summary", "ok": f"reachable · p50 {p} ms"}[t]
            lat = f' · {p:,} ms' if t == "warn" and p else (f" · {p} ms" if p else "")
            health = f'<span title="{e(tip)}">{state(t)}<span class="fui-small fui-muted">{lat if t != "warn" else ""}</span></span>'
            import tpl
            last = tpl.rel("09:41:52") if t != "neutral" else '<span class="mk-dash">never</span>'
            out.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link mk-svc-name" href="{HREF.get(k, "service.html")}">{e(n)}</a><span class="mk-svc-role">{e(role)}</span></th>'
                       + td("Host", f'<code title="{e(url)}">{e(host(url))}</code>', role="sub") + td("Health", health, role="meta-2")
                       + td("Last probe", last, role="meta-q") + td("", svc_menu(n, menu_open == k), "fui-row-menu", role="menu") + "</tr>")
    return out


def summary_chips(active=None):
    chips = [("all", "9 services", None), ("warn", "1 degraded", "warn"), ("neutral", "1 not probed", "neutral"), ("unset", "2 not set up", None), ("attention", "Needs attention · 2", None)]
    out = ""
    for k, w, t in chips:
        on = (active or "all") == k
        dot = health_dot(t) if t else ""
        out += f'<button type="button" class="fui-chip-check" aria-pressed="{"true" if on else "false"}">{dot}{w}</button>'
    return f'<div class="fui-chip-checks mk-svc-summary" role="group" aria-label="Show">{out}</div>'


def services_page(toast_html="", overlay=None, loc_edit=False, attention=False, menu_open=None, dock_html=""):
    rows = svc_rows(attention, menu_open)
    if loc_edit:
        rows = [r.replace('<span class="mk-group-policy">Policy: fail closed after 5 s', '<span class="mk-group-policy mk-group-policy-edit">'
                          + field("lo-fail", "On a localization failure", select("lo-fail", [("closed", "Fail closed (§14.1)"), ("open", "Ask every node")], "closed"))
                          + field("lo-timeout", "Budget", '<div class="fui-input-group">' + input_("lo-timeout", "4000", extra='inputmode="numeric" class="fui-input fui-config-changed"') + '<span class="fui-input-suffix">ms</span></div>', help_="was 5000 ms")
                          + '<span hidden>') if "mk-group-policy" in r else r for r in rows]
    import tpl
    t = tpl.card(table("Services", th("Service") + th("Host") + th("Health") + th("Last probe") + th('<span class="fui-sr-only">Actions</span>'), rows, scroll_label="Services"), label="Services", count="7 set up, 2 not set up" if not attention else "2 need attention").replace("fui-list-card", "fui-list-card mk-svc-list")
    head = page_heading("Services", actions=link_button("primary", "Set up a service", "services-setup.html", "plus"),
                        subtitle="The outside services the gateway asks for identity, localization, consent, audit and terminology.")
    content = head + tpl.list_body("", "", tpl.active(["Needs attention"]) if attention else "", t)
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
        f'<span class="fui-choice-card-text"><span class="fui-choice-card-title">{n}</span><span class="fui-choice-card-sub">{e(enable)} · {e(g)}</span></span>'
        f'<span class="fui-choice-card-check" aria-hidden="true">{icon("check", "12")}</span></label>'
        for g, k, n, role, sec, url, t, p, appr, enable in SERVICES if t == "unset")
    dialog = ('<div role="dialog" aria-modal="true" aria-labelledby="su-title" class="fui-dialog fui-modal mk-open" open>'
              f'<h2 id="su-title" class="fui-dialog-title">{icon("plug", "18")} Set up a service</h2>'
              '<p>Only the services this gateway does not ask yet are offered.</p>'
              f'<fieldset class="fui-choice-cards mk-ff-grants"><legend class="fui-sr-only">Service</legend>{opts}</fieldset>'
              f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{link_button("primary", "Continue", "service-new.html", "arrow-right")}</div></div>')
    return services_page(overlay=dialog)


def page_services_localization_edit():
    d = dock(dock_field("lo-reason", "Reason", "The pilot's queries need an answer sooner; localize within 4 s", "Why this changes", keys="r"),
             link_button("secondary", "Discard the draft", "services.html", "x") + link_button("primary", "Review the changes", "config-review.html", "git-compare"))
    return services_page(loc_edit=True, dock_html=d)


TABS = [("overview", "Overview", "service.html"), ("connection", "Connection", "service-connection.html"), ("requests", "Requests", "service-requests.html"), ("audit", "Audit", "service-audit.html")]


def service_page(tab, panel, dock_html="", top="", overlay=None):
    ag = action_group(link_button("primary", "Edit the connection", "service-connection-edit.html", "pencil"), [menu_item("Probe now", "activity")], label="More actions for Mitz")
    head = page_heading("Mitz", actions=ag, badge=state("warn", "503"),
                        subtitle="Consent · <code>[nl_gf.mitz]</code> · the Dutch Generic Functions' consent pre-filter, asked before a query reaches a node. Changes need a second operator's approval (consent).",
                        crumbs=[("Services", "services.html")])
    content = head + f'<div class="fui-stack">{top}{tabbed(tabs("Mitz", TABS, tab), panel)}</div>'
    return shell("Mitz", "services", content, dock_html=dock_html, overlay=overlay)


def page_service():
    hc = card("Health", f'<p class="fui-cluster">{state("warn", "503 since 09:38")}<span class="fui-small fui-muted">last probe {when("2026-10-21T09:41:52", seconds=True)} · every 60 s</span></p>'
              + sparkline(BAD_PROBES, tuple(range(13, 30))) + '<p class="fui-spark-legend"><span>30 min ago</span><span>dashed: half the per-node budget</span><span>now</span></p>'
              + '<p class="fui-small">While Mitz answers 503 the pre-filter cannot ask, so each query names it <strong>not-asked</strong> and goes on, as §13 lets a pre-filter that did not ask say so.</p>')
    errs = card("Recent errors", '<ul class="mk-plain-list fui-small"><li><time class="fui-time">09:41:52</time> 503 Service Unavailable · geslotenautorisatievraag</li>'
                '<li><time class="fui-time">09:40:52</time> 503 Service Unavailable · probe</li><li><time class="fui-time">09:38:04</time> TLS handshake slow: 1,840 ms</li></ul>')
    mapped = card("Who it serves", definitions([("Nodes mapped", '5 of 6 · <a href="node-settings.html">node-a-lab is not mapped</a>'), ("Policy in force", 'exclusions disclosed (<a href="configuration.html">Configuration</a>) · in an emergency it applies (<a href="service-connection.html">Connection</a>)')]))
    panel = f'<div class="fui-stack"><div class="fui-grid fui-grid-start">{hc}{errs}</div>{mapped}</div>'
    top = notice("warn", "triangle-alert", "<strong>Degraded.</strong> Mitz answers 503 since 09:38. Queries go on with consent not asked; the access record says so for each one.")
    return service_page("overview", panel, top=top)


def connection_body(edit=False):
    from pg_config import secret_field
    if not edit:
        d = definitions([("URL", "<code class=\"fui-break\">https://mitz.example.org/geslotenautorisatievraag</code>"), ("Purpose of use", "TREAT, treatment"),
                         ("Data categories", "<code>GGC002</code>"), ("Namespaces that stand for the BSN", "<code>urn:oid:2.999.1</code>"), ("Timeout", "1,000 ms, one round of questions"),
                         ("In an emergency", "apply the pre-filter (<code>federation.consent.emergency</code>)"),
                         ("Mutual TLS", f'<span class="fui-secret-now">{icon("lock", "12")}set, read from <code>/run/secrets/ferrofed/mitz-client.pem</code></span>'),
                         ("Password", f'<span class="fui-secret-now">{icon("lock", "12")}set, read from <code>/run/secrets/ferrofed/mitz-password</code></span>'),
                         ("Approval", "consent: a second operator approves a change, which then runs on trial for 2 minutes")])
        d = d.replace('<dl class="fui-facts">', '<dl class="fui-facts">', 1)
        return '<div class="fui-stack">' + card("Connection", d + '<p class="fui-small fui-muted">Configured in <code>[nl_gf.mitz]</code> · changes need a second operator\'s approval (consent), then run on trial for 2 minutes.</p>') + card("Set elsewhere", definitions([("Each node's care provider", '5 of 6 nodes mapped, on each node\'s Settings · <a href="node-settings.html">node-a-lab is not</a>'), ("Consent exclusions", 'disclosed · <a href="configuration.html">Set in Configuration</a>, as it also covers the nodes\' own refusals')])) + "</div>"
    fields = ('<div class="fui-columns">'
              + field("mz-url", "URL", input_("mz-url", "https://mitz.example.org/geslotenautorisatievraag", extra='spellcheck="false" class="fui-input fui-mono"'))
              + field("mz-purpose", "Purpose of use", select("mz-purpose", [("TREAT", "TREAT, treatment"), ("COC", "COC, continuity of care")], "TREAT"))
              + field("mz-timeout", "Timeout", '<div class="fui-input-group">' + input_("mz-timeout", "1500", extra='inputmode="numeric" class="fui-input fui-config-changed"') + '<span class="fui-input-suffix">ms</span></div>', help_="was 1000 ms · one round of questions, within the query's budget")
              + field("mz-cats", "Data categories", input_("mz-cats", "GGC002", extra='spellcheck="false" class="fui-input fui-mono"'), help_="The Mitz categories asked about, comma-separated.")
              + field("mz-emerg", "In an emergency", select("mz-emerg", [("apply", "Apply the pre-filter"), ("pass", "Pass to the node")], "apply"), help_="federation.consent.emergency")
              + "</div>")
    secrets = secret_field("client_identity", "keep", now="/run/secrets/ferrofed/mitz-client.pem", idx=20) + secret_field("credentials.password", "keep", now="/run/secrets/ferrofed/mitz-password", idx=21)
    return ('<div class="fui-stack">' + revision_strip("draft", "Draft of [nl_gf.mitz] · 1 change", "Approved by a second operator (consent), then on trial for 2 minutes after the apply.")
            + card("Connection", fields) + card("Mutual TLS and credentials", secrets) + "</div>")


def page_service_connection():
    return service_page("connection", connection_body())


def page_service_connection_edit():
    d = dock(dock_field("mz-reason", "Reason", "Mitz answers slower in the pilot; give it 1.5 s", "Why this changes", keys="r"),
             link_button("secondary", "Discard the draft", "service-connection.html", "x") + link_button("primary", "Review the changes", "config-review.html", "git-compare"))
    return service_page("connection", connection_body(edit=True), dock_html=d)


def page_service_requests():
    rows = [("09:41:52", "mitz", "geslotenautorisatievraag, 3 nodes", "503", "1,000 ms", "not-asked", "01JAF3M0R2KD"),
            ("09:37:10", "mitz", "geslotenautorisatievraag, 4 nodes", "200", "212 ms", "2 permitted, 2 denied", "01JAF3KG7VNA"),
            ("09:31:44", "mitz", "geslotenautorisatievraag, 2 nodes", "200", "188 ms", "2 permitted", "01JAF3K1T9PE")]
    panel = ('<div class="fui-stack"><p class="fui-small fui-muted">The questions the gateway asked Mitz: never a patient identifier or a value, as the access record keeps them.</p>'
             + requests_table(rows).replace("<th scope=\"col\">Endpoint</th>", "<th scope=\"col\">Service</th>") + "</div>")
    return service_page("requests", panel)


def page_service_audit():
    rows = [("2026-10-08T13:15", "Demo Operator", "revision 2", "turned on the Mitz pre-filter for the pilot", "Turn on the Mitz pre-filter for the pilot"),
            ("2026-10-08T13:22", "Demo Reviewer", "approval", "approved revision 2 (consent)", None),
            ("2026-10-08T13:25", "Demo Operator", "confirmed", "confirmed revision 2 after its trial", None)]
    body = [f'<tr><th scope="row">{when(t)}</th>' + td("By", e(by), "mk-nowrap") + td("Kind", badge("neutral", k)) + td("What", e(w)) + td("Reason", e(r) if r else '<span class="mk-dash">–</span>') + "</tr>" for t, by, k, w, r in rows]
    return service_page("audit", table("Audit", th("When", "t", "descending") + th("By") + th("Kind") + th("What") + th("Reason"), body))


def page_service_new():
    from pg_config import secret_field
    fields = ('<div class="fui-columns">'
              + field("xc-sender", "The gateway's device OID", input_("xc-sender", "2.999.40.1", extra='class="fui-input fui-mono"'))
              + field("xc-home", "Home community", input_("xc-home", "2.999.40", extra='class="fui-input fui-mono"'))
              + field("xc-url", "Responding gateway", input_("xc-url", "https://xcpd.region.example.org/RespondingGateway", extra='spellcheck="false" class="fui-input fui-mono"'))
              + field("xc-device", "Its device OID", input_("xc-device", "2.999.50.1", extra='class="fui-input fui-mono"'))
              + field("xc-audit", "Audit", select("xc-audit", [("log", "log, to the gateway's log"), ("repository", "repository, to the audit record repository")], "repository"))
              + "</div>")
    body = (revision_strip("draft", "Draft based on revision 7 · XCPD set up", "Approved by a second operator (localization). Each node's community is set on the node once XCPD runs.")
            + card("Service", field("xc-kind", "Service", select("xc-kind", [("xcpd", "XCPD localization"), ("mcsd", "Care services directory")], "xcpd"), help_="Only the services not set yet are offered."))
            + card("Connection", fields) + card("Mutual TLS", secret_field("client_identity", "file", now="", now_state="unset", path="/run/secrets/ferrofed/xcpd-client.pem", idx=30)))
    content = page_heading("Set up a service", subtitle="A service the gateway does not ask yet. Saved as a revision you review, have approved and apply.", crumbs=[("Services", "services.html")]) + f'<div class="fui-stack">{body}</div>'
    d = dock(dock_field("xc-reason", "Reason", "The region's XCPD community goes live on 1 November", "Why this changes", keys="r"),
             link_button("secondary", "Cancel", "services.html", "x") + link_button("primary", "Review the changes", "config-review.html", "git-compare"))
    return shell("Set up a service", "services", content, dock_html=d)
