#[derive(Debug, thiserror::Error)]
pub enum ClayError {
    #[error("{0}")]
    PlatformError(String),
}

pub type Result<T = ()> = std::result::Result<T, ClayError>;
