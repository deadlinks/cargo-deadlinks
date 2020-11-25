use log::LevelFilter;
use pico_args::Error;
use std::fmt::{self, Display};

/// Initalizes the logger according to the provided config flags.
pub fn init_logger(debug: bool, verbose: bool, krate: &str) {
    let mut builder = env_logger::Builder::new();
    match (debug, verbose) {
        (true, _) => {
            builder.filter(Some(krate), LevelFilter::Debug);
        }
        (false, true) => {
            builder.filter(Some(krate), LevelFilter::Info);
        }
        _ => {}
    }
    builder.parse_default_env().init();
}

// See https://github.com/RazrFalcon/pico-args/pull/26
pub struct PicoError(pub Error);

impl Display for PicoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Error::ArgumentParsingFailed { cause } => {
                write!(f, "failed to parse arguments: {}", cause)
            }
            Error::Utf8ArgumentParsingFailed { value, cause } => {
                write!(f, "failed to parse '{}': {}", value, cause)
            }
            _ => self.0.fmt(f),
        }
    }
}

impl From<Error> for PicoError {
    fn from(err: Error) -> Self {
        Self(err)
    }
}
