use std::collections::HashMap;

use readline::mal_readline;

use types;
use types::{MalType, MalError, MalResult, MalHashMapKey};
use types::MalType::*;
use env::Env;
use reader::read_str;
use printer::{pr_str, println};

// READ
fn read(str: String) -> MalResult {
    read_str(str).or_else(|e| mal_error!(e))
}

fn eval_ast(ast: MalType, env: Env) -> MalResult {
    match ast {
        MalSymbol(ref v) => {
            match env.clone().get(v.to_string()) {
                Some(ast) => Ok(ast.clone()),
                None => return mal_error!(format!("'{}' not found", v)),
            }
        }
        MalList(list, _) => {
            let mut new_list = vec![];
            for ast in list {
                new_list.push(try!(eval(ast, env.clone())));
            }
            Ok(MalList(new_list, Box::new(None)))
        }
        MalVector(list, _) => {
            let mut new_list = vec![];
            for ast in list {
                new_list.push(try!(eval(ast, env.clone())));
            }
            Ok(MalVector(new_list, Box::new(None)))
        }
        MalHashMap(hash_map, _) => {
            let mut new_hash_map: HashMap<MalHashMapKey, MalType> = HashMap::new();
            for (key, value) in hash_map.iter() {
                let value = try!(eval(value.clone(), env.clone()));
                new_hash_map.insert(key.clone(), value);
            }

            Ok(MalHashMap(new_hash_map, Box::new(None)))
        }
        v => Ok(v),
    }
}

// EVAL
fn eval(ast: MalType, env: Env) -> MalResult {
    let list = match ast {
        MalList(list, _) => list,
        _ => return eval_ast(ast, env),
    };
    if list.len() == 0 {
        return Ok(MalList(list, Box::new(None)));
    }

    {
        let a0 = list.get(0).unwrap();
        match a0 {
            &MalSymbol(ref v) if v == "def!" => {
                let key = &list[1];
                let key = match key {
                    &MalSymbol(ref v) => v,
                    _ => {
                        return mal_error!(format!("unexpected symbol. expected: symbol, actual: \
                                                   {:?}",
                                                  key))
                    }
                };
                let value = &list[2];
                let ret = try!(eval(value.clone(), env.clone()));
                return Ok(env.set(key.to_string(), ret));
            }
            &MalSymbol(ref v) if v == "let*" => {
                let let_env = try!(Env::new(Some(env), vec![], vec![]));
                let pairs = &list[1];
                let expr = &list[2];
                let list = seq!(pairs.clone());
                for kv in list.chunks(2) {
                    let key = &kv[0];
                    let value = &kv[1];
                    let key = match key {
                        &MalSymbol(ref v) => v,
                        _ => {
                            return mal_error!(format!("unexpected symbol. expected: symbol, \
                                                       actual: {:?}",
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

    let ast = try!(eval_ast(MalList(list, Box::new(None)), env.clone()));
    let list = seq!(ast);
    if list.len() == 0 {
        return mal_error!("unexpected state: len == 0".to_string());
    }

    let f = &list[0];
    let f = match f {
        &MalFunc(ref f, _) => f,
        _ => return mal_error!(format!("unexpected symbol. expected: function, actual: {:?}", f)),
    };
    f.apply((&list[1..]).to_vec())
}

// PRINT
fn print(exp: MalType) -> Result<String, MalError> {
    Ok(pr_str(&exp, true))
}

pub fn rep(str: &str, env: &Env) -> Result<String, MalError> {
    let ast = try!(read(str.to_string()));
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

pub fn run() {
    let repl_env = Env::new(None, vec![], vec![]).unwrap();
    repl_env.set("+".to_string(), types::func_from_bootstrap(add));
    repl_env.set("-".to_string(), types::func_from_bootstrap(sub));
    repl_env.set("*".to_string(), types::func_from_bootstrap(mul));
    repl_env.set("/".to_string(), types::func_from_bootstrap(div));

    loop {
        let line = mal_readline("user> ");
        if let None = line {
            break;
        }
        let result = rep(&line.unwrap(), &repl_env);
        match result {
            Ok(message) => println(message),
            Err(MalError::ErrorMessage(message)) => println(message),
            Err(MalError::ThrowAST(ref ast)) => {
                println(format!("receive exception: {}", pr_str(ast, true)))
            }
        }
    }
}
