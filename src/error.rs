use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("wip")]
    Error,
}
