# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Configuration, as the book describes it, in its two states:

- `config.ui = "managed"`: revisions, History, rollback, and Import and
  export; each section works file-first or console-first. Every page here
  draws the one revision fixture of data.py (1 to 7 before today, 8 to 10
  today).
- `config.ui = "off"`: a read-only view of ferrofed.toml, with no History
  tab and no revisions.

The core (profile, server, metrics, auth, trust, config, dev) always stays
in ferrofed.toml. Credentials of endpoints live on the endpoint's row."""
from common import (
    MENU_SEP, action_group, badge, busy_button, button, card, definitions, dock, dock_field, e, field, hint,
    icon, icon_button, input_, link_button, menu_item, notice, page_heading, panel_head, pill, quiet_line,
    refusal, rel, section, sections, select, shell, table, td, textarea, th, toast, toasts, well, when,
)
import data
from data import REASON_8, REASON_9, REASON_10, REVISIONS, FUTURE
from parts import D, change_del, change_ins, date_field_open, effect, month_grid, mode_badge, picker, revision_strip, PICK_TODAY
import routes
import tpl

CORE_KEYS = ["profile", "server", "metrics", "auth", "trust", "config", "dev"]
CORE = ('<ul class="fui-bullets fui-small">' + "".join(f"<li><code>{k}</code></li>" for k in CORE_KEYS) + "</ul>"
        '<p class="fui-small fui-muted">Who may call, the listeners, the admin listener this page talks to, and the security floor the profile sets.</p>')
REST = ('<ul class="fui-bullets fui-small"><li>Organisations, their nodes and endpoints, on their own pages</li>'
        '<li>Endpoint credentials, on each endpoint</li>'
        '<li>Services, each on its page</li>'
        '<li>The deployment-wide settings, on this page</li></ul>')

TABS = [("federation", "Federation behaviour", "configuration.html"), ("fanout", "Fan-out and paging", "configuration-fanout.html"),
        ("consent", "Consent", "configuration-consent.html"), ("stored", "Stored queries", "configuration-stored.html"),
        ("access", "Access log", "configuration-access.html"), ("fhir", "FHIR face", "configuration-fhir.html"),
        ("auth", "Authentication", "configuration-auth.html"),
        ("history", "History", "configuration-history.html"), ("io", "Import and export", "configuration-import.html")]
OFF_TABS = [("federation", "Federation behaviour", "configuration-file-mode.html"), ("fanout", "Fan-out and paging", "r-config-off-fanout.html"),
            ("consent", "Consent", "configuration-file-mode-history.html"), ("stored", "Stored queries", "r-config-off-stored.html"),
            ("access", "Access log", "r-config-off-access.html"), ("fhir", "FHIR face", "r-config-off-fhir.html"),
            ("auth", "Authentication", "configuration-file-mode-auth.html")]
# The tabs of the core, which only ferrofed.toml sets: read-only in both states, with no Edit.
READ_ONLY = ("auth",)


# ---- the mode dialog ----

def mode_dialog(mode):
    if mode == "managed":
        lead = 'Revisions are <strong>managed</strong> on this gateway (<code>config.ui = "managed"</code>).'
        here = ('<h3>' + icon("history", "16") + ' Everything else: a revision</h3><p class="fui-small fui-muted">Each change is a draft you review against what runs, saved as a revision with a reason and applied; '
                'every replica follows within 5 seconds. A credential or a binding section runs on trial until you confirm it, and the kinds your deployment names take a second operator\'s approval.</p>')
        ways = ('<p class="fui-small fui-muted mk-mode-other"><strong>File-first or console-first, per section.</strong> A file-first section lives in ferrofed.toml and in version control; each reload records it as a revision, and it is read-only here. '
                'A console-first section is changed here; export a revision to keep it in version control. This gateway is console-first, except the credentials of node-b-pub, which ferrofed.toml sets.</p>'
                '<p class="fui-small fui-muted mk-mode-other">To turn revisions off, set <code>config.ui = "off"</code> and restart: the gateway then reads everything from ferrofed.toml.</p>')
    else:
        lead = 'Revisions are <strong>off</strong> on this gateway (<code>config.ui = "off"</code>): everything is read from ferrofed.toml.'
        here = ('<h3>' + icon("file-text", "16") + ' Everything else: also in ferrofed.toml</h3><p class="fui-small fui-muted">Changed in the file and applied by a reload or, for a restart key, a restart; '
                'shown here read-only. There is no history and no rollback on this page.</p>')
        ways = ('<p class="fui-small fui-muted mk-mode-other">To manage revisions, set <code>config.ui = "managed"</code> and restart. Your file stays as it is: each section stays file-first until you move it to the console by export and import.</p>')
    return (
        '<div id="mode-dialog" role="dialog" aria-modal="true" aria-labelledby="mode-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="mode-title" class="fui-dialog-title">{icon("info", "18")} How configuration works</h2>'
        f'<p>{lead} The core always stays in the file.</p>'
        '<div class="fui-modal-body"><div class="fui-ownership">'
        '<div><h3>' + icon("lock", "16") + ' The core: always in ferrofed.toml</h3><p class="fui-small fui-muted">Never in a revision, so no screen can lock you out of the admin listener or lower the security floor the file sets.</p>' + CORE + '</div>'
        '<div>' + here + REST + '</div></div>'
        '<p class="fui-small fui-muted mk-mode-other">Never configuration: an organisation or node a care services directory provides, the gateway\'s own <code>eu.ferrofed.eehrxf</code> queries, and a stored-query version. They show a lock.</p>'
        + ways + '</div>'
        f'<div class="fui-dialog-actions">{button("secondary", "Close", "x", keys="Esc", attrs="autofocus")}</div></div>'
    )


def mode_help(mode):
    href = "configuration-mode-dialog.html" if mode == "managed" else "configuration-file-mode-dialog.html"
    return link_button("ghost", "How configuration works", href, "info", attrs='aria-haspopup="dialog"')


def config_heading(mode="managed", menu_open=False, running="7", loading=False):
    if mode == "managed":
        if loading:
            ag = f'<div class="fui-action-group">{mode_help("managed")}<button type="button" class="fui-button fui-button-secondary fui-more" disabled aria-label="More configuration actions">More{icon("chevron-down", "16")}</button></div>'
        else:
            ag = action_group(mode_help("managed"), [menu_item(f"Export revision {running} as TOML", "download"), menu_item("Import TOML…", "upload", href="configuration-import.html")],
                              open_=menu_open, label="More configuration actions")
        sub = "The deployment-wide settings; everything else is set on its own page."
    else:
        ag = action_group(mode_help("off"), [], label="More configuration actions")
        sub = "What ferrofed.toml sets, as the gateway read it."
    return page_heading("Configuration", actions=ag, subtitle=sub, badge=mode_badge(mode))


# ---- the revision line (T3): one line; an event replaces it with a strip ----

LINES = {
    "7": ("2026-10-19", "Demo Operator"), "8": ("today 09:58", "you"), "9": ("today 10:01", "you"), "10": ("today 10:13", "you, confirmed"),
}


def compact_strip(n="7"):
    at, by = LINES[n]
    return (f'<div class="fui-revision fui-revision-compact">{icon("file-text", "16")}<p><strong>Revision {n}</strong> · applied {at} by {by} · '
            f'<span class="fui-pill fui-tone-ok" title="gateway-0 and gateway-1 run revision {n}">In step</span></p></div>')


def replicas_inline(g1=("ok", "In step")):
    return f'<ul class="fui-replicas"><li><code>gateway-0</code>{pill("ok", "In step")}</li><li><code>gateway-1</code>{pill(*g1)}</li></ul>'


# ---- the settings tabs ----

def key_hint(k, show=False):
    return f'<button type="button" class="fui-key-hint{" is-shown" if show else ""}" aria-label="Copy the key {e(k)}" title="Copy the key"><code>{e(k)}</code>{icon("copy", "12")}</button>'


def srow(label, desc, value, key=None, restart=False, control=None, edit=False, error=None, changed=None, hint_=False, pending=None):
    nxt = f'<span class="fui-effect fui-effect-restart" title="Takes effect at the next start of each replica">{icon("refresh-cw", "12")}Next start</span>' if restart else ""
    if edit and control:
        ctl = control
    elif pending:
        ctl = f'<span class="fui-setting-value">{value} <span class="fui-small fui-muted">running now · {pending} at the next start</span></span>'
        nxt = ""
    else:
        ctl = f'<span class="fui-setting-value">{value}</span>'
    er = f'<p class="fui-help fui-error mk-row-error">{icon("circle-alert", "14")} {error}</p>' if error else ""
    was = f'<span class="fui-config-was">was {e(changed)}</span>' if changed else ""
    kh = key_hint(key, hint_) if key else ""
    return (f'<div class="fui-setting-row{" is-changed" if changed else ""}{" is-invalid" if error else ""}{" mk-hover" if hint_ else ""}"><div class="fui-setting-text"><span class="fui-setting-label">{label}{kh}</span>'
            f'<p class="fui-setting-desc">{desc}</p>{er}</div><div class="fui-setting-control">{ctl}{nxt}{was}</div></div>')


def dur(id_, v, unit, changed=False, invalid=False):
    cls = "fui-input" + (" fui-config-changed" if changed else "")
    inv = ' aria-invalid="true"' if invalid else ""
    return f'<div class="fui-input-group mk-dur"><input id="{id_}" class="{cls}" value="{v}" inputmode="numeric"{inv} /><span class="fui-input-suffix">{unit}</span></div>'


def sw(id_, on, label):
    return f'<input type="checkbox" class="fui-switch" id="{id_}"{" checked" if on else ""} aria-label="{e(label)}" />'


RESTART_ALL = "Every row takes effect at the next start of each replica."
LIVE = "A change applies at once."


def tab_sections(edit=False, off=False, hint_=False, pending=None, invalid=True):
    """Each tab's (intro, rows). edit: Federation behaviour in its draft;
    pending: the values a restart applies."""
    fe = edit
    pending = pending or {}
    per_desc = "How long one node may take: a positive number of milliseconds, no more than the overall timeout." + (" Default 10,000 ms." if off else "")
    ov_desc = "How long a whole query may take. With the 1,000 ms margin it stays below each replica's own request timeout." + (" Default 25,000 ms." if off else "")
    fed = (srow("Per-node timeout", per_desc, "2,500 ms", "federation.per_node_timeout_ms", restart=True,
                control=dur("cfg-pn", "3000", "ms", changed=True), edit=fe, changed="2,500 ms" if fe else None, hint_=hint_ and not fe, pending=pending.get("pn"))
           + srow("Overall timeout", ov_desc, "4,000 ms", "federation.overall_timeout_ms", restart=True,
                  control=dur("cfg-ov", "9500" if invalid else "5000", "ms", changed=True, invalid=invalid), edit=fe, changed="4,000 ms" if fe else None,
                  error="With the 1,000 ms margin it must stay below gateway-0's request timeout of 10,000 ms: at most 8,999 ms." if fe and invalid else None, pending=pending.get("ov"))
           + srow("A late node", "A node past its budget is named time-out and left out of the answer. This is fixed.", "Abandon and mark")
           + srow("Node selection", "Which nodes a patient's query asks.", "Localized: the nodes the localizer names", "federation.node_selection", restart=True,
                  control=select("cfg-ns", [("localized", "Localized"), ("ask-all", "Ask every node")], "localized", cls="fui-select"), edit=fe)
           + srow("Best effort", "A client may ask for the rows that came back when a node does not answer.", "Offered", "federation.best_effort", restart=True,
                  control=sw("cfg-be", True, "Best effort"), edit=fe)
           + srow("When the localizer fails", 'The query fails and says why. Set on <a href="services.html">Services</a>, under Localization.', "Ask no node · after 1,500 ms"))
    fan = (srow("Stored-query fan-out", "Send each new stored version on to the nodes as well.", "Off", "federation.fan_out_stored_queries", restart=True)
           + srow("Template upload fan-out", "Send an uploaded template to several nodes, not one chosen node.", "Off", "federation.fan_out_template_upload", restart=True)
           + srow("Paging", "Each node is asked for offset plus limit rows; the gateway merges, sorts and cuts the page, within the window.", "Per node, merged", "federation.offset_strategy", restart=True)
           + srow("Paging window", "The most rows, offset plus limit, one page may ask of a node; a positive number.", "1,000 rows", "federation.max_offset_window", restart=True)
           + srow("Rows per node", "The most rows read of one node for a query without LIMIT; a positive number.", "9,999 rows", "federation.max_node_rows", restart=True))
    con = (srow("Disclose exclusions", "Name a node the consent pre-filter excludes as consent-denied. Off, it reads as a node without the patient, as Regulation (EU) 2025/327 Art 8 may require.", "On", "federation.consent.disclose")
           + srow("In an emergency", "Whether the consent pre-filter still applies when the caller declares an emergency purpose.", "Apply the pre-filter", "federation.consent.emergency"))
    sto = (srow("Backend", "Where the stored queries are kept.", "PostgreSQL, shared by every replica", "stored_queries.backend", restart=True)
           + srow("Connection", "A secret, by reference; replace it in Edit.", f'<span class="fui-secret-now mk-nowrap">{icon("lock", "12")}Set · <code>/run/secrets/ferrofed/stored-queries-url</code></span>', "stored_queries.url_file", restart=True))
    acc = (srow("Retention", "Years each access record is kept: at least 3.", "3 years", "access_log.retention.years")
           + srow("Emergency purposes", "Purposes of use that mark an emergency access.", '<span class="fui-tag">ETREAT</span>', "access_log.emergency_purpose")
           + srow("Category map", "Template ids, and archetype ids under access_log.archetypes, each mapped to the categories of the data under it.", "42 ids mapped", "access_log.templates"))
    fh = (srow("Base", "Where the exchange-format face is served, beside the openEHR API.", "<code>/fhir</code>", "fhir.base", restart=True)
          + srow("Document URL lifetime", "How long a listed document's URL stays valid: at least 1 s.", "300 s", "fhir.document_url_lifetime_secs", restart=True)
          + srow("Received categories", "The documents the face accepts.", '<span class="fui-tag">Patient summaries</span> <span class="fui-tag">Laboratory reports</span>', "fhir.receive", restart=True))
    au = (srow("Audience", "The audiences a caller's token must name for this gateway.", "<code>https://gateway.demo-regio.example.org</code>", "auth.audience")
          + srow("Operator scope", "The scope a token needs for the operator surface and the admin actions.", "<code>__ferrofed.operator</code>", "auth.product_scope")
          + srow("Token issuer", "An issuer whose access tokens the gateway verifies.", '<code class="fui-break">https://idp.demo-regio.example.org/realms/ferrofed</code>', "auth.issuer[].issuer")
          + srow("Key set file", "The file this issuer's signing keys are read from, when they are not fetched.", '<code class="fui-break">/etc/ferrofed/keys/demo-regio.jwks</code>', "auth.issuer[].jwks_file")
          + srow("Trusted issuer", "An issuer whose callers the gateway admits, with the policy below it.", '<code class="fui-break">https://ncp.demo-land.example.org/realms/ncp</code>', "trust.issuer[].issuer")
          + srow("Introspection secret", "The secret the gateway authenticates to this issuer's introspection endpoint with, named by the file it is read from.",
                 f'<span class="fui-secret-now">{icon("lock", "12")}Set · <code class="fui-break">/run/secrets/ferrofed/ncp-introspection</code></span>', "trust.issuer[].client_secret")
          + srow("Family name claim", "The claim carrying the professional's family name. Set, it declares the issuer a national contact point.", "<code>family_name</code>", "trust.issuer[].national_contact_point.family_name")
          + srow("Given name claim", "The claim carrying the professional's given name.", "<code>given_name</code>", "trust.issuer[].national_contact_point.given_name")
          + srow("Country code claim", "The claim carrying the country that issued the professional's identification data.", "<code>issuing_country</code>", "trust.issuer[].national_contact_point.country_code")
          + srow("Professional's issuing authority claim", "The claim carrying the authority that issued the professional's identifier.", "<code>professional_authority</code>", "trust.issuer[].national_contact_point.professional_issuing_authority")
          + srow("Provider's issuing authority claim", "The claim carrying the authority that issued the provider's identifier.", "<code>provider_authority</code>", "trust.issuer[].national_contact_point.provider_issuing_authority")
          + srow("Provider address claim", "The claim carrying the healthcare provider's address.", "<code>provider_address</code>", "trust.issuer[].national_contact_point.provider_address")
          + srow("Correlation header", "The request header the contact point's connector sends its correlation identifier in, recorded with the access.", "<code>X-Correlation-Id</code>", "trust.issuer[].national_contact_point.correlation_header")
          + srow("Added countries", "The countries beside the 27 Member States a relayed country code may name.", '<span class="fui-tag">IS</span> <span class="fui-tag">NO</span>', "trust.issuer[].national_contact_point.additional_countries"))
    if off:
        con = con.replace("</span></div></div>", f'</span><span class="fui-effect fui-effect-restart">{icon("refresh-cw", "12")}Next reload</span></div></div>')
    return {
        "federation": ("How a query fans out to the nodes and when it gives up. " + RESTART_ALL, fed),
        "fanout": ("How a query is paged, and what is sent on to the nodes. " + RESTART_ALL, fan),
        "consent": ("What callers learn about consent. " + ("A change takes a second operator's approval and applies at once." if not off else "Read again at each reload."), con),
        "stored": ('The store the stored queries are kept in; the queries themselves are on <a href="stored-queries.html">Stored queries</a>. ' + RESTART_ALL, sto),
        "access": ("How each access record is classified and kept. " + (LIVE if not off else "Read again at each reload."), acc),
        "fhir": ("The face that serves and receives the European exchange format. " + RESTART_ALL, fh),
        "auth": ("Who may call: the issuers the gateway trusts, a national contact point among them. Only ferrofed.toml sets these, so this tab is read-only.", au),
    }


def config_page(tab, mode="managed", top="", overlay=None, toast_html="", menu_open=False, dock_html="", running="7", body=None, edit=False,
                drafts=(), status_html=None, hint_=False, pending=None, invalid=True):
    off = mode == "off"
    status = status_html if status_html is not None else (compact_strip(running) if not off else
                                                          f'<p class="fui-small fui-muted mk-file-line">{icon("lock", "12")} Revisions are off on this gateway (<code>config.ui = "off"</code>). Set in <code>/etc/ferrofed/ferrofed.toml</code> · read at start · <a href="configuration-file-mode-dialog.html">How configuration works</a></p>')
    if body is not None:
        inner = body
    else:
        intro, rows = tab_sections(edit=edit, off=off, hint_=hint_, pending=pending, invalid=invalid)[tab]
        act = "" if off or tab in READ_ONLY else link_button("secondary", "Edit", "config-settings-edit.html" if tab == "federation" else routes.href("config", tab, "edit"), "pencil", small=True, keys="e")
        inner = tpl.settings_panel(rows, intro, act)
    tabs_html = tpl.settings_tabs("Configuration sections", OFF_TABS if off else TABS, tab, inner, drafts=drafts, many=True)
    content = config_heading(mode, menu_open, running) + f'<div class="fui-stack">{status}{top}{tabs_html}</div>'
    return shell("Configuration", "configuration", content, overlay=overlay, toast_html=toast_html, dock_html=dock_html)


def page_config_settings():
    return config_page("federation")


def page_configuration_key_hint():
    return config_page("federation", hint_=True)


def _tab_page(key):
    def f():
        return config_page(key)
    return f


page_configuration_fanout = _tab_page("fanout")
page_configuration_consent = _tab_page("consent")
page_configuration_stored = _tab_page("stored")
page_configuration_access = _tab_page("access")
page_configuration_fhir = _tab_page("fhir")
page_configuration_auth = _tab_page("auth")


def page_configuration_mode_dialog():
    return config_page("federation", overlay=mode_dialog("managed"))


def page_configuration_actions():
    return config_page("federation", menu_open=True)


def page_configuration_loading():
    from parts import skeleton_lines
    rows = "".join(f'<div class="fui-setting-row" aria-hidden="true"><div class="fui-setting-text">{skeleton_lines([30, 60])}</div><div class="fui-setting-control">{skeleton_lines([50])}</div></div>' for _ in range(6))
    body = tpl.settings_panel(rows, '<span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Reading the revisions from gateway-0</span>')
    strip = f'<div class="fui-revision fui-revision-compact" aria-hidden="true">{icon("file-text", "16")}<p>{skeleton_lines([40])}</p></div>'
    html = config_page("federation", body=body, status_html=strip)
    return html.replace(config_heading("managed"), config_heading("managed", loading=True), 1)


def page_configuration_export_done():
    t = toasts(toast("ok", "download", "Export ready", "ferrofed-revision-7.toml · 4 KB · every secret as its file reference; the core is not part of it.",
                     action=button("secondary", "Download", "download", small=True)))
    return config_page("federation", toast_html=t)


# ---- the Federation behaviour draft, its review, and what can happen next ----

def edit_dock(primary_href=None, disabled=False, reason=REASON_9):
    p = (button("primary", "Review changes", "git-compare", attrs='disabled aria-describedby="dock-why"') if disabled
         else link_button("primary", "Review changes", primary_href, "git-compare"))
    msg = '<p id="dock-why" class="fui-small fui-muted">Fix the overall timeout first.</p>' if disabled else ""
    return dock(dock_field("cfg-reason", "Reason", reason, "Required: kept with the revision", keys="r"), link_button("secondary", "Discard", "configuration.html", "x") + p, msg)


def page_config_settings_edit():
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 1 value needs a change before you can review</p>'
               '<ul><li><a href="#cfg-ov">Overall timeout: at most 8,999 ms on gateway-0</a></li></ul></div>')
    intro, rows = tab_sections(edit=True)["federation"]
    body = tpl.settings_panel(rows, "A draft of this tab, based on revision 7: 2 changes, 1 to fix. The other tabs stay as they run.",
                              '', top=summary)
    return config_page("federation", body=body, dock_html=edit_dock(disabled=True), drafts=("federation",))


def diff_table(rows, base="Running", right="Your draft"):
    """rows: (label, key, before, after, effect)."""
    body = [f'<tr><th scope="row">{e(lbl)}{hint(k) if k else ""}</th>' + td(base, change_del(e(a)) if a else '<span class="mk-dash">not set</span>')
            + td(right, change_ins(e(b)) if b else '<span class="mk-dash">removed</span>') + td("Takes effect", effect(fx) if fx in ("live", "restart") else e(fx)) + "</tr>" for lbl, k, a, b, fx in rows]
    return table("What changes", th("Setting") + th(base) + th(right) + th("Takes effect"), body)


SET_DIFF = [("Per-node timeout", "federation.per_node_timeout_ms", "2,500 ms", "3,000 ms", "restart"),
            ("Overall timeout", "federation.overall_timeout_ms", "4,000 ms", "5,000 ms", "restart")]


def review_dock(reason=REASON_9, back="config-settings-edit.html", save_href=None, apply_href="config-restart.html", primary="Save and apply…", approval=False):
    fld = dock_field("cfg-reason", "Reason", reason, "Required: kept with the revision", keys="r")
    if approval:
        acts = link_button("secondary", "Back to editing", back, "pencil") + link_button("primary", "Save for approval", apply_href, "save")
    else:
        acts = link_button("secondary", "Back to editing", back, "pencil") + (link_button("secondary", "Save only", save_href, "save") if save_href else "") + link_button("primary", primary, apply_href, "check")
    return dock(fld, acts)


def page_config_settings_review():
    body = ('<div class="fui-settings-panel"><div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">Your draft against revision 8, which runs: 2 changes. Checked as <code>ferrofed config check</code> would: no problems.</p></div>'
            + '<div class="fui-panel-body fui-stack">' + diff_table(SET_DIFF)
            + '<p class="fui-small">Once each replica restarts, every query gives each node 3 s instead of 2.5 s, within an overall 5 s: answers that took 2.8 s, which timed out, then arrive.</p></div></div>')
    return config_page("federation", body=body, dock_html=review_dock(save_href="config-pending.html"), drafts=("federation",), running="8")


def pending_card(title, line, steps, actions="", tone="", glyph="pencil", body=""):
    t = f" fui-tone-{tone}" if tone else ""
    role = ' role="status"' if tone else ""
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    return (f'<section class="fui-pending{t}"{role} aria-label="{e(title)}"><div class="fui-pending-head">{icon(glyph, "18")}'
            f'<div class="fui-revision-text"><strong>{title}</strong><span>{line}</span></div>{a}</div>{lifecycle(steps)}{body}</section>')


def lifecycle(steps):
    lis = ""
    for st, title, line in steps:
        glyph = {"done": icon("check", "12"), "failed": icon("x", "12"), "skipped": icon("minus", "12")}.get(st, "")
        lis += f'<li class="is-{st}"><span class="fui-lifecycle-dot" aria-hidden="true">{glyph}</span><span class="fui-lifecycle-text"><strong>{title}</strong><span>{line}</span></span></li>'
    return f'<ol class="fui-lifecycle" aria-label="Where this revision stands">{lis}</ol>'


def page_config_pending():
    card_ = pending_card("Revision 9 is saved, not applied", f"Saved today 10:00 by you: “{REASON_9}.” It changes 2 settings, both at the next start.",
                         [("done", "Saved", "by you, today 10:00"), ("skipped", "Approval", "not needed for these settings"), ("current", "Apply", "yours, or any operator's"), ("skipped", "Confirm", "not needed")],
                         link_button("secondary", "Review the changes", "config-settings-review.html", "git-compare", small=True) + link_button("primary", "Apply revision 9…", "config-restart.html", "check", small=True))
    return config_page("federation", top=card_, running="8")


def page_config_restart():
    strip = revision_strip("warn", "Revision 9 runs; 2 settings wait for the next start",
                           f"Applied today 10:01 by you. Until each replica restarts, it keeps the per-node timeout at 2,500 ms and the overall at 4,000 ms. {replicas_inline(('warn', 'Pending restart'))}",
                           link_button("ghost", "Roll back…", "config-rollback.html", "rotate-ccw", small=True))
    return config_page("federation", status_html=strip, running="9", pending={"pn": "3,000 ms", "ov": "5,000 ms"})


def page_config_conflict():
    strip = revision_strip("bad", "Revision 8 was applied while you edited",
                           f"You applied it today at 09:58 from the new organisation: “{REASON_8}.” Your draft was based on revision 7, so it was not saved (409{hint('revision-stale')}). "
                           "Revision 8 changes none of your settings, so your draft rebases cleanly.",
                           button("primary", "Rebase my draft on revision 8", "git-compare", small=True) + link_button("secondary", "Discard my draft", "configuration.html", "x", small=True))
    body = [f'<tr><th scope="row">{lbl}{hint(k)}</th>' + td("Revision 7, your base", a) + td("Revision 8, running", b) + td("Your draft", c) + td("", badge("accent", "Yours alone")) + "</tr>"
            for lbl, k, a, b, c in [("Per-node timeout", "federation.per_node_timeout_ms", "2,500 ms", "2,500 ms", "3,000 ms"),
                                    ("Overall timeout", "federation.overall_timeout_ms", "4,000 ms", "4,000 ms", "5,000 ms")]]
    t = (section("Your draft against revision 8", table("Your draft against revision 8", th("Setting") + th("Revision 7, your base") + th("Revision 8, running") + th("Your draft") + th('<span class="fui-sr-only">Whose</span>'), body)
                 + '<p class="fui-small fui-muted">Rebasing keeps both your changes; then review them against revision 8.</p>'))
    return config_page("federation", status_html=strip, top=f'<div class="fui-card fui-panel">{t}</div>', running="8", drafts=("federation",))


def page_config_replica_refused():
    strip = revision_strip("refused", "Revision 9 runs on gateway-0; gateway-1 refused it",
                           "gateway-1's own ferrofed.toml sets its request timeout to 6,000 ms; the overall timeout of 5,000 ms plus the 1,000 ms margin is not below it, so gateway-1 keeps running revision 8. "
                           "Raise its request timeout there and restart it, or lower the overall timeout in a new revision. The alert FerroFEDConfigurationDrift fires after 10 minutes. "
                           + replicas_inline(("bad", "Refused revision 9")))
    return config_page("federation", status_html=strip, running="9", pending={"pn": "3,000 ms", "ov": "5,000 ms"})


# ---- a revision from draft to running: the fixture's revisions 8 and 10 ----

MEMBER_DIFF = [("Organisation Zorgcentrum Beta-Testerveen", "registry.members[org-zbt]", "", "org-zbt", "live"),
               ("Its node node-f", "registry.members[org-zbt].nodes", "", "node-f · cdr-f.example.org", "live"),
               ("node-f's endpoint", "registry.members[org-zbt].endpoints", "", "node-f-pub · openEHR Query API", "live"),
               ("node-f in Mitz", "nl_gf.mitz.holders.node-f", "", "Care provider URA 90000106", "live"),
               ("node-f in PIXm", "pixm.manager[0].members.node-f", "", "urn:oid:2.999.26", "live")]
CHECKED = "Checked as <code>ferrofed config check</code> would: no problems."


def form_page(title, crumbs, sub, body, dock_html, current="configuration", **kw):
    content = page_heading(title, subtitle=sub, crumbs=crumbs) + f'<div class="fui-stack">{body}</div>'
    return shell(title, current, content, dock_html=dock_html, **kw)


def review_body(rows, effects, notes="", against="7"):
    return sections(section("What the draft changes", f'<p class="fui-small">{CHECKED}</p>{notes}' + diff_table(rows, base=f"Revision {against}") + effects, count=f"{len(rows)} settings"))


def page_config_review():
    eff = '<p class="fui-small">What applying does: Zorgcentrum Beta-Testerveen takes part as the sixth organisation, and node-f is probed within a minute.</p>'
    return form_page("Review the new organisation", [("Organisations", "organisations.html"), ("New organisation", "organisation-new.html")],
                     "Your draft against revision 7, which runs; saved and applied, it is revision 8.", review_body(MEMBER_DIFF, eff),
                     review_dock(REASON_8, back="organisation-new.html", apply_href="config-apply.html"), current="orgs")


def apply_dialog(n, lines, reason, back, go, confirm_note="", restore="7"):
    return (
        '<div role="dialog" aria-modal="true" aria-labelledby="apply-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="apply-title" class="fui-dialog-title">{icon("check", "18")} Apply revision {n}?</h2>'
        f'<p>{lines}</p>' + confirm_note
        + well(f'<p class="fui-small"><strong>Reason:</strong> {e(reason)}</p><p class="fui-small fui-muted">Recorded in the security log{hint("ferrofed::security")} with your name. Revision {restore} stays in History and comes back in one step with Roll back.</p>')
        + f'<div class="fui-dialog-actions">{link_button("secondary", "Go back", back, keys="Esc")}{link_button("primary", f"Apply revision {n}", go, "check")}</div></div>'
    )


def overlay(html, dialog):
    return (html.replace('</div></body></html>', f'<div class="mk-scrim" aria-hidden="true"></div>{dialog}</div></body></html>')
            .replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1))


def page_config_apply():
    html = page_config_review()
    return overlay(html, apply_dialog("8", "Every change is live at once on gateway-0, and gateway-1 follows within 5 seconds. Zorgcentrum Beta-Testerveen takes part.",
                                      REASON_8, "config-review.html", "config-applied.html"))


def page_config_applied():
    import pg_orgs
    top = notice("ok", "circle-check", f"<strong>Revision 8 runs on both replicas.</strong> Applied today 09:58 by you. Revision 7 comes back in one step from History.")
    return pg_orgs.org_page("zbt", "overview", pg_orgs.org_overview("zbt"), top=top)


# ---- secrets and credentials (F3), on the endpoint ----

def secret_now(ref, state="set"):
    if state == "unset":
        return f'<span class="fui-secret-now is-unset">{icon("lock", "12")}not set</span>'
    if state == "broken":
        return f'<span class="fui-secret-now is-broken">{icon("triangle-alert", "12")}set, but <code>{e(ref)}</code> cannot be read</span>'
    return f'<span class="fui-secret-now">{icon("lock", "12")}set, read from <code>{e(ref)}</code></span>'


def secret_field(label, mode="keep", now="/run/secrets/ferrofed/node-a-lab-client.pem", now_state="set", path="", error=None, idx=0, key=None):
    opts = [("keep", "Keep"), ("value", "Set a value"), ("file", "Read from a file"), ("clear", "Clear")]
    if now_state == "unset":
        opts = [("keep", "Leave unset"), ("value", "Set a value"), ("file", "Read from a file")]
    segs = "".join(f'<label class="fui-segment"><input type="radio" class="fui-sr-only" name="s{idx}"{" checked" if v == mode else ""} />{w}</label>' for v, w in opts)
    ctrl = ""
    inv = ' aria-invalid="true"' if error else ""
    er = f'<p class="fui-help fui-error">{icon("circle-alert", "14")} {error}</p>' if error else ""
    if mode == "value":
        ctrl = (f'<div class="fui-secret-control"><label for="sv{idx}" class="fui-label">Value, never shown again</label><input id="sv{idx}" type="password" class="fui-input" autocomplete="new-password" value="" {inv} />'
                f'{er}<p class="fui-help">Written to a new file the gateway alone reads; the revision holds only its path.</p></div>')
    elif mode == "file":
        ctrl = (f'<div class="fui-secret-control"><label for="sf{idx}" class="fui-label">Absolute path of the file it is read from</label>{input_(f"sf{idx}", path, extra="spellcheck=" + chr(34) + "false" + chr(34) + " class=" + chr(34) + "fui-input fui-mono" + chr(34) + inv)}{er}'
                '<p class="fui-help">A path on every replica\'s host; the gateway checks it can read it before it saves.</p></div>')
    elif mode == "clear":
        ctrl = f'<p class="fui-secret-clear">{icon("triangle-alert", "14")}The secret and its reference are removed. The endpoint is reached without it from the apply on.</p>'
    return (f'<div class="fui-secret"><div class="fui-secret-head"><span class="fui-label">{e(label)}{hint(key) if key else ""}</span>{secret_now(now, now_state)}</div>'
            f'<div class="fui-segmented" role="radiogroup" aria-label="What to do with the {e(label.lower())}">{segs}</div>{ctrl}</div>')


GRANTS = [("basic", "Basic", "A user and a password"), ("oauth", "OAuth 2.0 client credentials", "A token for a signed client assertion or a secret"),
          ("exchange", "OAuth 2.0 token exchange", "The caller's token exchanged, with DPoP"), ("nuts", "Nuts", "A DPoP-bound token for the gateway's credentials"),
          ("fapi", "FAPI 2.0", "A signed client assertion, DPoP and authorization details"), ("tls", "TLS client certificate only", "Mutual TLS, no token")]


def grant_cards(chosen):
    cards = ""
    for v, w, sub in GRANTS:
        on = v == chosen
        cards += (f'<label class="fui-choice-card"><input type="radio" class="fui-sr-only" name="grant" value="{v}"{" checked" if on else ""} />'
                  f'<span class="fui-choice-card-text"><span class="fui-choice-card-title">{w}</span><span class="fui-choice-card-sub">{sub}</span></span>'
                  f'<span class="fui-choice-card-check" aria-hidden="true">{icon("check", "12")}</span></label>')
    return f'<fieldset class="fui-choice-cards mk-ff-grants"><legend class="fui-label">Grant</legend>{cards}</fieldset>'


EP_FACT = '<div class="fui-field"><span class="fui-label">Endpoint</span><p><code>node-a-lab-pub</code> · node-a-lab of Ziekenhuis Voorbeeldstad</p></div>'
CRED_INTRO = ("A draft of node-a-lab-pub's credentials. A second operator approves it (onward credentials), and it runs on trial for 2 minutes after the apply until you confirm it.")


def cred_form(grant="oauth", errors=None):
    errors = errors or {}
    head = section("Endpoint and grant", EP_FACT + grant_cards(grant))
    if grant == "oauth":
        te_err = errors.get("te")
        fields = ('<div class="fui-columns">'
                  + field("c-te", "Token endpoint", input_("c-te", "auth.cdr-a-lab.example.org/oauth2/token" if te_err else "https://auth.cdr-a-lab.example.org/oauth2/token", extra='spellcheck="false" class="fui-input fui-mono"' + (' aria-invalid="true"' if te_err else "")),
                          help_="Or the issuer, whose metadata names it.", error=te_err)
                  + field("c-cid", "Client id", input_("c-cid", "ferrofed-gateway", extra='spellcheck="false" class="fui-input fui-mono"'))
                  + field("c-auth", "Client authentication", select("c-auth", [("pkj", "Signed client assertion (private_key_jwt)"), ("basic", "Client secret, Basic scheme"), ("post", "Client secret, in the body")], "pkj"))
                  + field("c-scope", "Scope", input_("c-scope", "system/aql-*.s system/composition-*.r", extra='spellcheck="false" class="fui-input fui-mono"'))
                  + "</div>")
        sec = secret_field("Private key", "file", now="", now_state="unset", path="/run/secrets/ferrofed/node-a-lab-client.pem", error=errors.get("key"), key="oauth.private_key_file")
        body = section("Grant settings", fields) + section("Secrets", '<p class="fui-small fui-muted">Write-only: a secret is set by a value or a file and never shown again.</p>' + sec)
    else:
        fields = ('<div class="fui-columns">'
                  + field("n-as", "Authorization server", input_("n-as", "https://nuts.cdr-a-lab.example.org/oauth2/ferrofed", extra='spellcheck="false" class="fui-input fui-mono"'))
                  + field("n-did", "DID", input_("n-did", "did:web:gateway.demo-regio.example.org", extra='spellcheck="false" class="fui-input fui-mono"'), help_="The gateway's own DID document is served at its did:web address.")
                  + field("n-scope", "Scope", input_("n-scope", "eOverdracht-sender", extra='spellcheck="false" class="fui-input fui-mono"'))
                  + field("n-kid", "Key id", input_("n-kid", "gateway-2026-10", extra='spellcheck="false" class="fui-input fui-mono"'))
                  + "</div>")
        sec = secret_field("DPoP key", "file", now="", now_state="unset", path="/run/secrets/ferrofed/nuts-dpop.pem", idx=1, key="nuts.dpop_key_file")
        body = section("Grant settings", '<p class="fui-small fui-muted">A DPoP-bound token for the gateway\'s credentials, from the Nuts node.</p>' + fields) + section("Secrets", sec)
    return head + body


def ep_page(title, sub, body, dock_html):
    content = page_heading(title, subtitle=sub, crumbs=[("Organisations", "organisations.html"), ("Ziekenhuis Voorbeeldstad", "organisation.html"), ("node-a-lab", routes.node_href("node-a-lab", "endpoints"))]) + f'<div class="fui-stack">{body}</div>'
    return shell(title, "orgs", content, dock_html=dock_html, org="zvs", node="node-a-lab")


def cred_dock(reason="", disabled_why=None):
    back = routes.node_href("node-a-lab", "endpoints")
    if disabled_why:
        p = button("primary", "Review changes", "git-compare", attrs='disabled aria-describedby="dock-why"')
        msg = f'<p id="dock-why" class="fui-small fui-muted">{disabled_why}</p>'
    else:
        p, msg = link_button("primary", "Review changes", "config-review-approval.html", "git-compare"), ""
    return dock(dock_field("cfg-reason", "Reason", reason, "Required: kept with the revision", keys="r"), link_button("secondary", "Discard", back, "x") + p, msg)


def page_credential_new():
    body = sections(panel_head(CRED_INTRO) + section("Endpoint and grant", EP_FACT + grant_cards(None)))
    return ep_page("New credentials", "How the gateway signs in to node-a-lab-pub; the form shows the grant's own fields.", body, cred_dock(disabled_why="Choose a grant first."))


def page_credential_oauth():
    return ep_page("New credentials", "OAuth 2.0 client credentials for node-a-lab-pub.", sections(panel_head(CRED_INTRO) + cred_form("oauth")), cred_dock(REASON_10))


def page_credential_nuts():
    return ep_page("New credentials", "The Nuts grant for node-a-lab-pub.", sections(panel_head(CRED_INTRO) + cred_form("nuts")), cred_dock("The laboratory CDR joins the regional Nuts network"))


def page_credential_invalid():
    errs = {"te": "The token endpoint is not an absolute https URL. Outside development every outbound URL is https.",
            "key": "/run/secrets/ferrofed/node-a-lab-client.pem cannot be read on gateway-1: no such file."}
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 2 fields need a change before you can review</p>'
               '<ul><li><a href="#c-te">Token endpoint: not an absolute https URL</a></li><li><a href="#sf0">Private key: gateway-1 cannot read the file</a></li></ul></div>')
    return ep_page("New credentials", "OAuth 2.0 client credentials for node-a-lab-pub.", summary + sections(panel_head(CRED_INTRO) + cred_form("oauth", errs)), cred_dock(REASON_10, disabled_why="Fix the 2 fields first."))


def page_secret_states():
    def frame(title, line, html):
        return f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p><div>{html}</div></div>'
    frames = (
        frame("Set, kept", "The default when a secret is set: what is set, by reference, and Keep chosen", secret_field("Password", "keep", now="/var/lib/ferrofed/secrets/node-b-1f3c.secret", idx=10, key="basic.password_file"))
        + frame("Set a value", "A password field the page never fills; the gateway writes it to a new file", secret_field("Password", "value", now="/var/lib/ferrofed/secrets/node-b-1f3c.secret", idx=11, key="basic.password_file"))
        + frame("Read from a file", "An absolute path every replica can read", secret_field("Private key", "file", now="/run/secrets/ferrofed/node-a-client.pem", path="/run/secrets/ferrofed/node-a-client-2026-11.pem", idx=12, key="oauth.private_key_file"))
        + frame("Clear", "The secret and its reference go; said as a warning before the save", secret_field("Password", "clear", now="/var/lib/ferrofed/secrets/node-b-1f3c.secret", idx=13, key="basic.password_file"))
        + frame("Not set", "Leave unset in place of Keep, and no Clear", secret_field("Client secret", "keep", now="", now_state="unset", idx=14, key="oauth.client_secret_file"))
        + frame("Set, unreadable", "The reference exists but a replica cannot read it: said in the tone of a fault", secret_field("Private key", "keep", now="/run/secrets/ferrofed/node-c-client.pem", now_state="broken", idx=15, key="oauth.private_key_file"))
        + frame("Value refused", "No secrets directory is set, so a value cannot be written; a file still works", secret_field("Password", "value", now="/var/lib/ferrofed/secrets/node-b-1f3c.secret", error=f"This gateway has no secrets directory{hint('config.secrets_dir')}, so it cannot write a value: read it from a file.", idx=16, key="basic.password_file"))
    )
    rules = card("Rules", definitions([
        ("Never shown", "A secret's value never reaches a page, a revision, a diff, an export or a log; the page holds no value after a save"),
        ("Reference", "The revision names the file a secret is read from; a value becomes a new file, never an existing one, so a rollback still reads the old one"),
    ]))
    content = page_heading("Secret field, states", subtitle="The write-only secret of an endpoint's credentials or a service's connection.") + f'<div class="fui-stack mk-dock-sheet">{frames}{rules}</div>'
    return shell("Secret field states", "configuration", content)


CRED_DIFF = [("Token endpoint", 'credentials."node-a-lab-pub".oauth.token_endpoint', "", "https://auth.cdr-a-lab.example.org/oauth2/token", "Runs on trial after approval"),
             ("Client id", 'credentials."node-a-lab-pub".oauth.client_id', "", "ferrofed-gateway", "Runs on trial after approval"),
             ("Client authentication", 'credentials."node-a-lab-pub".oauth.client_auth', "", "private_key_jwt", "Runs on trial after approval"),
             ("Scope", 'credentials."node-a-lab-pub".oauth.scope', "", "system/aql-*.s system/composition-*.r", "Runs on trial after approval"),
             ("Private key", 'credentials."node-a-lab-pub".oauth.private_key_file', "", "/run/secrets/ferrofed/node-a-lab-client.pem", "Runs on trial after approval")]
APPROVAL_NOTE = notice("info", "user-check", f"<strong>A second operator approves this, then it runs on trial.</strong> This deployment approves onward credentials twice{hint('config.approval')}. After the apply, confirm it within 2 minutes or the gateway rolls it back.")


def cred_review(readonly=False):
    eff = '<p class="fui-small">The gateway asked the token endpoint for its metadata as a check: it answered, and names private_key_jwt among its methods. No token was requested.</p>'
    body = review_body(CRED_DIFF, eff, APPROVAL_NOTE, against="9")
    sub = "Your draft against revision 9, which runs; saved, it is revision 10." if not readonly else "Revision 10 by Demo Operator, against revision 9. Read-only: only its author edits a draft."
    title = "Review the new credentials" if not readonly else "Revision 10"
    d = "" if readonly else review_dock(REASON_10, back="endpoint-credentials-oauth.html", apply_href="config-awaiting-approval.html", approval=True)
    return form_page(title, [("Organisations", "organisations.html"), ("Ziekenhuis Voorbeeldstad", "organisation.html"), ("node-a-lab", routes.node_href("node-a-lab", "endpoints"))] if not readonly else [("Configuration", "configuration.html"), ("History", "configuration-history.html")],
                     sub, body, d, current="orgs" if not readonly else "configuration")


def page_credential_review():
    return cred_review()


def approval_card(viewer="author", approved=False):
    if approved:
        steps = [("done", "Saved", "by Demo Operator, today 10:02"), ("done", "Approved", "by Demo Reviewer, today 10:09"), ("current", "Apply", "any operator"), ("waiting", "Confirm", "within 2 min of the apply")]
        acts = link_button("secondary", "Review revision 10", "r-review-10.html", "git-compare", small=True) + link_button("primary", "Apply revision 10…", "config-apply-trial.html", "check", small=True)
        return pending_card("Revision 10 is approved", "Demo Reviewer approved it at 10:09. Apply it, then confirm it once the gateway reaches node-a-lab-pub with it.", steps, acts, glyph="badge-check")
    steps = [("done", "Saved", "by Demo Operator, today 10:02"), ("current", "Approval", "an operator other than its author"), ("waiting", "Apply", "after the approval"), ("waiting", "Confirm", "within 2 min of the apply")]
    if viewer == "author":
        acts = button("secondary", "Copy a link for the approver", "copy", small=True) + button("primary", "Apply revision 10…", "check", small=True, attrs='disabled aria-describedby="ap-why"')
        return pending_card("Revision 10 waits for a second operator", "It changes the credentials of node-a-lab-pub. You saved it, so another operator approves it.", steps, acts, glyph="user-check",
                            body='<p id="ap-why" class="fui-small fui-muted">Apply opens once an operator other than you approves it.</p>')
    acts = link_button("secondary", "Review revision 10", "r-review-10.html", "git-compare", small=True) + link_button("primary", "Approve revision 10…", "config-approve-dialog.html", "user-check", small=True)
    return pending_card("Revision 10 asks for your approval", f"Demo Operator saved it at 10:02: “{REASON_10}.” It changes the credentials of node-a-lab-pub.", steps, acts, glyph="user-check")


def page_config_awaiting_approval():
    return config_page("federation", top=approval_card("author"), running="9")


def reviewer_page(overlay_=None):
    html = config_page("federation", top=approval_card("reviewer"), overlay=overlay_, running="9")
    return html.replace("Demo Operator</span>", "Demo Reviewer</span>", 1).replace("<strong>Demo Operator</strong>", "<strong>Demo Reviewer</strong>")


def page_config_approver():
    return reviewer_page()


def page_config_approve_dialog():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="ap-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="ap-title" class="fui-dialog-title">{icon("user-check", "18")} Approve revision 10?</h2>'
        '<p>You approve it as an operator other than its author, and the approval names you. Approving applies nothing; any operator can then apply it.</p>'
        + well(definitions([("Changes", "the credentials of node-a-lab-pub: 5 settings, one secret by reference"), ("Saved by", "Demo Operator, today 10:02"), ("Reason", REASON_10)]))
        + f'<div class="fui-dialog-actions">{link_button("secondary", "Cancel", "config-approver.html", keys="Esc")}{link_button("primary", "Approve revision 10", "config-approved.html", "user-check")}</div></div>'
    )
    return reviewer_page(dialog)


def page_config_approved():
    return config_page("federation", top=approval_card(approved=True), running="9")


def page_config_apply_trial():
    note = notice("warn", "timer", "<strong>It runs on trial.</strong> Once applied, confirm it within 2 minutes, once a query reaches node-a-lab-pub with it. Unless someone confirms it, the gateway rolls it back to revision 9's content by itself.")
    d = apply_dialog("10", "The credentials of node-a-lab-pub are live at once on gateway-0; gateway-1 follows within 5 seconds.", REASON_10, "config-approved.html", "config-trial.html", note, restore="9")
    return overlay(page_config_approved(), d)


def countdown_ring(left=102, total=120, until="10:14:42"):
    r = 13
    circ = 2 * 3.14159 * r
    m, s = divmod(left, 60)
    return (f'<span class="fui-countdown fui-tone-warn" role="timer" aria-live="off"><svg viewBox="0 0 32 32" aria-hidden="true"><circle class="fui-countdown-track" cx="16" cy="16" r="{r}"/>'
            f'<circle class="fui-countdown-left" cx="16" cy="16" r="{r}" stroke-dasharray="{circ:.2f}" stroke-dashoffset="{circ * (1 - left / total):.2f}"/></svg>'
            f'<span class="fui-countdown-words"><strong>{m}:{s:02d}</strong><span>left, until {until}</span></span></span>')


def page_config_trial():
    steps = [("done", "Saved", "by Demo Operator, 10:02"), ("done", "Approved", "by Demo Reviewer, 10:09"), ("done", "Applied", "by you, 10:12:42"), ("current", "Confirm", "by 10:14:42, or it rolls back")]
    card_ = pending_card("Revision 10 runs on trial", "It sets the credentials of node-a-lab-pub. Unless an operator confirms it by 10:14:42, the gateway restores revision 9's content.",
                         steps, countdown_ring() + link_button("secondary", "Roll back now", "config-rolled-back.html", "rotate-ccw", small=True) + link_button("primary", "Confirm revision 10", "config-confirmed.html", "check", small=True), tone="warn", glyph="timer",
                         body='<p class="fui-small">Check it first: <a href="query.html">run a query</a> with node-a-lab-pub in scope. The last one, at 10:12:58, reached it: <strong>answered, 41 ms</strong>.</p>')
    return config_page("federation", status_html="", top=card_)


def page_config_confirmed():
    strip = revision_strip("ok", "Revision 10 runs, confirmed", f"Confirmed by you at 10:13:58, 44 seconds before the deadline; the gateway no longer rolls it back. {replicas_inline()}",
                           link_button("ghost", "Roll back…", "config-rollback.html", "rotate-ccw", small=True))
    return config_page("federation", status_html=strip, running="10")


def page_config_rolled_back():
    steps = [("done", "Saved", "by Demo Operator, 10:02"), ("done", "Approved", "by Demo Reviewer, 10:09"), ("done", "Applied", "10:12:42"), ("failed", "Not confirmed", "the deadline passed at 10:14:42")]
    card_ = pending_card("Revision 10 was rolled back: no one confirmed it in time", "The gateway saved revision 9's content as revision 11 and applied it at 10:14:42, with the reason “revision 10 was not confirmed by its deadline”. node-a-lab-pub is reached without credentials again.",
                         steps, link_button("secondary", "Compare 10 with 11", routes.compare_href(11), "git-compare", small=True) + link_button("primary", "New draft from revision 10", "endpoint-credentials-oauth.html", "pencil", small=True), tone="bad", glyph="rotate-ccw")
    return config_page("federation", status_html="", top=card_)


def page_lifecycle_states():
    def frame(title, line, html):
        return f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p><div>{html}</div></div>'
    frames = (
        frame("Saved, no approval needed", "An organisation change: approval and confirmation are not needed, so their steps are a dash",
              lifecycle([("done", "Saved", "by you, 09:51"), ("skipped", "Approval", "not needed"), ("current", "Apply", "yours, or any operator's"), ("skipped", "Confirm", "not needed")]))
        + frame("Waiting for approval", "The author sees whose turn it is",
                lifecycle([("done", "Saved", "by Demo Operator, 10:02"), ("current", "Approval", "another operator"), ("waiting", "Apply", "after the approval"), ("waiting", "Confirm", "within 2 min")]))
        + frame("On trial", "Applied; the confirmation is the current step",
                lifecycle([("done", "Saved", "10:02"), ("done", "Approved", "Demo Reviewer, 10:09"), ("done", "Applied", "10:12:42"), ("current", "Confirm", "by 10:14:42")]))
        + frame("Confirmed", "Every step done",
                lifecycle([("done", "Saved", "10:02"), ("done", "Approved", "10:09"), ("done", "Applied", "10:12:42"), ("done", "Confirmed", "by you, 10:13:58")]))
        + frame("Rolled back", "The failed step in the bad tone, with its cause",
                lifecycle([("done", "Saved", "10:02"), ("done", "Approved", "10:09"), ("done", "Applied", "10:12:42"), ("failed", "Not confirmed", "deadline 10:14:42")]))
    )
    keys = card("Rules", definitions([
        ("Steps", "Saved, Approval, Apply, Confirm: a step a revision does not take is drawn as not needed, never hidden, so every revision reads the same way"),
        ("Words first", "Each step says its state in words and a time; the dot and the rule repeat it"),
    ]))
    content = page_heading("Revision lifecycle, states", subtitle="Where a saved revision stands and whose turn it is.") + f'<div class="fui-stack mk-dock-sheet">{frames}{keys}</div>'
    return shell("Revision lifecycle states", "configuration", content)


# ---- History, read at 10:20 with revision 10 running ----

ALL_REVS = REVISIONS + FUTURE
ORIGIN_WORD = {"operator": ("neutral", "Operator"), "file": ("info", "File"), "rollback": ("warn", "Rollback"), "import": ("accent", "Import"), "gateway": ("bad", "Gateway")}


def history_rows(items):
    rows = []
    for n, origin, by, at, reason, what in items:
        st = pill("ok", "Running") if n == "10" else '<span class="fui-small fui-muted mk-nowrap">Ran before</span>'
        acts = (f'<a class="fui-icon-button" href="{routes.compare_href(n)}" aria-label="Compare revision {n} with the running one" title="Compare with the running one">{icon("git-compare", "16")}</a>' if n != "10" else "") + icon_button("download", f"Export revision {n} as TOML")
        tone, w = ORIGIN_WORD[origin]
        rows.append(f'<tr><th scope="row" class="mk-nowrap" data-role="title">Revision {n}</th>' + td("Origin", badge(tone, w), role="meta-q") + td("By", e(by), "mk-nowrap", role="meta-q") + td("When", when(at), role="meta-q")
                    + td("Reason", e(reason), role="sub") + td("State", st, role="meta-q") + td("", f'<span class="fui-cluster mk-nowrap">{acts}</span>', "fui-row-menu mk-nowrap", role="menu") + "</tr>")
    return rows


def history_tools(origin_word=None, picker_html=None):
    org_btn = link_button("secondary", "Origin" + (f" · {origin_word}" if origin_word else ""), "configuration-history-filtered.html", "list-filter")
    if picker_html:
        rng = (f'<span class="fui-anchor">{button("secondary", "2026-10-12 to 2026-10-19", "calendar", attrs="aria-haspopup=" + chr(34) + "dialog" + chr(34) + " aria-expanded=" + chr(34) + "true" + chr(34))}'
               f'<span class="mk-range-pop">{picker_html}</span></span>')
    else:
        rng = link_button("secondary", "Any date", "configuration-history-range.html", "calendar")
    return '<div class="fui-panel-section">' + tpl.toolbar("Reason, who or a revision number", "", [org_btn, rng]) + "</div>"


def history_panel(items=None, tools=None, chips=""):
    items = list(reversed(ALL_REVS)) if items is None else items
    head = panel_head("Every revision, who made it and why, newest first.", link_button("secondary", "Roll back…", "config-rollback.html", "rotate-ccw", small=True))
    tbl = '<div class="fui-worklist">' + table("Revisions", th("Revision") + th("Origin") + th("By") + th("When") + th("Reason") + th("State") + th('<span class="fui-sr-only">Actions</span>'), history_rows(items)) + "</div>"
    return f'<div class="fui-panel">{head}{tools or history_tools()}{chips}{tbl}</div>'


def page_configuration_history():
    return config_page("history", body=history_panel(), running="10")


def page_configuration_history_filtered():
    items = [r for r in reversed(ALL_REVS) if r[1] in ("rollback", "import")]
    chips = f'<div class="fui-panel-section">{tpl.active(["Origin: rollback, import"], shown=(len(items), len(ALL_REVS)))}</div>'
    return config_page("history", body=history_panel(items, history_tools("rollback, import"), chips), running="10")


def page_configuration_history_range():
    rng = (D(2026, 10, 12), D(2026, 10, 19))
    months = (month_grid(2026, 9, rng=rng, max_d=PICK_TODAY, title_btn=False, nav="both") + month_grid(2026, 10, rng=rng, max_d=PICK_TODAY, title_btn=False, nav="both"))
    p = picker("Revisions between", f'<div class="fui-datepicker-pair">{months}</div>', presets=[("Today", False), ("Last 7 days", False), ("Last 30 days", False), ("This month", False)],
               status="2026-10-12 to 2026-10-19 · 8 days.")
    return config_page("history", body=history_panel(tools=history_tools(picker_html=p)), running="10")


CHANGES_AFTER = {
    "6": [("Organisation Revalidatiecentrum Demo-West", "registry.members[org-rdw]", "", "org-rdw with node-d", "live"), ("Node node-a-lab", "registry.members[org-zvs].nodes", "", "node-a-lab · cdr-a-lab.example.org", "live")],
    "7": [("Per-node timeout", "federation.per_node_timeout_ms", "2,000 ms", "2,500 ms", "restart")],
    "8": [("Organisation Zorgcentrum Beta-Testerveen", "registry.members[org-zbt]", "", "org-zbt with node-f", "live")],
    "9": [("Per-node timeout", "federation.per_node_timeout_ms", "2,500 ms", "3,000 ms", "restart"), ("Overall timeout", "federation.overall_timeout_ms", "4,000 ms", "5,000 ms", "restart")],
    "10": [("Credentials of node-a-lab-pub", 'credentials."node-a-lab-pub"', "", "OAuth 2.0 client credentials", "live")],
}


def compare_rows(n):
    """Every setting that differs between revision n and the running one, 10."""
    out = {}
    for k in range(int(n) + 1, 11):
        for lbl, key, a, b, fx in CHANGES_AFTER.get(str(k), []):
            if key in out:
                out[key] = (lbl, key, out[key][2], b, fx)
            else:
                out[key] = (lbl, key, a, b, fx)
    return list(out.values())


def compare_page(n):
    rows = compare_rows(n) if str(n) != "11" else [("Credentials of node-a-lab-pub", 'credentials."node-a-lab-pub"', "OAuth 2.0 client credentials", "", "live")]
    rev = next(r for r in ALL_REVS if r[0] == str(n)) if str(n) != "11" else ("11", "gateway", "the gateway", "2026-10-21T10:14:42", "revision 10 was not confirmed by its deadline", "")
    body = [f'<tr><th scope="row">{e(lbl)}{hint(k)}</th>' + td(f"Revision {n}", change_del(e(a)) if a else '<span class="mk-dash">not set</span>') + td("Running", change_ins(e(b)) if b else '<span class="mk-dash">removed</span>') + td("Takes effect", effect(fx)) + "</tr>" for lbl, k, a, b, fx in rows]
    t = table(f"Revision {n} against the running one", th("Setting") + th(f"Revision {n}") + th("Running") + th("Takes effect"), body) if rows else quiet_line("Nothing differs.", boxed=False)
    tone, w = ORIGIN_WORD.get(rev[1], ("bad", "Gateway"))
    other = "11" if str(n) == "11" else "10"
    title = "Revision 10 against revision 11" if str(n) == "11" else f"Revision {n} against revision 10"
    content = (page_heading(title, subtitle="Each setting that differs, with how it took effect.", crumbs=[("Configuration", "configuration.html"), ("History", "configuration-history.html")])
               + '<div class="fui-stack">' + card(f'{len(rows)} setting{"s" if len(rows) != 1 else ""} differ', t)
               + card(f"Revision {n}", definitions([("Origin", badge(tone, w)), ("By", e(rev[2])), ("When", when(rev[3])), ("Reason", e(rev[4]))]),
                      actions=button("secondary", f"Export revision {n} as TOML", "download", small=True)) + "</div>")
    return shell("Configuration", "configuration", content)


def page_configuration_compare():
    return compare_page("5")


def page_config_rollback():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="rb-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="rb-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Roll back revision 10?</h2>'
        '<p>Revision 9\'s content, which ran before it, is saved as revision 11 and applied: node-a-lab-pub is reached without credentials again, at once. A rollback takes no approval and no confirmation.</p>'
        + field("rb-reason", "Reason", textarea("rb-reason", "The laboratory CDR's authorization server rejects the new client"), help_="Required. Kept with the revision.")
        + f'<div class="fui-dialog-actions">{link_button("secondary", "Cancel", "configuration-history.html", keys="Esc")}{button("primary", "Roll back to revision 9", "rotate-ccw")}</div></div>'
    )
    return config_page("history", body=history_panel(), overlay=dialog, running="10")


# ---- Import and export, under managed ----

IMPORT_TOML = '[access_log.retention]\nyears = 5\n\n[credentials."node-d-pub".nuts]\nscope = "eOverdracht-sender"\n'


def io_panel(toml=IMPORT_TOML, saved=False, refused=None):
    export = section("Export", f'<p class="fui-small fui-muted">Revision 10 as TOML: every secret as its file reference; the core is not part of it.</p>'
                     f'<div>{button("secondary", "Download revision 10 as TOML", "download", small=True)}</div>')
    form = (field("imp-toml", "TOML to import", textarea("imp-toml", toml, rows=7, extra='spellcheck="false"', cls="fui-textarea mk-code"),
                  help_="What an export wrote, or a file kept in version control. Saved as a dry-run revision; the import applies nothing.")
            + field("imp-reason", "Reason", input_("imp-reason", "Retention and the Nuts scope, as merged in the config repository"), help_="Required, with the import.")
            + f'<div>{button("primary", "Import as a dry run", "upload", small=True) if not saved else ""}</div>')
    parts = section("Import", form)
    if refused:
        parts += f'<div class="fui-panel-section">{refused}</div>'
    if saved:
        rows = [("Retention", "access_log.retention.years", "3 years", "5 years", "live"), ("Scope of node-d-pub's credentials", 'credentials."node-d-pub".nuts.scope', "eOverdracht", "eOverdracht-sender", "live")]
        parts += section("Saved as revision 11, a dry run", '<p class="fui-small">Checked as <code>ferrofed config check</code> would: no problems. It changes a credential, so a second operator approves it before anyone can apply it.</p>'
                         + diff_table(rows, base="Revision 10", right="Revision 11")
                         + f'<div class="fui-actions">{button("secondary", "Copy a link for the approver", "copy", small=True)}{link_button("secondary", "Leave it unapplied", "configuration-history.html", small=True)}</div>'
                         + '<p class="fui-small fui-muted">Left unapplied, it stays in History.</p>')
    return f'<div class="fui-panel">{export}{parts}</div>'


def page_configuration_import():
    return config_page("io", body=io_panel(saved=True), running="10")


def page_configuration_import_refused():
    toml = IMPORT_TOML + '\n[auth]\nproduct_scope = "__ferrofed.admin"\n'
    ref = refusal("The import was refused (422): nothing was saved.", issues=[("bad", "Error", "auth.product_scope", "a key of the core: [auth] stays in ferrofed.toml, so no revision changes who may call.")])
    return config_page("io", body=io_panel(toml, refused=ref), running="10")


# ---- revisions off: a read-only view of ferrofed.toml ----

def page_configuration_file_mode(overlay_=None):
    return config_page("federation", mode="off", overlay=overlay_)


def page_configuration_file_mode_dialog():
    return page_configuration_file_mode(overlay_=mode_dialog("off"))


def page_configuration_file_mode_history():
    return config_page("consent", mode="off")


def page_configuration_file_mode_auth():
    return config_page("auth", mode="off")


# ---- route-only pages ----

def route_pages():
    out = {}
    for k in ("fanout", "stored", "access", "fhir"):
        out[f"r-config-off-{k}"] = lambda k=k: config_page(k, mode="off")
    for k in ("fanout", "consent", "stored", "access", "fhir"):
        out[routes.slug("config", k, "edit")] = lambda k=k: config_page(k, body=tpl.settings_panel(tab_sections()[k][1], "A draft of this tab, based on revision 7: no change yet.", ''),
                                                                     drafts=(k,), dock_html=dock(dock_field("cfg-reason", "Reason", "", "Required: kept with the revision", keys="r"),
                                                                                                 link_button("secondary", "Discard", "configuration.html", "x") + button("primary", "Review changes", "git-compare", attrs="disabled")))
    for r in ALL_REVS:
        if r[0] not in ("5", "10"):
            out[f"r-compare-{r[0]}"] = lambda n=r[0]: compare_page(n)
    out["r-compare-11"] = lambda: compare_page("11")
    out["r-review-10"] = lambda: cred_review(readonly=True)
    return out
