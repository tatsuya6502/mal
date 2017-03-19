use readline::mal_readline;

use types::MalError;
use printer::println;

// READ
fn read(str: String) -> Result<String, String> {
    Ok(str)
}

// EVAL
fn eval(ast: String, _env: String) -> Result<String, String> {
    Ok(ast)
}

// PRINT
fn print(exp: String) -> Result<String, MalError> {
    Ok(exp)
}

pub fn rep(str: &str) -> Result<String, MalError> {
    let ast = try!(read(str.to_string()));
    let exp = try!(eval(ast, "".to_string()));
    print(exp)
}

pub fn run() {
    loop {
        let line = mal_readline("user> ");
        if line.is_none() {
            break;
        }
        let result = rep(&line.unwrap());
        match result {
            Ok(message) |
            Err(MalError::ErrorMessage(message)) => println(message),
            Err(MalError::ThrowAST(ast)) => panic!("{:?}", ast),
        }
    }
}
