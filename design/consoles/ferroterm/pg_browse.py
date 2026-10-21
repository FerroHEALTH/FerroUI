# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The concept browser, a workbench and the one home of a concept: the code
system, version and language in one options row, then three panes: the
search, the hierarchy (M1, one level at a time) and the concept read with
$lookup. Every move is in the address, so a walk is a link."""
from common import button, e, empty_quiet, field, icon, input_, link_button, notice, page_heading, select, shell, table, td, th
from data import FINDINGS
from term import coding, concept_tree, inactive_mark, plural, quiet, section, tree_item

SUB = "Search a code system, read a concept and walk its hierarchy."
BY = {c: (d, p, i, nl) for c, d, p, i, nl in FINDINGS}


def options(system="sf", version="3", lang="server", lang_open=False):
    sysopts = [("sf", "Synthetic clinical findings"), ("national", "Synthetic national findings"), ("iso", "ISO 3166-1 country codes"), ("", "Choose a code system")]
    vers = {"sf": [("3", "3 (default)")], "national": [("2026-10", "2026-10 (default)")], "iso": [("48", "48 (default)")], "": [("", "Choose a code system first")]}[system]
    langs = [("server", "The server's own choice"), ("en", "English (en)"), ("nl", "Dutch (nl)")]
    f = (field("b-sys", "Code system", select("b-sys", sysopts, system))
         + field("b-ver", "Version", select("b-ver", vers, version, extra="disabled" if not system else ""))
         + field("b-lang", "Display language", select("b-lang", langs, lang, extra='title="Sent as displayLanguage on every call this screen makes"')))
    return f'<div class="fui-transform-options">{f}</div>'


def pane(title, body, actions=""):
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    return f'<section class="fui-browser-pane" aria-label="{e(title)}"><div class="fui-browser-head"><h2 class="fui-browser-title">{title}</h2>{a}</div>{body}</section>'


def search_pane(value="", matches=(), selected=None, none=False, busy=False):
    """One search field, the icon inside, searching on Enter; its label is
    for assistive technology only."""
    sf = (f'<div class="fui-browser-body"><form class="fui-browser-find" role="search"><div class="fui-list-find">'
          f'<span class="fui-search-glyph">{icon("search", "16")}</span><label for="b-q" class="fui-sr-only">Search the designations</label>'
          f'{input_("b-q", value, "Search the designations", "search")}</div></form>')
    if busy:
        body = sf + '<p class="fui-small"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Searching</span></p></div>'
    elif none:
        body = sf + "</div>" + quiet(f"No concept matches “{e(value)}”.")
    elif matches:
        items = "".join(f'<li{" aria-selected=\"true\"" if c == selected else ""}>{coding(d, c, inactive=i, href="browse.html")}</li>' for c, d, i in matches)
        body = sf + f'<p class="fui-small fui-muted" role="status">{plural(len(matches), "concept")}</p></div><ul class="fui-match-list" aria-label="Concepts the search matched">{items}</ul>'
    else:
        body = sf + "</div>" + quiet("Search to see concepts.")
    return pane("Search", body)


def sf_tree(selected="SF-1210", focus=None, expand_general=False, loading=False):
    def leaf(code, on_path=False):
        d, p, i, _ = BY[code]
        return tree_item(code, d, leaf=True, selected=code == selected, inactive=i, focus=focus == code)
    cough = tree_item("SF-1210", "Cough", [leaf("SF-1211"), leaf("SF-1212")], expanded=True, selected=selected == "SF-1210", on_path=selected in ("SF-1211", "SF-1212"), focus=focus == "SF-1210")
    chest = tree_item("SF-1200", "Finding of the chest", [cough, leaf("SF-1220")], expanded=selected not in ("SF-1390",), on_path=True)
    general_kids = [leaf("SF-1310"), leaf("SF-1320"), leaf("SF-1390")] if expand_general else []
    general = tree_item("SF-1300", "General finding", general_kids, expanded=expand_general or loading, on_path=selected == "SF-1390", loading=loading and not expand_general)
    if selected == "SF-1390":
        chest = tree_item("SF-1200", "Finding of the chest", expanded=False)
    root = tree_item("SF-1000", "Synthetic finding", [tree_item("SF-1100", "Finding of the skin"), chest, general, tree_item("SF-1400", "Finding of the abdomen")], expanded=True, on_path=True)
    return concept_tree([root])


def tree_pane(tree_html=None, flat=False, start=False):
    if start:
        body = quiet("Choose a code system to walk its hierarchy.")
    elif flat:
        body = quiet("This version has no hierarchy to walk: its concepts are a flat list.")
    else:
        body = tree_html
    return pane("Hierarchy", body)


def concept_pane(code="SF-1210", start=False, iso=False):
    if start:
        return pane("The concept", quiet("Choose a concept to read it."))
    if iso:
        d, nl, parents, kids, inactive = "Netherlands", "Nederland", [], [], False
        ver, props = "48", [("display", "Netherlands")]
        desig = [("Netherlands", "English (en)", "display"), ("Nederland", "Dutch (nl)", "display")]
    else:
        d, p, inactive, nl = BY[code]
        ver = "3"
        parents = [p] if p else []
        kids = [c for c, (_, pp, _, _) in BY.items() if pp == code]
        props = [("parent", p or "none")] + [("child", k) for k in kids] + [("inactive", "true" if inactive else "false")]
        desig = [(d, "English (en)", "display"), (nl, "Dutch (nl)", "display")]
    rel = ""
    if parents or kids:
        par = "".join(f'<span class="fui-browser-rel-item">{coding(BY[x][0], x, inline=True, href="browse.html")}</span>' for x in parents) or '<span class="fui-muted">none</span>'
        ch = "".join(f'<span class="fui-browser-rel-item">{coding(BY[x][0], x, inline=True, href="browse.html")}</span>' for x in kids) or '<span class="fui-muted">none</span>'
        rel = f'<dl class="fui-facts fui-browser-rel"><dt>Parents</dt><dd>{par}</dd><dt>Children</dt><dd>{ch}</dd><dt>Inactive</dt><dd>{"Yes" if inactive else "No"}</dd></dl>'
    mark = f" {inactive_mark()}" if inactive else ""
    note = notice("neutral", "info", "This concept is inactive in its code system. The server served it and marked it, which is an answer rather than a refusal.") if inactive else ""
    head = (f'<div class="fui-browser-body"><h3 class="fui-concept-title">{e(d)}</h3>'
            f'<p class="fui-small fui-muted"><code>{e("NL" if iso else code)}</code> · read from version {ver}{mark}</p>{note}{rel}</div>')
    dt = table("Designations", th("Term") + th("Language") + th("Use"), [f'<tr><th scope="row" data-role="title">{e(t)}</th>{td("Language", l, role="sub")}{td("Use", u, role="meta-q")}</tr>' for t, l, u in desig])
    pt = table("Properties", th("Property") + th("Value"), [f'<tr><th scope="row" data-role="title"><code>{e(k)}</code></th>{td("Value", f"<code>{e(v)}</code>", role="sub")}</tr>' for k, v in props])
    body = head + f'<div><h4 class="fui-clause-sub fui-browser-sub">Designations</h4>{dt}</div>'
    acts = link_button("ghost", "All properties", "lookup-answered.html", "search", small=True, attrs='title="This concept\'s whole $lookup answer, in Operations"')
    return pane("The concept", body, acts)


def browse_page(opts, search, tree, concept, status="up"):
    h = page_heading("Concept browser", subtitle=SUB)
    panel = f'<div class="fui-card fui-panel">{opts}<div class="fui-browser">{search}{tree}{concept}</div></div>'
    return shell("Concept browser", "browse", h + f'<div class="fui-stack">{panel}</div>', status=status)


COUGH = [("SF-1210", "Cough", False), ("SF-1211", "Dry cough", False), ("SF-1212", "Productive cough", False)]


def page_browse():
    return browse_page(options(), search_pane("cough", COUGH, "SF-1210"), tree_pane(sf_tree()), concept_pane())


def page_browse_results():
    return browse_page(options(), search_pane("respir", [("SF-1200", "Finding of the chest", False)], "SF-1200"), tree_pane(sf_tree("SF-1200")), concept_pane("SF-1200"))


def page_browse_keyboard():
    return browse_page(options(), search_pane("cough", COUGH, "SF-1211"), tree_pane(sf_tree("SF-1211", focus="SF-1212")), concept_pane("SF-1211"))


def page_browse_inactive():
    m = [("SF-1390", "Malaise, old code", True)]
    return browse_page(options(), search_pane("malaise", m, "SF-1390"), tree_pane(sf_tree("SF-1390", expand_general=True)), concept_pane("SF-1390"))


def page_browse_loading():
    return browse_page(options(), search_pane("cough", COUGH, "SF-1210"), tree_pane(sf_tree(loading=True)), concept_pane())


def page_browse_no_match():
    return browse_page(options(), search_pane("asthma", none=True), tree_pane(sf_tree()), concept_pane())


def page_browse_flat():
    m = [("NL", "Netherlands", False), ("NO", "Norway", False), ("NZ", "New Zealand", False)]
    return browse_page(options("iso", "48"), search_pane("n", m, "NL"), tree_pane(flat=True), concept_pane(iso=True))


def page_browse_start():
    return browse_page(options("", ""), search_pane(), tree_pane(start=True), concept_pane(start=True))


def page_browse_more():
    kids = [tree_item(f"NF-2{n:02d}", d, leaf=True) for n, d in [(1, "Cough lasting over three weeks"), (2, "Blood in sputum"), (3, "Chest pain on exertion"), (4, "Shortness of breath at rest")]]
    more = '40 children, of which the page shows 25 · <a href="browse-more.html">Show the next 25</a>'
    resp = tree_item("NF-200", "Reasons for a chest referral", kids, expanded=True, more=more, on_path=True, selected=True)
    root = tree_item("NF-100", "Synthetic national finding", [resp, tree_item("NF-300", "Reasons for a skin referral")], expanded=True, on_path=True)
    tree = concept_tree([root], "The hierarchy of Synthetic national findings")
    concept = pane("The concept", '<div class="fui-browser-body"><h3 class="fui-concept-title">Reasons for a chest referral</h3><p class="fui-small fui-muted"><code>NF-200</code> · read from version 2026-10</p>'
                   '<dl class="fui-facts fui-browser-rel"><dt>Parents</dt><dd>' + coding("Synthetic national finding", "NF-100", inline=True, href="browse.html") + '</dd><dt>Children</dt><dd>40, in the tree</dd></dl></div>')
    return browse_page(options("national", "2026-10"), search_pane(), tree_pane(tree), concept)
