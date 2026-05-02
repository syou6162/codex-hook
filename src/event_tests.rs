use super::*;

#[test]
fn display_all_event_types() {
    let cases = [
        (HookEventType::PreToolUse, "PreToolUse"),
        (HookEventType::PostToolUse, "PostToolUse"),
        (HookEventType::PermissionRequest, "PermissionRequest"),
        (HookEventType::Notification, "Notification"),
        (HookEventType::Stop, "Stop"),
        (HookEventType::SubagentStop, "SubagentStop"),
        (HookEventType::SubagentStart, "SubagentStart"),
        (HookEventType::PreCompact, "PreCompact"),
        (HookEventType::SessionStart, "SessionStart"),
        (HookEventType::SessionEnd, "SessionEnd"),
        (HookEventType::UserPromptSubmit, "UserPromptSubmit"),
    ];
    for (event, expected) in cases {
        assert_eq!(event.to_string(), expected);
    }
}
