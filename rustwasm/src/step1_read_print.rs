use readline::mal_readline;

use types::{MalType, MalError, MalResult};
use reader::read_str;
use printer::{pr_str, println};

// READ
fn read(str: String) -> MalResult {
    read_str(str).or_else(|e| mal_error!(e))
}

// EVAL
fn eval(ast: MalType, _env: String) -> MalResult {
    Ok(ast)
}

// PRINT
fn print(exp: MalType) -> Result<String, MalError> {
    Ok(pr_str(&exp, true))
}

pub fn rep(str: &str) -> Result<String, MalError> {
    let ast = try!(read(str.to_string()));
    let exp = try!(eval(ast, "".to_string()));
    print(exp)
}

pub fn run() {
    loop {
        let line = mal_readline("user> ");
        if let None = line {
            break;
        }
        let result = rep(&line.unwrap());
        match result {
            Ok(message) => println(message),
            Err(MalError::ErrorMessage(message)) => println(message),
            Err(MalError::ThrowAST(ref ast)) => {
                println(format!("receive exception: {}", pr_str(ast, true)))
            }
        }
    }
}
