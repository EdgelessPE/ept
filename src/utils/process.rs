use sysinfo::{ProcessExt, System, SystemExt};

pub fn kill_with_name(name: String) -> bool {
    let s = System::new_all();
    let mut res=true;
    for process in s.processes_by_exact_name(&name) {
        if !process.kill(){
            res=false;
        }
    }
    res
}

pub fn is_alive_with_name(name:String) -> bool {
    let s = System::new_all();
    let mut processes=s.processes_by_exact_name(&name);
    processes.next().is_some()
}


#[test]
fn test_kill_with_name(){
    let res=kill_with_name("哔哩哔哩.exe".to_string());
    assert!(res);
}

#[test]
fn test_is_alive_with_name(){
    let res=is_alive_with_name("Code.exe".to_string());
    assert!(res);
    
    let res=is_alive_with_name("code.exe".to_string());
    assert_eq!(res,false);
}