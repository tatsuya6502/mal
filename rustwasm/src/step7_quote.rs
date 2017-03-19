#[cfg(not(target_arch="wasm32"))]
use std::process;
use std::collections::HashMap;

use readline::mal_readline;

use types::{MalType, MalError, MalResult, MalHashMapKey};
use types::MalType::*;
use types::{func_from_lisp, func_for_eval};
use core;
use env::Env;
use reader::read_str;
use printer::{pr_str, println};

// READ
fn read(str: String) -> MalResult {
    read_str(str).or_else(|e| mal_error!(e))
}

fn is_pair(ast: MalType) -> bool {
    match ast {
        MalList(list, _) |
        MalVector(list, _) => !list.is_empty(),
        _ => false,
    }
}

fn quasiquote(ast: MalType) -> MalResult {
    if !is_pair(ast.clone()) {
        return Ok(MalList(vec![MalSymbol("quote".to_string()), ast.clone()],
                          Box::new(None)));
    }

    let list = seq!(ast);

    let arg1 = match list.get(0) {
        Some(ast) => ast,
        None => return mal_error!("quasiquote: 1 or 2 argument(s) required".to_string()),
    };
    if let MalSymbol(ref symbol) = *arg1 {
        if symbol == "unquote" {
            match list.get(1) {
                Some(ast) => return Ok(ast.clone()),
                None => return mal_error!("unquote: 1 argument required".to_string()),
            };
        }
    }

    if is_pair(arg1.clone()) {
        let arg1_list = seq!(arg1.clone());
        if let Some(arg11) = arg1_list.get(0) {
            if let MalSymbol(ref symbol) = *arg11 {
                if symbol == "splice-unquote" {
                    let arg12 = match arg1_list.get(1) {
                        Some(ast) => ast,
                        None => {
                            return mal_error!("splice-unquote: 1 argument required".to_string())
                        }
                    };
                    return Ok(MalList(vec![MalSymbol("concat".to_string()),
                                           arg12.clone(),
                                           try!(quasiquote(MalList((&list[1..]).to_vec(),
                                                                   Box::new(None))))],
                                      Box::new(None)));
                }
            }
        };

    }

    Ok(MalList(vec![MalSymbol("cons".to_string()),
                    try!(quasiquote(arg1.clone())),
                    try!(quasiquote(MalList((&list[1..]).to_vec(), Box::new(None))))],
               Box::new(None)))
}

fn eval_ast(ast: MalType, env: &Env) -> MalResult {
    match ast {
        MalSymbol(ref v) => {
            match env.get(v.to_string()) {
                Some(ast) => Ok(ast.clone()),
                None => mal_error!(format!("'{}' not found", v)),
            }
        }
        MalList(list, _) => {
            let mut new_list = Vec::new();
            for ast in list {
                new_list.push(eval(ast, env.clone())?);
            }
            Ok(MalList(new_list, Box::new(None)))
        }
        MalVector(list, _) => {
            let mut new_list = Vec::new();
            for ast in list {
                new_list.push(eval(ast, env.clone())?);
            }
            Ok(MalVector(new_list, Box::new(None)))
        }
        MalHashMap(hash_map, _) => {
            let mut new_hash_map: HashMap<MalHashMapKey, MalType> = HashMap::new();
            for (key, value) in hash_map {
                let value = eval(value.clone(), env.clone())?;
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
            _ => return eval_ast(ast.clone(), &env),
        };

        if list.is_empty() {
            return Ok(MalList(list, Box::new(None)));
        }

        {
            let a0 = &list[0];
            match *a0 {
                MalSymbol(ref v) if v == "def!" => {
                    let key = &list[1];
                    let key = match *key {
                        MalSymbol(ref v) => v,
                        _ => {
                            return mal_error!(format!("unexpected symbol. expected: symbol, \
                                                       actual: {:?}",
                                                      key))
                        }
                    };
                    let value = &list[2];
                    let ret = eval(value.clone(), env.clone())?;
                    return Ok(env.set(key.to_string(), ret));
                }
                MalSymbol(ref v) if v == "let*" => {
                    env = Env::new(Some(env.clone()), Vec::new(), Vec::new())?;
                    let pairs = &list[1];
                    let expr = &list[2];
                    let list = seq!(pairs.clone());
                    for kv in list.chunks(2) {
                        let key = &kv[0];
                        let value = &kv[1];
                        let key = match *key {
                            MalSymbol(ref v) => v,
                            _ => {
                                return mal_error!(format!("unexpected symbol. expected: symbol, \
                                                           actual: {:?}",
                                                          key))
                            }
                        };
                        env.set(key.to_string(), eval(value.clone(), env.clone())?);
                    }

                    ast = expr.clone();
                    continue 'tco;
                }
                MalSymbol(ref v) if v == "quote" => {
                    let arg = list.get(1);
                    let arg = match arg {
                        Some(v) => v,
                        None => return mal_error!("quote argument is required".to_string()),
                    };
                    return Ok(arg.clone());
                }
                MalSymbol(ref v) if v == "quasiquote" => {
                    let arg = list.get(1);
                    let arg = match arg {
                        Some(v) => v,
                        None => return mal_error!("quasiquote argument is required".to_string()),
                    };
                    ast = quasiquote(arg.clone())?;
                    continue 'tco;
                }
                MalSymbol(ref v) if v == "do" => {
                    let len = list.len();
                    let exprs = &list[1..(len - 1)];
                    eval_ast(MalList(exprs.to_vec(), Box::new(None)), &env)?;
                    ast = list[list.len() - 1].clone();
                    continue 'tco;
                }
                MalSymbol(ref v) if v == "if" => {
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

                    let b = match eval(cond.clone(), env.clone())? {
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
                MalSymbol(ref v) if v == "fn*" => {
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

        let ret = eval_ast(MalList(list, Box::new(None)), &env)?;
        let list = seq!(ret);
        if list.is_empty() {
            return mal_error!("unexpected state: len == 0".to_string());
        }

        let f = &list[0];
        let args = (&list[1..]).to_vec();
        let f = match *f {
            MalFunc(ref f, _) => f,
            _ => {
                return mal_error!(format!("unexpected symbol. expected: function, actual: {:?}", f))
            }
        };
        if let Some(v) = f.tco_apply(args.clone())? {
            ast = v.0;
            env = v.1;
            continue 'tco;
        }
        return f.apply(args);
    }
}

// PRINT
fn print(exp: &MalType) -> Result<String, MalError> {
    Ok(pr_str(exp, true))
}

pub fn rep(str: &str, env: &Env) -> Result<String, MalError> {
    let ast = try!(read(str.to_string()));
    let exp = try!(eval(ast, env.clone()));
    print(&exp)
}

fn rep_or_panic(str: &str, env: &Env, line: u32) {
    rep(str, env).expect(&format!("rep on `{}` failed at {}:{}", str, file!(), line));
}

pub fn new_repl_env() -> Env {
    let repl_env = Env::new(None, Vec::new(), Vec::new()).unwrap();

    // core.EXT: defined using Racket
    for (key, value) in core::ns() {
        repl_env.set(key.to_string(), value.clone());
    }
    repl_env.set("*ARGV*".to_string(), MalList(Vec::new(), Box::new(None)));
    repl_env.set("eval".to_string(), func_for_eval(eval, repl_env.clone()));

    // core.mal: defined using the language itself
    rep_or_panic("(def! not (fn* (a) (if a false true)))", &repl_env, line!());
    rep_or_panic(r##"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) ")")))))"##,
                 &repl_env,
                 line!());

    repl_env
}

#[cfg(not(target_arch="wasm32"))]
fn load_file(source: &str, env: &Env) {
    let load = format!(r##"(load-file "{}")"##, source);
    let ret = rep(&load, env);
    match ret {
        Ok(_) => process::exit(0),
        Err(v) => {
            println!("{:?}", v);
            process::exit(1);
        }
    };
}

#[cfg(target_arch="wasm32")]
fn load_file(_source: &str, _env: &Env) {
    unimplemented!()
}

pub fn run(args: &[String]) {
    let repl_env = new_repl_env();

    if 2 <= args.len() {
        let source = &args[1];

        let args = args.iter().skip(2).map(|str| MalString(str.clone())).collect::<Vec<_>>();
        repl_env.set("*ARGV*".to_string(), MalList(args, Box::new(None)));

        load_file(source, &repl_env);
        return;
    }

    loop {
        let line = mal_readline("user> ");
        if line.is_none() {
            break;
        }
        let result = rep(&line.unwrap(), &repl_env);
        match result {
            Ok(message) |
            Err(MalError::ErrorMessage(message)) => println(message),
            Err(MalError::ThrowAST(ref ast)) => {
                println(format!("receive exception: {}", pr_str(ast, true)))
            }
        };
    }
}
