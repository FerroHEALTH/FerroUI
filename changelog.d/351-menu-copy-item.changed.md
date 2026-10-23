- Breaking: `widget::action_menu::Act` gains the `Copy` variant, so a
  console that matches on `Act` exhaustively adds a `Copy` arm.
  `widget::action_menu::menu_button` takes a `copy` argument
  (`Option<widget::copy::CopyWords>`) before its children, and
  `island::MenuButton` a `copy` prop: `None` for a menu with no copy item
  (#351).
