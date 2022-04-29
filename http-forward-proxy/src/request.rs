mod config;
use config::intercept::InterceptConfig;
use proxysaur_bindings::{config as proxysaur_config, http};

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
    let mut request = http::request::http_request_get().expect("should fetch the request");
    let host_config = match config.host_config(request.host.as_str()) {
        Some(config) => config,
        None => {
            proxysaur_config::set_invalid_data("No host configuration found.");
            return;
        }
    };

    if let Some(redirect) = &host_config.redirect {
        redirect.redirect_request(&mut request);
    }

    for rewrite in &host_config.request_rewrites {
        if rewrite.should_rewrite_request(&request) {
            request = rewrite.rewrite(request);
        }
    }

    let headers: Vec<(&str, &str)> = request
        .headers
        .iter()
        .map(|(n, v)| (n.as_str(), v.as_str()))
        .collect();

    http::request::http_request_set(http::request::HttpRequestParam {
        authority: &request.authority,
        body: &request.body,
        headers: &headers,
        host: &request.host,
        method: &request.method,
        path: &request.path,
        scheme: &request.scheme,
        version: &request.version,
    });
}
