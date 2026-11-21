use std::io::Result;

use crate::info;
use cec_linux::{
    CecDevice, CecLogAddrType, CecLogAddrs, CecLogicalAddress, CecModeFollower, CecModeInitiator,
    CecOpcode, CecPrimDevType, CecUserControlCode, VendorID, Version,
};
pub struct CecDev {
    pub device: CecDevice,
    log_addr: CecLogicalAddress,
}

impl CecDev {
    pub fn new(device_number: u8) -> Result<Self> {
        let device_path = format!("/dev/cec{}", device_number);
        let device = CecDevice::open(&device_path)?;
        device
            .set_mode(CecModeInitiator::Send, CecModeFollower::All)
            .expect("could not set cec mode");
        let capabilities = device.get_capas();
        info!("started device: ${device_path} with capabilities: ${capabilities:?}");

        // clear existing logical addresses
        let logical_address = CecLogAddrs::default();
        device
            .set_log(logical_address)
            .expect("failed to reset logical address");

        let logical_address = CecLogAddrs::new(
            VendorID::NONE,
            Version::V1_4,
            "pi4".to_string().try_into().unwrap(),
            &[CecPrimDevType::TUNER],
            &[CecLogAddrType::TUNER],
        );
        device
            .set_log(logical_address)
            .expect("failed to set logical address");
        Ok(CecDev {
            log_addr: CecLogicalAddress::Tuner1,
            device,
        })
    }

    pub fn exec(&self, command: CecOpcode) -> Result<()> {
        let (from, to) = (self.log_addr, CecLogicalAddress::Tv);
        self.device.transmit(from, to, command)
    }

    pub fn exec_user_control(&self, command: CecUserControlCode) -> Result<()> {
        let (from, to) = (self.log_addr, CecLogicalAddress::Tv);
        self.device
            .transmit_data(from, to, CecOpcode::UserControlPressed, &[command.into()])
    }

    pub fn get_power_status(&self) -> Result<&str> {
        let message = self.device.request_data(
            self.log_addr,
            CecLogicalAddress::Tv,
            CecOpcode::GiveDevicePowerStatus,
            b"",
            CecOpcode::ReportPowerStatus,
        );
        message.map(|payload| match payload[0] {
            0 => "on",
            1 => "standby",
            2 => "turning on",
            3 => "turning off",
            _ => "unknown status",
        })
    }
}
