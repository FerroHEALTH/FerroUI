- `view::state::state(mark, word, detail)` draws the F12 state mark a table
  cell, a fact or a title badge shows: the shape of `view::state::Mark` (`Ok`
  a filled circle, `Warn` a diamond, `Bad` a cross, `Neutral` a hollow ring,
  `Off` the hollow ring with the word muted), the word, and an optional quiet
  detail. `Mark` converts from `view::flow::FlowState`. The subject line, the
  version history and the topology's list draw their states through it, and
  the topology's list now hides the shape from assistive technology, since
  the word beside it names the state.
