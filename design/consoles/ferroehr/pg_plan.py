# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The planned screens, unreachable from the live pages: the Audit log and
Subscriptions after they move to FerroSYS. Each is a mirror line and Open
in FerroSYS, with no figures of its own."""
from common import link_button, page_heading, shell
from ehr import planned


def mirror(title, current, intro, issue, text, what):
    h = page_heading(title, subtitle=intro)
    body = (planned(issue, text)
            + f'<section class="fui-card fui-panel"><div class="fui-panel-body fui-stack"><p>{what}</p>'
            f'<div class="fui-cluster">{link_button("primary", "Open in FerroSYS", "#", "arrow-right", attrs="rel=" + chr(34) + "external" + chr(34))}</div></div></section>')
    return shell(title, current, h + f'<div class="fui-stack">{body}</div>')


def page_audit_ferrosys():
    return mirror("Audit log", "audit", "The CDR's security audit trail: who accessed what, and with what outcome.", "3661",
                  "FerroSYS stores, reviews, retains and serves the access log; FerroEHR records and spools it locally.",
                  "This CDR's audit trail will be kept in FerroSYS, read-only from here.")


def page_subscriptions_ferrosys():
    return mirror("Subscriptions", "subscriptions", "Get a message on the event queue for each change that matches.", "3382",
                  "Notification subscriptions move to FerroSYS; the CDR keeps publishing what they select.",
                  "This CDR's subscriptions will be managed in FerroSYS, read-only from here.")
