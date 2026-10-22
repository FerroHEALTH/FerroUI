# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Operations, one workbench like FerroBRIDGE's Playground: binder tabs
Look up | Check | Expand | Translate, each its own route with its
parameters in the address. B5's options row with Run, then the answer as
panel sections: the verdict as B4, the details, and the request. The runs
made here are kept in this browser only."""
from common import (
    busy_button, button, copy_button, disclosure, e, empty_quiet, field, icon, input_, link_button, notice, page_heading, pager,
    refusal, select, shell, table, tabbed, tabs, td, th,
)
from data import FHIR_BASE, REL_WORD, SF
from pg_browse import language_field, version_field
from term import codeview, coding, inactive_mark, load_report, relation, section, sections

SUB = "Run the server's terminology operations; nothing is saved on the server."
TABS = [("lookup", "Look up", "lookup.html"), ("check", "Check", "check.html"), ("expand", "Expand", "expand.html"), ("translate", "Translate", "translate.html")]

SYS = [("sf", "Synthetic clinical findings"), ("care", "Synthetic care settings"), ("national", "Synthetic national findings"), ("iso", "ISO 3166-1 country codes"), ("gender", "AdministrativeGender")]
VS = [("resp", "Synthetic respiratory findings"), ("sfall", "Every synthetic finding"), ("referral", "Referral reasons"), ("benelux", "Benelux countries"), ("loinc", "Every LOINC code (implicit)")]


def run_button(label, running=False, busy_word="Running", glyph="play", op=None):
    tip = f' title="Runs {e(op)}"' if op else ""
    if running:
        b = busy_button("primary", busy_word) + button("secondary", "Cancel", "x")
    else:
        b = button("primary", label, glyph, keys="Ctrl Enter", type_="submit", attrs=tip)
    return f'<div class="fui-transform-run">{b}</div>'


def opts(fields, run, error=None):
    """The options row; a field's error sits under the whole row, so the row
    and its Run keep their places in every state."""
    er = f'<p class="fui-help fui-error mk-options-error">{icon("circle-alert", "14")} {error}</p>' if error else ""
    return f'<div class="fui-transform-options is-dense">{fields}{run}</div>{er}'


def request_section(url, doc=None, doc_title="The Parameters, as the server sent them"):
    line = f'<p class="fui-alert-request"><code>{e(url).replace("/", "/<wbr>").replace("&amp;", "&amp;<wbr>")}</code></p>'
    body = f'<div class="fui-request">{line}{copy_button("Copy", "the request")}</div>'
    if doc:
        body += disclosure(doc_title, codeview("Parameters", doc, '<span class="fui-tag">FHIR R4</span>', doc_title))
    return section("The request", body)


RUNS = [("Look up", "SF-1211 in Synthetic clinical findings", "lookup-answered.html"),
        ("Check", "SF-1220 against the value set Synthetic respiratory findings", "check-invalid.html"),
        ("Expand", "Every synthetic finding, 10 a page", "expand-answered.html")]


def runs_section(open_=False, current=None):
    """Recent runs in this browser, the run just made first."""
    rows = ([current] if current else []) + [r for r in RUNS if r != current]
    rows = rows[:4]
    items = "".join(f'<li><a href="{h}">{e(s)}</a> <span class="fui-muted">· {e(w)}</span></li>' for s, w, h in rows)
    body = f'<p class="fui-small fui-muted">Kept in this browser only, never sent to the server.</p><ul class="fui-bullets">{items}</ul>'
    return disclosure(f"Recent runs in this browser · {len(rows)}", body, open_)


def ops_page(tab, panel, overlay=None, runs=True, runs_open=False, current=None):
    h = page_heading("Operations", subtitle=SUB)
    extra = runs_section(runs_open, current) if runs else ""
    return shell("Operations", "operations", h + f'<div class="fui-stack">{tabbed(tabs("Operations", TABS, tab), panel)}{extra}</div>', overlay=overlay)


def answer(*parts, idle=None):
    if idle:
        return f'<p class="fui-transform-empty">{e(idle)}</p>'
    return "".join(parts)


def working(text="The server is working on it."):
    return f'<p class="fui-transform-empty">{e(text)}</p>'


def panel(options_html, answer_html):
    return f'<div class="fui-panel">{options_html}{answer_html}</div>'


# ---- Look up ----

def lookup_opts(code="SF-1211", running=False, error=None):
    f = (field("lk-sys", "Code system", select("lk-sys", SYS, "sf"))
         + version_field("lk-ver", "sf", "3")
         + field("lk-code", "Code", input_("lk-code", code, "The system's code", extra='class="fui-input fui-mono"' + (' aria-invalid="true" aria-describedby="lk-code-error"' if error else "")))
         + language_field("lk-lang"))
    return opts(f, run_button("Look up", running, "Looking up", "search", "CodeSystem/$lookup"), error)


LOOKUP_JSON = """{
  "resourceType": "Parameters",
  "parameter": [
    { "name": "name", "valueString": "SyntheticClinicalFindings" },
    { "name": "version", "valueString": "3" },
    { "name": "display", "valueString": "Dry cough" },
    { "name": "designation", "part": [
      { "name": "language", "valueCode": "nl" },
      { "name": "value", "valueString": "Droge hoest" }
    ] },
    { "name": "property", "part": [
      { "name": "code", "valueCode": "parent" },
      { "name": "value", "valueCode": "SF-1210" }
    ] },
    { "name": "property", "part": [
      { "name": "code", "valueCode": "inactive" },
      { "name": "value", "valueBoolean": false }
    ] }
  ]
}"""

LK_URL = f"GET {FHIR_BASE}/CodeSystem/$lookup?system={SF}&code=SF-1211"
LK_RUN = ("Look up", "SF-1211 in Synthetic clinical findings", "lookup-answered.html")


def prop_label(word, code):
    return f'{word}<span class="fui-row-sub"><code>{e(code)}</code></span>'


def lookup_answer():
    v = load_report("ok", "Found <strong>Dry cough</strong> · <code>SF-1211</code> in Synthetic clinical findings, version 3",
                    actions=link_button("secondary", "Open in the concept browser", "browse-keyboard.html", "network", small=True))
    disp = section("Display", '<p>Dry cough</p>')
    d = table("Designations", th("Term") + th("Language") + th("Use"),
              [f'<tr><th scope="row" data-role="title">Droge hoest</th>{td("Language", "Dutch (nl)", role="sub")}{td("Use", "Display", role="meta-q")}</tr>'])
    p = table("Properties", th("Property") + th("Value"),
              [f'<tr><th scope="row" data-role="title">{prop_label("Parent", "parent")}</th>{td("Value", coding("Cough", "SF-1210", inline=True), role="sub")}</tr>',
               f'<tr><th scope="row" data-role="title">{prop_label("Inactive", "inactive")}</th>{td("Value", "No", role="sub")}</tr>'])
    return (section("", v) + disp + section("Designations", f'{d}', count=1)
            + section("Properties", f'{p}', count=2) + request_section(LK_URL, LOOKUP_JSON))


IDLE_LOOKUP = "Type a code and look it up."


def page_lookup():
    return ops_page("lookup", panel(lookup_opts(""), answer(idle=IDLE_LOOKUP)))


def page_lookup_invalid():
    return ops_page("lookup", panel(lookup_opts("", error="Type a code to look up."), answer(idle=IDLE_LOOKUP)))


def page_lookup_running():
    return ops_page("lookup", panel(lookup_opts(running=True), working()))


def page_lookup_answered(overlay=None):
    return ops_page("lookup", panel(lookup_opts(), lookup_answer()), overlay=overlay, current=LK_RUN)


def page_lookup_runs():
    return ops_page("lookup", panel(lookup_opts(), lookup_answer()), runs_open=True, current=LK_RUN)


def page_lookup_not_found():
    rf = refusal("The server refused the request.", request=f"GET {FHIR_BASE}/CodeSystem/$lookup?system={SF}&code=SF-1299",
                 issues=[("bad", "Error", "not-found", "Unknown code SF-1299 in the code system https://terminology.example.org/fhir/CodeSystem/synthetic-findings, version 3.")])
    return ops_page("lookup", panel(lookup_opts("SF-1299"), section("", rf)), current=("Look up", "SF-1299 in Synthetic clinical findings", "lookup-not-found.html"))


# ---- Check ----

def seg(mode):
    segs = "".join(f'<label class="fui-segment"><input type="radio" class="fui-sr-only" name="ck-mode"{" checked" if v == mode else ""} />{w}</label>'
                   for v, w in (("membership", "Membership"), ("relation", "Relation")))
    return f'<div class="fui-field"><span class="fui-label" id="ck-mode-l">What to check</span><div role="radiogroup" aria-labelledby="ck-mode-l" class="fui-segmented">{segs}</div></div>'


def check_opts(code="SF-1211", vs="resp", running=False):
    f = (seg("membership")
         + field("ck-vs", "Value set", select("ck-vs", VS, vs))
         + field("ck-code", "Code", input_("ck-code", code, "The system's code", extra='class="fui-input fui-mono"'))
         + field("ck-sys", "Its code system", select("ck-sys", SYS, "sf")))
    return opts(f, run_button("Check", running, "Checking", "shield-check", "ValueSet/$validate-code"))


def ck_url(code):
    return f"GET {FHIR_BASE}/ValueSet/$validate-code?url=https://terminology.example.org/fhir/ValueSet/synthetic-respiratory&system={SF}&code={code}"


def page_check():
    return ops_page("check", panel(check_opts(""), answer(idle="Name a code and the value set to check it against, then run it.")))


def page_check_valid():
    v = load_report("ok", "Valid: <strong>Dry cough</strong> · <code>SF-1211</code> is in Synthetic respiratory findings", glyph="circle-check")
    echo = section("Concept", coding("Dry cough", "SF-1211", "Synthetic clinical findings, version 3"))
    return ops_page("check", panel(check_opts(), section("", v) + echo + request_section(ck_url("SF-1211"))),
                    current=("Check", "SF-1211 against the value set Synthetic respiratory findings", "check-valid.html"))


def page_check_invalid():
    v = load_report("bad", "Not valid: <strong>Wheeze</strong> · <code>SF-1220</code> is not in Synthetic respiratory findings", glyph="circle-x")
    iss = section("Issues", '<ul class="fui-issues"><li><span class="fui-badge fui-tone-bad">Error</span><span class="fui-issue-head"><code>not-in-vs</code>:</span>'
                  '<span class="fui-issue-text">The code SF-1220 from Synthetic clinical findings version 3 is not in the value set: an exclude takes it back out.</span></li></ul>', count=1)
    return ops_page("check", panel(check_opts("SF-1220"), section("", v) + iss + request_section(ck_url("SF-1220"))),
                    current=("Check", "SF-1220 against the value set Synthetic respiratory findings", "check-invalid.html"))


def page_check_inactive():
    v = load_report("warn", "Valid: <strong>Malaise, old code</strong> · <code>SF-1390</code> is in Every synthetic finding", glyph="triangle-alert")
    echo = section("Concept", coding("Malaise, old code", "SF-1390", "Synthetic clinical findings, version 3", inactive=True)
                   + '<p class="fui-small">SF-1390 is inactive in its code system, but still in the value set.</p>')
    return ops_page("check", panel(check_opts("SF-1390", "sfall"), section("", v) + echo + request_section(ck_url("SF-1390").replace("synthetic-respiratory", "synthetic-findings-all"))),
                    current=("Check", "SF-1390 against the value set Every synthetic finding", "check-inactive.html"))


def relation_opts(sysv="sf", a="SF-1200", b="SF-1211", running=False):
    f = (seg("relation") + field("rl-sys", "Code system", select("rl-sys", SYS, sysv))
         + field("rl-a", "Code A", input_("rl-a", a, extra='class="fui-input fui-mono"')) + field("rl-b", "Code B", input_("rl-b", b, extra='class="fui-input fui-mono"')))
    return opts(f, run_button("Check the relation", running, "Checking", "git-branch", "CodeSystem/$subsumes"))


def page_check_relation():
    v = load_report("ok", "Dry cough (<code>SF-1211</code>) falls under Finding of the chest (<code>SF-1200</code>).", glyph="git-branch")
    url = f"GET {FHIR_BASE}/CodeSystem/$subsumes?system={SF}&codeA=SF-1200&codeB=SF-1211"
    return ops_page("check", panel(relation_opts(), section("", v) + request_section(url)),
                    current=("Check", "SF-1200 and SF-1211 in Synthetic clinical findings", "check-relation.html"))


def page_check_relation_unsupported():
    f = seg("relation") + field("rl-sys", "Code system", select("rl-sys", SYS, "care"))
    n = section("", notice("info", "info", "Synthetic care settings does not record a hierarchy, so there is no relation to check in it. Choose a code system with one, such as Synthetic clinical findings."))
    return ops_page("check", panel(f'<div class="fui-transform-options is-dense">{f}</div>', n))


# ---- Expand ----

def expand_opts(vs="sfall", count="10", running=False):
    f = (field("ex-vs", "Value set", select("ex-vs", VS, vs))
         + field("ex-filter", "Filter", input_("ex-filter", "", "Text in a designation"))
         + field("ex-count", "Page size", input_("ex-count", count, extra='inputmode="numeric" class="fui-input mk-num-input"'))
         + '<div class="fui-field fui-field-checks">' + '<label class="fui-check"><input type="checkbox" /><span class="fui-check-text"><span class="fui-label">Active concepts only</span></span></label>'
         + '<label class="fui-check"><input type="checkbox" /><span class="fui-check-text"><span class="fui-label">Include designations</span></span></label></div>')
    return opts(f, run_button("Expand", running, "Expanding", "list-tree", "ValueSet/$expand"))


SF_ROWS = [("SF-1000", "Synthetic finding"), ("SF-1100", "Finding of the skin"), ("SF-1110", "Rash"), ("SF-1120", "Itch"), ("SF-1200", "Finding of the chest"),
           ("SF-1210", "Cough"), ("SF-1211", "Dry cough"), ("SF-1212", "Productive cough"), ("SF-1220", "Wheeze"), ("SF-1300", "General finding")]


def page_expand():
    return ops_page("expand", panel(expand_opts("resp", "25"), answer(idle="Expand the value set to see its concepts.")))


def page_expand_running():
    return ops_page("expand", panel(expand_opts(running=True), working()))


def applied(label, key, value):
    return f'<div><dt>{label}</dt><dd>{value} <code class="fui-muted">{key}</code></dd></div>'


def page_expand_answered():
    v = load_report("ok", "Concepts 1 to 10 of <strong>14</strong> in Every synthetic finding", glyph="list-tree")
    rows = [f'<tr><th scope="row" data-role="title">{coding(d, c, inline=True)}</th>{td("System", "Synthetic clinical findings", role="sub")}{td("Version", "3", role="meta-q")}</tr>' for c, d in SF_ROWS]
    t = table("The concepts of this page", th("Concept") + th("System") + th("Version"), rows)
    pg = pager("Expansion pages", 14, 10, 1, sizes=None)
    params = section("Applied parameters", '<dl class="fui-facts-row">' + applied("Page size", "count", "10") + applied("Start at", "offset", "0")
                     + applied("Active concepts only", "activeOnly", "No") + applied("Display language", "displayLanguage", '<span class="fui-muted">Not sent</span>') + "</dl>")
    url = f"GET {FHIR_BASE}/ValueSet/$expand?url=https://terminology.example.org/fhir/ValueSet/synthetic-findings-all&count=10&offset=0"
    return ops_page("expand", panel(expand_opts(), section("", v) + section("Concepts", f'{t}{pg}') + params + request_section(url)),
                    current=("Expand", "Every synthetic finding, 10 a page", "expand-answered.html"))


def page_expand_too_costly():
    rf = refusal("The server refused to expand a selection this large in one answer.", request=f"GET {FHIR_BASE}/ValueSet/$expand?url=http://loinc.org/vs",
                 issues=[("bad", "Error", "too-costly", "The expansion has more than 1,000 members. Ask for a page size and the console reads it a page at a time.")])
    rf += f'<div class="fui-cluster">{button("secondary", "Run with a page size of 100", "play")}</div>'
    return ops_page("expand", panel(expand_opts("loinc", ""), section("", rf)), current=("Expand", "Every LOINC code, in one answer", "expand-too-costly.html"))


# ---- Translate ----

def translate_opts(code="LF-04", running=False):
    f = (field("tr-code", "Code", input_("tr-code", code, "The system's code", extra='class="fui-input fui-mono"'))
         + field("tr-sys", "Its code system", select("tr-sys", [("legacy", "Legacy findings"), ("care", "Synthetic care settings"), ("gender", "AdministrativeGender")], "legacy"))
         + field("tr-map", "The map", select("tr-map", [("", "Any map"), ("legacy", "Legacy findings to synthetic findings")], "legacy"))
         + field("tr-target", "Target code system", select("tr-target", [("", "Any"), ("sf", "Synthetic clinical findings")], "")))
    return opts(f, run_button("Translate", running, "Translating", "corner-down-right", "ConceptMap/$translate"))


def tr_url(code):
    return f"GET {FHIR_BASE}/ConceptMap/$translate?url=https://terminology.example.org/fhir/ConceptMap/legacy-findings&system=https://terminology.example.org/fhir/CodeSystem/legacy-findings&code={code}"


def page_translate():
    return ops_page("translate", panel(translate_opts(""), answer(idle="Name a code and the system it belongs to, then translate it.")))


def page_translate_answered():
    v = load_report("ok", "Translated <code>LF-04</code> Chest complaint through Legacy findings to synthetic findings: <strong>1 match</strong>", glyph="corner-down-right")
    t = table("Matches", th("Target") + th("Relationship"),
              [f'<tr><th scope="row" data-role="title">{coding("Wheeze", "SF-1220", "Synthetic clinical findings")}</th>{td("Relationship", relation(REL_WORD["narrower"]), role="sub")}</tr>'])
    cm = section("Comment", '<p class="fui-small">Only the wheeze part of the old code survives.</p>')
    return ops_page("translate", panel(translate_opts(), section("", v) + section("Matches", f'{t}', count=1) + cm + request_section(tr_url("LF-04"))),
                    current=("Translate", "LF-04 through Legacy findings to synthetic findings", "translate-answered.html"))


def page_translate_nomap():
    v = load_report("neutral", "<code>LF-07</code> Feeling unwell maps to nothing: the map marks it unmatched on purpose.", glyph="ban")
    return ops_page("translate", panel(translate_opts("LF-07"), section("", v) + request_section(tr_url("LF-07"))),
                    current=("Translate", "LF-07 through Legacy findings to synthetic findings", "translate-nomap.html"))


def page_translate_no_match():
    v = load_report("neutral", "Legacy findings to synthetic findings has no mapping for <code>LF-99</code>.", glyph="search-x")
    return ops_page("translate", panel(translate_opts("LF-99"), section("", v) + request_section(tr_url("LF-99"))),
                    current=("Translate", "LF-99 through Legacy findings to synthetic findings", "translate-no-match.html"))
