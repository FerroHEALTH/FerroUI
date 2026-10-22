# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Configuration, T3, file mode only (owner decision, review round 1):
FerroBRIDGE has no configuration store, so ferrobridge.toml is the one
source and every tab here, like every face Settings tab, is a read-only view
of it, read at start. Mapping files are content, not configuration: they are
edited on the Mapping set, saved with a reason."""
import tpl
from common import button, copy_button, icon, page_heading, shell
from settings import LINE, on, srow

SECTIONS = ('<ul class="fui-bullets fui-small"><li>Here: logging, the mapping directories, the server and who may call</li>'
            '<li>On each face: the FHIR facade and its operations, HL7 v2, the OMOP ETL, under Settings</li>'
            '<li>On each connection: the CDR, the terminology server, the CDM database, with their credentials by reference</li></ul>')


def mode_dialog():
    return (
        '<div id="mode-dialog" role="dialog" aria-modal="true" aria-labelledby="mode-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="mode-title" class="fui-dialog-title">{icon("info", "18")} How configuration works</h2>'
        '<p>The bridge reads its whole configuration from <code>ferrobridge.toml</code> once, at start, with <code>FERROBRIDGE__</code> overrides and <code>_file</code> secrets.</p>'
        '<div class="fui-modal-body"><div class="fui-ownership">'
        '<div><h3>' + icon("file-text", "16") + ' Where you see it</h3><p class="fui-small fui-muted">Read-only, each value with its TOML key on hover.</p>' + SECTIONS + '</div>'
        '<div><h3>' + icon("refresh-cw", "16") + ' How you change it</h3><p class="fui-small fui-muted">Change the file, review it in version control, and restart the bridge. '
        'A value the bridge refuses stops the start and names the key, never the value.</p></div></div>'
        '<p class="fui-small fui-muted mk-mode-other">Mapping files are content: you edit them on the Mapping set and save each with a reason.</p></div>'
        f'<div class="fui-dialog-actions">{button("secondary", "Close", "x", keys="Esc", attrs="autofocus")}</div></div>'
    )


def heading():
    return page_heading("Configuration", subtitle="The settings in ferrobridge.toml that no face or connection owns.")


FILE_LINE = ('Set in <code>/etc/ferrobridge/ferrobridge.toml</code> ' + copy_button("Copy", "the file's path")
             + ' · read at start, <time class="fui-time" datetime="2026-10-21T06:00" title="2026-10-21 06:00 CEST">today 06:00</time>'
             + ' · <a href="configuration-file-mode-dialog.html">How configuration works</a>')


TABS = [("logging", "Logging", "configuration.html"), ("mappings", "Mappings", "configuration-mappings.html"),
        ("core", "Server and access", "configuration-core.html")]


def sections(hint=False):
    lg = (srow("Format", "auto, json or pretty; auto is pretty on a terminal and JSON otherwise.", "json", "telemetry.format", hint=hint)
          + srow("Filter", "Which events reach the log, as a RUST_LOG directive.", "<code>info,hyper=warn</code> and 2 more", "telemetry.filter")
          + srow("Logged query parameters", "The parameters whose values may reach the request log.", '<span class="mk-dash">none</span>', "telemetry.logged_query_parameters"))
    mp = (srow("Mapping directory", "Where the FHIRconnect files are read from.", "<code>/etc/ferrobridge/mappings</code>", "mappings.directory")
          + srow("Templates", "The operational templates the operations compile against.", "<code>/etc/ferrobridge/templates</code>", "mappings.templates")
          + srow("OMOCL directory", "Where the ETL reads its OMOCL files from.", "<code>/etc/ferrobridge/omocl</code>", "mappings.omocl"))
    issuers = ('<ul class="mk-plain-list"><li><code>https://idp.demo-regio.example.org/realms/bridge</code></li>'
               '<li><code>https://idp.demo-region-backup.example.org/realms/bridge</code></li></ul>')
    core = (srow("Listener", "The socket address the bridge binds.", "<code>0.0.0.0:8080</code>", "server.listen")
            + srow("Request limits", "How long one request may take, and the largest body read.", "30 s · 1 MiB", "server.request_timeout_ms",
                   keys=("server.request_timeout_ms", "server.body_limit_bytes"))
            + srow("Issuers", "Who signs the tokens the bridge accepts.", issuers, "auth.issuer")
            + srow("Audience", "The audience every token must name.", "<code>https://bridge.demo-regio.example.org/fhir</code>", "auth.audience")
            + srow("Operator scope", "The scope the operator API and this console need.", "<code>__ferrobridge.operator</code>", "auth.product_scope")
            + srow("Operator API", "The routes this console reads and saves mapping files through.", on(), "operator.enabled")
            + srow("Console's mapping directory", "Where a mapping file saved here is written.", "<code>/var/lib/ferrobridge/console</code>", "operator.mapping_dir")
            + srow("Usage report", "One report at start and one a day to FerroPULSE, no clinical data.", on(), "usage_report.enabled"))
    return {"logging": lg, "mappings": mp, "core": core}


def config_page(tab, overlay=None, hint=False, toast_html=""):
    rows = sections(hint)[tab]
    body = tpl.settings_panel(rows, FILE_LINE, "")
    tabs_html = tpl.settings_tabs("Configuration sections", TABS, tab, body)
    content = heading() + f'<div class="fui-stack">{tabs_html}</div>'
    return shell("Configuration", "configuration", content, overlay=overlay, toast_html=toast_html)


def page_configuration(overlay=None, hint=False, toast_html=""):
    return config_page("logging", overlay=overlay, hint=hint, toast_html=toast_html)


def page_configuration_file_mode_dialog():
    return page_configuration(overlay=mode_dialog())


def page_configuration_mappings():
    return config_page("mappings")


def page_configuration_core():
    return config_page("core")


def page_configuration_key_hint():
    return page_configuration(hint=True)


def page_configuration_actions():
    from common import toast, toasts
    return page_configuration(toast_html=toasts(toast("ok", "circle-check", "Path copied", "/etc/ferrobridge/ferrobridge.toml is on your clipboard.")))
