use std::collections::HashMap;

use types::func_from_bootstrap;
use types::MalType;
use types::MalType::*;
use types::MalResult;

use printer::{pr_str as printer_pr_str, println as printer_println};

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

fn list(args: Vec<MalType>) -> MalResult {
    Ok(MalList(args))
}

fn is_list(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return Err("list?: 1 argument required".to_string());
    }
    let v = args.get(0).unwrap();

    let ret = match v {
        &MalList(_) => true,
        _ => false,
    };
    Ok(MalBool(ret))
}

fn is_empty(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return Err("empty?: 1 argument required".to_string());
    }
    let v = args.get(0).unwrap();

    let list = match v {
        &MalList(ref list) |
        &MalVector(ref list) => list,
        _ => return Ok(MalBool(false)),
    };
    Ok(MalBool(list.len() == 0))
}

fn count(args: Vec<MalType>) -> MalResult {
    if args.len() != 1 {
        return Err("count: 1 argument required".to_string());
    }
    let v = args.get(0).unwrap();

    let list = match v {
        &MalList(ref list) |
        &MalVector(ref list) => list,
        _ => return Ok(MalNumber(0)),
    };
    Ok(MalNumber(list.len() as i64))
}

fn equal(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return Err("=: 2 arguments required".to_string());
    }

    let v0 = args.get(0).unwrap();
    let v1 = args.get(1).unwrap();

    match (v0, v1) {
        (&MalList(ref a), &MalVector(ref b)) |
        (&MalVector(ref a), &MalList(ref b)) => {
            if a.len() != b.len() {
                return Ok(MalBool(false));
            }
            for i in 0..a.len() {
                let a_v = &a[i];
                let b_v = &b[i];
                let ret = try!(equal(vec![a_v.clone(), b_v.clone()]));
                match ret {
                    MalBool(true) => continue,
                    MalBool(false) => return Ok(MalBool(false)),
                    v => return Err(format!("unexpected symbol. expecte: bool, actual: {:?}", v)),
                }
            }
            Ok(MalBool(true))
        }
        (v0, v1) => Ok(MalBool(v0 == v1)),
    }
}

fn less_than(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return Err("<: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalBool(a < b)),
        _ => {
            Err(format!("unexpected symbol. expected: number & number, actual: {:?}",
                        args))
        }
    }
}

fn less_than_or_equal(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return Err("<=: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalBool(a <= b)),
        _ => {
            Err(format!("unexpected symbol. expected: number & number, actual: {:?}",
                        args))
        }
    }
}

fn greater_than(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return Err(">: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalBool(a > b)),
        _ => {
            Err(format!("unexpected symbol. expected: number & number, actual: {:?}",
                        args))
        }
    }
}

fn greater_than_equal(args: Vec<MalType>) -> MalResult {
    if args.len() != 2 {
        return Err(">=: 2 arguments required".to_string());
    }
    match (&args[0], &args[1]) {
        (&MalNumber(a), &MalNumber(b)) => Ok(MalBool(a >= b)),
        _ => {
            Err(format!("unexpected symbol. expected: number & number, actual: {:?}",
                        args))
        }
    }
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

pub fn ns() -> HashMap<String, MalType> {
    let mut ns = HashMap::new();

    ns.insert("pr-str".to_string(), func_from_bootstrap(pr_str));
    ns.insert("str".to_string(), func_from_bootstrap(str));
    ns.insert("prn".to_string(), func_from_bootstrap(prn));
    ns.insert("println".to_string(), func_from_bootstrap(println));
    ns.insert("list".to_string(), func_from_bootstrap(list));
    ns.insert("list?".to_string(), func_from_bootstrap(is_list));
    ns.insert("empty?".to_string(), func_from_bootstrap(is_empty));
    ns.insert("count".to_string(), func_from_bootstrap(count));

    ns.insert("=".to_string(), func_from_bootstrap(equal));
    ns.insert("<".to_string(), func_from_bootstrap(less_than));
    ns.insert("<=".to_string(), func_from_bootstrap(less_than_or_equal));
    ns.insert(">".to_string(), func_from_bootstrap(greater_than));
    ns.insert(">=".to_string(), func_from_bootstrap(greater_than_equal));

    ns.insert("+".to_string(), func_from_bootstrap(add));
    ns.insert("-".to_string(), func_from_bootstrap(sub));
    ns.insert("*".to_string(), func_from_bootstrap(mul));
    ns.insert("/".to_string(), func_from_bootstrap(div));

    ns
}
