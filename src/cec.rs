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
            &[CecPrimDevType::TUNER],
            &[CecLogAddrType::TUNER],
        );
        device.set_log(log).expect("failed to set logical address");
        CecDev {
            log_addr: CecLogicalAddress::Tuner1,
            device,
        }
    }

    pub fn exec(&self, command: CecOpcode) -> Result<(), String> {
        let (from, to) = (self.log_addr, CecLogicalAddress::Tv);

        match self.device.transmit(from, to, command) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("transmitting opcode failed: {e:#?}").to_string()),
        }
    }

    pub fn exec_user_control(&self, command: CecUserControlCode) -> Result<(), String> {
        let (from, to) = (self.log_addr, CecLogicalAddress::Tv);

        match self
            .device
            .transmit_data(from, to, CecOpcode::UserControlPressed, &[command.into()])
        {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("transmitting user control command failed: {e:#?}").to_string()),
        }
    }

    pub fn get_power_status(&self) -> Result<String, String> {
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
