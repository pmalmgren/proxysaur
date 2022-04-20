use thiserror::Error;

pub use self::request::add_to_linker;
use self::request::{BodyParam, BodyResult, HttpRequestResult};

mod request;
mod response;

#[derive(Error, Debug)]
pub enum HttpProtocolError {}

pub struct HttpRequest {
    request: http::Request<Vec<u8>>,
    _response: http::Response<Vec<u8>>,
}

impl From<&http::Request<Vec<u8>>> for HttpRequestResult {
    fn from(req: &http::Request<Vec<u8>>) -> Self {
        let uri = req.uri();
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
        let method = req.method().as_str().to_string();
        let version = format!("{:?}", req.version());
        let headers = req
            .headers()
            .iter()
            .flat_map(|(name, value)| match value.to_str() {
                Ok(value) => Some((name.to_string(), value.to_string())),
                Err(_) => None,
            })
            .collect();
        HttpRequestResult {
            path,
            authority,
            scheme,
            version,
            headers,
            method,
            host,
        }
    }
}

impl request::Request for HttpRequest {
    fn http_request_get(&mut self) -> Result<HttpRequestResult, request::Error> {
        Ok(HttpRequestResult::from(&self.request))
    }

    fn http_request_set(
        &mut self,
        req: request::HttpRequestParam<'_>,
    ) -> Result<(), request::Error> {
        let uri = http::Uri::builder()
            .authority(req.authority)
            .path_and_query(req.path)
            .scheme(req.scheme)
            .build()
            .map_err(|err| format!("Error building URI: {}", err))?;
        let version = match req.version {
            "HTTP/0.9" => Ok(http::Version::HTTP_09),
            "HTTP/1.0" => Ok(http::Version::HTTP_10),
            "HTTP/1.1" => Ok(http::Version::HTTP_11),
            "HTTP/2.0" => Ok(http::Version::HTTP_2),
            "HTTP/3.0" => Ok(http::Version::HTTP_3),
            _ => Err(format!("Invalid HTTP version: {}", req.version)),
        }?;
        let req = http::Request::builder()
            .method(req.method)
            .version(version)
            .uri(uri)
            .body(vec![])
            .map_err(|err| format!("Error setting the request: {err}"))?;
        self.request = req;
        Ok(())
    }

    fn http_request_body_get(&mut self) -> Result<BodyResult, request::Error> {
        Ok(self.request.body().clone())
    }

    fn http_request_body_set(&mut self, body: BodyParam<'_>) -> Result<(), request::Error> {
        *self.request.body_mut() = body.to_vec();
        Ok(())
    }
}
