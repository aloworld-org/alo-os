//! What a person's shell is told about the machines this one is paired with,
//! and the ones it is in the middle of pairing with.
//!
//! Four requests on the person's door — propose, confirm, revoke, list — and
//! these are the shapes their answers take. ADR 0003: a pairing is mutual,
//! deliberate, enumerated, revocable in one action and expiring, and every
//! one of those words is a field a shell can draw.
//!
//! # What a person is shown to confirm is the code and the list
//!
//! [`WaitingToPair::code`] is the six digits both people compare (ADR 0031)
//! and [`WaitingToPair::may`] is the enumerated list, each arm with the
//! sentence the person reads. A shell that confirmed a pairing without
//! drawing both would have removed the only check two people have against
//! somebody standing between their machines, and the confirmation request
//! carries the code back so that what is confirmed is what was shown.
//!
//! # Nothing here names a moment or an address
//!
//! A pairing ends in `ends_in` seconds and a proposal lapses in `lapses_in`,
//! for `standing.rs`'s reason: two ends of a socket have two clocks. Where the
//! other machine answers is not here at all — discovery measured it and the
//! daemon dials it, and a shell has no use for an address it could not act on
//! (`docs/contracts/local-network-wire.md`).
//!
//! # A machine is named by its identity, and called by its name
//!
//! `machine` is the identity discovery found the other machine by — thirty-two
//! characters a person did not choose — and it is what every request names a
//! machine by. `called`, beside it on a pairing, is the name the person here
//! gave that machine (`name-machine`), kept on this machine in the person's own
//! file and absent until they give one. It is for reading and decides nothing:
//! it never reaches the other machine, and nothing finds, dials or proves a
//! machine by it.

use alo_strings::Said;
use serde::{Deserialize, Serialize};

use crate::wording::Wording;

/// One thing a paired machine may ask this one for, as a person reads it.
///
/// `named` is the word the wire spells it by, which is what a shell sends back
/// in a proposal; `sentence` is what the person reads beside it. Both, so that
/// a shell never has to translate the list itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Permitted {
    /// The word on the wire.
    named: String,
    /// The sentence a person reads.
    sentence: Wording,
}

impl Permitted {
    /// One arm, by its wire word and the sentence declared for it.
    #[must_use]
    pub fn of(named: &str, sentence: &Said) -> Self {
        Self {
            named: named.to_owned(),
            sentence: Wording::of(sentence),
        }
    }

    /// The word on the wire.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.named
    }

    /// The sentence a person reads.
    #[must_use]
    pub fn sentence(&self) -> &Wording {
        &self.sentence
    }
}

/// Which of the two machines the person is standing in front of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SideOf {
    /// This machine proposed.
    Asking,
    /// This machine was proposed to.
    Asked,
}

/// Where the two people stand on a proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Confirmed {
    /// Whether the person at this machine has confirmed.
    pub here: bool,
    /// Whether the person at the other machine has confirmed, as far as this
    /// machine has been told and could verify.
    pub there: bool,
}

/// A proposal waiting for two people, as a shell draws it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WaitingToPair {
    /// The other machine, by its identity.
    machine: String,
    /// Which machine this one is in the proposal.
    side: SideOf,
    /// What the other machine would be permitted to ask for.
    may: Vec<Permitted>,
    /// How long the pairing would last, in seconds, as stated where it was made.
    seconds: u64,
    /// The six digits both people compare, once the other machine has
    /// answered; nothing before then, and there is nothing to confirm before
    /// then.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    /// Where the two people stand.
    confirmed: Confirmed,
    /// How many seconds are left before the proposal lapses, and nothing once
    /// it has.
    lapses_in: Option<u64>,
}

impl WaitingToPair {
    /// A proposal waiting, as it goes on the wire.
    #[must_use]
    pub fn of(
        machine: &str,
        side: SideOf,
        may: Vec<Permitted>,
        seconds: u64,
        code: Option<&str>,
        confirmed: Confirmed,
        lapses_in: Option<u64>,
    ) -> Self {
        Self {
            machine: machine.to_owned(),
            side,
            may,
            seconds,
            code: code.map(ToOwned::to_owned),
            confirmed,
            lapses_in,
        }
    }

    /// The other machine, by its identity.
    #[must_use]
    pub fn machine(&self) -> &str {
        &self.machine
    }

    /// Which machine this one is in the proposal.
    #[must_use]
    pub const fn side(&self) -> SideOf {
        self.side
    }

    /// What the other machine would be permitted to ask for.
    #[must_use]
    pub fn may(&self) -> &[Permitted] {
        &self.may
    }

    /// How long the pairing would last, in seconds.
    #[must_use]
    pub const fn seconds(&self) -> u64 {
        self.seconds
    }

    /// The six digits both people compare, once known here.
    #[must_use]
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// Where the two people stand.
    #[must_use]
    pub const fn confirmed(&self) -> Confirmed {
        self.confirmed
    }

    /// How many seconds are left before it lapses.
    #[must_use]
    pub const fn lapses_in(&self) -> Option<u64> {
        self.lapses_in
    }
}

/// One machine this one is paired with, as a shell draws it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paired {
    /// The other machine, by its identity.
    machine: String,
    /// What it may ask this machine for.
    may: Vec<Permitted>,
    /// How many seconds ago the two people agreed.
    made_ago: u64,
    /// How many seconds are left before the pairing ends.
    ends_in: u64,
    /// Whether this pairing lets the person here choose that machine to
    /// answer their questions at the moment — so a shell offers only what can
    /// be chosen. Absent in a list written before this field existed, which
    /// reads as *no*.
    #[serde(default)]
    may_answer_questions: bool,
    /// What the person here called that machine, if they gave it a name.
    /// Absent — never empty — when they did not, and in a list written before
    /// this field existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    called: Option<String>,
}

impl Paired {
    /// A pairing, as it goes on the wire.
    #[must_use]
    pub fn of(machine: &str, may: Vec<Permitted>, made_ago: u64, ends_in: u64) -> Self {
        Self {
            machine: machine.to_owned(),
            may,
            made_ago,
            ends_in,
            may_answer_questions: false,
            called: None,
        }
    }

    /// The same pairing, carrying the name the person here gave that machine.
    #[must_use]
    pub fn that_is_called(mut self, called: Option<&str>) -> Self {
        self.called = called.map(ToOwned::to_owned);
        self
    }

    /// What the person here called that machine, if they gave it a name.
    #[must_use]
    pub fn called(&self) -> Option<&str> {
        self.called.as_deref()
    }

    /// The same pairing, saying whether the person may choose that machine to
    /// answer their questions.
    #[must_use]
    pub const fn that_may_answer_questions(mut self, may: bool) -> Self {
        self.may_answer_questions = may;
        self
    }

    /// Whether the person may choose that machine to answer their questions
    /// at the moment the list was made.
    #[must_use]
    pub const fn may_answer_questions(&self) -> bool {
        self.may_answer_questions
    }

    /// The other machine, by its identity.
    #[must_use]
    pub fn machine(&self) -> &str {
        &self.machine
    }

    /// What it may ask this machine for.
    #[must_use]
    pub fn may(&self) -> &[Permitted] {
        &self.may
    }

    /// How many seconds ago the two people agreed.
    #[must_use]
    pub const fn made_ago(&self) -> u64 {
        self.made_ago
    }

    /// How many seconds are left before it ends.
    #[must_use]
    pub const fn ends_in(&self) -> u64 {
        self.ends_in
    }
}

/// What became of the person's confirmation.
///
/// Three, and the third is the honest one: a pairing is made the moment both
/// people have confirmed, whether or not this machine could then write it
/// down. One that could not be written stands until the machine restarts, and
/// the person is told so rather than told it was refused — it was not — or
/// told nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AfterConfirming {
    /// The person here has confirmed; the other has not yet.
    WaitingForTheOtherPerson,
    /// Both have confirmed, and the pairing is kept and written down.
    Paired,
    /// Both have confirmed, and the pairing could not be written down: it
    /// stands until this machine restarts.
    PairedUntilARestart,
}

/// What became of the person's revocation.
///
/// A revocation takes effect at once whether or not the file could be
/// written; the second arm says the file could not be, so that a person is not
/// surprised by a pairing coming back after a restart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AfterRevoking {
    /// Revoked, and written down as such.
    Revoked,
    /// Revoked until this machine restarts, because the file could not be
    /// written.
    RevokedUntilARestart,
}

/// What became of a name the person gave a machine, or took away.
///
/// A name takes effect at once whether or not the file could be written; the
/// second arm says the file could not be, so that a person is not surprised by
/// a name coming back — or going — after a restart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AfterNaming {
    /// Named, or the name taken away, and written down as such.
    Kept,
    /// Named, or the name taken away, until this machine restarts, because the
    /// file could not be written.
    KeptUntilARestart,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use crate::words;
    use alo_strings::Filling;

    /// A list with one arm on it, in this crate's own words for want of the
    /// pairing crate's.
    fn a_list() -> Vec<Permitted> {
        vec![Permitted::of(
            "models",
            &in_english().say(&words::NOT_READABLE.key(), &Filling::nothing()),
        )]
    }

    /// **What a shell draws to confirm is the code and the list**, and a
    /// proposal not yet answered carries no code at all rather than an empty
    /// one.
    #[test]
    fn what_is_waiting_carries_the_code_and_the_list_and_reads_back() {
        let shown = WaitingToPair::of(
            "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            SideOf::Asked,
            a_list(),
            86_400,
            Some("482910"),
            Confirmed {
                here: false,
                there: true,
            },
            Some(540),
        );
        assert_eq!(shown.code(), Some("482910"));
        assert_eq!(shown.may().first().unwrap().named(), "models");
        let written = serde_json::to_string(&shown).unwrap();
        assert!(written.contains(r#""code":"482910""#), "{written}");
        assert!(written.contains(r#""side":"asked""#), "{written}");
        let back: WaitingToPair = serde_json::from_str(&written).unwrap();
        assert_eq!(back, shown);

        let unanswered = WaitingToPair::of(
            "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            SideOf::Asking,
            a_list(),
            86_400,
            None,
            Confirmed {
                here: false,
                there: false,
            },
            Some(600),
        );
        let written = serde_json::to_string(&unanswered).unwrap();
        assert!(!written.contains("code"), "{written}");
        let back: WaitingToPair = serde_json::from_str(&written).unwrap();
        assert_eq!(back.code(), None);
    }

    /// **A pairing is drawn with when it was made and when it ends, as
    /// durations**, and nothing here names a moment or an address.
    #[test]
    fn a_pairing_is_drawn_with_durations_and_no_address() {
        let paired = Paired::of("0f1e2d3c4b5a69788796a5b4c3d2e1f0", a_list(), 60, 86_340);
        let written = serde_json::to_string(&paired).unwrap();
        assert!(written.contains(r#""made_ago":60"#), "{written}");
        assert!(written.contains(r#""ends_in":86340"#), "{written}");
        for not_here in ["address", "port", "trusted", "\"name\""] {
            assert!(!written.contains(not_here), "{written}");
        }
        let back: Paired = serde_json::from_str(&written).unwrap();
        assert_eq!(back, paired);
    }

    /// **A pairing says whether its machine may be chosen to answer
    /// questions**, and a list written before the field existed reads as *no*
    /// rather than as a machine a shell may offer.
    #[test]
    fn a_pairing_says_whether_its_machine_may_answer_questions() {
        let paired = Paired::of("0f1e2d3c4b5a69788796a5b4c3d2e1f0", a_list(), 60, 86_340)
            .that_may_answer_questions(true);
        assert!(paired.may_answer_questions());
        let written = serde_json::to_string(&paired).unwrap();
        assert!(
            written.contains(r#""may_answer_questions":true"#),
            "{written}"
        );
        assert_eq!(serde_json::from_str::<Paired>(&written).unwrap(), paired);

        let older: Paired = serde_json::from_str(
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","may":[],"made_ago":1,"ends_in":1}"#,
        )
        .unwrap();
        assert!(!older.may_answer_questions());
    }

    /// **A pairing carries the name the person here gave its machine beside
    /// the identity**, absent rather than empty when there is none, and a list
    /// written before the field existed reads as having none.
    #[test]
    fn a_pairing_carries_its_machines_name_beside_the_identity() {
        let called = Paired::of("0f1e2d3c4b5a69788796a5b4c3d2e1f0", a_list(), 60, 86_340)
            .that_is_called(Some("the reception machine"));
        assert_eq!(called.called(), Some("the reception machine"));
        assert_eq!(called.machine(), "0f1e2d3c4b5a69788796a5b4c3d2e1f0");
        let written = serde_json::to_string(&called).unwrap();
        assert!(
            written.contains(r#""machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0""#)
                && written.contains(r#""called":"the reception machine""#),
            "{written}"
        );
        assert_eq!(serde_json::from_str::<Paired>(&written).unwrap(), called);

        let nameless = Paired::of("0f1e2d3c4b5a69788796a5b4c3d2e1f0", a_list(), 60, 86_340)
            .that_is_called(None);
        assert!(!serde_json::to_string(&nameless).unwrap().contains("called"));
        let older: Paired = serde_json::from_str(
            r#"{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","may":[],"made_ago":1,"ends_in":1}"#,
        )
        .unwrap();
        assert_eq!(older.called(), None);
        assert_eq!(
            serde_json::to_string(&AfterNaming::KeptUntilARestart).unwrap(),
            r#""kept-until-a-restart""#
        );
    }

    /// A field nobody declared is refused rather than read around, so a
    /// shell cannot be handed *trusted* under another name.
    #[test]
    fn a_field_nobody_declared_is_refused() {
        for message in [
            r#"{"machine":"m","may":[],"made_ago":1,"ends_in":1,"trusted":true}"#,
            r#"{"machine":"m","may":[],"made_ago":1,"ends_in":1,"address":"10.0.0.1"}"#,
        ] {
            assert!(
                serde_json::from_str::<Paired>(message).is_err(),
                "{message}"
            );
        }
    }

    /// The two outcomes are spelt in kebab-case, one word each way.
    #[test]
    fn what_became_of_a_confirmation_or_a_revocation_reads_back() {
        for became in [
            AfterConfirming::WaitingForTheOtherPerson,
            AfterConfirming::Paired,
            AfterConfirming::PairedUntilARestart,
        ] {
            let written = serde_json::to_string(&became).unwrap();
            assert_eq!(
                serde_json::from_str::<AfterConfirming>(&written).unwrap(),
                became
            );
        }
        assert_eq!(
            serde_json::to_string(&AfterRevoking::RevokedUntilARestart).unwrap(),
            r#""revoked-until-a-restart""#
        );
    }
}
