use crate::config::{
    redirect::RequestRedirect,
    rewrite::{RequestRewrite, ResponseRewrite},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    scheme: String,
    #[serde(default = "default_resp_rewrite")]
    pub response_rewrites: Vec<ResponseRewrite>,
    #[serde(default = "default_req_rewrite")]
    pub request_rewrites: Vec<RequestRewrite>,
    pub redirect: Option<RequestRedirect>,
}

fn default_resp_rewrite() -> Vec<ResponseRewrite> {
    vec![]
}

fn default_req_rewrite() -> Vec<RequestRewrite> {
    vec![]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptConfig {
    hosts: HashMap<String, HostConfig>,
}

impl InterceptConfig {
    /// Fetches the configuration for interception & rewriting associated with the host.
    pub fn host_config(&self, hostname: &str) -> Option<&HostConfig> {
        self.hosts.get(hostname)
    }

    pub fn should_intercept(&self, hostname: &str) -> bool {
        self.hosts.contains_key(hostname)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::rewrite::{
        HeaderMatch, HeaderRewrite, MatchValue, Rewrite, RuleMatch, StatusRewrite,
    };

    const CONFIG: &'static str = r#"
    hosts:
      test3.com:
        scheme: https
        redirect:
          to:
            url:
              url: https://duckduckgo.com
              replace_path_and_query: true
      test2.com:
        scheme: https
        redirect:
          to:
            file:
              path: /usr/local/www
              root_index: true
              replace_path: true
              file_suffix: .html
              content_type: text/html; charset=UTF-8
      test.com:
        scheme: https
        response_rewrites:
          - when:
              - path:
                  exact: /
            rewrite:
              status:
                exact: "303"
              new_status: "200"
        request_rewrites:
          - when:
              - path:
                  exact: /
            rewrite:
              match:
                header_name:
                  exact: access-control-allow-origin
                header_value:
                  contains: ""
              new_header_name: $0
              new_header_value: "*"
    "#;

    #[test]
    fn test_deserialize() {
        let mut hosts = HashMap::new();
        let req_rewrite = RequestRewrite {
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
        let resp_rewrite = ResponseRewrite {
            when: vec![RuleMatch::PathMatch(MatchValue::Exact("/".into()))],
            rewrite: Rewrite::Status(StatusRewrite {
                status: MatchValue::Exact("303".into()),
                new_status: "200".into(),
            }),
        };
        hosts.insert(
            "test.com".into(),
            HostConfig {
                scheme: "https".into(),
                response_rewrites: vec![resp_rewrite],
                request_rewrites: vec![req_rewrite],
                redirect: None,
            },
        );
        let config = InterceptConfig { hosts };
        serde_yaml::to_string(&config).expect("should deserialize");
    }

    #[test]
    fn test_serialize() {
        let config: InterceptConfig = serde_yaml::from_str(&CONFIG).expect("should serialize");
        let host = config
            .hosts
            .get("test.com".into())
            .expect("should contain the key");
        assert_eq!(host.response_rewrites.len(), 1);
    }
}
