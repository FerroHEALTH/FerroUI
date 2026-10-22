- Breaking: `widget::menu::UserMenu::choices` is a `Vec<Choices>`, so the
  user menu holds one group per control the top bar shows from 40 rem up,
  such as a brand and the FHIR version. A console that passed
  `Some(choices)` passes `vec![choices]`, and `None` becomes `Vec::new()`.
