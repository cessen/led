- Proper handling of non-uniform-width characters.  Specifically, this needs
  to address tabs.  But it should be done to handle the general case anyway,
  since that's unlikely to be more complex and will future-proof things.
- Line number display
- Editor info display (filename, current line/column, indentation style, etc.)
- File opening by entering path
- UI that wraps editors, for split view.
