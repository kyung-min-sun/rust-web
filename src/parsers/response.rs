use std::{io::Write, net::TcpStream};

use super::json::JsonValue;

pub enum HttpCode {
    Ok,
    BadRequest,
    NotFound,
    UnknownError,
}

impl HttpCode {
    fn value(&self) -> i32 {
        match &self {
            Self::Ok => 200,
            Self::BadRequest => 400,
            Self::NotFound => 404,
            _ => 300,
        }
    }
}

impl ToString for HttpCode {
    fn to_string(&self) -> String {
        match &self {
            Self::Ok => "Ok".to_string(),
            Self::BadRequest => "BAD REQUEST".to_string(),
            Self::NotFound => "NOT FOUND".to_string(),
            _ => "".to_string(),
        }
    }
}

pub struct HttpResponse {
    pub code: HttpCode,
    pub body: Box<dyn ToString>,
}

pub fn http_error(code: HttpCode, error_message: &str) -> HttpResponse {
    match code {
        HttpCode::BadRequest => HttpResponse {
            code: HttpCode::BadRequest,
            body: Box::new(JsonValue::String(error_message.to_string())),
        },
        HttpCode::NotFound => HttpResponse {
            code: HttpCode::NotFound,
            body: Box::new(JsonValue::String(error_message.to_string())),
        },
        _ => HttpResponse {
            code: HttpCode::UnknownError,
            body: Box::new(JsonValue::String(error_message.to_string())),
        },
    }
}

pub fn send_response(mut stream: TcpStream, response: HttpResponse) {
    let status_line = format!(
        "HTTP/1.1 {} {}\r\n\r\n",
        response.code.value(),
        response.code.to_string()
    );
    let response = format!("{status_line}\r\n{}", response.body.to_string());
    stream.write_all(response.as_bytes()).unwrap();
}
