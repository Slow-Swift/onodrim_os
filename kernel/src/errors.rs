
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorStatus {
    GraphicsPixelFormatNotSupported
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