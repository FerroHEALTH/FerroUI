- `fui-unit` and `fui-nowrap` keep their text whole from the stylesheet's
  `utilities` layer, which comes after every component rule, in place of
  `!important`, so a console's own unlayered rule can now override them. The
  menu sheet's title, the lockup's accent word and the phone's Keyboard tab
  hold by the cascade alone, and draw as before.
