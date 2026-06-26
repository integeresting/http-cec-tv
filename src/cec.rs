pub enum CecCommand {
    On,
    Off,
}

pub fn cec(command: CecCommand) -> Option<String> {
    match command {
        CecCommand::On => Some("on".to_string()),
        CecCommand::Off => Some("off".to_string()),
    }
}
