- A signed-in page leaves for sign-in the moment its session ends:
  `island::SessionWatch` (`widget::session`) loads the console's sign-in
  page with the page's address as `return` when the session is gone, warns
  with a modal alert dialog at least 20 seconds (120 by default) before an
  idle timeout with Stay signed in and Sign out (WCAG 2.2 SC 2.2.1), asks
  again when the tab is shown or focused, and ends every tab of the console
  together over a `BroadcastChannel`. Only the user's own input extends the
  session. A console draws the watch in the shell of a signed-in page,
  answers its session route on the wire `docs/consoles.md` § The session's
  end documents (`GET` reads, `POST` extends, `401` when gone), draws
  `island::SessionEnded` on its signed-out page, and calls
  `widget::session::gone()` for a `401` its own code meets.
