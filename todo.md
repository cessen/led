- Custom line iterator code for file loading, because rust's built-in one
  only recognizes LF and CRLF.
- Line number display
- File opening by entering path
- UI that wraps editors, for split view.
- Redo functionality


- Clean up text buffer interface:
    - Editing (these are undoable):
        //- insert_text
        //- remove_text
        - move_text
    - Undo functionality:
        //- Undo
        //- Redo
        - Op section begin (for delimiting composite edit operations)
        - Op section end
    - Info:
        - byte_count (useful when saving the file)
        //- grapheme_count
        //- line_count
    - Position conversions:
        //- index -> line_col
        //- line_col -> index
        //- index -> vis_2d
        //- vis_2d -> index
    - Reading text:
        - grapheme at index (includes visual width in return)
        - Bidirectional grapheme iterator (useful for search code, etc.)
        - Bidirectional ine iterator (useful for display code)
