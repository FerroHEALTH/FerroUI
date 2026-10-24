- The FerroEHR New contribution mockup draws an Apply submit beside each
  staged change's What changes, as a console has to with no WebAssembly,
  since choosing a kind redraws the fields under it. The select and Apply
  share one row inside the field, `view::layout::field_row(FieldRow::WithActions, ..)`,
  with the help line under both. COMPONENTS.md
  § Interaction states the rule: a select whose choice changes the page
  carries a visible submit beside it and never posts on its change alone
  (WCAG 2.2 SC 3.2.2 On Input), as the phone tab select's Go does. A
  console that draws Apply already needs no change, and can move it into
  that row to match the design.
