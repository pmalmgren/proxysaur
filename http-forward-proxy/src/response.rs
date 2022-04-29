mod config;

use config::intercept::InterceptConfig;
use config::rewrite::ResponseRewrite;
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

    let response = http::response::http_response_get().expect("should get the response");

    let host_config = match config.host_config(response.request_host.as_str()) {
        Some(config) => config,
        None => {
            proxysaur_config::set_invalid_data("No host configuration found.");
            return;
        }
    };

    let response_rewrites = &host_config.response_rewrites;
    let resp_rewrites: Vec<&ResponseRewrite> = response_rewrites
        .into_iter()
        .filter_map(|rewrite| {
            if rewrite.should_rewrite_response(&response.request) {
                Some(rewrite)
            } else {
                None
            }
        })
        .collect();
}
