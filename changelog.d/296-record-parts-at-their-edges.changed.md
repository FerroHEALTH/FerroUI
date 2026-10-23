- Breaking: `view::scope::Grant` has a `Request(Request)` variant and
  `view::row_preview::RowValue` a `Null` variant, so a console that matches
  either whole adds the arm. A console that builds its grants with
  `Scope::parse` and refuses every error should move to `ScopeGrant::read`,
  which no longer marks `openid`, `fhirUser` or `launch/patient`
  malformed. The masked identifier's busy Reveal is `aria-disabled` where
  it was `disabled`, so it can hold the focus. `RowEnd::Kept` draws its
  reason as a visible line under the row and its lock is hidden from
  assistive technology, where the reason was the disabled button's name
  and tooltip.
