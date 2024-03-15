mod infra;
mod parsers;
mod routes;

use infra::thread_pool::ThreadPool;
use parsers::{request, response};

use std::net::{TcpListener, TcpStream};

pub fn run() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    let pool = ThreadPool::new(None);
    for stream in listener.incoming() {
        pool.execute(|| {
            let stream = match stream {
                Ok(value) => value,
                Err(_) => return,
            };
            handle_connection(stream);
        })
    }
}

fn handle_connection(stream: TcpStream) {
    let (headers, body) = match request::split_request(&stream) {
        Some(value) => value,
        None => {
            return response::send_response(
                stream,
                response::http_error(response::HttpCode::BadRequest, "could not parse headers"),
            )
        }
    };

    let request = match request::parse_request(headers, body) {
        Ok(request) => request,
        Err(response) => return response::send_response(stream, response),
    };

    routes::handle_request(request, stream);
}
