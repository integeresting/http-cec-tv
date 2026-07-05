use crate::{error, info};

use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

pub enum Status {
    OK,
    BadRequest,
    NotFound,
    InternalServerError,
}

pub struct Response {
    status_code: u16,
    reason_phrase: String,
    body: String,
}

impl Response {
    pub fn error(body: &str) -> Self {
        Response::new(Status::InternalServerError, body)
    }

    pub fn bad(body: &str) -> Self {
        Response::new(Status::BadRequest, body)
    }

    pub fn ok(body: &str) -> Self {
        Response::new(Status::OK, body)
    }

    pub fn new(status: Status, body: &str) -> Self {
        let (status_code, reason_phrase) = match status {
            Status::OK => (200, "OK"),
            Status::BadRequest => (400, "Bad Request"),
            Status::NotFound => (401, "Not Found"),
            Status::InternalServerError => (500, "Internal Server Error"),
        };
        Response {
            reason_phrase: reason_phrase.to_string(),
            status_code,
            body: body.to_string(),
        }
    }
    pub fn to_string(&self) -> String {
        format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
            self.status_code,
            self.reason_phrase,
            self.body.len(),
            self.body
        )
        .to_string()
    }
}

pub struct Server<'a> {
    listener: TcpListener,
    request_handler: Box<dyn 'a + Fn(String, String) -> Response>,
}

impl<'a> Server<'a> {
    pub fn new(port: u16, handler: impl Fn(String, String) -> Response + 'a) -> Self {
        let ip = format!("0.0.0.0:{}", port);
        let listener = match TcpListener::bind(ip) {
            Ok(listener) => listener,
            Err(err) => {
                error!("failed to start server: {}", err);
                std::process::exit(1);
            }
        };
        Self {
            listener,
            request_handler: Box::new(handler),
        }
    }

    pub fn serve(&self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let response = self.routing(&stream);
                    match stream.write_all(response.to_string().as_bytes()) {
                        Err(err) => error!("failed to send response: {}", err),
                        Ok(_) => (),
                    }
                }
                Err(err) => error!("connection failed: {}", err),
            };
        }
    }

    fn validate_request(&self, stream: &TcpStream) -> Result<(String, String), String> {
        let reader = BufReader::new(stream);
        let header = reader.lines().next();
        match header {
            Some(Ok(header)) => {
                let mut entries = header.split_whitespace();
                match (entries.next(), entries.next()) {
                    (Some(method), Some(route)) => Ok((route.to_string(), method.to_string())),
                    (None, _) => Err("request is missing method type!".to_string()),
                    (_, None) => Err("request is missing route!".to_string()),
                }
            }
            Some(Err(err)) => Err(format!("invalid request: {}", err)),
            None => Err("Missing header!".to_string()),
        }
    }

    fn routing(&self, stream: &TcpStream) -> Response {
        match self.validate_request(stream) {
            Ok((route, method)) => {
                info!("request: {} {}", method, route);
                (self.request_handler)(route, method)
            }
            Err(msg) => Response::bad(&msg),
        }
    }
}
