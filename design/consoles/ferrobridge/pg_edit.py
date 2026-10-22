# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The editor, a workbench like FerroFED's query console: B2 (the mapping
editor) beside B3 (its diagnostics, in step with the gutter), Check, and
Save through the reason and the confirmation in the docked bar."""
from common import (
    busy_button, button, card, copy_button, dock, dock_field, e, field, icon, input_, kbd, link_button, notice, page_heading,
    refusal, shell, toast, toasts, when,
)
from bridge import code_lines, diags_list, load_report, locked, owner_tag, quiet, short
from data import CHECK_WARN, CONTEXT_YAML, DRAFT_ERRORS, DRAFT_OK, DRAFT_YAML, REFUSED_ERRORS

FILE = "fhirconnect/synthetic_body_weight.context.yml"
OPENED = "5712e3f7…f2e0e3fa"
SAVED = "9c41d0e2…6b3fa7d2"


def editor_html(text, diags=(), current=None, readonly=False):
    ro = " readonly" if readonly else ""
    cls = "fui-editor" + (" is-readonly" if readonly else "")
    inv = ' aria-invalid="true"' if any(d[0] == "error" for d in diags) else ""
    return (f'<div class="{cls}"{inv}><div class="fui-editor-stack"><pre class="fui-editor-layer" aria-hidden="true">{code_lines(text, diags, current, inline=False)}</pre>'
            f'<textarea id="ed" name="ed" class="fui-editor-input" aria-label="The draft of {e(short(FILE))}" autocomplete="off" autocapitalize="off" spellcheck="false" wrap="soft"{ro}>{e(text)}</textarea></div></div>')


def editor_card(text, diags=(), current=None, checking=False, dirty=True, lang="FHIRconnect context", pos=None, owner="console", readonly=False, new=False):
    n = len(text.rstrip("\n").split("\n"))
    if pos is None:
        d0 = next((d for d in diags if d[3] == current), None)
        pos = f"Ln {d0[3]}, Col {d0[4]}" if d0 else "Ln 1, Col 1"
    if checking:
        check = busy_button("secondary", "Checking", small=True)
    elif readonly:
        check = f'<button type="button" class="fui-button fui-button-secondary fui-button-small" disabled>{icon("list-checks", "16")}Check</button>'
    else:
        check = button("secondary", "Check", "list-checks", small=True, keys="Ctrl Enter")
    bar = (f'<div class="fui-editor-bar"><span>YAML · {lang} · {n} lines</span>'
           f'<div class="fui-actions">{check}</div></div>')
    d = '<span>New file</span>' if new else ('<span class="fui-editor-dirty">Changed since opened</span>' if dirty else '<span>Unchanged</span>')
    status = (f'<div class="fui-editor-status"><span>{pos}</span><span>UTF-8 · LF · spaces: 2</span>{d}'
              f'<span class="fui-editor-status-end"><span>{owner_tag(owner)}</span><span class="fui-wide-only">F8 next diagnostic</span></span></div>')
    return f'<section class="fui-card fui-panel fui-mapping-editor" aria-label="Editor">{bar}{editor_html(text, diags, current, readonly)}{status}</section>'


def diag_card(state_="unchecked", diags=(), current=None, href="edit.html"):
    """B3 alone: the outcome of a check is said once, in the docked bar."""
    n = ""
    if state_ == "unchecked":
        body = quiet("Not checked yet: Check compiles the whole set with this draft, and writes nothing.")
    elif state_ == "checking":
        body = '<div class="fui-panel-body"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Compiling 27 files with your draft</span></div>'
    elif not diags:
        body = quiet("No diagnostics for this draft.")
    else:
        body = diags_list(diags, current=current, href=href)
        n = f' <span class="mk-count">· {len(diags)}</span>'
    return (f'<section class="fui-card fui-panel" aria-label="Diagnostics"><div class="fui-panel-head"><h2 class="fui-card-title"><span>Diagnostics{n}</span></h2></div>{body}</section>')


def msg(glyph, text):
    return f'<p class="fui-dock-message">{icon(glyph, "16")}<span>{text}</span></p>'


def disabled_primary(label, why):
    return f'<button type="button" class="fui-button fui-button-primary" disabled title="{e(why)}">{icon("save", "16")}{label}</button>'


def save_dock(reason="", state_="ready", error=None, label="Save and reload…", back="Discard", checked=None, back_href="file.html", blocked_by=None, extra=""):
    """The docked bar: the reason and Save. A disabled Save says why in the bar itself."""
    fld = dock_field("ed-reason", "Reason", reason, "Why you save it", error=error, help_="One line, at most 200 characters", disabled=state_ == "saving")
    m = ""
    if checked == "clean":
        m = msg("circle-check", "The set compiles with this draft: 0 errors, 1 warning.")
    if state_ == "confirm":
        m = msg("circle-alert", "<strong>Save and reload the set?</strong> The bridge compiles the set with the draft again and writes the file only when it compiles; a refused draft changes nothing.")
        acts = button("secondary", "Go back", keys="Esc") + button("primary", "Confirm the save", "save")
    elif state_ == "saving":
        acts = button("secondary", "Go back", attrs="disabled") + busy_button("primary", "Saving and reloading")
    elif state_ == "saved":
        acts = link_button("secondary", "Close", "file.html", "x")
    elif state_ in ("blocked", "blocked-quiet"):
        if state_ == "blocked":
            m = msg("circle-x", "The set does not compile with this draft: 2 errors. Fix them to save.")
        acts = link_button("secondary", back, back_href, "x") + disabled_primary(label, "The set does not compile with this draft")
    elif blocked_by:
        m = msg("circle-alert", blocked_by)
        acts = link_button("secondary", back, back_href, "x") + extra + disabled_primary(label, blocked_by)
    else:
        if not reason or error:
            why = "Give a reason to save." if not reason else "Fix the reason to save."
            if not error:
                m = m or msg("info", why)
            acts = link_button("secondary", back, back_href, "x") + disabled_primary(label, why)
        else:
            acts = link_button("secondary", back, back_href, "x") + button("primary", label, "save", keys="Ctrl S")
    return dock(fld, acts, m)


def edit_page(text=DRAFT_OK, diags=(), dstate="unchecked", current=None, checking=False, dock_html=None, strip="", title=None, sub=None, crumbs=None, overlay=None,
              toast_html="", owner="console", dirty=True, readonly=False, top_fields="", new=False, href="edit.html"):
    t = title or "Edit"
    s = sub or f'Your draft, opened {when("2026-10-21T09:31")} · SHA-256 <code>{OPENED}</code>.'
    h = page_heading(t, subtitle=s, crumbs=crumbs or [("Mapping set", "mappings-files.html"), (short(FILE), "file.html")], here=False)
    body = (f'<div class="fui-stack">{strip}{top_fields}<div class="fui-editor-split">{editor_card(text, diags, current, checking, dirty, owner=owner, readonly=readonly, new=new)}'
            f'{diag_card(dstate, diags, current, href)}</div></div>')
    return shell(f"{t} · {short(FILE)}" if t == "Edit" else t, "mappings", h + body, dock_html=dock_html if dock_html is not None else save_dock(), overlay=overlay, toast_html=toast_html)


def page_edit(overlay=None):
    return edit_page(overlay=overlay)


def page_edit_checking():
    return edit_page(checking=True, dstate="checking")


def page_edit_check_clean():
    return edit_page(diags=CHECK_WARN, dstate="clean", dock_html=save_dock(checked="clean"), href="edit-check-clean.html")


ERR_LINE = next(d[3] for d in DRAFT_ERRORS if d[1] == "unknown-extension")


def page_edit_check_errors():
    return edit_page(DRAFT_YAML, DRAFT_ERRORS, "errors", current=ERR_LINE, href="edit-check-errors.html", dock_html=save_dock("Name the detail model of the body weight context", state_="blocked"))


REASON = "Follow version 1.3.0 of the regional body weight profile"


def page_edit_save():
    return edit_page(diags=CHECK_WARN, dstate="clean", dock_html=save_dock(REASON, checked="clean"), href="edit-save.html")


def page_edit_save_confirm():
    return edit_page(diags=CHECK_WARN, dstate="clean", dock_html=save_dock(REASON, "confirm"), href="edit-save-confirm.html")


def page_edit_saving():
    return edit_page(diags=CHECK_WARN, dstate="clean", readonly=True, dock_html=save_dock(REASON, "saving"), href="edit-saving.html")


def page_edit_saved():
    rep = load_report("ok", f'Saved {when("2026-10-21T09:46")}, and the set reloaded with it. SHA-256 now <code>{SAVED}</code>',
                      [("0", "errors in the set"), ("3", "warnings in the set")], link_button("secondary", "Open the file", "file.html", "arrow-right", small=True))
    return edit_page(diags=CHECK_WARN, dstate="clean", strip=rep, dirty=False, sub=f'The file as saved {when("2026-10-21T09:46")} · SHA-256 <code>{SAVED}</code>.',
                     dock_html=save_dock("", state_="saved"), href="edit-saved.html")


REFUSED_LINE = REFUSED_ERRORS[0][3]


def page_edit_refused():
    rf = refusal("Not saved: at the save the set no longer compiled with your draft, so nothing was written and the running set is unchanged.",
                 request="PUT …/synthetic_body_weight.context.yml",
                 issues=[("bad", "error", "unmapped-start", f'a file the draft relies on changed after your check. <a href="edit-refused.html#L{REFUSED_LINE}">See line {REFUSED_LINE}</a>')])
    return edit_page(DRAFT_OK, REFUSED_ERRORS, "errors", current=REFUSED_LINE, href="edit-refused.html", strip=rf, dock_html=save_dock(REASON, state_="blocked-quiet"))


def page_edit_conflict():
    rf = refusal("The file changed since you opened it, so nothing was written.",
                 request="PUT …/synthetic_body_weight.context.yml · 412",
                 issues=[("neutral", "now", "e81f02a7…44c9b1d3", "saved by Demo Reviewer at 09:44 with the reason “Add the device cluster to the body weight context”.")])
    acts = (f'<div class="fui-cluster">{link_button("secondary", "Open the current version", "file.html", "arrow-right", small=True)}'
            f'{copy_button("Copy your draft", "your draft")}</div>')
    return edit_page(diags=CHECK_WARN, dstate="clean", strip=rf + acts, href="edit-conflict.html",
                     dock_html=save_dock(REASON, blocked_by="Your draft is based on an older version: open the current one and apply your change to it."))


PROV = "fhirconnect/synthetic_vital_signs.context.yml"


def path_card(value="fhirconnect/synthetic_vital_signs_local.context.yml", error=None):
    inv = ' aria-invalid="true"' if error else ""
    pf = field("ed-path", "New path", input_("ed-path", value, extra='spellcheck="false" class="fui-input fui-mono"' + inv),
               help_=None if error else "A path of its own, below fhirconnect/, ending in .yml.", error=error, cls="fui-span-row")
    return card("Where to save it", f'<div class="fui-form-grid">{pf}</div>')


def prov_page(dock_html, strip="", path_html=""):
    from pg_maps import PROV_YAML
    note = notice("info", "lock", "<strong>This file is provisioned,</strong> so the console never saves over it.")
    return edit_page(PROV_YAML.replace("version: 2.0.1", "version: 2.1.0"), [], "clean", strip=note + strip, owner="provisioned",
                     title="Check a draft", top_fields=path_html,
                     sub='A draft of a provisioned file, opened at SHA-256 <code>b20c9e41…07d1aa35</code>.',
                     crumbs=[("Mapping set", "mappings-files.html"), (short(PROV), "file-provisioned.html")], dock_html=dock_html)


PROV_REASON = "Local variant of vital signs with the regional profile"


def page_edit_provisioned():
    return prov_page(save_dock(PROV_REASON, label="Save under the new path…", back="Close", back_href="file-provisioned.html"), path_html=path_card())


def page_edit_provisioned_409():
    rf = refusal("Not saved: a provisioned file holds this path.", request="PUT …/synthetic_vital_signs.context.yml · 409")
    return prov_page(save_dock(PROV_REASON, label="Save under the new path…", back="Close", back_href="file-provisioned.html", blocked_by="Choose another path to save."),
                     strip=rf, path_html=path_card(PROV, "Choose a path no provisioned file holds."))


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
  archetypes:
    - "EVALUATION.adverse_reaction_risk.v1"
  start: "EVALUATION.adverse_reaction_risk.v1"
"""

PATH_ERROR = "Remove the .. step: a file stays inside the console's mapping directory."


def new_fields(path="fhirconnect/synthetic_allergy_reaction.context.yml", error=None):
    seg = ('<div role="radiogroup" aria-label="Language" class="fui-segmented"><label class="fui-segment"><input type="radio" class="fui-sr-only" name="lang" checked />FHIRconnect</label>'
           '<label class="fui-segment"><input type="radio" class="fui-sr-only" name="lang" />OMOCL</label></div>')
    kind = ('<div role="radiogroup" aria-label="Kind" class="fui-segmented"><label class="fui-segment"><input type="radio" class="fui-sr-only" name="kind" checked />Context</label>'
            '<label class="fui-segment"><input type="radio" class="fui-sr-only" name="kind" />Model</label><label class="fui-segment"><input type="radio" class="fui-sr-only" name="kind" />Search</label></div>')
    inv = ' aria-invalid="true"' if error else ""
    pf = field("nf-path", "Path", input_("nf-path", path, extra='spellcheck="false" class="fui-input fui-mono"' + inv),
               help_=None if error else "Below fhirconnect/ or omocl/, ending in .yml.", error=error, cls="fui-span-row")
    body = (f'<div class="fui-form-grid">{pf}<div class="fui-field fui-span-2"><span class="fui-label">Language</span>{seg}</div>'
            f'<div class="fui-field fui-span-2"><span class="fui-label">Kind</span>{kind}</div></div>')
    return card("The file", body)


NEW_SUB = "Written beside the provisioned files once the set compiles with it."
NEW_REASON = "Allergy reactions for the regional summary"


def page_edit_new():
    return edit_page(NEW_YAML, [], "unchecked", title="New mapping file", sub=NEW_SUB, new=True,
                     crumbs=[("Mapping set", "mappings.html")], top_fields=new_fields(), dock_html=save_dock(NEW_REASON, label="Create and reload…", back="Cancel", back_href="mappings.html"))


def page_edit_new_invalid():
    summary = ('<div class="fui-error-summary" role="alert"><p>' + icon("circle-alert", "16") + ' 1 field needs a change before you can create the file</p>'
               f'<ul><li><a href="#nf-path">Path: {PATH_ERROR}</a></li></ul></div>')
    return edit_page(NEW_YAML, [], "unchecked", title="New mapping file", sub=NEW_SUB, new=True,
                     crumbs=[("Mapping set", "mappings.html")], strip=summary,
                     top_fields=new_fields("fhirconnect/../allergy.context.yml", PATH_ERROR),
                     dock_html=save_dock(NEW_REASON, label="Create and reload…", back="Cancel", back_href="mappings.html", blocked_by="Fix the path to create the file."))


def page_edit_reason_invalid():
    bad = "Follow version 1.3.0 of the regional profile ✓"
    return edit_page(diags=CHECK_WARN, dstate="clean", href="edit-reason-invalid.html",
                     dock_html=save_dock(bad, error="Use plain letters, digits and punctuation: remove ✓.", checked=None))
