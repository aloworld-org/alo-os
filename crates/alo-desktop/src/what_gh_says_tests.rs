//! **gh's own lines are read as what they say, and nothing else is.**

use super::*;

/// **The code, the page and the login are read from gh's lines** — with and
/// without colour.
#[test]
fn the_code_the_page_and_the_login_are_read() {
    assert_eq!(
        the_code_on("! First copy your one-time code: 4F2A-9C1D").as_deref(),
        Some("4F2A-9C1D")
    );
    assert_eq!(
        the_code_on(
            "\u{1b}[0;33m!\u{1b}[0m First copy your one-time code: \u{1b}[1m4F2A-9C1D\u{1b}[0m"
        )
        .as_deref(),
        Some("4F2A-9C1D")
    );
    assert_eq!(
        the_page_on(
            "Open this URL to continue in your web browser: https://github.com/login/device"
        )
        .as_deref(),
        Some("https://github.com/login/device")
    );
    assert_eq!(the_login_on("✓ Logged in as anna").as_deref(), Some("anna"));
    assert_eq!(
        the_login_on("✓ Logged in as anna-k2 to github.com").as_deref(),
        Some("anna-k2")
    );
}

/// **The refusal path: nothing that is not shaped like what is looked for is
/// read as it** — so gh's output cannot put words of its own anywhere.
#[test]
fn nothing_misshapen_is_read() {
    for line in [
        "! First copy your one-time code: hello",
        "! First copy your one-time code: 4F2A9C1D",
        "! First copy your one-time code: 4f2a-9c1d",
        "! First copy your one-time code:",
        "Press Enter to open github.com in your browser...",
        "",
    ] {
        assert_eq!(the_code_on(line), None, "{line:?}");
    }
    assert_eq!(
        the_page_on("Open this URL: https://github.com.example.org/login/device"),
        None
    );
    assert_eq!(
        the_page_on("Open this URL: http://github.com/login/device"),
        None
    );
    let long = "a".repeat(101);
    for login in ["", "-anna", "anna smith", "anna;rm", long.as_str()] {
        assert_eq!(a_login(login), None, "{login:?}");
    }
    assert_eq!(the_login_on("✓ Authentication complete."), None);
}

/// **A failure is said with gh's last line that has words on it.**
#[test]
fn the_last_line_with_words_is_said() {
    let lines = [
        "error connecting to api.github.com".to_owned(),
        "check your internet connection or https://githubstatus.com".to_owned(),
        "   ".to_owned(),
    ];
    assert_eq!(
        the_last_line(&lines).as_deref(),
        Some("check your internet connection or https://githubstatus.com")
    );
    assert_eq!(the_last_line(&[]), None);
    assert_eq!(the_last_line(&[" ".to_owned()]), None);
}
