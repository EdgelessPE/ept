use crate::{p2s, parsers::parse_workflow, types::mixed_fs::MixedFS};
use anyhow::Result;
use std::path::PathBuf;

pub fn get_expanded_mixed_fs(mut mixed_fs: MixedFS, workflows_path: PathBuf) -> Result<MixedFS> {
    debug_assert!(workflows_path.exists());
    let expand_workflow_path = workflows_path.join("expand.toml");
    log!("Debug:Parse expand workflow at {expand_workflow_path:?}");
    if expand_workflow_path.exists() {
        log!("Debug:Expanding mixed_fs with expand.toml at {expand_workflow_path:?}");
        let expand_workflow = parse_workflow(&p2s!(expand_workflow_path))?;
        for step in expand_workflow {
            step.body.get_manifest(&mut mixed_fs);
        }
    }

    Ok(mixed_fs)
}
