use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{msg}")]
    InvalidSyntax { msg: &'static str },
    #[error("{0}")]
    IoError(#[from] io::Error),
}
