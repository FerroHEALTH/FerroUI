- `widget::action` answers a plain post on its page: a console wraps its
  router in `draw_answers`, draws `answer_field()` in an `<ActionForm>` and
  calls `answered(&action)` where it makes the action, and with no
  WebAssembly the post draws the page the form was on with the function's
  answer, or its error, as the action's value and the posted fields
  (`Posted`) in the form, `Cache-Control: no-store`. The redirect mode of
  `return_field` and `with_error` is unchanged, and with WebAssembly the
  action runs in the page as before. `widget::hydration::hydrate` puts the
  page's own address in the history before it hydrates such a page.
  `docs/consoles.md` § An action that answers on its page documents both
  modes.
