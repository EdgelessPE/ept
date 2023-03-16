use std::process::Command;

pub fn kill_with_name(name: String) -> bool {
    let cmd = format!("TASKKILL /F /IM {} /T", &name);
    let output = Command::new("cmd").args(["/C", &cmd]).output();

    output.is_ok()
}
