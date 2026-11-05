// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The script modules' behaviour that no Rust copy holds, held to pinned
//! answers (#626). Each test runs a script in the gallery's browser that
//! drives one module over a fixture or a gallery page and reads back what
//! the module did, and fails naming every row that differs from the
//! answer pinned here. Each test's name carries its module's file name, so
//! a change to the module runs it (`scripts/checks/affected.sh`).

use std::error::Error;

use serde_json::Value;

use crate::browser::with_browser;
use crate::journeys_enabled;
use crate::stack::Gallery;

/// The script each test's body runs inside: `mod(name)` imports the module
/// `name` as the loader does, and the body's answer comes back as JSON.
const HARNESS: &str = r#"const done = arguments[arguments.length - 1];
(async () => {
  const loader = document.querySelector("script[src*='fui.js']");
  const mod = (name) => import(new URL(name + '.js', loader.src).href);
  BODY
})().then((out) => done({ out }), (error) => done({ error: String(error && error.stack || error) }));"#;

/// Runs `body` at the gallery page `page` and fails naming every row of its
/// answer that differs from `expected`, a JSON array.
async fn pinned(page: &str, body: &str, expected: &str) -> Result<(), Box<dyn Error>> {
    if !journeys_enabled() {
        return Ok(());
    }
    let expected: Vec<Value> = serde_json::from_str(expected)?;
    let script = HARNESS.replace("BODY", body);
    let gallery = Gallery::start().await?;
    with_browser(async |browser| {
        browser.goto(&gallery.url(page)).await?;
        let answer = browser.driver().execute_async(&script, Vec::new()).await?;
        let answer = answer.json();
        if let Some(error) = answer.get("error") {
            return Err(format!("the script at {page} threw: {error}").into());
        }
        let Some(Value::Array(found)) = answer.get("out") else {
            return Err(format!("the script at {page} answered {answer}").into());
        };
        let differ: Vec<String> = expected
            .iter()
            .zip(found)
            .filter(|(pinned, found)| pinned != found)
            .map(|(pinned, found)| format!("  pinned {pinned}\n  found  {found}"))
            .collect();
        assert!(
            differ.is_empty() && expected.len() == found.len(),
            "{} of {} rows at {page} differ from the pinned answer ({} found):\n{}",
            differ.len(),
            expected.len(),
            found.len(),
            differ.join("\n")
        );
        browser.console_clean(page).await
    })
    .await
}

/// The combobox's keys over a fixture of three options and of none, local and
/// remote: each key's `defaultPrevented`, `aria-expanded`,
/// `aria-activedescendant` and the posted value.
const COMBOBOX: &str = r#"const m = await mod("combobox");
  const run = (keys, remote, listed) => {
    const root = document.createElement("div");
    if (remote) root.dataset.fuiRemote = "q";
    const options = listed ? '<option value="a">Alpha</option><option value="b" selected>Beta</option><option value="c">Gamma</option>' : "";
    root.innerHTML = '<label for="pin-pick">Pick</label><select id="pin-pick" name="pick"><option value=""></option>' + options + "</select>";
    document.body.append(root);
    m.enhance(root);
    const input = root.querySelector("input[role='combobox']");
    const hidden = root.querySelector("input[type='hidden']");
    input.focus();
    const seen = keys.map((key) => {
      let prevented = null;
      if (key.startsWith("type:")) {
        input.value = key.slice(5);
        input.dispatchEvent(new Event("input", { bubbles: true }));
      } else {
        const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true });
        input.dispatchEvent(event);
        prevented = event.defaultPrevented;
      }
      return [key, prevented, input.getAttribute("aria-expanded"), input.getAttribute("aria-activedescendant"), hidden.value];
    });
    root.remove();
    return seen;
  };
  const out = [];
  for (const remote of [false, true]) {
    out.push(...run(["Home", "Enter", "ArrowUp", "ArrowUp", "ArrowUp", "ArrowDown", "ArrowDown", "ArrowDown", "Home", "End", "Tab", "Escape", "Escape", "ArrowDown", "Enter"], remote, true).map((row) => [remote ? "remote" : "local", ...row]));
    out.push(...run(["type:zzz", "ArrowDown", "Home", "End", "Enter", "Escape", "type:a", "End", "Enter"], remote, true).map((row) => [remote ? "remote" : "local", ...row]));
    out.push(...run(["type:g", "Enter", "ArrowDown", "Escape", "End"], remote, true).map((row) => [remote ? "remote" : "local", ...row]));
    out.push(...run(["ArrowDown", "ArrowUp", "Home", "End", "Enter", "Escape"], remote, false).map((row) => [remote ? "remote" : "local", ...row]));
  }
  return out;"#;

#[tokio::test(flavor = "multi_thread")]
async fn the_combobox_keys_move_open_and_choose_as_pinned() -> Result<(), Box<dyn Error>> {
    pinned(
        "/ferropix/forms",
        COMBOBOX,
        r#"[
["local", "Home", false, "false", null, "b"],
["local", "Enter", false, "false", null, "b"],
["local", "ArrowUp", true, "true", "pin-pick-option-1", "b"],
["local", "ArrowUp", true, "true", "pin-pick-option-0", "b"],
["local", "ArrowUp", true, "true", "pin-pick-option-0", "b"],
["local", "ArrowDown", true, "true", "pin-pick-option-1", "b"],
["local", "ArrowDown", true, "true", "pin-pick-option-2", "b"],
["local", "ArrowDown", true, "true", "pin-pick-option-2", "b"],
["local", "Home", true, "true", "pin-pick-option-0", "b"],
["local", "End", true, "true", "pin-pick-option-2", "b"],
["local", "Tab", false, "true", "pin-pick-option-2", "b"],
["local", "Escape", true, "false", null, "b"],
["local", "Escape", false, "false", null, "b"],
["local", "ArrowDown", true, "true", "pin-pick-option-1", "b"],
["local", "Enter", true, "false", null, "b"],
["local", "type:zzz", null, "true", null, "b"],
["local", "ArrowDown", true, "true", null, "b"],
["local", "Home", true, "true", null, "b"],
["local", "End", true, "true", null, "b"],
["local", "Enter", true, "true", null, "b"],
["local", "Escape", true, "false", null, "b"],
["local", "type:a", null, "true", "pin-pick-option-0", "b"],
["local", "End", true, "true", "pin-pick-option-2", "b"],
["local", "Enter", true, "false", null, "c"],
["local", "type:g", null, "true", "pin-pick-option-2", "b"],
["local", "Enter", true, "false", null, "c"],
["local", "ArrowDown", true, "true", "pin-pick-option-2", "c"],
["local", "Escape", true, "false", null, "c"],
["local", "End", false, "false", null, "c"],
["local", "ArrowDown", true, "true", null, ""],
["local", "ArrowUp", true, "true", null, ""],
["local", "Home", true, "true", null, ""],
["local", "End", true, "true", null, ""],
["local", "Enter", true, "true", null, ""],
["local", "Escape", true, "false", null, ""],
["remote", "Home", false, "false", null, "b"],
["remote", "Enter", false, "false", null, "b"],
["remote", "ArrowUp", true, "true", "pin-pick-option-1", "b"],
["remote", "ArrowUp", true, "true", "pin-pick-option-0", "b"],
["remote", "ArrowUp", true, "true", "pin-pick-option-0", "b"],
["remote", "ArrowDown", true, "true", "pin-pick-option-1", "b"],
["remote", "ArrowDown", true, "true", "pin-pick-option-2", "b"],
["remote", "ArrowDown", true, "true", "pin-pick-option-2", "b"],
["remote", "Home", true, "true", "pin-pick-option-0", "b"],
["remote", "End", true, "true", "pin-pick-option-2", "b"],
["remote", "Tab", false, "true", "pin-pick-option-2", "b"],
["remote", "Escape", true, "false", null, "b"],
["remote", "Escape", false, "false", null, "b"],
["remote", "ArrowDown", true, "true", "pin-pick-option-1", "b"],
["remote", "Enter", true, "false", null, "b"],
["remote", "type:zzz", null, "true", "pin-pick-option-0", "b"],
["remote", "ArrowDown", true, "true", "pin-pick-option-1", "b"],
["remote", "Home", true, "true", "pin-pick-option-0", "b"],
["remote", "End", true, "true", "pin-pick-option-2", "b"],
["remote", "Enter", true, "false", null, "c"],
["remote", "Escape", false, "false", null, "c"],
["remote", "type:a", null, "true", "pin-pick-option-0", "c"],
["remote", "End", true, "true", "pin-pick-option-2", "c"],
["remote", "Enter", true, "false", null, "c"],
["remote", "type:g", null, "true", "pin-pick-option-0", "b"],
["remote", "Enter", true, "false", null, "a"],
["remote", "ArrowDown", true, "true", "pin-pick-option-0", "a"],
["remote", "Escape", true, "false", null, "a"],
["remote", "End", false, "false", null, "a"],
["remote", "ArrowDown", true, "true", null, ""],
["remote", "ArrowUp", true, "true", null, ""],
["remote", "Home", true, "true", null, ""],
["remote", "End", true, "true", null, ""],
["remote", "Enter", false, "true", null, ""],
["remote", "Escape", true, "false", null, ""]
]"#,
    )
    .await
}

/// The gallery's date range as days are chosen and hovered: the classes of
/// every day of the month that carries more than `fui-day`.
const DATE_RANGE: &str = r#"const button = document.querySelector("form#date-range button[aria-haspopup='dialog']");
  for (let n = 0; n < 600 && button.hidden; n++) await new Promise((r) => requestAnimationFrame(r));
  button.click();
  const picker = document.getElementById("audit-range-picker");
  const day = (d) => picker.querySelector("button[data-day='" + d + "'], button[data-out-day='" + d + "']");
  const marked = () => [...picker.querySelectorAll("button.fui-day")].filter((b) => !["fui-day", "fui-day fui-day-out"].includes(b.className)).map((b) => (b.dataset.day || b.dataset.outDay) + " " + b.className);
  const seen = [["opened", marked()]];
  const steps = [["click", "2026-10-12"], ["enter", "2026-10-15"], ["enter", "2026-10-12"], ["enter", "2026-10-08"], ["leave", null], ["click", "2026-10-05"], ["enter", "2026-10-01"], ["click", "2026-10-14"], ["enter", "2026-10-17"], ["click", "2026-10-20"], ["click", "2026-10-20"], ["click", "2026-10-20"]];
  for (const [what, d] of steps) {
    if (what === "click") day(d).click();
    else if (what === "enter") day(d).dispatchEvent(new MouseEvent("mouseenter"));
    else picker.dispatchEvent(new MouseEvent("mouseleave"));
    seen.push([what + " " + d, marked()]);
  }
  return seen;"#;

#[tokio::test(flavor = "multi_thread")]
async fn the_date_range_day_classes_follow_the_choice_and_the_hover_as_pinned()
-> Result<(), Box<dyn Error>> {
    pinned(
        "/ferropix/forms",
        DATE_RANGE,
        r#"[
["opened", ["2026-10-15 fui-day fui-day-start", "2026-10-16 fui-day fui-day-between", "2026-10-17 fui-day fui-day-between", "2026-10-18 fui-day fui-day-between", "2026-10-19 fui-day fui-day-between", "2026-10-20 fui-day fui-day-between", "2026-10-21 fui-day fui-day-end"]],
["click 2026-10-12", []],
["enter 2026-10-15", ["2026-10-12 fui-day fui-day-start", "2026-10-13 fui-day fui-day-between fui-day-tentative", "2026-10-14 fui-day fui-day-between fui-day-tentative", "2026-10-15 fui-day fui-day-end fui-day-tentative-end"]],
["enter 2026-10-12", []],
["enter 2026-10-08", []],
["leave null", []],
["click 2026-10-05", ["2026-10-05 fui-day fui-day-start", "2026-10-06 fui-day fui-day-between", "2026-10-07 fui-day fui-day-between", "2026-10-08 fui-day fui-day-between", "2026-10-09 fui-day fui-day-between", "2026-10-10 fui-day fui-day-between", "2026-10-11 fui-day fui-day-between", "2026-10-12 fui-day fui-day-end"]],
["enter 2026-10-01", ["2026-10-05 fui-day fui-day-start", "2026-10-06 fui-day fui-day-between", "2026-10-07 fui-day fui-day-between", "2026-10-08 fui-day fui-day-between", "2026-10-09 fui-day fui-day-between", "2026-10-10 fui-day fui-day-between", "2026-10-11 fui-day fui-day-between", "2026-10-12 fui-day fui-day-end"]],
["click 2026-10-14", []],
["enter 2026-10-17", ["2026-10-14 fui-day fui-day-start", "2026-10-15 fui-day fui-day-between fui-day-tentative", "2026-10-16 fui-day fui-day-between fui-day-tentative", "2026-10-17 fui-day fui-day-end fui-day-tentative-end"]],
["click 2026-10-20", ["2026-10-14 fui-day fui-day-start", "2026-10-15 fui-day fui-day-between", "2026-10-16 fui-day fui-day-between", "2026-10-17 fui-day fui-day-between", "2026-10-18 fui-day fui-day-between", "2026-10-19 fui-day fui-day-between", "2026-10-20 fui-day fui-day-end"]],
["click 2026-10-20", []],
["click 2026-10-20", []]
]"#,
    )
    .await
}

/// The gallery's mapping editor as the caret moves on and off its marked
/// lines: the current line, the current diagnostic rows, the status and the
/// position.
const EDITOR: &str = r#"const area = document.getElementById("gallery-mapping");
  const root = area.closest(".fui-editor");
  for (let n = 0; n < 600 && !root.querySelector(".fui-editor-layer .fui-code-line"); n++) await new Promise((r) => requestAnimationFrame(r));
  const panel = area.closest(".fui-editor-split") || document;
  const lineStart = (line) => area.value.split("\n").slice(0, line - 1).reduce((n, l) => n + l.length + 1, 0);
  const seen = [];
  for (const [line, column] of [[16, 3], [16, 1], [6, 2], [1, 1], [16, 12], [3, 1]]) {
    const at = lineStart(line) + column - 1;
    area.setSelectionRange(at, at);
    area.dispatchEvent(new Event("keyup"));
    seen.push([
      line, column,
      [...root.querySelectorAll(".fui-editor-layer > *.is-current")].map((e) => e.id + " " + e.className),
      [...panel.querySelectorAll("li[data-line]")].map((r) => r.dataset.line + ":" + r.classList.contains("is-current") + ":" + r.getAttribute("aria-current")),
      panel.querySelector(".fui-editor-status output")?.textContent ?? null,
      panel.querySelector("[data-fui-position]")?.textContent ?? null,
    ]);
  }
  return seen;"#;

#[tokio::test(flavor = "multi_thread")]
async fn the_editor_follows_the_caret_to_its_marks_and_position_as_pinned()
-> Result<(), Box<dyn Error>> {
    pinned(
        "/ferrobridge/forms",
        EDITOR,
        r#"[
[16, 3, ["gallery-mapping-L16 fui-code-line fui-mark-error is-current"], ["16:true:true", "6:false:null"], "Line 16 has an error.", "Ln 16, Col 3"],
[16, 1, ["gallery-mapping-L16 fui-code-line fui-mark-error is-current"], ["16:true:true", "6:false:null"], "Line 16 has an error.", "Ln 16, Col 1"],
[6, 2, ["gallery-mapping-L6 fui-code-line fui-mark-warning is-current"], ["16:false:null", "6:true:true"], "Line 6 has a warning.", "Ln 6, Col 2"],
[1, 1, [], ["16:false:null", "6:false:null"], "", "Ln 1, Col 1"],
[16, 12, ["gallery-mapping-L16 fui-code-line fui-mark-error is-current"], ["16:true:true", "6:false:null"], "Line 16 has an error.", "Ln 16, Col 12"],
[3, 1, [], ["16:false:null", "6:false:null"], "", "Ln 3, Col 1"]
]"#,
    )
    .await
}

/// The announced keys over a fixture: a single key outside a field, a chord
/// from a field, a repeat, a key that focuses a field, the legend's `?` and
/// a go-to pair, with what each prevented, focused, pressed or followed.
const KEYS: &str = r##"const m = await mod("keys");
  m.enhance();
  const root = document.createElement("div");
  root.innerHTML = '<form id="pin-form"><input id="pin-field" type="text" aria-keyshortcuts="F6"><button type="button" id="pin-press" aria-keyshortcuts="F7 Control+F8">Press</button><input id="pin-other" type="text"></form><input id="pin-outer" type="text"><div data-fui-go="f9"><a id="pin-go" href="#pin-gone" data-fui-go-key="f10">Go</a></div>';
  document.body.append(root);
  const said = [];
  root.querySelector("#pin-press").addEventListener("click", () => said.push("pressed"));
  root.querySelector("#pin-go").addEventListener("click", (event) => { event.preventDefault(); said.push("followed"); });
  const cases = [
    ["body", "F7", {}], ["body", "F7", { repeat: true }], ["pin-other", "F7", {}], ["pin-other", "F8", { ctrlKey: true }],
    ["pin-outer", "F8", { metaKey: true }], ["body", "F8", {}], ["body", "F6", {}], ["pin-outer", "F6", {}],
    ["body", "?", {}], ["body", "F9", {}], ["body", "F10", {}], ["body", "F10", {}], ["body", "F9", {}], ["pin-other", "F10", {}],
    ["body", "F9", {}], ["body", "F7", {}], ["body", "F11", {}], ["body", "F7", { isComposing: true }],
  ];
  const out = [];
  for (const [at, key, extra] of cases) {
    const target = at === "body" ? document.body : document.getElementById(at);
    if (at === "body") document.activeElement?.blur(); else target.focus();
    said.length = 0;
    const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...extra });
    target.dispatchEvent(event);
    out.push([at, key, Object.keys(extra).join("+"), event.defaultPrevented, document.activeElement?.id || "", said.join(",")]);
  }
  root.remove();
  return out;"##;

#[tokio::test(flavor = "multi_thread")]
async fn the_keys_press_focus_and_follow_as_pinned() -> Result<(), Box<dyn Error>> {
    pinned(
        "/ferropix/forms",
        KEYS,
        r#"[
["body", "F7", "", true, "", "pressed"],
["body", "F7", "repeat", true, "", ""],
["pin-other", "F7", "", false, "pin-other", ""],
["pin-other", "F8", "ctrlKey", true, "pin-other", "pressed"],
["pin-outer", "F8", "metaKey", true, "pin-outer", "pressed"],
["body", "F8", "", false, "", ""],
["body", "F6", "", true, "pin-field", ""],
["pin-outer", "F6", "", false, "pin-outer", ""],
["body", "?", "", false, "", ""],
["body", "F9", "", true, "", ""],
["body", "F10", "", true, "", "followed"],
["body", "F10", "", false, "", ""],
["body", "F9", "", true, "", ""],
["pin-other", "F10", "", false, "pin-other", ""],
["body", "F9", "", true, "", ""],
["body", "F7", "", true, "", "pressed"],
["body", "F11", "", false, "", ""],
["body", "F7", "isComposing", false, "", ""]
]"#,
    )
    .await
}

/// The top bar's typeahead over a stubbed `fetch`, by GET to a route with and
/// without a query and by POST: what it asked and what it drew for hits, no
/// hits, a refusal, an unreadable answer, an answer with no list of groups,
/// a `null` answer, a group with no list of hits and an unreachable route,
/// and whether the form's text then holds the word "undefined".
const SEARCH: &str = r#"const m = await mod("search");
  const until = async (ok) => { for (let n = 0; n < 600 && !ok(); n++) await new Promise((r) => requestAnimationFrame(r)); return ok(); };
  const calls = [];
  const replies = {
    hits: () => new Response(JSON.stringify({ query: "an", groups: [{ kind: "Patients", hits: [{ href: "/p/1", name: "Ana", line: "one" }, { href: "/p/2", name: "Jan", line: "" }] }], more: { href: "/more", words: "More" } }), { status: 200 }),
    none: () => new Response(JSON.stringify({ query: "an", groups: [] }), { status: 200 }),
    refused: () => new Response("no", { status: 503 }),
    unreadable: () => new Response("not json", { status: 200 }),
    grouped: () => new Response(JSON.stringify({ groups: "no list" }), { status: 200 }),
    nothing: () => new Response("null", { status: 200 }),
    hitless: () => new Response(JSON.stringify({ query: "an", groups: [{ kind: "Patients" }] }), { status: 200 }),
    unreachable: () => { throw new TypeError("refused"); },
  };
  let reply = null;
  window.fetch = async (url, init) => { calls.push([String(url), init.method, JSON.stringify(init.headers), init.body ?? null, init.credentials]); return replies[reply](); };
  const out = [];
  for (const [method, route] of [["get", "/pin/suggest"], ["get", "/pin/suggest?scope=all"], ["post", "/pin/suggest?scope=all"]]) {
    const form = document.querySelector("form[data-fui='search']").cloneNode(true);
    form.method = method;
    form.dataset.fuiSuggest = route;
    for (const node of [form, ...form.querySelectorAll("[id]")]) if (node.id) node.id = "pin-" + node.id;
    form.querySelector("[role=listbox]").replaceChildren();
    document.body.append(form);
    m.enhance(form);
    const field = form.querySelector("input[type='search']");
    const said = form.querySelector(":scope > output");
    for (const name of Object.keys(replies)) {
      reply = name;
      calls.length = 0;
      said.textContent = "";
      field.value = name === "hits" ? "  an " : "an";
      field.dispatchEvent(new Event("input", { bubbles: true }));
      await until(() => calls.length > 0 && said.textContent !== "" && said.textContent !== "Searching…");
      out.push([method, name, calls.slice(), said.textContent, form.querySelectorAll("[role=option]").length, form.querySelector(".fui-search-results").hidden, field.getAttribute("aria-expanded"), form.textContent.includes("undefined")]);
    }
    form.remove();
  }
  return out;"#;

#[tokio::test(flavor = "multi_thread")]
async fn the_search_asks_its_route_and_draws_each_reply_as_pinned() -> Result<(), Box<dyn Error>> {
    pinned(
        "/ferropix/forms",
        SEARCH,
        r#"[
["get", "hits", [["/pin/suggest?q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "Results for “an”: 2", 4, false, "true", false],
["get", "none", [["/pin/suggest?q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "No matches for “an”", 0, false, "true", false],
["get", "refused", [["/pin/suggest?q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search did not answer (503).", 0, false, "true", false],
["get", "unreadable", [["/pin/suggest?q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["get", "grouped", [["/pin/suggest?q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["get", "nothing", [["/pin/suggest?q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["get", "hitless", [["/pin/suggest?q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["get", "unreachable", [["/pin/suggest?q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search could not be reached.", 0, false, "true", false],
["get", "hits", [["/pin/suggest?scope=all&q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "Results for “an”: 2", 4, false, "true", false],
["get", "none", [["/pin/suggest?scope=all&q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "No matches for “an”", 0, false, "true", false],
["get", "refused", [["/pin/suggest?scope=all&q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search did not answer (503).", 0, false, "true", false],
["get", "unreadable", [["/pin/suggest?scope=all&q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["get", "grouped", [["/pin/suggest?scope=all&q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["get", "nothing", [["/pin/suggest?scope=all&q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["get", "hitless", [["/pin/suggest?scope=all&q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["get", "unreachable", [["/pin/suggest?scope=all&q=an", "GET", "{\"Accept\":\"application/json\"}", null, "same-origin"]], "The search could not be reached.", 0, false, "true", false],
["post", "hits", [["/pin/suggest?scope=all", "POST", "{\"Accept\":\"application/json\",\"Content-Type\":\"application/x-www-form-urlencoded\"}", "q=an", "same-origin"]], "Results for “an”: 2", 4, false, "true", false],
["post", "none", [["/pin/suggest?scope=all", "POST", "{\"Accept\":\"application/json\",\"Content-Type\":\"application/x-www-form-urlencoded\"}", "q=an", "same-origin"]], "No matches for “an”", 0, false, "true", false],
["post", "refused", [["/pin/suggest?scope=all", "POST", "{\"Accept\":\"application/json\",\"Content-Type\":\"application/x-www-form-urlencoded\"}", "q=an", "same-origin"]], "The search did not answer (503).", 0, false, "true", false],
["post", "unreadable", [["/pin/suggest?scope=all", "POST", "{\"Accept\":\"application/json\",\"Content-Type\":\"application/x-www-form-urlencoded\"}", "q=an", "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["post", "grouped", [["/pin/suggest?scope=all", "POST", "{\"Accept\":\"application/json\",\"Content-Type\":\"application/x-www-form-urlencoded\"}", "q=an", "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["post", "nothing", [["/pin/suggest?scope=all", "POST", "{\"Accept\":\"application/json\",\"Content-Type\":\"application/x-www-form-urlencoded\"}", "q=an", "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["post", "hitless", [["/pin/suggest?scope=all", "POST", "{\"Accept\":\"application/json\",\"Content-Type\":\"application/x-www-form-urlencoded\"}", "q=an", "same-origin"]], "The search answered in a form this page cannot read.", 0, false, "true", false],
["post", "unreachable", [["/pin/suggest?scope=all", "POST", "{\"Accept\":\"application/json\",\"Content-Type\":\"application/x-www-form-urlencoded\"}", "q=an", "same-origin"]], "The search could not be reached.", 0, false, "true", false]
]"#,
    )
    .await
}
