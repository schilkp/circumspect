// ==== Bindgen-produced svdpi.h types =========================================

// svdpi.h interface kept in private module, with only selective re-exports:
mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use ffi::svBit;
