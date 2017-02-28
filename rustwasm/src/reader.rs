use regex::Regex;

use types::MalType;
use types::MalType::*;

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

pub fn read_str(input: String) -> Result<MalType, String> {
    let tokens = tokenizer(input);
    let mut reader = Reader::new(tokens);
    read_form(&mut reader)
}

fn tokenizer(str: String) -> Vec<String> {
    let regexp =
        Regex::new(r##"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"|;.*|[^\s\[\]{}('"`,;)]*)"##)
            .unwrap();
    let mut tokens = vec![];
    for cap in regexp.captures_iter(&str) {
        let group = match cap.get(1) {
            None => break,
            Some(x) => x.as_str(),
        };
        if group.starts_with(";") {
            continue;
        }
        tokens.push(group.to_owned());
    }
    tokens
}

fn read_form(reader: &mut Reader) -> Result<MalType, String> {
    let token = try!(reader.peek());
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
            try!(reader.next()); // drop
            let sym = MalSymbol("with-meta".to_string());
            let meta = try!(read_form(reader));
            let target = try!(read_form(reader));
            Ok(MalList(vec![sym, target, meta]))
        }
        _ => read_atom(reader),
    }
}

fn read_special_symbol(reader: &mut Reader, name: String) -> Result<MalType, String> {
    try!(reader.next()); // drop
    let sym = MalSymbol(name);
    let target = try!(read_form(reader));
    Ok(MalList(vec![sym, target]))
}

fn read_list(reader: &mut Reader) -> Result<MalType, String> {
    let token = try!(reader.next()); // drop open paren
    if token != "(" {
        return Err(format!("unexpected token {}, expected (", token));
    }

    let mut list: Vec<MalType> = vec![];
    loop {
        let next = try!(reader.peek());
        if next == ")" {
            break;
        }
        list.push(try!(read_form(reader)));
    }

    try!(reader.next()); // drop close paren

    Ok(MalList(list))
}

fn read_vector(reader: &mut Reader) -> Result<MalType, String> {
    let token = try!(reader.next()); // drop open paren
    if token != "[" {
        return Err(format!("unexpected token {}, expected [", token));
    }

    let mut list: Vec<MalType> = vec![];
    loop {
        let next = try!(reader.peek());
        if next == "]" {
            break;
        }
        list.push(try!(read_form(reader)));
    }

    try!(reader.next()); // drop close paren

    Ok(MalVector(list))
}

fn read_hash_map(reader: &mut Reader) -> Result<MalType, String> {
    let token = try!(reader.next()); // drop open paren
    if token != "{" {
        return Err(format!("unexpected token {}, expected {{", token));
    }

    let mut list: Vec<MalType> = vec![];
    loop {
        let next = try!(reader.peek());
        if next == "}" {
            break;
        }
        list.push(try!(read_form(reader)));
    }

    try!(reader.next()); // drop close paren

    Ok(MalHashMap(list))
}

fn read_atom(reader: &mut Reader) -> Result<MalType, String> {
    let token = try!(reader.next());
    if let Ok(v) = token.parse::<i64>() {
        return Ok(MalNumber(v));
    }
    if token.starts_with(r#"""#) && token.ends_with(r#"""#) {
        let str = &token[1..token.len() - 1];
        let str = str.replace("\\\"", "\"")
            .replace("\\n", "\n")
            .replace("\\\\", "\\");
        return Ok(MalString(str));
    }
    if token.starts_with(":") {
        return Ok(MalKeyword(token[1..].to_string()));
    }
    match token.as_ref() {
        "nil" => Ok(MalNil),
        "true" => Ok(MalBool(true)),
        "false" => Ok(MalBool(false)),
        v => Ok(MalSymbol(v.to_string())),
    }
}
