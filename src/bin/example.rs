use std::io::Error;

use cec_linux::{CecOpcode, CecUserControlCode};
use http_cec_tv::{
    cec::CecDev,
    http::{Response, Server, Status},
};

// port for the http server
const PORT: u16 = 7777;
// the literal device number: /dev/cec<Number>
const DEVICE: u8 = 0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize a new client
    let cec = CecDev::new(DEVICE)?;
    // define a routing schema for HTTP 1.1:
    let routing = |route: &str, method: &str| -> Result<Response, Error> {
        match (method, route) {
            // simple commands
            ("GET", "/on") => cec.exec(CecOpcode::ImageViewOn).map(|_| Response::ok("on")),
            ("GET", "/off") => cec.exec(CecOpcode::Standby).map(|_| Response::ok("on")),

            // remote-like commands
            ("GET", "/volume/up") => cec
                .exec_user_control(CecUserControlCode::VolumeUp)
                .map(|_| Response::ok("volume up")),
            ("GET", "/volume/down") => cec
                .exec_user_control(CecUserControlCode::VolumeDown)
                .map(|_| Response::ok("volume down")),

            // multi step commands
            ("GET", "/hdmi/2") => (|| -> Result<(), Error> {
                cec.exec_user_control(CecUserControlCode::InputSelect)?;
                std::thread::sleep(std::time::Duration::from_millis(800));
                cec.exec_user_control(CecUserControlCode::Down)?;
                std::thread::sleep(std::time::Duration::from_millis(200));
                cec.exec_user_control(CecUserControlCode::Select)?;
                Ok(())
            })()
            .map(|_| Response::ok("on")),

            ("GET", "/status") => cec.get_power_status().map(Response::ok),

            // arbitrary responses
            ("GET", "/checkhealth") => Ok(Response::ok("healthy")),
            (_, _) => Ok(Response::new(Status::NotFound, "")),
        }
    };

    let server = Server::new(PORT, routing)?;

    server.serve();

    Ok(())
}
