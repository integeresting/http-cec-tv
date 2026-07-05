use http_tv::{
    cec::{CecCommand, CecDev},
    http::{Response, Server, Status},
};

const PORT: u16 = 6969;
const DEVICE: u8 = 0;

const USAGE_MSG: &str = "USAGE:
    /on          -> turn on tv
    /off         -> put tv in standby
    /status      -> get current tv power status
    /checkhealth -> test server status
";

fn main() {
    let port = std::env::var("HTTP_TV_PORT").map_or(PORT, |v| {
        v.parse::<u16>()
            .expect("'HTTP_TV_PORT' should be a valid u16")
    });
    let device = std::env::var("HTTP_TV_DEVICE_NR").map_or(DEVICE, |v| {
        v.parse::<u8>()
            .expect("'HTTP_TV_DEVICE_NR' should be a valid u8")
    });

    let cec = CecDev::new(device);
    let routing = |route: String, method: String| -> Response {
        match (method.as_str(), route.as_str()) {
            ("GET", "/on") => match cec.exec(CecCommand::On) {
                Ok(msg) => Response::ok(&msg),
                Err(e) => Response::error(&e),
            },
            ("GET", "/off") => match cec.exec(CecCommand::Off) {
                Ok(msg) => Response::ok(&msg),
                Err(e) => Response::error(&e),
            },
            ("GET", "/status") => match cec.exec(CecCommand::Status) {
                Ok(msg) => Response::ok(&msg),
                Err(e) => Response::error(&e),
            },
            ("GET", "/") => Response::new(Status::OK, USAGE_MSG),
            ("GET", "/checkhealth") => Response::new(Status::OK, "healthy"),
            (_, _) => Response::new(Status::NotFound, ""),
        }
    };

    let server = Server::new(port, routing);
    server.serve();
}
