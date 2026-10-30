- The loader marks an element `data-fui-ready` (`script::READY`) once every
  module its `data-fui` names has enhanced it, so a console's journey can
  wait for a component whose module changes nothing it can see before it
  drives it by the keyboard; `html[data-fui-script]` is written before any
  module arrives.
