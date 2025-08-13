mod cmd_annotate;
mod cmd_completion;
#[cfg(feature = "open")]
mod cmd_open;
#[cfg(feature = "serve")]
mod cmd_serve;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[command(name = "cspect")]
pub struct Cli {
    #[arg(long, short, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub cmd: CliCmd,
}

#[derive(Parser, Debug)]
pub enum CliCmd {
    Annotate(cmd_annotate::Cmd),
    Completion(cmd_completion::Cmd),
    #[cfg(feature = "open")]
    Open(cmd_open::Cmd),
    #[cfg(feature = "serve")]
    Serve(cmd_serve::Cmd),
}
