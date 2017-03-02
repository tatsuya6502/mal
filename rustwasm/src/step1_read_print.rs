use readline::mal_readline;

use types::{MalType, MalResult};
use reader::read_str;
use printer::{pr_str, println};

// READ
fn read(str: String) -> MalResult {
    read_str(str)
}

// EVAL
fn eval(ast: MalType, _env: String) -> MalResult {
    Ok(ast)
}

// PRINT
fn print(exp: MalType) -> Result<String, String> {
    Ok(pr_str(&exp, true))
}

fn rep(str: String) -> Result<String, String> {
    let ast = try!(read(str));
    let exp = try!(eval(ast, "".to_string()));
    print(exp)
}

pub fn run() {
    loop {
        let line = mal_readline("user> ");
        if let None = line {
            break;
        }
        let result = rep(line.unwrap());
        match result {
            Ok(message) => println(message),
            Err(message) => println(message),
        }
    }
}
