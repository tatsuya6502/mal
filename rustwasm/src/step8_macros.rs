#[cfg(not(target_arch="wasm32"))]
use std::process;
use std::collections::HashMap;

use readline::mal_readline;

use types::{MalType, MalError, MalResult, MalHashMapKey};
use types::MalType::*;
use types::{func_from_lisp, func_for_eval, macro_from_lisp};
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
        MalVector(list, _) => list.len() != 0,
        _ => false,
    }
}

fn is_macro(ast: MalType, env: Env) -> bool {
    let list = match ast {
        MalList(list, _) |
        MalVector(list, _) => list,
        _ => return false,
    };

    let symbol = match list.get(0) {
        Some(&MalSymbol(ref symbol)) => symbol,
        _ => return false,
    };

    let env = match env.find(symbol.clone()) {
        Some(v) => v,
        None => return false,
    };

    let f = match env.get(symbol.clone()) {
        Some(v) => v,
        None => return false,
    };
    let f = match f {
        MalFunc(v, _) => v,
        _ => return false,
    };

    f.is_macro()
}

fn macroexpand(ast: MalType, env: Env) -> MalResult {
    let mut ast = ast;
    while is_macro(ast.clone(), env.clone()) {
        let list = seq!(ast);

        if list.len() < 1 {
            return mal_error!("macroexpand: 1 or more argument(s) is required".to_string());
        }

        let symbol = list.get(0).unwrap();
        let symbol = match symbol {
            &MalSymbol(ref v) => v,
            _ => {
                return mal_error!(format!("unexpected symbol. expected: symbol, actual: {:?}",
                                          symbol))
            }
        };

        let f = env.get(symbol.to_string());
        let f = match f {
            Some(MalFunc(ref v, _)) => v,
            _ => {
                return mal_error!(format!("unexpected symbol. expected: function, actual: {:?}", f))
            }
        };

        ast = try!(f.apply((&list[1..]).to_vec()));
    }

    Ok(ast)
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
    if let &MalSymbol(ref symbol) = arg1 {
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
            if let &MalSymbol(ref symbol) = arg11 {
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
        match ast {
            MalList(_, _) => {}
            _ => return eval_ast(ast.clone(), env),
        };

        ast = try!(macroexpand(ast, env.clone()));
        let list = match ast {
            MalList(list, _) => list,
            _ => return eval_ast(ast.clone(), env),
        };

        if list.is_empty() {
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
                        env.set(key.to_string(), try!(eval(value.clone(), env.clone())));
                    }

                    ast = expr.clone();
                    continue 'tco;
                }
                &MalSymbol(ref v) if v == "quote" => {
                    let arg = list.get(1);
                    let arg = match arg {
                        Some(v) => v,
                        None => return mal_error!("quote argument is required".to_string()),
                    };
                    return Ok(arg.clone());
                }
                &MalSymbol(ref v) if v == "quasiquote" => {
                    let arg = list.get(1);
                    let arg = match arg {
                        Some(v) => v,
                        None => return mal_error!("quasiquote argument is required".to_string()),
                    };
                    ast = try!(quasiquote(arg.clone()));
                    continue 'tco;
                }
                &MalSymbol(ref v) if v == "defmacro!" => {
                    let key = list.get(1);
                    let key = match key {
                        Some(v) => v,
                        None => return mal_error!("key is required".to_string()),
                    };
                    let symbol = match key {
                        &MalSymbol(ref str) => str,
                        _ => {
                            return mal_error!(format!("unexpected symbol. expected: symbol, \
                                                       actual: {:?}",
                                                      key))
                        }
                    };
                    let value = list.get(2);
                    let value = match value {
                        Some(v) => v,
                        None => return mal_error!("value expr is required".to_string()),
                    };

                    let f = try!(eval(value.clone(), env.clone()));
                    let f = match f {
                        MalFunc(ref v, _) => v,
                        _ => {
                            return mal_error!(format!("unexpected symbol. expected: function, \
                                                       actual: {:?}",
                                                      f))
                        }
                    };
                    let f = try!(macro_from_lisp(f.clone()));
                    return Ok(env.set(symbol.to_string(), f));
                }
                &MalSymbol(ref v) if v == "macroexpand" => {
                    let v = list.get(1);
                    let v = match v {
                        Some(v) => v,
                        None => return mal_error!("value is required".to_string()),
                    };
                    return macroexpand(v.clone(), env);
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
        if list.is_empty() {
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

pub fn rep(str: &str, env: &Env) -> Result<String, MalError> {
    let ast = try!(read(str.to_string()));
    let exp = try!(eval(ast, env.clone()));
    print(exp)
}

fn rep_or_panic(str: &str, env: &Env, line: u32) {
    rep(str, env).expect(&format!("rep on `{}` failed at {}:{}", str, file!(), line));
}

pub fn new_repl_env() -> Env {
    let repl_env = Env::new(None, vec![], vec![]).unwrap();

    // core.EXT: defined using Racket
    for (key, value) in core::ns().iter() {
        repl_env.set(key.to_string(), value.clone());
    }
    repl_env.set("*ARGV*".to_string(), MalList(vec![], Box::new(None)));
    repl_env.set("eval".to_string(), func_for_eval(eval, repl_env.clone()));

    // core.mal: defined using the language itself
    rep_or_panic("(def! not (fn* (a) (if a false true)))",
                 &repl_env, line!());
    rep_or_panic(r##"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) ")")))))"##,
                 &repl_env, line!());
    rep_or_panic(r##"(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw "odd number of forms to cond")) (cons 'cond (rest (rest xs)))))))"##,
                 &repl_env, line!());
    rep_or_panic(r##"(defmacro! or (fn* (& xs) (if (empty? xs) nil (if (= 1 (count xs)) (first xs) `(let* (or_FIXME ~(first xs)) (if or_FIXME or_FIXME (or ~@(rest xs))))))))"##,
                 &repl_env, line!());

    repl_env
}

#[cfg(not(target_arch="wasm32"))]
fn load_file(source: String, env: &Env) {
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
fn load_file(_source: String, _env: &Env) {
    unimplemented!()
}

pub fn run(args: Vec<String>) {
    let repl_env = new_repl_env();

    if 2 <= args.len() {
        let source = args.get(1).unwrap();

        let args = args.iter().skip(2).map(|str| MalString(str.clone())).collect::<Vec<_>>();
        repl_env.set("*ARGV*".to_string(), MalList(args, Box::new(None)));

        load_file(source.to_string(), &repl_env);
        return;
    }

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
