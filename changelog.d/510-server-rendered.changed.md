- **Breaking:** the library renders server-side HTML from Askama templates
  and depends on no Leptos crate. Every component is a function or a
  struct that returns `Html`, which a console's own Askama template writes
  unescaped with `{{ part }}`; a component that needs words takes the
  request's `&Locale`. The shell writes the whole document, and the
  interactive behaviour comes from small plain ES modules under
  `/ferrohealth-ui/<version>/`, which the one loader `fui.js` imports for
  each element that asks for one with `data-fui`. Every page works with the
  modules refused. `serve::assets()` (the default `axum` feature) serves
  the stylesheet, the brand themes and the modules. `docs/migration-0.6.md`
  maps every removed item to its successor.
