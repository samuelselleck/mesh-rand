use thiserror::Error;

#[derive(Error, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MeshRandError {
    #[error("failed to initialize: {0}")]
    Initialization(String),
}
