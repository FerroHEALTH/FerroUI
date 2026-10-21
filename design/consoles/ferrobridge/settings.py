# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""T3 setting rows, exactly as Configuration draws them, shared by the
Settings tabs of the face pages (FHIR facade, HL7 v2, OMOP ETL, each
connection) and Configuration itself. FerroBRIDGE runs in file mode only
(owner decision, review round 1): every row is a read-only view of
ferrobridge.toml, read at start, the TOML key on hover."""
import tpl
from common import badge, e, icon

LINE = 'Set in <code>ferrobridge.toml</code> · read at start · <a href="configuration-file-mode-dialog.html">How configuration works</a>'


def key_hint(k, show=False):
    return f'<code class="mk-key-hint{" is-shown" if show else ""}" title="Copy the key">{e(k)}{icon("copy", "12")}</code>'


def on(v=True):
    """A switch read from the file: a state badge, never a raw on or off."""
    return badge("ok", "On") if v else badge("neutral", "Off")


def srow(label, desc, value, key, hint=False):
    return (f'<div class="fui-setting-row{" mk-hover" if hint else ""}"><div class="fui-setting-text"><span class="fui-setting-label">{label}{key_hint(key, hint)}</span>'
            f'<p class="fui-setting-desc">{desc}</p></div><div class="fui-setting-control"><span class="fui-setting-value">{value}</span></div></div>')


def secret_now(ref, state="set"):
    if state == "unset":
        return f'<span class="fui-secret-now is-unset">{icon("lock", "12")}not set</span>'
    if state == "broken":
        return f'<span class="fui-secret-now is-broken">{icon("triangle-alert", "12")}set, but <code>{e(ref)}</code> cannot be read</span>'
    return f'<span class="fui-secret-now">{icon("lock", "12")}set, read from <code>{e(ref).replace("/", "/<wbr>")}</code></span>'


def settings_tab(rows, top=""):
    """One Settings tab: one quiet line (where it comes from), then the rows."""
    return tpl.settings_panel(rows, LINE, "", top=top)
