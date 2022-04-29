pub mod request {
    wit_bindgen_rust::import!("src/request.wit");

    pub use request::*;
}

pub mod response {
    wit_bindgen_rust::import!("src/response.wit");

    pub use response::*;
}

pub mod pre_request {
    wit_bindgen_rust::import!("src/pre-request.wit");

    pub use pre_request::*;
}
