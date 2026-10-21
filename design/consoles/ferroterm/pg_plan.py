# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The P group: proposed screens, reachable from no live page. Each carries
the Proposed mark and says what it would need. The synchronisation run
records follow the shape the sync service writes today (GET /runs and
/runs/{id}); the figures are a synthetic example of that shape."""
import tpl
from common import MENU_SEP, action_group, button, e, icon, link_button, menu_item, page_heading, shell, table, tabbed, tabs, td, th, when
from term import coding, facts, load_report, proposed, section, sections, state

RUNS = [("2026-10-21T03:00", "schedule", "ok", "1 release taken, activated", "1 finding"),
        ("2026-10-20T03:00", "schedule", "ok", "nothing new", "none"),
        ("2026-10-19T14:12", "POST /run", "bad", "build failed, nothing activated", "not run"),
        ("2026-10-19T03:00", "schedule", "ok", "nothing new", "none")]


def mark(text):
    return proposed(text + " The figures are a synthetic example of the run record the sync service writes.")


def runs_rows():
    out = []
    for i, (w, trig, tone, what, f) in enumerate(RUNS):
        d, hm = w.split("T")
        out.append(f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="sync-run.html">{d} {hm}</a></th>'
                   + td("Trigger", "Schedule" if trig == "schedule" else "Run now", role="sub") + td("Outcome", state(tone, "Succeeded" if tone == "ok" else "Failed"), role="meta-2")
                   + td("What it did", what, role="wide") + td("Findings", f, role="meta-q") + "</tr>")
    return table("Runs", th("Run", "n", "descending") + th("Trigger") + th("Outcome") + th("What it did") + th("Findings"), out)


def runs_page(overlay=None, more_open=False):
    items = [menu_item("Activate what is staged…", "play", href="sync-run-now.html", desc="Nothing is staged now: activation is auto")]
    ag = action_group(button("primary", "Run now", "refresh-cw"), items, open_=more_open, label="More synchronisation actions")
    h = page_heading("Synchronisation", actions=ag, subtitle="The runs the synchronisation service made, newest first, read from its admin listener.")
    top = mark("A Synchronisation page in the Console group, reading GET /runs, and Run now and Activate, which need the console's forward allow-list widened to POST /run and /activate.")
    c = tpl.card(runs_rows(), label="Runs", title="Runs", count=4)
    return shell("Synchronisation", "", h + f'<div class="fui-list-page">{top}{c}</div>', overlay=overlay)


def page_sync_runs():
    return runs_page()


def run_heading():
    ag = action_group(button("primary", "Download the record", "download"), [])
    return page_heading("Run of 2026-10-21 03:00", actions=ag, subtitle='Started by the schedule; took 4 min 12 s.', crumbs=[("Synchronisation", "sync-runs.html")], badge=state("ok", "Succeeded"))


RUN_TABS = [("summary", "Summary", "sync-run.html"), ("findings", "Findings", "sync-run-findings.html")]


def page_sync_run():
    sm = facts([("Trigger", "The schedule, daily at 03:00 Europe/Amsterdam"), ("Started", when("2026-10-21T03:00")), ("Ended", when("2026-10-21T03:04")),
                ("Source", "nts · the feed offered 412 entries, the FHIR API listed 1,920 resources"),
                ("Taken", "Synthetic national findings 2026-10, 3.1 MB, digest verified"),
                ("Left behind", "411 entries: 388 outside your subscription, 22 releases you already serve, 1 delta rather than a snapshot"),
                ("Activation", "Reloaded once, the server answered 200; retention removed release 2026-04"),
                ("Revalidation", "12 locally written resources read, 214 codes checked, 1 finding")])
    body = sections(section("", mark("The run record, Summary and Findings, from GET /runs/{id}.")), section("What the run did", sm))
    return shell("Run of 2026-10-21 03:00", "", run_heading() + f'<div class="fui-stack">{tabbed(tabs("Views of this run", RUN_TABS, "summary"), body)}</div>')


def page_sync_run_findings():
    rows = [f'<tr><th scope="row" data-role="title"><a class="fui-row-link" href="value-set-finding.html">Referral reasons</a></th>'
            + td("Code", coding("Shortness of breath at rest", "NF-204", "Synthetic national findings"), role="sub")
            + td("What changed", "The release made this code inactive", role="meta-q") + td("Release", "2026-10", role="meta-2") + "</tr>"]
    t = table("What the release did", th("Resource") + th("Code") + th("What changed") + th("Release"), rows)
    body = sections(section("", mark("The findings, linked to each resource; the resource shows a read-only line for each.")),
                    section("What the release did to your own content", f'{t}', count=1))
    return shell("Run of 2026-10-21 03:00", "", run_heading() + f'<div class="fui-stack">{tabbed(tabs("Views of this run", RUN_TABS, "findings"), body)}</div>')


def page_sync_run_now():
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="run-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="run-title" class="fui-dialog-title">{icon("refresh-cw", "18")} Run the synchronisation now?</h2>'
        '<p>The service reads the feed, takes what you subscribed to and do not hold, builds it, and asks the server to reload. A run that fails leaves the served set as it was. The next scheduled run still starts at 03:00.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Run now", "refresh-cw")}</div></div>'
    )
    return runs_page(overlay=dialog)


def page_reload():
    import pg_console
    dialog = (
        '<div role="dialog" aria-modal="true" aria-labelledby="reload-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="reload-title" class="fui-dialog-title">{icon("refresh-cw", "18")} Reload the served set?</h2>'
        '<p>The server opens every artifact and resource directory again and swaps the whole set only when every one opens. A request in flight finishes on the old set. The resources written here stay as they are.</p>'
        + proposed("A Reload action on Server, through the admin listener's POST /reload, which needs the admin scope and a console route it does not have today.")
        + f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", "Reload the served set", "refresh-cw")}</div></div>'
    )
    html = pg_console.page_server()
    return (html.replace('</div></body></html>', f'<div class="mk-scrim" aria-hidden="true"></div>{dialog}</div></body></html>')
            .replace('class="fui-page"', 'class="fui-page mk-page-overlaid"', 1))


def page_delete():
    import pg_res
    items = pg_res.vs_more() + [MENU_SEP, menu_item("Delete this value set…", "trash-2", danger=True)]
    from common import page_heading as ph
    from term import canon, status_mark
    ag = action_group(link_button("primary", "Edit", "value-set-edit.html", "pencil"), items, open_=True, label="More actions for Synthetic respiratory findings")
    h = ph("Synthetic respiratory findings", actions=ag, subtitle=f'{canon(pg_res.RESP[2])} · Version 2 · written here', crumbs=[("Value sets", "value-sets.html")], badge=status_mark("active"))
    note = section("", proposed("Delete for a resource written here, last in More after a separator, behind a confirmation. The server takes DELETE today; the console offers none."))
    panel = f'{sections(note)}{pg_res.vs_definition()}'
    return pg_res.vs_page("definition", panel, h)
