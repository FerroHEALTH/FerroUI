# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The terminology resources: Code systems, Value sets and Concept maps (T1)
and each resource's own page (T2, the one home of the resource, its
versions and its history). History is a tab only on a resource written
here; a loaded one says once, on its first tab, why it has none."""
import tpl
from common import (
    MENU_SEP, action_group, badge, button, busy_button, e, empty_quiet, empty_state, field, icon, link_button, menu, menu_item,
    notice, page_heading, refusal, select, shell, table, tabbed, tabs, td, th, when,
)
from data import (CODE_SYSTEMS, CONCEPT_MAPS, FINDINGS, LEGACY, LEGACY_MAPPINGS, REL_WORD, SF, SF_HISTORY, VALUE_SETS)
from parts import skeleton_lines
from term import (canon, clause, codeview, coding, facts, inactive_mark, load_report, relation, section, sections, source_tag, state, status_mark)

CONTENT_WORD = {"complete": "All concepts", "not-present": "Not listed"}
CONTENT_TIP = {"complete": "content: complete, every concept is in the resource", "not-present": "content: not-present, the server answers codes by their grammar"}


def content_word(c):
    return f'<span title="{e(CONTENT_TIP[c])}">{CONTENT_WORD[c]}</span>'


def name_cell(name, url, href):
    return (f'<th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{e(name)}</a>'
            f'<span class="fui-row-sub">{canon(url)}</span></th>')


# ---- Code systems (T1) ----

SRC_ITEMS = [("Written here", None, 3, True), ("Resource directory", None, 5, True), ("Built index", None, 3, True), ("Built in", None, 4, True)]


def cs_rows(items=None, hover=None):
    rows = []
    for i, (key, name, url, vers, src, content, hier, subs, status, n, href) in enumerate(items if items is not None else CODE_SYSTEMS):
        cls = ' class="mk-hover"' if i == hover else ""
        v = e(", ".join(vers)) if vers else '<span class="mk-dash">none declared</span>'
        rows.append(f'<tr{cls}>{name_cell(name, url, href)}'
                    + td("Source", source_tag(src), role="meta-q") + td("Versions", v, role="meta-2")
                    + td("Content", content_word(content), role="wide") + td("Hierarchy", "Tree" if hier else "Flat", role="wide") + "</tr>")
    return table("Code systems", th("Code system", "n", "ascending") + th("Source") + th("Versions") + th("Content") + th("Hierarchy", title="Whether the served version declares an operator to walk its hierarchy by"), rows, scroll_label="Code systems")


def cs_heading(visitor=False):
    act = "" if visitor else action_group(link_button("primary", "New code system", "code-system-new.html", "file-plus"), [])
    return page_heading("Code systems", actions=act, subtitle="Every code system this root declares in its terminology capabilities.")


def cs_page(body=None, value="", source=None, open_=False, chips=(), visitor=False, count="15"):
    items = [(w, t, n, (source is None or w == source)) for w, t, n, _ in SRC_ITEMS]
    tb = tpl.toolbar("Name or canonical", value, [tpl.filter_button("Source", source, open_, items)])
    c = tpl.card(body if body is not None else cs_rows(), label="Code systems", title="Code systems", count=count)
    content = cs_heading(visitor) + tpl.list_body("", tb, tpl.active(chips), c)
    return shell("Code systems", "systems", content, visitor=visitor)


def page_code_systems():
    return cs_page()


def page_code_systems_hover():
    return cs_page(cs_rows(hover=0))


def page_code_systems_filtered():
    written = [c for c in CODE_SYSTEMS if c[4] == "written"]
    return cs_page(cs_rows(written), source="Written here", open_=True, chips=["Source: Written here"], count="3 of 15")


def page_code_systems_no_match():
    body = empty_state("search-x", "No code system matches “ICD-11”",
                       "The filter is empty, not the root: 15 code systems are served on FHIR R4.",
                       button("secondary", "Clear filters", "x"), filtered=True)
    return cs_page(f'<div class="fui-panel-body">{body}</div>', value="ICD-11", chips=["Name or canonical: ICD-11"], count="0 of 15")


def page_code_systems_loading():
    sk = '<div class="fui-panel-body" aria-busy="true">' + "".join(f'{skeleton_lines([60, 90])}' for _ in range(6)) + '</div><p class="fui-sr-only" role="status">Reading the terminology capabilities</p>'
    return cs_page(sk, count="–")


# ---- one code system (T2) ----

CS_TABS_WRITTEN = [("overview", "Overview", "code-system.html"), ("versions", "Versions", "code-system-versions.html"),
                   ("history", "History", "code-system-history.html"), ("json", "JSON", "code-system-json.html")]
CS_TABS_LOADED = [("overview", "Overview", "code-system-builtin.html"), ("versions", "Versions", "code-system-index.html"), ("json", "JSON", "code-system-json.html")]


def cs_more(writable=True, open_=False):
    items = []
    if writable:
        items.append(menu_item("Browse concepts", "network", href="browse.html"))
    items += [menu_item("Look up a code in it", "search", href="lookup.html"), menu_item("Check a code in it", "shield-check", href="check.html"),
              menu_item("Expand as a value set", "list-tree", href="expand.html"), MENU_SEP,
              menu_item("Copy canonical", "copy"), menu_item("Download JSON", "download")]
    return items


def cs_header(name="Synthetic clinical findings", url=SF, status="active", line="Version 3 · written here", writable=True, more_open=False, visitor=False, reader=False):
    if writable and not (visitor or reader):
        prim = link_button("primary", "Edit", "code-system-edit.html", "pencil")
    else:
        prim = link_button("primary", "Browse concepts", "browse.html", "network")
    ag = action_group(prim, cs_more(writable and not (visitor or reader), more_open), open_=more_open, label=f"More actions for {name}")
    return page_heading(name, actions=ag, subtitle=f"{canon(url)} · {line}", crumbs=[("Code systems", "code-systems.html")], badge=status_mark(status))


CS_JSON = """{
  "resourceType": "CodeSystem",
  "id": "synthetic-findings",
  "meta": { "versionId": "3", "lastUpdated": "2026-10-21T06:41:40Z" },
  "url": "https://terminology.example.org/fhir/CodeSystem/synthetic-findings",
  "version": "3",
  "name": "SyntheticClinicalFindings",
  "title": "Synthetic clinical findings",
  "status": "active",
  "experimental": true,
  "publisher": "Synthetic data team",
  "description": "Synthetic findings for testing value sets and maps. No clinical meaning.",
  "caseSensitive": true,
  "hierarchyMeaning": "is-a",
  "content": "complete",
  "count": 14,
  "property": [
    { "code": "parent", "type": "code" },
    { "code": "inactive", "uri": "http://hl7.org/fhir/concept-properties#inactive", "type": "boolean" }
  ],
  "concept": [
    {
      "code": "SF-1000",
      "display": "Synthetic finding",
      "designation": [ { "language": "nl", "value": "Synthetische bevinding" } ],
      "concept": [
        { "code": "SF-1100", "display": "Finding of the skin" },
        { "code": "SF-1200", "display": "Finding of the chest" }
      ]
    }
  ]
}"""


def cs_overview():
    said = facts([("Computer name", "SyntheticClinicalFindings"), ("Experimental", "Yes"), ("Publisher", "Synthetic data team"), ("Last changed", when("2026-10-21T08:41")),
                  ("Description", "Synthetic findings for testing value sets and maps. No clinical meaning.")])
    shape = facts([("Content", "All concepts, 14 of them"), ("Hierarchy meaning", "Is-a"), ("Case sensitive", "Yes"),
                   ("Version needed", "No"), ("Supplements", '<span class="fui-muted">none</span>')])
    props = table("Declared properties", th("Property") + th("Type") + th("Meaning"),
                  [f'<tr><th scope="row" data-role="title"><code>parent</code></th>{td("Type", "code", role="sub")}{td("Meaning", "The concept one level up", role="meta-q")}</tr>',
                   f'<tr><th scope="row" data-role="title"><code>inactive</code></th>{td("Type", "boolean", role="sub")}{td("Meaning", "The concept is retired and keeps its meaning", role="meta-q")}</tr>'])
    return sections(section("Details", said), section("Structure", shape),
                    section("Declared properties", f'{props}', count=2))


VER_FILTERS = [("concept", "is-a, descendent-of, is-not-a, =, in"), ("parent", "="), ("inactive", "=")]


def cs_versions(rows=(("3", "a request naming version 3, and an unversioned request"),), langs="English (en), Dutch (nl)", props="parent, child, inactive", subs=True, filters=VER_FILTERS, release=None):
    out = []
    for v, answers in rows:
        f = table(f"Filters version {v} accepts", th("Property") + th("Operators"),
                  [f'<tr><th scope="row" data-role="title"><code>{p}</code></th>{td("Operators", f"<code>{e(o)}</code>", role="sub")}</tr>' for p, o in filters])
        body = facts([("Answers", answers), ("Subsumption", "Yes, so Check offers Relation" if subs else "No"),
                      ("Designation languages", langs), ("$lookup properties", f"<code>{props}</code>")] + ([("Release", release)] if release else []))
        out.append(section(f"Version {v}", body + f'{f}'))
    intro = section("", '<p class="fui-small fui-muted">What the server declares it can do with each version.</p>')
    return sections(intro, *out)


def cs_page_detail(tab="overview", panel=None, header=None, tabs_list=CS_TABS_WRITTEN, toast_html="", overlay=None, visitor=False, reader=False, dock_html=""):
    h = header or cs_header(visitor=visitor, reader=reader)
    t = tabs("Views of this code system", tabs_list, tab)
    p = panel if panel is not None else {"overview": cs_overview, "versions": cs_versions}[tab]()
    return shell("Synthetic clinical findings", "systems", h + f'<div class="fui-stack">{tabbed(t, p)}</div>', toast_html=toast_html, overlay=overlay, visitor=visitor, reader=reader, dock_html=dock_html)


def page_code_system(toast_html=""):
    return cs_page_detail(toast_html=toast_html)


def page_code_system_more():
    return cs_page_detail(header=cs_header(more_open=True))


def page_code_system_versions():
    return cs_page_detail("versions")


def page_code_system_json():
    return cs_page_detail("json", codeview("The CodeSystem, as the server sent it", CS_JSON, '<span class="fui-tag">FHIR R4</span>', "The CodeSystem as JSON"))


def page_code_system_visitor():
    return cs_page_detail(visitor=True)


def page_code_system_builtin():
    head = cs_header("ISO 3166-1 country codes", "urn:iso:std:iso:3166", line="Version 48 · built in", writable=False)
    note = section("", notice("info", "lock", "Built into the server from Unicode CLDR 48, with no write path, so it has no history and no Edit."))
    said = facts([("Computer name", "ISO3166Part1"), ("Publisher", "ISO, as published through Unicode CLDR"), ("Content", "All concepts, 249 of them"), ("Hierarchy meaning", '<span class="fui-muted">none: a flat list</span>'), ("Case sensitive", "Yes")])
    panel = sections(note, section("Details", said))
    tl = [("overview", "Overview", "code-system-builtin.html"), ("versions", "Versions", "code-system-builtin-versions.html"), ("json", "JSON", "code-system-json.html")]
    return cs_page_detail("overview", panel, head, tl)


def page_code_system_builtin_versions():
    head = cs_header("ISO 3166-1 country codes", "urn:iso:std:iso:3166", line="Version 48 · built in", writable=False)
    tl = [("overview", "Overview", "code-system-builtin.html"), ("versions", "Versions", "code-system-builtin-versions.html"), ("json", "JSON", "code-system-json.html")]
    panel = cs_versions(rows=(("48", "a request naming version 48, and an unversioned request"),), langs="English (en)", props="display", subs=False, filters=[("code", "=, in")])
    return cs_page_detail("versions", panel, head, tl)


def page_code_system_index():
    head = cs_header("SNOMED CT, Netherlands edition", "http://snomed.info/sct", line="2 releases · built index", writable=False)
    tl = [("overview", "Overview", "code-system-index-overview.html"), ("versions", "Versions", "code-system-index.html"), ("json", "JSON", "code-system-json.html")]
    panel = cs_versions(rows=(("20260930", "a request naming this release, and an unversioned request: the newest release is the default"),
                              ("20260331", "a request naming this release")),
                        langs="Dutch (nl), English (en)", props="parent, child, inactive, effectiveTime, moduleId", filters=[("concept", "is-a, descendent-of, <<, <, in"), ("constraint", "= (an ECL expression)")],
                        release="Read from the index root <code>/srv/ferroterm/snomed-nl/</code>")
    return cs_page_detail("versions", panel, head, tl)


def page_code_system_index_overview():
    head = cs_header("SNOMED CT, Netherlands edition", "http://snomed.info/sct", line="2 releases · built index", writable=False)
    tl = [("overview", "Overview", "code-system-index-overview.html"), ("versions", "Versions", "code-system-index.html"), ("json", "JSON", "code-system-json.html")]
    note = section("", notice("info", "lock", "Served from a built index, with no write path, so it has no history and no Edit. A new release arrives as a new directory under the index root."))
    said = facts([("Releases", "20260930 (the default), 20260331"), ("Content", "All concepts"), ("Hierarchy meaning", "Is-a"), ("Compositional grammar", "Yes"), ("Concepts", "Read through the operations; the console shows no SNOMED CT content in these mockups")])
    return cs_page_detail("overview", sections(note, section("Details", said)), head, tl)


# ---- History (a tab on a resource written here) ----

def history_rows(items=SF_HISTORY, restore_href="code-system-restore.html", focus=None):
    rows = []
    for i, (v, w, how, src) in enumerate(items):
        cur = i == 0
        restore = '<span class="fui-small fui-muted">current</span>' if cur else link_button("ghost", f"Restore…", restore_href, "rotate-ccw", small=True, attrs=f'aria-label="Restore version {v}"')
        rows.append(f'<tr><th scope="row" data-role="title">Version {v}</th>' + td("Changed", when(w), role="sub")
                    + td("Interaction", "Created" if how == "create" else "Updated", role="meta-q")
                    + td("Source system", '<span class="mk-dash">not stated</span>', role="wide") + td("", restore, "fui-row-menu", role="menu") + "</tr>")
    return table("Versions", th("Version") + th("Changed") + th("Interaction") + th("Source system", title="meta.source: a source system, not a person") + th('<span class="fui-sr-only">Restore</span>'), rows)


def compare_form(earlier="2", later="3", busy=False):
    opts = [("3", "Version 3, current"), ("2", "Version 2"), ("1", "Version 1")]
    run = busy_button("secondary", "Comparing") if busy else button("secondary", "Compare", "git-compare", type_="submit")
    return (f'<div class="fui-transform-options">{field("cmp-a", "Earlier version", select("cmp-a", opts, earlier))}'
            f'{field("cmp-b", "Later version", select("cmp-b", opts, later))}<div class="fui-transform-run">{run}</div></div>')


def diff_table(rows):
    from parts import change_del, change_ins
    out = []
    for path, kind, before, after in rows:
        b = change_del(e(before)) if before else '<span class="mk-dash">absent</span>'
        a = change_ins(e(after)) if after else '<span class="mk-dash">absent</span>'
        out.append(f'<tr><th scope="row" data-role="title"><code>{e(path)}</code></th>{td("Change", kind, role="sub")}{td("Earlier", b, role="meta-q")}{td("Later", a, role="meta-2")}</tr>')
    return table("What differs", th("Element") + th("Change") + th("Earlier") + th("Later"), out)


SF_DIFF = [("concept[SF-1000].concept[SF-1300].concept[SF-1390].property[inactive]", "Changed", "false", "true"),
           ("concept[SF-1000].concept[SF-1400]", "Added", None, "SF-1400 Finding of the abdomen"),
           ("count", "Changed", "13", "14")]


def history_panel(compared=False, kind="code system", restore_href="code-system-restore.html", items=SF_HISTORY, diff=SF_DIFF, done=None):
    how = '<p class="fui-small fui-muted">The latest versions, read back one at a time.</p>'
    top = done or ""
    lst = section("Versions", how + f'{history_rows(items, restore_href)}', count=len(items))
    if compared:
        res = (f'<p class="fui-small" role="status">{len(diff)} elements differ between version 2 and version 3, leaving out the version id and time the server sets on every write.</p>'
               f'{diff_table(diff)}')
    else:
        res = '<p class="fui-small fui-muted">Choose two versions to see the elements that differ.</p>'
    cmp_ = section("Compare two versions", compare_form() + res)
    return (f'<div class="fui-stack">{top}</div>' if top else "") + sections(lst, cmp_)


def page_code_system_history():
    return cs_page_detail("history", history_panel())


def page_code_system_compare():
    return cs_page_detail("history", history_panel(compared=True))


def restore_dialog(kind="code system", v="2", now="3"):
    return (
        '<div role="dialog" aria-modal="true" aria-labelledby="restore-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="restore-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Restore version {v}?</h2>'
        f'<p>The server writes version {v}\'s whole resource back as a new version 4. Version {now} stays in the history. '
        f'If someone changed the {kind} since you opened it, the server refuses with 412 and nothing changes.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", f"Restore version {v}", "rotate-ccw")}</div></div>'
    )


def page_code_system_restore():
    return cs_page_detail("history", history_panel(), overlay=restore_dialog())


def page_code_system_restored():
    done = load_report("ok", f'Restored version 2 {when("2026-10-21T09:44")}. The server now holds version 4.')
    items = [("4", "2026-10-21T09:44", "update", None)] + list(SF_HISTORY)
    return cs_page_detail("history", history_panel(items=items, done=done))


def page_code_system_restore_conflict():
    done = refusal("The restore was refused: someone changed this code system since you opened it.", request="PUT /r4/CodeSystem/synthetic-findings · If-Match: W/\"3\"",
                   issues=[("bad", "error", "conflict", 'The server holds version 4, written 09:43 by another client. Reload the versions to see it, then restore again if you still want version 2.')])
    done += f'<div class="fui-cluster">{button("primary", "Reload the versions", "refresh-cw")}</div>'
    return cs_page_detail("history", history_panel(done=done))


# ---- Value sets (T1) ----

VS_SRC = [("Written here", None, 8, True), ("Resource directory", None, 5, True)]


def vs_rows(items=None):
    rows = []
    for key, title, url, ver, src, status, href, sel in (items if items is not None else VALUE_SETS):
        rows.append(f'<tr>{name_cell(title, url, href)}' + td("Source", source_tag(src), role="meta-q") + td("Version", e(ver), role="wide")
                    + td("Status", status_mark(status), role="meta-2") + td("Selects", e(sel), role="wide") + "</tr>")
    return table("Value sets", th("Value set", "n", "ascending") + th("Source") + th("Version") + th("Status") + th("Selects"), rows, scroll_label="Value sets")


def vs_list(body=None, value="", visitor=False, count="13", chips=()):
    act = "" if visitor else action_group(link_button("primary", "New value set", "value-set-new.html", "file-plus"), [])
    h = page_heading("Value sets", actions=act, subtitle="The ValueSet resources this root publishes.")
    tb = tpl.toolbar("Title or canonical", value, [tpl.filter_button("Source", None)])
    c = tpl.card(body if body is not None else vs_rows(), label="Value sets", title="Value sets", count=count)
    return shell("Value sets", "valuesets", h + tpl.list_body("", tb, tpl.active(chips), c), visitor=visitor)


def page_value_sets():
    return vs_list()


def page_value_sets_empty():
    body = empty_state("layers", "This root publishes no value set",
                       "No ValueSet is loaded or written on FHIR R4 yet; Expand still takes an implicit one.",
                       link_button("primary", "New value set", "value-set-new.html", "file-plus") + link_button("secondary", "Expand a value set", "expand.html", "list-tree"))
    act = action_group(link_button("primary", "New value set", "value-set-new.html", "file-plus"), [])
    h = page_heading("Value sets", actions=act, subtitle="The ValueSet resources this root publishes.")
    return shell("Value sets", "valuesets", h + f'<div class="fui-stack">{body}</div>')


# ---- one value set (T2) ----

RESP = VALUE_SETS[0]
VS_TABS = [("definition", "Definition", "value-set.html"), ("expansion", "Expansion", "value-set-expansion.html"),
           ("history", "History", "value-set-history.html"), ("json", "JSON", "value-set-json.html")]


def vs_more(writable=True, open_=False):
    items = []
    if writable:
        items.append(menu_item("Expand in Operations", "list-tree", href="expand-answered.html"))
    items += [menu_item("Check a code in it", "shield-check", href="check.html"), MENU_SEP, menu_item("Copy canonical", "copy"), menu_item("Download JSON", "download")]
    return items


def vs_header(title=RESP[1], url=RESP[2], status="active", line="Version 2 · written here", writable=True, more_open=False, visitor=False, reader=False):
    if writable and not (visitor or reader):
        prim = link_button("primary", "Edit", "value-set-edit.html", "pencil")
    else:
        prim = link_button("primary", "Expand", "expand-answered.html", "list-tree")
    ag = action_group(prim, vs_more(writable and not (visitor or reader)), open_=more_open, label=f"More actions for {title}")
    return page_heading(title, actions=ag, subtitle=f"{canon(url)} · {line}", crumbs=[("Value sets", "value-sets.html")], badge=status_mark(status))


def resp_clauses():
    inc = clause("include", 'From <a href="code-system.html">Synthetic clinical findings</a>, version 3',
                 "Includes <code>SF-1200</code> Finding of the chest and everything below it.")
    exc = clause("exclude", 'From <a href="code-system.html">Synthetic clinical findings</a>, version 3',
                 "Excludes 1 code: Wheeze.")
    return inc + exc


def vs_definition(top=""):
    declared = facts([("Computer name", "SyntheticRespiratoryFindings"), ("Experimental", "Yes"), ("Publisher", "Synthetic data team"), ("Last changed", when("2026-10-21T09:29")),
                      ("Purpose", "Test data for the respiratory screens. No clinical meaning."), ("Immutable", "No"), ("Inactive codes", '<span class="fui-muted">left to the expansion parameters</span>')])
    sel = section("What the definition selects", f'<div class="fui-clauses">{resp_clauses()}</div>', count="2 clauses")
    return (top) + sections(sel, section("Details", declared))


EXP_ROWS = [("SF-1200", "Finding of the chest"), ("SF-1210", "Cough"), ("SF-1211", "Dry cough"), ("SF-1212", "Productive cough")]


def exp_table(rows=EXP_ROWS, system="Synthetic clinical findings", ver="3"):
    out = [f'<tr><th scope="row" data-role="title">{coding(d, c, inline=True)}</th>{td("System", e(system), role="sub")}{td("Version", ver, role="meta-q")}</tr>' for c, d in rows]
    return table("The codes the value set expands to", th("Concept") + th("System") + th("Version"), out)


def vs_expansion():
    head = load_report("ok", f'Expanded {when("2026-10-21T09:42")}: <strong>4 concepts</strong>, the whole selection on one page', actions=link_button("secondary", "Open in Operations", "expand-answered.html", "list-tree", small=True))
    return sections(section("", head), section("The codes the value set expands to", f'{exp_table()}', count=4))


VS_JSON = """{
  "resourceType": "ValueSet",
  "id": "synthetic-respiratory",
  "meta": { "versionId": "2", "lastUpdated": "2026-10-21T07:29:02Z" },
  "url": "https://terminology.example.org/fhir/ValueSet/synthetic-respiratory",
  "version": "2",
  "name": "SyntheticRespiratoryFindings",
  "title": "Synthetic respiratory findings",
  "status": "active",
  "experimental": true,
  "publisher": "Synthetic data team",
  "purpose": "Test data for the respiratory screens. No clinical meaning.",
  "compose": {
    "include": [
      {
        "system": "https://terminology.example.org/fhir/CodeSystem/synthetic-findings",
        "version": "3",
        "filter": [ { "property": "concept", "op": "is-a", "value": "SF-1200" } ]
      }
    ],
    "exclude": [
      {
        "system": "https://terminology.example.org/fhir/CodeSystem/synthetic-findings",
        "version": "3",
        "concept": [ { "code": "SF-1220", "display": "Wheeze" } ]
      }
    ]
  }
}"""

VS_HISTORY = [("2", "2026-10-21T09:29", "update", None), ("1", "2026-10-02T13:10", "create", None)]
VS_DIFF = [("compose.exclude", "Added", None, "SF-1220 Wheeze"), ("purpose", "Changed", "Test data.", "Test data for the respiratory screens. No clinical meaning.")]


def vs_page(tab="definition", panel=None, header=None, tabs_list=VS_TABS, toast_html="", overlay=None, visitor=False, reader=False, title=RESP[1]):
    h = header or vs_header(visitor=visitor, reader=reader)
    p = panel if panel is not None else {"definition": vs_definition, "expansion": vs_expansion}[tab]()
    return shell(title, "valuesets", h + f'<div class="fui-stack">{tabbed(tabs("Views of this value set", tabs_list, tab), p)}</div>', toast_html=toast_html, overlay=overlay, visitor=visitor, reader=reader)


def page_value_set(toast_html=""):
    return vs_page(toast_html=toast_html)


def page_value_set_more():
    return vs_page(header=vs_header(more_open=True))


def page_value_set_expansion():
    return vs_page("expansion")


def page_value_set_history():
    return vs_page("history", history_panel(kind="value set", restore_href="value-set-restore.html", items=VS_HISTORY))


def page_value_set_compare():
    return vs_page("history", history_panel(compared=True, kind="value set", restore_href="value-set-restore.html", items=VS_HISTORY, diff=VS_DIFF))


def page_value_set_restore():
    html = vs_page("history", history_panel(kind="value set", restore_href="value-set-restore.html", items=VS_HISTORY), overlay=restore_dialog("value set", "1", "2"))
    return html.replace("as a new version 4", "as a new version 3")


def page_value_set_json():
    return vs_page("json", codeview("The ValueSet, as the server sent it", VS_JSON, '<span class="fui-tag">FHIR R4</span>', "The ValueSet as JSON"))


def page_value_set_reader():
    return vs_page(reader=True)


def page_value_set_loaded():
    t = "AdministrativeGender"
    h = vs_header(t, "http://hl7.org/fhir/ValueSet/administrative-gender", line="Version 4.0.1 · resource directory", writable=False)
    note = section("", notice("info", "lock", "Read from a resource directory at start, with no write path, so it has no history and no Edit."))
    inc = clause("include", 'From <a href="code-system-builtin.html">AdministrativeGender</a>', "Includes every code of AdministrativeGender.")
    sel = section("What the definition selects", f'<div class="fui-clauses">{inc}</div>', count="1 clause")
    decl = section("Details", facts([("Computer name", "AdministrativeGender"), ("Publisher", "HL7 (FHIR Project)"), ("Immutable", "Yes")]))
    tl = [("definition", "Definition", "value-set-loaded.html"), ("expansion", "Expansion", "value-set-expansion.html"), ("json", "JSON", "value-set-json.html")]
    return vs_page("definition", sections(note, sel, decl), h, tl, title=t)


def page_value_set_finding():
    t = "Referral reasons"
    h = vs_header(t, VALUE_SETS[7][2], line="Version 4 · written here")
    mirror = notice("warn", "triangle-alert",
                    '<strong>The last release made 1 code of this value set inactive.</strong> <code>NF-204</code> Shortness of breath at rest, in Synthetic national findings release 2026-10, '
                    f'activated by the synchronisation service {when("2026-10-21T03:00")}. A read-only copy of its finding; the run record lives with the synchronisation service.')
    inc = clause("include", 'From <a href="code-system-builtin.html">Synthetic national findings</a>', "Includes 6 codes.",
                 '<ul class="fui-clause-codes">' + "".join(f"<li>{coding(d, c, inline=True, inactive=(c == 'NF-204'))}</li>" for c, d in [
                     ("NF-201", "Cough lasting over three weeks"), ("NF-202", "Blood in sputum"), ("NF-203", "Chest pain on exertion"), ("NF-204", "Shortness of breath at rest"),
                     ("NF-205", "Unexplained weight loss"), ("NF-206", "Night sweats")]) + "</ul>")
    sel = section("What the definition selects", f'<div class="fui-clauses">{inc}</div>', count="1 clause")
    decl = section("Details", facts([("Computer name", "ReferralReasons"), ("Publisher", "Synthetic data team"), ("Last changed", when("2026-10-14T10:02"))]))
    return vs_page("definition", f'<div class="fui-stack">{mirror}{sections(sel, decl)}</div>', h, title=t)


def page_value_set_expansion_refused():
    t = "Mime Types"
    h = vs_header(t, "http://hl7.org/fhir/ValueSet/mimetypes", line="Version 4.0.1 · resource directory", writable=False)
    rf = refusal("The server did not expand this value set: it draws in a grammar, not a list.", request="GET /r4/ValueSet/mimetypes/$expand",
                 issues=[("bad", "error", "too-costly", "The value set includes urn:ietf:bcp:13, whose codes are defined by a grammar and cannot be listed. Check a code against it instead.")])
    rf += f'<div class="fui-cluster">{link_button("secondary", "Check a code in it", "check.html", "shield-check")}</div>'
    tl = [("definition", "Definition", "value-set-loaded.html"), ("expansion", "Expansion", "value-set-expansion-refused.html"), ("json", "JSON", "value-set-json.html")]
    return vs_page("expansion", sections(section("", rf)), h, tl, title=t)


# ---- Concept maps (T1) ----

def cm_rows():
    rows = []
    for key, title, url, ver, src, status, frm, to, n, href in CONCEPT_MAPS:
        rows.append(f'<tr>{name_cell(title, url, href)}' + td("Source", source_tag(src), role="meta-q") + td("From", e(frm), role="wide") + td("To", e(to), role="wide")
                    + td("Mappings", str(n), "fui-num", role="wide") + td("Status", status_mark(status), role="meta-2") + "</tr>")
    return table("Concept maps", th("Concept map", "n", "ascending") + th("Source") + th("From") + th("To") + th("Mappings", num=True) + th("Status"), rows, scroll_label="Concept maps")


def page_concept_maps():
    act = action_group(link_button("primary", "New concept map", "concept-map-new.html", "file-plus"), [])
    h = page_heading("Concept maps", actions=act, subtitle="The ConceptMap resources this root publishes.")
    return shell("Concept maps", "conceptmaps", h + tpl.list_body("", "", "", tpl.card(cm_rows(), label="Concept maps", title="Concept maps", count=4)))


# ---- one concept map (T2) ----

LEG = CONCEPT_MAPS[0]
CM_TABS = [("mappings", "Mappings", "concept-map.html"), ("overview", "Overview", "concept-map-overview.html"),
           ("history", "History", "concept-map-history.html"), ("json", "JSON", "concept-map-json.html")]


def cm_header(title=LEG[1], url=LEG[2], status="active", line="Version 2 · written here", writable=True, more_open=False):
    prim = link_button("primary", "Edit", "concept-map-edit.html", "pencil") if writable else link_button("primary", "Translate a code", "translate.html", "corner-down-right")
    items = ([menu_item("Translate a code through it", "corner-down-right", href="translate.html")] if writable else []) + [MENU_SEP, menu_item("Copy canonical", "copy"), menu_item("Download JSON", "download")]
    if items[0] == MENU_SEP:
        items = items[1:]
    ag = action_group(prim, items, open_=more_open, label=f"More actions for {title}")
    return page_heading(title, actions=ag, subtitle=f"{canon(url)} · {line}", crumbs=[("Concept maps", "concept-maps.html")], badge=status_mark(status))


def mapping_table(rows=LEGACY_MAPPINGS, src_name="Legacy findings", src_ver="1", tgt_name="Synthetic clinical findings", tgt_ver="3", src_href="code-system-builtin.html", tgt_href="code-system.html"):
    group = (f'<tr class="fui-group-row"><th scope="rowgroup" colspan="4"><span class="mk-group-name">From <a href="{src_href}">{e(src_name)}</a>, version {src_ver}, '
             f'to <a href="{tgt_href}">{e(tgt_name)}</a>, version {tgt_ver}</span></th></tr>')
    out = [group]
    for s, sd, rel, t, tdp, cm in rows:
        r = relation(REL_WORD.get(rel, rel), none=rel is None)
        tgt = coding(tdp, t, inline=False) if t else '<span class="mk-dash" aria-label="no target">–</span>'
        out.append(f'<tr><th scope="row" data-role="title">{coding(sd, s)}</th>{td("Relationship", r, "fui-relation-cell", role="sub")}{td("Target", tgt, role="meta-q")}'
                   f'{td("Comment", e(cm) if cm else "", role="wide")}</tr>')
    return table("Every mapping the map writes", th("Source") + th("Relationship") + th("Target") + th("Comment"), out)


def cm_page(tab="mappings", panel=None, header=None, tabs_list=CM_TABS, title=LEG[1], overlay=None):
    h = header or cm_header()
    p = panel if panel is not None else sections(section("Every mapping the map writes", f'{mapping_table()}', count=7))
    return shell(title, "conceptmaps", h + f'<div class="fui-stack">{tabbed(tabs("Views of this concept map", tabs_list, tab), p)}</div>', overlay=overlay)


def page_concept_map():
    return cm_page()


def page_concept_map_overview():
    said = facts([("Computer name", "LegacyFindingsToSyntheticFindings"), ("Publisher", "Synthetic data team"), ("Last changed", when("2026-10-14T10:18")), ("Purpose", "Carry old test records onto the current synthetic codes.")])
    scope = facts([("Translates from", f'<a href="code-system-builtin.html">Legacy findings</a> <span class="fui-muted">{canon(LEGACY)}</span>'),
                   ("Translates to", f'<a href="code-system.html">Synthetic clinical findings</a> <span class="fui-muted">{canon(SF)}</span>'),
                   ("Groups", "1"), ("Codes mapped", "7, one of them to nothing")])
    return cm_page("overview", sections(section("Details", said), section("What it maps", scope)))


CM_HISTORY = [("2", "2026-10-14T10:18", "update", None), ("1", "2026-10-01T09:00", "create", None)]
CM_DIFF = [("group[0].element[LF-07].target", "Removed", "SF-1320 Fatigue", None), ("group[0].element[LF-07].noMap", "Added", None, "true")]


def page_concept_map_history():
    return cm_page("history", history_panel(kind="concept map", restore_href="concept-map-history.html", items=CM_HISTORY, diff=CM_DIFF))


CM_JSON = """{
  "resourceType": "ConceptMap",
  "id": "legacy-findings",
  "meta": { "versionId": "2", "lastUpdated": "2026-10-14T08:18:00Z" },
  "url": "https://terminology.example.org/fhir/ConceptMap/legacy-findings",
  "version": "2",
  "name": "LegacyFindingsToSyntheticFindings",
  "status": "active",
  "group": [
    {
      "source": "https://terminology.example.org/fhir/CodeSystem/legacy-findings",
      "target": "https://terminology.example.org/fhir/CodeSystem/synthetic-findings",
      "element": [
        { "code": "LF-01", "display": "Cough, any", "target": [ { "code": "SF-1210", "equivalence": "equivalent" } ] },
        { "code": "LF-07", "display": "Feeling unwell", "noMap": true }
      ]
    }
  ]
}"""


def page_concept_map_json():
    return cm_page("json", codeview("The ConceptMap, as the server sent it", CM_JSON, '<span class="fui-tag">FHIR R4</span>', "The ConceptMap as JSON"))


def page_concept_map_loaded():
    t = CONCEPT_MAPS[2][1]
    h = cm_header(t, CONCEPT_MAPS[2][2], line="Version 4.0.1 · resource directory", writable=False)
    rows = [("male", "Male", "equivalent", "M", "Male", ""), ("female", "Female", "equivalent", "F", "Female", ""),
            ("other", "Other", "source-is-narrower-than-target", "A", "Ambiguous", ""), ("unknown", "Unknown", "equivalent", "U", "Unknown", "")]
    note = section("", notice("info", "lock", "Read from a resource directory at start, with no write path, so it has no history and no Edit."))
    tbl = section("Every mapping the map writes", f'{mapping_table(rows, "AdministrativeGender", "4.0.1", "v2 Administrative Sex", "2.9", "code-system-builtin.html", "code-system-builtin.html")}', count=4)
    tl = [("mappings", "Mappings", "concept-map-loaded.html"), ("overview", "Overview", "concept-map-overview.html"), ("json", "JSON", "concept-map-json.html")]
    return cm_page("mappings", sections(note, tbl), h, tl, title=t)
