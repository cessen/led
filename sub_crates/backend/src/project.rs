use std::path::PathBuf;

/// The set of files and directories specified as part of a project.
///
/// These represent the items explicitly added by the user.  Files
/// that are already under one of the `directories` aren't duplicated
/// in the `files` list.
pub struct ProjectSet {
    pub directories: Vec<PathBuf>,
    pub files: Vec<PathBuf>,
}
