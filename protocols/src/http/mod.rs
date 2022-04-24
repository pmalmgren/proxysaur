use http::{Request, Response, Uri, Version};
use hyper::Body;
use thiserror::Error;

pub use self::request::add_to_linker as request_add_to_linker;
pub use self::response::add_to_linker as response_add_to_linker;
use self::{request::HttpRequest, response::HttpResponse};

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

pub struct ProxyHttpResponse {
    response: HttpResponse,
}

impl TryFrom<ProxyHttpResponse> for Response<Body> {
    type Error = ProxyHttpError;

    fn try_from(value: ProxyHttpResponse) -> Result<Self, Self::Error> {
        let resp = Response::builder()
            .status(value.response.status)
            .body(Body::from(value.response.body))?;
        Ok(resp)
    }
}

impl ProxyHttpResponse {
    pub async fn new(response: Response<Body>) -> Result<Self, ProxyHttpError> {
        let (parts, body) = response.into_parts();
        let headers = parts
            .headers
            .iter()
            .flat_map(|(name, value)| match value.to_str() {
                Ok(value) => Some((name.to_string(), value.to_string())),
                Err(_) => None,
            })
            .collect();
        let body = hyper::body::to_bytes(body).await?.to_vec();
        let response = HttpResponse {
            headers,
            status: parts.status.as_u16(),
            body,
        };

        Ok(Self { response })
    }
}

impl response::Response for ProxyHttpResponse {
    fn http_response_get(&mut self) -> Result<HttpResponse, response::Error> {
        Ok(self.response.clone())
    }

    fn http_response_set_status(&mut self, status: u16) -> Result<(), response::Error> {
        self.response.status = status;
        Ok(())
    }

    fn http_response_set_body(
        &mut self,
        body: response::BodyParam<'_>,
    ) -> Result<(), response::Error> {
        self.response.body = body.to_vec();
        Ok(())
    }

    fn http_response_set_header(
        &mut self,
        header: &str,
        value: &str,
    ) -> Result<(), response::Error> {
        match self
            .response
            .headers
            .iter()
            .enumerate()
            .find(|(_idx, (name, _value))| name == header)
        {
            Some((idx, _)) => {
                self.response.headers[idx].1 = value.to_string();
            }
            None => self
                .response
                .headers
                .push((header.to_string(), value.to_string())),
        };
        Ok(())
    }

    fn http_response_rm_header(&mut self, header: &str) -> Result<(), response::Error> {
        if let Some((idx, _)) = self
            .response
            .headers
            .iter()
            .enumerate()
            .find(|(_idx, (name, _value))| name == header)
        {
            self.response.headers.remove(idx);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ProxyHttpRequest {
    request: HttpRequest,
}

impl TryFrom<ProxyHttpRequest> for Request<Body> {
    type Error = ProxyHttpError;

    fn try_from(req: ProxyHttpRequest) -> Result<Self, Self::Error> {
        let request = req.request;
        tracing::info!(?request, "Building the request.");
        let body = Body::from(request.body);
        let uri = Uri::builder()
            .authority(request.authority)
            .scheme(request.scheme.as_str())
            .path_and_query(request.path)
            .build()?;
        tracing::info!(?uri, "Built URI.");
        let request = Request::builder()
            .method(request.method.as_str())
            .version(convert_version(&request.version)?)
            .uri(uri)
            .body(body)
            .map_err(ProxyHttpError::from)?;
        tracing::info!(?request, "Built request.");

        Ok(request)
    }
}

impl ProxyHttpRequest {
    pub async fn new(
        req: http::Request<Body>,
        scheme: &str,
        authority: &str,
    ) -> Result<Self, ProxyHttpError> {
        let (parts, body) = req.into_parts();
        let uri = parts.uri;
        let host = uri.host().map(String::from).unwrap_or_else(String::new);
        let authority = uri
            .authority()
            .map(|auth| auth.to_string())
            .unwrap_or_else(|| String::from(authority));
        let path = uri.path().to_string();
        let scheme = uri
            .scheme()
            .map(|scheme| scheme.to_string())
            .unwrap_or_else(|| String::from(scheme));
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
