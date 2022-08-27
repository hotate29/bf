#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{msg}")]
    InvalidSyntax { msg: String },
}
