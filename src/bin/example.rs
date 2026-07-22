use cec_linux::{CecOpcode, CecUserControlCode};
use http_tv::{
    cec::CecDev,
    http::{Response, Server, Status},
};

// port for the http server
const PORT: u16 = 7777;
// the literal device number: /dev/cec<Number>
const DEVICE: u8 = 0;

fn main() {
    // initialize a new client
    let cec = CecDev::new(DEVICE);
    // define a routing schema for HTTP 1.1:
    let routing = |route: String, method: String| -> Response {
        match (method.as_str(), route.as_str()) {
            // simple commands
            ("GET", "/on") => match cec.exec(CecOpcode::ImageViewOn) {
                Ok(_) => Response::ok("on"),
                Err(e) => Response::error(&e),
            },
            ("GET", "/off") => match cec.exec(CecOpcode::Standby) {
                Ok(_) => Response::ok("off"),
                Err(e) => Response::error(&e),
            },

            // remote-like commands
            ("GET", "/volume/up") => {
                match cec.exec_user_control(CecUserControlCode::VolumeUp) {
                    Ok(_) => Response::ok("volume up"),
                    Err(e) => Response::error(&e),
                }
            }
            ("GET", "/volume/down") => {
                match cec.exec_user_control(CecUserControlCode::VolumeDown) {
                    Ok(_) => Response::ok("volume down"),
                    Err(e) => Response::error(&e),
                }
            }

            // multi step commands
            ("GET", "/hdmi/2") => match (|| -> Result<(), String> {
                cec.exec_user_control(CecUserControlCode::InputSelect)?;
                std::thread::sleep(std::time::Duration::from_millis(800));
                cec.exec_user_control(CecUserControlCode::Down)?;
                std::thread::sleep(std::time::Duration::from_millis(200));
                cec.exec_user_control(CecUserControlCode::Select)?;
                Ok(())
            })() {
                Ok(_) => Response::ok("on"),
                Err(e) => Response::error(&e),
            },

            ("GET", "/status") => match cec.get_power_status() {
                Ok(msg) => Response::ok(&msg),
                Err(e) => Response::error(&e),
            },

            // arbitrary responses
            ("GET", "/checkhealth") => Response::new(Status::OK, "healthy"),
            (_, _) => Response::new(Status::NotFound, ""),
        }
    };

    let server = Server::new(PORT, routing);
    server.serve();
}
