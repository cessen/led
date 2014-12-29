- Proper handling of non-uniform-width characters.  Specifically, this needs
  to address tabs.  But it should be done to handle the general case anyway,
  since that's unlikely to be more complex and will future-proof things.
- Line number display
- Editor info display (filename, current line/column, indentation style, etc.)
- File opening by entering path
- UI that wraps editors, for split view.
- Unit testing for text block, text node, and text buffer.  They must
  be reliable!

- Change text data structure to store lines explicitly.  Still use a tree
  structure to hold the lines, but just store the lines themselves as
  straight vectors for now.  Be mindful to keep the API's clean enough
  that you can substitute another internal storage approach for lines
  later on.