- Your preferences as library views: `view::preferences::preferences_page`
  draws exactly three tabs, Appearance (theme and density as R28 radio
  preview cards drawing both themes' preview accents, reduce motion),
  Language and region (`reading: true` names it Reading, as FerroEHR does;
  the language as an R29 select with each language in its own name, the
  first day of the week) and Keyboard (shortcut hints and the legend,
  hidden below 40 rem through `data-tab="keyboard"`). A console passes the
  preferences it read, its routes and the cookie name it just kept
  (`saved`), and puts its own preferences inside the three tabs (`own`)
  with `preference_row`, `select_preference` and `switch_preference`.
  Every preference is a plain form post that works with no WebAssembly
  (the new `island::PreferenceForm` posts on change once hydrated and
  returns the focus to the row just saved); Saved has a fixed slot that
  never moves its control; Reset to defaults asks first and posts to the
  console's route. The library writes no cookie.
- `theme::Preference` (each preference's cookie name, cookie value and
  parse, a missing or unknown value read as the default), the new
  preferences `Motion`, `WeekStart`, `Hints` and `Language`,
  `theme::Preferences` with `Preferences::read` over a `Cookie` header, and
  `theme::cookie`.
- The stylesheet reads `data-motion="reduce"` as `prefers-reduced-motion`
  does and `data-hints="always"` or `"never"` for the key hints; write both
  on `<html>` from their cookies. Under reduced motion the Saved mark stays
  instead of fading.
