use clap::ValueEnum;
use std::fmt;

#[derive(Clone, ValueEnum)]
#[value(rename_all = "PascalCase")]
pub(crate) enum HookEventType {
    PreToolUse,
    PostToolUse,
    PermissionRequest,
    Notification,
    Stop,
    SubagentStop,
    SubagentStart,
    PreCompact,
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
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
#[path = "event_tests.rs"]
mod event_tests;
