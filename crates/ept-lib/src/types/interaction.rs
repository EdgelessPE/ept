use std::fmt::Debug;
use std::sync::Arc;

/// 交互提供者 trait，用于解耦 ept-lib 与具体的交互实现（CLI、GUI 等）
pub trait InteractionProvider: Send + Sync + Debug {
    /// 询问用户是/否问题
    ///
    /// # 参数
    /// - `prompt`: 提示信息
    /// - `default_value`: 默认返回值（当用户未响应或交互不可用时）
    ///
    /// # 返回值
    /// 用户的回答，true 表示是，false 表示否
    fn ask_yn(&self, prompt: &str, default_value: bool) -> bool;

    /// 在工作流步骤中询问用户是/否问题
    ///
    /// # 参数
    /// - `step_name`: 步骤名称
    /// - `prompt`: 提示信息
    /// - `default_value`: 默认返回值
    ///
    /// # 返回值
    /// 用户的回答
    fn ask_yn_in_step(&self, step_name: &str, prompt: &str, default_value: bool) -> bool;

    /// 显示信息消息（可选实现）
    fn show_message(&self, message: &str) {
        // 默认空实现
        let _ = message;
    }
}

/// 空交互实现，用于不需要交互的场景（如自动化脚本）
#[derive(Debug)]
pub struct NoInteraction;

impl InteractionProvider for NoInteraction {
    #[cfg(test)]
    fn ask_yn(&self, _prompt: &str, _default_value: bool) -> bool {
        true
    }

    #[cfg(not(test))]
    fn ask_yn(&self, _prompt: &str, default_value: bool) -> bool {
        default_value
    }

    #[cfg(test)]
    fn ask_yn_in_step(&self, _step_name: &str, _prompt: &str, _default_value: bool) -> bool {
        true
    }

    #[cfg(not(test))]
    fn ask_yn_in_step(&self, _step_name: &str, _prompt: &str, default_value: bool) -> bool {
        default_value
    }
}

/// 交互提供者包装类型，用于在 Cfg 中存储
pub type InteractionProviderArc = Arc<dyn InteractionProvider>;

#[test]
fn test_no_interaction() {
    let no_interaction = NoInteraction;

    // NoInteraction 会在测试模式下总是返回 true
    assert!(no_interaction.ask_yn("Test prompt?", true));
    assert!(no_interaction.ask_yn("Test prompt?", false));
    assert!(no_interaction.ask_yn_in_step("Step", "Test prompt?", true));
    assert!(no_interaction.ask_yn_in_step("Step", "Test prompt?", false));
}
