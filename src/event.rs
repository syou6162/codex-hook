use clap::ValueEnum;
use std::fmt;

#[derive(Clone, ValueEnum)]
#[value(rename_all = "PascalCase")]
pub(crate) enum HookEventType {
    PreToolUse,
    PostToolUse,
    SessionStart,
    UserPromptSubmit,
    Stop,
    SubagentStop,
    Notification,
    PreCompact,
}

impl fmt::Display for HookEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .to_possible_value()
            .expect("all variants have a possible value")
            .get_name()
            .to_owned();
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_all_event_types() {
        let cases = [
            (HookEventType::PreToolUse, "PreToolUse"),
            (HookEventType::PostToolUse, "PostToolUse"),
            (HookEventType::SessionStart, "SessionStart"),
            (HookEventType::UserPromptSubmit, "UserPromptSubmit"),
            (HookEventType::Stop, "Stop"),
            (HookEventType::SubagentStop, "SubagentStop"),
            (HookEventType::Notification, "Notification"),
            (HookEventType::PreCompact, "PreCompact"),
        ];
        for (event, expected) in cases {
            assert_eq!(event.to_string(), expected);
        }
    }
}
