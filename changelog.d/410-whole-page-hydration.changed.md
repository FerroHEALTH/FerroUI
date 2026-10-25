- Breaking: a console hydrates the whole page and draws every page after the
  first in the browser, so no link or form reloads the document; islands mode
  is gone. Every ready component of `island` is a `#[component]`, the crate no
  longer turns on the `leptos` feature `islands`, and its `split` feature is
  gone. A console drops `islands` and `split`, draws each page as a
  `leptos_router` `LazyRoute`, hydrates with `leptos::mount::hydrate_lazy`,
  writes `<HydrationScripts options/>`, builds with
  `cargo leptos build --split`, and provides the same locale and state in both
  halves (`docs/consoles.md`, How a console builds). A repeatable list's blank
  row is a hidden, disabled `fieldset` in place of a `template`, so it
  hydrates with the page and never posts.
