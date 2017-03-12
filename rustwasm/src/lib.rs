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
pub mod printer;

mod step0_repl;
mod step1_read_print;
mod step2_eval;
mod step3_env;
mod step4_if_fn_do;
mod step5_tco;
mod step6_file;
mod step7_quote;
mod step8_macros;

pub use step0_repl::run as step0_repl_run;
pub use step1_read_print::run as step1_read_print_run;
pub use step2_eval::run as step2_eval_run;
pub use step3_env::run as step3_env_run;
pub use step4_if_fn_do::run as step4_if_fn_do_run;
pub use step5_tco::run as step5_tco_run;
pub use step6_file::run as step6_file_run;
pub use step7_quote::run as step7_quote_run;
pub use step8_macros::run as step8_macros_run;

#[cfg(target_arch="wasm32")]
pub mod wasm {
    use step8_macros as latest_step;

    use std::ffi::{CString, CStr};
    use std::os::raw::c_char;

    use env::Env;

    #[no_mangle]
    pub fn c_new_repl_env() -> *mut Env {
        let repl_env = latest_step::new_repl_env();

        Box::into_raw(Box::new(repl_env)) as *mut Env
    }

    #[no_mangle]
    pub fn c_env_free(ptr: *mut Env) {
        unsafe { Box::from_raw(ptr) };
    }

    #[no_mangle]
    pub fn c_rep(ptr: *mut Env,
                 v: *const c_char,
                 f: extern "C" fn(*const c_char, *const c_char, *const c_char)) {
        let env = unsafe { &mut *ptr };

        let v = unsafe { CStr::from_ptr(v) };
        let v = v.to_str().unwrap().to_string();

        let ret = latest_step::rep(v, env);
        let mal_result = match ret {
            Ok(ref v) => v.to_string(),
            _ => "".to_string(),
        };
        let mal_error = match ret {
            Err(ref v) => v.to_string(),
            _ => "".to_string(),
        };
        let stdout = "".to_string();

        let mal_result = CString::new(mal_result).unwrap().into_raw();
        let mal_error = CString::new(mal_error).unwrap().into_raw();
        let stdout = CString::new(stdout).unwrap().into_raw();
        f(mal_result, mal_error, stdout);
        unsafe {
            CString::from_raw(mal_result);
            CString::from_raw(mal_error);
            CString::from_raw(stdout);
        }
    }
}
