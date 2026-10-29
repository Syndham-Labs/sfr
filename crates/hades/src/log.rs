#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        log::info!("[HADES/INFO] {}", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        log::warn!("[HADES/WARN] {}", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {
        log::error!("[HADES/ERROR] {}", format_args!($($arg)*));
    };
}
