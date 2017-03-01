#[cfg(not(target_arch="wasm32"))]
pub use self::normal::*;
#[cfg(target_arch="wasm32")]
pub use self::jsbridge::*;

#[cfg(not(target_arch="wasm32"))]
mod normal {
    // Based on: https://github.com/shaleh/rust-readline (MIT)
    use libc;

    use std::env;
    use std::path::PathBuf;
    use std::ffi::{CStr, CString};
    use std::fs::{OpenOptions, File};
    use std::io::BufReader;
    use std::io::prelude::*;
    use std::str;

    mod ext_readline {
        extern crate libc;
        use self::libc::c_char;
        #[link(name = "readline")]
        extern "C" {
            pub fn add_history(line: *const c_char);
            pub fn readline(p: *const c_char) -> *const c_char;
        }
    }

    pub fn add_history(line: &str) {
        unsafe {
            ext_readline::add_history(CString::new(line).unwrap().as_ptr());
        }
    }

    pub fn readline(prompt: &str) -> Option<String> {
        let cprmt = CString::new(prompt).unwrap();
        unsafe {
            let ptr = ext_readline::readline(cprmt.as_ptr());
            if ptr.is_null() {
                // user pressed Ctrl-D
                None
            } else {
                let ret = str::from_utf8(CStr::from_ptr(ptr).to_bytes());
                let ret = ret.ok().map(|s| s.to_string());
                libc::free(ptr as *mut _);
                return ret;
            }
        }
    }

    // --------------------------------------------

    static mut HISTORY_LOADED: bool = false;

    fn get_history_file() -> PathBuf {
        let mut path = env::home_dir().unwrap_or(PathBuf::from("/home/joelm"));
        path.push(".mal-history".to_string());
        path
    }

    fn load_history() {
        unsafe {
            if HISTORY_LOADED {
                return;
            }
            HISTORY_LOADED = true;
        }

        let history_file = get_history_file();
        let file = match File::open(history_file) {
            Ok(f) => f,
            Err(..) => return,
        };
        let file = BufReader::new(file);
        for line in file.lines() {
            let rt: &[_] = &['\r', '\n'];
            let line2 = line.unwrap();
            let line3 = line2.trim_right_matches(rt);
            add_history(line3);
        }
    }

    fn append_to_history(line: &str) {
        let history_file = get_history_file();
        let file = OpenOptions::new()
            .append(true)
            .write(true)
            .create(true)
            .open(history_file);
        let mut file = match file {
            Ok(f) => f,
            Err(..) => return,
        };
        let _ = file.write_all(line.as_bytes());
        let _ = file.write_all(b"\n");
    }

    pub fn mal_readline(prompt: &str) -> Option<String> {
        load_history();
        let line = readline(prompt);
        if let Some(ref s) = line {
            add_history(s);
            append_to_history(s);
        }
        line
    }
}

#[cfg(target_arch="wasm32")]
mod jsbridge {
    pub fn mal_readline(_prompt: &str) -> Option<String> {
        unimplemented!()
    }
}
