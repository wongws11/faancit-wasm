mod jyutping;

pub use jyutping::{Faancit, JyutpingChar, Tone, get_faancit, get_jyutping};

#[cfg(target_arch = "wasm32")]
pub mod abi;
