#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),
    /// Generic
    #[error("{0:#}")]
    Anyhow(#[from] anyhow::Error),
}
