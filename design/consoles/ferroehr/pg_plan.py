# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The planned screens, unreachable from the live pages: the Audit log and
Subscriptions after they move to FerroSYS. Each is a mirror line and Open
in FerroSYS, with no figures of its own."""
from common import icon, page_heading, shell
from ehr import planned


def mirror(title, current, intro, issue, text):
    h = page_heading(title, subtitle=intro)
    btn = (f'<button type="button" class="fui-button fui-button-primary" disabled title="FerroSYS is planned; it does not exist yet">{icon("arrow-right", "16")}Open in FerroSYS</button>')
    body = planned(issue, text) + f'<div class="fui-cluster">{btn}<span class="fui-small fui-muted">FerroSYS is planned; the link opens it once it exists.</span></div>'
    return shell(title, current, h + f'<div class="fui-stack">{body}</div>')


def page_audit_ferrosys():
    return mirror("Audit log", "audit", "The CDR's security audit log: who accessed what, and with what outcome.", "3661",
                  "This CDR's audit log will be kept in FerroSYS, which stores, reviews and serves it; FerroEHR records it and spools it locally.")


def page_subscriptions_ferrosys():
    return mirror("Subscriptions", "subscriptions", "Get a message on the event queue for each change that matches.", "3382",
                  "This CDR's subscriptions will be managed in FerroSYS; the CDR keeps publishing what they select.")
