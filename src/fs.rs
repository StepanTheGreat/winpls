//! A _cross-platform_ file system access API

use std::{fs, io, path, string};

/// A file system error
#[derive(Debug)]
pub enum FSError {
    /// An IO error (for example a file doesn't exist)
    IO(io::Error),

    /// A unicode conversion error (failed to convert bytes into a string)
    Unicode(string::FromUtf8Error),
}

/// A file system result (either T, or a file system error)
pub type FSResult<T> = Result<T, FSError>;

/// Read bytes from the provided file path
pub fn read_bytes(path: impl AsRef<path::Path>) -> FSResult<Vec<u8>> {
    let path = path.as_ref();

    fs::read(path).map_err(FSError::IO)
}

/// Read a string from the provided file path
///
/// Under the hood it will just call [read_bytes] with [String::from_utf8]
pub fn read_string(path: impl AsRef<path::Path>) -> FSResult<String> {
    let bytes = read_bytes(path)?;

    String::from_utf8(bytes).map_err(FSError::Unicode)
}
