#![allow(non_camel_case_types)]
#![allow(dead_code)]
//! Select FFI types from `svdpi.sv`

// typedef uint8_t svScalar;
pub type svScalar = u8;

// typedef svScalar svBit; /* scalar */
pub type svBit = svScalar;

// typedef svScalar svLogic; /* scalar */
pub type svLogic = svScalar;
