use std::collections::HashMap;

use readline::mal_readline;

use types;
use types::{MalType, MalError, MalResult, MalHashMapKey};
use types::MalType::*;
use reader::read_str;
use printer::{pr_str, println};

// READ
fn read(str: String) -> MalResult {
    match read_str(str) {
        Ok(v) => Ok(v),
        Err(v) => mal_error!(v),
    }
}

fn eval_ast(ast: MalType, env: Env) -> MalResult {
    match ast {
        MalSymbol(ref v) => {
            match env.get(v) {
                Some(ast) => Ok(ast.clone()),
                None => return mal_error!(format!("'{}' not found", v)),
            }
        }
        MalList(list) => {
            let mut new_list = vec![];
            for ast in list {
                new_list.push(try!(eval(ast, env.clone())));
            }
            Ok(MalList(new_list))
        }
        MalVector(list) => {
            let mut new_list = vec![];
            for ast in list {
                new_list.push(try!(eval(ast, env.clone())));
            }
            Ok(MalVector(new_list))
        }
        MalHashMap(hash_map) => {
            let mut new_hash_map: HashMap<MalHashMapKey, MalType> = HashMap::new();
            for (key, value) in hash_map.iter() {
                let value = try!(eval(value.clone(), env.clone()));
                new_hash_map.insert(key.clone(), value);
            }

            Ok(MalHashMap(new_hash_map))
        }
        v => Ok(v),
    }
}

// EVAL
fn eval(ast: MalType, env: Env) -> MalResult {
    let list = match ast {
        MalList(list) => list,
        _ => return eval_ast(ast, env),
    };
    if list.len() == 0 {
        return Ok(MalList(list));
    }

    let ast = try!(eval_ast(MalList(list), env.clone()));
    let list = seq!(ast);
    if list.len() == 0 {
        return mal_error!("unexpected state: len == 0".to_string());
    }

    let f = &list[0];
    let f = match f {
        &MalFunc(ref f) => f,
        _ => return mal_error!(format!("unexpected symbol. expected: function, actual: {:?}", f)),
    };
    f.apply((&list[1..]).to_vec())
}

// PRINT
fn print(exp: MalType) -> Result<String, MalError> {
    Ok(pr_str(&exp, true))
}

pub fn rep(str: String, env: &Env) -> Result<String, MalError> {
    let ast = try!(read(str));
    let exp = try!(eval(ast, env.clone()));
    print(exp)
}

fn add(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("+: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalNumber(a + b)),
        _ => {
            mal_error!(format!("unexpected symbol. expected: number & number, actual: {:?}",
                               args))
        }
    }
}

fn sub(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("+: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalNumber(a - b)),
        _ => {
            mal_error!(format!("unexpected symbol. expected: number & number, actual: {:?}",
                               args))
        }
    }
}

fn mul(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("+: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalNumber(a * b)),
        _ => {
            mal_error!(format!("unexpected symbol. expected: number & number, actual: {:?}",
                               args))
        }
    }
}

fn div(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("+: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalNumber(a / b)),
        _ => {
            mal_error!(format!("unexpected symbol. expected: number & number, actual: {:?}",
                               args))
        }
    }
}

type Env = HashMap<String, MalType>;

pub fn run() {
    let mut repl_env: Env = HashMap::new();
    repl_env.insert("+".to_string(), types::func_from_bootstrap(add));
    repl_env.insert("-".to_string(), types::func_from_bootstrap(sub));
    repl_env.insert("*".to_string(), types::func_from_bootstrap(mul));
    repl_env.insert("/".to_string(), types::func_from_bootstrap(div));

    loop {
        let line = mal_readline("user> ");
        if let None = line {
            break;
        }
        let result = rep(line.unwrap(), &repl_env);
        match result {
            Ok(message) => println(message),
            Err(MalError::ErrorMessage(message)) => println(message),
            Err(MalError::ThrowAST(ref ast)) => {
                println(format!("receive exception: {}", pr_str(ast, true)))
            }
        }
    }
}
