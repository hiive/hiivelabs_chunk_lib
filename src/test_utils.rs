use env_logger;
pub(crate) fn setup_test_logger() {
    #[cfg(debug_assertions)]
    {
        let result = env_logger::Builder::new()
            .write_style(env_logger::WriteStyle::Always)
            // Include all events in tests
            .filter_level(log::LevelFilter::max())
            // Ensure events are captured by `cargo test`
            .is_test(true)
            .format_target(false)
            // Ignore errors initializing the logger if tests race to configure it
            .try_init();

        // println!("Logger initialized.")
    }
}

pub(crate) const CONSOLE_RED: &str = "\x1b[31m";
pub(crate) const CONSOLE_GREEN: &str = "\x1b[32m";
pub(crate) const CONSOLE_BRIGHT_GREEN: &str = "\x1b[92m";
pub(crate) const CONSOLE_YELLOW: &str = "\x1b[33m";
pub(crate) const CONSOLE_BLUE: &str = "\x1b[34m";
pub(crate) const CONSOLE_BRIGHT_BLUE: &str = "\x1b[94m";
pub(crate) const CONSOLE_DEFAULT_COLOR: &str = "\x1b[0m";