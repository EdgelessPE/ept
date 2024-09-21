use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Flag {
    Cache,
    Confirm,
    Debug,
    NoWarning,
    Offline,
    QA,
}

lazy_static! {
    static ref FLAGS: Mutex<HashMap<Flag, bool>> = Mutex::new(HashMap::new());
}

pub fn set_flag(key: Flag, value: bool) {
    let mut map = FLAGS.lock().unwrap();
    map.insert(key, value);
}

pub fn get_flag(key: Flag, default_value: bool) -> bool {
    let map = FLAGS.lock().unwrap();
    *map.get(&key).unwrap_or(&default_value)
}
