use std::fmt;
use std::convert::From;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use printer::pr_str;

use env::Env;
use self::MalType::*;
use self::MalError::*;

pub type MalResult = Result<MalType, MalError>;
type MalF = fn(args: Vec<MalType>) -> MalResult;
type MalMeta = Box<Option<MalType>>;

#[derive(PartialEq, Eq, Clone)]
pub enum MalType {
    MalList(Vec<MalType>, MalMeta),
    MalVector(Vec<MalType>, MalMeta),
    MalHashMap(HashMap<MalHashMapKey, MalType>, MalMeta), // Key is MalType::MalString or MalType::MalKeyword
    MalNumber(i64),
    MalString(String),
    MalKeyword(String),
    MalSymbol(String),
    MalNil,
    MalBool(bool),
    MalFunc(MalFuncData, MalMeta),
    MalAtom(Rc<RefCell<MalType>>),
}

impl fmt::Debug for MalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", pr_str(self, true))
    }
}

impl MalType {
    pub fn hash_key(&self) -> Result<MalHashMapKey, MalError> {
        match *self {
            MalString(ref v) => Ok(MalHashMapKey::MalString(v.to_string())),
            MalKeyword(ref v) => Ok(MalHashMapKey::MalKeyword(v.to_string())),
            _ => {
                mal_error!(format!("unexpected symbol. expected: string or keyword, actual: {:?}",
                                   self))
            }
        }
    }
}

pub fn vec_to_hash_map(args: Vec<MalType>) -> MalResult {
    if args.len() % 2 != 0 {
        return mal_error!(format!("unexpected hash-map state. length: {}", args.len()));
    }

    let mut new_hash_map: HashMap<MalHashMapKey, MalType> = HashMap::new();

    for kv in args.chunks(2) {
        let key = &kv[0];
        let value = &kv[1];

        let key = match *key {
            MalString(ref v) => MalHashMapKey::MalString(v.to_string()),
            MalKeyword(ref v) => MalHashMapKey::MalKeyword(v.to_string()),
            _ => {
                return mal_error!(format!("unexpected symbol. expected: string or keyword, \
                                           actual: {:?}",
                                          key))
            }
        };

        new_hash_map.insert(key, value.clone());
    }

    Ok(MalHashMap(new_hash_map, Box::new(None)))
}

// I want to use MalKeyword and MalSymbol to HashMap keys.
// But MalHashMap can't be a Key of HashMap. (recursive)
// I decided to extract key enum to use HashMap of key.
#[derive(PartialEq, Eq,Hash, Clone)]
pub enum MalHashMapKey {
    MalKeyword(String),
    MalString(String),
}

pub enum MalError {
    ErrorMessage(String),
    ThrowAST(MalType),
}

impl From<String> for MalError {
    fn from(err: String) -> MalError {
        ErrorMessage(err)
    }
}

impl From<MalType> for MalError {
    fn from(ast: MalType) -> MalError {
        ThrowAST(ast)
    }
}

impl fmt::Debug for MalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match *self {
            ErrorMessage(ref message) => message.clone(),
            ThrowAST(ref ast) => format!("receive exception: {}", pr_str(ast, true)),
        };
        write!(f, "{}", message)
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct MalFuncData {
    // from bootstrap
    func: Option<MalF>,

    // from lisp
    closure: Option<Box<MalFuncDataFromLisp>>,

    // eval
    eval: Option<Box<MalFuncDataForEval>>,
}

#[derive(PartialEq, Eq, Clone)]
struct MalFuncDataFromLisp {
    eval: fn(ast: MalType, env: Env) -> MalResult,
    env: Env,
    params: Vec<String>,
    ast: MalType,
    is_macro: bool,
}

#[derive(PartialEq, Eq, Clone)]
struct MalFuncDataForEval {
    eval: fn(ast: MalType, env: Env) -> MalResult,
    env: Env,
}

impl MalFuncData {
    pub fn apply(&self, args: Vec<MalType>) -> MalResult {
        if let Some(func) = self.func {
            func(args)
        } else if let Some(ref container) = self.closure {
            let new_env = Env::new(Some(container.env.clone()), container.params.clone(), args)?;
            (container.eval)(container.ast.clone(), new_env)
        } else if let Some(ref container) = self.eval {
            if args.len() != 1 {
                return mal_error!(format!("unexpected argument length. expected: 1, actual: {}",
                                          args.len()));
            }
            (container.eval)((&args[0]).clone(), container.env.clone())
        } else {
            mal_error!("undefined function".to_string())
        }
    }

    pub fn tco_apply(&self, args: Vec<MalType>) -> Result<Option<(MalType, Env)>, String> {
        if let Some(ref data) = self.closure {
            let new_env = Env::new(Some(data.env.clone()), data.params.clone(), args)?;
            Ok(Some((data.ast.clone(), new_env)))
        } else {
            Ok(None)
        }
    }

    pub fn is_macro(&self) -> bool {
        if let Some(ref container) = self.closure {
            container.is_macro
        } else {
            false
        }
    }
}

pub fn func_from_bootstrap(f: MalF) -> MalType {
    MalFunc(MalFuncData {
                func: Some(f),
                closure: None,
                eval: None,
            },
            Box::new(None))
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
            return mal_error!("bind should be symbol".to_string());
        }
    }

    Ok(MalFunc(MalFuncData {
                   func: None,
                   closure: Some(Box::new(MalFuncDataFromLisp {
                       eval: eval,
                       env: env,
                       ast: exprs,
                       params: bind_strs,
                       is_macro: false,
                   })),
                   eval: None,
               },
               Box::new(None)))
}

pub fn macro_from_lisp(func_data: MalFuncData) -> MalResult {
    let data = match func_data.closure {
        Some(v) => v,
        None => return mal_error!("unexpected function pattern".to_string()),
    };

    Ok(MalFunc(MalFuncData {
                   func: None,
                   closure: Some(Box::new(MalFuncDataFromLisp {
                       eval: data.clone().eval,
                       env: data.clone().env,
                       ast: data.clone().ast,
                       params: data.clone().params,
                       is_macro: true,
                   })),
                   eval: None,
               },
               Box::new(None)))
}

pub fn func_for_eval(eval: fn(ast: MalType, env: Env) -> MalResult, env: Env) -> MalType {
    MalFunc(MalFuncData {
                func: None,
                closure: None,
                eval: Some(Box::new(MalFuncDataForEval {
                    eval: eval,
                    env: env,
                })),
            },
            Box::new(None))
}
