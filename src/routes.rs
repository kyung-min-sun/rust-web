use std::{fs, net::TcpStream};

use crate::parsers::{
    json::JsonValue,
    request::HttpRequest,
    response::{self, HttpCode, HttpResponse},
};

pub fn handle_request(request: HttpRequest, stream: TcpStream) {
    let HttpRequest {
        method,
        uri,
        headers,
        body,
    } = request;

    match (method.as_str(), uri.as_str()) {
        ("GET", "/") => response::send_response(
            stream,
            hello_world(HttpRequest {
                method,
                uri,
                headers,
                body,
            }),
        ),
        ("POST", "/") => response::send_response(
            stream,
            test_post(HttpRequest {
                method,
                uri,
                headers,
                body,
            }),
        ),
        _ => response::send_response(
            stream,
            response::http_error(
                response::HttpCode::NotFound,
                &fs::read_to_string("./src/404.html").unwrap(),
            ),
        ),
    }
}

fn hello_world(_: HttpRequest) -> HttpResponse {
    HttpResponse {
        code: HttpCode::Ok,
        body: Box::new(fs::read_to_string("./src/hello.html").unwrap()),
    }
}

fn test_post(request: HttpRequest) -> HttpResponse {
    HttpResponse {
        code: HttpCode::Ok,
        body: Box::new(request.body.unwrap_or(JsonValue::Array(vec![]))),
    }
}
