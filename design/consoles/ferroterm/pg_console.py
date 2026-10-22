# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The Console group and the reader's own pages: Server (T3, what each
FHIR root declares and the evidence the build ships), Configuration (T3,
file mode only, a read-only view of console.toml), Your preferences
(cookies) and the sign-in states."""
import tpl
from common import (
    OPERATOR, ROOTS, action_group, badge, button, card, centred, e, icon, link_button, lockup, menu_item, notice, page_heading, select, shell, table, td, th, when,
)
from data import BARS, ECOSYSTEM, OPERATIONS, VERSION
from parts import choice_cards, density_preview, setting_row, theme_preview
from term import root_head, section, sections

# ---- Server ----

SERVER_TABS = [("versions", "FHIR versions", "server.html"), ("operations", "Operations", "server-operations.html"), ("evidence", "Evidence", "server-evidence.html")]


def server_heading(actions=""):
    return page_heading("Server", actions=actions, subtitle="What each FHIR root of <code>terminology.example.org</code> declares, and the evidence this build ships.")


def matrix(caption, rows, answered=(True, True, True, True)):
    head = th("Declaration") + "".join(f'<th scope="col">{root_head(r, a)}</th>' for (r, _), a in zip(ROOTS, answered))
    body = []
    for name, vals in rows:
        cells = ""
        for (r, _), v, a in zip(ROOTS, vals, answered):
            if not a:
                cells += f'<td data-label="{r}" class="is-none">no answer</td>'
            elif v is None:
                cells += f'<td data-label="{r}" class="is-none">not declared</td>'
            else:
                cells += f'<td data-label="{r}">{v}</td>'
        body.append(f'<tr><th scope="row" data-role="title">{name}</th>{cells}</tr>')
    t = f'<table class="fui-table"><caption class="fui-sr-only">{e(caption)}</caption><thead><tr>{head}</tr></thead><tbody>{"".join(body)}</tbody></table>'
    return f'<div class="fui-root-matrix"><div class="fui-table-scroll" role="region" aria-label="{e(caption)}" tabindex="0">{t}</div></div>'


VERSION_ROWS = [
    ("Software", [f"FerroTERM {VERSION}"] * 4),
    ("Base address", ["<code>/r4</code>", "<code>/r4b</code>", "<code>/r5</code>", "<code>/r6</code>"]),
    ("Code systems", ["15", "15", "15", "0"]),
    ("Security service", ["SMART-on-FHIR"] * 4),
    ("What the root says it serves", [None, None, None, "The FHIR R6 ballot (6.0.0-ballot5). Behaviour grounded only in the ballot may change when R6 is published."]),
]


def server_page(tab, panel_html, line, status="up", actions="", top=""):
    intro = f'<div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">{line}</p></div>'
    tabs_html = tpl.settings_tabs("Server sections", SERVER_TABS, tab, f'<div class="fui-settings-panel">{intro}{top}{panel_html}</div>')
    return shell("Server", "server", server_heading(actions) + f'<div class="fui-stack">{tabs_html}</div>', status=status)


def page_server(answered=(True, True, True, True), actions=""):
    down = not all(answered)
    line = "3 of 4 roots answered, from their CapabilityStatements." if down else "4 of 4 roots answered, from their CapabilityStatements."
    top = ""
    if down:
        from term import load_report
        top = load_report("bad", "FHIR R6 did not answer at <code>/r6</code>, so its column is empty.", actions=button("secondary", "Retry", "refresh-cw", small=True))
    return server_page("versions", matrix("What each root declares", VERSION_ROWS, answered), line, status="degraded" if down else "up", actions=actions, top=top)


def page_server_root_down():
    return page_server((True, True, True, False))


def op_name(op):
    """A FHIR operation by its name; FerroTERM's own operation says so."""
    if op == "$cache-control":
        return f'<code>{e(op)}</code> <span class="fui-tag" title="Not a FHIR operation: FerroTERM defines it">FerroTERM\'s own</span>'
    return f"<code>{e(op)}</code>"


def page_server_operations():
    rows = [(op_name(op), [None if v is None else v.replace("type, instance", "Type and instance").replace("type", "Type").replace("declared", "Declared") for v in vals]) for op, vals in OPERATIONS]
    return server_page("operations", matrix("Operations, and the levels each answers at", rows), "The operations each root declares, and whether it answers them on the resource type, on one resource, or both.")


def page_server_evidence():
    eco = table("The HL7 terminology ecosystem suite", th("Mode") + th("Surface") + th("Passed", num=True) + th("Content it needs"),
                [f'<tr><th scope="row" data-role="title"><code>{m}</code></th>{td("Surface", f"<code>{s}</code>", role="sub")}{td("Passed", f"{p} of {r}", "fui-num", role="meta-q")}'
                 f'{td("Content it needs", n or "<span class=" + chr(34) + "fui-muted" + chr(34) + ">No extra content</span>", role="meta-2")}</tr>' for m, s, p, r, n in ECOSYSTEM])
    bars = table("Latency", th("Benchmark") + th("Target", num=True) + th("Measured", num=True),
                 [f'<tr><th scope="row" data-role="title"><code>{b}</code></th>{td("Target", f"{bar} µs", "fui-num", role="sub")}{td("Measured", f"{m} µs", "fui-num", role="meta-q")}</tr>' for b, bar, m in BARS])
    body = sections(
        section("The HL7 terminology ecosystem suite", '<p class="fui-small fui-muted fui-section-intro">From the committed pass lists; only <code>general</code> runs in CI.</p>'
                + f'{eco}', count=7),
        section("Latency", '<p class="fui-small fui-muted fui-section-intro">Each target is the project\'s claim; the measurement is one Apple M2 run over 20,000 synthetic concepts.</p>'
                + f'{bars}', count=len(BARS)))
    return server_page("evidence", body, f"Read from the files the FerroTERM {VERSION} build commits, not from the server answering this page.")


# ---- Configuration (file mode) ----

def key_hint(k, show=False):
    return f'<button type="button" class="mk-key-hint{" is-shown" if show else ""}" aria-label="Copy the key {e(k)}" title="Copy the key"><code>{e(k)}</code>{icon("copy", "12")}</button>'


def srow(label, desc, value, key, hint=False):
    return (f'<div class="fui-setting-row{" mk-hover" if hint else ""}"><div class="fui-setting-text"><span class="fui-setting-label">{label}{key_hint(key, hint)}</span>'
            f'<p class="fui-setting-desc">{desc}</p></div><div class="fui-setting-control"><span class="fui-setting-value">{value}</span></div></div>')


def secret(ref):
    return f'<span class="fui-secret-now">{icon("lock", "12")}set, read from <code>{e(ref).replace("/", "/<wbr>")}</code></span>'


def on(v=True):
    return badge("ok", "On") if v else badge("neutral", "Off")


def config_rows(hint=False, oidc=True):
    conn = (srow("Server", "Where the console reaches the terminology server, on your private network.", "<code>http://ferroterm:8080/</code>", "ferroterm.base_url", hint)
            + srow("Public address", "The address readers see the FHIR API at, shown on the screens.", "<code>https://terminology.example.org/</code>", "ferroterm.public_url")
            + srow("Synchronisation service", "Where the console reads release checks from the synchronisation service.", "<code>http://ferroterm-sync:8090/</code>", "ferroterm.sync_url")
            + srow("Timeout", "How long one call to the server may take.", "30 s", "ferroterm.timeout_ms"))
    if oidc:
        sign = (srow("Identity provider", "Where readers sign in to edit, by OpenID Connect.", "<code>https://idp.example.org/realms/ferroterm</code>", "oidc.issuer")
                + srow("Client", "The console's client at the provider.", "<code>ferroterm-console</code>", "oidc.client_id")
                + srow("Client secret", "Read from a file; never shown.", secret("/run/secrets/console-client-secret"), "oidc.client_secret_file")
                + srow("Redirect address", "Where the provider sends a reader back.", "<code>https://console.terminology.example.org/<wbr>auth/<wbr>callback</code>", "oidc.redirect_uri")
                + srow("Scopes", "What the console asks the provider for.", "Sign-in and profile, editing code systems, value sets and concept maps, and release checks", "oidc.scopes")
                + srow("Sign-out", "Ends the session at the provider too.", "Set up", "oidc.end_session_endpoint"))
    else:
        sign = '<p class="fui-small fui-muted">Sign-in is not set up; reading stays open.</p>'
    sess = (srow("Secure cookie", "The session cookie is sent over HTTPS only.", on(), "session.secure_cookie")
            + srow("Idle timeout", "A session ends after this long without a request.", "30 min", "session.idle_timeout_s")
            + srow("Absolute timeout", "A session ends after this long, even while in use.", "8 h", "session.absolute_timeout_s")
            + srow("Session limit", "How many sessions the console holds at once.", "1,000", "session.max_sessions"))
    lst = (srow("Listener", "Address and port the console listens on.", "<code>0.0.0.0:3000</code>", "server.listen")
           + srow("Site root", "Where the console's built site is read from.", "<code>/site</code>", "server.site_root"))
    return {"connection": conn, "signin": sign, "sessions": sess, "listener": lst}


CONFIG_TABS = [("connection", "Connection", "configuration.html"), ("signin", "Sign-in", "configuration-sign-in.html"),
               ("sessions", "Sessions", "configuration-sessions.html"), ("listener", "Listener", "configuration-listener.html")]


def mode_dialog():
    return (
        '<div id="mode-dialog" role="dialog" aria-modal="true" aria-labelledby="mode-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="mode-title" class="fui-dialog-title">{icon("info", "18")} How configuration works</h2>'
        '<p>The console reads its settings from one file when it starts. An environment variable can override a value, and a secret can come from a file of its own.</p>'
        '<p class="fui-small fui-muted"><code>console.toml</code> · <code>FERROTERM_CONSOLE__</code> overrides · <code>_file</code> secrets</p>'
        '<div class="fui-modal-body"><div class="fui-ownership">'
        '<div><h3>' + icon("file-text", "16") + ' Where you see it</h3><p class="fui-small fui-muted">Here, read-only, each value with its key when you point at it. The server keeps its own settings; <a href="server.html">Server</a> shows what it declares.</p></div>'
        '<div><h3>' + icon("refresh-cw", "16") + ' How you change it</h3><p class="fui-small fui-muted">Edit the file, review it, and restart the console.</p></div></div>'
        '<p class="fui-small fui-muted mk-mode-other">Code systems, value sets and concept maps are content: you edit them on their own pages.</p></div>'
        f'<div class="fui-dialog-actions">{button("secondary", "Close", "x", keys="Esc", attrs="autofocus")}</div></div>'
    )


def config_heading(menu_open=False):
    help_ = link_button("ghost", "How configuration works", "configuration-file-mode-dialog.html", "info", attrs='aria-haspopup="dialog"')
    ag = action_group(help_, [menu_item("Copy the file's path", "copy", desc_html="<code>/etc/ferroterm/console.toml</code>")], open_=menu_open, label="More configuration actions")
    return page_heading("Configuration", actions=ag, subtitle="The settings in <code>console.toml</code>, read when the console starts.")


def config_page(tab, overlay=None, menu_open=False, hint=False, oidc=True):
    rows = config_rows(hint, oidc)[tab]
    body = tpl.settings_panel(rows)
    tabs_html = tpl.settings_tabs("Configuration sections", CONFIG_TABS, tab, body, many=True)
    return shell("Configuration", "configuration", config_heading(menu_open) + f'<div class="fui-stack">{tabs_html}</div>', overlay=overlay)


def page_configuration():
    return config_page("connection")


def page_configuration_file_mode_dialog():
    return config_page("connection", overlay=mode_dialog())


def page_configuration_sign_in():
    return config_page("signin")


def page_configuration_sessions():
    return config_page("sessions")


def page_configuration_listener():
    return config_page("listener")


def page_configuration_key_hint():
    return config_page("connection", hint=True)


def page_configuration_actions():
    return config_page("connection", menu_open=True)


def page_configuration_no_oidc():
    return config_page("signin", oidc=False)


# ---- Your preferences: three cookie tabs; who you are lives in the user menu ----

PREF_TABS = [("appearance", "Appearance", "preferences.html"), ("reading", "Language and region", "preferences-reading.html"),
             ("keyboard", "Keyboard", "preferences-keyboard.html", "fui-wide-only")]


def pref_rows(theme="system", saved=None):
    sel = lambda id_, opts, cur: select(id_, opts, cur, cls="fui-select fui-setting-select")
    return {
        "appearance": setting_row("Theme", "Light, dark, or the one your system uses. Applies at once.",
                                  choice_cards("theme", "Theme", [("light", "Light", "Teal on white"), ("dark", "Dark", "Teal on ink"), ("system", "System", "Follows your device")], theme, theme_preview), saved=saved == "theme")
        + setting_row("Density", "How much fits on a screen.",
                      choice_cards("density", "Density", [("comfortable", "Comfortable", "More room"), ("compact", "Compact", "More rows")], "comfortable", density_preview))
        + setting_row("Reduce motion", "Fewer animations in menus, panels and the tree.", sel("motion", [("system", "Follow the system"), ("always", "Always reduce")], "system"), id_="motion"),
        "reading": setting_row("Default FHIR version", "Used when an address names no version. The version select in the top bar switches only this session.", sel("fhirv", [(r, f"{r} · {l}") for r, l in ROOTS], "R4"), id_="fhirv", saved=saved == "fhirv")
        + setting_row("Display language", 'Language for concept names, for example nl-NL. Empty takes the server\'s own choice. <code class="fui-muted">displayLanguage</code>', '<input id="dlang" class="fui-input fui-setting-select" placeholder="for example nl-NL" />', id_="dlang")
        + setting_row("Page size", "How many rows a paged screen asks for, from 1 to 1,000.", '<input id="psize" class="fui-input mk-num-input" value="25" inputmode="numeric" />', id_="psize"),
        "keyboard": setting_row("Shortcut hints", "The key caps after a label. Phones never show them.", sel("hints", [("wide", "Wide screens only (default)"), ("always", "Always"), ("never", "Never")], "wide"), id_="hints")
        + setting_row("All shortcuts", "Every shortcut of the screen you are on, in one list.", link_button("secondary", "Open the shortcut legend", "shortcuts.html", "keyboard", small=True, keys="?")),
    }


def preferences_page(tab="appearance", theme="system", saved=None, overlay=None, lock_dark=False):
    inner = tpl.settings_panel(pref_rows(theme, saved)[tab])
    reset = f'<div class="fui-settings-foot"><p class="fui-small fui-muted">Each preference is its own cookie in this browser, read when each page is drawn.</p>{button("ghost", "Reset to defaults…", "rotate-ccw", small=True)}</div>'
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
    return preferences_page("reading", saved="fhirv")


def page_preferences_keyboard():
    return preferences_page("keyboard")


def page_preferences_reset():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="reset-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="reset-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Reset your preferences?</h2>'
        '<p>All preferences on this page go back to their defaults. Your session and the server are not touched.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Reset to defaults", "rotate-ccw")}</div></div>'
    )
    return preferences_page(overlay=dialog)


def page_preferences_dark():
    return preferences_page(theme="dark", saved="theme", lock_dark=True)


# ---- sign-in ----

DOCS = "https://ferroterm.eu/docs/"
PRIVACY = "https://terminology.example.org/privacy"


def sign_in_card(notice_html="", state=None, title="Sign in to edit"):
    retry = f'<a href="sign-in-redirecting.html" rel="external" class="fui-button fui-button-primary">{icon("refresh-cw", "16")}Try again</a>'
    read_line = '<p class="fui-small mk-sign-in-line"><a href="dashboard-visitor.html">Read without signing in</a>: every screen is open to read.</p>'
    foot = '<p class="fui-small fui-muted mk-sign-in-line">You sign in at your organisation\'s identity provider; the console never sees your password and the browser never holds a token.</p>'
    if state == "redirecting":
        main = '<button type="button" class="fui-button fui-button-primary" aria-busy="true" disabled><span class="fui-spinner-ring" aria-hidden="true"></span>Taking you to idp.example.org…</button>' + read_line
    elif state == "unavailable":
        main, foot = link_button("primary", "Read without signing in", "dashboard-visitor.html", "arrow-right"), ""
    elif state == "retry":
        main = retry + read_line
    elif state == "rejected":
        main = link_button("primary", "Read without signing in", "dashboard-visitor.html", "arrow-right")
    else:
        main = f'<a href="sign-in-redirecting.html" rel="external" class="fui-button fui-button-primary">{icon("log-in", "16")}Sign in at idp.example.org</a>' + read_line
    return (
        f'<div class="fui-card fui-sign-in"><div class="fui-sign-in-lockup">{lockup()}</div>'
        f'<h1 class="mk-sign-in-title">{title}</h1>'
        + notice_html + main + foot
        + f'<div class="fui-sign-in-links"><a href="{DOCS}" rel="external noopener">Documentation</a><a href="{PRIVACY}" rel="external noopener">Privacy notice</a></div></div>'
        '<p class="fui-sign-in-caption">The console of the FerroTERM terminology server. It holds no patient data.</p>'
    )


def sign_in_page(**kw):
    return centred("Sign in", f'<div class="fui-stack mk-sign-in-stack">{sign_in_card(**kw)}</div>')


def n(tone, role, text):
    return f'<div role="{role}" class="fui-sign-in-notice fui-tone-{tone}">{text}</div>'


def page_sign_in():
    return sign_in_page()


def page_sign_in_redirecting():
    return sign_in_page(state="redirecting", notice_html=n("info", "status", "You come back to this screen."))


def page_sign_in_refused():
    return sign_in_page(state="retry", notice_html=n("bad", "alert", "The identity provider refused the sign-in. Try again; if it refuses again, ask whoever runs it."))


def page_sign_in_unreachable():
    return sign_in_page(state="retry", notice_html=n("bad", "alert", "The console could not reach the identity provider. Try again in a moment; it may be down."))


def page_sign_in_rejected():
    return sign_in_page(state="rejected", notice_html=n("bad", "alert", "The sign-in could not be completed: the provider's answer did not pass the console's checks, so nothing was kept. Ask whoever runs the identity provider to check the console's registration."))


def page_sign_in_expired():
    return sign_in_page(notice_html=n("warn", "status", "Your session ended after 30 minutes without a request. Sign in again to edit."))


def page_sign_in_expired_absolute():
    return sign_in_page(notice_html=n("warn", "status", "Your session ended: a session lasts 8 hours at most, even while in use. Sign in again to edit."))


def page_sign_in_signed_out():
    return sign_in_page(notice_html=n("ok", "status", "You are signed out of the console and of the identity provider."))


def page_sign_in_unavailable():
    return sign_in_page(state="unavailable", title="Read-only console",
                        notice_html=n("warn", "status", 'This console has no sign-in configured. Every screen is open to read. <span class="fui-small fui-muted">No <code>[oidc]</code> section.</span>'))


def page_sign_in_full():
    return sign_in_page(notice_html=n("warn", "alert", "The console has no free sessions right now. Try again in a few minutes."))
