- A click or a submit while a lazy route still hydrates no longer panics
  the page. `widget::hydration::hydrate(App)`, the new browser entry in place
  of `leptos::mount::hydrate_lazy`, leaves every click and submit to the
  browser until the page and its lazy route have hydrated, so a link loads
  its page and a form posts as with no WebAssembly. It writes
  `data-ferroui-hydrated` when hydration ends, where `widget::hydration::mark()`
  wrote it from an effect that can run before a lazy route has hydrated;
  `mark()` is deprecated and does nothing once `hydrate` is the entry. A
  console calls `hydrate` from its browser entry and drops `mark()` (#448).
