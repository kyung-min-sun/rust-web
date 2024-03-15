use std::{
    collections::HashMap,
    io::{BufReader, Read},
    net::TcpStream,
};

use super::{
    json::{parse_json, JsonValue},
    response::{self, HttpResponse},
};

pub struct HttpRequest {
    pub method: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Option<JsonValue>,
}

pub fn split_request(stream: &TcpStream) -> Option<(Vec<String>, String)> {
    let buf_reader = BufReader::new(stream);

    let mut request_lines: Vec<String> = Vec::new();
    let mut char_sequence: Vec<u8> = Vec::new();
    let mut body_sequence: Vec<u8> = Vec::new();
    let mut content_length: usize = 0;

    for byte in buf_reader.bytes() {
        match content_length {
            0 => (),
            1.. => match byte {
                Ok(char) if body_sequence.len() == 0 && char.is_ascii_whitespace() => {
                    continue;
                }
                Ok(char) => {
                    body_sequence.push(char);
                    if body_sequence.len() == content_length {
                        break;
                    } else {
                        continue;
                    }
                }
                Err(_) => continue,
            },
        }

        let last_request_line = request_lines
            .last()
            .unwrap_or(&String::from(""))
            .trim()
            .to_lowercase();

        match &last_request_line {
            line if line.starts_with("content-length") => {
                content_length = match line.split_once(":") {
                    Some((_, length)) => match length.trim().parse() {
                        Ok(v @ 1..) => v,
                        Ok(0) => return Some((request_lines, String::from(""))),
                        Err(_) => return None,
                    },
                    None => return None,
                };
            }
            _ => (),
        }

        match byte {
            Ok(char) if char as char == '\n' => {
                match String::from_utf8(char_sequence.clone()) {
                    Ok(value) => request_lines.push(value),
                    Err(_) => return None,
                }
                char_sequence.clear();
            }
            Ok(char) => char_sequence.push(char),
            Err(_) => return None,
        };
    }

    let body_str = match String::from_utf8(body_sequence) {
        Ok(v) => v,
        Err(_) => String::from(""),
    };

    Some((request_lines, body_str))
}

pub fn parse_request(
    http_request_lines: Vec<String>,
    body: String,
) -> Result<HttpRequest, HttpResponse> {
    let parse_error =
        response::http_error(response::HttpCode::BadRequest, "could not parse request");

    let mut http_request_line_iter = http_request_lines.iter();
    let request_line = match http_request_line_iter.next() {
        Some(line) => line,
        None => return Err(parse_error),
    };

    let mut request_line_iter = request_line.split_whitespace();

    let method = match request_line_iter.next() {
        Some(method @ ("GET" | "POST" | "PATCH" | "DELETE")) => method,
        _ => return Err(parse_error),
    };

    let uri = request_line_iter.next();

    let header_vector: Vec<(String, String)> = http_request_line_iter
        .filter_map(|request_line| {
            let header_values: Vec<&str> = match request_line.find(":") {
                Some(_) => request_line.split(":").collect(),
                None => return None,
            };
            let header = header_values.get(0);
            let value = header_values.get(1);
            match (header, value) {
                (Some(header), Some(value)) => {
                    Some((header.trim().to_string(), value.trim().to_string()))
                }
                _ => None,
            }
        })
        .collect();

    let mut headers = HashMap::new();

    for (header, value) in header_vector {
        headers.insert(header, value);
    }

    match (method, uri) {
        (method, Some(uri)) => Ok(HttpRequest {
            method: method.to_string(),
            uri: uri.to_string(),
            headers,
            body: parse_json(&body),
        }),
        (_, _) => Err(parse_error),
    }
}
