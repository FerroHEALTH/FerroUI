- R10 takes a masked input with a hold-to-show eye: `view::form::masked_input`
  draws the input `type="password"` with no autocomplete, capitals or spelling
  check, and `view::form::Inner::HoldToShow` puts the eye inside its edge
  (`island::HoldToShow` over `widget::masked::hold_to_show`, words in
  `widget::masked::HoldWords`, named by the console with `HoldWords::named`).
  Held by the pointer, by touch or by Space or Enter, the eye shows the typed
  text with `aria-pressed` in step, and released, left or blurred, it masks it
  again. The eye names its input by id, so the typed value is never an island
  prop. Without WebAssembly it is disabled, its name says the field stays
  masked, and the field stays masked. `Inner` gains a variant, so a console
  that matches on `Inner` exhaustively adds an arm.
