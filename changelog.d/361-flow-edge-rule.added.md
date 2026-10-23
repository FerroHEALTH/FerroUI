- B9's edge rule is public: `view::flow::edge_rule` draws one line per
  `EdgeRule` (the state's line sample, word and line style in the library's
  words, then the console's meaning), and F9 can use it too.
  `view::flow::rule_dialog(FlowRule)` puts it in the short "How the flow is
  drawn" dialog with the console's closing paragraph. `rule_button(id)` is
  the "How it is drawn" button for the flow panel's head, which opens the
  dialog with no script. `FlowState::line` gives each state's line style.
