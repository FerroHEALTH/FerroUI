- The top bar's search writes its state only while it lives: a blur, a
  submit or a history `pageshow` that arrives after a navigation disposed
  the search no longer reads a disposed value.
