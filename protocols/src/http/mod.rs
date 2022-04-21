use http::{Request, Uri, Version};
use hyper::Body;
use thiserror::Error;

pub use self::request::add_to_linker;
use self::request::HttpRequest;

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

pub struct ProxyHttpRequest {
    request: HttpRequest,
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

impl TryFrom<ProxyHttpRequest> for Request<Body> {
    type Error = ProxyHttpError;

    fn try_from(req: ProxyHttpRequest) -> Result<Self, Self::Error> {
        let request = req.request;
        let body = Body::from(request.body);
        let request = Request::builder()
            .method(request.method.as_str())
            .version(convert_version(&request.version)?)
            .body(body)
            .map_err(ProxyHttpError::from)?;

        Ok(request)
    }
}

impl ProxyHttpRequest {
    pub async fn new(req: http::Request<Body>) -> Result<Self, ProxyHttpError> {
        let (parts, body) = req.into_parts();
        let uri = parts.uri;
        let host = uri.host().map(String::from).unwrap_or_else(String::new);
        let authority = uri
            .authority()
            .map(|auth| auth.to_string())
            .unwrap_or_else(|| String::from(""));
        let path = uri.path().to_string();
        let scheme = uri
            .scheme()
            .map(|scheme| scheme.to_string())
            .unwrap_or_else(|| String::from(""));
        let method = parts.method.as_str().to_string();
        let version = format!("{:?}", parts.version);
        let headers = parts
            .headers
            .iter()
            .flat_map(|(name, value)| match value.to_str() {
                Ok(value) => Some((name.to_string(), value.to_string())),
                Err(_) => None,
            })
            .collect();
        let body = hyper::body::to_bytes(body).await?.to_vec();
        let request = HttpRequest {
            path,
            authority,
            scheme,
            version,
            headers,
            method,
            host,
            body,
        };
        Ok(Self { request })
    }
}

impl request::Request for ProxyHttpRequest {
    fn http_request_get(&mut self) -> Result<request::HttpRequest, request::Error> {
        Ok(self.request.clone())
    }

    fn http_request_set_method(&mut self, method: &str) -> Result<(), request::Error> {
        self.request.method = method.into();
        Ok(())
    }

    fn http_request_set_header(&mut self, header: &str, value: &str) -> Result<(), request::Error> {
        match self
            .request
            .headers
            .iter()
            .enumerate()
            .find(|(_idx, (name, _value))| name == header)
        {
            Some((idx, _)) => {
                self.request.headers[idx].1 = value.to_string();
            }
            None => self
                .request
                .headers
                .push((header.to_string(), value.to_string())),
        };
        Ok(())
    }

    fn http_request_set_uri(&mut self, uri: &str) -> Result<(), request::Error> {
        let uri = Uri::try_from(uri).map_err(|err| format!("Invalid uri: {err}"))?;
        self.request.host = uri.host().map(String::from).unwrap_or_else(String::new);
        self.request.authority = uri
            .authority()
            .map(|auth| auth.to_string())
            .unwrap_or_else(|| String::from(""));
        self.request.path = uri.path().to_string();
        self.request.scheme = uri
            .scheme()
            .map(|scheme| scheme.to_string())
            .unwrap_or_else(|| String::from(""));
        Ok(())
    }

    fn http_request_set_version(&mut self, version: &str) -> Result<(), request::Error> {
        match version {
            "HTTP/0.9" | "HTTP/1.0" | "HTTP/1.1" | "HTTP/2.0" | "HTTP/3.0" => Ok(()),
            _ => Err(format!("Invalid version: {version}")),
        }?;

        self.request.version = version.to_string();

        Ok(())
    }

    fn http_request_rm_header(&mut self, header: &str) -> Result<(), request::Error> {
        if let Some((idx, _)) = self
            .request
            .headers
            .iter()
            .enumerate()
            .find(|(_idx, (name, _value))| name == header)
        {
            self.request.headers.remove(idx);
        }
        Ok(())
    }

    fn http_request_set_body(
        &mut self,
        body: request::BodyParam<'_>,
    ) -> Result<(), request::Error> {
        self.request.body = body.to_vec();
        Ok(())
    }
}
