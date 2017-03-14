use std::collections::HashMap;

use readline::mal_readline;

use types::{MalType, MalError, MalResult, MalHashMapKey};
use types::MalType::*;
use types::func_from_lisp;
use core;
use env::Env;
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
    let mut ast: MalType = ast;
    let mut env: Env = env;

    'tco: loop {
        let list = match ast {
            MalList(list, _) => list,
            _ => return eval_ast(ast.clone(), env),
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
                            return mal_error!(format!("unexpected symbol. expected: symbol, \
                                                       actual: {:?}",
                                                      key))
                        }
                    };
                    let value = &list[2];
                    let ret = try!(eval(value.clone(), env.clone()));
                    return Ok(env.set(key.to_string(), ret));
                }
                &MalSymbol(ref v) if v == "let*" => {
                    env = try!(Env::new(Some(env.clone()), vec![], vec![]));
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
                                return mal_error!(format!("unexpected symbol. expected: symbol, \
                                                           actual: {:?}",
                                                          key))
                            }
                        };
                        env.set(key.to_string(), try!(eval(value.clone(), env.clone())));
                    }

                    ast = expr.clone();
                    continue 'tco;
                }
                &MalSymbol(ref v) if v == "do" => {
                    let len = list.len();
                    let exprs = &list[1..(len - 1)];
                    try!(eval_ast(MalList(exprs.to_vec(), Box::new(None)), env.clone()));
                    ast = list[list.len() - 1].clone();
                    continue 'tco;
                }
                &MalSymbol(ref v) if v == "if" => {
                    let cond = list.get(1);
                    let cond = match cond {
                        Some(v) => v,
                        None => return mal_error!("cond expr is required".to_string()),
                    };
                    let then_expr = list.get(2);
                    let then_expr = match then_expr {
                        Some(v) => v,
                        None => return mal_error!("then expr is required".to_string()),
                    };
                    let else_expr = list.get(3);

                    let b = match try!(eval(cond.clone(), env.clone())) {
                        MalBool(false) | MalNil => false,
                        _ => true,
                    };
                    if b {
                        ast = then_expr.clone();
                    } else if let Some(else_expr) = else_expr {
                        ast = else_expr.clone();
                    } else {
                        ast = MalNil;
                    }
                    continue 'tco;
                }
                &MalSymbol(ref v) if v == "fn*" => {
                    let binds = list.get(1);
                    let binds = match binds {
                        Some(v) => v,
                        None => return mal_error!("binds is required".to_string()),
                    };
                    let binds = seq!(binds.clone());

                    let exprs = list.get(2);
                    let exprs = match exprs {
                        Some(v) => v,
                        None => return mal_error!("exprs is required".to_string()),
                    };

                    return func_from_lisp(eval, env, binds, exprs.clone());
                }
                _ => {}
            };
        }

        let ret = try!(eval_ast(MalList(list, Box::new(None)), env.clone()));
        let list = seq!(ret);
        if list.len() == 0 {
            return mal_error!("unexpected state: len == 0".to_string());
        }

        let f = &list[0];
        let args = (&list[1..]).to_vec();
        let f = match f {
            &MalFunc(ref f, _) => f,
            _ => {
                return mal_error!(format!("unexpected symbol. expected: function, actual: {:?}", f))
            }
        };
        if let Some(v) = try!(f.tco_apply(args.clone())) {
            ast = v.0;
            env = v.1;
            continue 'tco;
        }
        return f.apply(args);
    }
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

pub fn new_repl_env() -> Env {
    let repl_env = Env::new(None, vec![], vec![]).unwrap();

    // core.EXT: defined using Racket
    for (key, value) in core::ns().iter() {
        repl_env.set(key.to_string(), value.clone());
    }

    // core.mal: defined using the language itself
    match rep("(def! not (fn* (a) (if a false true)))".to_string(),
              &repl_env) {
        Err(x) => panic!("{:?}", x),
        _ => {}
    };

    repl_env
}

pub fn run() {
    let repl_env = new_repl_env();

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
