use readline::mal_readline;

use types;
use types::{MalType, MalResult};
use types::MalType::*;
use env::{Env, new_env};
use reader::read_str;
use printer::pr_str;

// READ
fn read(str: String) -> MalResult {
    read_str(str)
}

fn eval_ast(ast: MalType, env: Env) -> MalResult {
    match ast {
        MalSymbol(ref v) => {
            match env.clone().get(v.to_string()) {
                Some(ast) => Ok(ast.clone()),
                None => return Err(format!("{} not found", v)),
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
                new_list.push(try!(eval(list[i + 1].clone(), env.clone())));
            }

            Ok(MalHashMap(new_list))
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

    {
        let a0 = &list[0];
        match a0 {
            &MalSymbol(ref v) if v == "def!" => {
                let key = &list[1];
                let key = match key {
                    &MalSymbol(ref v) => v,
                    _ => {
                        return Err(format!("unexpected symbol. expected: symbol, actual: {:?}",
                                           key))
                    }
                };
                let value = &list[2];
                let ret = try!(eval(value.clone(), env.clone()));
                return Ok(env.set(key.to_string(), ret));
            }
            &MalSymbol(ref v) if v == "let*" => {
                let let_env = new_env(Some(env));
                let pairs = &list[1];
                let expr = &list[2];
                let list = seq!(pairs.clone());
                for i in 0..list.len() {
                    if i % 2 == 1 {
                        continue;
                    }
                    let key = &list[i];
                    let value = &list[i + 1];
                    let key = match key {
                        &MalSymbol(ref v) => v,
                        _ => {
                            return Err(format!("unexpected symbol. expected: symbol, actual: {:?}",
                                               key))
                        }
                    };
                    let_env.set(key.to_string(), try!(eval(value.clone(), let_env.clone())));
                }

                let ret = try!(eval(expr.clone(), let_env));
                return Ok(ret);
            }
            _ => {}
        };
    }

    let ast = try!(eval_ast(MalList(list), env.clone()));
    let list = seq!(ast);
    if list.len() == 0 {
        return Err("unexpected state: len == 0".to_string());
    }

    let f = &list[0];
    let f = match f {
        &MalFunc(f) => f,
        _ => return Err(format!("unexpected symbol. expected: function, actual: {:?}", f)),
    };
    f((&list[1..]).to_vec())
}

// PRINT
fn print(exp: MalType) -> Result<String, String> {
    Ok(pr_str(&exp, true))
}

fn rep(str: String, env: &Env) -> Result<String, String> {
    let ast = try!(read(str));
    let exp = try!(eval(ast, env.clone()));
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

pub fn run() {
    let repl_env = new_env(None);
    repl_env.set("+".to_string(), types::func_from_bootstrap(add));
    repl_env.set("-".to_string(), types::func_from_bootstrap(sub));
    repl_env.set("*".to_string(), types::func_from_bootstrap(mul));
    repl_env.set("/".to_string(), types::func_from_bootstrap(div));

    loop {
        let line = mal_readline("user> ");
        if let None = line {
            break;
        }
        let result = rep(line.unwrap(), &repl_env);
        match result {
            Ok(message) => println!("{}", message),
            Err(message) => println!("{}", message),
        }
    }
}
