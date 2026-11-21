use crate::{error, info};

use std::{
    fmt::Display,
    io::{BufRead, BufReader, Result, Write},
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
    reason_phrase: &'static str,
    body: String,
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
            self.status_code,
            self.reason_phrase,
            self.body.len(),
            self.body
        )
    }
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
            reason_phrase,
            status_code,
            body: body.to_string(),
        }
    }
}

type RequestHandler<'a> = Box<dyn 'a + Fn(&str, &str) -> Result<Response>>;

pub struct Server<'a> {
    listener: TcpListener,
    request_handler: RequestHandler<'a>,
}

impl<'a> Server<'a> {
    pub fn new(port: u16, handler: impl Fn(&str, &str) -> Result<Response> + 'a) -> Result<Self> {
        let ip = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(ip)?;
        Ok(Self {
            listener,
            request_handler: Box::new(handler),
        })
    }

    pub fn serve(&self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let response = self.routing(&stream);
                    if let Err(err) = stream.write_all(response.to_string().as_bytes()) {
                        error!("failed to send response: {}", err)
                    }
                }
                Err(err) => error!("connection failed: {}", err),
            };
        }
    }

    fn validate_request(&self, stream: &TcpStream) -> std::result::Result<(String, String), &str> {
        let reader = BufReader::new(stream);
        let header = reader.lines().next();
        match header {
            Some(Ok(header)) => {
                let mut entries = header.split_whitespace();
                match (entries.next(), entries.next()) {
                    (Some(method), Some(route)) => Ok((route.to_string(), method.to_string())),
                    (None, _) => Err("request is missing method type!"),
                    (_, None) => Err("request is missing route!"),
                }
            }
            Some(Err(_)) => Err("Bad request (I/O error)"),
            None => Err("Missing header!"),
        }
    }

    fn routing(&self, stream: &TcpStream) -> Response {
        match self.validate_request(stream) {
            Ok((route, method)) => {
                info!("request: {} {}", method, route);
                match (self.request_handler)(&route, &method) {
                    Ok(response) => response,
                    Err(error) => Response::error(&error.to_string()),
                }
            }
            Err(msg) => Response::bad(msg),
        }
    }
}
