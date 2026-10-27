- The shell's skip link (`view::shell::skip_link`) now moves the focus to
  the main area on a page that hydrates whole: it carries `target="_self"`,
  which the `leptos_router` link handler leaves to the browser, so the
  browser's fragment navigation moves the sequential focus starting point to
  `#main` and the next Tab reaches the page's first control (WCAG 2.2 SC
  2.4.1). Without hydration it stays a plain fragment link. A console that
  set `target="_self"` on the link after hydration can drop that.
