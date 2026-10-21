# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Configuration in the approved one-mode model: `[config] mode = "file" |
"console"`, the core always in ferrofed.toml, everything else in one place.
Console mode saves every change (made on an object's own page, or to the
bindings and settings here) as drafts, reviewed against what runs, saved as a revision with a reason and
applied, with FerroFED's second-operator approval and commit-confirm on
the resources that take them. File mode shows the same tabs read-only."""
from common import (
    MENU_SEP, badge, busy_button, button, card, check, chip, choice_group, definitions, disclosure, dock, dock_field, e,
    empty_state, facts_row, field, icon, icon_button, input_, link_button, menu, menu_item, notice, page_heading, pill,
    refusal, select, shell, status, table, tabbed, tabs, tag, td, textarea, th, titled_panel, toast, toasts, well, when,
)
from data import MEMBERS, REVISIONS
from parts import (D, change_del, change_ins, choice_cards, date_field_open, date_input, effect, filter_chips, fold_actions,
                   mode_badge, month_grid, picker, revision_strip, PICK_TODAY)

# ---- the model ----

CORE = ('<ul class="fui-bullets fui-small"><li><code>profile</code>, <code>[server]</code> and its TLS, <code>[telemetry]</code></li>'
        '<li><code>[metrics]</code>: the admin listener this page talks to</li>'
        '<li><code>[auth]</code> and <code>[trust]</code>: who may call, the operator scope among it</li>'
        '<li><code>[signing]</code>: the keys the gateway signs with</li>'
        '<li><code>[config]</code> itself, <code>[usage_report]</code> and <code>[dev]</code></li></ul>')
REST = ('<ul class="fui-bullets fui-small"><li>Organisations, their nodes and each node\'s endpoints, on their own pages</li>'
        '<li>Onward credentials, on each endpoint</li>'
        '<li>Services: PIXm, PDQm, PMIR, the NVI, XCPD, Mitz, the audit repository and the terminology server, each on its page</li>'
        '<li>The deployment-wide settings, here: budgets, completeness, node selection, consent disclosure, the stored-query registry, the access log and the FHIR face</li></ul>')


def mode_dialog(mode):
    if mode == "console":
        here = ('<h3>' + icon("monitor", "16") + ' Everything else: here, in the console</h3><p class="fui-small fui-muted">Each change is a draft you review against what runs, saved as a revision with a reason and applied; '
                'every replica follows within 5 seconds. A credential or a binding runs on trial until you confirm it, and the kinds your deployment names take a second operator\'s approval.</p>')
        other = ('<p class="fui-small fui-muted mk-mode-other">The other mode, <strong>file</strong>, keeps everything in ferrofed.toml for version control, and this console then shows it read-only. '
                 'A deployment uses one mode. To switch, export the running revision as TOML, put it beside the core, set <code>[config] mode = "file"</code> and restart.</p>')
    else:
        here = ('<h3>' + icon("file-text", "16") + ' Everything else: also in ferrofed.toml</h3><p class="fui-small fui-muted">Changed in the file, reviewed in git, applied by a reload (<code>SIGHUP</code>); '
                'shown here read-only, with each reading of the file in the history.</p>')
        other = ('<p class="fui-small fui-muted mk-mode-other">The other mode, <strong>console</strong>, keeps only the core in the file and edits everything else here. '
                 'A deployment uses one mode. To switch, set <code>[config] mode = "console"</code> and restart: the file\'s settings become revision 1, and the gateway asks you to remove them from the file.</p>')
    lead = ("This deployment runs in <strong>console mode</strong> (<code>[config] mode = \"console\"</code>)." if mode == "console"
            else "This deployment runs in <strong>file mode</strong> (<code>[config] mode = \"file\"</code>).")
    return (
        '<div id="mode-dialog" role="dialog" aria-modal="true" aria-labelledby="mode-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="mode-title" class="fui-dialog-title">{icon("info", "18")} How configuration works</h2>'
        f'<p>{lead} The core always stays in ferrofed.toml; everything else lives in one place, never in both.</p>'
        '<div class="fui-modal-body"><div class="fui-ownership">'
        '<div><h3>' + icon("lock", "16") + ' The core: always in ferrofed.toml</h3><p class="fui-small fui-muted">Never in a revision, so no screen can lock you out of the admin listener or lower the security floor the file sets.</p>' + CORE + '</div>'
        '<div>' + here + REST + '</div></div>'
        '<p class="fui-small fui-muted mk-mode-other">In both modes, some things are never configuration: an organisation or node the care services directory provides, the gateway\'s reserved <code>eu.ferrofed.eehrxf</code> queries, and a stored-query version, which never changes once stored. They show a lock.</p>'
        + other + '</div>'
        f'<div class="fui-dialog-actions">{button("secondary", "Close", "x", keys="Esc", attrs="autofocus")}</div></div>'
    )


def mode_help(mode):
    href = "configuration-mode-dialog.html" if mode == "console" else "configuration-file-mode-dialog.html"
    return link_button("ghost", "How configuration works", href, "info", attrs='aria-haspopup="dialog"')


def config_heading(mode="console", menu_open=False, running="7"):
    if mode == "console":
        items = [mode_help("console"), button("secondary", f"Export revision {running} as TOML", "download"), link_button("secondary", "Import TOML…", "configuration-import.html", "upload")]
        mitems = [menu_item("How configuration works", "info", href="configuration-mode-dialog.html"), menu_item(f"Export revision {running} as TOML", "download"),
                  menu_item("Import TOML…", "upload", href="configuration-import.html")]
        sub = "The deployment-wide settings no other page owns, the revision that runs, every change before it, and each revision waiting for its approval or apply."
    else:
        items = [mode_help("file"), button("secondary", "Export the running settings as TOML", "download")]
        mitems = [menu_item("How configuration works", "info", href="configuration-file-mode-dialog.html"), menu_item("Export the running settings as TOML", "download")]
        sub = "What ferrofed.toml sets, as the gateway read it at the last reload, and every reading before it."
    return page_heading("Configuration", actions=fold_actions(items, mitems, open_=menu_open), subtitle=sub, badge=mode_badge(mode))


TABS = [("settings", "Settings", "configuration.html"), ("history", "History", "configuration-history.html")]
FILE_TABS = [("settings", "Settings", "configuration-file-mode.html"), ("history", "Readings", "configuration-file-mode-history.html")]


def replicas_inline(refused=False):
    g1 = pill("bad", "refused revision 8") if refused else pill("ok", "in step")
    return f'<ul class="fui-replicas"><li><code>gateway-0</code>{pill("ok", "in step")}</li><li><code>gateway-1</code>{g1}</li></ul>'


def running_strip(n="7", by="Demo Operator", at="2026-10-19T14:12", reason="Raise the per-node timeout for node-b's slower link", extra_actions=""):
    line = f"Applied {when(at)} by {by}: “{reason}.” {replicas_inline()}"
    return revision_strip("running", f"Revision {n} runs", line, extra_actions + button("ghost", "Roll back…", "rotate-ccw", small=True))


def lifecycle(steps):
    """steps: (state, title, line) with state done|current|waiting|skipped|failed."""
    lis = ""
    for st, title, line in steps:
        glyph = {"done": icon("check", "12"), "failed": icon("x", "12"), "skipped": icon("minus", "12")}.get(st, "")
        lis += f'<li class="is-{st}"><span class="fui-lifecycle-dot" aria-hidden="true">{glyph}</span><span class="fui-lifecycle-text"><strong>{title}</strong><span>{line}</span></span></li>'
    return f'<ol class="fui-lifecycle" aria-label="Where this revision stands">{lis}</ol>'


def pending_card(title, line, steps, actions="", tone="", glyph="pencil", body=""):
    t = f" fui-tone-{tone}" if tone else ""
    role = ' role="status"' if tone else ""
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    return (f'<section class="fui-pending{t}"{role} aria-label="{e(title)}"><div class="fui-pending-head">{icon(glyph, "18")}'
            f'<div class="fui-revision-text"><strong>{title}</strong><span>{line}</span></div>{a}</div>{lifecycle(steps)}{body}</section>')


def config_page(tab, panel, mode="console", strip=None, overlay=None, toast_html="", menu_open=False, dock_html="", running="7"):
    if strip and 'class="fui-pending' in strip:
        panel = panel.replace("None: every saved revision runs or ran before. A change saved on an object\'s page waits here for its approval and its apply.",
                              "One revision waits: its card is at the top of the page, with whose turn it is.")
    t = tabs("Configuration", TABS if mode == "console" else FILE_TABS, tab)
    s = strip if strip is not None else running_strip()
    content = config_heading(mode, menu_open, running) + f'<div class="fui-stack">{s}{tabbed(t, panel)}</div>'
    return shell("Configuration", "configuration", content, overlay=overlay, toast_html=toast_html, dock_html=dock_html)


def locked(word):
    return f'<span class="fui-tag fui-tag-locked">{icon("lock", "12")}{e(word)}</span>'


def panel_head(title, line, actions=""):
    return (f'<div class="fui-panel-head mk-card-head"><div><h2 class="fui-card-title">{title}</h2><p class="fui-small fui-muted">{line}</p></div>'
            f'<div class="fui-actions">{actions}</div></div>')


def overview_panel(mode="console", pending=""):
    link = (lambda txt, href: link_button("ghost", txt, href, glyph_end="chevron-right", small=True))
    rows = [
        ("Organisations, nodes and endpoints", "5 organisations · 6 nodes · 7 endpoints", link("Manage in Organisations", "organisations.html")),
        ("Onward credentials", "6 of 7 endpoints have credentials", link("On each endpoint, in its node", "node.html")),
        ("Bindings", "5 services set · XCPD not set", link("Bindings tab", "services.html")),
        ("Settings", "13 settings in 5 sections", link("Settings tab", "configuration.html")),
        ("Stored queries", "6 queries, 10 versions · data, never a revision", link("Manage in Stored queries", "stored-queries.html")),
    ]
    body = [f'<tr><th scope="row">{a}</th>' + td("Now", b) + td("Home", c, "fui-table-actions") + "</tr>" for a, b, c in rows]
    line = ("A revision holds all of these, so each one shows here read-only; each is created and changed on its own page, and saved there as a revision you review and apply here."
            if mode == "console" else "In file mode ferrofed.toml sets all of these; every page shows them read-only.")
    holds = card("What a revision holds", f'<p class="fui-small fui-muted">{line}</p>' + table("What a revision holds", th("Part") + th("Now") + th("Home"), body))
    reps = card("Replicas", definitions([("gateway-0", pill("ok", "runs revision 7 · in step")), ("gateway-1", pill("ok", "runs revision 7 · in step")),
                                        ("Follows within", "5 seconds (<code>config.poll_ms</code>)")]))
    waits = pending or card("Waiting revisions", '<p class="fui-small fui-muted">None: every saved revision runs or ran before. A change saved on an object\'s page waits here for its approval and its apply.</p>')
    return f'<div class="fui-panel-body fui-stack">{waits}<div class="mk-ff-split">{holds}{reps}</div></div>'


# ---- members ----






def page_configuration(overlay=None, menu_open=False):
    return config_page("settings", settings_panel(), overlay=overlay, menu_open=menu_open)


def page_configuration_mode_dialog():
    return page_configuration(overlay=mode_dialog("console"))


def page_configuration_actions():
    return page_configuration(menu_open=True)


def page_configuration_loading():
    import tpl
    from parts import skeleton_lines
    rows = "".join(f'<div class="fui-setting-row" aria-hidden="true"><div class="fui-setting-text">{skeleton_lines([30, 60])}</div><div class="fui-setting-control">{skeleton_lines([50])}</div></div>' for _ in range(5))
    body = tpl.settings_panel(rows, '<span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Reading the revisions from gateway-0</span>')
    strip = '<div class="fui-revision" aria-hidden="true"><span class="fui-skeleton mk-w40"></span></div>'
    return config_page("federation", body=body, status_html=strip)


def edit_dock(primary="Review the changes", reason="", disabled=False, back=("Cancel", "configuration.html"), error=None, msg=""):
    fld = dock_field("cfg-reason", "Reason", reason, "Why this changes; kept with the revision", keys="r", error=error)
    acts = link_button("secondary", back[0], back[1], "x") + button("primary", primary, "git-compare", attrs="disabled" if disabled else "")
    return dock(fld, acts, msg)


def form_page(title, crumbs, sub, body, dock_html, strip="", current="configuration", **kw):
    content = page_heading(title, subtitle=sub, crumbs=crumbs) + f'<div class="fui-stack">{strip}{body}</div>'
    return shell(title, current, content, dock_html=dock_html, **kw)






# ---- review, save, apply ----

def diff_table(rows, base="Revision 7, running", right="Your draft"):
    def v(x):
        return f'<code class="fui-break">{e(x)}</code>' if ("/" in x or "." in x and " " not in x) else e(x)
    body = [f'<tr><th scope="row"><code class="fui-break">{k}</code></th>' + td(base, change_del(v(a)) if a else '<span class="mk-dash">not set</span>')
            + td(right, change_ins(v(b)) if b else '<span class="mk-dash">removed</span>') + td("Takes effect", effect(fx)) + "</tr>" for k, a, b, fx in rows]
    return table("What changes", th("Setting") + th(base) + th(right) + th("Takes effect"), body)


MEMBER_DIFF = [
    ("registry.organisations[org-zbt].name", "", "Zorgcentrum Beta-Testerveen", "live"),
    ("nl_gf.mitz.holders.org-zbt", "", "90000106", "live"),
    ("pixm.manager.domains.org-zbt", "", "urn:oid:2.999.2.6", "live"),
]


def review_dock(reason="Zorgcentrum Beta-Testerveen joins the federation in November", primary="Save and apply…", secondary="Save only", approval=False):
    fld = dock_field("cfg-reason", "Reason", reason, "Why this changes; kept with the revision", keys="r")
    if approval:
        acts = button("secondary", "Back to editing", "pencil") + button("primary", "Save for approval", "save")
    else:
        acts = button("secondary", "Back to editing", "pencil") + button("secondary", secondary, "save") + button("primary", primary, "check")
    return dock(fld, acts)


def review_body(rows=MEMBER_DIFF, effects=None, notes=""):
    eff = effects or ('<p class="fui-small">What applying does, as the gateway computed it: Zorgcentrum Beta-Testerveen takes part as the federation\'s sixth organisation. '
                      'It has no node yet, so no query reaches it until you add one on its page.</p>')
    return (revision_strip("draft", "Review your draft against revision 7", "Checked as ferrofed config check checks a file: no problems. Save it as revision 8, or save and apply it in one step.")
            + notes + card(f'What the draft changes <span class="mk-count">· {len(rows)} settings</span>', diff_table(rows) + eff))


def page_config_review():
    return form_page("Review the changes", [("Configuration", "configuration.html")],
                     "Each setting your draft changes, against the revision that runs. Every replica follows what you apply.",
                     review_body(), review_dock())


def apply_dialog(n="8", lines="", confirm_note=""):
    return (
        '<div role="dialog" aria-modal="true" aria-labelledby="apply-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="apply-title" class="fui-dialog-title">{icon("check", "18")} Apply revision {n}?</h2>'
        f'<p>{lines or "Every change is live at once: gateway-0 applies it, and gateway-1 follows within 5 seconds. Zorgcentrum Beta-Testerveen takes part; add its first node next."}</p>'
        + confirm_note
        + well('<p class="fui-small"><strong>Reason:</strong> Zorgcentrum Beta-Testerveen joins the federation in November</p><p class="fui-small fui-muted">Recorded under ferrofed::security with your name. Revision 7 stays in the history and comes back in one step with Roll back.</p>')
        + f'<div class="fui-dialog-actions">{button("secondary", "Go back", keys="Esc")}{button("primary", f"Apply revision {n}", "check")}</div></div>'
    )


def page_config_apply():
    html = form_page("Review the changes", [("Configuration", "configuration.html")],
                     "Each setting your draft changes, against the revision that runs.", review_body(), review_dock())
    return overlay(html, apply_dialog())


def overlay(html, dialog):
    return (html.replace('</div></body></html>', f'<div class="mk-scrim" aria-hidden="true"></div>{dialog}</div></body></html>')
            .replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1))




def page_config_applied():
    strip = revision_strip("ok", "Revision 8 runs", f"Applied {when('2026-10-21T09:58')} by you: “Zorgcentrum Beta-Testerveen joins the federation in November.” {replicas_inline()}",
                           button("ghost", "Roll back…", "rotate-ccw", small=True))
    t = toasts(toast("ok", "circle-check", "Revision 8 applied", "gateway-0 and gateway-1 run it. Recorded with your reason."))
    return config_page("settings", settings_panel(), strip=strip, toast_html=t, running="8")


def page_config_pending():
    card_ = pending_card("Revision 8 is saved, not applied", f"Saved {when('2026-10-21T09:51')} by you: “Zorgcentrum Beta-Testerveen joins the federation in November.” It changes 3 settings.",
                         [("done", "Saved", "by you, today 09:51"), ("skipped", "Approval", "not needed for an organisation"), ("current", "Apply", "yours, or any operator's"), ("waiting", "Running", "every replica follows")],
                         link_button("secondary", "Review the changes", "config-review.html", "git-compare", small=True) + link_button("primary", "Apply revision 8…", "config-apply.html", "check", small=True))
    return config_page("settings", settings_panel(), strip=running_strip() + card_)



# ---- credentials ----



def secret_now(ref, state="set"):
    if state == "unset":
        return f'<span class="fui-secret-now is-unset">{icon("lock", "12")}not set</span>'
    if state == "broken":
        return f'<span class="fui-secret-now is-broken">{icon("triangle-alert", "12")}set, but <code>{e(ref)}</code> cannot be read</span>'
    return f'<span class="fui-secret-now">{icon("lock", "12")}set, read from <code>{e(ref)}</code></span>'







GRANTS = [("basic", "Basic", "A user and a password (Basic authentication)"), ("oauth", "OAuth 2.0 client credentials", "A token for a signed client assertion or a secret"),
          ("exchange", "OAuth 2.0 token exchange", "The caller's token exchanged, with DPoP"), ("nuts", "Nuts", "A DPoP-bound token for the gateway's credentials"),
          ("fapi", "FAPI 2.0", "private_key_jwt, DPoP and authorization details"), ("tls", "TLS client certificate only", "Mutual TLS, no token")]


def grant_cards(chosen):
    cards = ""
    for v, w, sub in GRANTS:
        on = v == chosen
        cards += (f'<label class="fui-choice-card"><input type="radio" class="fui-sr-only" name="grant" value="{v}"{" checked" if on else ""} />'
                  f'<span class="fui-choice-card-text"><span class="fui-choice-card-title">{w}</span><span class="fui-choice-card-sub">{sub}</span></span>'
                  f'<span class="fui-choice-card-check" aria-hidden="true">{icon("check", "12")}</span></label>')
    return f'<fieldset class="fui-choice-cards mk-ff-grants"><legend class="fui-label">Grant</legend>{cards}</fieldset>'


def secret_field(key, mode="keep", now="/run/secrets/ferrofed/node-a-lab-client.pem", now_state="set", path="", error=None, idx=0):
    opts = [("keep", "Keep"), ("value", "Set a value"), ("file", "Read from a file"), ("clear", "Clear")]
    if now_state == "unset":
        opts = [("keep", "Leave unset"), ("value", "Set a value"), ("file", "Read from a file")]
    segs = "".join(f'<label class="fui-segment"><input type="radio" class="fui-sr-only" name="s{idx}"{" checked" if v == mode else ""} />{w}</label>' for v, w in opts)
    ctrl = ""
    inv = ' aria-invalid="true"' if error else ""
    er = f'<p class="fui-help fui-error">{icon("circle-alert", "14")} {error}</p>' if error else ""
    if mode == "value":
        ctrl = (f'<div class="fui-secret-control"><label for="sv{idx}" class="fui-label">Value, never shown again</label><input id="sv{idx}" type="password" class="fui-input" autocomplete="new-password" value="" {inv} />'
                f'{er}<p class="fui-help">Written to a new file in <code>config.secrets_dir</code>, readable by the gateway alone. The revision holds only its path.</p></div>')
    elif mode == "file":
        ctrl = (f'<div class="fui-secret-control"><label for="sf{idx}" class="fui-label">Absolute path of the file it is read from</label>{input_(f"sf{idx}", path, extra="spellcheck=\"false\" class=\"fui-input fui-mono\"" + inv)}{er}'
                '<p class="fui-help">A path on every replica\'s host; the gateway checks it can read it before it saves.</p></div>')
    elif mode == "clear":
        ctrl = f'<p class="fui-secret-clear">{icon("triangle-alert", "14")}The secret and its reference are removed. The endpoint is reached without it from the apply on.</p>'
    return (f'<div class="fui-secret"><div class="fui-secret-head"><span class="fui-label">{key}</span>{secret_now(now, now_state)}</div>'
            f'<div class="fui-segmented" role="radiogroup" aria-label="What to do with {key}">{segs}</div>{ctrl}</div>')


def cred_form(grant="oauth", errors=None):
    errors = errors or {}
    ep = field("c-ep", "Endpoint", select("c-ep", [("node-a-lab-pub", "node-a-lab-pub · node-a-lab, Ziekenhuis Voorbeeldstad"), ("node-f-fhir", "node-a-pub-2 · Ziekenhuis Voorbeeldstad")], "node-a-lab-pub"),
               help_="The endpoint is set: credentials are added from its row on the node.")
    head = card("Endpoint and grant", ep + grant_cards(grant))
    if grant == "oauth":
        te_err = errors.get("te")
        fields = ('<div class="fui-columns">'
                  + field("c-te", "Token endpoint", input_("c-te", "auth.cdr-a-lab.example.org/oauth2/token" if te_err else "https://auth.cdr-a-lab.example.org/oauth2/token", extra='spellcheck="false" class="fui-input fui-mono"' + (' aria-invalid="true"' if te_err else "")),
                          help_="Or the issuer, whose RFC 8414 metadata names it.", error=te_err)
                  + field("c-cid", "Client id", input_("c-cid", "ferrofed-gateway", extra='spellcheck="false" class="fui-input fui-mono"'))
                  + field("c-auth", "Client authentication", select("c-auth", [("pkj", "Signed client assertion (private_key_jwt)"), ("basic", "Client secret, Basic scheme"), ("post", "Client secret, in the body")], "pkj"))
                  + field("c-scope", "Scope", input_("c-scope", "system/aql-*.s system/composition-*.r", extra='spellcheck="false" class="fui-input fui-mono"'))
                  + "</div>")
        sec = secret_field("oauth.private_key", "file", now="", now_state="unset", path="/run/secrets/ferrofed/node-a-lab-client.pem", error=errors.get("key"))
        body = card("Grant settings", fields) + card("Secrets", '<p class="fui-small fui-muted">Write-only: a secret is set by a value or a file and never shown again.</p>' + sec)
    else:
        fields = ('<div class="fui-columns">'
                  + field("n-as", "Authorization server", input_("n-as", "https://nuts.cdr-a-lab.example.org/oauth2/ferrofed", extra='spellcheck="false" class="fui-input fui-mono"'))
                  + field("n-did", "DID", input_("n-did", "did:web:gateway.demo-regio.example.org", extra='spellcheck="false" class="fui-input fui-mono"'), help_="The gateway's own DID document is served at its did:web address.")
                  + field("n-scope", "Scope", input_("n-scope", "eOverdracht-sender", extra='spellcheck="false" class="fui-input fui-mono"'))
                  + field("n-kid", "Key id", input_("n-kid", "gateway-2026-10", extra='spellcheck="false" class="fui-input fui-mono"'))
                  + "</div>")
        sec = secret_field("nuts.dpop_key", "keep", now="/run/secrets/ferrofed/nuts-dpop.pem", idx=1)
        body = card("Grant settings", '<p class="fui-small fui-muted">Annex B §B.4: a DPoP-bound token for a Verifiable Presentation of the gateway\'s credentials, from the Nuts node.</p>' + fields) + card("Secrets", sec)
    return f'<div class="fui-stack">{head}{body}</div>'


def ep_page(title, sub, body, dock_html, strip=""):
    from common import page_heading as ph
    content = ph(title, subtitle=sub, crumbs=[("Organisations", "organisations.html"), ("Ziekenhuis Voorbeeldstad", "organisation.html"), ("node-a-lab", "node-endpoints.html")]) + f'<div class="fui-stack">{strip}{body}</div>'
    return shell(title, "orgs", content, dock_html=dock_html, org="zvs", node="node-a-lab")


def cred_strip():
    return revision_strip("draft", "Draft based on revision 7 · credentials for node-a-lab-pub", "This change takes a second operator's approval (config.approval: onward-credentials), and runs on trial for 2 minutes after the apply until you confirm it.")


def page_credential_new():
    body = f'<div class="fui-stack">{card("Endpoint and grant", field("c-ep", "Endpoint", select("c-ep", [("node-a-lab-pub", "node-a-lab-pub · node-a-lab, Ziekenhuis Voorbeeldstad")], "node-a-lab-pub"), help_="The endpoint is set: credentials are added from its row on the node.") + grant_cards(None))}</div>'
    return ep_page("Add credentials", "Choose how the gateway reaches the endpoint; the form shows that grant's fields.", body,
                     edit_dock(reason="", disabled=True, back=("Cancel", "node-endpoints.html")), cred_strip())


def page_credential_oauth():
    return ep_page("Add credentials", "OAuth 2.0 client credentials for node-a-lab-pub.", cred_form("oauth"),
                     edit_dock(reason="Credentials for the laboratory CDR, client registered at its authorization server", back=("Cancel", "node-endpoints.html")), cred_strip())


def page_credential_nuts():
    return ep_page("Edit the credentials of node-d-pub", "The Nuts grant toward Revalidatiecentrum Demo-West's authorization server.", cred_form("nuts"),
                     edit_dock(reason="New Nuts scope after the eOverdracht upgrade", back=("Cancel", "node-endpoints.html")),
                     revision_strip("draft", "Draft of node-d-pub's credentials · 1 change", "The DPoP key is kept as it is; only the scope changes."))


def page_credential_invalid():
    errs = {"te": "The token endpoint is not an absolute https URL. Outside development every outbound URL is https.",
            "key": "/run/secrets/ferrofed/node-a-lab-client.pem cannot be read on gateway-1: no such file."}
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 2 fields need a change before you can review</p>'
               '<ul><li><a href="#c-te">Token endpoint: not an absolute https URL</a></li><li><a href="#sf0">oauth.private_key: the file cannot be read on gateway-1</a></li></ul></div>')
    return ep_page("Add credentials", "OAuth 2.0 client credentials for node-a-lab-pub.", summary + cred_form("oauth", errs),
                     edit_dock(reason="Credentials for the laboratory CDR", disabled=True, back=("Cancel", "node-endpoints.html")))


def page_secret_states():
    def frame(title, line, html):
        return f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p><div>{html}</div></div>'
    frames = (
        frame("Set, kept", "the default when a secret is set: what is set, by reference, and Keep chosen", secret_field("password", "keep", now="/var/lib/ferrofed/secrets/node-b-1f3c.secret", idx=10))
        + frame("Set a value", "a password field the page never fills; the gateway writes it to a new file", secret_field("password", "value", now="/var/lib/ferrofed/secrets/node-b-1f3c.secret", idx=11))
        + frame("Read from a file", "an absolute path every replica can read", secret_field("oauth.private_key", "file", now="/run/secrets/ferrofed/node-a-client.pem", path="/run/secrets/ferrofed/node-a-client-2026-11.pem", idx=12))
        + frame("Clear", "the secret and its reference go; said as a warning before the save", secret_field("password", "clear", now="/var/lib/ferrofed/secrets/node-b-1f3c.secret", idx=13))
        + frame("Not set", "Leave unset in place of Keep, and no Clear", secret_field("oauth.client_secret", "keep", now="", now_state="unset", idx=14))
        + frame("Set, unreadable", "the reference exists but a replica cannot read it: said in the tone of a fault", secret_field("oauth.private_key", "keep", now="/run/secrets/ferrofed/node-c-client.pem", now_state="broken", idx=15))
        + frame("Value refused", "the store has no secrets_dir, so a value cannot be written; a reference still works", secret_field("password", "value", now="/var/lib/ferrofed/secrets/node-b-1f3c.secret", error="This gateway names no config.secrets_dir, so it cannot write a value: read it from a file.", idx=16))
    )
    rules = card("Rules", definitions([
        ("Never shown", "a secret's value never reaches a page, a revision, a diff, an export or a log; the page holds no value after a save"),
        ("Reference", "the revision names the file a secret is read from; a value becomes a new file, never an existing one, so a rollback still reads the old one"),
        ("Phone", "the four choices wrap under the label; the control under them takes the full width"),
    ]))
    content = page_heading("Secret field, states", subtitle="NEW (FerroFED), F3: the write-only secret of an onward credential or a binding.") + f'<div class="fui-stack mk-dock-sheet">{frames}{rules}</div>'
    return shell("Secret field states", "configuration", content)


CRED_DIFF = [("credentials.\"node-a-lab-pub\".oauth.token_endpoint", "", "https://auth.cdr-a-lab.example.org/oauth2/token", "live"),
             ("credentials.\"node-a-lab-pub\".oauth.client_id", "", "ferrofed-gateway", "live"),
             ("credentials.\"node-a-lab-pub\".oauth.client_auth", "", "private_key_jwt", "live"),
             ("credentials.\"node-a-lab-pub\".oauth.scope", "", "system/aql-*.s system/composition-*.r", "live"),
             ("credentials.\"node-a-lab-pub\".oauth.private_key_file", "", "/run/secrets/ferrofed/node-a-lab-client.pem", "live")]


def page_credential_review():
    notes = notice("info", "user-check", "<strong>A second operator approves this before it can apply.</strong> This deployment names <code>onward-credentials</code> in <code>config.approval</code>. Save it, then ask another operator to approve revision 9. After the apply it runs on trial for 2 minutes until you confirm it, or the gateway rolls it back.")
    body = review_body(CRED_DIFF, '<p class="fui-small">The gateway asked the token endpoint for metadata as a check: it answered, and names <code>private_key_jwt</code> among its methods. No token was requested.</p>', notes).replace("revision 8", "revision 9").replace("Save it as revision 9, or save and apply it in one step.", "Save it for a second operator's approval.")
    html = form_page("Review the changes", [("Organisations", "organisations.html"), ("Ziekenhuis Voorbeeldstad", "organisation.html"), ("node-a-lab", "node.html")],
                     "Each setting your draft changes, against the revision that runs. A secret shows by its reference.", body,
                     review_dock("Credentials for the laboratory CDR, client registered at its authorization server", approval=True))
    return html


def approval_card(viewer="author", approved=False):
    if approved:
        steps = [("done", "Saved", "by you, today 10:02"), ("done", "Approved", "by Demo Reviewer, today 10:09"), ("current", "Apply", "yours, or any operator's"), ("waiting", "Confirm", "within 2 min of the apply")]
        title, line = "Revision 9 is approved", "Demo Reviewer approved it at 10:09. Apply it, then confirm it once the gateway reaches node-a-lab-pub with it."
        acts = link_button("secondary", "Review the changes", "config-review-approval.html", "git-compare", small=True) + link_button("primary", "Apply revision 9…", "config-apply-trial.html", "check", small=True)
        return pending_card(title, line, steps, acts, glyph="badge-check")
    steps = [("done", "Saved", "by Demo Operator, today 10:02"), ("current", "Approval", "by an operator other than its author"), ("waiting", "Apply", "after the approval"), ("waiting", "Confirm", "within 2 min of the apply")]
    if viewer == "author":
        title, line = "Revision 9 waits for a second operator", "It changes the onward credentials of node-a-lab-pub, which this deployment approves twice. You saved it, so you cannot approve it: ask another operator."
        acts = button("secondary", "Copy a link for the approver", "copy", small=True) + button("primary", "Apply revision 9…", "check", small=True, attrs='disabled aria-describedby="ap-why"')
        extra = '<p id="ap-why" class="fui-small fui-muted">Apply opens once an operator other than you approves it.</p>'
    else:
        title, line = "Revision 9 asks for your approval", "Demo Operator saved it at 10:02: “Credentials for the laboratory CDR, client registered at its authorization server.” It changes the onward credentials of node-a-lab-pub."
        acts = link_button("secondary", "Review the changes", "config-review-approval.html", "git-compare", small=True) + link_button("primary", "Approve revision 9…", "config-approve-dialog.html", "user-check", small=True)
        extra = ""
    return pending_card(title, line, steps, acts, glyph="user-check", body=extra)


def page_config_awaiting_approval():
    return config_page("settings", settings_panel(),
                       strip=running_strip() + approval_card("author"))


def reviewer_page(overlay_=None):
    html = config_page("settings", settings_panel(),
                       strip=running_strip() + approval_card("reviewer"), overlay=overlay_)
    return html.replace("Demo Operator</span>", "Demo Reviewer</span>").replace("<strong>Demo Operator</strong>", "<strong>Demo Reviewer</strong>")


def page_config_approver():
    return reviewer_page()


def page_config_approve_dialog():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="ap-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="ap-title" class="fui-dialog-title">{icon("user-check", "18")} Approve revision 9?</h2>'
        '<p>You approve it as an operator other than its author. The approval names you, and Demo Operator can then apply it. Approving applies nothing.</p>'
        + well(definitions([("Changes", "the onward credentials of node-a-lab-pub: 5 settings, one secret by reference"), ("Saved by", "Demo Operator, today 10:02"),
                            ("Reason", "Credentials for the laboratory CDR, client registered at its authorization server")]))
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Approve revision 9", "user-check")}</div></div>'
    )
    return reviewer_page(dialog)


def page_config_approved():
    return config_page("settings", settings_panel(),
                       strip=running_strip() + approval_card(approved=True))


def page_config_apply_trial():
    note = notice("warn", "timer", "<strong>It runs on trial.</strong> Once applied, confirm it within 2 minutes, once a query reaches node-a-lab-pub with it. Unless you confirm it, the gateway rolls it back to revision 7 by itself.")
    d = apply_dialog("9", "The credentials of node-a-lab-pub are live at once on gateway-0; gateway-1 follows within 5 seconds.", note).replace(
        "Zorgcentrum Beta-Testerveen joins the federation in November", "Credentials for the laboratory CDR")
    return overlay(page_config_approved(), d)


def countdown_ring(left=102, total=120):
    r = 13
    circ = 2 * 3.14159 * r
    m, s = divmod(left, 60)
    return (f'<span class="fui-countdown fui-tone-warn" role="timer" aria-live="off"><svg viewBox="0 0 32 32" aria-hidden="true"><circle class="fui-countdown-track" cx="16" cy="16" r="{r}"/>'
            f'<circle class="fui-countdown-left" cx="16" cy="16" r="{r}" stroke-dasharray="{circ:.2f}" stroke-dashoffset="{circ * (1 - left / total):.2f}"/></svg>'
            f'<span class="fui-countdown-words"><strong>{m}:{s:02d}</strong><span>left, until 10:14:42</span></span></span>')


def page_config_trial():
    steps = [("done", "Saved", "by you, 10:02"), ("done", "Approved", "by Demo Reviewer, 10:09"), ("done", "Applied", "by you, 10:12:42"), ("current", "Confirm", "by 10:14:42, or it rolls back")]
    card_ = pending_card("Revision 9 runs on trial", "It changed the onward credentials of node-a-lab-pub. Confirm it once the gateway works with it; unless an operator confirms it by 10:14:42, the gateway rolls it back to revision 7.",
                         steps, countdown_ring() + button("secondary", "Roll back now", "rotate-ccw", small=True) + button("primary", "Confirm revision 9", "check", small=True), tone="warn", glyph="timer",
                         body='<p class="fui-small">Check it first: <a href="query.html">run a query</a> with node-a-lab-pub in scope. The last one at 10:13:20 reached it: <strong>active, 41 ms</strong>.</p>')
    return config_page("settings", settings_panel(),
                       strip=card_, running="9")


def page_config_confirmed():
    strip = revision_strip("ok", "Revision 9 runs, confirmed", f"Confirmed by you at 10:13:58, 44 seconds before the deadline. The gateway no longer rolls it back. {replicas_inline()}", button("ghost", "Roll back…", "rotate-ccw", small=True))
    t = toasts(toast("ok", "circle-check", "Revision 9 confirmed", "It stays running. Recorded under ferrofed::security."))
    return config_page("settings", settings_panel(),
                       strip=strip, toast_html=t, running="9")


def page_config_rolled_back():
    steps = [("done", "Saved", "by Demo Operator, 10:02"), ("done", "Approved", "by Demo Reviewer, 10:09"), ("done", "Applied", "10:12:42"), ("failed", "Not confirmed", "the deadline passed at 10:14:42")]
    card_ = pending_card("Revision 9 was rolled back: no one confirmed it in time", "The gateway saved revision 7's content as revision 10 and applied it at 10:14:42, with the reason “revision 9 was not confirmed by its deadline”. node-a-lab-pub is reached without credentials again.",
                         steps, link_button("secondary", "Compare 9 with 10", "configuration-compare.html", "git-compare", small=True) + button("primary", "Edit a new draft from revision 9", "pencil", small=True), tone="bad", glyph="rotate-ccw")
    return config_page("settings", settings_panel(), strip=card_, running="10")


# ---- bindings ----













# ---- settings ----

SETTINGS = [
    ("Federation behaviour", "[federation]", [
        ("per_node_timeout_ms", "Per node", "2500", "ms", "live", "how long one node may take"),
        ("overall_timeout_ms", "Overall", "4000", "ms", "live", "how long one query may take, every node together"),
        ("timeout_policy", "Timeout policy", "abandon-and-mark", "", "live", "a late node is named time-out and left out"),
        ("node_selection", "Node selection", "localized", "", "restart", "ask only the nodes that hold the patient"),
        ("completeness", "Best effort", "offered", "", "live", "a caller may ask for a partial answer"),
    ]),
    ("Fan-out and paging", "[federation]", [
        ("stored_query_fanout", "Stored-query fan-out", "off", "", "restart", "a stored version is not pushed to every node"),
        ("template_fanout", "Template upload fan-out", "off", "", "restart", "a template goes to one chosen node"),
        ("max_fetch", "Largest fetch", "1000", "rows", "live", "the most rows one answer carries"),
        ("offset_strategy", "Offset strategy", "bounded", "", "live", "offset is applied per node within a bound"),
    ]),
    ("Consent disclosure", "[federation.consent]", [
        ("disclose", "Disclose exclusions", "true", "", "live", "an excluded node is named consent-denied; off, not-resolved with one neutral text (A53)"),
    ]),
    ("Stored-query registry", "[stored_queries]", [
        ("backend", "Backend", "postgres", "", "restart", "one store every replica shares"),
        ("url_file", "Connection", "/run/secrets/ferrofed/stored-queries-url", "", "restart", "a secret, by reference"),
    ]),
    ("Access log", "[access_log]", [
        ("retention.default", "Retention", "3", "years", "live", "at least three, by category and origin"),
        ("emergency_purpose", "Emergency purposes", "ETREAT", "", "live", "purposes of use that mark an emergency access"),
    ]),
    ("FHIR face", "[fhir]", [
        ("fhir.base", "Base", "/fhir", "", "restart", "the Annex II 2.1 face, outside {base}"),
        ("fhir.document_url_lifetime_secs", "Document URL lifetime", "300", "s", "live", "how long a listed document's URL stays valid"),
    ]),
]


def settings_panel(mode="view", draft=None, errors=None, readonly=False):
    draft = draft or {}
    errors = errors or {}
    cards = ""
    for title, tbl, rows in SETTINGS:
        trs = ""
        for key, label, v, unit, fx, means in rows:
            nv = draft.get(key, v)
            if mode == "edit" and key != "url_file":
                changed = nv != v
                inv = ' aria-invalid="true"' if key in errors else ""
                cls = "fui-input" + (" fui-config-changed" if changed else "")
                was = f'<span class="fui-config-was">was {e(v)}</span>' if changed else ""
                er = f'<span class="fui-config-error">{errors[key]}</span>' if key in errors else ""
                if v.isdigit():
                    cell = f'<span class="fui-config-input"><input class="{cls}" id="cfg-{key}" value="{e(nv)}" inputmode="numeric" aria-label="{e(label)}"{inv} />{was}{er}</span>'
                else:
                    cell = select(f"cfg-{key}", [(v, v)], v, cls="fui-select fui-select-small mk-ff-cfg-select")
            else:
                cell = f"<code>{e(nv)}</code>" if key == "url_file" else f"{e(nv)}{' ' + unit if unit else ''}"
            trs += (f'<tr><th scope="row">{label}<span class="fui-small fui-muted mk-key"><code>{key}</code></span></th>' + td("Value", cell, "fui-num" if v.isdigit() else "")
                    + td("Means", means) + td("Takes effect", effect(fx)) + "</tr>")
        t = table(title, th("Setting") + th("Value") + th("Means") + th("Takes effect"), [trs])
        cards += card(f'{title} <span class="mk-count">· <code>{tbl}</code></span>', t)
    act = "" if (readonly or mode == "edit") else button("primary", "Edit", "pencil", small=True, keys="e")
    head = panel_head("Settings", "The deployment-wide settings no other page owns. Organisations, nodes, endpoints, credentials and services are set on their own pages. Most change at once; a few take the next start, as each row says.", act)
    return head + f'<div class="fui-panel-body mk-ff-sections">{cards}</div>'


def page_config_settings():
    return config_page("settings", settings_panel())


SET_DRAFT = {"per_node_timeout_ms": "3000", "overall_timeout_ms": "5000"}


def page_config_settings_edit():
    strip = revision_strip("draft", "Draft based on revision 7 · 2 changes", "Nothing runs until you review, save and apply it.")
    d = edit_dock("Review the changes", "node-b's link stays slow; give every node 3 s", back=("Discard the draft", "configuration.html"))
    return config_page("settings", settings_panel("edit", SET_DRAFT), strip=strip, dock_html=d)


def page_config_settings_invalid():
    draft = {"per_node_timeout_ms": "6000", "overall_timeout_ms": "5000"}
    err = {"per_node_timeout_ms": "Must not exceed Overall (5000 ms)."}
    strip = revision_strip("draft", "Draft based on revision 7 · 2 changes", "Nothing runs until you review, save and apply it.")
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 1 value needs a change before you can review</p>'
               '<ul><li><a href="#cfg-per_node_timeout_ms">Per node: must not exceed Overall (5000 ms)</a></li></ul></div>')
    d = edit_dock("Review the changes", "node-b's link stays slow; give every node 3 s", disabled=True, back=("Discard the draft", "configuration.html"))
    return config_page("settings", settings_panel("edit", draft, err), strip=strip + summary, dock_html=d)


def page_config_restart():
    strip = revision_strip("warn", "Revision 8 runs, and one setting waits for a restart",
                           "node_selection = \"all\" takes effect at the next start of each replica. Until then every replica selects nodes as revision 7 did (localized).",
                           pill("warn", "pending restart"))
    rows = [("federation.node_selection", "localized", "all", "restart")]
    below = card("Waiting for the restart", diff_table(rows, right="Revision 8") + '<p class="fui-small fui-muted">Restart each replica as usual; the dashboard and this strip say when both run it. Everything else in revision 8 runs already.</p>')
    return config_page("settings", below + settings_panel(), strip=strip, running="8")


def page_config_conflict():
    strip = revision_strip("bad", "Demo Reviewer applied revision 8 while you edited",
                           f"Applied {when('2026-10-21T09:51')}: “Lower the overall budget for the pilot.” Your draft is kept and was not saved (409 revision-stale). Base it on revision 8, or discard it.",
                           button("primary", "Rebase my draft on revision 8", "git-compare", small=True) + button("secondary", "Discard my draft", "x", small=True))
    body = [f'<tr><th scope="row"><code>{k}</code></th>' + td("Revision 7, your base", a) + td("Revision 8, now running", b) + td("Your draft", c) + td("", s) + "</tr>"
            for k, a, b, c, s in [
                ("federation.per_node_timeout_ms", "2500", "2500", "3000", badge("accent", "yours alone")),
                ("federation.overall_timeout_ms", "4000", "3500", "5000", badge("bad", "both changed")),
            ]]
    t = card("Three revisions side by side", table("Your draft against revision 8", th("Setting") + th("Revision 7, your base") + th("Revision 8, now running") + th("Your draft") + th('<span class="fui-sr-only">Whose</span>'), body)
             + '<p class="fui-small fui-muted">Rebasing keeps your per-node change. Both of you changed the overall budget, so you choose the value before you review the result.</p>')
    return config_page("settings", settings_panel(), strip=strip + t, running="8")


def _old_rollback():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="rb-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="rb-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Roll back revision 7?</h2>'
        '<p>The content of revision 6, which ran before it, is saved as revision 8 and applied in one step: the per-node timeout goes back to 2,000 ms. A rollback takes no approval and no confirmation; a second rollback returns to revision 7.</p>'
        + field("rb-reason", "Reason", textarea("rb-reason", "Queries to node-b still time out; back to the earlier budget"), help_="Required. Kept with the revision.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Roll back to revision 6", "rotate-ccw")}</div></div>'
    )
    return config_page("settings", settings_panel(), overlay=dialog)


def page_config_replica_refused():
    strip = revision_strip("warn", "Revision 8 runs on gateway-0; gateway-1 refused it",
                           "gateway-1 cannot read /etc/ferrofed/registry.json, which revision 8 names, so it keeps running revision 7. Fix the file on that host, then reload it or apply a newer revision. The alert FerroFEDConfigurationDrift fires after 10 minutes.",
                           replicas_inline(refused=True))
    return config_page("settings", settings_panel(), strip=strip, running="8")


# ---- history, compare, import, export ----

ORIGIN_TONE = {"operator": "neutral", "file": "info", "rollback": "warn", "import": "accent", "gateway": "bad"}


def history_rows(items):
    rows = []
    for n, origin, by, at, reason in items:
        st = pill("ok", "running") if n == "7" else '<span class="fui-small fui-muted mk-nowrap">applied before</span>'
        acts = (f'<a class="fui-icon-button" href="configuration-compare.html" aria-label="Compare revision {n} with running" title="Compare with running">{icon("git-compare", "16")}</a>' if n != "7" else "") + icon_button("download", f"Export revision {n} as TOML")
        rows.append(f'<tr><th scope="row" class="mk-nowrap">Revision {n}</th>' + td("Origin", badge(ORIGIN_TONE[origin], origin)) + td("By", e(by), "mk-nowrap") + td("When", when(at)) + td("Reason", e(reason))
                    + td("State", st) + td("", f'<span class="fui-cluster mk-nowrap">{acts}</span>', "fui-table-actions") + "</tr>")
    return rows


def history_tools(origins=("operator", "file", "rollback", "import", "gateway"), picker_html=None):
    import tpl
    word = None if len(origins) == 5 else ", ".join(origins)
    rng = (f'<span class="fui-anchor">{button("secondary", "2026-10-12 to 2026-10-19", "calendar", attrs="aria-haspopup=" + chr(34) + "dialog" + chr(34) + " aria-expanded=" + chr(34) + "true" + chr(34))}'
           f'<span class="mk-range-pop">{picker_html}</span></span>') if picker_html else button("secondary", "Any date", "calendar")
    return '<div class="fui-panel-body">' + tpl.toolbar("Reason, who or a revision number", "", [tpl.filter_button("Origin", word), rng]) + "</div>"


def history_panel(items=REVISIONS, tools=None, count=None):
    head = (f'<div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">Every revision, who made it and why, newest first{count or ""}. Roll back returns to what ran before.</p>'
            f'<div class="fui-actions">{button("secondary", "Roll back…", "rotate-ccw", small=True)}</div></div>')
    return head + (tools or history_tools()) + table("Revisions", th("Revision") + th("Origin") + th("By") + th("When") + th("Reason") + th("State") + th('<span class="fui-sr-only">Actions</span>'), history_rows(items))


def page_configuration_history():
    return config_page("history", history_panel())


def page_configuration_history_filtered():
    items = [r for r in REVISIONS if r[1] in ("rollback", "import")]
    import tpl
    chips = tpl.active(["Origin: rollback, import"])
    return config_page("history", history_panel(items, history_tools(("rollback", "import")), count=" · 2 of 7") + f'<div class="fui-panel-body">{chips}</div>')


def page_configuration_history_range():
    rng = (D(2026, 10, 12), D(2026, 10, 19))
    months = (month_grid(2026, 9, rng=rng, max_d=PICK_TODAY, title_btn=False, nav="prev") + month_grid(2026, 10, rng=rng, max_d=PICK_TODAY, title_btn=False, nav="next"))
    p = picker("Revisions between", f'<div class="fui-datepicker-pair">{months}</div>', presets=[("Today", False), ("Last 7 days", False), ("Last 30 days", False), ("This month", True)],
               status="2026-10-12 to 2026-10-19 · 8 days.")
    ph = p
    return config_page("history", history_panel(tools=history_tools(picker_html=ph)))


def page_configuration_compare():
    rows = [("federation.per_node_timeout_ms", "2000", "2500", "live"), ("registry.nodes[node-d]", "", "node-d · cdr-d.example.org · 1 endpoint", "live")]
    body = [f'<tr><th scope="row"><code class="fui-break">{k}</code></th>' + td("Revision 5", change_del(a) if a else '<span class="mk-dash">not set</span>') + td("Revision 7, running", change_ins(b)) + td("Takes effect", effect(fx)) + "</tr>" for k, a, b, fx in rows]
    t = table("Revision 5 against revision 7", th("Setting") + th("Revision 5") + th("Revision 7, running") + th("Takes effect"), body)
    content = (page_heading("Revision 5 against revision 7", subtitle="Each setting that differs, with how it took effect. Every other setting is the same.", crumbs=[("Configuration", "configuration.html"), ("History", "configuration-history.html")])
               + '<div class="fui-stack">' + card("2 settings differ", t)
               + card("Revision 5", definitions([("Origin", badge("warn", "rollback")), ("By", "Demo Operator"), ("When", when("2026-10-15T17:30")), ("Reason", "Roll back revision 4: the NVI rejected the new client key")]),
                      actions=button("secondary", "Export revision 5 as TOML", "download", small=True)) + "</div>")
    return shell("Configuration", "configuration", content)


IMPORT_TOML = '[federation]\nper_node_timeout_ms = 3000\noverall_timeout_ms = 5000\n\n[credentials."node-d-pub".nuts]\nscope = "eOverdracht-sender"\n'


def import_page(toml=IMPORT_TOML, saved=True, refused=None):
    form = field("imp-toml", "TOML", textarea("imp-toml", toml, rows=9, extra='spellcheck="false"', cls="fui-textarea mk-code"),
                 help_="Paste what an export wrote, or a file kept in version control. Saved as a dry-run revision and never applied by the import. A key of the core or a secret written inline is refused, naming the key.")
    parts = card("The TOML", form)
    if refused:
        parts += refusal("The import was refused (422 configuration-invalid): nothing was saved.", issues=refused)
        dk = dock(dock_field("imp-reason", "Reason", "Budgets and Nuts scope as merged in the config repository", "Why this changes"),
                  link_button("secondary", "Cancel", "configuration.html") + button("primary", "Import as a dry run", "upload"))
    elif saved:
        rows = [("federation.per_node_timeout_ms", "2500", "3000", "live"), ("federation.overall_timeout_ms", "4000", "5000", "live"), ("credentials.\"node-d-pub\".nuts.scope", "eOverdracht", "eOverdracht-sender", "live")]
        parts += revision_strip("draft", "Saved as revision 8, a dry run", "Checked: no problems. It changes 3 settings against revision 7, one of them a credential, so it takes a second operator's approval before you can apply it.")
        parts += card("What revision 8 changes", diff_table(rows, right="Revision 8"))
        dk = dock(dock_field("imp-reason", "Reason", "Budgets and Nuts scope as merged in the config repository", "Why this changes"),
                  button("secondary", "Discard revision 8", "x") + button("primary", "Ask for approval", "user-check"))
    content = (page_heading("Import a configuration", subtitle="From a file kept in version control: paste the TOML, read what it changes, then apply it as any revision.",
                            crumbs=[("Configuration", "configuration.html")]) + f'<div class="fui-stack">{parts}</div>')
    return shell("Import a configuration", "configuration", content, dock_html=dk)


def page_configuration_import():
    return import_page()


def page_configuration_import_refused():
    toml = IMPORT_TOML + '\n[auth]\nproduct_scope = "__ferrofed.admin"\n\n[credentials."node-b-pub"]\npassword = "••••••••"\n'
    return import_page(toml, saved=False, refused=[("bad", "error", "auth.product_scope", "a key of the core: [auth] stays in ferrofed.toml, so no revision can change who may call."),
                                                    ("bad", "error", "credentials.\"node-b-pub\".password", "a secret written inline: give its file as password_file. The value was not read into any log.")])


def page_configuration_export_done():
    t = toasts(toast("ok", "download", "Export ready", "ferrofed-revision-7.toml · 4 KB · every secret as its file reference; the core is not part of it.",
                     action=button("secondary", "Download", "download", small=True)))
    return config_page("settings", settings_panel(), toast_html=t)


# ---- file mode ----

def file_strip():
    return revision_strip("running", "ferrofed.toml, read at the last reload",
                          f"Read {when('2026-10-21T06:00')} after a change to the file (config repository commit 4f2a9c1). {replicas_inline()}",
                          mode_help("file"))


def page_configuration_file_mode(overlay_=None):
    return config_page("settings", mode="file", overlay=overlay_)


def page_configuration_file_mode_dialog():
    return page_configuration_file_mode(overlay_=mode_dialog("file"))



def page_configuration_file_mode_history():
    items = [("4", "file", "ferrofed.toml", "2026-10-21T06:00", "Read after a change to the file · commit 4f2a9c1"),
             ("3", "file", "ferrofed.toml", "2026-10-14T06:00", "Read after a change to the file · commit 9b07e31"),
             ("2", "file", "ferrofed.toml", "2026-10-13T06:00", "Read at the start")]
    rows = []
    for n, origin, by, at, reason in items:
        st = pill("ok", "running") if n == "4" else '<span class="fui-small fui-muted">read before</span>'
        acts = (link_button("ghost", "Compare with running", "configuration-compare.html", "git-compare", small=True) if n != "4" else "") + icon_button("download", f"Export reading {n} as TOML")
        rows.append(f'<tr><th scope="row">Reading {n}</th>' + td("From", f"<code>{by}</code>") + td("When", when(at)) + td("Note", e(reason)) + td("State", st) + td("", f'<span class="fui-cluster mk-nowrap">{acts}</span>', "fui-table-actions") + "</tr>")
    head = panel_head("Readings", "Each reading of the file, newest first. To go back, revert the file and reload: in file mode the console changes nothing.")
    panel = head + table("Readings", th("Reading") + th("From") + th("When") + th("Note") + th("State") + th('<span class="fui-sr-only">Actions</span>'), rows)
    return config_page("history", panel, mode="file")


# ---- storing a stored-query version (the Stored queries page) ----

def stored_new_page(overlay_=None, refused=False):
    from pg_query import editor
    aql = ("SELECT\n  c/context/start_time/value AS recorded,\n  o/data[at0001]/events[at0002]/data[at0003]/items[at0004]/value/magnitude AS weight\n"
           "FROM EHR e\n  CONTAINS COMPOSITION c\n  CONTAINS OBSERVATION o[openEHR-EHR-OBSERVATION.body_weight.v2]\nWHERE e/ehr_status/subject/external_ref/id/value = $patient")
    name_err = "org.demo-regio::vital_signs 1.1.0 is held already: a held version never changes. Store it as 1.2.0." if refused else None
    form = (card("The version", '<div class="fui-columns">'
                 + field("sn-name", "Qualified name", input_("sn-name", "org.demo-regio::vital_signs" if refused else "org.demo-regio::body_weight", extra='spellcheck="false" class="fui-input fui-mono"' + (' aria-invalid="true"' if refused else "")),
                         help_="A namespace and a name, such as org.demo-regio::vital_signs.", error=name_err)
                 + field("sn-version", "Version", input_("sn-version", "1.1.0" if refused else "1.0.0", extra='spellcheck="false" class="fui-input fui-mono"'), help_="major.minor.patch.")
                 + "</div>"
                 + f'<div class="fui-field"><label for="sn-aql" class="fui-label">AQL</label><div class="fui-editor-bar"><span>AQL · 7 lines · names $patient</span></div>{editor(aql, "sn-aql")}'
                 '<p class="fui-help">Checked as the gateway parses it. A patient is named as a parameter, never in the text.</p></div>'))
    content = (page_heading("Store a new version", subtitle="Stored at once in the stored-query store, which every replica serves. It is data, not configuration: no revision, and it can never be changed or removed afterwards.",
                            crumbs=[("Stored queries", "stored-queries.html")]) + f'<div class="fui-stack">{form}</div>')
    dk = dock(dock_field("sn-reason", "Reason", "Body weight for the dietetics pilot, ticket OPS-131", "Why this version is stored"),
              link_button("secondary", "Cancel", "stored-queries.html") + button("primary", "Store the version…", "database"))
    return shell("Store a new version", "stored", content, dock_html=dk, overlay=overlay_)


def page_stored_new():
    return stored_new_page()


def page_stored_new_confirm():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="sn-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="sn-title" class="fui-dialog-title">{icon("database", "18")} Store body_weight 1.0.0?</h2>'
        '<p>The gateway stores it at once and every replica serves it. A stored version cannot be changed or removed afterwards: a correction is version 1.0.1.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Go back", keys="Esc")}{button("primary", "Store the version", "database")}</div></div>'
    )
    return stored_new_page(dialog)


def page_stored_new_held():
    return stored_new_page(refused=True)


def page_lifecycle_states():
    def frame(title, line, html):
        return f'<div class="fui-card mk-state-frame"><p class="mk-state-label"><span>{title}</span><span>{line}</span></p><div>{html}</div></div>'
    frames = (
        frame("Saved, no approval needed", "an organisation change: approval is not needed, so its step is a dash",
              lifecycle([("done", "Saved", "by you, 09:51"), ("skipped", "Approval", "not needed"), ("current", "Apply", "yours, or any operator's"), ("waiting", "Running", "every replica follows")]))
        + frame("Waiting for approval", "the author sees whose turn it is",
                lifecycle([("done", "Saved", "by Demo Operator, 10:02"), ("current", "Approval", "another operator"), ("waiting", "Apply", "after the approval"), ("waiting", "Confirm", "within 2 min")]))
        + frame("On trial", "applied, the confirmation is the current step",
                lifecycle([("done", "Saved", "10:02"), ("done", "Approved", "Demo Reviewer, 10:09"), ("done", "Applied", "10:12:42"), ("current", "Confirm", "by 10:14:42")]))
        + frame("Confirmed", "every step done",
                lifecycle([("done", "Saved", "10:02"), ("done", "Approved", "10:09"), ("done", "Applied", "10:12:42"), ("done", "Confirmed", "by you, 10:13:58")]))
        + frame("Rolled back", "the failed step in the bad tone, with its cause",
                lifecycle([("done", "Saved", "10:02"), ("done", "Approved", "10:09"), ("done", "Applied", "10:12:42"), ("failed", "Not confirmed", "deadline 10:14:42")]))
    )
    keys = card("Rules", definitions([
        ("Steps", "Saved, Approval, Apply, Confirm: the ones a revision does not take are drawn as not needed, never hidden, so every revision reads the same way"),
        ("Words first", "each step says its state in words and a time; the dot and the rule only repeat it"),
        ("Phone", "the steps stack, the rule runs down the dots"),
    ]))
    content = page_heading("Revision lifecycle, states", subtitle="NEW (FerroFED), F1: where a saved revision stands and whose turn it is.") + f'<div class="fui-stack mk-dock-sheet">{frames}{keys}</div>'
    return shell("Revision lifecycle states", "configuration", content)


# ================================================================
# Configuration, redone on the setting-rows pattern (R26, R27): a section
# sub-navigation, one readable column of section cards made of rows, a
# compact revision line, and History as the last section.
# ================================================================

TABS = [("federation", "Federation behaviour", "configuration.html"), ("fanout", "Fan-out and paging", "configuration-fanout.html"),
        ("consent", "Consent", "configuration-consent.html"), ("stored", "Stored queries", "configuration-stored.html"),
        ("access", "Access log", "configuration-access.html"), ("fhir", "FHIR face", "configuration-fhir.html"),
        ("history", "History", "configuration-history.html")]


def compact_strip(n="7", at="2026-10-19", by="Demo Operator", tone="ok", word="in step", tip="gateway-0 runs revision 7 · gateway-1 runs revision 7"):
    return (f'<div class="fui-revision mk-revision-compact">{icon("file-text", "16")}<p><strong>Revision {n}</strong> · applied {at} by {by} · '
            f'<span class="fui-pill fui-tone-{tone}" title="{e(tip)}">{word}</span></p></div>')


def key_hint(k, show=False):
    return f'<code class="mk-key-hint{" is-shown" if show else ""}" title="Copy the key">{e(k)}{icon("copy", "12")}</code>'


def srow(label, desc, value, key, restart=False, control=None, edit=False, error=None, changed=None, hint=False):
    nxt = f'<span class="fui-effect fui-effect-restart" title="Takes effect at the next start of each replica">{icon("refresh-cw", "12")}Next start</span>' if restart else ""
    ctl = control if edit and control else f'<span class="fui-setting-value">{value}</span>'
    er = f'<p class="fui-help fui-error mk-row-error">{icon("circle-alert", "14")} {error}</p>' if error else ""
    was = f'<span class="fui-config-was">was {e(changed)}</span>' if changed else ""
    return (f'<div class="fui-setting-row{" is-changed" if changed else ""}{" is-invalid" if error else ""}{" mk-hover" if hint else ""}"><div class="fui-setting-text"><span class="fui-setting-label">{label}{key_hint(key, hint)}</span>'
            f'<p class="fui-setting-desc">{desc}</p>{er}</div><div class="fui-setting-control">{ctl}{nxt}{was}</div></div>')


def dur(id_, v, unit, changed=False, invalid=False):
    cls = "fui-input" + (" fui-config-changed" if changed else "")
    inv = ' aria-invalid="true"' if invalid else ""
    return f'<div class="fui-input-group mk-dur"><input id="{id_}" class="{cls}" value="{v}" inputmode="numeric"{inv} /><span class="fui-input-suffix">{unit}</span></div>'


def seg(name, opts, cur):
    return '<div role="radiogroup" class="fui-segmented">' + "".join(f'<label class="fui-segment"><input type="radio" class="fui-sr-only" name="{name}"{" checked" if v == cur else ""} />{w}</label>' for v, w in opts) + "</div>"


def sw(id_, on, label):
    return f'<input type="checkbox" class="fui-switch" id="{id_}"{" checked" if on else ""} aria-label="{e(label)}" />'


LIVE = "Changes apply at once unless a row says Next start."


def tab_sections(edit=False, readonly=False, hint=False):
    """Each tab's (intro, rows). edit: Federation behaviour in its draft."""
    fe = edit
    fed = (srow("Per-node timeout", "How long one node may take, 200 to 10,000 ms.", "2,500 ms", "federation.per_node_timeout_ms",
                control=dur("cfg-pn", "3000", "ms", changed=True), edit=fe, changed="2,500 ms" if fe else None, hint=hint and not fe)
           + srow("Overall timeout", "How long a whole query may take, 500 to 30,000 ms, and at least the per-node timeout.", "4,000 ms", "federation.overall_timeout_ms",
                  control=dur("cfg-ov", "2000", "ms", invalid=True), edit=fe, error="Must be at least the per-node timeout (3,000 ms)." if fe else None)
           + srow("Timeout policy", "What happens to a node past its budget.", "abandon and mark", "federation.timeout_policy",
                  control=seg("tp", [("abandon", "Abandon and mark"), ("wait", "Wait for all")], "abandon"), edit=fe)
           + srow("Node selection", "Which nodes a patient's query asks.", "localized", "federation.node_selection", restart=True,
                  control=seg("ns", [("localized", "Localized"), ("all", "All nodes")], "localized"), edit=fe)
           + srow("Best effort", "A caller may ask for a partial answer when a node does not answer.", "offered", "federation.completeness",
                  control=sw("cfg-be", True, "Best effort"), edit=fe))
    fan = (srow("Stored-query fan-out", "Push each stored version to every node as well.", "off", "federation.stored_query_fanout", restart=True)
           + srow("Template upload fan-out", "Send an uploaded template to every node, not one chosen node.", "off", "federation.template_fanout", restart=True)
           + srow("Largest fetch", "The most rows one answer carries, 1 to 10,000.", "1,000 rows", "federation.max_fetch")
           + srow("Offset strategy", "How an offset is applied across nodes.", "bounded, per node", "federation.offset_strategy"))
    con = (srow("Disclose exclusions", "Name a node consent excluded as consent-denied; off, it is not-resolved with one neutral text, as Regulation (EU) 2025/327 Art 8 may require.", "on", "federation.consent.disclose")
           + srow("In an emergency", "Whether the consent pre-filter still applies.", '<a href="service-connection.html">set on Mitz</a> · apply', "federation.consent.emergency"))
    sto = (srow("Backend", "Where the stored queries are kept.", "PostgreSQL, shared by every replica", "stored_queries.backend", restart=True)
           + srow("Connection", "A secret, by reference.", f'<span class="fui-secret-now">{icon("lock", "12")}Set · <code>/run/secrets/ferrofed/stored-queries-url</code></span>{button("ghost", "Replace", small=True) if not readonly else ""}', "stored_queries.url_file", restart=True))
    acc = (srow("Retention", "Years each access record is kept, at least 3.", "3 years", "access_log.retention.default")
           + srow("Emergency purposes", "Purposes of use that mark an emergency access.", '<span class="fui-tag">ETREAT</span>', "access_log.emergency_purpose")
           + srow("Categories", "Template and archetype ids mapped to the Art 14(1) categories.", f'42 entries {button("ghost", "View", small=True)}', "access_log.categories"))
    fh = (srow("Base", "Where the exchange-format face is served, outside the ITS-REST base.", "<code>/fhir</code>", "fhir.base", restart=True)
          + srow("Document URL lifetime", "How long a listed document's URL stays valid, 60 to 3,600 s.", "300 s", "fhir.document_url_lifetime_secs")
          + srow("Received categories", "The documents the face accepts.", '<span class="fui-tag">Patient summaries</span> <span class="fui-tag">Laboratory reports</span>', "fhir.receive"))
    return {
        "federation": ("How a query fans out to the nodes and when it gives up. " + LIVE, fed),
        "fanout": (LIVE, fan),
        "consent": ("What callers learn about consent. " + LIVE, con),
        "stored": ('The store the stored queries are kept in; the queries themselves are on <a href="stored-queries.html">Stored queries</a>.', sto),
        "access": ("How each access record is classified and kept. " + LIVE, acc),
        "fhir": ("The Annex II 2.1 face that serves and receives the exchange format. " + LIVE, fh),
    }


def config_page(tab, panel=None, mode="console", strip=None, overlay=None, toast_html="", menu_open=False, dock_html="", running="7", body=None, edit=None, top="", drafts=(), status_html=None, hint=False):
    import tpl
    tab = "federation" if tab == "settings" else tab
    readonly = mode == "file"
    status = compact_strip(running) if mode == "console" else compact_strip(running, "2026-10-21", "a reading of ferrofed.toml")
    if strip and ("fui-tone-ok" in strip or "fui-tone-warn" in strip or "fui-tone-bad" in strip) and "fui-pending" not in strip:
        status = strip
    elif strip and "fui-pending" in strip:
        top = strip + top
    if status_html is not None:
        status = status_html
    if tab != "history" and panel and body is None:
        top = top + panel
    if tab == "history":
        inner = panel if body is None else body
        inner = f'<div class="fui-panel">{inner}</div>'
    elif body is not None:
        inner = body
    else:
        intro, rows = tab_sections(edit=bool(edit), readonly=readonly, hint=hint)[tab]
        if edit:
            act = '<span class="fui-pill fui-tone-accent">editing</span>'
        elif readonly:
            act = ""
        else:
            act = link_button("secondary", "Edit", "config-settings-edit.html" if tab == "federation" else "#", "pencil", small=True)
        inner = tpl.settings_panel(rows, intro, act)
    head = config_heading(mode, menu_open, running)
    tabs_html = tpl.settings_tabs("Configuration sections", TABS, tab, inner, drafts=drafts, many=True)
    content = head + f'<div class="fui-stack">{status}{top}{tabs_html}</div>'
    return shell("Configuration", "configuration", content, overlay=overlay, toast_html=toast_html, dock_html=dock_html)


def settings_panel(mode="view", draft=None, errors=None, readonly=False):
    return ""


def config_heading(mode="console", menu_open=False, running="7"):
    from common import action_group
    if mode == "console":
        ag = action_group(mode_help("console"), [menu_item(f"Export revision {running} as TOML", "download"), menu_item("Import TOML…", "upload", href="configuration-import.html")],
                          open_=menu_open, label="More configuration actions")
        sub = "The deployment-wide settings; everything else is set on its own page."
    else:
        ag = action_group(mode_help("file"), [menu_item("Export the running settings as TOML", "download")], open_=menu_open, label="More configuration actions")
        sub = "What ferrofed.toml sets, as the gateway read it at the last reload."
    return page_heading("Configuration", actions=ag, subtitle=sub, badge=mode_badge(mode))


def _tab_page(key):
    def f():
        return config_page(key)
    return f


def page_configuration_key_hint():
    return config_page("federation", hint=True)


page_configuration_fanout = _tab_page("fanout")
page_configuration_consent = _tab_page("consent")
page_configuration_stored = _tab_page("stored")
page_configuration_access = _tab_page("access")
page_configuration_fhir = _tab_page("fhir")


def page_config_settings():
    return config_page("settings")


def page_config_settings_edit():
    import tpl
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 1 value needs a change before you can review</p>'
               '<ul><li><a href="#cfg-ov">Overall timeout: must be at least the per-node timeout (3,000 ms)</a></li></ul></div>')
    intro, rows = tab_sections(edit=True)["federation"]
    body = tpl.settings_panel(rows, "A draft of this tab, based on revision 7: 1 change, 1 to fix. The other tabs stay as they run until you apply or discard it.",
                              '<span class="fui-pill fui-tone-accent">editing</span>', top=summary)
    d = dock(dock_field("cfg-reason", "Reason", "node-b's link stays slow; give every node 3 s", "Why this changes; kept with the revision", keys="r"),
             link_button("secondary", "Discard", "configuration.html", "x") + button("primary", "Review changes", "git-compare", attrs="disabled"))
    return config_page("federation", body=body, dock_html=d, drafts=("federation",))


def page_config_settings_review():
    rows = [("federation.per_node_timeout_ms", "2500", "3000", "live")]
    body = ('<div class="fui-settings-panel"><div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">Your draft against revision 7, checked the way the gateway checks its file: no problems.</p></div>'
            + '<div class="fui-panel-body fui-stack">' + diff_table(rows) + '<p class="fui-small">Every query gives each node 3 s instead of 2.5 s, within the overall 4 s; node-b\'s answers at 2.4 s stop timing out.</p></div></div>')
    d = dock(dock_field("cfg-reason", "Reason", "node-b's link stays slow; give every node 3 s", "Why this changes; kept with the revision", keys="r"),
             link_button("secondary", "Back to editing", "config-settings-edit.html", "pencil") + button("secondary", "Save only", "save") + link_button("primary", "Save and apply…", "config-apply.html", "check"))
    return config_page("federation", body=body, dock_html=d, drafts=("federation",))


def page_config_pending():
    banner = pending_card("Draft by Demo Operator, waiting for approval", "Revision 9 changes the credentials of node-a-lab-pub; an operator other than its author approves it, then it runs on trial.",
                          [("done", "Saved", "by Demo Operator, 10:02"), ("current", "Approval", "another operator"), ("waiting", "Apply", "after the approval"), ("waiting", "Confirm", "within 2 min")],
                          link_button("primary", "Review", "config-review-approval.html", "git-compare", small=True), glyph="user-check")
    return config_page("settings", top=banner)


def page_configuration_history():
    return config_page("history", history_panel())


def page_configuration_history_filtered():
    items = [r for r in REVISIONS if r[1] in ("rollback", "import")]
    import tpl
    chips = tpl.active(["Origin: rollback, import"])
    return config_page("history", history_panel(items, history_tools(("rollback", "import")), count=" · 2 of 7") + f'<div class="fui-panel-body">{chips}</div>')


def page_config_rollback():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="rb-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="rb-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Roll back revision 7?</h2>'
        '<p>The content of revision 6, which ran before it, is saved as revision 8 and applied in one step: the per-node timeout goes back to 2,000 ms. A rollback takes no approval and no confirmation; a second rollback returns to revision 7.</p>'
        + field("rb-reason", "Reason", textarea("rb-reason", "Queries to node-b still time out; back to the earlier budget"), help_="Required. Kept with the revision.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Roll back to revision 6", "rotate-ccw")}</div></div>'
    )
    return config_page("history", history_panel(), overlay=dialog)
