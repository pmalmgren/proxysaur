use request::HttpRequest;

wit_bindgen_rust::import!("../../request.wit");

fn main() {
    let request: HttpRequest = request::http_request_get().expect("should get the request");

    if request.method.to_lowercase() == "get" {
        request::http_request_set_method("post").expect("should set the method");
        request::http_request_set_body("haha!".as_bytes()).expect("should set the body");
    }
}
