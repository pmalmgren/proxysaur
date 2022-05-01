mod config;

use config::intercept::InterceptConfig;
use config::rewrite::ResponseRewrite;
use proxysaur_bindings::{
    config as proxysaur_config,
    http::{self, request::HttpRequestResult},
};

fn main() {
    let config_data: Vec<u8> = proxysaur_config::get_config_data();
    let config: InterceptConfig = match serde_yaml::from_slice(&config_data) {
        Ok(config) => config,
        Err(err) => {
            let msg = err.to_string();
            proxysaur_config::set_invalid_data(&msg);
            return;
        }
    };

    let mut response = http::response::http_response_get().expect("should get the response");

    let host_config = match config.host_config(response.request_host.as_str()) {
        Some(config) => config,
        None => {
            proxysaur_config::set_invalid_data("No host configuration found.");
            return;
        }
    };

    let request = HttpRequestResult {
        path: response.request_path.clone(),
        authority: response.request_authority.clone(),
        host: response.request_host.clone(),
        scheme: response.request_scheme.clone(),
        version: response.request_version.clone(),
        method: response.request_method.clone(),
        body: vec![],
        headers: response.request_headers.clone(),
    };

    let response_rewrites = &host_config.response_rewrites;
    let resp_rewrites: Vec<&ResponseRewrite> = response_rewrites
        .iter()
        .filter(|rewrite| rewrite.should_rewrite_response(&request))
        .collect();

    for rewrite in resp_rewrites.iter() {
        rewrite.rewrite(&mut response);
    }
    let headers: Vec<(&str, &str)> = response
        .headers
        .iter()
        .map(|(h, v)| (h.as_str(), v.as_str()))
        .collect();

    let _res = http::response::http_response_set_body(&response.body);
    let _res = http::response::http_response_set_status(response.status);
    let _res = http::response::http_response_set_headers(headers.as_slice());
}
