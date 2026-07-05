#[macro_export]
macro_rules! _log {
    ($lvl: literal, $msg: expr $(, $args: expr)*) => {
        println!(
            "[{}][{}][{}:{}] {}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            $lvl,
            file!(),
            line!(),
            format_args!($msg $(, $args)*),
        )
    };
}

#[macro_export]
macro_rules! info {
    ($msg: expr $(, $args: expr)*) => {
        $crate::_log!("INFO", $msg $(, $args)*)
    };
}

#[macro_export]
macro_rules! warn {
    ($msg: expr $(, $args: expr)*) => {
        $crate::_log!("INFO", $msg $(, $args)*)
    };
}

#[macro_export]
macro_rules! error {
    ($msg: expr $(, $args: expr)*) => {
        $crate::_log!("INFO", $msg $(, $args)*)
    };
}
