- A tab that signs out lands on the signed-out page when the session
  watch's read answers `401` during the sign-out. The watch no longer
  sends a page whose own form is already loading the next document to the
  expired sign-in page; it tells the other tabs, which still leave for
  sign-in with their own address as `return`.
