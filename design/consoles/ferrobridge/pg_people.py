# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Your preferences, behind the operator's name, and the sign-in states of
the console's OpenID Connect sign-in."""
from common import (
    OPERATOR, badge, button, centred, check, e, icon, link_button, lockup, select, shell, page_heading, when,
)
from parts import choice_cards, density_preview, listbox, select_button, setting_card, setting_row, theme_preview

PREF_TABS = [("appearance", "Appearance", "preferences.html"), ("language", "Language and region", "preferences-language.html"),
             ("keyboard", "Keyboard", "preferences-keyboard.html", "fui-wide-only")]
LANGUAGES = [("browser", "Follow the browser", "English, from your browser"), ("en", "English", None), ("nl", "Nederlands", None)]


def pref_rows(theme="system", saved=None, lang_open=False):
    sel = lambda id_, opts, cur: select(id_, opts, cur, cls="fui-select fui-setting-select")
    return {
        "appearance": setting_row("Theme", "Light, dark, or the one your system uses. Applies at once.",
                                  choice_cards("theme", "Theme", [("light", "Light", "Indigo on white"), ("dark", "Dark", "Indigo on ink"), ("system", "System", "Follows your device")], theme, theme_preview), saved=saved == "theme")
        + setting_row("Density", "Compact fits more rows in every list; comfortable, the default, gives rows and forms room.",
                      choice_cards("density", "Density", [("comfortable", "Comfortable", "More room"), ("compact", "Compact", "More rows")], "comfortable", density_preview), saved=saved == "density")
        + setting_row("Reduce motion", "Fewer animations in menus, panels and the flow's pulse.", sel("motion", [("system", "Follow the system (default)"), ("always", "Always reduce")], "system"), id_="motion"),
        "language": setting_row("Language", "The console's words. Each language is listed in its own name.", select_button("lang", "English", lang_open, listbox(LANGUAGES, "en")), id_="lang", saved=saved == "lang")
        + setting_row("Dates", 'Written year first, with today said in words.<span class="fui-wide-only"> Hover any time for the full timestamp.</span>', '<span class="fui-setting-value"><time class="fui-time">2026-10-21</time> · today 09:42</span>')
        + setting_row("First day of the week", "Follows the language unless you change it.", sel("week", [("auto", "Monday, from the language (default)"), ("mon", "Monday"), ("sun", "Sunday")], "auto"), id_="week")
        + setting_row("Time zone", "Times are shown in the bridge's zone; your browser's is the same.", '<span class="fui-setting-value">Europe/Amsterdam · CEST, UTC+2</span>'),
        "keyboard": setting_row("Shortcut hints", "The key caps after a label. Phones never show them.", sel("hints", [("wide", "Wide screens with a keyboard (default)"), ("always", "All wide screens"), ("never", "Never")], "wide"), id_="hints")
        + setting_row("All shortcuts", "Every shortcut of the screen you are on, in one list.", link_button("secondary", "Open the shortcut legend", "shortcuts.html", "keyboard", small=True, keys="?")),
    }


def preferences_page(tab="appearance", theme="system", saved=None, lang_open=False, overlay=None, lock_dark=False):
    import tpl
    inner = tpl.settings_panel(pref_rows(theme, saved, lang_open)[tab])
    reset = f'<div class="fui-settings-foot"><p class="fui-small fui-muted">Kept in this browser as cookies, read when each page is drawn. Nothing here names a patient.</p>{button("ghost", "Reset to defaults…", "rotate-ccw", small=True)}</div>'
    content = (page_heading("Your preferences", subtitle='Only for you, in this browser.')
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
    return preferences_page("language", saved="lang")


def page_preferences_keyboard():
    return preferences_page("keyboard")


def page_preferences_reset():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="reset-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="reset-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Reset your preferences?</h2>'
        '<p>Theme, density, motion, language, the first day of the week and the shortcut hints go back to their defaults: their cookies in this browser are cleared. The bridge\'s configuration and the audit log are not touched.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Reset to defaults", "rotate-ccw")}</div></div>'
    )
    return preferences_page(overlay=dialog)


def page_preferences_dark():
    return preferences_page(theme="dark", saved="theme", lock_dark=True)


# ---- sign-in ----

DOCS = "https://ferrobridge.eu/docs/"
PRIVACY = "https://bridge.demo-regio.example.org/console/privacy"


def sign_in_card(notice_html="", state=None):
    if state == "redirecting":
        provider = '<button type="button" class="fui-button fui-button-primary" aria-busy="true" disabled><span class="fui-spinner-ring" aria-hidden="true"></span>Taking you to FerroHEALTH ID…</button>'
    elif state == "unavailable":
        provider = f'<button type="button" class="fui-button fui-button-primary" disabled>{icon("log-in", "16")}Sign in with FerroHEALTH ID</button>'
    else:
        provider = f'<a href="sign-in-redirecting.html" rel="external" class="fui-button fui-button-primary">{icon("log-in", "16")}Sign in with FerroHEALTH ID</a>'
    return (
        f'<div class="fui-card fui-sign-in"><div class="fui-sign-in-lockup">{lockup(link=False)}</div>'
        '<h1 class="fui-sign-in-title">Sign in to the console</h1>'
        + notice_html + provider
        + '<p class="fui-small fui-muted fui-sign-in-line">You sign in at your organisation\'s identity provider; the console never sees your password.</p>'
        f'<div class="fui-sign-in-links"><a href="{DOCS}" rel="external noopener">Documentation</a><a href="{PRIVACY}" rel="external noopener">Privacy notice</a></div></div>'
        '<p class="fui-sign-in-caption">ferrobridge-console, for the operators of FerroBRIDGE. It holds no clinical data.</p>'
    )


def sign_in_page(**kw):
    return centred("Sign in", f'<div class="fui-stack fui-sign-in-stack">{sign_in_card(**kw)}</div>')


def n(tone, role, text, hint=None):
    h = f' <span class="fui-small fui-muted">{hint}</span>' if hint else ""
    return f'<div role="{role}" class="fui-sign-in-notice fui-tone-{tone}">{text}{h}</div>'


def page_sign_in():
    return sign_in_page()


def page_sign_in_redirecting():
    return sign_in_page(state="redirecting")


def page_sign_in_refused():
    return sign_in_page(notice_html=n("bad", "alert", "Not signed in: the sign-in could not be completed at FerroHEALTH ID. Try again; if it keeps failing, the provider may be down.",
                                      "The console could not exchange the code for tokens (502)."))


def page_sign_in_rejected():
    return sign_in_page(notice_html=n("bad", "alert", "Not signed in: FerroHEALTH ID did not sign you in, so nothing was kept. If it keeps happening, ask your provider's administrator.",
                                      "For the administrator: the provider refused the sign-in, or the ID token it sent did not pass the console's checks (401)."))


def page_sign_in_expired():
    return sign_in_page(notice_html=n("warn", "status", "Your session ended after 30 minutes without a request. Sign in again to go on."))


def page_sign_in_signed_out():
    return sign_in_page(notice_html=n("ok", "status", "You are signed out of the console and of FerroHEALTH ID."))


def page_sign_in_unavailable():
    return sign_in_page(state="unavailable", notice_html=n("warn", "status", "Sign-in is not set up yet. Whoever runs this console adds an identity provider to its configuration file.",
                                                           "No <code>[oidc]</code> section in ferrobridge-console.toml (503)."))


def page_sign_in_full():
    return sign_in_page(notice_html=n("warn", "alert", "The console has reached its session limit, so it cannot start yours. Try again in a few minutes; sessions end after 30 minutes without a request."))
