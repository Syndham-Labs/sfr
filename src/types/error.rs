use std::fmt::Display;

pub enum ErrorSeverity {
    LOW,
    MEDIUM,
    HIGH,
    CRITICAL,
}

impl Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::LOW => write!(f, "LOW"),
            ErrorSeverity::MEDIUM => write!(f, "MEDIUM"),
            ErrorSeverity::HIGH => write!(f, "HIGH"),
            ErrorSeverity::CRITICAL => write!(f, "CRITICAL"),
        }
    }
}

pub struct FunctionError<T = ()> {
    pub message: String,
    pub severity: ErrorSeverity,
    pub payload: Option<T>,
}

impl<T> FunctionError<T> {
    pub fn new(message: String, severity: ErrorSeverity, payload: Option<T>) -> Self {
        Self {
            message,
            severity,
            payload,
        }
    }
}

impl Display for FunctionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]: {} [Additional payloads may be attached]\n{:#?}",
            self.severity, self.message, self.payload
        )
    }
}
