use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use http_tv::{
    cec::{CecCommand, cec},
    http::{Response, Status},
};

const PORT: u16 = 3000;
const USAGE_MSG: &str = "USAGE:
    /on -> turn on tv
    /off -> put tv in standby
    /checkhealth -> test server status
";

fn main() {
    let ip = format!("127.0.0.1:{}", PORT);
    let listener = match TcpListener::bind(ip) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Failed to start server: {}", err);
            std::process::exit(1);
        }
    };

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let response = validate_request_format(&stream);
                println!("{}", response.to_string());
                match stream.write_all(response.to_string().as_bytes()) {
                    Err(err) => eprintln!("Failed to send response: {}", err),
                    Ok(_) => (),
                }
            }
            Err(err) => eprintln!("Connection failed: {}", err),
        };
    }
}

fn validate_request_format(stream: &TcpStream) -> Response {
    let reader = BufReader::new(stream);
    let header = reader.lines().next();
    match header {
        Some(Ok(header)) => {
            let mut entries = header.split_whitespace();
            match (entries.next(), entries.next()) {
                (Some(method), Some(route)) => routing(method, route),
                (None, _) => Response::bad("request is missing method type!"),
                (_, None) => Response::bad("request is missing route!"),
            }
        }
        Some(Err(err)) => Response::bad(&format!("invalid request: {}", err)),
        None => Response::bad("Missing header!"),
    }
}

fn routing(method: &str, route: &str) -> Response {
    match (method, route) {
        ("GET", "/on") => match cec(CecCommand::On) {
            Some(msg) => Response::ok(&msg),
            None => Response::error("Failed to execute!"),
        },
        ("GET", "/off") => match cec(CecCommand::Off) {
            Some(msg) => Response::ok(&msg),
            None => Response::error("Failed to execute!"),
        },
        ("GET", "/") => Response::new(Status::OK, USAGE_MSG),
        ("GET", "/checkhealth") => Response::new(Status::OK, "healthy"),
        (_, _) => Response::new(Status::NotFound, ""),
    }
}
