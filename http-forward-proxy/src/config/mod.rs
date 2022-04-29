pub mod intercept;
pub mod redirect;
pub mod rewrite;

use std::{path::PathBuf, str::FromStr};

use http::uri::Uri;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize_uri<S: Serializer>(uri: &Uri, serializer: S) -> Result<S::Ok, S::Error> {
    let s: String = format!("{}", uri);
    serializer.serialize_str(&s)
}

pub fn deserialize_uri<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Uri, D::Error> {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RedirectDestination {
    #[serde(rename = "file")]
    File(FileDestination),
    #[serde(rename = "url")]
    Url(UrlDestination),
}

fn default_when() -> Vec<RuleMatch> {
    vec![]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestRedirect {
    #[serde(default = "default_when")]
    pub when: Vec<RuleMatch>,
    pub to: RedirectDestination,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Rewrite {
    Header(HeaderRewrite),
    Body(BodyRewrite),
    Status(StatusRewrite),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusRewrite {
    pub status: MatchValue,
    pub new_status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BodyRewrite {
    pub replace_with: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeaderRewrite {
    #[serde(rename = "match")]
    pub header_match: HeaderMatch,
    pub new_header_name: String,
    pub new_header_value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestRewrite {
    /// when condition(s) trigger a rewrite
    pub when: Vec<RuleMatch>,
    pub rewrite: Rewrite,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseRewrite {
    /// when condition(s) trigger a rewrite
    pub when: Vec<RuleMatch>,
    pub rewrite: Rewrite,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RuleMatch {
    #[serde(rename = "path")]
    PathMatch(MatchValue),
    #[serde(rename = "header")]
    HeaderMatch(HeaderMatch),
}

#[derive(Clone, Debug, Serialize)]
pub struct HostConfig {
    pub scheme: String,
    pub response_rewrites: Vec<ResponseRewrite>,
    pub request_rewrites: Vec<RequestRewrite>,
    pub redirect: Option<RequestRedirect>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeaderMatch {
    pub header_name: MatchValue,
    pub header_value: MatchValue,
}

pub fn serialize_regex<S: Serializer>(regex: &Regex, serializer: S) -> Result<S::Ok, S::Error> {
    let val = regex.as_str();
    serializer.serialize_str(val)
}

pub fn deserialize_regex<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Regex, D::Error> {
    let str_val = String::deserialize(deserializer)?;
    Regex::from_str(&str_val).map_err(serde::de::Error::custom)
}

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
