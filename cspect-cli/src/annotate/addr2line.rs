use anyhow::anyhow;
use log::{trace, warn};
use std::{borrow::Cow, path::Path};

use addr2line::{Loader, Location, fallible_iterator::FallibleIterator, gimli};

use crate::utils;

use super::Annotater;

pub struct Addr2LineAnnotator {
    ctx: addr2line::Loader,
    demangle: bool,
    do_inlines: bool,
}

impl Addr2LineAnnotator {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        let ctx = Loader::new(path).map_err(|_| anyhow!("failed to load addr2line file!"))?;
        Ok(Self {
            ctx,
            demangle: true,
            do_inlines: true,
        })
    }

    fn source_loc_str(&self, loc: &Option<Location>) -> String {
        let Some(loc) = loc else {
            return "?".to_string();
        };

        // Attempt to trim filename down:
        let file = loc.file.map(|x| {
            Path::new(x)
                .file_name()
                .map(|x| x.to_string_lossy())
                .unwrap_or(x.into())
        });

        match (file, loc.line, loc.column) {
            (None, _, _) => "?".to_string(),
            (Some(file), None, _) => file.to_string(),
            (Some(file), Some(line), None) => format!("{}:{}", file, line),
            (Some(file), Some(line), Some(column)) => format!("{}:{}:{}", file, line, column),
        }
    }

    fn func_str(&self, name: Option<&str>, language: Option<gimli::DwLang>) -> String {
        if let Some(name) = name {
            if self.demangle {
                format!("{}", addr2line::demangle_auto(Cow::from(name), language))
            } else {
                name.to_string()
            }
        } else {
            "?".to_string()
        }
    }

    fn frames_str(&mut self, addr: u64) -> String {
        trace!("addr2line lookup 0x{addr:X}");
        let Ok(frames) = self.ctx.find_frames(addr) else {
            trace!("  find_frames failed.");
            return "a2l err".into();
        };

        let mut frames = frames.peekable();
        let mut result: String = String::with_capacity(32);

        loop {
            let Ok(frame) = frames.next() else {
                trace!("  frames.next failed.");
                return "a2l err".into();
            };
            let Some(frame) = frame else {
                break;
            };

            // The logic below is very heavily based on the addr2line reference
            // implementation linked below, which is released under MIT/Apache-2:
            // https://github.com/gimli-rs/addr2line/blob/c4eada981907ec85e91dcb37c7eed085bbbf66f0/src/bin/addr2line.rs#L239

            let symbol = if matches!(frames.peek(), Ok(None)) {
                self.ctx.find_symbol(addr)
            } else {
                None
            };

            let func_str = if symbol.is_some() {
                // Prefer the symbol table over the DWARF name because:
                // - the symbol can include a clone suffix
                // - llvm may omit the linkage name in the DWARF with -g1
                Some(self.func_str(symbol, None))
            } else {
                frame
                    .function
                    .map(|func| self.func_str(func.raw_name().ok().as_deref(), func.language))
            };

            let loc_str = frame.location.map(|loc| self.source_loc_str(&Some(loc)));

            if !result.is_empty() {
                result.push_str("->");
            }

            match (loc_str, func_str) {
                (None, None) => result.push('?'),
                (None, Some(f)) => result.push_str(&f),
                (Some(l), None) => result.push_str(&l),
                (Some(l), Some(f)) => {
                    result.push_str(&l);
                    result.push(':');
                    result.push_str(&f);
                }
            }

            if !self.do_inlines {
                break;
            }
        }

        let result = if result.is_empty() {
            "?".into()
        } else {
            result
        };

        trace!(" resolved to: {result}");
        result
    }
}

impl Annotater for Addr2LineAnnotator {
    fn accepts_keys(&self) -> Vec<String> {
        vec!["a2l".to_string()]
    }

    fn annotate(&mut self, placeholder: &super::Placeholder) -> anyhow::Result<Option<String>> {
        let Ok(addr) = utils::string_to_u64(&placeholder.value) else {
            warn!(
                "addr2line: invalid number '{}' - skipping.",
                placeholder.value
            );
            return Ok(None);
        };

        let frames_str = self.frames_str(addr);
        Ok(Some(format!("{frames_str} (0x{addr:08x})")))
    }
}
