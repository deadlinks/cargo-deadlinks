use log::LevelFilter;

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
