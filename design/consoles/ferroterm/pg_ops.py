# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Operations, one workbench like FerroBRIDGE's Playground: binder tabs
Look up | Check | Expand | Translate, each its own route with its
parameters in the address. B5's options row with Run, then the answer as
panel sections: the verdict as B4, the details, and the request. The runs
made on the page are kept in the page only."""
from common import (
    busy_button, button, copy_button, disclosure, e, empty_quiet, field, icon, input_, link_button, notice, page_heading, pager,
    refusal, select, shell, table, tabbed, tabs, td, th,
)
from data import FHIR_BASE, SF
from term import codeview, coding, load_report, relation, section, sections

SUB = "Run the server's terminology operations; nothing is kept."
TABS = [("lookup", "Look up", "lookup.html"), ("check", "Check", "check.html"), ("expand", "Expand", "expand.html"), ("translate", "Translate", "translate.html")]

SYS = [("sf", "Synthetic clinical findings"), ("care", "Synthetic care settings"), ("national", "Synthetic national findings"), ("iso", "ISO 3166-1 country codes"), ("gender", "AdministrativeGender")]
VS = [("resp", "Synthetic respiratory findings"), ("sfall", "Every synthetic finding"), ("referral", "Referral reasons"), ("benelux", "Benelux countries"), ("loinc", "Every LOINC code (implicit)")]


def run_button(label, running=False, busy_word="Running", glyph="play", op=None):
    tip = f' title="Runs {e(op)}"' if op else ""
    b = busy_button("primary", busy_word) if running else button("primary", label, glyph, keys="Ctrl Enter", type_="submit", attrs=tip)
    return f'<div class="fui-transform-run">{b}</div>'


def opts(fields, run):
    return f'<div class="fui-transform-options is-dense">{fields}{run}</div>'


def request_section(url, doc=None, doc_title="The Parameters, as the server sent them"):
    line = f'<p class="fui-alert-request"><code>{e(url).replace("/", "/<wbr>").replace("&amp;", "&amp;<wbr>")}</code></p>'
    body = f'<div class="fui-request">{line}{copy_button("Copy", "the request")}</div>'
    if doc:
        body += disclosure(doc_title, codeview(doc_title, doc, '<span class="fui-tag">FHIR R4</span>'))
    return section("The request", body)


def runs_section(open_=False):
    rows = [("Look up", "SF-1211 in Synthetic clinical findings", "lookup-answered.html"),
            ("Check", "SF-1220 in Synthetic respiratory findings", "check-invalid.html"),
            ("Expand", "Every synthetic finding, 10 a page", "expand-answered.html")]
    items = "".join(f'<li><a href="{h}">{e(s)}</a> <span class="fui-muted">· {e(w)}</span></li>' for s, w, h in rows)
    body = f'<p class="fui-small fui-muted">Kept in this page only, never sent anywhere.</p><ul class="fui-bullets">{items}</ul>'
    return disclosure("Recent runs in this browser · 3", body, open_)


def ops_page(tab, panel, overlay=None, runs=True, runs_open=False):
    h = page_heading("Operations", subtitle=SUB)
    extra = runs_section(runs_open) if runs else ""
    return shell("Operations", "operations", h + f'<div class="fui-stack">{tabbed(tabs("Operations", TABS, tab), panel)}{extra}</div>', overlay=overlay)


def answer(*parts, idle=None):
    if idle:
        return f'<p class="fui-transform-empty">{e(idle)}</p>'
    return "".join(parts)


def panel(options_html, answer_html):
    return f'<div class="fui-panel">{options_html}{answer_html}</div>'


# ---- Look up ----

def lookup_opts(code="SF-1211", running=False, error=None):
    f = (field("lk-sys", "Code system", select("lk-sys", SYS, "sf"))
         + field("lk-ver", "Version", select("lk-ver", [("", "Default"), ("3", "3")], ""))
         + field("lk-code", "Code", input_("lk-code", code, "As the system writes it", extra='class="fui-input fui-mono"' + (' aria-invalid="true"' if error else "")), error=error)
         + field("lk-lang", "Display language", input_("lk-lang", "", "for example en or nl-NL")))
    return opts(f, run_button("Look up", running, "Looking the code up", "search", "CodeSystem/$lookup"))


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


def lookup_answer():
    v = load_report("ok", "Found <strong>Dry cough</strong> · <code>SF-1211</code> in Synthetic clinical findings, version 3",
                    actions=link_button("secondary", "Open in the concept browser", "browse-keyboard.html", "network", small=True))
    d = table("Designations", th("Term") + th("Language") + th("Use"),
              [f'<tr><th scope="row" data-role="title">Dry cough</th>{td("Language", "English (en)", role="sub")}{td("Use", "display", role="meta-q")}</tr>',
               f'<tr><th scope="row" data-role="title">Droge hoest</th>{td("Language", "Dutch (nl)", role="sub")}{td("Use", "display", role="meta-q")}</tr>'])
    p = table("Properties", th("Property") + th("Value"),
              [f'<tr><th scope="row" data-role="title"><code>parent</code></th>{td("Value", coding("Cough", "SF-1210", inline=True), role="sub")}</tr>',
               f'<tr><th scope="row" data-role="title"><code>inactive</code></th>{td("Value", "No", role="sub")}</tr>'])
    return (section("", v) + section("Designations", f'{d}', count=2)
            + section("Properties", f'{p}', count=2) + request_section(LK_URL, LOOKUP_JSON))


def page_lookup():
    return ops_page("lookup", panel(lookup_opts(""), answer(idle="Choose a code system, type a code and look it up.")), runs=False)


def page_lookup_invalid():
    return ops_page("lookup", panel(lookup_opts("", error="The code to read: nothing is sent until this is filled in."), answer(idle="Choose a code system, type a code and look it up.")), runs=False)


def page_lookup_running():
    return ops_page("lookup", panel(lookup_opts(running=True), '<div class="fui-panel-section"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Looking the code up</span></div>'))


def page_lookup_answered(overlay=None):
    return ops_page("lookup", panel(lookup_opts(), lookup_answer()), overlay=overlay)


def page_lookup_runs():
    return ops_page("lookup", panel(lookup_opts(), lookup_answer()), runs_open=True)


def page_lookup_not_found():
    rf = refusal("The server refused the request.", request=f"GET {FHIR_BASE}/CodeSystem/$lookup?system={SF}&code=SF-1299",
                 issues=[("bad", "error", "not-found", "Unknown code SF-1299 in the code system https://terminology.example.org/fhir/CodeSystem/synthetic-findings, version 3.")])
    return ops_page("lookup", panel(lookup_opts("SF-1299"), section("", rf)))


# ---- Check ----

def seg(mode):
    segs = "".join(f'<label class="fui-segment"><input type="radio" class="fui-sr-only" name="ck-mode"{" checked" if v == mode else ""} />{w}</label>'
                   for v, w in (("membership", "Membership"), ("relation", "Relation")))
    return f'<div class="fui-field"><span class="fui-label" id="ck-mode-l">What to check</span><div role="radiogroup" aria-labelledby="ck-mode-l" class="fui-segmented">{segs}</div></div>'


def check_opts(code="SF-1211", vs="resp", running=False):
    f = (seg("membership")
         + field("ck-against", "Check against", select("ck-against", [("vs", "A value set"), ("cs", "A code system")], "vs"))
         + field("ck-vs", "Value set", select("ck-vs", VS, vs))
         + field("ck-code", "Code", input_("ck-code", code, "As the system writes it", extra='class="fui-input fui-mono"'))
         + field("ck-sys", "Its code system", select("ck-sys", SYS, "sf")))
    return opts(f, run_button("Check", running, "Checking", "shield-check", "ValueSet/$validate-code"))


def ck_url(code):
    return f"GET {FHIR_BASE}/ValueSet/$validate-code?url=https://terminology.example.org/fhir/ValueSet/synthetic-respiratory&system={SF}&code={code}"


def page_check():
    return ops_page("check", panel(check_opts(""), answer(idle="Name a code and the resource to check it against, then run it.")), runs=False)


def page_check_valid():
    v = load_report("ok", "Valid: <strong>Dry cough</strong> · <code>SF-1211</code> is in Synthetic respiratory findings", glyph="circle-check")
    echo = section("The concept the server echoed", coding("Dry cough", "SF-1211", "Synthetic clinical findings, version 3"))
    return ops_page("check", panel(check_opts(), section("", v) + echo + request_section(ck_url("SF-1211"))))


def page_check_invalid():
    v = load_report("bad", "Not valid: <strong>Wheeze</strong> · <code>SF-1220</code> is not in Synthetic respiratory findings", glyph="circle-x")
    iss = section("The issues the server itemised", '<ul class="fui-issues"><li><span class="fui-badge fui-tone-bad">error</span><span class="fui-issue-head"><code>not-in-vs</code>:</span>'
                  '<span class="fui-issue-text">The code SF-1220 from Synthetic clinical findings version 3 is not in the value set: an exclude takes it back out.</span></li></ul>', count=1)
    return ops_page("check", panel(check_opts("SF-1220"), section("", v) + iss + request_section(ck_url("SF-1220"))))


def page_check_inactive():
    v = load_report("warn", "Valid, and inactive: <strong>Malaise, old code</strong> · <code>SF-1390</code> is in Every synthetic finding", glyph="triangle-alert")
    n = section("", '<p class="fui-small">Inactive in its code system: the server answered and marked it, which is not a refusal.</p>')
    return ops_page("check", panel(check_opts("SF-1390", "sfall"), section("", v) + n + request_section(ck_url("SF-1390").replace("synthetic-respiratory", "synthetic-findings-all"))))


def relation_opts(sysv="sf", a="SF-1200", b="SF-1211", running=False):
    f = (seg("relation") + field("rl-sys", "Code system", select("rl-sys", SYS, sysv))
         + field("rl-a", "Code A", input_("rl-a", a, extra='class="fui-input fui-mono"')) + field("rl-b", "Code B", input_("rl-b", b, extra='class="fui-input fui-mono"')))
    return opts(f, run_button("Check the relation", running, "Checking", "git-branch", "CodeSystem/$subsumes"))


def page_check_relation():
    v = load_report("ok", "<code>SF-1200</code> Finding of the chest subsumes <code>SF-1211</code> Dry cough: SF-1211 is a descendant of SF-1200.", glyph="git-branch")
    url = f"GET {FHIR_BASE}/CodeSystem/$subsumes?system={SF}&codeA=SF-1200&codeB=SF-1211"
    return ops_page("check", panel(relation_opts(), section("", v) + request_section(url)))


def page_check_relation_unsupported():
    f = seg("relation") + field("rl-sys", "Code system", select("rl-sys", SYS, "care"))
    n = section("", notice("info", "info", "Synthetic care settings declares that it does not answer subsumption, so there is no relation to check in it. Choose a code system with a hierarchy, such as Synthetic clinical findings."))
    return ops_page("check", panel(f'<div class="fui-transform-options">{f}</div>', n), runs=False)


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
    return ops_page("expand", panel(expand_opts("resp", ""), answer(idle="Choose a value set and expand it.")), runs=False)


def page_expand_running():
    return ops_page("expand", panel(expand_opts(running=True), '<div class="fui-panel-section"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Running the expansion</span></div>'))


def page_expand_answered():
    v = load_report("ok", "Concepts 1 to 10 of <strong>14</strong> in Every synthetic finding", glyph="list-tree")
    rows = [f'<tr><th scope="row" data-role="title">{coding(d, c, inline=True)}</th>{td("System", "Synthetic clinical findings", role="sub")}{td("Version", "3", role="meta-q")}</tr>' for c, d in SF_ROWS]
    t = table("The concepts of this page", th("Concept") + th("System") + th("Version"), rows)
    pg = pager("Expansion pages", 14, 10, 1, sizes=(10, 25, 100))
    params = section("The parameters the server says it applied", '<dl class="fui-facts-row"><div><dt>count</dt><dd><code>10</code></dd></div><div><dt>offset</dt><dd><code>0</code></dd></div><div><dt>activeOnly</dt><dd><code>false</code></dd></div><div><dt>displayLanguage</dt><dd><span class="fui-muted">not sent</span></dd></div></dl>')
    url = f"GET {FHIR_BASE}/ValueSet/$expand?url=https://terminology.example.org/fhir/ValueSet/synthetic-findings-all&count=10&offset=0"
    return ops_page("expand", panel(expand_opts(), section("", v) + section("The concepts of this page", f'{t}<div class="fui-list-foot">{pg}</div>', count=10) + params + request_section(url)))


def page_expand_too_costly():
    rf = refusal("The server refused to expand a selection this large in one answer.", request=f"GET {FHIR_BASE}/ValueSet/$expand?url=http://loinc.org/vs",
                 issues=[("bad", "error", "too-costly", "The expansion has more than 1,000 members. Ask for a page size and the runner walks it a page at a time.")])
    rf += f'<div class="fui-cluster">{button("primary", "Run with a page size of 100", "play")}</div>'
    return ops_page("expand", panel(expand_opts("loinc", ""), section("", rf)))


# ---- Translate ----

def translate_opts(code="LF-04", running=False):
    f = (field("tr-code", "Code", input_("tr-code", code, extra='class="fui-input fui-mono"'))
         + field("tr-sys", "Its code system", select("tr-sys", [("legacy", "Legacy findings"), ("care", "Synthetic care settings"), ("gender", "AdministrativeGender")], "legacy"))
         + field("tr-map", "The map", select("tr-map", [("", "Any map"), ("legacy", "Legacy findings to synthetic findings")], "legacy"))
         + field("tr-target", "Target code system", select("tr-target", [("", "Any"), ("sf", "Synthetic clinical findings")], "")))
    return opts(f, run_button("Translate", running, "Translating", "corner-down-right", "ConceptMap/$translate"))


def tr_url(code):
    return f"GET {FHIR_BASE}/ConceptMap/$translate?url=https://terminology.example.org/fhir/ConceptMap/legacy-findings&system=https://terminology.example.org/fhir/CodeSystem/legacy-findings&code={code}"


def page_translate():
    return ops_page("translate", panel(translate_opts(""), answer(idle="Name a code and the system it belongs to, then translate it.")), runs=False)


def page_translate_answered():
    v = load_report("ok", "Translated <code>LF-04</code> Chest complaint through Legacy findings to synthetic findings: <strong>1 match</strong>", glyph="corner-down-right")
    t = table("The matches the server reported", th("Target") + th("Relationship") + th("From map"),
              [f'<tr><th scope="row" data-role="title">{coding("Wheeze", "SF-1220", "Synthetic clinical findings")}</th>{td("Relationship", relation("broader than the target"), role="sub")}'
               f'{td("From map", "<a href=" + chr(34) + "concept-map.html" + chr(34) + ">Legacy findings to synthetic findings</a>", role="meta-q")}</tr>'])
    cm = section("What came with it", '<p class="fui-small">Target comment: Only the wheeze part of the old code survives.</p>')
    return ops_page("translate", panel(translate_opts(), section("", v) + section("The matches the server reported", f'{t}', count=1) + cm + request_section(tr_url("LF-04"))))


def page_translate_nomap():
    v = load_report("neutral", "<code>LF-07</code> Feeling unwell maps to nothing in the target: the map says so on purpose (noMap).", glyph="ban")
    return ops_page("translate", panel(translate_opts("LF-07"), section("", v) + request_section(tr_url("LF-07"))))


def page_translate_no_match():
    v = load_report("neutral", "The server reported no match for <code>LF-99</code>: no map it holds names that code.", glyph="search-x")
    return ops_page("translate", panel(translate_opts("LF-99"), section("", v) + request_section(tr_url("LF-99"))))
