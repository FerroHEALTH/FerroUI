- `view::transform`, the transform pair (B5): the options row
  (`transform_options`, plain or `Density::Dense`) with Run at its end
  (`run_button`, ready, running or off with its reason), the input and
  answer panes with their sticky heads and the answer's state
  (`transform_pair`, `TransformPane`, `AnswerState`), the answer before the
  first run, running, answered and refused (`Answer`), the request an
  answer came from with Copy and the raw answer (`request`), and a group of
  check boxes as one field (`field_checks`).
- `Field::holds_id`, an options row's id field (`is-id`), wide enough for a
  whole uuid or versioned object uid.
