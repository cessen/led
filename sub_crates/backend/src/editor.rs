use std::path::PathBuf;

use crate::buffer::Buffer;

/// A struct holding the current editor state.
///
/// The Editor represents all currently open buffers available for editing.
#[derive(Debug)]
pub struct Editor {
    open_buffers: Vec<(BufferID, Buffer)>,
}

/// An ID for an open text buffer.
#[derive(Debug, Clone)]
pub enum BufferID {
    File(PathBuf), // A buffer for a normal file on disk, using the full on-disk path as the ID
    Temp(usize),   // A temporary buffer, with a number ID
}
