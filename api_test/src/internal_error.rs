use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum InternalError {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Generic error {text})")]
    GenericError { text: String },
}

#[allow(dead_code)]
pub type InternalResult<T> = std::result::Result<T, InternalError>;
