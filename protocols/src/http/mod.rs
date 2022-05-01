use http::Version;
use thiserror::Error;

mod config;
mod hostname;
mod pre_request;
mod request;
mod response;

pub mod proxy;

#[derive(Error, Debug)]
pub enum ProxyHttpError {
    #[error(transparent)]
    HyperError(#[from] hyper::Error),
    #[error(transparent)]
    HttpError(#[from] http::Error),
    #[error("invalid version: {0}")]
    InvalidVersion(String),
}

fn convert_version(version: &str) -> Result<Version, ProxyHttpError> {
    match version {
        "HTTP/0.9" => Ok(Version::HTTP_09),
        "HTTP/1.0" => Ok(Version::HTTP_10),
        "HTTP/1.1" => Ok(Version::HTTP_11),
        "HTTP/2.0" => Ok(Version::HTTP_2),
        "HTTP/3.0" => Ok(Version::HTTP_3),
        _ => Err(ProxyHttpError::InvalidVersion(version.to_string())),
    }
}
