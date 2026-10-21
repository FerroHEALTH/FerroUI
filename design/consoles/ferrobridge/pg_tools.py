# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The Playground, a workbench: the bridge's operations as the operator,
keeping nothing. Binder tabs per direction: $tofhir, $toopenehr, the
proposed openEHR to OMOP rows and HL7 v2 to FHIR dry runs, and $validate as
the last tab. B5 (the transform pair), B6 (the OMOP row preview) and B8 (the
HL7 v2 message viewer) are drawn here."""
import re

from common import (
    MENU_SEP, badge, busy_button, button, card, e, empty_quiet, field, icon, input_, link_button, menu, menu_item, notice, page_heading,
    refusal, select, shell, table, tabbed, tabs, td, th,
)
from bridge import SEV, input_tools, json_html, json_panel, pane, pane_tools, plan_badge, planned, proposed, section, state, transform
from data import BUNDLE_JSON, COMPOSITION_FLAT, HL7_SEGMENTS, OBSERVATION_JSON, PATIENT_REF

SUB = "Runs the bridge's operations on what you enter, and keeps nothing."

LIVE_TABS = [("tofhir", "openEHR → FHIR", "playground.html"), ("toopenehr", "FHIR → openEHR", "playground-toopenehr.html"), ("validate", "Validate a resource", "validate.html")]


def json_lines(text):
    return "".join(f'<span class="fui-code-line">{json_html(l) or " "}</span>' for l in text.split("\n"))


def json_editor(text, label, invalid=False):
    inv = ' aria-invalid="true"' if invalid else ""
    return (f'<div class="fui-editor"{inv}><div class="fui-editor-stack"><pre class="fui-editor-layer" aria-hidden="true">{json_lines(text)}</pre>'
            f'<textarea class="fui-editor-input" aria-label="{e(label)}" spellcheck="false" autocomplete="off" wrap="off">{e(text)}</textarea></div></div>')


def json_out(text, label):
    return f'<div class="fui-code-panel fui-code-panel-numbered" role="region" aria-label="{e(label)}" tabindex="0"><pre>{json_lines(text)}</pre></div>'


def seg_field(label, name, opts, cur):
    segs = "".join(f'<label class="fui-segment"><input type="radio" class="fui-sr-only" name="{name}"{" checked" if v == cur else ""} />{w}</label>' for v, w in opts)
    return f'<div class="fui-field"><span class="fui-label" id="{name}-l">{label}</span><div role="radiogroup" aria-labelledby="{name}-l" class="fui-segmented">{segs}</div></div>'


def run_button(label, running=False, off=None, op=None):
    """The run button says what it does in plain words; the operation it
    calls is the quiet hint in its tooltip and in the answer's status."""
    tip = f' title="Runs {e(op)}"' if op else ""
    if running:
        b = busy_button("primary", "Converting" if label.startswith("Convert") else "Running")
    elif off:
        b = f'<button type="button" class="fui-button fui-button-primary" disabled title="{e(off)}">{icon("play", "16")}{label}</button>'
    else:
        b = button("primary", label, "play", keys="Ctrl Enter", type_="submit", attrs=tip)
    return f'<div class="fui-transform-run">{b}</div>'


def options(fields, run):
    return f'<div class="fui-transform-options">{fields}{run}</div>'


def tofhir_options(running=False, off=None):
    fmt = seg_field("Composition format", "fmt", [("canonical", "canonical"), ("flat", "FLAT")], "flat")
    tid = field("pg-tid", "Template", select("pg-tid", [("synthetic.vital_signs.v1", "synthetic.vital_signs.v1"), ("synthetic.lab_report.v1", "synthetic.lab_report.v1")], "synthetic.vital_signs.v1"))
    pat = field("pg-pat", 'Patient<span class="fui-field-key">context.patient</span>', input_("pg-pat", PATIENT_REF, extra='spellcheck="false" class="fui-input fui-mono"'))
    return options(fmt + tid + pat, run_button("Convert to FHIR", running, off, "$tofhir"))


def answer_idle(line="Convert to see the Bundle."):
    return f'<p class="fui-transform-empty">{e(line)}</p>'


def page_layout(tab, panel, overlay=None, tabs_list=None):
    h = page_heading("Playground", subtitle=SUB)
    return shell("Playground", "playground", h + f'<div class="fui-stack">{tabbed(tabs("Playground", tabs_list or LIVE_TABS, tab), panel)}</div>', overlay=overlay)


def status_line(tone, word, rest=""):
    r = f'<span class="fui-muted">{rest}</span>' if rest else ""
    return f'<span class="fui-transform-status">{state(tone, word)}{r}</span>'


def tofhir_panel(answer=None, running=False, invalid=None, status_html="", off=None, refused=None):
    left_body = json_editor(COMPOSITION_FLAT if invalid is None else "", "Composition, canonical or FLAT JSON", invalid=bool(invalid))
    if invalid:
        left_body += f'<p class="fui-help fui-error fui-transform-error">{icon("circle-alert", "14")} {invalid}</p>'
    left = pane("Composition", left_body, '<span class="fui-tag">FLAT</span>', input_tools("the composition", example=False))
    if running:
        rbody = '<div class="fui-transform-body"><span class="fui-spinner"><span class="fui-spinner-ring" aria-hidden="true"></span>Converting with $tofhir</span></div>'
    elif answer:
        rbody = json_out(answer, "The Bundle as JSON")
    elif refused:
        rbody = f'<div class="fui-transform-body">{refused}</div>'
    else:
        rbody = answer_idle()
    right = pane("Bundle", rbody, '<span class="fui-tag">FHIR R4</span>', pane_tools("the Bundle") if answer else "", status_html)
    return f'<div class="fui-panel">{tofhir_options(running, off)}{transform(left, right)}</div>'


def page_playground():
    return page_layout("tofhir", tofhir_panel())


def page_playground_invalid():
    return page_layout("tofhir", tofhir_panel(invalid="The composition is empty: enter a canonical or FLAT composition."))


def page_playground_running():
    return page_layout("tofhir", tofhir_panel(running=True))


def page_playground_answered():
    return page_layout("tofhir", tofhir_panel(answer=BUNDLE_JSON, status_html=status_line("ok", "200", "$tofhir · 1 Observation · 84 ms")))


def page_playground_refused():
    rf = refusal("The bridge refused the transform.", request="POST /fhir/$tofhir",
                 issues=[("bad", "error", "processing", "No context maps template synthetic.lab_report.v1 with the archetype openEHR-EHR-OBSERVATION.body_weight.v2."),
                         ("warn", "warning", "not-found", "The FLAT path of the body weight is in no template the context names.")])
    return page_layout("tofhir", tofhir_panel(refused=rf, status_html=status_line("bad", "422", "$tofhir · 1 error, 1 warning")))


def page_playground_off():
    rf = refusal("The operations lane is off, so the bridge answers no transform.", request="POST /fhir/$tofhir",
                 issues=[("neutral", "cause", "operations.enabled", 'is false. It is set in ferrobridge.toml; <a href="facade-settings.html">the FHIR facade\'s Settings</a> show it.')])
    return page_layout("tofhir", tofhir_panel(refused=rf, status_html=status_line("bad", "503", "$tofhir"), off="The operations lane is off on this bridge"))


def page_playground_example():
    menu_html = menu([menu_item(f"synthetic.{t}", "file-code") for t in ("vital_signs.v1", "lab_report.v1", "diagnosis.v1")], label="Example from a template", sheet=True)
    ex = f'<span class="fui-anchor"><button type="button" class="fui-button fui-button-ghost fui-button-small" aria-haspopup="menu" aria-expanded="true">{icon("file-code", "16")}Example{icon("chevron-down", "14")}</button>{menu_html}</span>'
    left = pane("Composition", json_editor(COMPOSITION_FLAT, "Composition"), '<span class="fui-tag">FLAT</span>', ex + button("ghost", "Load file", "upload", small=True) + button("ghost", "Clear", "x", small=True))
    right = pane("Bundle", answer_idle(), '<span class="fui-tag">FHIR R4</span>')
    note = '<div class="fui-panel-section">' + planned(611, "The Example menu fills the composition from a synthetic example generated from the template's web template. Until then the live Example button is disabled and says so.") + "</div>"
    return page_layout("tofhir", f'<div class="fui-panel">{note}{tofhir_options()}{transform(left, right)}</div>')


COMPOSITION_CANONICAL = """{
  "_type": "COMPOSITION",
  "name": { "value": "Synthetic vital signs" },
  "archetype_details": {
    "archetype_id": { "value": "openEHR-EHR-COMPOSITION.encounter.v1" },
    "template_id": { "value": "synthetic.vital_signs.v1" },
    "rm_version": "1.1.0"
  },
  "language": { "terminology_id": { "value": "ISO_639-1" }, "code_string": "en" },
  "territory": { "terminology_id": { "value": "ISO_3166-1" }, "code_string": "NL" },
  "composer": { "_type": "PARTY_IDENTIFIED", "name": "Demo Clinician" },
  "content": [ "… 1 OBSERVATION: body_weight, 72.4 kg …" ]
}"""


def page_playground_toopenehr():
    fmt = seg_field("Composition format (answer)", "fmt2", [("canonical", "canonical"), ("flat", "FLAT")], "canonical")
    left = pane("Bundle", json_editor(BUNDLE_JSON, "Bundle, JSON"), '<span class="fui-tag">FHIR R4</span>', input_tools("the Bundle", example=False))
    right = pane("Composition", json_out(COMPOSITION_CANONICAL, "The composition"), '<span class="fui-tag">canonical</span>', pane_tools("the composition"),
                 status_line("ok", "200", "$toopenehr · 1 entry · 102 ms"))
    return page_layout("toopenehr", f'<div class="fui-panel">{options(fmt, run_button("Convert to openEHR", op="$toopenehr"))}{transform(left, right)}</div>')


PROPOSED_TABS = [("tofhir", "openEHR → FHIR", "playground.html"), ("toopenehr", "FHIR → openEHR", "playground-toopenehr.html"),
                 ("omop", 'openEHR → OMOP rows <span class="fui-badge fui-tone-neutral">Proposed</span>', "playground-omop.html"),
                 ("hl7v2", 'HL7 v2 → FHIR <span class="fui-badge fui-tone-neutral">Proposed</span>', "playground-hl7v2.html"),
                 ("validate", "Validate a resource", "validate.html")]


# ---- proposed: openEHR to OMOP rows (B6) ----

def rowgraph():
    """B6: each row the engine would write, as column and value pairs."""
    cz = f'<span class="fui-concept-zero">{icon("triangle-alert", "12")}0, concept 0</span>'

    def tb(name, note, fields):
        rows = "".join(f'<div><dt>{c}</dt><dd>{v}</dd></div>' for c, v in fields)
        return f'<section class="fui-rowgraph-table"><h4 class="fui-rowgraph-name">{name}<span class="fui-muted">{note}</span></h4><dl class="fui-rowgraph-fields">{rows}</dl></section>'
    return (
        '<div class="fui-rowgraph">'
        + tb("MEASUREMENT", "1 row · synthetic_measurement", [("measurement_concept_id", "3025315 · Body weight"), ("value_as_number", "72.4"),
                                                               ("unit_concept_id", "9529 · kilogram"), ("measurement_date", "2026-10-20"), ("person_id", "the EHR's person")])
        + tb("OBSERVATION", "1 row · the method", [("observation_concept_id", cz), ("value_as_concept_id", cz), ("observation_date", "2026-10-20")])
        + tb("PERSON", "placeholder · create on first sight", [("person_id", "new"), ("gender_concept_id", "0"), ("year_of_birth", "0")])
        + "</div>")


def omop_panel(answered):
    note = '<div class="fui-panel-section">' + proposed("A dry run of the OMOCL engine on one composition: the rows it would write, table by table, concept 0 marked. It needs an operator API route the bridge does not have.") + "</div>"
    left = pane("Composition", json_editor(COMPOSITION_FLAT, "Composition, FLAT JSON"), '<span class="fui-tag">FLAT</span>', input_tools("the composition", example=False))
    if answered:
        right = pane("Rows", rowgraph(), '<span class="fui-tag">OMOP CDM v5.4</span>', pane_tools("the rows"), status_line("warn", "3 rows", "2 concept 0"))
    else:
        right = pane("Rows", answer_idle("Preview to see the rows."), '<span class="fui-tag">OMOP CDM v5.4</span>')
    tid = field("om-tid", "Template", select("om-tid", [("synthetic.vital_signs.v1", "synthetic.vital_signs.v1")], "synthetic.vital_signs.v1"))
    return f'<div class="fui-panel">{note}{options(tid, run_button("Preview the rows"))}{transform(left, right)}</div>'


def page_playground_omop():
    return page_layout("omop", omop_panel(False), tabs_list=PROPOSED_TABS)


def page_playground_omop_answered():
    return page_layout("omop", omop_panel(True), tabs_list=PROPOSED_TABS)


# ---- proposed: HL7 v2 to FHIR (B8) ----

def hl7_raw():
    from data import HL7_SEGMENTS as S
    lines = []
    raw = ["MSH|^~\\&|DEMO-LAB|DEMO-HOSP|FERROBRIDGE|DEMO-REGIO|20261021094211||ORU^R01^ORU_R01|MSG-2026-1021-0007|P|2.5.1",
           "PID|1||SYN-0042^^^DEMO-HOSP^MR||TEST^SYNTHETIC^^^^^L||19800101|U",
           "OBR|1|ORD-77^DEMO-LAB|FIL-77^DEMO-LAB|29463-7^Body weight^LN|||20261021091500",
           "OBX|1|NM|29463-7^Body weight^LN||72.4|kg^kilogram^UCUM|||||F"]
    for l in raw:
        seg, rest = l[:3], l[3:]
        rest = "".join(f'<span class="fui-hl7-sep">{e(ch)}</span>' if ch in "|^~&\\" else e(ch) for ch in rest)
        lines.append(f'<span class="fui-hl7-segid">{seg}</span>{rest}')
    return '<pre class="fui-hl7-raw" role="region" aria-label="The message, one segment per line" tabindex="0">' + "\n".join(lines) + "</pre>"


def hl7_segments():
    segs = ""
    for sid, name, fields in HL7_SEGMENTS:
        rows = "".join(f'<div><dt>{p}</dt><dd class="fui-hl7-name">{e(n)}</dd><dd>{e(v)}</dd></div>' for p, n, v in fields)
        segs += f'<li class="fui-hl7-seg"><p class="fui-hl7-seg-head"><span class="fui-hl7-seg-id">{sid}</span><span>{name}</span></p><dl class="fui-hl7-fields">{rows}</dl></li>'
    return f'<ul class="fui-hl7-segs" aria-label="The message as read, segment by segment">{segs}</ul>'


def hl7_panel(answered):
    note = '<div class="fui-panel-section">' + proposed("A dry run of the HL7 v2 face on one message: how the guide's maps read it, and the Bundle it maps to. Nothing is committed and no acknowledgment is sent. It needs an operator API route the bridge does not have.") + "</div>"
    left = pane("Message", hl7_raw(), '<span class="fui-tag">v2.5.1</span>', input_tools("the message", example=False))
    if answered:
        right = pane("Bundle", json_out(BUNDLE_JSON.replace('"collection"', '"transaction"'), "The Bundle as JSON"), '<span class="fui-tag">FHIR R4</span>', pane_tools("the Bundle"),
                     status_line("ok", "would answer AA", "1 Observation"))
        segs = section("The message as read", hl7_segments(), "4 segments")
    else:
        right = pane("Bundle", answer_idle("Map to see the Bundle."), '<span class="fui-tag">FHIR R4</span>')
        segs = ""
    cs = field("hl-cs", "Character set if MSH-18 is empty", select("hl-cs", [("ASCII", "ASCII, the face's default")], "ASCII"))
    return f'<div class="fui-panel">{note}{options(cs, run_button("Map the message"))}{transform(left, right)}{segs}</div>'


def page_playground_hl7v2():
    return page_layout("hl7v2", hl7_panel(False), tabs_list=PROPOSED_TABS)


def page_playground_hl7v2_answered():
    return page_layout("hl7v2", hl7_panel(True), tabs_list=PROPOSED_TABS)


# ---- $validate ----

def issue_list(items):
    out = ""
    for sev, code, where, msg in items:
        if sev == "information":
            tone, g, w = "ok", "circle-check", "Information"
        else:
            tone, g, w = SEV[sev]
        out += (f'<li class="fui-diag fui-tone-{tone}"><span class="fui-diag-sev">{icon(g, "14")}{w}</span>'
                f'<span class="fui-diag-head"><code class="fui-diag-code">{e(code)}</code><span class="fui-diag-where">{e(where)}</span></span><p class="fui-diag-msg">{e(msg)}</p></li>')
    return f'<ul class="fui-diags" aria-label="Issues, by severity">{out}</ul>'


def validate_panel(result=None, refused=None):
    rt = field("v-type", "Resource type", select("v-type", [("Observation", "Observation"), ("Condition", "Condition"), ("AllergyIntolerance", "AllergyIntolerance")], "Observation"))
    text = OBSERVATION_JSON if result != "issues" else OBSERVATION_JSON.replace('  "status": "final",\n', "")
    left = pane("Resource", json_editor(text, "Resource, JSON"), '<span class="fui-tag">FHIR R4</span>', input_tools("the resource", example=False))
    st = ""
    if result == "issues":
        body = issue_list([("error", "required", "Observation.status", "Observation.status: minimum required = 1, but only found 0."),
                           ("warning", "invariant dom-6", "Observation", "A resource should have narrative for robust management.")])
        st = status_line("bad", "1 error", "$validate · 1 warning")
    elif result == "clean":
        body = issue_list([("information", "informational", "Observation", "No issue: the resource conforms to synthetic-body-weight.")])
        st = status_line("ok", "No issue", "$validate")
    elif refused:
        body = f'<div class="fui-transform-body">{refused}</div>'
        st = status_line("bad", "400", "$validate")
    else:
        body = answer_idle("Validate to see the issues.")
    right = pane("Issues", body, '<span class="fui-tag">OperationOutcome</span>', pane_tools("the OperationOutcome") if result in ("issues", "clean") else "", st)
    return f'<div class="fui-panel">{options(rt, run_button("Validate", op="$validate"))}{transform(left, right)}</div>'


def page_validate():
    return page_layout("validate", validate_panel())


def page_validate_issues():
    return page_layout("validate", validate_panel("issues"))


def page_validate_clean():
    return page_layout("validate", validate_panel("clean"))


def page_validate_refused():
    rf = refusal("The resource's resourceType is not the type chosen.", request="POST …/Observation/$validate",
                 issues=[("bad", "error", "structure", "The body names resourceType Condition; choose Condition, or send an Observation.")])
    return page_layout("validate", validate_panel(refused=rf))
