# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Events: the change-feed consumers and the AMQP topology, one T2 page with
no tabs. Each consumer's source, reader and position, its state, what it
published since the bridge started and its last failure; the events a
consumer parked in the dead-letter store, each with a Replay form that posts
to the page; then the exchange, the routing-key grammar and the bound
queues. The page is read-only apart from Replay: the topology changes in
the configuration. No envelope body is ever shown."""
from common import badge, button, definitions, e, notice, page_heading, shell, table, td, th, when
from bridge import quiet, section, sections, state

# The consumers as GET /operator/events reports them: name, reader, source,
# positions, state, published, last published, last failure.
FOLLOWING = [
    ("fhir-notify", "ferrobridge-events", "FerroEHR outbox feed", "clinical seq 18204; party seq 412", "ok", "1,284", "2026-10-21T09:41:58"),
    ("omop-incremental", "ferrobridge-omop", "AQL poll of the CDR", "2026-10-21T09:40:00Z, 3 versions at that instant", "ok", "0", None),
]

STALLED_CAUSE = "no context maps the template demo.vital_signs.v2"

# The parked events: consumer, reader, id, position, contribution, cause,
# passes, parked at. A row names its consumer over the position it holds.
PARKED = [
    ("fhir-notify", "ferrobridge-events", 7, "clinical seq 18205", "6f1c2a90-3b4e-4d8a-9f17-2c5e8b0d4a61", STALLED_CAUSE, "5", "2026-10-21T09:38:12"),
]

QUEUES = [("ferroehr.events.compositions", "COMPOSITION.#"), ("ferroehr.events.audit", "#")]


def events_heading(tone=None, word=None, off=False):
    sub = ("Off, so no consumer follows the change feed." if off
           else "Follows the CDR's commit events, each consumer acknowledging only what it finished.")
    return page_heading("Events", subtitle=sub, badge=badge("neutral", "Off") if off else state(tone, word))


def consumers_table(stalled=False):
    rows = []
    for name, reader, source, position, tone, published, last in FOLLOWING:
        st = state("warn", "degraded") if stalled and name == "fhir-notify" else state(tone, "following")
        pub = "1,279" if stalled and name == "fhir-notify" else published
        last_html = when(last, seconds=True) if last else "–"
        rows.append(f'<tr><th scope="row" data-role="title">{e(name)}<span class="fui-row-sub"><code>{e(reader)}</code></span></th>'
                    + td("Source", e(source), role="sub") + td("Position", e(position)) + td("State", st)
                    + td("Published", pub, cls="fui-num") + td("Last published", last_html) + "</tr>")
    head = th("Consumer") + th("Source") + th("Position") + th("State") + th("Published", num=True) + th("Last published")
    return table("Consumers", head, rows)


def failures(stalled=False):
    if not stalled:
        return quiet("No pass has failed since the bridge started.")
    return definitions([("Consumer", "fhir-notify"), ("When", when("2026-10-21T09:38:12", seconds=True)), ("Cause", e(STALLED_CAUSE))])


def replay_form(name, number):
    label = f"Replay parked event {number} of {name}"
    return (f'<form method="post" action="events-replayed.html" class="fui-action-form">'
            f'<input type="hidden" name="consumer" value="{e(name)}"><input type="hidden" name="id" value="{number}">'
            + button("secondary", "Replay", glyph="rotate-ccw", type_="submit", attrs=f'aria-label="{e(label)}"') + "</form>")


def parked_table(requested=False):
    rows = []
    for name, reader, number, position, contribution, cause, passes, at in PARKED:
        act = state("neutral", "replay requested") if requested else replay_form(name, number)
        rows.append(f'<tr><th scope="row" data-role="title">{e(name)}<span class="fui-row-sub">{e(position)}</span></th>'
                    + td("Contribution", f"<code>{e(contribution)}</code>") + td("Cause", e(cause))
                    + td("Passes", passes, cls="fui-num") + td("Parked", when(at, seconds=True)) + td("Actions", act, cls="fui-table-actions") + "</tr>")
    head = th("Consumer") + th("Contribution") + th("Cause") + th("Passes", num=True) + th("Parked") + th("Actions")
    return table("Parked events", head, rows)


def topology():
    facts = definitions([("Exchange", "<code>ferroehr.events</code>"), ("Routing key", "<code>&lt;kind&gt;.&lt;change_type&gt;.&lt;template_id&gt;</code>")])
    rows = [f'<tr><th scope="row" data-role="title"><code>{e(q)}</code></th>' + td("Binding", f"<code>{e(b)}</code>") + "</tr>" for q, b in QUEUES]
    queues = table("Bound queues", th("Queue") + th("Binding"), rows)
    return section("Topology", facts) + section("Bound queues", queues, len(QUEUES))


def events_page(stalled=False, parked=False, requested=False, answer=""):
    tone, word = ("warn", "degraded") if stalled else ("ok", "following")
    parked_body = parked_table(requested) if parked else quiet("No event is parked: every event a consumer read, it finished.")
    body = sections(section("Consumers", consumers_table(stalled), len(FOLLOWING)), section("Last failure", failures(stalled)),
                    section("Parked events", parked_body, len(PARKED) if parked else 0), topology())
    return shell("Events", "events", events_heading(tone, word) + f'<div class="fui-stack">{answer}{body}</div>')


def page_events():
    return events_page()


def page_events_degraded():
    return events_page(stalled=True, parked=True)


def page_events_replayed():
    answer = notice("ok", "check", "Replay requested: the consumer offers the event again before its next page.")
    return events_page(stalled=True, parked=True, requested=True, answer=answer)


def page_events_off():
    body = quiet('No commit event is published to a broker. <a href="configuration.html">See the running configuration</a>')
    return shell("Events", "events", events_heading(off=True) + f'<div class="fui-stack">{body}</div>')
