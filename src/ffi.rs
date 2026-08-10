#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[link(name = "ext2fs", kind = "static")]
#[link(name = "com_err", kind = "static")]
#[link(name = "e2p", kind = "static")]
extern "C" {}

