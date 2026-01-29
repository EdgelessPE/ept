use anyhow::{anyhow, Result};
use shell_words::split;

pub fn split_command(cmd: &str) -> Result<Vec<String>> {
    split(cmd).map_err(|e| anyhow!("Error:Failed to split command as valid posix command : {e}"))
}

#[test]
fn test_split_command() {
    let args = split_command(
        "\"C:/Program Files/Oray/SunLogin/SunloginClient/SunloginClient.exe\" --mod=uninstall",
    )
    .unwrap();
    assert_eq!(
        args,
        vec![
            "C:/Program Files/Oray/SunLogin/SunloginClient/SunloginClient.exe".to_string(),
            "--mod=uninstall".to_string()
        ]
    );

    let args=split_command("\"C:/Program Files/Edgeless PE/乙烯丙烯 三元@聚合物/edgeless bot-ver.114514.exe\" -f -d -t \"Task A,Task b/C\" -c ").unwrap();
    assert_eq!(
        args,
        vec![
            "C:/Program Files/Edgeless PE/乙烯丙烯 三元@聚合物/edgeless bot-ver.114514.exe",
            "-f",
            "-d",
            "-t",
            "Task A,Task b/C",
            "-c",
        ]
    );
}
