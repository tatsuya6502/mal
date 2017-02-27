use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use types::MalType;

#[derive(Clone)]
pub struct Env {
    env: Rc<RefCell<EnvData>>,
}

struct EnvData {
    outer: Option<Env>,
    data: HashMap<String, MalType>,
}

pub fn new_env(outer: Option<Env>) -> Env {
    let new_env = Rc::new(RefCell::new(EnvData {
        outer: outer,
        data: HashMap::new(),
    }));
    Env { env: new_env }
}

impl Env {
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
                v.map(|v| v.clone())
            }
            None => None,
        }
    }
}
