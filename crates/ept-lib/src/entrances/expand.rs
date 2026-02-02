use crate::types::context::RuntimeContext;
use crate::{
    executor::workflow_executor,
    log, log_ok_last, p2s,
    parsers::{parse_package, parse_workflow},
    types::constants::{DIR_WORKFLOWS, FILE_PACKAGE, WORKFLOW_EXPAND},
    utils::fs::try_recycle,
};
use anyhow::{anyhow, Result};
use std::path::Path;

pub fn is_workshop_expandable(workshop_path: &str) -> bool {
    let base = Path::new(workshop_path);
    let expand_workflow_path = base.join(DIR_WORKFLOWS).join(WORKFLOW_EXPAND);
    expand_workflow_path.exists()
}

// 给定一个工作目录，对该目录执行展开
pub fn expand_workshop(ctx: &RuntimeContext, workshop_path: &str) -> Result<()> {
    log!("Info:Expanding nep package...");
    let base = Path::new(workshop_path);
    // 检查展开工作流是否存在
    let expand_workflow_path = base.join(DIR_WORKFLOWS).join(WORKFLOW_EXPAND);
    if !expand_workflow_path.exists() {
        return Err(anyhow!(
            "Error:Invalid expandable nep package : can't find expand workflow"
        ));
    }

    // 读取包
    let package_struct = parse_package(ctx, &p2s!(base.join(FILE_PACKAGE)), workshop_path, false)?;

    // 执行展开工作流
    let expand_workflow = parse_workflow(&p2s!(expand_workflow_path))?;
    workflow_executor(
        ctx,
        expand_workflow,
        p2s!(base.join(&package_struct.package.name)),
        package_struct,
    ).map_err(|e|anyhow!("Error:Failed to execute expand workflow : '{e}'. If this error persists, consider changing 'preference.expandable' to 'low-priority' in config"))?;

    // 删掉展开工作流
    try_recycle(expand_workflow_path)?;

    log_ok_last!("Info:Expanding nep package...");
    Ok(())
}

#[test]
fn test_expand_workshop() {
    use crate::utils::test::_default_test_cfg;
    use crate::utils::test::{_ensure_clear_test_dir, _run_static_file_server};
    use std::fs::copy;

    let (_addr, mut handler) = _run_static_file_server();

    // 准备文件
    _ensure_clear_test_dir();
    copy("examples/VSCode/VSCode/Code.exe", "test/Code.exe").unwrap();
    crate::utils::fs::copy_dir("examples/VSCodeE", "test/VSCodeE").unwrap();

    let cfg = &_default_test_cfg();

    // 对工作目录进行展开
    expand_workshop(cfg, "test/VSCodeE").unwrap();

    // 断言文件是否存在
    assert!(Path::new("test/VSCodeE/VSCodeE/Code.exe").exists());
    assert!(!Path::new("test/VSCodeE/workflows/expand.toml").exists());

    handler.kill().unwrap();
}
