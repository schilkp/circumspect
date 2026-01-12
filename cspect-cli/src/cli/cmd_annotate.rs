use std::path::PathBuf;

use clap::Parser;

use crate::{
    annotate::{addr2line::Addr2LineAnnotator, annotate, disasm::DisasmAnnotater, Annotater},
    open::{open_trace, serve_trace},
};

#[derive(Parser, Debug)]
#[command(about = "Annotate Trace")]
pub struct Cmd {
    /// Perfetto trace file to convert
    #[arg()]
    pub input: PathBuf,

    /// Location to store annotated trace (default: overwrite)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Disassemble $da_*-annotated instructions
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub disasm: bool,

    /// Convert $a2l-annotated addresses to lines using given elf file.
    #[arg(long)]
    pub addr2line: Option<PathBuf>,

    /// Open annotated trace in perfetto
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub open: bool,

    /// Serve annotated trace for perfetto
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub serve: bool,
}

impl Cmd {
    pub fn run(&self) -> anyhow::Result<()> {
        // Construct Annotators:
        let mut annotators: Vec<Box<dyn Annotater>> = vec![];

        if let Some(path) = &self.addr2line {
            annotators.push(Box::new(Addr2LineAnnotator::new(path)?));
        }

        if self.disasm {
            annotators.push(Box::new(DisasmAnnotater::new()));
        }

        // Perform annotation:
        annotate(&self.input, self.output.as_deref(), annotators)?;

        // Serve/Open:
        let file_to_open = self.output.as_ref().unwrap_or(&self.input);
        if self.open {
            open_trace(file_to_open)?;
        }
        if self.serve {
            serve_trace(file_to_open)?;
        }

        Ok(())
    }
}
