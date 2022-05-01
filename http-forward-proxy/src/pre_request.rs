mod config;
use config::intercept::InterceptConfig;
use proxysaur_bindings::{config as proxysaur_config, http::pre_request::ProxyMode};

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

    let host = proxysaur_bindings::http::pre_request::http_request_get();
    if config.should_intercept(&host.host) {
        proxysaur_bindings::http::pre_request::http_set_proxy_mode(ProxyMode::Intercept);
    } else {
        proxysaur_bindings::http::pre_request::http_set_proxy_mode(ProxyMode::Pass);
    }
}
