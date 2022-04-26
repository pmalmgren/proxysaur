use pre_request::{HttpPreRequest, ProxyMode};
wit_bindgen_rust::import!("../../pre-request.wit");

fn main() {
    let request: HttpPreRequest = pre_request::http_request_get();

    if request.authority == "localhost:8000" {
        pre_request::http_set_proxy_mode(ProxyMode::Intercept);
    }
}
