- F7: a list nested in a `Fields::Card` row adds rows in place in a card
  that Add inserted. `Repeat::token` names each list's own key token
  (`ROW` for a top-level list, a token of its own such as `__profile__` for
  a nested one), and the island rekeys the content of nested templates. It
  also takes the presses of a nested list in a card it copied, and finds
  only its own blank row, rows, empty line and Add. Before, a list whose
  drawn cards held a nested blank row copied that nested row when Add was
  pressed. Breaking: a `Repeat` literal adds `token: ROW`, and a list nested
  in a sub-card names its own token in it and in its blank row's ids and
  names.
