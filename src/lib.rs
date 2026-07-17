mod jyutping;

pub use jyutping::{Faancit, JyutpingChar, get_faancit, get_jyutping};

#[cfg(target_arch = "wasm32")]
mod web;
