use types::MalType;
use types::MalType::*;

#[cfg(target_arch="wasm32")]
pub mod wasm_stdout {
    use std::ffi::{CString, CStr};
    use std::os::raw::c_char;

    use std::rc::Rc;
    use std::cell::{RefCell, Cell};

    type Container = Rc<RefCell<Cell<extern "C" fn(*const c_char)>>>;

    thread_local!(static STDOUT: Container = Rc::new(RefCell::new(Cell::new(stdoutln_mock))));

    extern "C" fn stdoutln_mock(v: *const c_char) {
        let v = unsafe { CStr::from_ptr(v) };
        let v = v.to_str().unwrap();
        println!("{}", v);
    }

    #[no_mangle]
    pub extern "C" fn c_set_stdoutln(f: Option<extern "C" fn(*const c_char)>) {
        STDOUT.with(|stdout| {
            let cell = stdout.borrow_mut();
            match f {
                Some(f) => cell.set(f),
                None => cell.set(stdoutln_mock),
            };
        });
    }

    pub fn stdoutln(str: String) {
        STDOUT.with(|stdout| {
            let cell = stdout.borrow();
            let str = CString::new(str).unwrap().into_raw();
            cell.get()(str);
            unsafe {
                CString::from_raw(str);
            }
        });
    }
}

#[cfg(not(target_arch="wasm32"))]
pub fn println(str: String) {
    println!("{}", str);
}

#[cfg(target_arch="wasm32")]
pub use self::wasm_stdout::c_set_stdoutln;

#[cfg(target_arch="wasm32")]
pub fn println(str: String) {
    wasm_stdout::stdoutln(str);
}

pub fn pr_str(v: &MalType, print_readably: bool) -> String {
    match v {
        &MalList(ref list) => {
            let value =
                list.iter().map(|x| pr_str(x, print_readably)).collect::<Vec<_>>().join(" ");
            format!("({})", value)
        }
        &MalVector(ref list) => {
            let value =
                list.iter().map(|x| pr_str(x, print_readably)).collect::<Vec<_>>().join(" ");
            format!("[{}]", value)
        }
        &MalHashMap(ref list) => {
            let value =
                list.iter().map(|x| pr_str(x, print_readably)).collect::<Vec<_>>().join(" ");
            format!("{{{}}}", value)
        }
        &MalNumber(ref v) => format!("{}", v),
        &MalSymbol(ref v) => format!("{}", v),
        &MalBool(v) => format!("{}", v),
        &MalString(ref v) => {
            if print_readably {
                let v = v.replace("\\", "\\\\")
                    .replace("\"", "\\\"")
                    .replace("\n", "\\n");
                format!(r#""{}""#, v)
            } else {
                v.to_string()
            }
        }
        &MalNil => "nil".to_string(),
        &MalKeyword(ref v) => format!(":{}", v),
        &MalFunc(_) => "#<function>".to_string(),
    }
}
