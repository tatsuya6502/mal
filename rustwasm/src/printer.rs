use types::MalType;
use types::MalType::*;

pub fn pr_str(v: &MalType, print_readably: bool) -> String {
    match v {
        &MalList(ref list) => {
            let value =
                list.iter().map(|x| pr_str(x, print_readably)).collect::<Vec<_>>().join(" ");
            format!("({})", value)
        }
        &MalVector(ref list) => {
            let value =
                list.iter().map(|x| pr_str(x, print_readably)).collect::<Vec<_>>().join(" ");
            format!("[{}]", value)
        }
        &MalHashMap(ref list) => {
            let value =
                list.iter().map(|x| pr_str(x, print_readably)).collect::<Vec<_>>().join(" ");
            format!("{{{}}}", value)
        }
        &MalNumber(ref v) => format!("{}", v),
        &MalSymbol(ref v) => format!("{}", v),
        &MalBool(v) => format!("{}", v),
        &MalString(ref v) => format!(r#""{}""#, v),
        &MalNil => "nil".to_string(),
        &MalKeyword(ref v) => format!(":{}", v),
        _ => unimplemented!(),
    }
}
