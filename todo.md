- Clean separation of text buffer from rest of code:
    //- Create a "BufferFormatter" trait that informs Buffer how its text
    //  should be formatted.  Such formatters then determine how the
    //  text buffer graphemes map to a 2d display (e.g. how tabs are handles,
    //  glyph size, spacing, line wrapping, etc.).  Buffers then use
    //  BufferFormatters for maintaing information necessary for
    //  1d <-> 2d operations.
    - Buffer should only store the text itself and the encoding type.  The
      undo stack and line ending type should be moved into the editor since
      they only apply to editing operations.  The buffer should only store
      data related to how the text is statically presented or stored on
      disk.
    - Create BufferFormatters for console and for freetype, including
      preferences for tab width (specified in spaces) and line wrapping.
      The freetype formatter should not reference SDL at all, and should
      have only the freetype library itself as a dependency.
    //- Handle tab settings properly after the refactor
    //- Buffer needs a "reformat" method, which can be run after the formatter
    //  is changed in some way (e.g. toggling line wrapping).
    - Possibly split the text buffer out into its own library...?  Would
      likely be useful to other people as well, and would encourage me to
      keep the API clean.

- Custom line iterator code for file loading, because rust's built-in one
  only recognizes LF and CRLF.
- File loading is currently very slow.  Investigate.
- Line number display
- Line wrapping
- File opening by entering path
- UI that wraps editors, for split view.
- Persistent infinite undo
- Multiple cursors
- "Projects"

