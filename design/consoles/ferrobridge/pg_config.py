# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Configuration, T3, file mode only (owner decision, review round 1):
FerroBRIDGE has no configuration store, so ferrobridge.toml is the one
source and every tab here, like every face Settings tab, is a read-only view
of it, read at start. Mapping files are content, not configuration: they are
edited on the Mapping set, saved with a reason."""
import tpl
from common import action_group, button, icon, link_button, menu_item, page_heading, shell
from parts import mode_badge
from settings import LINE, on, srow

SECTIONS = ('<ul class="fui-bullets fui-small"><li>Here: logging, the mapping directories, the server and who may call</li>'
            '<li>On each face: the FHIR facade and its operations, HL7 v2, the OMOP ETL, under Settings</li>'
            '<li>On each connection: the CDR, the terminology server, the CDM database, with their credentials by reference</li></ul>')


def mode_dialog():
    return (
        '<div id="mode-dialog" role="dialog" aria-modal="true" aria-labelledby="mode-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="mode-title" class="fui-dialog-title">{icon("info", "18")} How configuration works</h2>'
        '<p>FerroBRIDGE runs in <strong>file mode</strong>: its whole configuration is <code>ferrobridge.toml</code>, with <code>FERROBRIDGE__</code> overrides and <code>_file</code> secrets, read once at start.</p>'
        '<div class="fui-modal-body"><div class="fui-ownership">'
        '<div><h3>' + icon("file-text", "16") + ' Where you see it</h3><p class="fui-small fui-muted">Read-only, each value with its TOML key on hover.</p>' + SECTIONS + '</div>'
        '<div><h3>' + icon("refresh-cw", "16") + ' How you change it</h3><p class="fui-small fui-muted">Change the file, review it in version control, and restart the bridge. '
        'A value the bridge refuses stops the start and names the key, never the value.</p></div></div>'
        '<p class="fui-small fui-muted mk-mode-other">Mapping files are content, not configuration: you edit them on the Mapping set and save each with a reason. '
        'The family\'s console mode is only for products with their own configuration store; the bridge has none.</p></div>'
        f'<div class="fui-dialog-actions">{button("secondary", "Close", "x", keys="Esc", attrs="autofocus")}</div></div>'
    )


def heading(menu_open=False):
    help_ = link_button("ghost", "How configuration works", "configuration-file-mode-dialog.html", "info", attrs='aria-haspopup="dialog"')
    ag = action_group(help_, [menu_item("Copy the file's path", "copy", desc="/etc/ferrobridge/ferrobridge.toml")], open_=menu_open, label="More configuration actions")
    return page_heading("Configuration", actions=ag, subtitle="The settings in ferrobridge.toml that no face or connection owns.", badge=mode_badge("file"))


def strip():
    return (f'<div class="fui-revision mk-revision-compact">{icon("file-text", "16")}<p><strong>ferrobridge.toml</strong> · read at start, '
            f'<time class="fui-time" datetime="2026-10-21T06:00" title="2026-10-21 06:00 CEST">today 06:00</time></p></div>')


TABS = [("logging", "Logging", "configuration.html"), ("mappings", "Mappings", "configuration-mappings.html"),
        ("core", "Server and access", "configuration-core.html"), ("console", "Console", "configuration-console.html")]


def sections(hint=False):
    lg = (srow("Format", "auto, json or pretty; auto is pretty on a terminal and JSON otherwise.", "json", "telemetry.format", hint=hint)
          + srow("Filter", "Which events reach the log, as a RUST_LOG directive.", "<code>info,hyper=warn</code> and 2 more", "telemetry.filter")
          + srow("Logged query parameters", "The parameters whose values may reach the request log.", '<span class="mk-dash">none</span>', "telemetry.logged_query_parameters"))
    mp = (srow("Mapping directory", "Where the FHIRconnect files are read from.", "<code>/etc/ferrobridge/mappings</code>", "mappings.directory")
          + srow("Templates", "The operational templates the operations compile against.", "<code>/etc/ferrobridge/templates</code>", "mappings.templates")
          + srow("OMOCL directory", "Where the ETL reads its OMOCL files from.", "<code>/etc/ferrobridge/omocl</code>", "mappings.omocl"))
    core = (srow("Listener", "The socket address the bridge binds.", "<code>0.0.0.0:8080</code>", "server.listen")
            + srow("Request limits", "How long one request may take, and the largest body read.", "30 s · 1 MiB", "server.request_timeout_ms")
            + srow("Issuers", "Who signs the tokens the bridge accepts.", "2 issuers", "auth.issuer")
            + srow("Audience", "The audience every token must name.", "<code>https://bridge.demo-regio.example.org/fhir</code>", "auth.audience")
            + srow("Operator scope", "The scope the operator API and this console need.", "<code>__ferrobridge.operator</code>", "auth.product_scope")
            + srow("Operator API", "The routes this console reads and saves mapping files through.", on(), "operator.enabled")
            + srow("Console's mapping directory", "Where a mapping file saved here is written.", "<code>/var/lib/ferrobridge/console</code>", "operator.mapping_dir")
            + srow("Usage report", "One report at start and one a day to FerroPULSE, no clinical data.", on(), "usage_report.enabled"))
    con = (srow("Bridge", "The bridge this console reads, with your own token.", "<code>https://bridge.demo-regio.example.org</code>", "bridge.base")
           + srow("Identity provider", "Where operators sign in.", "<code>idp.demo-regio.example.org</code>", "oidc.issuer")
           + srow("Client", "The console's client at the provider.", "<code>ferrobridge-console</code>", "oidc.client_id")
           + srow("Sessions", "Idle timeout, absolute timeout, and how many are held.", "30 min · 12 h · 10,000", "session.idle_timeout_s"))
    return {"logging": lg, "mappings": mp, "core": core, "console": con}


CONSOLE_LINE = 'Set in <code>ferrobridge-console.toml</code> · read at start · the console\'s own file, not the bridge\'s'


def config_page(tab, overlay=None, menu_open=False, hint=False):
    rows = sections(hint)[tab]
    body = tpl.settings_panel(rows, CONSOLE_LINE if tab == "console" else LINE, "")
    tabs_html = tpl.settings_tabs("Configuration sections", TABS, tab, body)
    content = heading(menu_open) + f'<div class="fui-stack">{strip()}{tabs_html}</div>'
    return shell("Configuration", "configuration", content, overlay=overlay)


def page_configuration(overlay=None, menu_open=False, hint=False):
    return config_page("logging", overlay=overlay, menu_open=menu_open, hint=hint)


def page_configuration_file_mode_dialog():
    return page_configuration(overlay=mode_dialog())


def page_configuration_mappings():
    return config_page("mappings")


def page_configuration_core():
    return config_page("core")


def page_configuration_console():
    return config_page("console")


def page_configuration_key_hint():
    return page_configuration(hint=True)


def page_configuration_actions():
    return page_configuration(menu_open=True)
