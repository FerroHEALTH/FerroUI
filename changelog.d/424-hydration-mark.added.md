- `widget::hydration`: `mark()` writes `data-ferroui-hydrated` on the root
  element once the page has hydrated, which a console's code, scripts and
  tests wait for before they change the page, and in a debug build
  `explain()` has a panic before that point say its likely cause, a page
  changed before hydration ended, where Leptos names none. The consumer
  guide gives the rule (`docs/consoles.md`, How a console builds).
