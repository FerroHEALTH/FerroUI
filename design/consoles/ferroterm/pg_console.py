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


def server_heading():
    return page_heading("Server", subtitle="What each FHIR root of <code>terminology.example.org</code> declares, and the evidence this build ships.")


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


def server_page(tab, panel_html, line):
    intro = f'<div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">{line}</p></div>'
    tabs_html = tpl.settings_tabs("Server sections", SERVER_TABS, tab, f'<div class="fui-settings-panel">{intro}{panel_html}</div>')
    return shell("Server", "server", server_heading() + f'<div class="fui-stack">{tabs_html}</div>')


def page_server(answered=(True, True, True, True)):
    line = "4 of 4 roots answered, from their CapabilityStatements." if all(answered) else "3 of 4 roots answered, from their CapabilityStatements."
    rows = VERSION_ROWS
    return server_page("versions", matrix("What each root declares", rows, answered), line)


def page_server_root_down():
    return page_server((True, True, True, False))


def page_server_operations():
    rows = [(f"<code>{e(op)}</code>", [None if v is None else v.replace("type, instance", "Type and instance").replace("type", "Type").replace("declared", "Declared") for v in vals]) for op, vals in OPERATIONS]
    return server_page("operations", matrix("Operations, and the levels each answers at", rows), "The operations each root declares, and whether it answers them on the resource type, on one resource, or both.")


def page_server_evidence():
    eco = table("The HL7 terminology ecosystem suite", th("Mode") + th("Surface") + th("Passed", num=True) + th("Needs"),
                [f'<tr><th scope="row" data-role="title"><code>{m}</code></th>{td("Surface", f"<code>{s}</code>", role="sub")}{td("Passed", f"{p} of {r}", "fui-num", role="meta-q")}{td("Needs", n, role="meta-2")}</tr>' for m, s, p, r, n in ECOSYSTEM])
    bars = table("The latency the project claims", th("Benchmark") + th("Bar", num=True) + th("Measured", num=True),
                 [f'<tr><th scope="row" data-role="title"><code>{b}</code></th>{td("Bar", f"{bar} µs", "fui-num", role="sub")}{td("Measured", f"{m} µs", "fui-num", role="meta-q")}</tr>' for b, bar, m in BARS])
    body = sections(
        section("The HL7 terminology ecosystem suite", '<p class="fui-small fui-muted fui-section-intro">From the committed pass lists; only <code>general</code> runs in CI.</p>'
                + f'{eco}', count=7),
        section("The latency the project claims", '<p class="fui-small fui-muted fui-section-intro">Each bar is the claim; the measurement is one Apple M2 run over 20,000 synthetic concepts.</p>'
                + f'{bars}', count=len(BARS)))
    return server_page("evidence", body, f"Read from the files the FerroTERM {VERSION} build commits, not from the server answering this page.")


# ---- Configuration (file mode) ----

LINE = 'Set in <code>console.toml</code> · read at start · <a href="configuration-file-mode-dialog.html">How configuration works</a>'


def key_hint(k, show=False):
    return f'<code class="mk-key-hint{" is-shown" if show else ""}" title="Copy the key">{e(k)}{icon("copy", "12")}</code>'


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
            + srow("Synchronisation service", "Its admin listener, read for the release findings.", "<code>http://ferroterm-sync:8090/</code>", "ferroterm.sync_url")
            + srow("Timeout", "How long one call to the server may take.", "30 s", "ferroterm.timeout_ms"))
    if oidc:
        sign = (srow("Identity provider", "Where readers sign in to edit, by OpenID Connect.", "<code>https://idp.example.org/realms/ferroterm</code>", "oidc.issuer")
                + srow("Client", "The console's client at the provider.", "<code>ferroterm-console</code>", "oidc.client_id")
                + srow("Client secret", "Read from a file; never shown.", secret("/run/secrets/console-client-secret"), "oidc.client_secret_file")
                + srow("Redirect address", "Where the provider sends a reader back.", "<code>https://console.terminology.example.org/<wbr>auth/<wbr>callback</code>", "oidc.redirect_uri")
                + srow("Scopes", "What the console asks for: the three write scopes and the synchronisation listener's.", "<code>openid</code>, <code>fhirUser</code>, 3 write scopes and <code>__ferroterm.sync</code>", "oidc.scopes")
                + srow("Sign-out", "Ends the session at the provider and revokes its token.", "end session and revocation set", "oidc.end_session_endpoint"))
    else:
        sign = ""
    sess = (srow("Secure cookie", "The session cookie is sent over HTTPS only.", on(), "session.secure_cookie")
            + srow("Idle timeout", "A session ends after this long without a request.", "30 min", "session.idle_timeout_s")
            + srow("Absolute timeout", "A session ends after this long, whatever it does.", "8 h", "session.absolute_timeout_s")
            + srow("Most sessions", "How many sessions the console holds at once.", "1,000", "session.max_sessions"))
    lst = (srow("Listener", "The socket address the console binds.", "<code>0.0.0.0:3000</code>", "server.listen")
           + srow("Site root", "Where the console's built site is read from.", "<code>/site</code>", "server.site_root"))
    return {"connection": conn, "signin": sign, "sessions": sess, "listener": lst}


CONFIG_TABS = [("connection", "Connection", "configuration.html"), ("signin", "Sign-in", "configuration-sign-in.html"),
               ("sessions", "Sessions", "configuration-sessions.html"), ("listener", "Listener", "configuration-listener.html")]


def mode_dialog():
    return (
        '<div id="mode-dialog" role="dialog" aria-modal="true" aria-labelledby="mode-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="mode-title" class="fui-dialog-title">{icon("info", "18")} How configuration works</h2>'
        '<p>The console runs in <strong>file mode</strong>: its configuration is <code>console.toml</code>, with <code>FERROTERM_CONSOLE__</code> overrides and <code>_file</code> secrets, read once at start.</p>'
        '<div class="fui-modal-body"><div class="fui-ownership">'
        '<div><h3>' + icon("file-text", "16") + ' Where you see it</h3><p class="fui-small fui-muted">Here, read-only, each value with its key on hover; the server\'s own environment is not visible, and <a href="server.html">Server</a> shows what it declares.</p></div>'
        '<div><h3>' + icon("refresh-cw", "16") + ' How you change it</h3><p class="fui-small fui-muted">Change the file, review it, and restart the console.</p></div></div>'
        '<p class="fui-small fui-muted mk-mode-other">Code systems, value sets and concept maps are content, not configuration: you edit them on their own pages. '
        'The family\'s console mode is only for products with their own configuration store; FerroTERM has none.</p></div>'
        f'<div class="fui-dialog-actions">{button("secondary", "Close", "x", keys="Esc", attrs="autofocus")}</div></div>'
    )


def config_heading(menu_open=False):
    help_ = link_button("ghost", "How configuration works", "configuration-file-mode-dialog.html", "info", attrs='aria-haspopup="dialog"')
    ag = action_group(help_, [menu_item("Copy the file's path", "copy", desc="/etc/ferroterm/console.toml")], open_=menu_open, label="More configuration actions")
    return page_heading("Configuration", actions=ag, subtitle="The settings in console.toml, the console's own file.")


def config_page(tab, overlay=None, menu_open=False, hint=False, oidc=True):
    rows = config_rows(hint, oidc)[tab]
    top = ""
    if tab == "signin" and not oidc:
        top = notice("info", "info", "This console has no <code>[oidc]</code> section, so it offers no sign-in and no editing screen. Reading stays open.")
    body = tpl.settings_panel(rows, LINE, "", top=top)
    tabs_html = tpl.settings_tabs("Configuration sections", CONFIG_TABS, tab, body)
    strip = (f'<div class="fui-revision mk-revision-compact">{icon("file-text", "16")}<p><strong>console.toml</strong> · read at start, '
             f'<time class="fui-time" datetime="2026-10-21T06:00" title="2026-10-21 06:00 CEST">today 06:00</time></p></div>')
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


# ---- Your preferences ----

PREF_TABS = [("appearance", "Appearance", "preferences.html"), ("reading", "Reading", "preferences-reading.html"),
             ("keyboard", "Keyboard", "preferences-keyboard.html"), ("account", "Account", "preferences-account.html")]


def pref_rows(theme="light", saved=None, visitor=False):
    sel = lambda id_, opts, cur: select(id_, opts, cur, cls="fui-select fui-setting-select")
    if visitor:
        acct = (setting_row("Signed in", "You read as a visitor. Signing in is for editing.", '<span class="fui-setting-value">Not signed in</span>')
                + setting_row("Sign in", "At idp.example.org, by OpenID Connect; the console never sees your password.", link_button("secondary", "Sign in", "sign-in.html", "log-in", small=True)))
    else:
        acct = (setting_row("Signed in as", "Your name, as the identity provider states it.", f'<span class="fui-setting-value fui-setting-strong">{OPERATOR}</span>')
                + setting_row("Identity provider", "Where you signed in, by OpenID Connect.", '<span class="fui-setting-value">idp.example.org</span>')
                + setting_row("May edit", "The write scopes your token carries; the server checks them on every write.", f'<span class="fui-setting-value mk-scopes">{badge("accent", "CodeSystem")}{badge("accent", "ValueSet")}{badge("accent", "ConceptMap")}</span>')
                + setting_row("Session", "Ends after 30 minutes without a request, and after 8 hours at the latest.", f'<span class="fui-setting-value">since {when("2026-10-21T08:30")} · until today 16:30</span>')
                + setting_row("Sign out", "Ends this session, revokes its token at the provider, then signs you out there.", link_button("secondary", "Sign out", "sign-in-signed-out.html", "log-out", small=True)))
    return {
        "appearance": setting_row("Theme", "Light, dark, or the one your system uses. Applies at once.",
                                  choice_cards("theme", "Theme", [("light", "Light", "Teal on white"), ("dark", "Dark", "Teal on ink"), ("system", "System", "Follows your device")], theme, theme_preview), saved=saved == "theme")
        + setting_row("Density", "Compact fits more rows in every list; comfortable, the default, gives rows and forms room.",
                      choice_cards("density", "Density", [("comfortable", "Comfortable", "More room"), ("compact", "Compact", "More rows")], "comfortable", density_preview))
        + setting_row("Reduce motion", "Fewer animations in menus, panels and the tree.", sel("motion", [("system", "Follow the system"), ("always", "Always reduce")], "system"), id_="motion"),
        "reading": setting_row("Default FHIR version", "Used when an address carries no version of its own. The version select in the top bar changes it too.", sel("fhirv", [(r, f"{r} · {l}") for r, l in ROOTS], "R4"), id_="fhirv", saved=saved == "fhirv")
        + setting_row("Display language", "A BCP 47 tag sent as displayLanguage. Leave it empty to take the server's own choice.", '<input id="dlang" class="fui-input fui-setting-select" placeholder="for example nl-NL" />', id_="dlang")
        + setting_row("Page size", "How many rows a paged screen asks for, from 1 to 1,000.", '<input id="psize" class="fui-input mk-num-input" value="25" inputmode="numeric" />', id_="psize"),
        "keyboard": setting_row("Shortcut hints", "The key caps after a label. Phones never show them.", sel("hints", [("wide", "Wide screens only (default)"), ("always", "Always"), ("never", "Never")], "wide"), id_="hints")
        + setting_row("All shortcuts", "Every shortcut of the screen you are on, in one list.", link_button("secondary", "Open the shortcut legend", "shortcuts.html", "keyboard", small=True, keys="?")),
        "account": acct,
    }


def preferences_page(tab="appearance", theme="light", saved=None, overlay=None, lock_dark=False, visitor=False):
    inner = tpl.settings_panel(pref_rows(theme, saved, visitor)[tab])
    reset = f'<div class="fui-settings-foot"><p class="fui-small fui-muted">Each preference is its own cookie in this browser, read when each page is drawn.</p>{button("ghost", "Reset to defaults…", "rotate-ccw", small=True)}</div>'
    content = (page_heading("Your preferences", subtitle="Only for you, in this browser.")
               + f'<div class="fui-stack">{tpl.settings_tabs("Preference sections", PREF_TABS, tab, inner)}{reset}</div>')
    html = shell("Your preferences", "", content, overlay=overlay, visitor=visitor)
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


def page_preferences_account():
    return preferences_page("account")


def page_preferences_visitor():
    return preferences_page("account", visitor=True)


def page_preferences_reset():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="reset-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="reset-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Reset your preferences?</h2>'
        '<p>Theme, density, motion, the default FHIR version, the display language, the page size and the hints go back to their defaults: their cookies in this browser are cleared. Your session and the server are not touched.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Reset to defaults", "rotate-ccw")}</div></div>'
    )
    return preferences_page(overlay=dialog)


def page_preferences_dark():
    return preferences_page(theme="dark", saved="theme", lock_dark=True)


# ---- sign-in ----

def sign_in_card(notice_html="", state=None):
    if state == "redirecting":
        provider = '<button type="button" class="fui-button fui-button-primary" aria-busy="true" disabled><span class="fui-spinner-ring" aria-hidden="true"></span>Taking you to idp.example.org…</button>'
    elif state == "unavailable":
        provider = link_button("primary", "Read without signing in", "dashboard-visitor.html", "arrow-right")
    else:
        provider = f'<a href="sign-in-redirecting.html" rel="external" class="fui-button fui-button-primary">{icon("log-in", "16")}Sign in at idp.example.org</a>'
    read = "" if state == "unavailable" else '<p class="fui-small mk-sign-in-line"><a href="dashboard-visitor.html">Read without signing in</a>: every screen is open to read.</p>'
    return (
        f'<div class="fui-card fui-sign-in"><div class="fui-sign-in-lockup">{lockup()}</div>'
        '<h1 class="mk-sign-in-title">Sign in to edit</h1>'
        + notice_html + provider + read
        + '<p class="fui-small fui-muted mk-sign-in-line">You sign in at your organisation\'s identity provider; the console never sees your password and the browser never holds a token.</p>'
        '<div class="fui-sign-in-links"><a href="#" rel="external noopener">Documentation</a><a href="#" rel="external noopener">Privacy notice</a></div></div>'
        '<p class="fui-sign-in-caption">The console of the FerroTERM terminology server. It holds no patient data.</p>'
    )


def sign_in_page(**kw):
    return centred("Sign in", f'<div class="fui-stack mk-sign-in-stack">{sign_in_card(**kw)}</div>')


def n(tone, role, text):
    return f'<div role="{role}" class="fui-sign-in-notice fui-tone-{tone}">{text}</div>'


def page_sign_in():
    return sign_in_page()


def page_sign_in_redirecting():
    return sign_in_page(state="redirecting", notice_html=n("info", "status", "Opening idp.example.org. You come back to the screen you were on when you are signed in."))


def page_sign_in_refused():
    return sign_in_page(notice_html=n("bad", "alert", "Not signed in: the identity provider refused the sign-in or could not be reached (502). Try again; if it keeps failing, the provider may be down."))


def page_sign_in_rejected():
    return sign_in_page(notice_html=n("bad", "alert", "Not signed in: the ID Token the provider sent did not pass the console's checks (401). Nothing was kept. Ask your provider's administrator to check the console's client registration."))


def page_sign_in_expired():
    return sign_in_page(notice_html=n("warn", "status", "Your session ended after 30 minutes without a request. Sign in again to edit; reading needs no session."))


def page_sign_in_signed_out():
    return sign_in_page(notice_html=n("ok", "status", "You are signed out of the console and of the identity provider, and the session's token is revoked."))


def page_sign_in_unavailable():
    return sign_in_page(state="unavailable", notice_html=n("warn", "status", "This console has no sign-in: its configuration has no <code>[oidc]</code> section, so there is no sign-in to complete. Every screen is open to read; editing is not offered."))


def page_sign_in_full():
    return sign_in_page(notice_html=n("warn", "alert", "The console holds as many sessions as it allows (503), so it cannot start yours. Try again in a few minutes; reading needs no session."))
