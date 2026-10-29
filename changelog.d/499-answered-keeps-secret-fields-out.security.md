- A field an answer form marks with `widget::action::secret_mark`, a password
  or a key, goes to the server function alone: the page `draw_answers` draws
  with the answer leaves it out of `Posted`, so its value is no longer written
  into the page's hydration data. `view::secret::secret_field` marks its value
  input itself; a console marks its own secret inputs, `view::form::masked_input`
  among them (GHSA-25wh-2jcm-gvpr).
