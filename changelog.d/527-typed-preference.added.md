- `view::preferences::typed_preference` draws a console's own typed
  preference, a text or a number field (`Typed::Text` with its length
  bounds, `Typed::Number` with its `min`, `max` and `step`) with its
  placeholder, in the same plain form as `select_preference`: its Save
  button posts it with no script, and once the `preference` module runs it
  posts when the field changes and the row just saved gives the field the
  focus back. The module sends one post when Enter both changes and
  submits a field.
