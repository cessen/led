- Custom line iterator code for file loading, because rust's built-in one
  only recognizes LF and CRLF.
- Line number display
- File opening by entering path
- UI that wraps editors, for split view.
- Redo functionality


- Clean up text buffer interface:
    - Buffer should know its own tab size, font, etc.  Led will NOT support
      multiple views into the same buffer with different fonts, tab sizes, etc.
      This may seem an odd choice, but it helps to think of the text buffer as
      a 2d representation of the text, for which it needs that information to
      know the relative positions of things.
    - Editing (these are undoable):
        - insert_text
        - remove_text
        - move_text
    - Undo functionality:
        - Undo
        - Redo
        - Op section begin (for delimiting composite edit operations)
        - Op section end
    - Info:
        - byte_count (useful when saving the file)
        - grapheme_count
        - line_count
    - Position conversions:
        - index -> line_col
        - line_col -> index
        - index -> vis_2d
        - vis_2d -> index
    - Reading text:
        - grapheme at index (includes visual width in return)
        - Bidirectional grapheme iterator (useful for search code, etc.)
        - Bidirectional ine iterator (useful for display code)
