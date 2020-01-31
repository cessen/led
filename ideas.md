# General

- Persistent editing state, so you can close and open a file (or the entire editor) and still keep your unsaved edits.
- No concept of tabs or open-but-invisible buffers.  The only open files are the ones you see.
- Auto-file saving by default, so you never have "dirty" buffers.  The zed-app workflow.

# Undo

- Persistent, infinite undo.
- The undo command doesn't pop anything off the undo stack, rather it walks back until the user is happy and then pushes that state onto the top of the undo stack.  The idea is that you never lose any undo state, even when undoing.  This will definitely feel weird to people used to more standard undo systems, but it unlocks the possibility of e.g. only undoing within a selected area, among other things.

# UI

- Split view, but only auto-layout.  The user just specifies how many views they want, and the editor figures out how to lay them out.  Probably the user can tweak knobs for how the auto-layout works.
- Only text-wrapped views, no horizontal scrolling.  But the text wrapping needs to be good, e.g. preserving indent level etc.