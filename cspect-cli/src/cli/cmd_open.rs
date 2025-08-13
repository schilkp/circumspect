use std::path::PathBuf;

use clap::Parser;

use crate::open::open_trace;

#[derive(Parser, Debug)]
#[command(about = "Open trace file in perfetto")]
pub struct Cmd {
    /// Perfetto trace file to be opened
    #[arg()]
    pub input: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> anyhow::Result<()> {
        open_trace(&self.input)
    }
}
