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

# One control and one wording for the version and the display language,
# shared with Look up.
LANGS = [("server", "The server's own choice"), ("en", "English (en)"), ("nl", "Dutch (nl)")]
VERSIONS = {"sf": [("3", "3 (default)")], "national": [("2026-10", "2026-10 (default)")], "iso": [("48", "CLDR 48 (default)")], "": [("", "Choose a code system first")]}


def version_field(id_, system, version):
    return field(id_, "Version", select(id_, VERSIONS[system], version, extra="disabled" if not system else ""))


def language_field(id_, lang="server"):
    return field(id_, "Display language", select(id_, LANGS, lang, extra='title="Sent as displayLanguage on every call this screen makes"'))


def options(system="sf", version="3", lang="server", lang_open=False):
    sysopts = [("", "Choose a code system"), ("sf", "Synthetic clinical findings"), ("national", "Synthetic national findings"), ("iso", "ISO 3166-1 country codes")]
    f = field("b-sys", "Code system", select("b-sys", sysopts, system)) + version_field("b-ver", system, version) + language_field("b-lang", lang)
    return f'<div class="fui-transform-options">{f}</div>'


def pane(title, body, actions=""):
    a = f'<div class="fui-actions">{actions}</div>' if actions else ""
    return f'<section class="fui-browser-pane" aria-label="{e(title)}"><div class="fui-browser-head"><h2 class="fui-browser-title">{title}</h2>{a}</div>{body}</section>'


def line(text):
    """An empty pane: one quiet line at the top, under the pane's heading."""
    return f'<div class="fui-browser-body"><p class="fui-small fui-muted">{text}</p></div>'


def search_pane(value="", matches=(), selected=None, none=False, busy=False, disabled=False):
    """One search field, the icon inside, searching on Enter; its label is
    for assistive technology only."""
    dis = " disabled" if disabled else ""
    sf = (f'<div class="fui-browser-body"><form class="fui-browser-find" role="search"><div class="fui-list-find">'
          f'<span class="fui-search-glyph">{icon("search", "16")}</span><label for="b-q" class="fui-sr-only">Search the designations</label>'
          f'{input_("b-q", value, "Search the designations", "search", extra=dis.strip())}</div></form>')
    if disabled:
        body = sf + '<p class="fui-small fui-muted">Choose a code system first.</p></div>'
    elif busy:
        body = sf + '<p class="fui-small"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Searching</span></p></div>'
    elif none:
        body = sf + f'<p class="fui-small fui-muted">No concept matches “{e(value)}”.</p></div>'
    elif matches:
        items = "".join(f'<li>{coding(d, c, inactive=i, href="browse.html", current=c == selected)}</li>' for c, d, i in matches)
        body = sf + f'<p class="fui-small fui-muted" role="status">{plural(len(matches), "concept")}</p></div><ul class="fui-match-list" aria-label="Concepts the search matched">{items}</ul>'
    else:
        body = sf + '<p class="fui-small fui-muted">Search to see concepts.</p></div>'
    return pane("Search", body)


def sf_tree(selected="SF-1210", focus=None, expand_general=False, loading=False):
    """The walk to the selected concept: its ancestors open, every other
    branch closed."""
    def leaf(code):
        d, p, i, _ = BY[code]
        return tree_item(code, d, leaf=True, selected=code == selected, inactive=i, focus=focus == code)
    cough_open = selected in ("SF-1210", "SF-1211", "SF-1212") or focus in ("SF-1211", "SF-1212")
    cough = tree_item("SF-1210", "Cough", [leaf("SF-1211"), leaf("SF-1212")], expanded=cough_open, selected=selected == "SF-1210",
                      on_path=selected in ("SF-1211", "SF-1212"), focus=focus == "SF-1210")
    chest_open = selected in ("SF-1200", "SF-1210", "SF-1211", "SF-1212", "SF-1220")
    chest = tree_item("SF-1200", "Finding of the chest", [cough, leaf("SF-1220")], expanded=chest_open, selected=selected == "SF-1200",
                      on_path=chest_open and selected != "SF-1200")
    general_kids = [leaf("SF-1310"), leaf("SF-1320"), leaf("SF-1390")] if expand_general else []
    general = tree_item("SF-1300", "General finding", general_kids, expanded=expand_general or loading, on_path=selected == "SF-1390", loading=loading and not expand_general)
    root = tree_item("SF-1000", "Synthetic finding", [tree_item("SF-1100", "Finding of the skin"), chest, general, tree_item("SF-1400", "Finding of the abdomen")], expanded=True, on_path=True)
    return concept_tree([root])


def tree_pane(tree_html=None, start=False):
    if start:
        return pane("Hierarchy", line("Choose a code system to walk its hierarchy."))
    return pane("Hierarchy", tree_html)


def prop_label(word, code):
    return f'{word}<span class="fui-row-sub"><code>{e(code)}</code></span>'


def concept_body(title, code, ver, designations, props, rel_rows, inactive=False):
    """The concept as Look up answers it: its display, its designations and
    its properties, each property in words with its code as a hint."""
    mark = f" {inactive_mark()}" if inactive else ""
    retired = '<p class="fui-small">Retired in its code system; the server still serves it.</p>' if inactive else ""
    rel = ""
    if rel_rows:
        rel = '<dl class="fui-facts fui-browser-rel">' + "".join(f"<dt>{k}</dt><dd>{v}</dd>" for k, v in rel_rows) + "</dl>"
    head = (f'<div class="fui-browser-body"><h3 class="fui-concept-title">{e(title)}</h3>'
            f'<p class="fui-small fui-muted"><code>{e(code)}</code> · read from version {ver}{mark}</p>{retired}{rel}</div>')
    dt = table("Designations", th("Term") + th("Language") + th("Use"),
               [f'<tr><th scope="row" data-role="title">{e(t)}</th>{td("Language", l, role="sub")}{td("Use", u, role="meta-q")}</tr>' for t, l, u in designations])
    pt = table("Properties", th("Property") + th("Value"),
               [f'<tr><th scope="row" data-role="title">{prop_label(w, k)}</th>{td("Value", v, role="sub")}</tr>' for w, k, v in props])
    return head + section("Designations", dt, count=len(designations)) + section("Properties", pt, count=len(props))


def all_props():
    return link_button("ghost", "All properties", "lookup-answered.html", glyph_end="arrow-right", small=True, attrs='title="This concept\'s whole Look up answer, in Operations"')


def concept_pane(code="SF-1210", start=False, iso=False):
    if start:
        return pane("The concept", line("Choose a concept to read it."))
    if iso:
        body = concept_body("Netherlands", "NL", "CLDR 48", [("Nederland", "Dutch (nl)", "Display")], [("Display", "display", "Netherlands")], [])
        return pane("The concept", body, all_props())
    d, p, inactive, nl = BY[code]
    kids = [c for c, (_, pp, _, _) in BY.items() if pp == code]
    rel_rows = []
    if p:
        rel_rows.append(("Parent", f'<span class="fui-browser-rel-item">{coding(BY[p][0], p, inline=True, href="browse.html")}</span>'))
    if kids:
        rel_rows.append(("Children", "".join(f'<span class="fui-browser-rel-item">{coding(BY[x][0], x, inline=True, href="browse.html")}</span>' for x in kids)))
    props = ([("Parent", "parent", coding(BY[p][0], p, inline=True))] if p else []) + [("Child", "child", coding(BY[k][0], k, inline=True)) for k in kids] + [("Inactive", "inactive", "Yes" if inactive else "No")]
    body = concept_body(d, code, "3", [(nl, "Dutch (nl)", "Display")], props, rel_rows, inactive)
    return pane("The concept", body, all_props())


def browse_page(opts, search, tree, concept, status="up", two=False):
    h = page_heading("Concept browser", subtitle=SUB)
    panes = f"{search}{tree}{concept}"
    panel = f'<div class="fui-card fui-panel">{opts}<div class="fui-browser{" is-two-pane" if two else ""}">{panes}</div></div>'
    return shell("Concept browser", "browse", h + f'<div class="fui-stack">{panel}</div>', status=status)


COUGH = [("SF-1210", "Cough", False), ("SF-1211", "Dry cough", False), ("SF-1212", "Productive cough", False)]


def page_browse():
    return browse_page(options(), search_pane("cough", COUGH, "SF-1210"), tree_pane(sf_tree()), concept_pane())


def page_browse_results():
    return browse_page(options(), search_pane("chest", [("SF-1200", "Finding of the chest", False)], "SF-1200"), tree_pane(sf_tree("SF-1200")), concept_pane("SF-1200"))


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
    m = [("NL", "Netherlands", False), ("BQ", "Caribbean Netherlands", False)]
    return browse_page(options("iso", "48"), search_pane("netherlands", m, "NL"), "", concept_pane(iso=True), two=True)


def page_browse_start():
    return browse_page(options("", ""), search_pane(disabled=True), tree_pane(start=True), concept_pane(start=True))


NF_KIDS = ["Cough lasting over three weeks", "Blood in sputum", "Chest pain on exertion", "Shortness of breath at rest", "Unexplained weight loss",
           "Night sweats", "Hoarseness over three weeks", "Recurrent chest infection", "Wheeze in an adult", "Chest pain at rest",
           "Palpitations with fainting", "Swelling of both ankles", "Shortness of breath lying flat", "Clubbed fingers", "Abnormal chest X-ray",
           "Pleural effusion", "Persistent cough in a smoker", "Coughing up mucus daily", "Breathlessness on stairs", "Raised heart rate at rest",
           "Low oxygen on a home reading", "Chest wall pain", "Snoring with pauses", "Cough after an infection", "Barking cough"]


def page_browse_more():
    kids = [tree_item(f"NF-2{n:02d}", d, leaf=True) for n, d in enumerate(NF_KIDS, start=1)]
    more = '<span>25 of 40 children shown. <a href="browse-more.html">Show the other 15</a></span>'
    resp = tree_item("NF-200", "Reasons for a chest referral", kids, expanded=True, more=more, on_path=True, selected=True)
    root = tree_item("NF-100", "Synthetic national finding", [resp, tree_item("NF-300", "Reasons for a skin referral")], expanded=True, on_path=True)
    tree = concept_tree([root], "The hierarchy of Synthetic national findings")
    rel = [("Parent", f'<span class="fui-browser-rel-item">{coding("Synthetic national finding", "NF-100", inline=True, href="browse.html")}</span>'), ("Children", "40 (see the tree)")]
    props = [("Parent", "parent", coding("Synthetic national finding", "NF-100", inline=True)), ("Children", "child", "40 (see the tree)"), ("Inactive", "inactive", "No")]
    body = concept_body("Reasons for a chest referral", "NF-200", "2026-10", [("Redenen voor een verwijzing voor de borst", "Dutch (nl)", "Display")], props, rel)
    concept = pane("The concept", body, all_props())
    return browse_page(options("national", "2026-10"), search_pane(), tree_pane(tree), concept)
