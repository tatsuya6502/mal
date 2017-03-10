use std::fmt;

use printer::pr_str;

use env::Env;
use self::MalType::*;

pub type MalResult = Result<MalType, String>;
type MalF = fn(args: Vec<MalType>) -> MalResult;

#[derive(PartialEq, Eq, Clone)]
pub enum MalType {
    MalList(Vec<MalType>),
    MalVector(Vec<MalType>),
    MalHashMap(Vec<MalType>), // Mal's HashMap is immutable. odd value is key, even value is value.
    MalNumber(i64),
    MalString(String),
    MalKeyword(String),
    MalSymbol(String),
    MalNil,
    MalBool(bool),
    MalFunc(MalFuncData),
}

impl fmt::Debug for MalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", pr_str(self, true))
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct MalFuncData {
    // from bootstrap
    func: Option<MalF>,

    // from lisp
    closure: Option<Box<MalFuncDataFromLisp>>,
}

#[derive(PartialEq, Eq, Clone)]
struct MalFuncDataFromLisp {
    eval: fn(ast: MalType, env: Env) -> MalResult,
    env: Env,
    params: Vec<String>,
    ast: MalType,
}

impl MalFuncData {
    pub fn apply(&self, args: Vec<MalType>) -> MalResult {
        if let Some(func) = self.func {
            func(args)
        } else if let Some(ref container) = self.closure {
            let new_env =
                try!(Env::new(Some(container.env.clone()), container.params.clone(), args));
            (container.eval)(container.ast.clone(), new_env)
        } else {
            Err("undefined function".to_string())
        }
    }

    pub fn tco_apply(&self, args: Vec<MalType>) -> Result<Option<(MalType, Env)>, String> {
        if let Some(ref data) = self.closure {
            let new_env = try!(Env::new(Some(data.env.clone()), data.params.clone(), args));
            Ok(Some((data.ast.clone(), new_env)))
        } else {
            Ok(None)
        }
    }
}

pub fn func_from_bootstrap(f: MalF) -> MalType {
    MalFunc(MalFuncData {
        func: Some(f),
        closure: None,
    })
}

pub fn func_from_lisp(eval: fn(ast: MalType, env: Env) -> MalResult,
                      env: Env,
                      binds: Vec<MalType>,
                      exprs: MalType)
                      -> MalResult {
    let mut bind_strs: Vec<String> = vec![];
    for bind in binds {
        if let MalSymbol(v) = bind {
            bind_strs.push(v);
        } else {
            return Err("bind should be symbol".to_string());
        }
    }

    Ok(MalFunc(MalFuncData {
        func: None,
        closure: Some(Box::new(MalFuncDataFromLisp {
            eval: eval,
            env: env,
            ast: exprs,
            params: bind_strs,
        })),
    }))
}
