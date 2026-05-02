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
