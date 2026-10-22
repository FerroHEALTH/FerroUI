- `view::timeline::timeline_window` draws the newest `Window::limit` events
  of a long history and a foot saying how many show of how many, with a
  plain link to the console's page of older events (`Window::older`).
  `view::scope::Request` reads the SMART App Launch 2.2.0 scopes that ask
  for no resource (`openid`, `fhirUser`, `launch`, `launch/<resource>`
  with its `?role=`, `online_access`, `offline_access`) and
  `ScopeGrant::read` reads any raw scope into its grant.
  `view::row_preview::RowValue::Null` draws a column with no value as
  `NULL`, apart from an empty text, which now reads "empty text".
