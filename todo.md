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

- Text encoding support:
    - Buffers need to know what encoding they represent.
    - Loading/saving code for different encodings.
    - Auto-detecting text encodings from file data (this one will be tricky).

- Word wrap.
- Get non-wrapping text working again.
- Line number display
- File opening by entering path
- UI that wraps editors, for split view.
- Persistent infinite undo
- "Projects"