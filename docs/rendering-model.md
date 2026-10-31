<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# The rendering model and what would reopen it

Every FerroHEALTH console is server-rendered HTML from Askama templates with
a few plain ES modules: no WebAssembly, no hydration and no JavaScript
library (owner, 2026-10-29, FerroUI #510, shipped in 0.6.0). A review on
2026-10-31 weighed the move against the alternatives and confirmed it
(FerroUI #545). This page records the decision, what was weighed, what the
move measured, and what would reopen it.

No specification governs the rendering model: our own design.

## The decision

The consoles are forms, tables, trees, editors and work-lists. Each of them
must work with the scripts refused, under the policy every console sends:

```text
default-src 'self'; script-src 'self'; style-src 'self'
```

So the server draws every page whole, a link or a form loads one complete
document, and a module under `scripts/` enhances what the server drew. The
interaction contract is in
[`crates/ferrohealth-ui/CLAUDE.md`](../crates/ferrohealth-ui/CLAUDE.md) and
the rules are in [`.claude/rules/console-ui.md`](../.claude/rules/console-ui.md).

## The candidates weighed

| Candidate | What it costs under the policy | WebAssembly and payload | Verdict |
|---|---|---|---|
| Askama with plain ES modules (chosen) | Nothing: no nonce, no eval, no inline style | None. A page loads only the modules it marks | Kept |
| Leptos 0.8, full hydration (before 0.6) | `'wasm-unsafe-eval'`, and a nonce as our stack wired it | 528 557 bytes of brotli WebAssembly for the bridge console, measured at FerroUI 0.5.0, before any click works | Left |
| Leptos islands | `'wasm-unsafe-eval'` on every page that hosts an island | About 24 kB uncompressed for a page with no island, 166 kB with one counter (Leptos book) | Rejected as a layer |
| Dioxus 0.7 fullstack | `'wasm-unsafe-eval'` | A WebAssembly bundle, split by WASM-Split | Rejected |
| Yew 0.23 | `'wasm-unsafe-eval'` | A WebAssembly bundle, client-rendered by default | Rejected |
| Sycamore 0.9 | `'wasm-unsafe-eval'` | A WebAssembly bundle | Rejected |
| Datastar 1.0 | `'unsafe-eval'`: expressions compile through the `Function` constructor | None | Rejected |
| htmx 2.x or 4.0 | `allowEval = false`, which loses `hx-on` and trigger filters, and `includeIndicatorStyles = false`, or htmx injects a `<style>` | None, but a third-party library of about 50 kB, more than the 16 KiB a module may hold | Rejected |

A WebAssembly module compiles under a CSP only with `'wasm-unsafe-eval'` in
`script-src` (WebAssembly CSP proposal), so every framework in the table
that runs Rust in the browser widens the policy on every page it draws.
htmx is also a JavaScript library, which FerroUI's mandate refuses
(`console-ui.md` §0), and it moved to a breaking 4.0 line on 2026-08-28, so
adopting it starts a migration on the first day.

Sources:

- Leptos book, Islands: <https://book.leptos.dev/islands.html>
- Dioxus 0.7 release notes: <https://dioxuslabs.com/blog/release-070>
- Yew 0.22 release notes: <https://yew.rs/blog/2025/11/29/release-0-22>
- Sycamore documentation: <https://sycamore.dev/>
- Datastar security reference: <https://data-star.dev/reference/security>
- htmx documentation, configuration (`allowEval`,
  `includeIndicatorStyles`): <https://htmx.org/docs/>
- htmx 4.0 announcement:
  <https://four.htmx.org/announcements/2026-08-28-htmx-4.0.0-is-released>
- WebAssembly Content Security Policy proposal:
  <https://github.com/WebAssembly/content-security-policy/blob/main/proposals/CSP.md>

## What the move measured

- **Code.** Five console pull requests (FerroEHR #4440, FerroBRIDGE #1357,
  FerroFED #1461, FerroTERM #949, FerroPIX #391) added 29 546 lines and
  removed 46 556 over 855 files: 17 010 lines fewer.
- **Payload.** The bridge console no longer ships 528 557 bytes of brotli
  WebAssembly before a click works.
- **Build and supply chain.** One `cargo build`. Each console dropped
  cargo-leptos, wasm-bindgen, wasm-opt and the `wasm32` lint lane, the site
  bundle with its budget and scripts, and the release `site` job.
  FerroBRIDGE's `Cargo.lock` went from 652 to 578 packages.
- **Defects gone by construction.** The hydration panics and the journeys
  racing WebAssembly start-up: FerroBRIDGE#1331, FerroFED#1458,
  FerroEHR#4137 and FerroEHR#4161.
- **Policy.** The consoles moved from `script-src 'self'
  'wasm-unsafe-eval' 'nonce-…'` to the policy above, with no nonce.
- **State kept.** A tab or a tree node toggles what the server drew, so
  expansion, scroll and selection survive. The gallery's `navigation.rs`
  journeys hold it: no document load on a tab switch, zero layout shift,
  tree folds that survive a reload, and kept scroll.

The move also cost something: two consoles lost their keyboard shortcuts
and typed preferences for a day until 0.6.2 shipped them (#526, #527,
#530), and some logic lives twice, in the Rust highlighters and their
JavaScript twins.

## What would reopen it

A reactive client wins only where the client must hold state the server
cannot draw. Three needs would reopen the decision:

- **Offline use:** a console that must work with no connection to its
  server, with a client-side store and a sync.
- **Collaborative live editing:** several people editing one document at
  the same time, each seeing the others' changes as they type.
- **A graphical canvas editor:** a drag-and-drop template or mapping
  designer, where the drawing is the state.

A live view that only updates (a queue count, an ETL run's progress) does
not reopen it: a module over `EventSource` that patches server-drawn
fragments fits the model.

Even when one of the three comes, the answer is one isolated widget on its
own route, with its own policy that adds what the widget needs on that route
only. It is never a framework for a whole console, and the rest of the
console keeps the policy above.

## Why speculative prefetch stays off

Speculation rules can prefetch or prerender the page a link points to. They
stay off in every console:

- A prefetch is a real `GET`. In a console that shows patient data, every
  page read is audited, so a prefetch is an audited patient read nobody
  asked for.
- The consoles send `Cache-Control: no-store` on every document, so a
  prefetched document cannot be reused and the read buys nothing.

The flash between two documents is the stylesheet's to take away, with
cross-document view transitions in CSS (#547), and no request goes out
ahead of the click.
