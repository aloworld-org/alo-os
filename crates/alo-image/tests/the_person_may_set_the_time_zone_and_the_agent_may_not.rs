//! **The person may set the time zone and the agent may not** — as far as a
//! test without a booted machine can show it.
//!
//! Settings changes the zone through systemd's own time-zone service, and the
//! service asks polkit; the image's rule
//! (`image/usr/share/polkit-1/rules.d/50-alo-time-zone.rules`, ADR 0138) is
//! polkit's answer. **What this file proves** is the rule's text against the
//! image's own accounts: that the account it refuses is the account the agent
//! runs as, that the refusal comes before anything is allowed, that it is
//! about the time zone and nothing else, and that the recipe ships it.
//!
//! **What it cannot prove** is polkit evaluating it: no JavaScript engine runs
//! here, and no system bus. That half belongs to the next walk on a booted
//! machine — `docs/walks/checks-for-the-next-walk.md` lists it — and this file
//! says so rather than standing in for it.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on a file that is not there is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// A file of the repository, read whole.
fn read(at: &str) -> String {
    std::fs::read_to_string(the_repository().join(at))
        .unwrap_or_else(|why| panic!("{at} could not be read: {why}"))
}

/// The rule, without its comments.
fn the_rule() -> String {
    read("image/usr/share/polkit-1/rules.d/50-alo-time-zone.rules")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The account a unit names as the agent's, from the image's own accounts.
fn the_agents_account() -> String {
    read("image/usr/lib/sysusers.d/alo.conf")
        .lines()
        .find_map(|line| {
            let mut words = line.split_whitespace();
            (words.next() == Some("u"))
                .then(|| words.next())
                .flatten()
                .filter(|name| name.contains("agent"))
                .map(str::to_owned)
        })
        .expect("the image declares the agent's account")
}

/// **The account refused is the agent's, and it is refused before anything
/// is allowed** — so no order of rules can let the agent through on the
/// person's terms.
#[test]
fn the_agents_account_is_refused_before_anything_is_allowed() {
    let rule = the_rule();
    let agent = the_agents_account();
    let refused = rule
        .find(&format!("subject.user === \"{agent}\""))
        .unwrap_or_else(|| panic!("the rule does not name the agent's account, {agent}"));
    let no = rule[refused..]
        .find("polkit.Result.NO")
        .map(|at| at + refused)
        .expect("the agent's account is named and not refused");
    let yes = rule
        .find("polkit.Result.YES")
        .expect("the rule allows nobody");
    assert!(no < yes, "something is allowed before the agent is refused");
}

/// **Only the time zone, and only a person at the machine**: the rule is
/// about one action, and allows only a local, active session.
#[test]
fn the_rule_is_about_the_time_zone_and_a_person_at_the_machine() {
    let rule = the_rule();
    assert!(rule.contains("action.id !== \"org.freedesktop.timedate1.set-timezone\""));
    assert_eq!(rule.matches("polkit.Result.YES").count(), 1);
    assert!(rule.contains("subject.local && subject.active"));
    assert!(
        !rule.contains("set-time\""),
        "the rule reaches setting the clock itself"
    );
}

/// **The recipe ships the rule**, where polkit reads a distribution's.
#[test]
fn the_recipe_ships_the_rule() {
    let recipe = read("image/Containerfile");
    assert!(
        recipe.contains("COPY image/usr/share/polkit-1/rules.d/ /usr/share/polkit-1/rules.d/"),
        "the rule is written and never reaches the machine"
    );
}
