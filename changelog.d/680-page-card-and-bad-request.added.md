- `error_page::Status::BadRequest` draws a 400, "Bad request · 400", for a
  request a console cannot read, so a console no longer shows a 403 in its
  place (R20).
- `surface::page_card` is a page of its own in the centred layout, such as
  a consent or a management page, whose title is the page's one `h1`
  (R30). The gallery draws it at `/consent/:brand` and the 400 at
  `/:brand/templates/bad-request`.
