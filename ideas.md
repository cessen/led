# General

- Persistent editing state, so you can close and open a file (or the entire editor) and still keep your unsaved edits.
- No concept of tabs or open-but-invisible buffers.  The only open files are the ones you see.
- Persistent, infinite undo.
- Auto-file saving by default, so you never have "dirty" buffers.  The zed-app workflow.

# UI

- Split view, but only auto-layout.  The user just specifies how many views they want, and the editor figures out how to lay them out.  Probably the user can tweak knobs for how the auto-layout works.
- Only text-wrapped views, no horizontal scrolling.  But the text wrapping needs to be good, e.g. preserving indent level etc.