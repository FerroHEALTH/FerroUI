- The shell can be drawn once and survive a navigation in the page, as
  #410 meant: `widget::nav::follow_path()`, called in the component that
  draws the shell inside the console's `Router`, lets the navigation mark
  the page in view and the rail's and the theme choice's `back` name it as
  the router moves, so a console no longer draws its shell again for each
  path. The search's words and the shell's state now survive a link click
  in the gallery (`docs/consoles.md`, How a console builds, step 10).
