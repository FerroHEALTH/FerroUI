- `widget::hydration::mark()`: its effect can run while a lazy route's code
  is still on its way, so the mark could come before the page hydrated.
  Hydrate with `widget::hydration::hydrate(App)`, which writes the mark when
  hydration ends (#448).
