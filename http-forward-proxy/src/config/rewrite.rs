use std::str::FromStr;

use crate::config::{deserialize_regex, serialize_regex};
use http::header::{HeaderName, HeaderValue};
use proxysaur_bindings::http::{request::HttpRequestResult as HttpRequest, response::HttpResponse};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Represents a String or a regular expression
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MatchValue {
    #[serde(rename = "exact")]
    Exact(String),
    #[serde(rename = "contains")]
    Contains(String),
    #[serde(
        serialize_with = "serialize_regex",
        deserialize_with = "deserialize_regex",
        rename = "regex"
    )]
    Regex(Regex),
}

impl Eq for MatchValue {}

impl PartialEq for MatchValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Exact(l0), Self::Exact(r0)) => l0 == r0,
            (Self::Contains(l0), Self::Contains(r0)) => l0 == r0,
            (Self::Regex(l0), Self::Regex(r0)) => l0.as_str() == r0.as_str(),
            _ => false,
        }
    }
}

impl MatchValue {
    pub fn matches(&self, value: &str) -> bool {
        match self {
            MatchValue::Exact(s) => value == s,
            MatchValue::Contains(s) => value.contains(s),
            MatchValue::Regex(regex) => regex.is_match(value),
        }
    }

    pub fn expand(&self, value: &str, replace_with: &str) -> String {
        match self {
            MatchValue::Exact(s) if s == value => replace_with.replace("$0", s),
            MatchValue::Contains(s) if value.contains(s) => {
                let replaced = replace_with.replace("$0", value);
                replaced.replace("$1", s)
            }
            MatchValue::Regex(regex) if regex.is_match(value) => {
                let mut into = String::new();
                if let Some(captures) = regex.captures(value) {
                    if captures.len() == 1 {
                        regex.replace_all(value, replace_with).into()
                    } else {
                        captures.expand(replace_with, &mut into);
                        into
                    }
                } else {
                    "".into()
                }
            }
            _ => "".into(),
        }
    }
}

#[cfg(test)]
mod test_match_value {
    use super::*;

    #[test]
    fn expand_exact() {
        let value = MatchValue::Exact("exactly this and nothing else".into());
        let template = "matched this: $0";
        let expanded = value.expand("exactly this and nothing else", template);
        assert_eq!("matched this: exactly this and nothing else", expanded);
    }

    #[test]
    fn expand_contains() {
        let value = MatchValue::Contains("exactly this".into());
        let template = "matched this: $1 in this: $0";
        let expanded = value.expand("exactly this and nothing else", template);
        assert_eq!(
            "matched this: exactly this in this: exactly this and nothing else",
            expanded
        );
    }

    #[test]
    fn expand_regex_named() {
        let regex = Regex::new("/api/v1/(?P<path>[A-Za-z0-9]+)/(?P<slug>[A-Za-z]+)")
            .expect("should compile the regex");
        let value = MatchValue::Regex(regex);
        let template = "matched path: $path and slug: $slug";
        let expanded = value.expand("/api/v1/resource/book", template);
        assert_eq!("matched path: resource and slug: book", expanded);
    }

    #[test]
    fn expand_regex_missing() {
        let regex =
            Regex::new("/api/v1/([A-Za-z0-9]+)/([A-Za-z]+)").expect("should compile the regex");
        let value = MatchValue::Regex(regex);
        let template = "matched path: $1 and slug: $2";
        let expanded = value.expand("/api/v2/resource/book", template);
        assert_eq!("", expanded);
    }

    #[test]
    fn expand_regex_no_groups() {
        let regex = Regex::new("v[0-5]").expect("should compile the regex");
        let value = MatchValue::Regex(regex);
        let template = "v8";
        let expanded = value.expand("/api/v2/resource/v3/book", template);
        assert_eq!("/api/v8/resource/v8/book", expanded);
    }
}

/// Where we specify the rewrite
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Rewrite {
    Header(HeaderRewrite),
    Body(BodyRewrite),
    Status(StatusRewrite),
}

impl Rewrite {
    pub fn rewrite_req(&self, mut req: HttpRequest) -> HttpRequest {
        match self {
            Rewrite::Body(rewrite) => {
                let replace = rewrite.replace_with.clone();
                req.body = replace;
                let inserted = match req
                    .headers
                    .iter_mut()
                    .find(|(h, _v)| h == http::header::CONTENT_LENGTH.as_str())
                {
                    Some(mut header) => {
                        header.1 = req.body.len().to_string();
                        true
                    }
                    None => false,
                };
                if !inserted {
                    req.headers.push((
                        http::header::CONTENT_LENGTH.to_string(),
                        req.body.len().to_string(),
                    ));
                }
                req
            }
            Rewrite::Header(rewrite) => {
                rewrite.do_rewrite(&mut req.headers);
                req
            }
            Rewrite::Status(_) => req,
        }
    }

    pub fn rewrite_resp(&self, resp: &mut HttpResponse) {
        match self {
            Rewrite::Status(rewrite) => {
                let status = resp.status.to_string();
                if rewrite.status.matches(status.as_str()) {
                    let new_status: Result<u16, _> = rewrite
                        .status
                        .expand(status.as_str(), &rewrite.new_status)
                        .parse();
                    if let Ok(new_status) = new_status {
                        resp.status = new_status;
                    }
                }
            }
            Rewrite::Body(rewrite) => {
                let replace = rewrite.replace_with.clone();
                resp.body = replace;
                let inserted = match resp
                    .headers
                    .iter_mut()
                    .find(|(h, _v)| h == http::header::CONTENT_LENGTH.as_str())
                {
                    Some(mut header) => {
                        header.1 = resp.body.len().to_string();
                        true
                    }
                    None => false,
                };
                if !inserted {
                    resp.headers.push((
                        http::header::CONTENT_LENGTH.to_string(),
                        resp.body.len().to_string(),
                    ));
                }
            }
            Rewrite::Header(rewrite) => {
                rewrite.do_rewrite(&mut resp.headers);
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusRewrite {
    pub(crate) status: MatchValue,
    pub(crate) new_status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BodyRewrite {
    #[serde(
        serialize_with = "serialize_replace",
        deserialize_with = "deserialize_replace"
    )]
    pub(crate) replace_with: Vec<u8>,
}

fn serialize_replace<S: Serializer>(replace_with: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    let val =
        std::str::from_utf8(replace_with).map_err(|_| serde::ser::Error::custom("UTF-8 error"))?;
    serializer.serialize_str(val)
}

fn deserialize_replace<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
    let str_val = String::deserialize(deserializer)?;
    Ok(str_val.as_bytes().to_vec())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeaderMatch {
    pub(crate) header_name: MatchValue,
    pub(crate) header_value: MatchValue,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeaderRewrite {
    #[serde(rename = "match")]
    pub(crate) header_match: HeaderMatch,
    pub(crate) new_header_name: String,
    pub(crate) new_header_value: String,
}

impl HeaderRewrite {
    pub fn do_rewrite(&self, headers: &mut [(String, String)]) {
        let matching_header = headers.iter().enumerate().find(|(_idx, (name, value))| {
            self.header_match
                .header_name
                .matches(name.to_lowercase().as_str())
                && self.header_match.header_value.matches(value)
        });

        if let Some((idx, (name, value))) = matching_header {
            let new_header_name = self
                .header_match
                .header_name
                .expand(name, &self.new_header_name);
            let new_header_value = self
                .header_match
                .header_value
                .expand(value, &self.new_header_value);

            let key = HeaderName::from_str(new_header_name.as_str());
            let value = HeaderValue::from_str(new_header_value.as_str());

            if let (Ok(_key), Ok(_value)) = (key, value) {
                headers[idx] = (new_header_name, new_header_value);
            }
        }
    }
}

/// Matches on either plaintext or a regular expression
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuleMatch {
    #[serde(rename = "path")]
    PathMatch(MatchValue),
    #[serde(rename = "header")]
    HeaderMatch(HeaderMatch),
}

impl RuleMatch {
    pub fn matches(&self, req: &HttpRequest) -> bool {
        match self {
            RuleMatch::PathMatch(path) => path.matches(&req.path),
            RuleMatch::HeaderMatch(HeaderMatch {
                header_name,
                header_value,
            }) => {
                let matched_header = req
                    .headers
                    .iter()
                    .filter_map(|(name, value)| {
                        if header_name.matches(name.as_str()) {
                            Some(value.as_str())
                        } else {
                            None
                        }
                    })
                    .next();
                if let Some(value) = matched_header {
                    header_value.matches(value)
                } else {
                    false
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResponseRewrite {
    /// when condition(s) trigger a rewrite
    pub when: Vec<RuleMatch>,
    pub rewrite: Rewrite,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestRewrite {
    /// when condition(s) trigger a rewrite
    pub when: Vec<RuleMatch>,
    pub rewrite: Rewrite,
}

impl RequestRewrite {
    pub fn should_rewrite_request(&self, req: &HttpRequest) -> bool {
        self.when[..]
            .iter()
            .all(|when: &RuleMatch| when.matches(req))
    }

    pub fn rewrite(&self, req: HttpRequest) -> HttpRequest {
        self.rewrite.rewrite_req(req)
    }
}

#[cfg(test)]
mod request_rewrite_tests {
    use super::*;

    #[test]
    fn request_header_rewrite() {
        let rewrite = RequestRewrite {
            when: vec![RuleMatch::PathMatch(MatchValue::Exact("/".into()))],
            rewrite: Rewrite::Header(HeaderRewrite {
                header_match: HeaderMatch {
                    header_name: MatchValue::Exact("access-control-allow-origin".into()),
                    header_value: MatchValue::Contains("".into()),
                },
                new_header_name: "$0".into(),
                new_header_value: "*".into(),
            }),
        };
        let req = HttpRequest {
            path: "/".into(),
            authority: "foo.com".into(),
            host: "foo.com".into(),
            scheme: "https".into(),
            version: "HTTP/1.1".into(),
            headers: vec![(
                http::header::ACCESS_CONTROL_ALLOW_ORIGIN.to_string(),
                "https://foo.com".to_string(),
            )],
            method: "GET".into(),
            body: vec![],
        };
        assert!(rewrite.should_rewrite_request(&req));
        let new_req = rewrite.rewrite(req);
        let new_value = new_req
            .headers
            .iter()
            .find(|(h, _v)| h == http::header::ACCESS_CONTROL_ALLOW_ORIGIN.as_str())
            .expect("should have a header");
        assert_eq!(new_value.1, "*");
    }

    #[test]
    fn request_body_rewrite() {
        let rewrite = RequestRewrite {
            when: vec![RuleMatch::PathMatch(MatchValue::Exact("/".into()))],
            rewrite: Rewrite::Body(BodyRewrite {
                replace_with: "hey!".into(),
            }),
        };
        let req = HttpRequest {
            path: "/".into(),
            authority: "foo.com".into(),
            host: "foo.com".into(),
            scheme: "https".into(),
            version: "HTTP/1.1".into(),
            headers: vec![(
                http::header::ACCESS_CONTROL_ALLOW_ORIGIN.to_string(),
                "https://foo.com".to_string(),
            )],
            method: "GET".into(),
            body: vec![],
        };
        assert!(rewrite.should_rewrite_request(&req));
        let new_req = rewrite.rewrite(req);
        let content_length_value = new_req
            .headers
            .iter()
            .find(|(h, _v)| h == http::header::CONTENT_LENGTH.as_str())
            .expect("should have a content length");
        let content_length: usize = content_length_value
            .1
            .parse()
            .expect("should parse to a number");
        assert_eq!(content_length, 4);
    }
}

impl ResponseRewrite {
    /// Exists because typically the hyper client will consume the request
    pub fn should_rewrite_response(&self, req: &HttpRequest) -> bool {
        self.when[..]
            .iter()
            .all(|when: &RuleMatch| when.matches(req))
    }

    pub fn rewrite(&self, resp: &mut HttpResponse) {
        self.rewrite.rewrite_resp(resp);
    }
}

#[cfg(test)]
mod response_rewrite_tests {
    use super::*;

    #[test]
    fn response_status_rewrite() {
        let rewrite = ResponseRewrite {
            when: vec![RuleMatch::PathMatch(MatchValue::Exact("/".into()))],
            rewrite: Rewrite::Status(StatusRewrite {
                status: MatchValue::Exact("303".into()),
                new_status: "200".into(),
            }),
        };
        let req = HttpRequest {
            path: "/".into(),
            authority: "foo.com".into(),
            host: "foo.com".into(),
            scheme: "https".into(),
            version: "HTTP/1.1".into(),
            headers: vec![(
                http::header::ACCESS_CONTROL_ALLOW_ORIGIN.to_string(),
                "https://foo.com".to_string(),
            )],
            method: "GET".into(),
            body: vec![],
        };
        let mut resp = HttpResponse {
            headers: vec![],
            status: http::status::StatusCode::SEE_OTHER.as_u16(),
            body: vec![],
            request_path: req.path.clone(),
            request_authority: req.authority.clone(),
            request_host: req.host.clone(),
            request_scheme: req.scheme.clone(),
            request_version: req.version.clone(),
            request_headers: vec![],
            request_method: "GET".into(),
        };
        assert!(rewrite.should_rewrite_response(&req));
        rewrite.rewrite(&mut resp);
        assert_eq!(resp.status, http::StatusCode::OK.as_u16());
    }

    #[test]
    fn response_body_rewrite() {
        let rewrite = ResponseRewrite {
            when: vec![RuleMatch::PathMatch(MatchValue::Exact("/".into()))],
            rewrite: Rewrite::Body(BodyRewrite {
                replace_with: "hey!".into(),
            }),
        };
        let req = HttpRequest {
            path: "/".into(),
            authority: "foo.com".into(),
            host: "foo.com".into(),
            scheme: "https".into(),
            version: "HTTP/1.1".into(),
            headers: vec![(
                http::header::ACCESS_CONTROL_ALLOW_ORIGIN.to_string(),
                "https://foo.com".to_string(),
            )],
            method: "GET".into(),
            body: vec![],
        };
        let mut resp = HttpResponse {
            headers: vec![],
            status: http::status::StatusCode::SEE_OTHER.as_u16(),
            body: vec![],
            request_path: req.path.clone(),
            request_authority: req.authority.clone(),
            request_host: req.host.clone(),
            request_scheme: req.scheme.clone(),
            request_version: req.version.clone(),
            request_headers: vec![],
            request_method: "GET".into(),
        };
        assert!(rewrite.should_rewrite_response(&req));
        rewrite.rewrite(&mut resp);

        let content_length_value = resp
            .headers
            .iter()
            .find(|(h, _v)| h == http::header::CONTENT_LENGTH.as_str())
            .expect("should have a content length");
        let content_length: usize = content_length_value
            .1
            .parse()
            .expect("should parse to a number");
        assert_eq!(content_length, 4);
    }

    #[test]
    fn response_header_rewrite() {
        let regex = Regex::new("Bearer (?P<token>[0-9A-Za-z]+)").expect("should compile the regex");
        let rewrite = ResponseRewrite {
            when: vec![RuleMatch::PathMatch(MatchValue::Exact("/".into()))],
            rewrite: Rewrite::Header(HeaderRewrite {
                header_match: HeaderMatch {
                    header_name: MatchValue::Exact("x-my-header".into()),
                    header_value: MatchValue::Regex(regex),
                },
                new_header_name: "$0".into(),
                new_header_value: "Basic $token".into(),
            }),
        };
        let req = HttpRequest {
            path: "/".into(),
            authority: "foo.com".into(),
            host: "foo.com".into(),
            scheme: "https".into(),
            version: "HTTP/1.1".into(),
            headers: vec![(
                http::header::ACCESS_CONTROL_ALLOW_ORIGIN.to_string(),
                "https://foo.com".to_string(),
            )],
            method: "GET".into(),
            body: vec![],
        };
        let mut resp = HttpResponse {
            headers: vec![("x-my-header".into(), "Bearer abcd1234".into())],
            status: http::status::StatusCode::SEE_OTHER.as_u16(),
            body: vec![],
            request_path: req.path.clone(),
            request_authority: req.authority.clone(),
            request_host: req.host.clone(),
            request_scheme: req.scheme.clone(),
            request_version: req.version.clone(),
            request_headers: vec![],
            request_method: "GET".into(),
        };

        assert!(rewrite.should_rewrite_response(&req));
        rewrite.rewrite(&mut resp);
        let rewritten_header = resp
            .headers
            .iter()
            .find(|(h, _v)| h == "x-my-header")
            .expect("X-My-Header should exist");
        let rewritten_header_text = &rewritten_header.1;
        assert_eq!(rewritten_header_text, "Basic abcd1234");
    }
}
