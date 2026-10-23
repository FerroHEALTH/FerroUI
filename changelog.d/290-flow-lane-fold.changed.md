- Breaking: `view::flow::FlowLane` takes `fold: Option<LaneFold>`, how the lane folds
  past `FOLD_PAST` cards; `None` keeps a lane that never folds. A struct
  literal that builds a lane names the new field.
