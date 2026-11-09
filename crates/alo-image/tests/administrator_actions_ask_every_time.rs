//! **Administrator actions ask the person's password every time, and the
//! image holds every piece of that to every other**
//! ([ADR 0150](../../../docs/decisions/0150-administrator-actions-ask-the-persons-password-each-time.md)).
//!
//! **What this file proves** is the text of what the image ships: polkit's
//! rule refuses the agent's login and keeps alo's action at `auth_admin`, the
//! action never keeps an answer, the person is made an administrator, and the
//! three sockets are installed, enabled and opened to whom they say.
//!
//! **What it cannot prove** is polkit evaluating any of it, PAM accepting a
//! password, or `chpasswd` writing under SELinux: no polkit, PAM or SELinux
//! runs here. Those are walked on a booted machine, and ADR 0150 lists them
//! as not yet walked.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on a file that is not there is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The repository this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// A file of the repository, read.
fn read(at: &str) -> String {
    std::fs::read_to_string(the_repository().join(at))
        .unwrap_or_else(|why| panic!("{at} could not be read: {why}"))
}

/// The rule, without its comments.
fn the_rule() -> String {
    read("image/usr/share/polkit-1/rules.d/49-alo-administrator.rules")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **The agent's login is refused everything, before anything else is
/// asked**, and alo's own action always asks for an administrator.
#[test]
fn the_agent_is_refused_first_and_alos_action_always_asks() {
    let rule = the_rule();
    let refused = rule
        .find(r#"subject.user === "alo-agent""#)
        .expect("the rule names the agent's login");
    let no = rule.find("polkit.Result.NO").expect("the rule refuses");
    let action = rule
        .find(r#"action.id === "org.aloworld.alo.run-as-administrator""#)
        .expect("the rule names alo's action");
    let asks = rule
        .find("polkit.Result.AUTH_ADMIN")
        .expect("the rule asks for an administrator");
    assert!(refused < no && no < action && action < asks, "{rule}");
    assert!(
        !rule.contains("Result.YES") && !rule.contains("AUTH_ADMIN_KEEP"),
        "nothing in the rule says yes or keeps an answer:\n{rule}"
    );
    let agent = read("image/usr/lib/sysusers.d/alo.conf");
    assert!(
        agent.contains("u alo-agent 60989:60989"),
        "the rule names a login the image does not make"
    );
}

/// **The action asks every time** — `auth_admin` at all three settings and
/// never `auth_admin_keep` — and is the action the runner asks about.
#[test]
fn the_action_asks_every_time() {
    let policy = read("image/usr/share/polkit-1/actions/org.aloworld.alo.policy");
    assert!(policy.contains(r#"<action id="org.aloworld.alo.run-as-administrator">"#));
    for setting in ["allow_any", "allow_inactive", "allow_active"] {
        assert!(
            policy.contains(&format!("<{setting}>auth_admin</{setting}>")),
            "{setting} does not ask for an administrator"
        );
    }
    assert!(!policy.contains("auth_admin_keep"), "an answer is kept");
    assert!(policy.contains("<message>$(command)</message>"));
    let asked = read("crates/alo-administrator/src/what_polkit_said.rs");
    assert!(asked.contains(r#"pub const THE_ACTION: &str = "org.aloworld.alo.run-as-administrator";"#));
    assert!(asked.contains(r#"pub const THE_COMMAND_DETAIL: &str = "command";"#));
}

/// **The person is an administrator**, by the base's own rule: a member of
/// `wheel`, declared and asserted by the build.
#[test]
fn the_person_is_an_administrator() {
    let accounts = read("image/usr/lib/sysusers.d/alo.conf");
    assert!(
        accounts.lines().any(|line| line.trim() == "m alo wheel"),
        "the person is not made a member of wheel"
    );
    assert!(
        !accounts.lines().any(|line| line.trim() == "m alo-agent wheel"),
        "the agent is made an administrator"
    );
    let recipe = read("image/Containerfile");
    assert!(recipe.contains("id -nG alo | tr ' ' '\\n' | grep -qx wheel"));
}

/// **The three sockets are installed, enabled, and opened to whom they say**:
/// the keeper to root alone, the checker to anybody (it checks only the
/// caller's own login), the runner to the person's group.
#[test]
fn the_three_sockets_are_installed_enabled_and_opened_as_they_say() {
    let recipe = read("image/Containerfile");
    for (socket, mode, group) in [
        ("alo-keeping-the-system-password", "0600", "root"),
        ("alo-checking-a-password", "0666", "root"),
        ("alo-running-as-administrator", "0660", "alo"),
    ] {
        let unit = read(&format!("image/usr/lib/systemd/system/{socket}.socket"));
        assert!(unit.contains(&format!("ListenStream=/run/{socket}.socket")), "{socket}");
        assert!(unit.contains(&format!("SocketMode={mode}")), "{socket}");
        assert!(unit.contains(&format!("SocketGroup={group}")), "{socket}");
        assert!(unit.contains("Accept=yes"), "{socket}");
        let service = read(&format!("image/usr/lib/systemd/system/{socket}@.service"));
        assert!(
            service.contains(&format!("ExecStart=/usr/libexec/{socket}")),
            "{socket}"
        );
        assert!(service.contains("StandardInput=socket"), "{socket}");
        assert!(
            recipe.contains(&format!("release/{socket} /usr/libexec/{socket}")),
            "{socket} is not copied into the image"
        );
        assert!(
            recipe.contains(&format!("{socket}.socket \\")),
            "{socket}.socket is not enabled"
        );
    }
    let keeper = read("image/usr/lib/systemd/system/alo-keeping-the-system-password@.service");
    assert!(keeper.contains("CapabilityBoundingSet=CAP_DAC_OVERRIDE\n"));
    assert!(keeper.contains("NoNewPrivileges=yes"));
    let named = read("crates/alo-administrator/src/keeping.rs");
    assert!(named.contains(
        r#"pub const THE_KEEPERS_SOCKET: &str = "/run/alo-keeping-the-system-password.socket";"#
    ));
    let named = read("crates/alo-administrator/src/the_helpers_conversation.rs");
    assert!(named.contains(r#"pub const THE_CHECKERS_SOCKET: &str = "/run/alo-checking-a-password.socket";"#));
    let named = read("crates/alo-administrator/src/answering_as_administrator.rs");
    assert!(named.contains(
        r#"pub const THE_RUNNERS_SOCKET: &str = "/run/alo-running-as-administrator.socket";"#
    ));
    let folder = read("image/usr/lib/tmpfiles.d/alo.conf");
    assert!(folder.contains("d /run/alo-the-system-password 0755 root root -"));
}

/// **The agent service still cannot gain a privilege**, which is why a `sudo`
/// line goes to the runner at all: if this ever changed, `sudo` would run
/// inside the service and nothing would ask the person through the sheet.
#[test]
fn the_agent_service_still_cannot_gain_a_privilege() {
    let service = read("image/usr/lib/systemd/system/alo-agentd.service");
    assert!(service.contains("NoNewPrivileges=yes"));
    assert!(service.contains("CapabilityBoundingSet=\n"));
}
