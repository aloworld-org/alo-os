use super::IsACommandRunning;

/// **The shell in front is a prompt, and nothing else is.**
#[test]
fn only_the_shell_in_front_is_idle() {
    assert_eq!(IsACommandRunning::of(Some(400), 400), IsACommandRunning::No);
    assert_eq!(
        IsACommandRunning::of(Some(401), 400),
        IsACommandRunning::Yes
    );
    assert_eq!(
        IsACommandRunning::of(None, 400),
        IsACommandRunning::NobodyCanTell
    );
}

/// **Closing asks unless the shell is known to be at a prompt**: a terminal
/// that would not say is asked about, never ended blind.
#[test]
fn closing_asks_unless_it_is_known_to_be_idle() {
    assert!(!IsACommandRunning::No.closing_asks_first());
    assert!(IsACommandRunning::Yes.closing_asks_first());
    assert!(IsACommandRunning::NobodyCanTell.closing_asks_first());
}
