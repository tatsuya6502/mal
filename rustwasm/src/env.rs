use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use types::MalType;
use types::MalType::*;

#[derive(PartialEq,Eq, Clone)]
pub struct Env {
    env: Rc<RefCell<EnvData>>,
}

#[derive(PartialEq,Eq, Clone)]
struct EnvData {
    outer: Option<Env>,
    data: HashMap<String, MalType>,
}

impl Env {
    pub fn new(outer: Option<Env>, binds: Vec<String>, exprs: Vec<MalType>) -> Result<Env, String> {
        let new_env = Rc::new(RefCell::new(EnvData {
            outer: outer,
            data: HashMap::new(),
        }));
        let new_env = Env { env: new_env };

        for i in 0..binds.len() {
            let bind = &binds[i];
            if bind == "&" {
                let next_bind = binds.get(i + 1);
                let next_bind = match next_bind {
                    Some(v) => v,
                    None => return Err("1 argument required after '&'".to_string()),
                };

                new_env.set(next_bind.to_string(),
                            MalList((&exprs[i..]).to_vec(), Box::new(None)));
                break;
            }
            let expr = &exprs[i];
            new_env.set(bind.to_string(), expr.clone());
        }

        Ok(new_env)
    }

    pub fn set(&self, key: String, value: MalType) -> MalType {
        self.env.borrow_mut().data.insert(key, value.clone());
        value
    }

    pub fn find(&self, key: String) -> Option<Env> {
        let env_data = self.env.borrow();
        if env_data.data.contains_key(key.as_str()) {
            return Some(self.clone());
        }

        let outer = match env_data.outer {
            Some(ref env) => env,
            None => return None,
        };
        outer.find(key)
    }

    pub fn get(&self, key: String) -> Option<MalType> {
        match self.find(key.clone()) {
            Some(env) => {
                let env_data = env.env.borrow();
                let v = env_data.data.get(key.as_str());
                v.cloned()
            }
            None => None,
        }
    }
}
