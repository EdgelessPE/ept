use anyhow::{anyhow, Result};
use sysinfo::System;

pub fn kill_with_name(name: &str) -> Result<bool> {
    let s = System::new_all()
        .map_err(|e| anyhow!("Error(Process):Failed to initialize system information : {e}"))?;
    let mut res = true;
    for process in s.processes_by_exact_name(name.as_ref()) {
        if !process.kill() {
            res = false;
        }
    }
    Ok(res)
}

pub fn is_alive_with_name(name: &str) -> Result<bool> {
    let s = System::new_all()
        .map_err(|e| anyhow!("Error(Process):Failed to initialize system information : {e}"))?;
    let mut processes = s.processes_by_exact_name(name.as_ref());
    Ok(processes.next().is_some())
}

#[test]
fn test_kill_with_name() {
    // let res = kill_with_name("哔哩哔哩.exe").unwrap();
    // assert!(res);
}

#[test]
fn test_is_alive_with_name() {
    let res = is_alive_with_name("cargo.exe").unwrap();
    assert!(res);

    let res = is_alive_with_name("Cargo.exe").unwrap();
    assert!(!res);

    let res = is_alive_with_name("cargo").unwrap();
    assert!(!res);
}
