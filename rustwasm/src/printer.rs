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
        &MalString(ref v) => {
            if print_readably {
                let v = v.replace("\\", "\\\\")
                    .replace("\"", "\\\"")
                    .replace("\n", "\\n");
                format!(r#""{}""#, v)
            } else {
                v.to_string()
            }
        }
        &MalNil => "nil".to_string(),
        &MalKeyword(ref v) => format!(":{}", v),
        &MalFunc(_) => "#<function>".to_string(),
    }
}
