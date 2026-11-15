//! Server entry point: read the configuration, then start the transport it names.

use std::process::ExitCode;

use lxagents_security::{config, identity::SERVER_ID, run::run};

fn main() -> ExitCode {
    match config::from_process_env() {
        Ok(config) => run(&config),
        Err(error) => {
            eprintln!("{SERVER_ID} {error}");
            ExitCode::FAILURE
        }
    }
}
