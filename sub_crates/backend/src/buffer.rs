use ropey::Rope;

#[derive(Debug, Clone)]
pub struct Buffer {
    // on_disk_encoding: Encoding,
    content_type: String,
    is_dirty: bool,
    text: Rope, // The actual text content.
}
