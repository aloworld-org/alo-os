//! Signing in to a proxy that asks who you are — the one door, on every road.
//!
//! `crate::deciding` answers *which way*; `crate::carried` is that answer in
//! the shape a road needs. Between them there is one thing left to do on a
//! company network, and it is the thing that was missing until
//! [ADR 0059](../../../docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md):
//! a great many of those proxies ask for a name, and a machine that reaches one
//! as somebody with no password is refused by the proxy — on a network whose
//! own header says there is no other route out.
//!
//! [`signed_in`] is that door and there is no second one. Every road out —
//! installing an application and updating one, the system's own update, a
//! provider's list and a turn's question — builds its [`crate::Carried`] here,
//! so [`crate::Carried::with_the_password`] keeps **one caller**, and the one
//! function in this crate that turns a password into text keeps one function
//! above it. A test in this file reads the workspace and says so, rather than
//! this paragraph being a promise about other people's future changes.
//!
//! # A road that asks for nothing opens nothing
//!
//! A road going straight out, and a proxy that wants no name, never reach the
//! store: [`signed_in`] answers from the [`crate::Way`] alone. So the ordinary
//! machine on an ordinary network meets none of this, and a test hands over a
//! store that panics if anything asks it.
//!
//! # Every refusal is a road not taken
//!
//! [`NotSignedIn`] is the whole list, and **not one member of it is a road
//! going straight out instead**, or a road taken to the proxy without the
//! credential. Both of those would be a machine going around its company's own
//! rule, or announcing itself to the company's proxy as nobody in particular,
//! with the person told neither. `crate::refusing` refuses an automatic
//! configuration it cannot work out for the same reason, one decision earlier.
//!
//! What a person reads says what to do and **quotes nothing that was stored**.
//! What was wrong with what was stored is kept on the refusal for whoever
//! administers the machine, the way `crate::NotOnTheRoad::because` keeps what a
//! company's own script printed.

use alo_strings::{Filling, Said, Strings};

use crate::carried::Carried;
use crate::password::{NotAPassword, Password, WhereThePasswordIs};
use crate::road::Way;
use crate::words;

/// Where this machine's own proxy passwords are kept, as a road asks for one.
///
/// One method, taking the name the setting holds. `crate::TheMachinesPasswords`
/// is the machine's; a test's stands in for it, and there is deliberately no
/// third kind — a road that could be handed a store of its own choosing is a
/// road whose credential came from somewhere nobody decided.
pub trait WhereThePasswordsAre {
    /// The password kept under this name.
    ///
    /// # Errors
    /// [`NotSignedIn`], on every one of which the road is **not taken**.
    fn password(&self, kept: &WhereThePasswordIs) -> Result<Password, NotSignedIn>;
}

/// Why a road out was not taken, because this machine could not sign in to the
/// proxy it has to go through.
///
/// **No `Display`**, so that the only road to words is [`NotSignedIn::said`]:
/// every one of these is about a credential, and a sentence one `to_string()`
/// away from a credential is a sentence somebody will eventually log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotSignedIn {
    /// This machine was given no passwords at all — the unit taking this road
    /// carries no credentials.
    NothingKeepsIt,
    /// This machine keeps passwords and has none under the name the setting
    /// gives.
    NotKept,
    /// The name the setting gives is not one thing a password can be looked up
    /// by, so nothing was looked up.
    NotALookableName,
    /// The password is kept where somebody other than its owner could read it.
    ReadableByAnybody,
    /// It is there and could not be read.
    ///
    /// Carries the kind of failure and **nothing else** — no path, no message
    /// and nothing that was stored.
    NotRead(std::io::ErrorKind),
    /// What is kept under that name is longer than any password.
    LongerThanAPassword,
    /// What is kept under that name is not a password.
    NotAPassword(NotAPassword),
}

impl NotSignedIn {
    /// The string this crate declares for this refusal.
    ///
    /// Three sentences for seven states, because what a person **does** about
    /// them is three things: the machine has not been given the password, the
    /// password it has is readable by others, or the password it has cannot be
    /// used. Which of the seven it was is [`NotSignedIn::because`]'s, for
    /// whoever administers the machine.
    #[must_use]
    pub const fn word(self) -> words::Word {
        match self {
            Self::NothingKeepsIt | Self::NotKept | Self::NotALookableName => {
                words::NOT_SIGNED_IN_NO_PASSWORD
            }
            Self::ReadableByAnybody => words::NOT_SIGNED_IN_READABLE_BY_ANYBODY,
            Self::NotRead(_) | Self::LongerThanAPassword | Self::NotAPassword(_) => {
                words::NOT_SIGNED_IN_NOT_USABLE
            }
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    /// **The road was refused before this was called**, as `crate::refusing`
    /// and `alo-egress` both arrange their own: a machine whose translations
    /// failed to load refuses exactly what it refused before.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }

    /// Whether this machine has the password at all.
    ///
    /// The half an administrator acts on first: *give this unit the credential*
    /// is a different job from *the credential it has is wrong*.
    #[must_use]
    pub const fn because(self) -> WhatIsWrong {
        match self {
            Self::NothingKeepsIt | Self::NotKept | Self::NotALookableName => {
                WhatIsWrong::TheMachineHasNotGotIt
            }
            Self::ReadableByAnybody => WhatIsWrong::AnybodyCouldReadIt,
            Self::NotRead(_) | Self::LongerThanAPassword | Self::NotAPassword(_) => {
                WhatIsWrong::WhatItHasCannotBeUsed
            }
        }
    }

    /// **Nothing left this machine.** Stated as a method rather than as a
    /// comment, the way `alo_secrets::NotStored::nothing_was_sent` is: a road
    /// refused here was refused before anything opened.
    #[must_use]
    pub const fn nothing_was_sent(self) -> bool {
        true
    }
}

/// What an administrator does about a road that could not be signed in to.
///
/// Three, matching the three sentences, and answered without matching on seven
/// members at every call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatIsWrong {
    /// The machine was never given the password this proxy asks for.
    TheMachineHasNotGotIt,
    /// It has one, kept where others could read it.
    AnybodyCouldReadIt,
    /// It has one and cannot use it.
    WhatItHasCannotBeUsed,
}

/// This road, signed in to where the proxy asks who you are.
///
/// **The one door.** Every road out of this machine builds its
/// [`Carried`] here, which is what keeps
/// [`Carried::with_the_password`] to a single caller and the one function that
/// turns a password into text to a single function above it.
///
/// A road going straight out, and a proxy that asks for no name, never reach
/// `passwords` at all.
///
/// # Errors
/// [`NotSignedIn`], on every one of which **the road is not taken** — never
/// taken straight out instead, and never taken to the proxy without the
/// credential it asked for.
pub fn signed_in(way: Way, passwords: &dyn WhereThePasswordsAre) -> Result<Carried, NotSignedIn> {
    let Some(kept) = way.through().and_then(|proxy| proxy.password()).cloned() else {
        return Ok(Carried::of(way));
    };
    let password = passwords.password(&kept)?;
    Ok(Carried::of(way).with_the_password(password))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::address::{ProxyAddress, SpokenTo};
    use crate::testing::{Keeping, NeverKept, in_english, translated};

    /// The company's proxy, as somebody was handed it on a slip of paper.
    fn the_companys() -> ProxyAddress {
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap()
    }

    /// The same proxy, asking who you are.
    fn asking_who_you_are() -> ProxyAddress {
        the_companys()
            .signing_in(
                "anna",
                WhereThePasswordIs::named("the company proxy").unwrap(),
            )
            .unwrap()
    }

    /// **A proxy that asks who you are is signed in to**, and the credential
    /// travels on the road that needs it and appears nowhere else.
    #[test]
    fn a_proxy_that_asks_who_you_are_is_signed_in_to() {
        let keeping = Keeping::of("the company proxy", "hunter2");
        let carried = signed_in(Way::Through(asking_who_you_are()), &keeping).unwrap();

        assert_eq!(
            carried.as_an_address(),
            Some("http://anna:hunter2@proxy.example.com:8080".to_owned())
        );
        assert_eq!(
            carried.shown(),
            Some("http://proxy.example.com:8080".to_owned()),
            "what a person reads carries no credential"
        );
        assert!(!format!("{carried:?}").contains("hunter2"));
    }

    /// **A road that asks for nothing opens nothing.** Neither a road going
    /// straight out nor a proxy that wants no name reaches the store.
    #[test]
    fn a_road_that_asks_for_nothing_never_reaches_the_store() {
        let straight = signed_in(Way::Straight, &NeverKept).unwrap();
        assert!(straight.is_straight());

        let plain = signed_in(Way::Through(the_companys()), &NeverKept).unwrap();
        assert_eq!(
            plain.as_an_address(),
            Some("http://proxy.example.com:8080".to_owned())
        );
    }

    /// **Every refusal is a road not taken**, and not one of them is a road
    /// going straight out or a road to the proxy without the credential.
    #[test]
    fn every_refusal_is_a_road_not_taken() {
        for why in every_refusal() {
            let refused = signed_in(Way::Through(asking_who_you_are()), &Keeping::refusing(why))
                .expect_err("a road was taken with no credential");
            assert_eq!(refused, why);
            assert!(refused.nothing_was_sent());
        }
    }

    /// Every refusal there is, so that a member added to the list is a member
    /// this file's tests walk.
    fn every_refusal() -> Vec<NotSignedIn> {
        vec![
            NotSignedIn::NothingKeepsIt,
            NotSignedIn::NotKept,
            NotSignedIn::NotALookableName,
            NotSignedIn::ReadableByAnybody,
            NotSignedIn::NotRead(std::io::ErrorKind::PermissionDenied),
            NotSignedIn::LongerThanAPassword,
            NotSignedIn::NotAPassword(NotAPassword::Blank),
            NotSignedIn::NotAPassword(NotAPassword::NotSendable),
        ]
    }

    /// **Each refusal says what to do and quotes nothing**, and says nothing
    /// was sent.
    #[test]
    fn each_refusal_says_what_to_do_and_quotes_nothing() {
        let strings = in_english();
        for why in every_refusal() {
            let said = why.said(&strings);
            assert!(!said.is_a_bug(), "{why:?}: {said}");
            assert!(said.unfilled().is_empty(), "{why:?}: {said}");
            assert!(said.text().contains("nothing was sent"), "{why:?}: {said}");
            assert!(
                said.text().contains("Ask whoever manages this machine"),
                "{why:?}: {said}"
            );
        }
    }

    /// The three things an administrator does about it are three sentences,
    /// and the seven states are told apart underneath them.
    #[test]
    fn three_sentences_for_the_three_things_to_do_about_it() {
        let strings = in_english();
        let mut said: Vec<String> = every_refusal()
            .into_iter()
            .map(|why| why.said(&strings).text().to_owned())
            .collect();
        said.sort();
        said.dedup();
        assert_eq!(said.len(), 3, "{said:?}");

        assert_eq!(
            NotSignedIn::NothingKeepsIt.because(),
            WhatIsWrong::TheMachineHasNotGotIt
        );
        assert_eq!(
            NotSignedIn::ReadableByAnybody.because(),
            WhatIsWrong::AnybodyCouldReadIt
        );
        assert_eq!(
            NotSignedIn::LongerThanAPassword.because(),
            WhatIsWrong::WhatItHasCannotBeUsed
        );
    }

    /// **The refusal reaches a person in their own language.** This is the
    /// sentence somebody meets when their machine has stopped reaching
    /// anything, and English is not what everybody reads.
    #[test]
    fn a_refusal_a_person_meets_is_read_in_their_own_language() {
        let strings = translated(&[(
            words::NOT_SIGNED_IN_NO_PASSWORD,
            "Dieser Rechner hat das Kennwort nicht, nach dem der Proxy fragt. Es wurde nichts \
             gesendet",
        )]);
        let said = NotSignedIn::NotKept.said(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().contains("nichts"), "{said}");
    }

    /// **Nothing formatted anywhere on this road carries the credential** —
    /// not the refusal, not the store's answer, not what is carried.
    #[test]
    fn nothing_formatted_on_this_road_carries_the_credential() {
        let keeping = Keeping::of("the company proxy", "hunter2");
        let carried = signed_in(Way::Through(asking_who_you_are()), &keeping).unwrap();
        for formatted in [
            format!("{carried:?}"),
            format!("{:?}", carried.way()),
            format!("{:?}", asking_who_you_are()),
            format!("{:?}", NotSignedIn::NotAPassword(NotAPassword::Blank)),
            format!("{:?}", keeping.asked_for()),
        ] {
            assert!(!formatted.contains("hunter2"), "{formatted}");
        }
    }

    /// **`with_the_password` has one caller that ships**, which is
    /// [`signed_in`] above — so the one function that turns a password into
    /// text has one function above it, and `grep` finds every road through it.
    ///
    /// Read off the workspace rather than asserted in prose, because the claim
    /// is about changes nobody has written yet. A test's own call is allowed:
    /// what this holds is the shipping code.
    #[test]
    fn the_password_reaches_a_road_through_this_door_and_no_other() {
        let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("the crates directory");
        let mut calling = Vec::new();
        for crate_directory in std::fs::read_dir(crates).expect("the crates directory") {
            let source = crate_directory.expect("a crate").path().join("src");
            for file in every_rust_file(&source) {
                let read = std::fs::read_to_string(&file).expect("a source file");
                if read.contains("with_the_password(") {
                    calling.push(
                        file.strip_prefix(crates)
                            .unwrap_or(&file)
                            .to_string_lossy()
                            .replace('\\', "/"),
                    );
                }
            }
        }
        calling.sort();
        assert_eq!(
            calling,
            [
                "alo-proxy/src/carried.rs".to_owned(),
                "alo-proxy/src/signing_in.rs".to_owned(),
            ],
            "a road builds its own credential instead of going through `signed_in`"
        );
    }

    /// Every `.rs` file under a directory, however deep.
    fn every_rust_file(directory: &std::path::Path) -> Vec<std::path::PathBuf> {
        let Ok(reading) = std::fs::read_dir(directory) else {
            return Vec::new();
        };
        let mut found = Vec::new();
        for entry in reading.flatten() {
            let at = entry.path();
            if at.is_dir() {
                found.extend(every_rust_file(&at));
            } else if at.extension().is_some_and(|kind| kind == "rs") {
                found.push(at);
            }
        }
        found
    }
}
