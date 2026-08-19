use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP request failed")]
    Http(#[from] reqwest::Error),
    #[error("gRPC call failed: {0}")]
    Grpc(#[from] tonic::Status),
    #[error("could not connect to gRPC server")]
    Transport(#[from] tonic::transport::Error),
    #[error("resource not found")]
    NotFound,
    #[error("authentication failed")]
    Unauthorized,
    #[error("server rejected the request: {0}")]
    InvalidRequest(String),
    #[error("server returned an error: {0}")]
    Server(String),
    #[error("server response is missing field: {0}")]
    MalformedResponse(&'static str),
    #[error("operation requires auth")]
    MissingToken,
}
