use std::fmt;

use printer::pr_str;

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
    MalFunc(MalF),
}

impl fmt::Debug for MalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", pr_str(self, true))
    }
}

pub fn func_from_bootstrap(f: MalF) -> MalType {
    MalFunc(f)
}
