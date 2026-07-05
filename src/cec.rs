use crate::info;
use cec_linux::{
    CecDevice, CecLogAddrType, CecLogAddrs, CecLogicalAddress, CecModeFollower, CecModeInitiator,
    CecOpcode, CecPrimDevType, VendorID, Version,
};
pub struct CecDev {
    pub device: CecDevice,
    log_addr: CecLogicalAddress,
}

pub enum CecCommand {
    On,
    Off,
    Status,
}

impl CecDev {
    pub fn new(device_nr: u8) -> Self {
        let device_str = format!("/dev/cec{}", device_nr);
        let device = CecDevice::open(&device_str).expect("Could not open device: {DEVICE}");
        device
            .set_mode(CecModeInitiator::Send, CecModeFollower::All)
            .expect("could not set cec mode");
        let capabilities = device.get_capas();
        info!("started device: ${device_str} with capabilities: ${capabilities:?}");

        // clear existing logical addresses
        let log = CecLogAddrs::default();
        device
            .set_log(log)
            .expect("failed to reset logical address");

        let log = CecLogAddrs::new(
            VendorID::NONE,
            Version::V1_4,
            "pi4".to_string().try_into().unwrap(),
            &[CecPrimDevType::RECORD],
            &[CecLogAddrType::RECORD],
        );
        device.set_log(log).expect("failed to set logical address");
        CecDev {
            log_addr: CecLogicalAddress::Record1,
            device,
        }
    }

    pub fn exec(&self, command: CecCommand) -> Result<String, String> {
        let (from, to) = (self.log_addr, CecLogicalAddress::Tv);

        match command {
            CecCommand::Status => return self.get_power_status(),
            _ => (),
        }

        let op_code = match command {
            CecCommand::On => CecOpcode::ImageViewOn,
            CecCommand::Off => CecOpcode::Standby,
            _ => panic!("covered before"),
        };

        let res = match self.device.transmit(from, to, op_code) {
            Ok(_) => Ok(match op_code {
                CecOpcode::ImageViewOn => "On",
                CecOpcode::Standby => "Off",
                _ => "Status unknown",
            }
            .to_string()),
            Err(e) => Err(format!("transmitting opcode failed: {e:#?}").to_string()),
        };

        res
    }

    fn get_power_status(&self) -> Result<String, String> {
        let msg = self.device.request_data(
            self.log_addr,
            CecLogicalAddress::Tv,
            CecOpcode::GiveDevicePowerStatus,
            b"",
            CecOpcode::ReportPowerStatus,
        );
        match msg {
            Ok(s) => {
                let res = match s[0] {
                    0 => "on",
                    1 => "standby",
                    2 => "turning on",
                    3 => "turning off",
                    _ => "unknown status",
                };
                Ok(res.to_string())
            }
            Err(e) => Err(format!("failed to get power status: {:#?}", e).to_string()),
        }
    }
}
