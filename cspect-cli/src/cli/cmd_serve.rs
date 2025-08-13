use std::path::PathBuf;

use clap::Parser;

use crate::open::serve_trace;

#[derive(Parser, Debug)]
#[command(about = "Serve trace file for perfetto")]
pub struct Cmd {
    /// Perfetto trace file to be served
    #[arg()]
    pub input: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> anyhow::Result<()> {
        serve_trace(&self.input)
    }
}
