#[cfg(not(target_arch="wasm32"))]
use regex::Regex;

use types::MalType;
use types::MalType::*;
use types::vec_to_hash_map;
use types::MalResult;

#[derive(Debug, Clone)]
struct Reader {
    tokens: Vec<String>,
    position: usize,
}

impl Reader {
    pub fn new(tokens: Vec<String>) -> Reader {
        Reader {
            tokens: tokens,
            position: 0,
        }
    }

    fn next(&mut self) -> Result<String, String> {
        let ret = self.peek();
        self.position += 1;
        ret
    }

    fn peek(&self) -> Result<String, String> {
        if self.position < self.tokens.len() {
            Ok(self.tokens[self.position].to_string())
        } else {
            Err("unexpected EOF".to_string())
        }
    }
}

pub fn read_str(input: String) -> MalResult {
    let tokens = tokenizer(input);
    let mut reader = Reader::new(tokens);
    read_form(&mut reader)
}

#[cfg(not(target_arch="wasm32"))]
fn tokenizer(str: String) -> Vec<String> {
    let regexp =
        Regex::new(r##"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"|;.*|[^\s\[\]{}('"`,;)]*)"##)
            .unwrap();
    let mut tokens = Vec::new();
    for cap in regexp.captures_iter(&str) {
        let group = match cap.get(1) {
            None => break,
            Some(x) => x.as_str(),
        };
        if group.starts_with(';') {
            continue;
        }
        tokens.push(group.to_owned());
    }
    tokens
}

#[cfg(target_arch="wasm32")]
mod jstokenizer {
    // regexp crate is so big binary.
    // If we can use RegExp class in JS, We use that one.

    use std::ffi::{CString, CStr};
    use std::os::raw::c_char;

    // in .cargo/config... #[link_args = "--js-library static/emcc-bind.js"]
    #[allow(improper_ctypes)]
    extern "C" {
        fn js_tokenizer(tokenizer: *mut JSTokenizer, str: *const c_char);
    }

    pub struct JSTokenizer {
        pub list: Vec<String>,
    }

    impl JSTokenizer {
        pub fn new() -> JSTokenizer {
            JSTokenizer { list: Vec::new() }
        }

        pub fn call_js(&mut self, str: String) -> Vec<String> {
            let str = CString::new(str).unwrap().into_raw();

            unsafe { js_tokenizer(self, str) };

            unsafe { CString::from_raw(str) };

            let mut ret_vec = Vec::new();
            for str in self.list.clone() {
                ret_vec.push(str.to_string());
            }
            ret_vec
        }
    }

    #[no_mangle]
    pub fn c_jstokenizer_append(ptr: *mut JSTokenizer, v: *const c_char) {
        let mut container: &mut JSTokenizer = unsafe { &mut *ptr };
        let v = unsafe { CStr::from_ptr(v) };
        let v = v.to_str().unwrap();
        container.list.push(v.to_string());
    }
}

#[cfg(target_arch="wasm32")]
#[no_mangle]
pub use self::jstokenizer::c_jstokenizer_append;

#[cfg(target_arch="wasm32")]
fn tokenizer(str: String) -> Vec<String> {
    jstokenizer::JSTokenizer::new().call_js(str)
}

fn read_form(reader: &mut Reader) -> MalResult {
    let token = reader.peek()?;
    match token.as_ref() {
        "(" => read_list(reader),
        "[" => read_vector(reader),
        "{" => read_hash_map(reader),
        "'" => read_special_symbol(reader, "quote".to_string()),
        "`" => read_special_symbol(reader, "quasiquote".to_string()),
        "~" => read_special_symbol(reader, "unquote".to_string()),
        "~@" => read_special_symbol(reader, "splice-unquote".to_string()),
        "@" => read_special_symbol(reader, "deref".to_string()),
        "^" => {
            reader.next()?; // drop
            let sym = MalSymbol("with-meta".to_string());
            let meta = read_form(reader)?;
            let target = read_form(reader)?;
            Ok(MalList(vec![sym, target, meta], Box::new(None)))
        }
        _ => read_atom(reader),
    }
}

fn read_special_symbol(reader: &mut Reader, name: String) -> MalResult {
    reader.next()?; // drop
    let sym = MalSymbol(name);
    let target = read_form(reader)?;
    Ok(MalList(vec![sym, target], Box::new(None)))
}

fn read_list(reader: &mut Reader) -> MalResult {
    let token = reader.next()?; // drop open paren
    if token != "(" {
        return mal_error!(format!("unexpected token {}, expected (", token));
    }

    let mut list: Vec<MalType> = Vec::new();
    loop {
        let next = reader.peek()?;
        if next == ")" {
            break;
        }
        list.push(read_form(reader)?);
    }

    reader.next()?; // drop close paren

    Ok(MalList(list, Box::new(None)))
}

fn read_vector(reader: &mut Reader) -> MalResult {
    let token = reader.next()?; // drop open paren
    if token != "[" {
        return mal_error!(format!("unexpected token {}, expected [", token));
    }

    let mut list: Vec<MalType> = Vec::new();
    loop {
        let next = reader.peek()?;
        if next == "]" {
            break;
        }
        list.push(read_form(reader)?);
    }

    reader.next()?; // drop close paren

    Ok(MalVector(list, Box::new(None)))
}

fn read_hash_map(reader: &mut Reader) -> MalResult {
    let token = reader.next()?; // drop open paren
    if token != "{" {
        return mal_error!(format!("unexpected token {}, expected {{", token));
    }

    let mut list: Vec<MalType> = Vec::new();
    loop {
        let next = reader.peek()?;
        if next == "}" {
            break;
        }
        list.push(read_form(reader)?);
    }

    reader.next()?; // drop close paren

    vec_to_hash_map(list)
}

fn read_atom(reader: &mut Reader) -> MalResult {
    let token = reader.next()?;
    if let Ok(v) = token.parse::<i64>() {
        return Ok(MalNumber(v));
    }
    if token.starts_with('"') && token.ends_with('"') {
        let str = &token[1..token.len() - 1];
        let str = str.replace("\\\"", "\"")
            .replace("\\n", "\n")
            .replace("\\\\", "\\");
        return Ok(MalString(str));
    }
    if token.starts_with(':') {
        return Ok(MalKeyword(token[1..].to_string()));
    }
    match token.as_ref() {
        "nil" => Ok(MalNil),
        "true" => Ok(MalBool(true)),
        "false" => Ok(MalBool(false)),
        v => Ok(MalSymbol(v.to_string())),
    }
}
