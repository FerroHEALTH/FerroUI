- `SearchBox::all` names the panel's option that submits the search, around
  the open `query`, for a console whose submit runs another search than the
  panel's (a patient search the panel never runs); `None` keeps "See all
  results for “{query}”". `SearchBox::signed_out` is the address the page
  loads when the answer route answers `401`, so a session that ended while
  the panel was open hands the page to the console's sign-in.
