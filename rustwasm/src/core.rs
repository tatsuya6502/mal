use std::error::Error;
use std::collections::HashMap;

use std::io::Read;
use std::fs::File;
use std::path::Path;

use std::rc::Rc;
use std::cell::RefCell;

use time;

use readline::mal_readline;

use types::{func_from_bootstrap, vec_to_hash_map};
use types::MalType;
use types::MalType::*;
use types::MalHashMapKey;
use types::MalResult;

use reader::read_str;

use printer::{pr_str as printer_pr_str, println as printer_println};

fn equal(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("=: 2 arguments required".to_string());
    }

    let v0 = &args[0];
    let v1 = &args[1];

    match (v0, v1) {
        (&MalList(ref a, _), &MalVector(ref b, _)) |
        (&MalVector(ref a, _), &MalList(ref b, _)) => {
            if a.len() != b.len() {
                return Ok(MalBool(false));
            }
            for i in 0..a.len() {
                let a_v = &a[i];
                let b_v = &b[i];
                let ret = equal(vec![a_v.clone(), b_v.clone()])?;
                match ret {
                    MalBool(true) => continue,
                    MalBool(false) => return Ok(MalBool(false)),
                    v => {
                        return mal_error!(format!("unexpected symbol. expected: bool, actual: \
                                                   {:?}",
                                                  v))
                    }
                }
            }
            Ok(MalBool(true))
        }
        (&MalHashMap(ref a, _), &MalHashMap(ref b, _)) => {
            if a.len() != b.len() {
                return Ok(MalBool(false));
            }

            for (a_key, a_v) in a {
                let b_v = b.get(a_key);
                let b_v = match b_v {
                    Some(v) => v,
                    None => return Ok(MalBool(false)),
                };
                let ret = equal(vec![a_v.clone(), b_v.clone()])?;
                match ret {
                    MalBool(true) => continue,
                    MalBool(false) => return Ok(MalBool(false)),
                    v => {
                        return mal_error!(format!("unexpected symbol. expected: bool, actual: \
                                                   {:?}",
                                                  v))
                    }
                }
            }

            Ok(MalBool(true))
        }
        (v0, v1) => Ok(MalBool(v0 == v1)),
    }
}

fn throw(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("throw: 1 argument required".to_string());
    }
    let v = &args[0];

    mal_error!(v.clone())
}

fn is_nil(args: Vec<MalType>) -> MalResult {
    let v = match args.get(0) {
        Some(&MalNil) => true,
        _ => false,
    };

    Ok(MalBool(v))
}

fn is_true(args: Vec<MalType>) -> MalResult {
    let v = match args.get(0) {
        Some(&MalBool(true)) => true,
        _ => false,
    };

    Ok(MalBool(v))
}

fn is_false(args: Vec<MalType>) -> MalResult {
    let v = match args.get(0) {
        Some(&MalBool(false)) => true,
        _ => false,
    };

    Ok(MalBool(v))
}

fn is_string(args: Vec<MalType>) -> MalResult {
    let v = match args.get(0) {
        Some(&MalString(_)) => true,
        _ => false,
    };

    Ok(MalBool(v))
}

fn symbol(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("symbol: 1 argument required".to_string());
    }
    let v = &args[0];
    let v = match *v {
        MalString(ref v) => v,
        _ => return mal_error!(format!("unexpected symbol. expected: string, actual: {:?}", v)),
    };

    Ok(MalSymbol(v.to_string()))
}

fn is_symbol(args: Vec<MalType>) -> MalResult {
    let v = match args.get(0) {
        Some(&MalSymbol(_)) => true,
        _ => false,
    };

    Ok(MalBool(v))
}

fn keyword(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("keyword: 1 argument required".to_string());
    }
    let v = &args[0];
    let v = match *v {
        MalString(ref v) => v,
        _ => return mal_error!(format!("unexpected symbol. expected: string, actual: {:?}", v)),
    };

    Ok(MalKeyword(v.to_string()))
}

fn is_keyword(args: Vec<MalType>) -> MalResult {
    let v = match args.get(0) {
        Some(&MalKeyword(_)) => true,
        _ => false,
    };

    Ok(MalBool(v))
}

fn pr_str(args: Vec<MalType>) -> MalResult {
    let ret = args.iter().map(|v| printer_pr_str(v, true)).collect::<Vec<_>>().join(" ");
    Ok(MalString(ret))
}

fn str(args: Vec<MalType>) -> MalResult {
    let ret = args.iter().map(|v| printer_pr_str(v, false)).collect::<Vec<_>>().join("");
    Ok(MalString(ret))
}

fn prn(args: Vec<MalType>) -> MalResult {
    let ret = args.iter().map(|v| printer_pr_str(v, true)).collect::<Vec<_>>().join(" ");
    printer_println(ret);
    Ok(MalNil)
}

fn println(args: Vec<MalType>) -> MalResult {
    let ret = args.iter().map(|v| printer_pr_str(v, false)).collect::<Vec<_>>().join(" ");
    printer_println(ret);
    Ok(MalNil)
}

fn read_string(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("read-string: 1 argument required".to_string());
    }
    let v = &args[0];

    let str = match *v {
        MalString(ref str) => str,
        _ => return mal_error!(format!("unexpected symbol. expected: string, actual: {:?}", v)),
    };

    match read_str(str.to_string()) {
        Ok(v) => Ok(v),
        Err(v) => mal_error!(v),
    }
}

fn readline(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("readline: 1 argument required".to_string());
    }
    let v = &args[0];

    let str = match *v {
        MalString(ref str) => str,
        _ => return mal_error!(format!("unexpected symbol. expected: string, actual: {:?}", v)),
    };

    let ret = mal_readline(str);

    let ret = match ret {
        Some(v) => MalString(v),
        None => MalNil,
    };
    Ok(ret)
}

fn slurp(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("slurp: 1 argument required".to_string());
    }
    let v = &args[0];

    let str = match *v {
        MalString(ref str) => str,
        _ => return mal_error!(format!("unexpected symbol. expected: string, actual: {:?}", v)),
    };

    let mut file = match File::open(Path::new(str)) {
        Ok(v) => v,
        Err(why) => return mal_error!(format!("can't open {}, {}", str, why.description())),
    };

    let mut s = String::new();
    if let Err(why) = file.read_to_string(&mut s) {
        return mal_error!(format!("can't read {}, {}", str, why.description()));
    };

    Ok(MalString(s))
}

fn less_than(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("<: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalBool(a < b)),
        _ => {
            mal_error!(format!("unexpected symbol. expected: number & number, actual: {:?}",
                               args))
        }
    }
}

fn less_than_or_equal(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("<=: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalBool(a <= b)),
        _ => {
            mal_error!(format!("unexpected symbol. expected: number & number, actual: {:?}",
                               args))
        }
    }
}

fn greater_than(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!(">: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalBool(a > b)),
        _ => {
            mal_error!(format!("unexpected symbol. expected: number & number, actual: {:?}",
                               args))
        }
    }
}

fn greater_than_equal(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!(">=: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalBool(a >= b)),
        _ => {
            mal_error!(format!("unexpected symbol. expected: number & number, actual: {:?}",
                               args))
        }
    }
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

fn time_ms(_args: Vec<MalType>) -> MalResult {
    let current_time = time::get_time();
    let milliseconds = (current_time.sec * 1000) + (current_time.nsec as i64 / 1000 / 1000);

    Ok(MalNumber(milliseconds))
}

fn list(args: Vec<MalType>) -> MalResult {
    Ok(MalList(args, Box::new(None)))
}

fn is_list(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("list?: 1 argument required".to_string());
    }
    let v = &args[0];

    let ret = match *v {
        MalList(_, _) => true,
        _ => false,
    };
    Ok(MalBool(ret))
}

fn vector(args: Vec<MalType>) -> MalResult {
    Ok(MalVector(args, Box::new(None)))
}

fn is_vector(args: Vec<MalType>) -> MalResult {
    let v = match args.get(0) {
        Some(&MalVector(_, _)) => true,
        _ => false,
    };

    Ok(MalBool(v))
}

fn hash_map(args: Vec<MalType>) -> MalResult {
    vec_to_hash_map(args)
}

fn is_map(args: Vec<MalType>) -> MalResult {
    let v = match args.get(0) {
        Some(&MalHashMap(_, _)) => true,
        _ => false,
    };

    Ok(MalBool(v))
}

fn assoc(args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        return mal_error!("assoc: 1 or more argument(s) is required".to_string());
    }

    let hash_map = &args[0];
    let hash_map = match *hash_map {
        MalHashMap(ref v, _) => v.clone(),
        _ => {
            return mal_error!(format!("unexpected symbol. expected: hash-map, actual: {:?}",
                                      hash_map))
        }
    };

    let rest = (&args[1..]).to_vec();
    let rest = vec_to_hash_map(rest)?;
    let rest = match rest {
        MalHashMap(ref v, _) => v.clone(),
        _ => {
            return mal_error!(format!("unexpected symbol. expected: hash-map, actual: {:?}", rest))
        }
    };

    let mut new_hash_map: HashMap<MalHashMapKey, MalType> = HashMap::new();
    new_hash_map.extend(hash_map);
    new_hash_map.extend(rest);

    Ok(MalHashMap(new_hash_map, Box::new(None)))
}

fn dissoc(args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        return mal_error!("dissoc: 1 or more argument(s) is required".to_string());
    }

    let hash_map = &args[0];
    let hash_map = match *hash_map {
        MalHashMap(ref v, _) => v.clone(),
        _ => {
            return mal_error!(format!("unexpected symbol. expected: hash-map, actual: {:?}",
                                      hash_map))
        }
    };

    let rest = (&args[1..]).to_vec();

    let mut new_hash_map: HashMap<MalHashMapKey, MalType> = HashMap::new();
    new_hash_map.extend(hash_map);

    for i in 0..rest.len() {
        let key = &rest[i];
        let key = match *key {
            MalString(ref v) => MalHashMapKey::MalString(v.to_string()),
            MalKeyword(ref v) => MalHashMapKey::MalKeyword(v.to_string()),
            _ => {
                return mal_error!(format!("unexpected symbol. expected: string or keyword, \
                                           actual: {:?}",
                                          key))
            }
        };
        new_hash_map.remove(&key);
    }

    Ok(MalHashMap(new_hash_map, Box::new(None)))
}

fn get(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("get: 2 arguments required".to_string());
    }

    let v = &args[0];
    let hash_map = match *v {
        MalNil => return Ok(MalNil),
        MalHashMap(ref v, _) => v,
        _ => return mal_error!(format!("unexpected symbol. expected: hash-map, actual: {:?}", v)),
    };

    let key = &args[1];
    let key = match *key {
        MalString(ref v) => MalHashMapKey::MalString(v.to_string()),
        MalKeyword(ref v) => MalHashMapKey::MalKeyword(v.to_string()),
        _ => {
            return mal_error!(format!("unexpected symbol. expected: string or keyword, actual: \
                                       {:?}",
                                      v))
        }
    };
    let value = hash_map.get(&key);
    match value {
        Some(v) => Ok(v.clone()),
        None => Ok(MalNil),
    }
}

fn is_contains(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("contains?: 2 arguments required".to_string());
    }

    let v = &args[0];
    let hash_map = match *v {
        MalHashMap(ref v, _) => v,
        _ => return mal_error!(format!("unexpected symbol. expected: hash-map, actual: {:?}", v)),
    };

    let key = &args[1];
    let key = match *key {
        MalString(ref v) => MalHashMapKey::MalString(v.to_string()),
        MalKeyword(ref v) => MalHashMapKey::MalKeyword(v.to_string()),
        _ => {
            return mal_error!(format!("unexpected symbol. expected: string or keyword, actual: \
                                       {:?}",
                                      v))
        }
    };
    Ok(MalBool(hash_map.contains_key(&key)))
}

fn keys(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("keys: 1 argument required".to_string());
    }
    let v = &args[0];
    let hash_map = match *v {
        MalHashMap(ref v, _) => v,
        _ => return mal_error!(format!("unexpected symbol. expected: hash-map, actual: {:?}", v)),
    };

    let mut ret = vec![];
    for key in hash_map.keys() {
        let key = match *key {
            MalHashMapKey::MalString(ref v) => MalString(v.to_string()),
            MalHashMapKey::MalKeyword(ref v) => MalKeyword(v.to_string()),
        };
        ret.push(key.clone());
    }

    Ok(MalList(ret, Box::new(None)))
}

fn vals(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("vals: 1 argument required".to_string());
    }
    let v = &args[0];
    let hash_map = match *v {
        MalHashMap(ref v, _) => v,
        _ => return mal_error!(format!("unexpected symbol. expected: hash-map, actual: {:?}", v)),
    };

    let mut ret = vec![];
    for value in hash_map.values() {
        ret.push(value.clone());
    }

    Ok(MalList(ret, Box::new(None)))
}

fn is_sequential(args: Vec<MalType>) -> MalResult {
    let v = match args.get(0) {
        Some(&MalList(_, _)) |
        Some(&MalVector(_, _)) => true,
        _ => false,
    };

    Ok(MalBool(v))
}

fn cons(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("reset!: 2 arguments required".to_string());
    }
    let v = &args[0];
    let list = &args[1];

    let list = seq!(list.clone());

    let mut ret = vec![v.clone()];
    ret.extend(list.iter().cloned());

    Ok(MalList(ret, Box::new(None)))
}

fn concat(args: Vec<MalType>) -> MalResult {
    let mut list = vec![];

    for v in args {
        let v = seq!(v);
        list.extend(v);
    }

    Ok(MalList(list, Box::new(None)))
}

fn nth(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("nth: 2 arguments are required".to_string());
    }

    let list = &args[0];
    let idx = &args[1];

    let list = seq!(list.clone());

    let idx = match *idx {
        MalNumber(v) => v,
        _ => return mal_error!(format!("unexpected symbol. expected: number , actual: {:?}", idx)),
    };

    let v = match list.get(idx as usize) {
        Some(v) => v,
        None => return mal_error!("nth: index out of range".to_string()),
    };

    Ok(v.clone())
}

fn first(args: Vec<MalType>) -> MalResult {
    if args.len() < 1 {
        return mal_error!("first: 1 or more argument(s) is required".to_string());
    }

    let list = &args[0];

    if list == &MalNil {
        return Ok(MalNil);
    };

    let list = seq!(list.clone());

    let ret = list.get(0);
    let ret = match ret {
        Some(v) => v,
        None => return Ok(MalNil),
    };
    Ok(ret.clone())
}

fn rest(args: Vec<MalType>) -> MalResult {
    if args.len() < 1 {
        return mal_error!("rest: 1 or more argument(s) is required".to_string());
    }

    let list = &args[0];

    if list == &MalNil {
        return Ok(MalList(vec![], Box::new(None)));
    };

    let list = seq!(list.clone());
    if list.is_empty() {
        return Ok(MalList(vec![], Box::new(None)));
    }

    Ok(MalList((&list[1..]).to_vec(), Box::new(None)))
}

fn is_empty(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("empty?: 1 argument required".to_string());
    }
    let v = &args[0];

    let list = match *v {
        MalList(ref list, _) |
        MalVector(ref list, _) => list,
        _ => return Ok(MalBool(false)),
    };
    Ok(MalBool(list.is_empty()))
}

fn count(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("count: 1 argument required".to_string());
    }
    let v = &args[0];

    let list = match *v {
        MalList(ref list, _) |
        MalVector(ref list, _) => list,
        _ => return Ok(MalNumber(0)),
    };
    Ok(MalNumber(list.len() as i64))
}

fn apply(args: Vec<MalType>) -> MalResult {
    if args.is_empty() {
        return mal_error!("apply: 1 or more argument(s) is required".to_string());
    }
    let f = &args[0];
    let f = match *f {
        MalFunc(ref v, _) => v,
        _ => return mal_error!(format!("unexpected symbol. expected: function, actual: {:?}", f)),
    };

    let tail = &args[args.len() - 1];
    let tail = seq!(tail.clone());

    let mut args = (&args[1..args.len() - 1]).to_vec();
    args.extend(tail);

    f.apply(args)
}

fn map(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        return mal_error!("map: 2 or more arguments required".to_string());
    }

    let f = &args[0];
    let f = match *f {
        MalFunc(ref v, _) => v,
        _ => return mal_error!(format!("unexpected symbol. expected: function, actual: {:?}", f)),
    };

    let list = &args[1];
    let list = seq!(list.clone());

    let mut ret = vec![];
    for v in list {
        let v = f.apply(vec![v])?;
        ret.push(v);
    }

    Ok(MalList(ret, Box::new(None)))
}

fn conj(args: Vec<MalType>) -> MalResult {
    if args.len() < 1 {
        return mal_error!("conj: 1 or more argument(s) is required".to_string());
    }

    let list = &args[0];

    let ret = match *list {
        MalList(ref list, _) => {
            let mut ret_list: Vec<MalType> = vec![];
            for i in 1..args.len() {
                let v = &args[i];
                let mut temp_list: Vec<MalType> = vec![v.clone()];
                temp_list.extend(ret_list);
                ret_list = temp_list;
            }
            for i in 0..list.len() {
                ret_list.push(list[i].clone());
            }

            MalList(ret_list, Box::new(None))
        }
        MalVector(ref list, _) => {
            let mut ret_list: Vec<MalType> = vec![];
            ret_list.extend(list.clone());
            ret_list.extend((&args[1..]).to_vec());

            MalVector(ret_list, Box::new(None))
        }
        _ => {
            return mal_error!(format!("unexpected symbol. expected: list or vector, actual: {:?}",
                                      list))
        }
    };

    Ok(ret)
}

fn seq(args: Vec<MalType>) -> MalResult {
    if args.len() < 1 {
        return mal_error!("seq: 1 or more argument(s) is required".to_string());
    }

    let v = &args[0];

    let ret = match *v {
        MalList(ref list, _) |
        MalVector(ref list, _) => {
            if list.is_empty() {
                MalNil
            } else {
                MalList(list.clone(), Box::new(None))
            }
        }
        MalString(ref str) => {
            if str == "" {
                MalNil
            } else {
                let str = str.to_string();
                let strs = str.split("").collect::<Vec<_>>();
                // "abc" -> ["", "a", "b", "c", ""]
                let strs = (&strs[1..(strs.len() - 1)]).to_vec();
                let strs = strs.into_iter().map(|v| MalString(v.to_string())).collect();
                MalList(strs, Box::new(None))
            }
        }
        MalNil => MalNil,
        _ => {
            return mal_error!(format!("unexpected symbol. expected: list or vector or string or \
                                       nil, actual: {:?}",
                                      v))
        }
    };

    Ok(ret)
}

fn meta(args: Vec<MalType>) -> MalResult {
    if args.len() < 1 {
        return mal_error!("meta: 1 or more argument(s) is required".to_string());
    }

    let v = &args[0];

    let ret = match *v {
        MalList(_, ref meta) |MalVector(_, ref meta) | MalHashMap(_, ref meta)  | MalFunc(_, ref meta) => {
            if let Some(meta) = *meta.clone() {
                meta
            } else {
                MalNil
            }
        }
        _ => MalNil,
    };

    Ok(ret)
}

fn with_meta(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("with-meta: 2 arguments are required".to_string());
    }

    let v = &args[0];
    let meta = &args[1];

    let ret = match v {
        &MalList(ref v, _) => MalList(v.clone(), Box::new(Some(meta.clone()))),
        &MalVector(ref v, _) => MalVector(v.clone(), Box::new(Some(meta.clone()))),
        &MalHashMap(ref v, _) => MalHashMap(v.clone(), Box::new(Some(meta.clone()))),
        &MalFunc(ref v, _) => MalFunc(v.clone(), Box::new(Some(meta.clone()))),
        v => v.clone(),
    };

    Ok(ret)
}

fn atom(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("atom: 1 argument required".to_string());
    }
    let v = &args[0];
    Ok(MalAtom(Rc::new(RefCell::new(v.clone()))))
}

fn is_atom(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("atom?: 1 argument required".to_string());
    }
    let v = &args[0];

    let ret = match *v {
        MalAtom(_) => true,
        _ => false,
    };
    Ok(MalBool(ret))
}

fn deref(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return mal_error!("deref: 1 argument required".to_string());
    }
    let v = &args[0];

    let v = match v {
        &MalAtom(ref v) => v,
        v => return mal_error!(format!("unexpected symbol. expected: atom, actual: {:?}", v)),
    };
    Ok(v.borrow().clone())
}

fn reset(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return mal_error!("reset!: 2 arguments required".to_string());
    }
    let atom = &args[0];
    let v = &args[1];

    match atom {
        &MalAtom(ref atom) => {
            let mut atom = atom.borrow_mut();
            *atom = v.clone();
        }
        v => return mal_error!(format!("unexpected symbol. expected: atom, actual: {:?}", v)),
    };
    Ok(v.clone())
}

fn swap(args: Vec<MalType>) -> MalResult {
    if args.len() < 2 {
        return mal_error!("swap!: 2 or more arguments required".to_string());
    }

    let atom = &args[0];
    let atom_value = match atom {
        &MalAtom(ref v) => v,
        v => return mal_error!(format!("unexpected symbol. expected: atom, actual: {:?}", v)),
    };

    let f = &args[1];
    let f = match f {
        &MalFunc(ref v, _) => v,
        v => return mal_error!(format!("unexpected symbol. expected: function, actual: {:?}", v)),
    };

    let mut func_args: Vec<MalType> = vec![atom_value.borrow().clone()];
    func_args.extend((&args[2..]).iter().cloned());

    let result = f.apply(func_args)?;
    (*atom_value.borrow_mut()) = result.clone();

    Ok(result)
}

pub fn ns() -> HashMap<String, MalType> {
    let mut ns = HashMap::new();

    ns.insert("=".to_string(), func_from_bootstrap(equal));
    ns.insert("throw".to_string(), func_from_bootstrap(throw));

    ns.insert("nil?".to_string(), func_from_bootstrap(is_nil));
    ns.insert("true?".to_string(), func_from_bootstrap(is_true));
    ns.insert("false?".to_string(), func_from_bootstrap(is_false));
    ns.insert("string?".to_string(), func_from_bootstrap(is_string));
    ns.insert("symbol".to_string(), func_from_bootstrap(symbol));
    ns.insert("symbol?".to_string(), func_from_bootstrap(is_symbol));
    ns.insert("keyword".to_string(), func_from_bootstrap(keyword));
    ns.insert("keyword?".to_string(), func_from_bootstrap(is_keyword));

    ns.insert("pr-str".to_string(), func_from_bootstrap(pr_str));
    ns.insert("str".to_string(), func_from_bootstrap(str));
    ns.insert("prn".to_string(), func_from_bootstrap(prn));
    ns.insert("println".to_string(), func_from_bootstrap(println));
    ns.insert("read-string".to_string(), func_from_bootstrap(read_string));
    ns.insert("readline".to_string(), func_from_bootstrap(readline));
    ns.insert("slurp".to_string(), func_from_bootstrap(slurp));

    ns.insert("<".to_string(), func_from_bootstrap(less_than));
    ns.insert("<=".to_string(), func_from_bootstrap(less_than_or_equal));
    ns.insert(">".to_string(), func_from_bootstrap(greater_than));
    ns.insert(">=".to_string(), func_from_bootstrap(greater_than_equal));
    ns.insert("+".to_string(), func_from_bootstrap(add));
    ns.insert("-".to_string(), func_from_bootstrap(sub));
    ns.insert("*".to_string(), func_from_bootstrap(mul));
    ns.insert("/".to_string(), func_from_bootstrap(div));
    ns.insert("time-ms".to_string(), func_from_bootstrap(time_ms));

    ns.insert("list".to_string(), func_from_bootstrap(list));
    ns.insert("list?".to_string(), func_from_bootstrap(is_list));
    ns.insert("vector".to_string(), func_from_bootstrap(vector));
    ns.insert("vector?".to_string(), func_from_bootstrap(is_vector));
    ns.insert("hash-map".to_string(), func_from_bootstrap(hash_map));
    ns.insert("map?".to_string(), func_from_bootstrap(is_map));
    ns.insert("assoc".to_string(), func_from_bootstrap(assoc));
    ns.insert("dissoc".to_string(), func_from_bootstrap(dissoc));
    ns.insert("get".to_string(), func_from_bootstrap(get));
    ns.insert("contains?".to_string(), func_from_bootstrap(is_contains));
    ns.insert("keys".to_string(), func_from_bootstrap(keys));
    ns.insert("vals".to_string(), func_from_bootstrap(vals));

    ns.insert("sequential?".to_string(),
              func_from_bootstrap(is_sequential));
    ns.insert("cons".to_string(), func_from_bootstrap(cons));
    ns.insert("concat".to_string(), func_from_bootstrap(concat));
    ns.insert("nth".to_string(), func_from_bootstrap(nth));
    ns.insert("first".to_string(), func_from_bootstrap(first));
    ns.insert("rest".to_string(), func_from_bootstrap(rest));
    ns.insert("empty?".to_string(), func_from_bootstrap(is_empty));
    ns.insert("count".to_string(), func_from_bootstrap(count));
    ns.insert("apply".to_string(), func_from_bootstrap(apply));
    ns.insert("map".to_string(), func_from_bootstrap(map));

    ns.insert("conj".to_string(), func_from_bootstrap(conj));
    ns.insert("seq".to_string(), func_from_bootstrap(seq));

    ns.insert("meta".to_string(), func_from_bootstrap(meta));
    ns.insert("with-meta".to_string(), func_from_bootstrap(with_meta));
    ns.insert("atom".to_string(), func_from_bootstrap(atom));
    ns.insert("atom?".to_string(), func_from_bootstrap(is_atom));
    ns.insert("deref".to_string(), func_from_bootstrap(deref));
    ns.insert("reset!".to_string(), func_from_bootstrap(reset));
    ns.insert("swap!".to_string(), func_from_bootstrap(swap));

    ns
}
