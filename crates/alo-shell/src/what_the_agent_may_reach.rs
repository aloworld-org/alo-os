//! The folders this person has granted, read from their own list, to be sent
//! with what they asked.
//!
//! **Measured on a booted machine on 2026-10-24.** A person granted
//! `/home/alo/Documents` in Settings and asked *what is in my Documents
//! folder*. The turn spent all eight of its rounds asking for
//! `/home/user/Documents`, `/home/user/Videos`, `/home/user/Downloads` — a
//! conventional home it had guessed — and every one was correctly refused. The
//! grant was real and the daemon enforced it exactly. **The model was never
//! told what it had.**
//!
//! # Why the person's side is the one that says so
//!
//! Three things could tell it, and only this one crosses nothing:
//!
//! - **`alo-agent` cannot read it.** It runs as the agent's own user (ADR
//!   0109), and the grants are the person's file in the person's directory.
//!   That separation is the whole reason `SO_PEERCRED` can tell the two doors
//!   apart, and it is not worth weakening for a hint.
//! - **The daemon knows, and `alo-turn` holds a deliberate rule that it must
//!   not say**: *a question asks the grants nothing — an agent putting a
//!   question to a model is what an agent is, not a capability it was given*.
//! - **This process is the person.** It runs as them, it reads their grants
//!   already to draw them in Settings, and it is where the grant was made. The
//!   side that granted the folder is the side that says so.
//!
//! # It is a hint, and that is why it may be sent at all
//!
//! Nothing here permits anything. Every verb is still decided by `alo-agentd`
//! against the grants **it** reads, and a request naming a folder nobody
//! granted sends the model to a path the daemon refuses — which is the
//! behaviour this exists to stop rather than a way round it.
//!
//! That is what makes it different from *who is asking*, which
//! `alo-agent`'s own header is emphatic about: that one the kernel answers,
//! because a forged person would run a turn against somebody else's grants.
//! A forged folder costs a wasted round and nothing else.
//!
//! # Nothing granted is still worth sending
//!
//! A machine with no grants sends an empty list, and `alo_instructing`
//! turns that into a sentence telling the model to say so rather than guess.
//! That is the case the eight rounds above were.

use std::path::PathBuf;
use std::time::SystemTime;

use alo_capability::Grantee;

use crate::SettingsPlaces;

/// The folders this person has granted to this machine's agent, right now.
///
/// Empty on a machine that has granted nothing, one whose grants will not
/// read, or one whose session never said what its agent is called — all three
/// being *we cannot name a folder it may reach*, which is what the model is
/// then told.
#[must_use]
pub fn what_the_agent_may_reach(places: &SettingsPlaces, now: SystemTime) -> Vec<PathBuf> {
    let Some(agent) = places.agent() else {
        return Vec::new();
    };
    let Ok(grants) = alo_remembering::remembered(places.grants(), now) else {
        return Vec::new();
    };
    let agent = Grantee::named(agent);
    grants
        .held_by(&agent, now)
        .filter_map(|held| held.grant.reach.as_path().map(std::path::Path::to_path_buf))
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_capability::{Grant, Grants, Reach};
    use std::ffi::OsStr;
    use std::time::Duration;

    /// A moment every test here shares.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// A folder for one test, taken away when it ends.
    struct Somewhere(PathBuf);
    impl Somewhere {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let at = std::env::temp_dir().join(format!(
                "alo-may-reach-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&at).unwrap();
            Self(at)
        }
    }
    impl Drop for Somewhere {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Places pointing at a grants file of this test's own.
    fn places(at: &Somewhere, agent: Option<&str>) -> SettingsPlaces {
        SettingsPlaces::of(
            None,
            Some(OsStr::new("/home/alo")),
            None,
            &at.0.join("grants"),
            &at.0.join("pairings"),
            &at.0.join("agentd.sock"),
            &at.0.join("how-code-runs.toml"),
            agent,
        )
    }

    /// Write these grants where the places say they are.
    fn kept(at: &Somewhere, grants: &Grants) {
        alo_remembering::kept(&at.0.join("grants"), grants, noon()).unwrap();
    }

    /// **The folder a person granted is the folder that is sent.**
    #[test]
    fn a_granted_folder_is_named() {
        let at = Somewhere::new();
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                "alo",
                Reach::Folder(PathBuf::from("/home/alo/Documents")),
                noon(),
                Duration::from_secs(3600),
            )
            .unwrap(),
        );
        kept(&at, &grants);

        let folders = what_the_agent_may_reach(&places(&at, Some("alo")), noon());
        assert_eq!(folders, [PathBuf::from("/home/alo/Documents")]);
    }

    /// **Another agent's grant is not this one's.** The test that would catch a
    /// folder being named to an agent nobody granted it to.
    #[test]
    fn a_grant_to_another_agent_is_not_sent() {
        let at = Somewhere::new();
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                "somebody-else",
                Reach::Folder(PathBuf::from("/home/alo/Private")),
                noon(),
                Duration::from_secs(3600),
            )
            .unwrap(),
        );
        kept(&at, &grants);

        assert!(what_the_agent_may_reach(&places(&at, Some("alo")), noon()).is_empty());
    }

    /// **An expired grant is not sent**, because it is not held.
    #[test]
    fn a_grant_that_has_run_out_is_not_sent() {
        let at = Somewhere::new();
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                "alo",
                Reach::Folder(PathBuf::from("/home/alo/Documents")),
                noon(),
                Duration::from_secs(60),
            )
            .unwrap(),
        );
        kept(&at, &grants);

        let later = noon() + Duration::from_secs(3600);
        assert!(what_the_agent_may_reach(&places(&at, Some("alo")), later).is_empty());
    }

    /// A session that never said what the agent is called names no folder,
    /// rather than guessing a name and sending somebody else's grants.
    #[test]
    fn a_machine_that_did_not_say_its_agents_name_sends_nothing() {
        let at = Somewhere::new();
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                "alo",
                Reach::Folder(PathBuf::from("/home/alo/Documents")),
                noon(),
                Duration::from_secs(3600),
            )
            .unwrap(),
        );
        kept(&at, &grants);

        assert!(what_the_agent_may_reach(&places(&at, None), noon()).is_empty());
    }

    /// A grants file that is not there is a machine that has granted nothing,
    /// not a reason to stop.
    #[test]
    fn a_machine_with_no_grants_file_sends_nothing() {
        let at = Somewhere::new();
        assert!(what_the_agent_may_reach(&places(&at, Some("alo")), noon()).is_empty());
    }
}
