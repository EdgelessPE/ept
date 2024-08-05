use sysinfo::System;

pub fn kill_with_name(name: &str) -> bool {
    let s = System::new_all();
    let mut res = true;
    for process in s.processes_by_exact_name(name.as_ref()) {
        if !process.kill() {
            res = false;
        }
    }
    res
}

pub fn is_alive_with_name(name: &str) -> bool {
    let s = System::new_all();
    let mut processes = s.processes_by_exact_name(name.as_ref());
    processes.next().is_some()
}

#[test]
fn test_kill_with_name() {
    // let res = kill_with_name(&"哔哩哔哩.exe".to_string());
    // assert!(res);
}

#[test]
fn test_is_alive_with_name() {
    let res = is_alive_with_name("cargo.exe");
    assert!(res);

    let res = is_alive_with_name("Cargo.exe");
    assert!(!res);

    let res = is_alive_with_name("cargo");
    assert!(!res);
}
