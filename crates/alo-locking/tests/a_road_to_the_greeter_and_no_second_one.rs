//! A locked screen has one road to the greeter, it carries no session, and no
//! agent can reach it.
//!
//! ADR 0061 widened what a lock screen may offer, once, and the whole of its
//! argument rests on what the road cannot carry. The unit tests in
//! `src/somebody_else.rs` show the road taken and refused; what they cannot show
//! is that there is no **second** one — a call that hands the greeter this
//! session's name, a field somebody adds to the answer later, a crate an agent
//! is answered in that can ask for any of it. Those are properties of the
//! crate's whole shipped source and of the workspace's manifests, so this reads
//! them.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;

/// The fixture file, which is `cfg(test)` and never compiled into the crate.
const THE_FIXTURE: &str = "testing.rs";

/// The file this decision is held in.
const THE_ROAD: &str = "somebody_else.rs";

/// The workspace's `crates` directory.
fn the_crates() -> PathBuf {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(crates) = here.parent() else {
        panic!("this crate is not inside a crates directory");
    };
    crates.to_path_buf()
}

/// A file's **shipped** code: what is compiled into the crate, with neither
/// its documentation nor its own `cfg(test)` module.
///
/// The test module matters here in a way it does not in this crate's other
/// readers. A test for the road has to build a locked seat, hand it a
/// notification and unlock it again, so the very names this file must refuse in
/// shipped code are names its tests use on purpose. Reading past `#[cfg(test)]`
/// would make the check either useless or impossible to satisfy.
fn shipped_code_of(text: &str) -> String {
    text.lines()
        .take_while(|line| !line.trim_start().starts_with("#[cfg(test)]"))
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every Rust file under this crate's `src`, as its name and its shipped code.
fn the_shipped_code() -> Vec<(String, String)> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let Ok(entries) = fs::read_dir(&src) else {
        panic!("this crate's source could not be read");
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            panic!("{} could not be read", path.display());
        };
        let named = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        if named == THE_FIXTURE {
            continue;
        }
        found.push((named, shipped_code_of(&text)));
    }
    assert!(
        found.iter().any(|(named, _)| named == THE_ROAD),
        "{THE_ROAD} is not where this test looks for it"
    );
    found
}

/// **There is one road from a locked screen to the greeter.** `Standing` is
/// what a greeter draws from, and the only shipped code in this crate that
/// names it is the one file ADR 0061 is held in — so a second, quieter road
/// cannot be added without this failing.
#[test]
fn the_road_to_the_greeter_is_one_file_and_one_call() {
    let code = the_shipped_code();
    let naming: Vec<&String> = code
        .iter()
        .filter(|(_, text)| text.contains("Standing"))
        .map(|(named, _)| named)
        .collect();
    assert_eq!(
        naming,
        vec![&THE_ROAD.to_owned()],
        "something other than the road names what a greeter stands at"
    );

    let Some((_, road)) = code.iter().find(|(named, _)| named == THE_ROAD) else {
        panic!("{THE_ROAD} is not where this test looks for it");
    };
    assert_eq!(
        road.matches("Standing::of(").count(),
        1,
        "what the greeter stands at is worked out somewhere other than once, from the accounts"
    );
    assert!(
        road.contains("pub fn somebody_else(&self, accounts: &Accounts)"),
        "the road does not take the seat by reference and the accounts as they are"
    );
}

/// **The road carries no session, and has no room for one.** Not the person,
/// not their number, not a notification it is holding, not what was open — and
/// no type parameter a caller could put any of them in.
#[test]
fn the_road_carries_nothing_of_the_locked_session() {
    let code = the_shipped_code();
    let Some((_, road)) = code.iter().find(|(named, _)| named == THE_ROAD) else {
        panic!("{THE_ROAD} is not where this test looks for it");
    };

    assert!(
        road.contains("pub enum SomebodyElse {"),
        "the answer has a type parameter, which is room for whatever a seat holds"
    );
    for carried in [
        "Session",
        "session()",
        "into_parts",
        "held",
        "uid",
        "Locked<",
        "alo_leaving",
        "leaving.toml",
        "Knock",
        "signs_in",
        "password",
    ] {
        assert!(
            !road.contains(carried),
            "{THE_ROAD} names `{carried}`: the road to the greeter carries nothing of the \
             session behind the lock"
        );
    }
}

/// **The check above is looking at real code.** A reader that found nothing
/// would pass on anything, so it is shown the shape it exists to refuse.
#[test]
fn the_reader_refuses_a_road_that_carries_the_session() {
    let would_be = shipped_code_of(
        "//! Session, in prose, is fine.\npub enum SomebodyElse<N> {\n    \
         TheGreeter(Standing, Session),\n}\n#[cfg(test)]\nmod tests {\n    use \
         super::Session;\n}\n",
    );
    assert!(!would_be.contains("pub enum SomebodyElse {"));
    assert!(would_be.contains("Session"));
    assert!(
        !would_be.contains("use super::Session;"),
        "the reader read past the test module"
    );
    assert!(
        !shipped_code_of("//! Session, only in prose\n").contains("Session"),
        "the reader read documentation as code"
    );
}

/// The crates an agent's request is carried out in, which must not be able to
/// reach this one — named so that a rename of any of them fails here rather
/// than passes quietly.
const WHERE_AN_AGENT_IS_ANSWERED: [&str; 5] = [
    "alo-agentd",
    "alo-turn",
    "alo-capability",
    "alo-protocol",
    "alo-broker",
];

/// **Switching to another person is not a thing an agent can ask for.** ADR
/// 0061's constraint, and the reason is not that nobody wrote the verb: a crate
/// an agent is answered in that could reach a seat could lock one, read what it
/// is holding, or hand somebody else's screen to a greeter. So none of them
/// depends on this crate, and this reads every manifest in the workspace for it.
#[test]
fn no_crate_an_agent_is_answered_in_can_reach_a_seat() {
    let Ok(entries) = fs::read_dir(the_crates()) else {
        panic!("the workspace's crates could not be read");
    };
    let mut read = 0_usize;
    let mut checked = Vec::new();
    for entry in entries.flatten() {
        let named = entry.file_name().to_string_lossy().into_owned();
        let Ok(manifest) = fs::read_to_string(entry.path().join("Cargo.toml")) else {
            continue;
        };
        read = read.saturating_add(1);
        if !WHERE_AN_AGENT_IS_ANSWERED.contains(&named.as_str()) {
            continue;
        }
        checked.push(named.clone());
        let depends = manifest
            .lines()
            .map(str::trim_start)
            .filter(|line| !line.starts_with('#'))
            .any(|line| line.starts_with("alo-locking"));
        assert!(
            !depends,
            "{named} depends on alo-locking: an agent's request must not reach a seat"
        );
    }
    assert!(read > 50, "the workspace's crates were not read: {read}");
    assert_eq!(
        checked.len(),
        WHERE_AN_AGENT_IS_ANSWERED.len(),
        "a crate an agent is answered in was renamed or moved: {checked:?}"
    );
}
