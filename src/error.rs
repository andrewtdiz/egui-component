#[derive(Debug, thiserror::Error)]
pub enum ComponentLibraryError {
    #[error("{0}")]
    Runtime(String),
}

pub type Result<T = ()> = std::result::Result<T, ComponentLibraryError>;
