use proxysaur_bindings::http::pre_request::{self, HttpPreRequest, ProxyMode};

fn main() {
    let request: HttpPreRequest = pre_request::http_request_get();

    if request.authority == "localhost:8000" || request.host == "petermalmgren.com" {
        pre_request::http_set_proxy_mode(ProxyMode::Intercept);
    }
}
