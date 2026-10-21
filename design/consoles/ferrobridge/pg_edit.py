# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The editor, a workbench like FerroFED's query console: B2 (the mapping
editor) beside B3 (its diagnostics, in step with the gutter), Check, and
Save through the reason and the confirmation in the docked bar."""
from common import (
    busy_button, button, card, dock, dock_field, e, empty_quiet, field, icon, input_, kbd, link_button, notice, page_heading,
    refusal, shell, toast, toasts, when,
)
from bridge import code_lines, diags_list, load_report, locked, owner_tag, short
from data import CONTEXT_YAML, DRAFT_ERRORS, DRAFT_YAML

FILE = "fhirconnect/synthetic_body_weight.context.yml"
DRAFT_OK = CONTEXT_YAML.replace("version: 1.2.0", "version: 1.3.0").replace("    - code\n", "    - code\n    - value-quantity\n")
CHECK_WARN = [("warning", "search-unbound-parameter", FILE, 18, 5, "The context names the search parameter date, which no projection binds, so the facade does not offer it.", None)]


def editor_html(text, diags=(), current=None, readonly=False):
    ro = " readonly" if readonly else ""
    cls = "fui-editor" + (" is-readonly" if readonly else "")
    inv = ' aria-invalid="true"' if any(d[0] == "error" for d in diags) else ""
    return (f'<div class="{cls}"{inv}><div class="fui-editor-stack"><pre class="fui-editor-layer" aria-hidden="true">{code_lines(text, diags, current, inline=False)}</pre>'
            f'<textarea id="ed" name="ed" class="fui-editor-input" aria-label="The draft of {e(short(FILE))}" autocomplete="off" autocapitalize="off" spellcheck="false" wrap="soft"{ro}>{e(text)}</textarea></div></div>')


def editor_card(text, diags=(), current=None, checking=False, dirty=True, lang="FHIRconnect context", pos="Ln 25, Col 11", owner="console", readonly=False):
    n = len(text.rstrip("\n").split("\n"))
    ne = sum(1 for d in diags if d[0] == "error")
    nw = len(diags) - ne
    marks = ""
    if diags:
        marks = f' · <span>{ne} error{"s" if ne != 1 else ""}, {nw} warning{"s" if nw != 1 else ""}</span>'
    check = busy_button("secondary", "Checking", small=True) if checking else button("secondary", "Check", "list-checks", small=True, keys="Ctrl Enter")
    bar = (f'<div class="fui-editor-bar"><span>YAML · {lang} · {n} lines{marks}</span>'
           f'<div class="fui-actions">{button("ghost", "Revert", "rotate-ccw", small=True)}{check}</div></div>')
    d = '<span class="fui-editor-dirty">Changed since opened</span>' if dirty else '<span>Unchanged</span>'
    status = (f'<div class="fui-editor-status"><span>{pos}</span><span>UTF-8 · LF · spaces: 2</span>{d}'
              f'<span class="fui-editor-status-end"><span>{owner_tag(owner)}</span><span>F8 next diagnostic</span></span></div>')
    return f'<section class="fui-card fui-panel fui-mapping-editor" aria-label="Editor">{bar}{editor_html(text, diags, current, readonly)}{status}</section>'


def diag_card(state_="unchecked", diags=(), current=None):
    """B3 alone: the outcome of a check is said once, in the docked bar."""
    n = ""
    if state_ == "unchecked":
        body = empty_quiet("list-checks", "Not checked yet", "Check compiles the whole set with this draft. Nothing is written.")
    elif state_ == "checking":
        body = '<div class="fui-panel-body"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Compiling 27 files with your draft</span></div>'
    elif not diags:
        body = empty_quiet("circle-check", "No diagnostic", "The set compiles with this draft.")
    else:
        body = diags_list(diags, current=current, href="edit-check-errors.html")
        n = f' <span class="mk-count">· {len(diags)}</span>'
    return (f'<section class="fui-card fui-panel" aria-label="Diagnostics"><div class="fui-panel-head"><h2 class="fui-card-title"><span>Diagnostics{n}</span></h2></div>{body}</section>')


def save_dock(reason="", state_="ready", error=None, label="Save and reload…", back="Discard", path_field="", checked=None, back_href="file.html"):
    fld = dock_field("ed-reason", "Reason", reason, "Why you save it", error=error, help_="One line, at most 200 characters")
    msg = ""
    if checked == "clean":
        msg = f'<p class="fui-dock-message">{icon("circle-check", "16")}<span>The set compiles with this draft: 0 errors, 1 warning.</span></p>'
    if state_ == "confirm":
        msg = (f'<p class="fui-dock-message">{icon("circle-alert", "16")}<span><strong>Save and reload the set?</strong> '
               'The bridge compiles the set with the draft again and writes the file only when it compiles; a refused draft changes nothing.</span></p>')
        acts = button("secondary", "Go back", keys="Esc") + button("primary", "Confirm the save", "save")
    elif state_ == "saving":
        acts = button("secondary", "Go back", attrs="disabled") + busy_button("primary", "Saving and reloading")
    elif state_ in ("blocked", "blocked-quiet"):
        if state_ == "blocked":
            msg = f'<p class="fui-dock-message">{icon("circle-x", "16")}<span>The set does not compile with this draft: 2 errors. Fix them to save.</span></p>'
        acts = link_button("secondary", back, back_href, "x") + f'<button type="button" class="fui-button fui-button-primary" disabled title="The set does not compile with this draft">{icon("save", "16")}{label}</button>'
    else:
        dis = "" if reason and not error else "disabled"
        why = ' title="Give a reason first"' if not reason else (' title="Fix the reason first"' if error else "")
        acts = link_button("secondary", back, back_href, "x") + (f'<button type="button" class="fui-button fui-button-primary" disabled{why}>{icon("save", "16")}{label}</button>' if dis else button("primary", label, "save", keys="Ctrl s"))
    return dock(path_field + fld, acts, msg)


def edit_page(text=DRAFT_OK, diags=(), dstate="unchecked", current=None, checking=False, dock_html=None, strip="", title=None, sub=None, crumbs=None, overlay=None,
              toast_html="", owner="console", dirty=True, readonly=False, top_fields=""):
    t = title or "Edit"
    s = sub or f'Your draft, opened at SHA-256 <code>5712e3f7…f2e0e3fa</code> {when("2026-10-21T09:31")}.'
    h = page_heading(t, subtitle=s, crumbs=crumbs or [("Mapping set", "mappings-files.html"), (short(FILE), "file.html")])
    body = (f'<div class="fui-stack">{strip}{top_fields}<div class="fui-editor-split">{editor_card(text, diags, current, checking, dirty, owner=owner, readonly=readonly)}'
            f'{diag_card(dstate, diags, current)}</div></div>')
    return shell(f"{t} · {short(FILE)}" if t == "Edit" else t, "mappings", h + body, dock_html=dock_html if dock_html is not None else save_dock(), overlay=overlay, toast_html=toast_html)


def page_edit(overlay=None):
    return edit_page(overlay=overlay)


def page_edit_checking():
    return edit_page(checking=True, dstate="checking")


def page_edit_check_clean():
    return edit_page(diags=CHECK_WARN, dstate="clean", dock_html=save_dock(checked="clean"))


def page_edit_check_errors():
    return edit_page(DRAFT_YAML, DRAFT_ERRORS, "errors", current=25, dock_html=save_dock("Name the detail model of the body weight context", state_="blocked"))


REASON = "Offer value-quantity search on body weight for the regional dashboard"


def page_edit_save():
    return edit_page(diags=CHECK_WARN, dstate="clean", dock_html=save_dock(REASON, checked="clean"))


def page_edit_save_confirm():
    return edit_page(diags=CHECK_WARN, dstate="clean", dock_html=save_dock(REASON, "confirm"))


def page_edit_saving():
    return edit_page(diags=CHECK_WARN, dstate="clean", dock_html=save_dock(REASON, "saving"))


def page_edit_saved():
    rep = load_report("ok", f'Saved {when("2026-10-21T09:46")}, and the set reloaded with it · SHA-256 now <code>9c41d0e2…6b3fa7d2</code>',
                      [("9", "contexts"), ("27", "files"), ("0", "errors"), ("3", "warnings")], link_button("secondary", "Open the file", "file.html", "arrow-right", small=True))
    return edit_page(diags=CHECK_WARN, dstate="clean", strip=rep, dirty=False, dock_html=save_dock("", label="Save and reload…"))


def page_edit_refused():
    rf = refusal("Not saved: at the save the set no longer compiled with your draft, so nothing was written and the running set is unchanged.",
                 request="PUT …/synthetic_body_weight.context.yml",
                 issues=[("bad", "error", "unknown-model", "the context names body_weight_detail, which no file of the set defines. A file the draft relied on was changed since your check.")])
    return edit_page(DRAFT_YAML, DRAFT_ERRORS, "errors", current=25, strip=rf, dock_html=save_dock(REASON, state_="blocked-quiet"))


def page_edit_conflict():
    rf = refusal("412: the file changed since you opened it, so nothing was written.",
                 request="PUT …/synthetic_body_weight.context.yml",
                 issues=[("neutral", "now", "e81f02a7…44c9b1d3", "saved by Demo Reviewer at 09:44 with the reason “Add the code search parameter”.")])
    acts = f'<div class="fui-cluster">{link_button("secondary", "Open the current version", "file.html", "arrow-right", small=True)}<span class="fui-small fui-muted">Your draft stays in the editor; copy it before you leave.</span></div>'
    return edit_page(diags=CHECK_WARN, dstate="clean", strip=rf + acts, dock_html=save_dock(REASON))


PROV = "fhirconnect/synthetic_vital_signs.context.yml"


def path_field(value="fhirconnect/synthetic_vital_signs_local.context.yml", error=None):
    inv = ' aria-invalid="true" aria-describedby="ed-path-error"' if error else ""
    er = f'<p id="ed-path-error" class="fui-error">{icon("circle-alert", "14")}{error}</p>' if error else ""
    return (f'<div class="fui-dock-field"><label for="ed-path" class="fui-label">New path</label>'
            f'{input_("ed-path", value, extra="spellcheck=" + chr(34) + "false" + chr(34) + " class=" + chr(34) + "fui-input fui-mono" + chr(34) + inv)}{er}</div>')


def prov_page(dock_html, strip=""):
    from pg_maps import PROV_YAML
    note = notice("info", "lock", "<strong>This file is provisioned,</strong> so the console never saves over it. To keep your draft, save it under a path of its own.")
    return edit_page(PROV_YAML.replace("version: 2.0.1", "version: 2.1.0"), CHECK_WARN[:0], "clean", strip=note + strip, owner="provisioned",
                     title="Check a draft",
                     sub='A draft of a provisioned file, opened at SHA-256 <code>b20c9e41…07d1aa35</code>.',
                     crumbs=[("Mapping set", "mappings-files.html"), (short(PROV), "file-provisioned.html")], dock_html=dock_html)


def page_edit_provisioned():
    return prov_page(save_dock("Local variant of vital signs with the regional profile", label="Save under the new path…", back="Close", path_field=path_field(), back_href="file-provisioned.html"))


def page_edit_provisioned_409():
    rf = refusal("409: a provisioned file holds this path, and the console never writes over one. Save the draft under a path of its own.",
                 request="PUT …/synthetic_vital_signs.context.yml")
    return prov_page(save_dock("Local variant of vital signs with the regional profile", label="Save under the new path…", back="Close",
                               path_field=path_field(PROV, "A provisioned file holds this path: choose another."), back_href="file-provisioned.html"), strip=rf)


NEW_YAML = """grammar: FHIRConnect/v1.0.0
type: context
metadata:
  name: synthetic_allergy_reaction.context
  version: 0.1.0
spec:
  system: FHIR
  version: R4
context:
  profile:
    url: "http://example.org/fhir/StructureDefinition/synthetic-allergy"
  template:
    id: "synthetic.allergy_list.v1"
  start: "openEHR-EHR-EVALUATION.adverse_reaction_risk.v1"
mappings:
  - name: adverse_reaction
"""


def new_fields(path="fhirconnect/synthetic_allergy_reaction.context.yml", error=None):
    seg = ('<div role="radiogroup" aria-label="Language" class="fui-segmented"><label class="fui-segment"><input type="radio" class="fui-sr-only" name="lang" checked />FHIRconnect</label>'
           '<label class="fui-segment"><input type="radio" class="fui-sr-only" name="lang" />OMOCL</label></div>')
    kind = ('<div role="radiogroup" aria-label="Kind" class="fui-segmented"><label class="fui-segment"><input type="radio" class="fui-sr-only" name="kind" checked />Context</label>'
            '<label class="fui-segment"><input type="radio" class="fui-sr-only" name="kind" />Model</label><label class="fui-segment"><input type="radio" class="fui-sr-only" name="kind" />Search</label></div>')
    inv = ' aria-invalid="true"' if error else ""
    pf = field("nf-path", "Path", input_("nf-path", path, extra='spellcheck="false" class="fui-input fui-mono"' + inv),
               help_=None if error else "Below fhirconnect/ or omocl/, ending in .yml.", error=error)
    body = (f'<div class="fui-form-grid">{pf}<div class="fui-field"><span class="fui-label">Language</span>{seg}</div><div class="fui-field"><span class="fui-label">Kind</span>{kind}</div></div>'
            '')
    return card("The file", body)


def page_edit_new():
    return edit_page(NEW_YAML, [], "unchecked", title="New mapping file", sub="Written beside the provisioned files once the set compiles with it.",
                     crumbs=[("Mapping set", "mappings.html")], top_fields=new_fields(), dock_html=save_dock("Allergy reactions for the regional summary", label="Create and reload…", back="Cancel", back_href="mappings.html"))


def page_edit_new_invalid():
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 1 field needs a change before you can create the file</p>'
               '<ul><li><a href="#nf-path">Path: the mapping file path is not plain names below a root</a></li></ul></div>')
    return edit_page(NEW_YAML, [], "unchecked", title="New mapping file", sub="Written beside the provisioned files once the set compiles with it.",
                     crumbs=[("Mapping set", "mappings.html")], strip=summary,
                     top_fields=new_fields("fhirconnect/../allergy.context.yml", "Not plain names below a root: remove the .. step, so the file stays inside mapping_dir."),
                     dock_html=save_dock("Allergy reactions for the regional summary", label="Create and reload…", back="Cancel", back_href="mappings.html"))


def page_edit_reason_invalid():
    bad = "Offer value-quantity search ✓ for the dashboard"
    return edit_page(diags=CHECK_WARN, dstate="clean",
                     dock_html=save_dock(bad, error="Use printable ASCII only: remove ✓.", checked=None))
