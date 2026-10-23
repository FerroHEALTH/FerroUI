- Breaking: `view::scope::Request` gains the `Profile` variant, so a console
  that matches on `Request` exhaustively adds a `Profile` arm. The library's
  catalog adds `scope-profile`, "Your profile: your name, picture and the
  like", so a console that translates the library's catalog adds it.
