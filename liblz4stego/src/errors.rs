use std::{error, fmt};

#[derive(Debug, Clone)]
pub struct Lz4DecompressError {
    message: String,
}

impl Lz4DecompressError {
    pub fn from_string(message: String) -> Self {
        Lz4DecompressError { message }
    }

    pub fn from_static_str(message: &'static str) -> Self {
        Lz4DecompressError {
            message: message.to_owned(),
        }
    }
}

impl error::Error for Lz4DecompressError {}

impl From<std::io::Error> for Lz4DecompressError {
    fn from(e: std::io::Error) -> Self {
        Self::from_string(e.to_string())
    }
}

impl From<Lz4DecompressError> for std::io::Error {
    fn from(e: Lz4DecompressError) -> Self {
        Self::new(std::io::ErrorKind::InvalidInput, e)
    }
}

impl fmt::Display for Lz4DecompressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "error during LZ4 decompressing at offset {}",
            self.message
        )
    }
}

pub type DecompressResult<T> = std::result::Result<T, Lz4DecompressError>;
