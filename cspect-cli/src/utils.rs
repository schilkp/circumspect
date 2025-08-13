fn detect_radix(s: &str) -> (&str, u32) {
    let s = s.trim();

    if let Some(hex_part) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        (hex_part, 16)
    } else if let Some(bin_part) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        (bin_part, 2)
    } else {
        (s, 10)
    }
}

pub fn string_to_u128(s: &str) -> Result<u128, std::num::ParseIntError> {
    let (numeric_part, radix) = detect_radix(s);
    u128::from_str_radix(numeric_part, radix)
}

pub fn string_to_u64(s: &str) -> Result<u64, std::num::ParseIntError> {
    let (numeric_part, radix) = detect_radix(s);
    u64::from_str_radix(numeric_part, radix)
}
