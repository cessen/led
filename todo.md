- Syntax highlighting:
    - It's tempting to want to do syntax highlighting only on the bare
      minimum parts of the text after an edit, but realistically there
      are always cases where the entire text has to be scanned again to
      get correct results.  So it must be something that can be done
      asynchronously.
    - Maybe a quick-n-dirty local update, followed by an async background
      update.
    - Should the syntax highlighting data be stored in the text buffer itself?
      Or should there be an accompanying structure on the side for that?
    - What do other editors do?

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

- Line number display
- File opening by entering path
- UI that wraps editors, for split view.
- Persistent infinite undo
- "Projects"