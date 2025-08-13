use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, CliCmd};
use log::error;

mod annotate;
mod cli;
#[cfg(feature = "serve")]
mod open;
mod utils;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mut log_setup = env_logger::builder();
    match cli.verbose {
        0 => log_setup.filter_level(log::LevelFilter::Info),
        1 => log_setup.filter_level(log::LevelFilter::Debug),
        _ => log_setup.filter_level(log::LevelFilter::Trace),
    };
    log_setup.init();

    let rst = match cli.cmd {
        CliCmd::Completion(cmd) => cmd.run(),
        CliCmd::Annotate(cmd) => cmd.run(),
        #[cfg(feature = "open")]
        CliCmd::Open(cmd) => cmd.run(),
        #[cfg(feature = "serve")]
        CliCmd::Serve(cmd) => cmd.run(),
    };

    match rst {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!("{}", e);
            ExitCode::FAILURE
        }
    }
}
