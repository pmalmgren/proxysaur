use proxysaur_bindings::http::response::{self, HttpResponse};

fn main() {
    let response: HttpResponse = response::http_response_get().expect("should get the response");

    if response.status == 200 {
        response::http_response_set_status(500).expect("should set the status");
        response::http_response_set_body("broken!".as_bytes()).expect("should set the body");
    }
}
