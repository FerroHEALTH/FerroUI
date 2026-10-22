# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Your preferences, behind the operator's name, and the sign-in states of
the console's OpenID Connect sign-in."""
from common import (
    OPERATOR, badge, button, centred, check, e, icon, link_button, lockup, select, shell, page_heading, when,
)
from parts import choice_cards, density_preview, listbox, select_button, setting_card, setting_row, theme_preview

PREF_TABS = [("appearance", "Appearance", "preferences.html"), ("language", "Language and region", "preferences-language.html"),
             ("keyboard", "Keyboard", "preferences-keyboard.html")]
LANGUAGES = [("browser", "Follow the browser", "English, from your browser"), ("en", "English", None), ("nl", "Nederlands", None)]


def pref_rows(theme="system", saved=None, lang_open=False):
    sel = lambda id_, opts, cur: select(id_, opts, cur, cls="fui-select fui-setting-select")
    return {
        "appearance": setting_row("Theme", "Light, dark, or the one your system uses. Applies at once.",
                                  choice_cards("theme", "Theme", [("light", "Light", "Azure on white"), ("dark", "Dark", "Azure on ink"), ("system", "System", "Follows your device")], theme, theme_preview), saved=saved == "theme")
        + setting_row("Density", "Compact fits more rows in every list; comfortable, the default, gives rows and forms room.",
                      choice_cards("density", "Density", [("comfortable", "Comfortable", "More room"), ("compact", "Compact", "More rows")], "comfortable", density_preview), saved=saved == "density")
        + setting_row("Reduce motion", "Fewer animations in menus, panels and the confirmation countdown.", sel("motion", [("system", "Follow the system"), ("always", "Always reduce")], "system"), id_="motion"),
        "language": setting_row("Language", "The console's words. Each language is listed in its own name.", select_button("lang", "English", lang_open, listbox(LANGUAGES, "en")), id_="lang", saved=saved == "lang")
        + setting_row("Dates", "Written year first, with today said in words. Hover any time for the full timestamp.", '<span class="fui-setting-value"><time class="fui-time">2026-10-21</time> · today 09:42</span>')
        + setting_row("First day of the week", "Follows the language unless you change it.", sel("week", [("auto", "Monday, from the language"), ("mon", "Monday"), ("sun", "Sunday")], "auto"), id_="week")
        + setting_row("Time zone", "Times are shown in the gateway's zone; your browser's is the same.", '<span class="fui-setting-value">Europe/Amsterdam · CEST, UTC+2</span>'),
        "query": setting_row("Answer view", "How an answer opens: its rows as a table, or the federated result set as JSON.", sel("q-view", [("table", "Table"), ("raw", "Raw JSON")], "table"), id_="q-view", saved=saved == "view")
        + setting_row("Fetch", "The rows a query asks for when you leave Fetch empty. The gateway may answer fewer.", sel("q-fetch", [("gw", "The gateway's default"), ("50", "50 rows"), ("100", "100 rows")], "50"), id_="q-fetch")
        + setting_row("Federation options", "Open the options under the parameters when the console opens.", '<input type="checkbox" class="fui-switch" id="q-opts" aria-label="Open the federation options" />')
        + setting_row("What is kept", "No query text, parameter or row once you leave the page; none in the browser's storage or a URL.", '<span class="fui-setting-value">nothing, by design</span>'),
        "keyboard": setting_row("Shortcut hints", "The key caps after a label, on a wide screen. A phone has no keyboard shortcuts.", sel("hints", [("wide", "On every wide screen (default)"), ("never", "Never")], "wide"), id_="hints", saved=saved == "hints")
        + setting_row("All shortcuts", "Every shortcut of the screen you are on, in one list.", link_button("secondary", "Open the shortcut legend", "shortcuts.html", "keyboard", small=True, keys="?")),
        "account": setting_row("Signed in as", "Your name as the gateway records your actions.", f'<span class="fui-setting-value fui-setting-strong">{OPERATOR}</span>')
        + setting_row("Identity provider", "Where you signed in, by OpenID Connect.", '<span class="fui-setting-value">FerroHEALTH ID</span>')
        + setting_row("Access", "The operator scope opens the operator surface and the admin actions.", f'<span class="fui-setting-value mk-scopes">{badge("accent", "__ferrofed.operator")}</span>')
        + setting_row("Session", "Ends after 30 minutes without a request, and after 12 hours at the latest.", f'<span class="fui-setting-value">since {when("2026-10-21T08:30")} · until today 20:30</span>')
        + setting_row("Sign out", "Ends this session on the console's server, then at FerroHEALTH ID.", button("secondary", "Sign out", "log-out", small=True))
        + setting_row("Privacy notice", "How the console treats your own data.", link_button("ghost", "Read the privacy notice", "#", "file-text", small=True)),
    }


def preferences_page(tab="appearance", theme="system", saved=None, lang_open=False, overlay=None, lock_dark=False):
    import tpl
    inner = tpl.settings_panel(pref_rows(theme, saved, lang_open)[tab])
    reset = f'<div class="fui-settings-foot"><p class="fui-small fui-muted">Kept in this browser as cookies, read when each page is drawn. Nothing here names a patient.</p>{button("ghost", "Reset to defaults…", "rotate-ccw", small=True)}</div>'
    content = (page_heading("Your preferences", subtitle='Only for you, in this browser. The gateway\'s settings are in <a href="configuration.html">Configuration</a>.')
               + f'<div class="fui-stack">{tpl.settings_tabs("Preference sections", PREF_TABS, tab, inner)}{reset}</div>')
    html = shell("Your preferences", "", content, overlay=overlay)
    if lock_dark:
        html = html.replace('<html lang="en"', '<html lang="en" data-lock-theme="dark"', 1)
    return html


def page_preferences():
    return preferences_page()


def page_preferences_language():
    return preferences_page("language", lang_open=True)


def page_preferences_saved():
    return preferences_page("appearance", saved="density")


def page_preferences_keyboard():
    return preferences_page("keyboard")


def page_preferences_reset():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="reset-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="reset-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Reset your preferences?</h2>'
        '<p>Every preference on this page goes back to its default, and its cookie in this browser is cleared. The gateway\'s configuration and the audit records are not touched.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Reset to defaults", "rotate-ccw")}</div></div>'
    )
    return preferences_page(overlay=dialog)


def page_preferences_dark():
    return preferences_page(theme="dark", saved="theme", lock_dark=True)


# ---- sign-in ----

def sign_in_card(notice_html="", state=None):
    if state == "redirecting":
        provider = '<button type="button" class="fui-button fui-button-primary" aria-busy="true" disabled><span class="fui-spinner-ring" aria-hidden="true"></span>Taking you to FerroHEALTH ID…</button>'
    elif state == "unavailable":
        provider = f'<button type="button" class="fui-button fui-button-primary" disabled>{icon("log-in", "16")}Sign in</button>'
    else:
        provider = f'<a href="sign-in-redirecting.html" class="fui-button fui-button-primary">{icon("log-in", "16")}Sign in with FerroHEALTH ID</a>'
    return (
        f'<div class="fui-card fui-sign-in"><div class="fui-sign-in-lockup">{lockup()}</div>'
        '<h1 class="mk-sign-in-title">Sign in to the console</h1>'
        + notice_html + provider
        + '<p class="fui-small fui-muted mk-sign-in-line">You sign in at your organisation\'s identity provider; the console never sees your password.</p>'
        '<div class="fui-sign-in-links"><a href="https://ferrofed.eu/docs/" rel="external noopener">Documentation</a><a href="https://ferrofed.eu/privacy/" rel="external noopener">Privacy notice</a></div></div>'
        '<p class="fui-sign-in-caption">The console for the operators of the federation gateway. It holds no clinical data.</p>'
    )


def sign_in_page(**kw):
    return centred("Sign in", f'<div class="fui-stack mk-sign-in-stack">{sign_in_card(**kw)}</div>')


def n(tone, role, text):
    return f'<div role="{role}" class="fui-sign-in-notice fui-tone-{tone}">{text}</div>'


def page_sign_in():
    return sign_in_page()


def page_sign_in_redirecting():
    return sign_in_page(state="redirecting", notice_html=n("info", "status", "You come back here when you are signed in."))


def page_sign_in_refused():
    return sign_in_page(notice_html=n("bad", "alert", "Not signed in: FerroHEALTH ID could not be reached (502). Try again; if it keeps failing, the provider may be down."))


def page_sign_in_rejected():
    return sign_in_page(notice_html=n("bad", "alert", "Not signed in: the sign-in answer from FerroHEALTH ID did not verify, or it refused you (401). Nothing was kept. Ask your provider's administrator to check the console's registration."))


def page_sign_in_expired():
    return sign_in_page(notice_html=n("warn", "status", "Your session ended after 30 minutes without a request. Sign in again to go on; a query that was open was not kept."))


def page_sign_in_signed_out():
    return sign_in_page(notice_html=n("ok", "status", "You are signed out of the console and of FerroHEALTH ID."))


def page_sign_in_unavailable():
    return sign_in_page(state="unavailable", notice_html=n("warn", "status", "Sign-in is not set up on this console (503): it names no identity provider<code class=\"fui-small fui-muted mk-term\"> [oidc]</code>. Its operator adds one to the console's configuration."))


def page_sign_in_full():
    return sign_in_page(notice_html=n("warn", "alert", "The console holds as many sessions as it allows (503), so it cannot start yours. Try again in a few minutes; sessions end after 30 minutes without a request."))
