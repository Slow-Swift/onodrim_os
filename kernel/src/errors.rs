
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorStatus {
    InvalidFileFormat
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
    status: ErrorStatus
}

impl Error {
    pub fn new(status: ErrorStatus) -> Error {
        Error { status }
    }
}