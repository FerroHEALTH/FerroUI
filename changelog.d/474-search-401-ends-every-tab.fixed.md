- A search whose answer route answers `401` now says to every other tab of
  the console that the session has ended (`widget::session::gone`), so they
  leave for sign-in with it; with `SearchBox::signed_out` set its own tab
  still loads the signed-out page, and its session watch stays put.
