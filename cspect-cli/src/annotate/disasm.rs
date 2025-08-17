use instruction_decoder::Decoder;
use log::warn;

use crate::utils;

use super::Annotater;

pub struct Disassembler {
    decoder: Decoder,
    instr_len: usize,
}

impl Disassembler {
    pub fn new(decoder: Decoder, instr_len: usize) -> Self {
        Self { decoder, instr_len }
    }
}

pub struct DisasmAnnotater {
    rv32: Disassembler,
    rv64: Disassembler,
}

impl DisasmAnnotater {
    pub fn new() -> Self {
        Self {
            rv32: new_rv32_translator(),
            rv64: new_rv64_translator(),
        }
    }
}

#[rustfmt::skip]
pub fn new_rv32_translator() -> Disassembler {
    let decoder = Decoder::new(&[
        include_str!("../../../third_party/instruction-decoder/toml/RV32I.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32M.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32A.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32F.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32_Zbb.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32_Zbkb.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32_Zbs.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32_Zknd.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32_Zkne.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32_Zfa.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32_Zicsr.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32C-lower.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32_Zcb-lower.toml") .to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32_Zcf-lower.toml") .to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV32_Zacas.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zcd-lower.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zfh.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zba.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zbc.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zbkc.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zbkx.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zfh.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zknh.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zksed.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zksh.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zawrs.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zicond.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zifencei.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zicbo.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zimop.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zihintntl.toml").to_string(),
    ])
    .expect("Can't build RV32 decoder");

    Disassembler::new(decoder, 32)
}

#[rustfmt::skip]
pub fn new_rv64_translator() -> Disassembler {
    let decoder = Decoder::new(&[
        include_str!("../../../third_party/instruction-decoder/toml/RV64I.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64M.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64A.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64D.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64_Zbb.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64_Zbkb.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64_Zbs.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64_Zknd.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64_Zkne.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64_Zacas.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64_Zfa.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64C-lower.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64_Zcb-lower.toml") .to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV64_Zcd-lower.toml") .to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RVV.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zvbb.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zvbc.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zvkg.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zvkned.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zvknha.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zvknhb.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zvksed.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zvksh.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zcd-lower.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zfh.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zba.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zbc.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zbkc.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zbkx.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zknh.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zksed.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zksh.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zawrs.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zicond.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zifencei.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zicbo.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zimop.toml").to_string(),
        include_str!("../../../third_party/instruction-decoder/toml/RV_Zihintntl.toml").to_string(),
    ])
    .expect("Can't build RV64 decoder");
    Disassembler::new(decoder, 32)
}

impl Annotater for DisasmAnnotater {
    fn accepts_keys(&self) -> Vec<String> {
        vec!["da-rv32".into(), "da-rv64".into()]
    }

    fn annotate(&mut self, placeholder: &super::Placeholder) -> anyhow::Result<Option<String>> {
        let disasm = match placeholder.kind.as_str() {
            "da-rv32" => &self.rv32,
            "da-rv64" => &self.rv64,
            _ => unreachable!(),
        };

        let Ok(val) = utils::string_to_u128(&placeholder.value) else {
            warn!("disasm: invalid number '{}' - skipping.", placeholder.value);
            return Ok(None);
        };

        match disasm.decoder.decode(val, disasm.instr_len) {
            Ok(instr) => Ok(Some(format!("{instr} (0x{val:08x})"))),
            Err(_err) => Ok(Some(format!("? (0x{val:08x})"))),
        }
    }
}
