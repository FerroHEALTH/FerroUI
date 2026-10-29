---
name: console-check
description: Reviews a diff against the console UI rules of .claude/rules/console-ui.md (server-rendered components on Askama templates, plain script modules, the no-script floor). Runs the mechanical checks, then walks every review-enforced rule over the changed lines and reports each finding with its file and line, the rule, why it matters and the fix. Use before opening a pull request that touches crates/ or app/, or when reviewing one.
allowed-tools: Read, Grep, Glob, Bash
argument-hint: "[base ref, default origin/main] or a PR number"
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# /console-check

A review pass, not a fix: it reports, and the caller decides. The rules and
their citations are `.claude/rules/console-ui.md`; this skill only says
where to look.

## Steps

1. **Take the diff.** With a PR number, run `gh pr diff <n>`. Otherwise run
   `git diff <base>...HEAD -- crates app`, where the base defaults to
   `origin/main` after a `git fetch`. List the changed `.rs`, `.html` and
   `.js` files and the changed line ranges. Read each changed file whole,
   because a rule can hang on code outside the hunk.
2. **Run the mechanical checks** and report every finding as it prints:
   - `bash scripts/checks/markup-rules.sh crates/ferrohealth-ui/src crates/ferrohealth-ui/templates app/ferrohealth-ui-gallery/src`
   - `bash scripts/checks/script-budget.sh`
   - `bash scripts/checks/stray-literals.sh crates/ferrohealth-ui/src crates/ferrohealth-ui/templates app/ferrohealth-ui-gallery/src`
3. **Walk the review-enforced rules** over the changed lines. Each row says
   what to search for, then what to ask.

   | Search | Ask (rule) |
   |---|---|
   | `Html::raw(` | Is every piece of text inside escaped with `Html::text` or the template (§2)? |
   | `format!(` building markup | Would a template read better, and is each attribute value escaped (§2)? |
   | `data-fui=` | Does the module only enhance what the server drew, and does the page work without it (§1, §3)? |
   | `hidden`, `disabled` on a control a module enables | Does the module show or enable it when it runs (§3)? |
   | `replaceChildren`, `innerHTML` in a module | Does a control the reader holds stay the same element (§3)? |
   | `sessionStorage`, `localStorage` | Is the key the page and the element's id, in the session, with no clinical content (§3)? |
   | `fetch(` | Is it one of the two documented wires (§0)? |
   | a new handler or post in the gallery | Does a post render its page or redirect with a flash toast (§1)? |
   | `<textarea` | Does the server HTML carry the escaped text (§5)? |
   | `<table`, `<p>` holding blocks | Is the HTML valid, with an explicit `<tbody>` (§4)? |
   | a new word in a template or a module | Is it a catalog message resolved on the server (§13)? |

4. **Check the tests that should move with it.**
   - A new interactive state belongs in its area's file under
     `tests/it/states/`.
   - A new behaviour needs a journey, and an assertion in the no-script
     journey (`tests/it/no_script.rs`).
   - A new class needs its stylesheet rule, and a new role its contrast
     pair (§7, §10, §11).
5. **Report.** Give one line per finding: `path:line`, the rule (§ or
   check), why it matters in one sentence, and the fix. Group the findings
   into "check findings" and "review findings". End with "clean" when
   there are none. Do not edit files; the caller applies the fixes.
