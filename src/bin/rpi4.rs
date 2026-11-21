use std::io::Error;

use cec_linux::{CecOpcode, CecUserControlCode};
use http_cec_tv::{
    cec::CecDev,
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

fn main() -> std::io::Result<()> {
    let port = std::env::var("HTTP_TV_PORT").map_or(PORT, |v| {
        v.parse::<u16>()
            .expect("'HTTP_TV_PORT' should be a valid u16")
    });
    let device = std::env::var("HTTP_TV_DEVICE_NR").map_or(DEVICE, |v| {
        v.parse::<u8>()
            .expect("'HTTP_TV_DEVICE_NR' should be a valid u8")
    });

    let cec = CecDev::new(device)?;
    let routing = |route: &str, method: &str| -> Result<Response, Error> {
        match (method, route) {
            ("GET", "/on") => (|| -> Result<(), Error> {
                cec.exec_user_control(CecUserControlCode::PowerOnFunction)?;
                std::thread::sleep(std::time::Duration::from_millis(1000));
                cec.exec_user_control(CecUserControlCode::InputSelect)?;
                std::thread::sleep(std::time::Duration::from_millis(800));
                cec.exec_user_control(CecUserControlCode::Down)?;
                std::thread::sleep(std::time::Duration::from_millis(200));
                cec.exec_user_control(CecUserControlCode::Select)?;
                Ok(())
            })()
            .map(|_| Response::ok("on")),

            ("GET", "/off") => cec.exec(CecOpcode::Standby).map(|_| Response::ok("off")),

            ("GET", "/volume/up") => cec
                .exec_user_control(CecUserControlCode::VolumeUp)
                .map(|_| Response::ok("volume up")),

            ("GET", "/volume/down") => cec
                .exec_user_control(CecUserControlCode::VolumeDown)
                .map(|_| Response::ok("volume down")),

            ("GET", "/status") => cec.get_power_status().map(Response::ok),

            ("GET", "/") => Ok(Response::ok(USAGE_MSG)),

            ("GET", "/checkhealth") => Ok(Response::ok("healthy")),

            (_, _) => Ok(Response::new(Status::NotFound, "")),
        }
    };

    let server = Server::new(port, routing)?;
    server.serve();
    Ok(())
}
