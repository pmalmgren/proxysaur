use std::path::PathBuf;

use http::Uri;
use proxysaur_bindings::http::request::HttpRequest;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::config::rewrite::RuleMatch;

fn serialize_uri<S: Serializer>(uri: &Uri, serializer: S) -> Result<S::Ok, S::Error> {
    let s: String = format!("{}", uri);
    serializer.serialize_str(&s)
}

fn deserialize_uri<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Uri, D::Error> {
    let str_val = String::deserialize(deserializer)?;
    let uri: Uri = str_val
        .parse()
        .map_err(|_e| serde::de::Error::custom("Invalid URI"))?;
    Ok(uri)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UrlDestination {
    #[serde(
        rename = "url",
        serialize_with = "serialize_uri",
        deserialize_with = "deserialize_uri"
    )]
    pub url: Uri,
    /// If specified, overwrite the path of the request, for example:
    /// https://google.com/path/a -> https://duckduckgo.com/path/a
    pub replace_path_and_query: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileDestination {
    pub path: PathBuf,
    /// Whether or not to interpret root paths as index.{content_type}
    pub root_index: bool,
    /// If specified, overwrite the path of the request, for example:
    /// https://google.com/path/a -> /usr/local/google.com/path/a
    pub replace_path: bool,
    /// An optional string to append to the end of the file, ex. for ".html"
    /// https://google.com/path/a -> /usr/local/google.com/path/a.html
    pub file_suffix: Option<String>,
    /// The content type of the file
    pub content_type: String,
}

#[derive(thiserror::Error, Debug)]
pub enum FileDestinationError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl FileDestination {
    pub fn path_for_request(&self, req: &HttpRequest) -> PathBuf {
        let mut path = self.path.clone();

        if !self.replace_path {
            return path;
        }

        let req_path = if req.path.starts_with('/') {
            &req.path[1..]
        } else {
            req.path.as_str()
        };
        path = path.join(req_path);

        if self.root_index {
            if req.path.ends_with('/') {
                path = path.join("index");
            }
        }

        if let Some(suffix) = &self.file_suffix {
            if let Some(Some(file_name)) = path.file_name().map(|f| f.to_str()) {
                let mut new_file_name = String::from(file_name);
                new_file_name.push_str(&suffix);
                path.set_file_name(new_file_name);
            }
        }
        path
    }

    /// Returns a response with the file, if it exists and is readable.
    pub fn resp(&self, req: &HttpRequest) -> Result<Vec<u8>, FileDestinationError> {
        let path = self.path_for_request(&req);
        std::fs::read(path).map_err(FileDestinationError::from)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RedirectDestination {
    #[serde(rename = "file")]
    File(FileDestination),
    #[serde(rename = "url")]
    Url(UrlDestination),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestRedirect {
    #[serde(default = "default_when")]
    pub when: Vec<RuleMatch>,
    pub to: RedirectDestination,
}

impl RequestRedirect {
    pub fn request(&self, method: &str, path_and_query: &str) -> Option<HttpRequest> {
        if let RedirectDestination::Url(destination) = &self.to {
            let authority = match destination.url.authority().map(|a| a.as_str()) {
                Some(dest) => dest,
                None => {
                    return None;
                }
            };
            let scheme = match destination.url.scheme().map(|a| a.as_str()) {
                Some(scheme) => scheme,
                None => {
                    return None;
                }
            };
            let req = HttpRequest {
                path: path_and_query.into(),
                authority: authority.into(),
                host: authority.into(),
                scheme: scheme.into(),
                version: "HTTP/1.1".into(),
                headers: vec![],
                method: method.into(),
                body: vec![],
            };
            Some(req)
        } else {
            None
        }
    }

    pub fn should_redirect_request(&self, req: &HttpRequest) -> bool {
        self.when[..]
            .into_iter()
            .all(|when: &RuleMatch| when.matches(req))
    }

    pub fn redirect_request(&self, req: &mut HttpRequest) {
        if !self.should_redirect_request(req) {
            return;
        }

        match &self.to {
            RedirectDestination::File(_) => {}
            RedirectDestination::Url(dest) => {
                let (scheme, authority) = match (dest.url.scheme(), dest.url.authority()) {
                    (Some(scheme), Some(authority)) => (scheme, authority),
                    _ => {
                        return;
                    }
                };
                req.authority = authority.to_string();
                req.scheme = scheme.to_string();
                if !dest.replace_path_and_query {
                    req.path = "".into();
                }
            }
        }
    }
}

fn default_when() -> Vec<RuleMatch> {
    vec![]
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use http::Request;
//     use tempdir::TempDir;
//     use test_case::test_case;

//     fn test_redirect_to_remote() {
//         let redirect = RequestRedirect {
//             when: vec![],
//             to: RedirectDestination::Url(UrlDestination {
//                 url: "https://duckduckgo.com"
//                     .parse()
//                     .expect("should build the url"),
//                 replace_path_and_query: true,
//             }),
//         };
//         redirect.redirect_request(&mut request);
//         let uri = format!("{}", request.request.uri());
//         assert_eq!(uri, "https://duckduckgo.com/my/path?and=query");
//     }

//     #[test_case(
//         true,
//         false,
//         "/usr/local/www",
//         Some(".json".to_string()),
//         "https://google.com/search/api/3",
//         "/usr/local/www/search/api/3.json"
//         ; "rewrite relative path"
//     )]
//     #[test_case(
//         true,
//         true,
//         "/usr/local/www",
//         Some(".json".to_string()),
//         "https://google.com/search/api/",
//         "/usr/local/www/search/api/index.json"
//         ; "rewrite index.html"
//     )]
//     #[test_case(
//         false,
//         false,
//         "/usr/local/www/file.json",
//         None,
//         "https://google.com/search/api/",
//         "/usr/local/www/file.json"
//         ; "rewrite without replacing"
//     )]
//     fn tests_file_redirect_calculate_path(
//         replace_path: bool,
//         root_index: bool,
//         file_path: &str,
//         file_suffix: Option<String>,
//         req_path: &str,
//         expected_path: &str,
//     ) {
//         let dest = FileDestination {
//             replace_path,
//             root_index,
//             path: PathBuf::from(file_path),
//             file_suffix,
//             content_type: "application/json".into(),
//         };
//         let req = Request::builder()
//             .uri(req_path)
//             .body(Body::empty())
//             .expect("should build the request");
//         let new_path = dest.path_for_request(&req);

//         assert_eq!(new_path.as_os_str(), std::ffi::OsStr::new(expected_path));
//     }

//     #[test]
//     fn redirects_to_file() {
//         let dir = TempDir::new("redirect").expect("should build a temporary directory");
//         let file_path = dir.path().join("index.html");
//         std::fs::write(file_path, "<html><body><h1>hi</h1></body></html>")
//             .expect("should write file");

//         let dest = FileDestination {
//             path: dir.path().to_path_buf(),
//             replace_path: true,
//             root_index: true,
//             file_suffix: Some(".html".into()),
//             content_type: "text/html; charset=UTF-8".into(),
//         };
//         let req = Request::builder()
//             .uri("https://www.google.com")
//             .body(Body::empty())
//             .expect("Should build the request");

//         let resp = dest.resp(&req).expect("should fetch the response");
//         let (parts, body) = resp.into_parts();
//         let body_bytes = hyper::body::to_bytes(body).await.expect("should read body");
//         let resp_str = std::str::from_utf8(&body_bytes).expect("should serialize to string");
//         assert_eq!(resp_str, "<html><body><h1>hi</h1></body></html>");
//         let content_type_header = parts
//             .headers
//             .get("Content-Type")
//             .expect("should have the header")
//             .to_str()
//             .expect("should have a valid UTF-8 header");
//         assert_eq!(content_type_header, "text/html; charset=UTF-8")
//     }
// }
