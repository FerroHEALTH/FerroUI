- `view::lifecycle`, the revision lifecycle and its pending card (F1):
  `lifecycle` draws Saved, Approval, Apply and Confirm as an ordered list,
  each `Step` done, current (`aria-current="step"`), waiting, not needed or
  failed (`StepState`) in a glyph, a colour and a hidden state word, one
  line from 40 rem up and stacked below it; `pending_card` puts a
  revision's words, its actions, its lifecycle and a line under them in one
  card (`PendingCard`), a polite status region when it carries a tone.
- `view::action::small_action_button`, an `Action` as a small button, for a
  card's header row.
