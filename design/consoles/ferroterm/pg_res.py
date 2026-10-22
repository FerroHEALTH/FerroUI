# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The terminology resources: Code systems, Value sets and Concept maps (T1)
and each resource's own page (T2, the one home of the resource, its
versions and its history). History is a tab only on a resource written
here; a loaded one says once, on its first tab, why it has none."""
import json

import tpl
from common import (
    MENU_SEP, action_group, badge, button, busy_button, e, empty_quiet, empty_state, field, icon, link_button, menu, menu_item,
    notice, page_heading, refusal, select, shell, table, tabbed, tabs, td, th, when,
)
from data import (R4_GENDER_V2_MAP, R4_MIMETYPES, CODE_SYSTEMS, CONCEPT_MAPS, FINDINGS, LEGACY, LEGACY_MAPPINGS, REL_WORD, SF, SF_HISTORY, VALUE_SETS)
from parts import skeleton_lines
from term import (canon, clause, codeview, coding, facts, inactive_mark, load_report, plural, quiet, relation, section, sections, source_tag, state, status_mark)

CONTENT_WORD = {"complete": "All concepts", "not-present": "Not listed"}
CONTENT_TIP = {"complete": "content: complete, every concept is in the resource", "not-present": "content: not-present, the server answers codes by their grammar"}


def content_word(c):
    return f'<span title="{e(CONTENT_TIP[c])}">{CONTENT_WORD[c]}</span>'


def name_cell(name, url, href):
    return (f'<th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{e(name)}</a>'
            f'<span class="fui-row-sub">{canon(url)}</span></th>')


# ---- Code systems (T1) ----

SRC_ITEMS = [("Stored on this server", None, 3, True), ("Resource directory", None, 5, True), ("Offline index", None, 3, True), ("Built-in standard", None, 4, True)]


def versions_word(vers):
    """A code system's versions in words, so a phone card that drops the
    column header still says what the number is."""
    if not vers:
        return '<span class="mk-dash">none declared</span>'
    if len(vers) == 1:
        v = vers[0]
        return e(f"Version {v}" if v[:1].isdigit() else v)
    return e("Versions " + ", ".join(vers))


def cs_rows(items=None, hover=None):
    rows = []
    for i, (key, name, url, vers, src, content, hier, subs, status, n, href) in enumerate(items if items is not None else CODE_SYSTEMS):
        cls = ' class="mk-hover"' if i == hover else ""
        v = versions_word(vers)
        rows.append(f'<tr{cls}>{name_cell(name, url, href)}'
                    + td("Source", source_tag(src), role="meta-q") + td("Versions", v, role="meta-2")
                    + td("Content", content_word(content), role="wide") + td("Hierarchy", "Tree" if hier else "Flat", role="wide") + "</tr>")
    return table("Code systems", th("Code system", "n", "ascending") + th("Source") + th("Versions") + th("Content") + th("Hierarchy", title="Whether the served version declares an operator to walk its hierarchy by"), rows, scroll_label="Code systems")


def cs_heading(visitor=False):
    act = "" if visitor else action_group(link_button("primary", "New code system", "code-system-new.html", "file-plus"), [])
    return page_heading("Code systems", actions=act, subtitle="Every code system this server offers on FHIR R4.")


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
    return cs_page(cs_rows(written), source="Stored on this server", open_=True, chips=["Source: Stored on this server"], count="3 of 15")


def page_code_systems_no_match():
    body = quiet('15 code systems are served on FHIR R4; none match ICD-11. <a href="code-systems.html">Clear the filter</a>')
    return cs_page(body, value="ICD-11", chips=["Name or canonical: ICD-11"], count="0 of 15")


def page_code_systems_loading():
    cells = "".join(f'<tr aria-hidden="true"><td>{skeleton_lines([60, 90])}</td><td>{skeleton_lines([70])}</td><td>{skeleton_lines([40])}</td><td>{skeleton_lines([60])}</td><td>{skeleton_lines([40])}</td></tr>' for _ in range(6))
    head = th("Code system") + th("Source") + th("Versions") + th("Content") + th("Hierarchy")
    sk = (f'<div class="fui-table-scroll" role="region" aria-label="Code systems" aria-busy="true" tabindex="0"><table class="fui-table"><caption>Code systems</caption>'
          f'<thead><tr>{head}</tr></thead><tbody>{cells}</tbody></table></div><p class="fui-sr-only" role="status">Reading the code systems</p>')
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
              menu_item("Copy canonical", "copy")]
    return items


def cs_header(name="Synthetic clinical findings", url=SF, status="active", line="Version 3 · stored on this server", writable=True, more_open=False, visitor=False, reader=False, quiet_edit=False):
    if writable and not (visitor or reader):
        prim = link_button("secondary" if quiet_edit else "primary", "Edit", "code-system-edit.html", "pencil")
    else:
        prim = link_button("primary", "Browse concepts", "browse.html", "network")
    ag = action_group(prim, cs_more(writable and not (visitor or reader), more_open), open_=more_open, label=f"More actions for {name}")
    return page_heading(name, actions=ag, subtitle=f"{canon(url)} · {line}", crumbs=[("Code systems", "code-systems.html")], badge=status_mark(status))


def package_json(doc):
    """A resource of the HL7 FHIR R4 core package, as data.py holds it."""
    return json.dumps(doc, indent=2, ensure_ascii=False)


def cs_json():
    """Synthetic clinical findings as the server holds version 3: all 14
    concepts, nested by parent, each with its Dutch designation."""
    kids = {}
    for c, d, p, i, nl in FINDINGS:
        kids.setdefault(p, []).append((c, d, i, nl))

    def node(c, d, i, nl):
        n = {"code": c, "display": d, "designation": [{"language": "nl", "value": nl}]}
        if i:
            n["property"] = [{"code": "inactive", "valueBoolean": True}]
        ch = [node(*k) for k in kids.get(c, [])]
        if ch:
            n["concept"] = ch
        return n
    doc = {
        "resourceType": "CodeSystem", "id": "synthetic-findings",
        "meta": {"versionId": "3", "lastUpdated": "2026-10-21T06:41:40Z"},
        "url": SF, "version": "3", "name": "SyntheticClinicalFindings", "title": "Synthetic clinical findings",
        "status": "active", "experimental": True, "publisher": "Synthetic data team",
        "description": "Synthetic findings for testing value sets and maps. No clinical meaning.",
        "caseSensitive": True, "hierarchyMeaning": "is-a", "content": "complete", "count": len(FINDINGS),
        "property": [{"code": "parent", "type": "code"},
                     {"code": "inactive", "uri": "http://hl7.org/fhir/concept-properties#inactive", "type": "boolean"}],
        "concept": [node(*k) for k in kids[None]],
    }
    return json.dumps(doc, indent=2, ensure_ascii=False)


def summary_json(doc):
    """A large code system read with _summary=true: the server leaves the
    concepts out and tags the answer as subsetted."""
    head = {"resourceType": "CodeSystem", "id": doc.pop("id"),
            "meta": {"tag": [{"system": "http://terminology.hl7.org/CodeSystem/v3-ObservationValue", "code": "SUBSETTED"}]}}
    head.update(doc)
    return json.dumps(head, indent=2, ensure_ascii=False)


def cs_overview():
    said = facts([("Computer name", "SyntheticClinicalFindings"), ("Experimental", "Yes"), ("Publisher", "Synthetic data team"), ("Last changed", when("2026-10-21T08:41")),
                  ("Description", "Synthetic findings for testing value sets and maps. No clinical meaning.")])
    shape = facts([("Content", "All concepts, 14 of them"), ("Hierarchy meaning", "Is-a"), ("Case sensitive", "Yes"),
                   ("Version needed", "No"), ("Supplements", '<span class="fui-muted">None</span>')])
    props = table("Declared properties", th("Property") + th("Type") + th("Meaning"),
                  [f'<tr><th scope="row" data-role="title"><code>parent</code></th>{td("Type", "code", role="sub")}{td("Meaning", "The concept one level up", role="meta-q")}</tr>',
                   f'<tr><th scope="row" data-role="title"><code>inactive</code></th>{td("Type", "boolean", role="sub")}{td("Meaning", "The concept is retired and keeps its meaning", role="meta-q")}</tr>'])
    return sections(section("Details", said), section("Structure", shape),
                    section("Declared properties", f'{props}', count=2))


def ops_cell(ops):
    """Filter operators, each a code that never breaks inside itself."""
    return ", ".join(f'<code class="fui-nowrap">{e(o)}</code>' for o in ops)


def ver_title(v, label="Version"):
    return f"{label} {v}" if v[:1].isdigit() else v


VER_FILTERS = [("concept", ["is-a", "descendent-of", "is-not-a", "=", "in"]), ("parent", ["="]), ("inactive", ["="])]


def cs_versions(rows=(("3", "For version 3, and for requests that name no version."),), langs="English (en), Dutch (nl)", props=("parent", "child", "inactive"),
                subs=True, filters=VER_FILTERS, release=None, label="Version"):
    out = []
    for v, answers in rows:
        title = ver_title(v, label)
        low = title[:1].lower() + title[1:] if title[:1].isupper() and title.split()[0] in ("Version", "Release") else title
        f = table(f"Filters {low} accepts", th("Property") + th("Operators"),
                  [f'<tr><th scope="row" data-role="title"><code>{p}</code></th>{td("Operators", ops_cell(o))}</tr>' for p, o in filters])
        body = facts([("Answers requests", answers), ("Subsumption", "Yes, so Check offers Relation" if subs else "No, so Check offers no Relation"),
                      ("Designation languages", langs), ("Properties Look up returns", ", ".join(f"<code>{p}</code>" for p in props))]
                     + ([("Release", release)] if release else []))
        out.append(section(title, body))
        out.append(section(f"Filters {low} accepts", f))
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
    return cs_page_detail("json", codeview("CodeSystem/synthetic-findings", cs_json(), '<span class="fui-tag">FHIR R4</span>', "The CodeSystem as JSON"))


def page_code_system_visitor():
    return cs_page_detail(visitor=True)


ISO_TABS = [("overview", "Overview", "code-system-builtin.html"), ("versions", "Versions", "code-system-builtin-versions.html"), ("json", "JSON", "code-system-builtin-json.html")]
SCT_TABS = [("overview", "Overview", "code-system-index-overview.html"), ("versions", "Versions", "code-system-index.html"), ("json", "JSON", "code-system-index-json.html")]


def iso_header():
    return cs_header("ISO 3166-1 country codes", "urn:iso:std:iso:3166", line="CLDR 48 · built-in standard", writable=False)


def sct_header():
    return cs_header("SNOMED CT, Netherlands edition", "http://snomed.info/sct", line="2 releases · offline index", writable=False)


def page_code_system_builtin():
    note = section("", notice("neutral", "lock", "Built into the server from Unicode CLDR 48, so the console cannot edit it and it keeps no history."))
    said = facts([("Computer name", "ISO3166Part1"), ("Publisher", "ISO, as published through Unicode CLDR"), ("Content", "All concepts, 249 of them"), ("Hierarchy meaning", '<span class="fui-muted">None: a flat list</span>'), ("Case sensitive", "Yes")])
    return cs_page_detail("overview", sections(note, section("Details", said)), iso_header(), ISO_TABS)


def page_code_system_builtin_versions():
    panel = cs_versions(rows=(("CLDR 48", "For CLDR 48, and for requests that name no version."),), langs="English (en)", props=("display",), subs=False, filters=[("code", ["=", "in"])])
    return cs_page_detail("versions", panel, iso_header(), ISO_TABS)


def page_code_system_builtin_json():
    doc = summary_json({"id": "iso3166", "url": "urn:iso:std:iso:3166", "version": "48", "name": "ISO3166Part1", "title": "ISO 3166-1 country codes",
                        "status": "active", "publisher": "ISO, as published through Unicode CLDR", "caseSensitive": True, "content": "complete", "count": 249})
    cv = codeview("CodeSystem/iso3166, summary", doc, '<span class="fui-tag">FHIR R4</span>', "The CodeSystem as JSON, without its concepts")
    note = '<p class="fui-small fui-muted">Read with <code>_summary=true</code>: the 249 concepts are left out. Look up reads one.</p>'
    return cs_page_detail("json", sections(section("", note), section("", cv)), iso_header(), ISO_TABS)


def page_code_system_index():
    panel = cs_versions(rows=(("20260930", "For this release, and for requests that name no release: the newest release is the default."),
                              ("20260331", "For this release only.")),
                        langs="Dutch (nl), English (en)", props=("parent", "child", "inactive", "effectiveTime", "moduleId"),
                        filters=[("concept", ["is-a", "descendent-of", "in"]), ("constraint", ["="])],
                        release="Read from the index root <code>/srv/ferroterm/snomed-nl/</code>; a constraint filter takes an ECL expression", label="Release")
    return cs_page_detail("versions", panel, sct_header(), SCT_TABS)


def page_code_system_index_overview():
    note = section("", notice("neutral", "lock", "Served from an index built offline, so the console cannot edit it and it keeps no history. A new release arrives as a new directory under the index root."))
    said = facts([("Releases", "20260930 (the default), 20260331"), ("Content", "All concepts"), ("Hierarchy meaning", "Is-a"), ("Compositional grammar", "Yes")])
    return cs_page_detail("overview", sections(note, section("Details", said)), sct_header(), SCT_TABS)


def page_code_system_index_json():
    doc = summary_json({"id": "sct-nl", "url": "http://snomed.info/sct", "version": "http://snomed.info/sct/11000146104/version/20260930", "name": "SNOMED_CT",
                        "title": "SNOMED CT, Netherlands edition", "status": "active", "publisher": "SNOMED International and the Dutch release centre",
                        "caseSensitive": False, "hierarchyMeaning": "is-a", "compositional": True, "content": "complete"})
    cv = codeview("CodeSystem/sct-nl, summary", doc, '<span class="fui-tag">FHIR R4</span>', "The CodeSystem as JSON, without its concepts")
    note = '<p class="fui-small fui-muted">Read with <code>_summary=true</code>: the concepts are left out. Look up reads one.</p>'
    return cs_page_detail("json", sections(section("", note), section("", cv)), sct_header(), SCT_TABS)


# ---- History (a tab on a resource stored on this server) ----

def diff_table(rows):
    """Each change in plain words first, its element path as a quiet hint."""
    from parts import change_del, change_ins
    out = []
    for label, path, kind, before, after in rows:
        b = change_del(e(before)) if before else '<span class="mk-dash">absent</span>'
        a = change_ins(e(after)) if after else '<span class="mk-dash">absent</span>'
        out.append(f'<tr><th scope="row" data-role="title">{e(label)}<span class="fui-row-sub"><code class="fui-canonical">{e(path)}</code></span></th>'
                   f'{td("Change", kind, role="sub")}{td("Earlier", b, role="meta-q")}{td("Later", a, role="meta-2")}</tr>')
    return table("What differs", th("What changed") + th("Change") + th("Earlier") + th("Later"), out)


SF_DIFF = [("SF-1390 Malaise, old code: inactive", "concept[SF-1390].property[inactive]", "Changed", "false", "true"),
           ("SF-1400 Finding of the abdomen", "concept[SF-1400]", "Added", None, "SF-1400 Finding of the abdomen"),
           ("Number of concepts", "count", "Changed", "13", "14")]



def history_rows(items=SF_HISTORY, restore_href="code-system-restore.html"):
    stated = any(it[3] for it in items)
    rows = []
    for i, (v, w, how, src) in enumerate(items):
        cur = i == 0
        cell = f'<span class="fui-version-cell">Revision {v}' + ('<span class="fui-badge fui-tone-accent">Current</span>' if cur else "") + "</span>"
        restore = "" if cur else link_button("ghost", "Restore…", restore_href, "rotate-ccw", small=True, attrs=f'aria-label="Restore revision {v}"')
        src_cell = td("Source system", e(src) if src else '<span class="mk-dash">not stated</span>', role="wide") if stated else ""
        rows.append(f'<tr><th scope="row" data-role="title">{cell}</th>' + td("Changed", when(w), role="sub")
                    + td("Interaction", "Created" if how == "create" else "Updated", role="meta-q")
                    + src_cell + td("", restore, "fui-row-menu", role="menu") + "</tr>")
    src_head = th("Source system", title="meta.source: a source system, not a person") if stated else ""
    return table("Revisions", th("Revision") + th("Changed") + th("Interaction") + src_head + th('<span class="fui-sr-only">Restore</span>'), rows)


def compare_form(items=SF_HISTORY, earlier="2", later="3", busy=False):
    opts = [(it[0], f"Revision {it[0]}, current" if i == 0 else f"Revision {it[0]}") for i, it in enumerate(items)]
    run = busy_button("secondary", "Comparing") if busy else button("secondary", "Compare", "git-compare", type_="submit")
    return (f'<div class="fui-transform-options">{field("cmp-a", "Earlier revision", select("cmp-a", opts, earlier))}'
            f'{field("cmp-b", "Later revision", select("cmp-b", opts, later))}<div class="fui-transform-run">{run}</div></div>')


def history_panel(compared=False, kind="code system", restore_href="code-system-restore.html", items=SF_HISTORY, diff=SF_DIFF, done=None, earlier="2", later="3"):
    how = '<p class="fui-small fui-muted">The revisions the server holds, read back one at a time.</p>'
    top = done or ""
    lst = section("Revisions", how + f'{history_rows(items, restore_href)}', count=len(items))
    if compared:
        res = (f'<p class="fui-small" role="status">{plural(len(diff), "element")} differ between revision {earlier} and revision {later}, leaving out the revision id and time the server sets on every write.</p>'
               f'{diff_table(diff)}')
    else:
        res = '<p class="fui-small fui-muted">Choose two revisions to see what differs.</p>'
    cmp_ = section("Compare two revisions", compare_form(items, earlier, later) + res)
    return (f'<div class="fui-stack">{top}</div>' if top else "") + sections(lst, cmp_)


def page_code_system_history():
    return cs_page_detail("history", history_panel())


def page_code_system_compare():
    return cs_page_detail("history", history_panel(compared=True))


def restore_dialog(kind="code system", v="2", now="3", new="4"):
    return (
        '<div role="dialog" aria-modal="true" aria-labelledby="restore-title" class="fui-dialog fui-modal mk-open" open>'
        f'<h2 id="restore-title" class="fui-dialog-title">{icon("rotate-ccw", "18")} Restore revision {v}?</h2>'
        f'<p>The server writes revision {v}\'s whole resource back as a new revision {new}. Revision {now} stays in the history. '
        f'If someone changed the {kind} since you opened it, the server refuses the restore and nothing changes.</p>'
        f'<div class="fui-dialog-actions">{button("secondary", "Cancel", keys="Esc")}{button("primary", f"Restore revision {v}", "rotate-ccw")}</div></div>'
    )


def page_code_system_restore():
    return cs_page_detail("history", history_panel(), overlay=restore_dialog())


def page_code_system_restored():
    done = load_report("ok", f'Restored revision 2 {when("2026-10-21T09:44")}. The server now holds revision 4.')
    items = [("4", "2026-10-21T09:44", "update", None)] + list(SF_HISTORY)
    head = cs_header(line="Version 2, restored as revision 4 · stored on this server")
    return cs_page_detail("history", history_panel(items=items, done=done, earlier="3", later="4"), header=head)


def page_code_system_restore_conflict():
    done = refusal("Someone changed this code system since you opened it, so the restore was refused.", request="PUT /r4/CodeSystem/synthetic-findings · If-Match: W/\"3\"",
                   issues=[("bad", "error", "conflict", "The server holds revision 4, written 09:43 by another client. Reload the revisions to see it, then restore again if you still want revision 2.")])
    done += f'<div class="fui-cluster">{button("primary", "Reload the revisions", "refresh-cw")}</div>'
    return cs_page_detail("history", history_panel(done=done), header=cs_header(quiet_edit=True))


# ---- Value sets (T1) ----

VS_SRC = [("Stored on this server", None, 8, True), ("Resource directory", None, 5, True)]


def vs_rows(items=None):
    rows = []
    for key, title, url, ver, src, status, href, sel in (items if items is not None else VALUE_SETS):
        rows.append(f'<tr>{name_cell(title, url, href)}' + td("Source", source_tag(src), role="meta-q") + td("Version", e(f"Version {ver}"), role="meta-v")
                    + td("Status", status_mark(status), role="meta-2") + td("Selects", e(sel), role="meta-v") + "</tr>")
    return table("Value sets", th("Value set", "n", "ascending") + th("Source") + th("Version") + th("Status") + th("Selects"), rows, scroll_label="Value sets")


VS_SUB = "The value sets this server holds for FHIR R4."


def vs_heading(visitor=False):
    act = "" if visitor else action_group(link_button("primary", "New value set", "value-set-new.html", "file-plus"), [])
    return page_heading("Value sets", actions=act, subtitle=VS_SUB)


def vs_list(body=None, value="", visitor=False, count="13", chips=()):
    tb = tpl.toolbar("Title or canonical", value, [tpl.filter_button("Source", None)])
    c = tpl.card(body if body is not None else vs_rows(), label="Value sets", title="Value sets", count=count)
    return shell("Value sets", "valuesets", vs_heading(visitor) + tpl.list_body("", tb, tpl.active(chips), c), visitor=visitor)


def page_value_sets():
    return vs_list()


def page_value_sets_empty():
    body = quiet('No value sets for FHIR R4 yet. <a href="value-set-new.html">Add the first one</a>')
    return shell("Value sets", "valuesets", vs_heading() + f'<div class="fui-stack">{body}</div>')


# ---- one value set (T2) ----

RESP = VALUE_SETS[0]
VS_TABS = [("definition", "Definition", "value-set.html"), ("expansion", "Expansion", "value-set-expansion.html"),
           ("history", "History", "value-set-history.html"), ("json", "JSON", "value-set-json.html")]


def vs_more(editable=True, open_=False):
    """Expand has its home on the Expansion tab and Download JSON on the JSON
    tab, so More carries neither."""
    items = [menu_item("Check a code in it", "shield-check", href="check.html"), MENU_SEP] if editable else []
    return items + [menu_item("Copy canonical", "copy")]


def vs_header(title=RESP[1], url=RESP[2], status="active", line="Version 2 · stored on this server", writable=True, more_open=False, visitor=False, reader=False):
    editable = writable and not (visitor or reader)
    if editable:
        prim = link_button("primary", "Edit", "value-set-edit.html", "pencil")
    else:
        prim = link_button("primary", "Check a code in it", "check.html", "shield-check")
    ag = action_group(prim, vs_more(editable), open_=more_open, label=f"More actions for {title}")
    return page_heading(title, actions=ag, subtitle=f"{canon(url)} · {line}", crumbs=[("Value sets", "value-sets.html")], badge=status_mark(status))


def resp_clauses():
    inc = clause("include", 'From <a href="code-system.html">Synthetic clinical findings</a>, version 3',
                 "Includes <code>SF-1200</code> Finding of the chest and everything below it.")
    exc = clause("exclude", 'From <a href="code-system.html">Synthetic clinical findings</a>, version 3',
                 f'Excludes 1 code: {coding("Wheeze", "SF-1220", inline=True)}')
    return inc + exc


def vs_definition(top="", changed="2026-10-21T09:29"):
    declared = facts([("Computer name", "SyntheticRespiratoryFindings"), ("Experimental", "Yes"), ("Publisher", "Synthetic data team"), ("Last changed", when(changed)),
                      ("Purpose", "Test data for the respiratory screens. No clinical meaning."), ("Immutable", "No"), ("Inactive codes", '<span class="fui-muted">Decided when expanding</span>')])
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
VS_DIFF = [("Exclude: SF-1220 Wheeze", "compose.exclude", "Added", None, "SF-1220 Wheeze"),
           ("Purpose", "purpose", "Changed", "Test data.", "Test data for the respiratory screens. No clinical meaning.")]


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


def vs_history(compared=False):
    return history_panel(compared=compared, kind="value set", restore_href="value-set-restore.html", items=VS_HISTORY, diff=VS_DIFF, earlier="1", later="2")


def page_value_set_history():
    return vs_page("history", vs_history())


def page_value_set_compare():
    return vs_page("history", vs_history(compared=True))


def page_value_set_restore():
    return vs_page("history", vs_history(), overlay=restore_dialog("value set", "1", "2", "3"))


def page_value_set_json():
    return vs_page("json", codeview("ValueSet/synthetic-respiratory", VS_JSON, '<span class="fui-tag">FHIR R4</span>', "The ValueSet as JSON"))


def page_value_set_reader():
    return vs_page(reader=True)


MIME = VALUE_SETS[12]
MIME_TABS = [("definition", "Definition", "value-set-loaded.html"), ("expansion", "Expansion", "value-set-expansion-refused.html"), ("json", "JSON", "value-set-loaded-json.html")]


def mime_header():
    return vs_header(MIME[1], MIME[2], line="Version 4.0.1 · resource directory", writable=False)


def page_value_set_loaded():
    note = section("", notice("neutral", "lock", "The console cannot edit it, and it keeps no history."))
    inc = clause("include", "From IETF BCP 13 media types", "Includes every media type: its codes follow a grammar, so they cannot be listed.")
    sel = section("What the definition selects", f'<div class="fui-clauses">{inc}</div>', count="1 clause")
    decl = section("Details", facts([("Computer name", "Mime Types"), ("Publisher", "HL7 International, FHIR Infrastructure"),
                                     ("Description", "Every code of BCP 13.")]))
    return vs_page("definition", sections(note, sel, decl), mime_header(), MIME_TABS, title=MIME[1])


def page_value_set_loaded_json():
    cv = codeview("ValueSet/mimetypes", package_json(R4_MIMETYPES), '<span class="fui-tag">FHIR R4</span>', "The ValueSet as JSON")
    return vs_page("json", cv, mime_header(), MIME_TABS, title=MIME[1])


def page_value_set_finding():
    t = "Referral reasons"
    h = vs_header(t, VALUE_SETS[7][2], line="Version 4 · stored on this server")
    mirror = notice("warn", "triangle-alert",
                    '<strong>The 2026-10 release of Synthetic national findings made 1 code of this value set inactive:</strong> <code>NF-204</code> Shortness of breath at rest. '
                    f'The synchronisation service activated that release {when("2026-10-21T03:00")}, and its run record has the details.')
    inc = clause("include", "From Synthetic national findings", "Includes 6 codes.",
                 '<ul class="fui-clause-codes">' + "".join(f"<li>{coding(d, c, inline=True, inactive=(c == 'NF-204'))}</li>" for c, d in [
                     ("NF-201", "Cough lasting over three weeks"), ("NF-202", "Blood in sputum"), ("NF-203", "Chest pain on exertion"), ("NF-204", "Shortness of breath at rest"),
                     ("NF-205", "Unexplained weight loss"), ("NF-206", "Night sweats")]) + "</ul>")
    sel = section("What the definition selects", f'<div class="fui-clauses">{inc}</div>', count="1 clause")
    decl = section("Details", facts([("Computer name", "ReferralReasons"), ("Publisher", "Synthetic data team"), ("Last changed", when("2026-10-14T10:02"))]))
    return vs_page("definition", f'<div class="fui-stack">{mirror}{sections(sel, decl)}</div>', h, title=t)


def page_value_set_expansion_refused():
    rf = refusal("The server did not expand this value set: its codes follow a grammar, so they cannot be listed.", request="GET /r4/ValueSet/mimetypes/$expand",
                 issues=[("bad", "Error", "too-costly", "The value set includes urn:ietf:bcp:13, whose codes are defined by a grammar and cannot be listed. Check a code against it instead.")])
    return vs_page("expansion", sections(section("", rf)), mime_header(), MIME_TABS, title=MIME[1])


# ---- Concept maps (T1) ----

def cm_rows():
    rows = []
    for key, title, url, ver, src, status, frm, to, n, href in CONCEPT_MAPS:
        name = (f'<th scope="row" data-role="title"><a class="fui-row-link" href="{href}">{e(title)}</a>'
                f'<span class="fui-row-sub">{canon(url)}</span><span class="fui-row-sub">{e(frm)} → {e(to)}</span></th>')
        rows.append(f'<tr>{name}' + td("Source", source_tag(src), role="meta-q")
                    + td("Mappings", plural(n, "mapping"), "fui-num", role="meta-v") + td("Status", status_mark(status), role="meta-2") + "</tr>")
    return table("Concept maps", th("Concept map", "n", "ascending") + th("Source") + th("Mappings", num=True) + th("Status"), rows, scroll_label="Concept maps")


def page_concept_maps():
    act = action_group(link_button("primary", "New concept map", "concept-map-new.html", "file-plus"), [])
    h = page_heading("Concept maps", actions=act, subtitle="The concept maps this server holds.")
    return shell("Concept maps", "conceptmaps", h + tpl.list_body("", "", "", tpl.card(cm_rows(), label="Concept maps", title="Concept maps", count=4)))


# ---- one concept map (T2) ----

LEG = CONCEPT_MAPS[0]
CM_TABS = [("mappings", "Mappings", "concept-map.html"), ("overview", "Overview", "concept-map-overview.html"),
           ("history", "History", "concept-map-history.html"), ("json", "JSON", "concept-map-json.html")]


def cm_header(title=LEG[1], url=LEG[2], status="active", line="Version 2 · stored on this server", writable=True, more_open=False):
    prim = link_button("primary", "Edit", "concept-map-edit.html", "pencil") if writable else link_button("primary", "Translate a code", "translate.html", "corner-down-right")
    items = ([menu_item("Translate a code through it", "corner-down-right", href="translate.html"), MENU_SEP] if writable else []) + [menu_item("Copy canonical", "copy")]
    ag = action_group(prim, items, open_=more_open, label=f"More actions for {title}")
    return page_heading(title, actions=ag, subtitle=f"{canon(url)} · {line}", crumbs=[("Concept maps", "concept-maps.html")], badge=status_mark(status))


def system_name(name, href=None, ver=None):
    """A code system in a group row: a link only where the console has its page."""
    n = f'<a href="{href}">{e(name)}</a>' if href else e(name)
    return n + (f", version {ver}" if ver else "")


def mapping_table(rows=LEGACY_MAPPINGS, src_name="Legacy findings", src_ver="1", tgt_name="Synthetic clinical findings", tgt_ver="3", src_href=None, tgt_href="code-system.html"):
    comments = any(r[5] for r in rows)
    cols = 4 if comments else 3
    group = (f'<tr class="fui-group-row"><th scope="rowgroup" colspan="{cols}"><span class="mk-group-name">From {system_name(src_name, src_href, src_ver)} '
             f'to {system_name(tgt_name, tgt_href, tgt_ver)}</span></th></tr>')
    out = [group]
    for s, sd, rel, t, tdp, cm in rows:
        r = relation(REL_WORD.get(rel, rel), none=t is None)
        tgt = coding(tdp, t, inline=False) if t else '<span class="mk-dash" aria-label="no target">–</span>'
        c = td("Comment", e(cm) if cm else "", role="wide") if comments else ""
        out.append(f'<tr><th scope="row" data-role="title">{coding(sd, s)}</th>{td("Relationship", r, "fui-relation-cell", role="sub")}{td("Target", tgt, role="meta-q")}{c}</tr>')
    return table("Every mapping the map writes", th("Source") + th("Relationship") + th("Target") + (th("Comment") if comments else ""), out)


def cm_page(tab="mappings", panel=None, header=None, tabs_list=CM_TABS, title=LEG[1], overlay=None):
    h = header or cm_header()
    p = panel if panel is not None else sections(section("Every mapping the map writes", f'{mapping_table()}', count=7))
    return shell(title, "conceptmaps", h + f'<div class="fui-stack">{tabbed(tabs("Views of this concept map", tabs_list, tab), p)}</div>', overlay=overlay)


def page_concept_map():
    return cm_page()


def page_concept_map_overview():
    said = facts([("Computer name", "LegacyFindingsToSyntheticFindings"), ("Publisher", "Synthetic data team"), ("Last changed", when("2026-10-14T10:18")), ("Purpose", "Carry old test records onto the current synthetic codes.")])
    scope = facts([("Translates from", f'Legacy findings <span class="fui-muted">{canon(LEGACY)}</span>'),
                   ("Translates to", f'<a href="code-system.html">Synthetic clinical findings</a> <span class="fui-muted">{canon(SF)}</span>'),
                   ("Groups", "1"), ("Codes mapped", "7, one of them to nothing")])
    return cm_page("overview", sections(section("Details", said), section("What it maps", scope)))


CM_HISTORY = [("2", "2026-10-14T10:18", "update", None), ("1", "2026-10-01T09:00", "create", None)]
CM_DIFF = [("LF-07 Feeling unwell: target", "group[0].element[LF-07].target[0].code", "Removed", "SF-1320 Fatigue", None),
           ("LF-07 Feeling unwell: equivalence", "group[0].element[LF-07].target[0].equivalence", "Changed", "equivalent", "unmatched")]


def cm_history(compared=False):
    return history_panel(compared=compared, kind="concept map", restore_href="concept-map-restore.html", items=CM_HISTORY, diff=CM_DIFF, earlier="1", later="2")


def page_concept_map_history():
    return cm_page("history", cm_history())


def page_concept_map_restore():
    return cm_page("history", cm_history(), overlay=restore_dialog("concept map", "1", "2", "3"))


def cm_json():
    """Legacy findings to synthetic findings, revision 2, as R4 writes it:
    every element, the one with no target as equivalence unmatched."""
    els = []
    for s, sd, rel, t, tdp, cm in LEGACY_MAPPINGS:
        tg = {"code": t, "display": tdp, "equivalence": rel} if t else {"equivalence": rel}
        if cm:
            tg["comment"] = cm
        els.append({"code": s, "display": sd, "target": [tg]})
    doc = {"resourceType": "ConceptMap", "id": "legacy-findings", "meta": {"versionId": "2", "lastUpdated": "2026-10-14T08:18:00Z"},
           "url": LEG[2], "version": "2", "name": "LegacyFindingsToSyntheticFindings", "title": LEG[1], "status": "active",
           "group": [{"source": LEGACY, "sourceVersion": "1", "target": SF, "targetVersion": "3", "element": els}]}
    return json.dumps(doc, indent=2, ensure_ascii=False)


def page_concept_map_json():
    return cm_page("json", codeview("ConceptMap/legacy-findings", cm_json(), '<span class="fui-tag">FHIR R4</span>', "The ConceptMap as JSON"))


GENDER = CONCEPT_MAPS[2]
GENDER_TABS = [("mappings", "Mappings", "concept-map-loaded.html"), ("overview", "Overview", "concept-map-loaded-overview.html"), ("json", "JSON", "concept-map-loaded-json.html")]
# The package's five targets (ConceptMap-cm-administrative-gender-v2.json), with the displays of AdministrativeGender and v2 table 0001.
GENDER_ROWS = [("male", "Male", "equal", "M", "Male", ""), ("female", "Female", "equal", "F", "Female", ""),
               ("other", "Other", "wider", "A", "Ambiguous", ""), ("other", "Other", "wider", "O", "Other", ""),
               ("unknown", "Unknown", "equal", "U", "Unknown", "")]


def gender_header():
    return cm_header(GENDER[1], GENDER[2], status=GENDER[5], line="Version 4.0.1 · resource directory", writable=False)


def gender_note():
    return section("", notice("neutral", "lock", "The console cannot edit it, and it keeps no history."))


def page_concept_map_loaded():
    tbl = section("Every mapping the map writes", mapping_table(GENDER_ROWS, "AdministrativeGender", None, "v2 table 0001", None, None, None), count=len(GENDER_ROWS))
    return cm_page("mappings", sections(gender_note(), tbl), gender_header(), GENDER_TABS, title=GENDER[1])


def page_concept_map_loaded_overview():
    said = facts([("Computer name", "v2.AdministrativeGender"), ("Publisher", "HL7 (FHIR Project)"), ("Date", "2019-11-01")])
    scope = facts([("Translates from", f'AdministrativeGender <span class="fui-muted">{canon("http://hl7.org/fhir/administrative-gender")}</span>'),
                   ("Translates to", f'v2 table 0001 <span class="fui-muted">{canon("http://terminology.hl7.org/CodeSystem/v2-0001")}</span>'),
                   ("Groups", "1"), ("Codes mapped", "4, one of them to two targets")])
    return cm_page("overview", sections(gender_note(), section("Details", said), section("What it maps", scope)), gender_header(), GENDER_TABS, title=GENDER[1])


def page_concept_map_loaded_json():
    cv = codeview("ConceptMap/cm-administrative-gender-v2", package_json(R4_GENDER_V2_MAP), '<span class="fui-tag">FHIR R4</span>', "The ConceptMap as JSON")
    return cm_page("json", cv, gender_header(), GENDER_TABS, title=GENDER[1])
