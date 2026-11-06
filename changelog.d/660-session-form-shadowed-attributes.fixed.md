- The session watch reads a submitted form's `method` and `target` content
  attributes, so a form that holds a control named `target` or `method`
  no longer throws `target.toLowerCase is not a function` on submit, and
  the watch still tells whether the form loads the next document in its
  tab.
