//! What `alo-going-back.service` does: decide going back again for itself,
//! check it is the one a person approved, and tell the base.
//!
//! This is the privileged half of `updates.roll-back`
//! ([ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md),
//! option B).
//!
//! # Nothing is handed over but the identity, and that is the whole design
//!
//! Going back has no argument the machine did not already have: the build
//! before, the build running and the update that would be set aside are in the
//! base's own status and the machine's record. So the broker hands over the
//! **identity the person approved** and nothing else, and this program decides
//! `alo_keeping_up::GoingBack` again from the machine in front of it. If what
//! it decides does not digest to that identity, the machine is not the one the
//! person was shown and nothing is set — whether because another build started
//! waiting, because the build before changed, or because somebody wrote a
//! different file into the folder.
//!
//! # It reaches no network, and takes no road
//!
//! The build is already on the disk. There is no download here, no proxy to
//! read and no credential to hold, and the unit says so line by line.
//!
//! # And it cannot restart the machine
//!
//! `alo_keeping_up::Returning` is asked for *at the next restart*, so the
//! instruction is one word and never carries `--apply`; the unit's capability
//! set holds no `CAP_SYS_BOOT`. *Restart and go back now* is the person's
//! restart through their own session, afterwards.

use std::path::Path;

use alo_broker::Identity;
use alo_keeping_up::{CannotGoBack, Returning, WhenItApplies};
use alo_updating::{AcrossRestarts, Base, NotGoneBack, NotRead, go_back, yesterday};

use crate::approved::GoingBackApproved;
use crate::for_the_unit::{self, Handing, NotHanded};

/// The machine's own record, where the update that replaced the build before
/// was written down.
///
/// `alo_updating::yesterday` reads it to say **when** the build before was
/// replaced and to tell an update apart from a return. It is read and never
/// written here: the broker's own record is somewhere else
/// (`crate::recording`), and this program writes nothing to either.
pub const THE_MACHINES_RECORD: &str = "/var/lib/alo/record.jsonl";

/// The most bytes the identity handed over may be: sixty-four characters and a
/// newline, with room for neither a second line nor a paragraph.
pub const LONGEST_HANDED_OVER: u64 = 128;

/// Why going back was not set. **On every one of these the machine starts the
/// build it runs now.**
#[derive(Debug)]
pub enum NotSetByTheUnit {
    /// The broker handed nothing over, or not a file of root's.
    NotHanded(NotHanded),
    /// What was handed over is not an identity.
    NotAnIdentity,
    /// The machine could not be read, so nothing was decided about it.
    NotRead(NotRead),
    /// Going back is not something this machine can offer at all.
    NotOffered(CannotGoBack),
    /// Going back as this machine is now is not what the person approved.
    NotWhatWasApproved,
    /// It was decided and the base did not set it, or it could not be noted.
    NotGoneBack(NotGoneBack),
}

impl std::fmt::Display for NotSetByTheUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotHanded(why) => write!(f, "nothing was handed over to go back to: {why}"),
            Self::NotAnIdentity => write!(
                f,
                "what was handed over is not an identity, so nothing was set"
            ),
            Self::NotRead(why) => write!(f, "this machine could not be read: {why:?}"),
            Self::NotOffered(why) => write!(
                f,
                "this machine cannot go back to the build before: {why:?}"
            ),
            Self::NotWhatWasApproved => write!(
                f,
                "going back on this machine now is not what the person approved, so nothing was \
                 set"
            ),
            Self::NotGoneBack(why) => write!(f, "going back was not set: {why:?}"),
        }
    }
}

impl std::error::Error for NotSetByTheUnit {}

/// Carry `updates.roll-back` out: set the machine to start the build before at
/// the next restart, when that is what the identity handed over stands for.
///
/// # Errors
/// [`NotSetByTheUnit`], on every one of which the machine starts the build it
/// runs now.
pub fn carried_out(
    base: &impl Base,
    handing: &Handing,
    record_at: &Path,
    kept: &AcrossRestarts,
) -> Result<Returning, NotSetByTheUnit> {
    let bytes = for_the_unit::handed(&handing.going_back(), LONGEST_HANDED_OVER)
        .map_err(NotSetByTheUnit::NotHanded)?;
    let approved = std::str::from_utf8(&bytes)
        .ok()
        .map(str::trim)
        .and_then(Identity::read)
        .ok_or(NotSetByTheUnit::NotAnIdentity)?;

    let machine = yesterday(base, record_at).map_err(NotSetByTheUnit::NotRead)?;
    let offer = machine
        .going_back()
        .map_err(|why| NotSetByTheUnit::NotOffered(why.clone()))?;
    if GoingBackApproved::offered(offer).identity() != approved {
        return Err(NotSetByTheUnit::NotWhatWasApproved);
    }
    go_back(base, offer, kept, WhenItApplies::AtTheNextRestart)
        .map_err(NotSetByTheUnit::NotGoneBack)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::rc::Rc;

    use alo_keeping_up::{Digest, GoingBack};
    use alo_updating::{NotAnswered, THE_STATUS};

    use super::*;

    /// A whole digest made of one repeated pair.
    fn whole(pair: &str) -> Digest {
        Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
    }

    /// What `bootc status` answers on a machine booted on `booted`, keeping
    /// `rollback`, with `staged` waiting.
    fn an_answer(booted: &Digest, staged: Option<&Digest>, rollback: Option<&Digest>) -> Vec<u8> {
        format!(
            r#"{{"apiVersion":"org.containers.bootc/v1","kind":"BootcHost",
            "spec":{{"image":{{"image":"ghcr.io/aloworld-org/alo-os","transport":"registry"}}}},
            "status":{{"staged":{},"rollback":{},"rollbackQueued":false,
              "booted":{},"type":"bootcHost"}}}}"#,
            staged.map_or_else(|| "null".to_owned(), deployment),
            rollback.map_or_else(|| "null".to_owned(), deployment),
            deployment(booted)
        )
        .into_bytes()
    }

    /// One deployment in that answer.
    fn deployment(digest: &Digest) -> String {
        format!(
            r#"{{"image":{{"image":{{"image":"ghcr.io/aloworld-org/alo-os","transport":"registry"}},
            "version":"0.0.5","imageDigest":"{}","architecture":"amd64"}},
            "ostree":{{"checksum":"abc","deploySerial":0,"stateroot":"default"}}}}"#,
            digest.as_str()
        )
    }

    /// What one stand-in base answers and remembers.
    #[derive(Debug)]
    struct Answers {
        /// What it answers `status` with.
        status: Vec<u8>,
        /// Every argument list it was given that was not the status.
        told: RefCell<Vec<Vec<String>>>,
    }

    /// A base answering one machine's status.
    #[derive(Debug, Clone)]
    struct ABase(Rc<Answers>);

    impl ABase {
        fn answering(status: Vec<u8>) -> Self {
            Self(Rc::new(Answers {
                status,
                told: RefCell::new(Vec::new()),
            }))
        }

        fn told(&self) -> Vec<Vec<String>> {
            self.0.told.borrow().clone()
        }
    }

    impl Base for ABase {
        fn asked(&self, arguments: &[String]) -> Result<Vec<u8>, NotAnswered> {
            if arguments == THE_STATUS {
                return Ok(self.0.status.clone());
            }
            self.0.told.borrow_mut().push(arguments.to_vec());
            Ok(Vec::new())
        }
    }

    /// A folder of this test's own, emptied.
    fn a_folder(named: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!(
            "alo-brokerd-returning-{}-{named}",
            std::process::id()
        ));
        drop(std::fs::remove_dir_all(&at));
        std::fs::create_dir_all(&at).unwrap();
        at
    }

    /// The machine as a test sets it up: what the base answers, what the
    /// broker handed over, and where the two files kept across a restart go.
    struct AMachine {
        base: ABase,
        handing: Handing,
        record: PathBuf,
        kept: AcrossRestarts,
    }

    /// A machine that updated from `aa` to `bb` and could go back, with
    /// whatever identity the broker handed over.
    fn a_machine(named: &str, handed: &str, staged: Option<&Digest>) -> AMachine {
        let folder = a_folder(named);
        let handing = Handing::at(&folder);
        let mut bytes = handed.as_bytes().to_vec();
        bytes.push(b'\n');
        for_the_unit::put(&handing.going_back(), &bytes).unwrap();
        AMachine {
            base: ABase::answering(an_answer(&whole("bb"), staged, Some(&whole("aa")))),
            handing,
            record: folder.join("record.jsonl"),
            kept: AcrossRestarts::in_folder(&folder),
        }
    }

    /// The offer this machine makes, decided the way the unit decides it.
    fn the_offer(machine: &AMachine) -> GoingBack {
        yesterday(&machine.base, &machine.record)
            .unwrap()
            .going_back()
            .unwrap()
            .clone()
    }

    /// **Going back a person approved is set, with the base told one word.**
    #[test]
    fn going_back_that_was_approved_is_set_and_the_base_is_told_one_word() {
        let machine = a_machine("carried", "", None);
        let approved = GoingBackApproved::offered(&the_offer(&machine)).identity();
        let mut bytes = approved.written().into_bytes();
        bytes.push(b'\n');
        for_the_unit::put(&machine.handing.going_back(), &bytes).unwrap();

        let returning = carried_out(
            &machine.base,
            &machine.handing,
            &machine.record,
            &machine.kept,
        )
        .unwrap();
        assert_eq!(returning.to(), &whole("aa"));
        assert_eq!(machine.base.told(), vec![vec!["rollback".to_owned()]]);
    }

    /// **Nothing the unit tells the base restarts the machine.**
    #[test]
    fn nothing_the_unit_tells_the_base_restarts_the_machine() {
        let machine = a_machine("no-apply", "", None);
        let approved = GoingBackApproved::offered(&the_offer(&machine)).identity();
        let mut bytes = approved.written().into_bytes();
        bytes.push(b'\n');
        for_the_unit::put(&machine.handing.going_back(), &bytes).unwrap();
        carried_out(
            &machine.base,
            &machine.handing,
            &machine.record,
            &machine.kept,
        )
        .unwrap();
        for told in machine.base.told() {
            assert!(!told.contains(&"--apply".to_owned()), "{told:?}");
        }
    }

    /// **A machine that is not the one the person approved sets nothing.**
    ///
    /// The case ADR 0053 names by hand: an update started waiting since the
    /// offer, which going back would set aside — so what would happen is not
    /// what the person was shown, and it is refused before the base is told.
    #[test]
    fn a_machine_with_an_update_waiting_since_the_offer_is_refused() {
        let as_offered = a_machine("as-offered", "", None);
        let approved = GoingBackApproved::offered(&the_offer(&as_offered)).identity();

        let since = a_machine("since", &approved.written(), Some(&whole("cc")));
        let refused =
            carried_out(&since.base, &since.handing, &since.record, &since.kept).unwrap_err();
        assert!(
            matches!(refused, NotSetByTheUnit::NotWhatWasApproved),
            "{refused}"
        );
        assert!(since.base.told().is_empty(), "the base was told something");
    }

    /// **An identity for some other machine's offer sets nothing.**
    #[test]
    fn an_identity_that_is_not_this_machines_offer_sets_nothing() {
        let somebody_elses = Identity::of_what_was_reported(b"another machine's offer");
        let machine = a_machine("not-ours", &somebody_elses.written(), None);
        let refused = carried_out(
            &machine.base,
            &machine.handing,
            &machine.record,
            &machine.kept,
        )
        .unwrap_err();
        assert!(
            matches!(refused, NotSetByTheUnit::NotWhatWasApproved),
            "{refused}"
        );
        assert!(machine.base.told().is_empty());
    }

    /// **A machine with nothing to go back to is refused before anything**,
    /// and never told to go back and left to fail half way.
    #[test]
    fn a_machine_with_nothing_to_go_back_to_sets_nothing() {
        let folder = a_folder("nothing-before");
        let handing = Handing::at(&folder);
        for_the_unit::put(
            &handing.going_back(),
            format!(
                "{}\n",
                Identity::of_what_was_reported(b"anything").written()
            )
            .as_bytes(),
        )
        .unwrap();
        let base = ABase::answering(an_answer(&whole("bb"), None, None));
        let refused = carried_out(
            &base,
            &handing,
            &folder.join("record.jsonl"),
            &AcrossRestarts::in_folder(&folder),
        )
        .unwrap_err();
        assert!(
            matches!(
                refused,
                NotSetByTheUnit::NotOffered(CannotGoBack::NothingBefore)
            ),
            "{refused}"
        );
        assert!(base.told().is_empty());
    }

    /// **A unit started with nothing handed over sets nothing**, which is what
    /// somebody starting the unit by hand meets.
    #[test]
    fn a_unit_started_with_nothing_handed_over_sets_nothing() {
        let folder = a_folder("nothing-handed");
        let handing = Handing::at(&folder);
        let base = ABase::answering(an_answer(&whole("bb"), None, Some(&whole("aa"))));
        let refused = carried_out(
            &base,
            &handing,
            &folder.join("record.jsonl"),
            &AcrossRestarts::in_folder(&folder),
        )
        .unwrap_err();
        assert!(
            matches!(refused, NotSetByTheUnit::NotHanded(_)),
            "{refused}"
        );
        assert!(base.told().is_empty());
    }

    /// **Something that is not an identity sets nothing.**
    #[test]
    fn something_that_is_not_an_identity_sets_nothing() {
        for handed in ["", "rollback --apply", "NOTHEX", &"ab".repeat(33)] {
            let machine = a_machine("not-an-identity", handed, None);
            let refused = carried_out(
                &machine.base,
                &machine.handing,
                &machine.record,
                &machine.kept,
            )
            .unwrap_err();
            assert!(
                matches!(refused, NotSetByTheUnit::NotAnIdentity),
                "{handed:?} was read as an identity"
            );
            assert!(machine.base.told().is_empty());
        }
    }

    /// **The record is read and never written.** A return this machine sets is
    /// written down at the next start by `alo_updating::after_a_restart`, from
    /// the note kept beside it, and nothing here touches the record at all.
    #[test]
    fn the_record_is_read_and_never_written() {
        let machine = a_machine("read-only", "", None);
        let approved = GoingBackApproved::offered(&the_offer(&machine)).identity();
        for_the_unit::put(
            &machine.handing.going_back(),
            format!("{}\n", approved.written()).as_bytes(),
        )
        .unwrap();
        carried_out(
            &machine.base,
            &machine.handing,
            &machine.record,
            &machine.kept,
        )
        .unwrap();
        assert!(
            !machine.record.exists(),
            "the unit wrote to the machine's record"
        );
        assert!(
            machine.kept.going_back_to().exists(),
            "the build chosen was not noted, so the next start would be recorded as an update"
        );
    }

    /// **The record this machine reads is the one the rest of alo OS writes.**
    #[test]
    fn the_record_read_is_the_machines_own() {
        assert_eq!(THE_MACHINES_RECORD, "/var/lib/alo/record.jsonl");
    }
}
