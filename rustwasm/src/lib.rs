#[cfg(not(target_arch="wasm32"))]
extern crate regex;

extern crate libc;

mod readline;

macro_rules! seq {
    ($ast:expr) => (
        match $ast {
            MalList(list) | MalVector(list) => list,
            _ => {
                let msg = format!("invalid symbol. expected: list or vector, actual: {:?}", $ast);
                return Err(msg);
            },
        }
    )
}

pub mod types;
mod core;
pub mod env;
pub mod reader;
mod printer;

pub mod step0_repl;
pub mod step1_read_print;
pub mod step2_eval;
pub mod step3_env;
pub mod step4_if_fn_do;
