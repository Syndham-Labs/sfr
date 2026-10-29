use std::sync::OnceLock;
use tokio::runtime::{Builder, Runtime};

use crate::types::error::{ErrorSeverity::CRITICAL, FunctionError};

#[derive(Debug)]
pub struct TokioMultiThreadRuntime;

impl TokioMultiThreadRuntime {
    fn runtime() -> &'static OnceLock<Runtime> {
        static INSTANCE: OnceLock<Runtime> = OnceLock::new();
        &INSTANCE
    }

    pub fn configure() -> Result<(), FunctionError> {
        if let Some(_) = Self::runtime().get() {
            return Err(FunctionError::new("Tokio Runtime somehow already initialized, ensure no Builder is running before the configuration of this struct".to_string(), CRITICAL, None));
        }

        let rt = match Builder::new_multi_thread().enable_all().build() {
            Err(e) => return Err(FunctionError::new(e.to_string(), CRITICAL, None)),
            Ok(_rt) => _rt,
        };

        match Self::runtime().set(rt) {
            Err(_) => {
                return Err(FunctionError::new(
                    "Unexpected error trying to initialize Tokio runtime".to_string(),
                    CRITICAL,
                    None,
                ));
            }
            Ok(_) => {}
        };

        Ok(())
    }

    pub fn get_runtime() -> Result<&'static Runtime, FunctionError> {
        match Self::runtime().get() {
            None => Err(FunctionError::new(
                "No Tokio runtime was initialized".to_string(),
                CRITICAL,
                None,
            )),
            Some(rt) => Ok(rt),
        }
    }
}
