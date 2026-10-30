#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        println!("[HADES/INFO] {}", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        println!("[HADES/WARN] {}", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => {
        println!("[HADES/ERROR] {}", format_args!($($arg)*));
    };
}
