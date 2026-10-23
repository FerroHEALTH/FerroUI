- The code editor (B2), the YAML editor and the upload field work with no
  WebAssembly: an editable text area takes keys from the first paint, shows
  its own text in ink over the layer's gutter until its island runs
  (`fui-editor-stack.is-static`), and hands what it holds as the page
  hydrates to `on_input`, so an early key is never lost or written over.
