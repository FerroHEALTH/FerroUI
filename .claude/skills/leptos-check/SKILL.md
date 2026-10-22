---
name: leptos-check
description: Reviews a diff against the Leptos rules of .claude/rules/leptos-ui.md (§14, the Leptos book's way, and §15, the islands rules). Runs the mechanical check, then walks every review-enforced rule over the changed lines and reports each finding with its file and line, the rule, why it matters and the fix. Use before opening a pull request that touches crates/ or app/, or when reviewing one.
allowed-tools: Read, Grep, Glob, Bash
argument-hint: "[base ref, default origin/main] or a PR number"
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# /leptos-check

A review pass, not a fix: it reports, and the caller decides. The rules and
their citations are `.claude/rules/leptos-ui.md` §14 and §15; this skill
only says where to look.

## Steps

1. **Take the diff.** With a PR number, run `gh pr diff <n>`. Otherwise run
   `git diff <base>...HEAD -- crates app`, where the base defaults to
   `origin/main` after a `git fetch`. List the changed `.rs` files and the
   changed line ranges. Read each changed file whole, because a rule can
   hang on code outside the hunk.
2. **Run the mechanical check.** Run `bash scripts/checks/leptos-rules.sh`.
   Report every finding as it prints (rules L1 to L10). An allow line added
   in the diff needs a reason and an open issue.
3. **Walk the review-enforced rules** over the changed lines. Each row says
   what to search for, then what to ask.

   | Search | Ask (rule) |
   |---|---|
   | `Effect::new`, `Effect::watch` | Does the effect write a signal it could derive? It may only sync with the DOM or a browser API (§14.2). |
   | `.get()` inside closures over `Vec`, `String`, maps | Would `.read()` or `.with()` avoid the clone (§14.2)? Is a read guard held across a write? |
   | `<For`, `collect_view`, `move \|\| rows(` | Is a list that changes keyed by data, not an index, and not rebuilt whole (§14.3)? |
   | `selected=`, `value=` on inputs and selects | Is a live value set through `prop:value` on the select or input (§14.4)? |
   | `<textarea` | Does the server HTML carry the escaped text (§5, #205)? |
   | `inner_html` | Is the markup escaped by the library and tested (§14.3)? |
   | `Memo::new` | Is the computation expensive enough to pay for a memo (§14.2)? |
   | `RwSignal::new` inside a `For` row or a branch, stored in a parent collection | Will it be read after disposal or leak (§14.2)? Should it be an `ArcRwSignal`? |
   | `#[island]` | Is it the smallest interactive part, with static parts as children (§15.1)? Are its props serializable, free of callbacks, `usize` and clinical records (§15.2)? Does it have a no-WebAssembly floor (§15.8)? |
   | `use_context`, `expect_context` inside an island or a widget | Was it provided by an enclosing island, never the root (§15.4)? |
   | `<Show`, `if`, `match` inside an island over its `children` | Are the children rendered and hidden instead (§15.3)? |
   | `tr(`, `i18n::` inside an island body | Should the words be `String` props resolved on the server (§15.4)? |
   | `Resource::`, `Suspense`, `Suspend::new` | Is the data loaded in a server component (§15.5)? Is no resource created inside a `Suspend`, and no signal set from a fetcher (§14.5)? Does primary content use a blocking resource or `SsrMode::Async` (§14.5)? |
   | `ActionForm`, `ServerAction`, `#[server]` | Is the form in a server component unless it needs live `pending()`/`value()` (§15.7)? Are fields named after the arguments? Is no server function in the library (§0)? |
   | `on:click` that navigates, `use_navigate`, `leptos_router` | Is navigation a link or a form (§14.7, §15.6)? |
   | `#[cfg(target_arch`, `cfg!(` | Does the browser code run in a handler or an effect, and does the view tree stay the same on both halves (§14.8)? |
   | `<table`, `<p>` holding blocks | Is the HTML valid, with an explicit `<tbody>` (§4, §14.3)? |
   | new `web-sys` features, new dependencies, generics with large bodies | Is the bundle cost justified and measured (§12, §14.11)? |
   | `get_element_by_id`, `query_selector` | Would a `NodeRef` or a directive do (§14.10, #214)? |

4. **Check the tests that should move with it.**
   - A new interactive state belongs in its area's file under
     `tests/it/states/`.
   - A new behaviour needs a journey, and an assertion in the
     no-WebAssembly journey.
   - A new class needs its stylesheet rule, and a new role its contrast
     pair (§7, §10, §11).
5. **Report.** Give one line per finding: `path:line`, the rule (§ and
   letter or check id), why it matters in one sentence, and the fix. Group
   the findings into "check findings" and "review findings". End with
   "clean" when there are none. Do not edit files; the caller applies the
   fixes.
