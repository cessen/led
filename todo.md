- Clean separation of text buffer from rest of code:
    //- Create a "BufferFormatter" trait that informs Buffer how its text
    //  should be formatted.  Such formatters then determine how the
    //  text buffer graphemes map to a 2d display (e.g. how tabs are handles,
    //  glyph size, spacing, line wrapping, etc.).  Buffers then use
    //  BufferFormatters for maintaing information necessary for
    //  1d <-> 2d operations.
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

- Undo:
    - Eventually, with global undo, the undo-stack is going to be project-wide,
      so don't think too hard about where to put it just yet.  For now, just
      put it somewhere convenient, outside of Buffer.

- Scripting:
    - What language to use for scripting?  Javascript, Lua, Python, Scheme, ...
      It should be something easy to integrate and small, so probably not
      Python.  Javascript, Lua, and Scheme all have small implementations
      that would be easy to integrate.  Scheme limits the target audience
      somewhat, as does Lua.  So Javascript is probably the best idea,
      even though it's not as simple/clean as lua or scheme.
    - In the end, only hard-code the core editing operations, and leave the
      rest to scripting.  If something ends up being too slow, you can always
      move it to be hard-coded for performance later.

- Formatting:
    - "Formatter" should really just be a factory for producing iterators for
      lines.  Everything else can be inferred from that.
    - Perhaps take the same approach as emacs, where scrolling is actually
      a percentage of the way through the data of the document, rather than
      a literal vertical position.  Alternatively, take a more complex approach
      like gedit/mousepad where the immediately visible text gets updated
      immediately, but a larger process runs in the background for a while to
      update the rest of the document.  The biggest benefit of the emacs
      approach is that it's simple and it completely decouples display of the
      buffer from the text stored in it.
    - Maybe the biggest lesson here is that regardless of how it's done, it
      shouldn't actually live inside the buffer.  Formatting information needs        to be stored outside of the buffer either way.
    - Start with the emacs approach, and you can always migrate to something
      more sophisticated later.

- Custom line iterator code for file loading, because rust's built-in one
  only recognizes LF and CRLF.
- File loading is currently very slow.  Investigate.
- Both Emacs and Vim do line-wrapping extremely efficiently, even for very
  large files.  Investigate how they do this.
- Line number display
- File opening by entering path
- UI that wraps editors, for split view.
- Persistent infinite undo
- Saving/loading files to/from buffers should be the Buffer's job, since it
  knows its own internals and can optimize things better.
- "Projects"