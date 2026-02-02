use colored::Colorize;
use dialoguer::Confirm;
use ept_lib::types::interaction::InteractionProvider;

/// 终端交互实现
#[derive(Debug)]
pub struct TerminalInteraction;

impl TerminalInteraction {
    fn get_question_head(default_value: bool) -> colored::ColoredString {
        if default_value {
            "Question".truecolor(103, 58, 183)
        } else {
            "Question".truecolor(255, 87, 34)
        }
    }

    fn fmt_log(head: colored::ColoredString, prompt: &str) -> String {
        format!("[{}] {}", head, prompt)
    }
}

impl InteractionProvider for TerminalInteraction {
    fn ask_yn(&self, prompt: &str, default_value: bool) -> bool {
        let formatted_prompt = Self::fmt_log(Self::get_question_head(default_value), prompt);
        Confirm::new()
            .with_prompt(&formatted_prompt)
            .default(default_value)
            .interact()
            .unwrap_or(default_value)
    }

    fn ask_yn_in_step(&self, step_name: &str, prompt: &str, default_value: bool) -> bool {
        let styled_prompt = format!("[{}] {}", step_name, prompt);
        self.ask_yn(&styled_prompt, default_value)
    }

    fn show_message(&self, message: &str) {
        println!("{}", message);
    }
}
