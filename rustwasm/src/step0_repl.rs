use readline::mal_readline;

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
fn print(exp: String) -> Result<String, String> {
    Ok(exp)
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
