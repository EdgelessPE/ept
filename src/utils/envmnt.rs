use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref GLOBAL_ENV: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub fn set(key: &str, value: &str) {
    let mut map = GLOBAL_ENV.lock().unwrap();
    map.insert(key.to_string(), value.to_string());
}

pub fn get_or(key: &str, default_value: &str) -> String {
    let map = GLOBAL_ENV.lock().unwrap();
    map.get(key).unwrap_or(&default_value.to_string()).clone()
}

pub fn get_u64(key: &str, default_value: u64) -> u64 {
    let map = GLOBAL_ENV.lock().unwrap();
    if let Some(value) = map.get(key) {
        value.parse::<u64>().unwrap()
    } else {
        default_value
    }
}
