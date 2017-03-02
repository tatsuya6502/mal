use readline::mal_readline;

use std::collections::HashMap;

use types;
use types::{MalType, MalResult};
use types::MalType::*;
use reader::read_str;
use printer::{pr_str, println};

// READ
fn read(str: String) -> MalResult {
    read_str(str)
}

fn eval_ast(ast: MalType, env: &Env) -> MalResult {
    match ast {
        MalSymbol(ref v) => {
            match env.get(v) {
                Some(ast) => Ok(ast.clone()),
                None => return Err(format!("{} not found", v)),
            }
        }
        MalList(list) => {
            let mut new_list = vec![];
            for ast in list {
                new_list.push(try!(eval(ast, env)));
            }
            Ok(MalList(new_list))
        }
        MalVector(list) => {
            let mut new_list = vec![];
            for ast in list {
                new_list.push(try!(eval(ast, env)));
            }
            Ok(MalVector(new_list))
        }
        MalHashMap(list) => {
            if list.len() % 2 != 0 {
                return Err(format!("invalid hash-map: len = {}", list.len()));
            }

            let mut new_list: Vec<MalType> = vec![];
            for i in 0..list.len() {
                if i % 2 == 1 {
                    continue;
                }
                new_list.push(list[i].clone());
                new_list.push(try!(eval(list[i + 1].clone(), env)));
            }

            Ok(MalHashMap(new_list))
        }
        v => Ok(v),
    }
}

// EVAL
fn eval(ast: MalType, env: &Env) -> MalResult {
    let list = match ast {
        MalList(list) => list,
        _ => return eval_ast(ast, env),
    };
    if list.len() == 0 {
        return Ok(MalList(list));
    }

    let ast = try!(eval_ast(MalList(list), env));
    let list = seq!(ast);
    if list.len() == 0 {
        return Err("unexpected state: len == 0".to_string());
    }

    let f = &list[0];
    let f = match f {
        &MalFunc(ref f) => f,
        _ => return Err(format!("unexpected symbol. expected: function, actual: {:?}", f)),
    };
    f.apply((&list[1..]).to_vec())
}

// PRINT
fn print(exp: MalType) -> Result<String, String> {
    Ok(pr_str(&exp, true))
}

fn rep(str: String, env: &Env) -> Result<String, String> {
    let ast = try!(read(str));
    let exp = try!(eval(ast, env));
    print(exp)
}

fn add(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return Err("+: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalNumber(a + b)),
        _ => {
            Err(format!("unexpected symbol. expected: number & number, actual: {:?}",
                        args))
        }
    }
}

fn sub(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return Err("+: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalNumber(a - b)),
        _ => {
            Err(format!("unexpected symbol. expected: number & number, actual: {:?}",
                        args))
        }
    }
}

fn mul(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return Err("+: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalNumber(a * b)),
        _ => {
            Err(format!("unexpected symbol. expected: number & number, actual: {:?}",
                        args))
        }
    }
}

fn div(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return Err("+: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalNumber(a / b)),
        _ => {
            Err(format!("unexpected symbol. expected: number & number, actual: {:?}",
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
            Err(message) => println(message),
        }
    }
}
