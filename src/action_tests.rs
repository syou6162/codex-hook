use super::*;

#[test]
fn execute_echo_command() {
    let status = execute_command("echo test").unwrap();
    assert!(status.success());
}

#[test]
fn execute_command_exit_code_nonzero() {
    let status = execute_command("exit 1").unwrap();
    assert!(!status.success());
    assert_eq!(status.code(), Some(1));
}

#[test]
fn execute_command_exit_code_zero() {
    let status = execute_command("true").unwrap();
    assert_eq!(status.code(), Some(0));
}

#[test]
fn execute_command_nonexistent_binary() {
    let status = execute_command("nonexistent_command_xyz_12345").unwrap();
    assert!(!status.success());
}

#[test]
fn execute_command_with_pipe() {
    let status = execute_command("echo hello | cat").unwrap();
    assert!(status.success());
}
