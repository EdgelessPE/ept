use std::path::Path;

use anyhow::{anyhow, Result};

use crate::{
    executor::workflow_executor,
    log, log_ok_last, p2s,
    parsers::{parse_package, parse_workflow},
    utils::fs::try_recycle,
};

// 给定一个工作目录，对该目录执行展开
fn expand_workshop(workshop_path: &String) -> Result<()> {
    log!("Info:Expanding nep package...");
    let base = Path::new(workshop_path);
    // 检查展开工作流是否存在
    let expand_workflow_path = base.join("workflows/expand.toml");
    if !expand_workflow_path.exists() {
        return Err(anyhow!(
            "Error:Invalid expandable nep package : can't find expand workflow"
        ));
    }

    // 读取包
    let package_struct = parse_package(&p2s!(base.join("package.toml")), workshop_path, false)?;

    // 执行展开工作流
    let expand_workflow = parse_workflow(&p2s!(expand_workflow_path))?;
    workflow_executor(
        expand_workflow,
        p2s!(base.join(&package_struct.package.name)),
        package_struct,
    )?;

    // 删掉展开工作流
    try_recycle(expand_workflow_path)?;

    log_ok_last!("Info:Expanding nep package...");
    Ok(())
}

#[test]
fn test_expand_workshop() {
    use crate::utils::test::{_ensure_clear_test_dir, _run_static_file_server};
    use std::fs::copy;

    let (_addr, mut handler) = _run_static_file_server();

    // 准备文件
    _ensure_clear_test_dir();
    copy("examples/VSCode/VSCode/Code.exe", "test/Code.exe").unwrap();
    crate::utils::fs::copy_dir("examples/VSCodeE", "test/VSCodeE").unwrap();

    // 对工作目录进行展开
    expand_workshop(&"test/VSCodeE".to_string()).unwrap();

    // 断言存在 Code.exe
    assert!(Path::new("test/VSCodeE/VSCodeE/Code.exe").exists());

    handler.kill().unwrap();
}
