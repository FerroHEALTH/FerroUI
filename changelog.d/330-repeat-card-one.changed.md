- Breaking: `widget::repeat::RepeatRow` gains the public field `heading`, so
  a `RepeatRow` literal adds `heading: None`, and `widget::repeat::Fields`
  gains the `One` and `Card` variants, so a console that matches on
  `Fields` exhaustively adds two arms.
