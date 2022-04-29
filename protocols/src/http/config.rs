use config::Proxy;
use proxysaur_wit_bindings::config::config::Config;

pub(crate) struct ProxyConfig {
    pub(crate) proxy: Proxy,
    pub(crate) error: String
}

impl Config for ProxyConfig {
    fn get_config_data(&mut self) -> Vec<u8> {
        match &self.proxy.wasi_configuration_bytes {
            Some(bytes) => bytes.to_vec(),
            None => vec![],
        }
    }

    fn set_invalid_data(&mut self, error: &str) {
        self.error = error.to_string();
    }
}
