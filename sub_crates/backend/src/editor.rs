use crate::buffer::Buffer;

/// A struct holding the current editor state.
///
/// The Editor represents all currently open buffers available for editing.
#[derive(Debug)]
pub struct Editor {
    open_buffers: Vec<Buffer>,
}
