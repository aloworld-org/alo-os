//! Asking what this machine did, and being answered from the record on the
//! disk.
//!
//! The plan's first acceptance is a sentence about where the answer comes from
//! — *a person asks and is answered from the record on the disk, not from
//! memory of the session* — so that is what this type is shaped around rather
//! than what its documentation promises.
//!
//! # It holds a path, and it holds nothing else
//!
//! [`Recounting`] is a place on a disk. It keeps no record, no account, no
//! entries and no answer from last time, and there is no constructor that takes
//! any of those. Every question re-reads the file through
//! `alo_keeping::Reading`, which is the crate that knows what a record is.
//!
//! That is not a performance decision made the lazy way round; it is the whole
//! guarantee. A surface holding entries in memory would answer *what did the
//! agent do* from whatever it was told during this session, and the two things
//! that would then differ from the file are exactly the two that matter: an
//! entry the daemon failed to write, and a record that has been shortened or
//! tampered with since. **The screen would show the version nothing could be
//! checked against.**
//!
//! # A record that is not there is not an empty answer
//!
//! `alo_keeping::Reading::at` refuses a missing file rather than answering with
//! a record of nothing, and this crate carries that refusal all the way to the
//! surface: [`Recounts::Refused`] with `alo_keeping::NotKept::NotThere`, whose
//! sentence says why a missing record is worth finding out about.
//! [`crate::NotRecounted::there_is_no_record`] is how a shell tells that from
//! every other refusal, because the one thing it must not do is draw an empty
//! list.
//!
//! # Nothing here is reachable by an agent
//!
//! There is no verb that reads the record and there could not be one. An agent
//! able to ask what it did is an agent able to find out what it has already
//! been refused, one call at a time, and to shape the next attempt around it;
//! and an agent able to hand a record to a surface is an agent that can hand
//! over a record of its own. `alo-protocol` has nothing that reaches this.
//!
//! **A person's own record is not an agent's context** either: ADR 0001 §4's
//! *context is offered, never watched* is why nothing here is reachable from a
//! turn.

use std::path::{Path, PathBuf};

use alo_keeping::Reading;
use alo_record::Asking;

use crate::account::Account;
use crate::bounding::AtMost;
use crate::refusing::NotRecounted;
use crate::surface::Compositor;
use crate::where_it_is::{THE_DESCRIPTION, where_the_record_is};

/// What one call to [`Recounting::show`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recounts {
    /// The account is in front of the person, as the record has it.
    Shown(Account),
    /// It could not be put to them, and this is what to say instead — in their
    /// language, through [`NotRecounted::said`].
    Refused(NotRecounted),
}

/// The record on this machine, asked what it did.
///
/// One per machine: the daemon knows where the record is kept, and a shell is
/// told. It holds no compositor, because whether there is one is a fact about
/// the session that can change while the machine runs — so it is handed in at
/// the moment of the call, exactly as the agent overlay's summoning, the egress
/// indicator and the approval surface take one.
///
/// ```
/// use alo_keeping::Writing;
/// use alo_record::{Asking, Entry, Only};
/// use alo_recounting::{AtMost, Outcome, Recounting};
/// use alo_capability::Grantee;
/// use std::time::{Duration, SystemTime};
///
/// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
/// let folder = std::env::temp_dir().join(format!("alo-recounting-doc-{}", std::process::id()));
/// std::fs::create_dir_all(&folder).expect("somewhere to keep a record");
/// let kept_at = folder.join("record.jsonl");
/// let _ = std::fs::remove_file(&kept_at);
///
/// // The daemon keeps the record. Nothing in this crate writes to it.
/// let mut writing = Writing::opening(&kept_at).expect("a record to write to");
/// writing
///     .keep(&Entry::answered_here(&Grantee::named("@files"), now))
///     .expect("an answer given on this machine");
/// drop(writing);
/// # #[cfg(unix)]
/// # {
/// #     use std::os::unix::fs::PermissionsExt;
/// #     let ours = std::fs::Permissions::from_mode(0o600);
/// #     std::fs::set_permissions(&kept_at, ours).expect("a record of our own");
/// # }
///
/// // Afterwards, somebody asks what it did — and the answer comes off the disk.
/// let recounting = Recounting::kept_at(&kept_at);
/// let account = recounting.about(&Asking::anything(), AtMost::ONE_SITTING).expect("this machine's record");
/// assert_eq!(account.how_many(), 1);
/// assert_eq!(
///     account.told().first().map(|told| told.outcome()),
///     Some(Outcome::AnsweredHere),
/// );
/// assert!(account.goes_all_the_way_back());
///
/// // Nothing was refused, so a question about refusals answers with nothing —
/// // and says so, rather than being an empty screen.
/// let refusals = recounting
///     .about(&Asking::anything().only(Only::Refusals), AtMost::ONE_SITTING)
///     .expect("this machine's record");
/// assert!(refusals.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recounting {
    /// Where the record is.
    kept_at: PathBuf,
}

impl Recounting {
    /// The record kept at this path.
    ///
    /// The only constructor, and it takes a place rather than a record: see
    /// this module's documentation for why that absence is the guarantee.
    #[must_use]
    pub fn kept_at(path: &Path) -> Self {
        Self {
            kept_at: path.to_path_buf(),
        }
    }

    /// The record this machine says it keeps.
    ///
    /// The door a surface on a running machine uses, and the one that makes an
    /// account reachable at all: where the record is kept is
    /// `docs/contracts/machine-description.md`'s `[record].path`, and until
    /// something read it on the person's behalf the only caller that could name
    /// the path was a test. `where_it_is.rs` is what is read and what is
    /// deliberately ignored.
    ///
    /// # Errors
    ///
    /// [`NotRecounted::Nowhere`] when this machine cannot say where it keeps a
    /// record — never *nothing happened*, which is a different sentence about a
    /// different machine.
    pub fn on_this_machine() -> Result<Self, NotRecounted> {
        Self::described_at(Path::new(THE_DESCRIPTION))
    }

    /// The same, from a description somewhere else.
    ///
    /// What [`Recounting::on_this_machine`] is, with the path written down:
    /// whoever stands a machine up, and this crate's own tests, are the two
    /// callers that name one.
    ///
    /// # Errors
    ///
    /// [`NotRecounted::Nowhere`], as above.
    pub fn described_at(description: &Path) -> Result<Self, NotRecounted> {
        where_the_record_is(description)
            .map(|kept_at| Self { kept_at })
            .map_err(NotRecounted::Nowhere)
    }

    /// Where the record is.
    #[must_use]
    pub fn where_it_is(&self) -> &Path {
        &self.kept_at
    }

    /// Ask the record a question, and be answered from the file.
    ///
    /// The file is read every time. What is asked is `alo_record::Asking` — the
    /// record's own question, in the record's own terms — because a surface
    /// with a question language of its own would be answering by matching text
    /// against lines that were worded for somebody else.
    ///
    /// **It is read through `alo_keeping::Reading::believed_at`**, which asks
    /// who may have written the file before it reads a word of it — not a link,
    /// root's or the person's own, and nobody else able to write it. A person
    /// being shown an account of their own machine is being asked to believe a
    /// file, and a record somebody else could have written is not evidence of
    /// anything.
    ///
    /// **And it is bounded.** [`AtMost`] is not optional and there is no door
    /// beside this one that answers with the whole record; what is kept is the
    /// most recent of what answered, and [`Account::is_all_that_answered`] says
    /// whether anything was left out.
    ///
    /// # Errors
    ///
    /// [`NotRecounted::Record`] carrying `alo_keeping::NotKept`, worded by the
    /// crate that refused: there is no record there, what is there is not one,
    /// it was written by a newer alo OS, the file is one this machine will not
    /// believe, or the machine would not read it. A missing record is **not** an
    /// empty account, and [`NotRecounted::there_is_no_record`] is how a caller
    /// tells them apart.
    pub fn about(&self, asking: &Asking, most: AtMost) -> Result<Account, NotRecounted> {
        let reading = Reading::believed_at(&self.kept_at).map_err(NotRecounted::Record)?;
        Ok(Account::of(&reading, asking, most))
    }

    /// Ask, and put the answer in front of the person.
    ///
    /// The compositor is asked first, because reading a year of evidence to
    /// discover there is nowhere to show it is work nobody asked for. With no
    /// compositor, or one that refuses, the answer is a refusal with a sentence
    /// in it — a person who asked what their machine did and was shown nothing
    /// has been told something about their machine, and on a machine whose
    /// record cannot be reached that something is false.
    pub fn show(
        &self,
        compositor: Option<&mut dyn Compositor>,
        asking: &Asking,
        most: AtMost,
    ) -> Recounts {
        let Some(compositor) = compositor else {
            return Recounts::Refused(NotRecounted::NoCompositor);
        };
        let account = match self.about(asking, most) {
            Ok(account) => account,
            Err(why) => return Recounts::Refused(why),
        };
        match compositor.show(account.clone()) {
            Ok(()) => Recounts::Shown(account),
            Err(refused) => Recounts::Refused(NotRecounted::Surface(refused)),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::surface::SurfaceRefused;
    use crate::testing::{
        a_record_at, an_afternoon, answered_here, archived, in_english, somewhere_of_our_own,
    };
    use crate::told::{Outcome, Told};
    use alo_keeping::{NotKept, Writing};
    use alo_record::Only;

    /// A screen, as this surface sees one: it keeps every account it was given,
    /// so *one question, one account* is a number rather than an impression.
    #[derive(Default)]
    struct Screen {
        /// Every account that arrived, in order.
        shown: Vec<Account>,
    }

    impl Compositor for Screen {
        fn show(&mut self, account: Account) -> Result<(), SurfaceRefused> {
            self.shown.push(account);
            Ok(())
        }
    }

    /// A compositor with nothing to put an account on.
    #[derive(Default)]
    struct NoScreen {
        /// How many accounts were offered to it and refused.
        offered: usize,
    }

    impl Compositor for NoScreen {
        fn show(&mut self, _account: Account) -> Result<(), SurfaceRefused> {
            self.offered += 1;
            Err(SurfaceRefused::NothingToShowOn)
        }
    }

    /// **The answer comes off the disk, every time.** An entry written after
    /// the question was first asked is in the second answer, which is only true
    /// of something that reads the file rather than remembering it.
    #[test]
    fn the_answer_is_read_off_the_disk_every_time_it_is_asked() {
        let kept_at = somewhere_of_our_own("off-the-disk");
        a_record_at(&kept_at, &[archived()]);
        let recounting = Recounting::kept_at(&kept_at);
        assert_eq!(recounting.where_it_is(), kept_at.as_path());

        let first = recounting
            .about(&Asking::anything(), AtMost::ONE_SITTING)
            .unwrap();
        assert_eq!(first.how_many(), 1);

        // The machine keeps working while somebody is reading.
        let mut writing = Writing::opening(&kept_at).unwrap();
        writing.keep(&answered_here()).unwrap();

        let again = recounting
            .about(&Asking::anything(), AtMost::ONE_SITTING)
            .unwrap();
        assert_eq!(again.how_many(), 2, "the answer came from memory");
        assert_eq!(
            again.told().last().map(Told::outcome),
            Some(Outcome::AnsweredHere)
        );

        // And the account somebody is already holding did not change under
        // them: it is one reading of one file, at one moment.
        assert_eq!(first.how_many(), 1);
    }

    /// **A record that is not there is refused, never answered as empty.** A
    /// machine that has done nothing and a machine whose record was deleted are
    /// indistinguishable from a screen, and only one of them is innocent.
    #[test]
    fn a_record_that_is_not_there_is_refused_rather_than_answered_as_nothing() {
        let kept_at = somewhere_of_our_own("missing");
        let recounting = Recounting::kept_at(&kept_at);

        let Err(why) = recounting.about(&Asking::anything(), AtMost::ONE_SITTING) else {
            panic!("a machine with no record answered as though nothing had happened");
        };
        assert!(matches!(
            why,
            NotRecounted::Record(NotKept::NotThere { .. })
        ));
        assert!(why.there_is_no_record());
        assert!(why.said(&in_english()).text().contains("has done nothing"));

        // And a record that was written and then deleted is the same refusal,
        // which is the case this rule exists for.
        a_record_at(&kept_at, &[archived()]);
        assert!(
            recounting
                .about(&Asking::anything(), AtMost::ONE_SITTING)
                .is_ok()
        );
        std::fs::remove_file(&kept_at).unwrap();
        assert!(
            recounting
                .about(&Asking::anything(), AtMost::ONE_SITTING)
                .is_err_and(|why| why.there_is_no_record())
        );
    }

    /// A file that is not a record, and one from an alo OS this one does not
    /// know, are refused in the words of the crate that reads records — rather
    /// than being read by guessing.
    #[test]
    fn nothing_that_is_not_this_machines_record_is_read_as_one() {
        let notes = somewhere_of_our_own("notes");
        std::fs::write(&notes, "notes about the invoice\n").unwrap();
        let refused = Recounting::kept_at(&notes)
            .about(&Asking::anything(), AtMost::ONE_SITTING)
            .unwrap_err();
        assert!(matches!(
            refused,
            NotRecounted::Record(NotKept::NotARecord { .. })
        ));
        assert!(!refused.there_is_no_record());

        let newer = somewhere_of_our_own("newer");
        std::fs::write(&newer, "{\"format\":2}\n").unwrap();
        let refused = Recounting::kept_at(&newer)
            .about(&Asking::anything(), AtMost::ONE_SITTING)
            .unwrap_err();
        assert!(matches!(
            refused,
            NotRecounted::Record(NotKept::FromANewerAlo { format: 2, .. })
        ));
        assert!(!refused.said(&in_english()).is_a_bug());
    }

    /// **One question, one account, on the screen once.** The compositor is
    /// handed the whole account, and what it was handed is what came back.
    #[test]
    fn one_question_puts_one_account_in_front_of_the_person() {
        let kept_at = somewhere_of_our_own("shown");
        a_record_at(&kept_at, &an_afternoon());
        let recounting = Recounting::kept_at(&kept_at);
        let mut screen = Screen::default();

        let Recounts::Shown(account) =
            recounting.show(Some(&mut screen), &Asking::anything(), AtMost::ONE_SITTING)
        else {
            panic!("a record that was there was not put in front of anybody");
        };
        assert_eq!(screen.shown.len(), 1);
        assert_eq!(screen.shown.first(), Some(&account));
        assert_eq!(account.how_many(), an_afternoon().len());
    }

    /// **Nowhere to put it is a refusal, not silence**, and nothing is drawn on
    /// a screen that is not there.
    #[test]
    fn nowhere_to_put_it_refuses_in_words() {
        let kept_at = somewhere_of_our_own("nowhere");
        a_record_at(&kept_at, &[archived()]);
        let recounting = Recounting::kept_at(&kept_at);

        // Nothing is drawing a screen at all.
        assert_eq!(
            recounting.show(None, &Asking::anything(), AtMost::ONE_SITTING),
            Recounts::Refused(NotRecounted::NoCompositor)
        );

        // Something is drawing, and there is no display to draw on.
        let mut nowhere = NoScreen::default();
        assert_eq!(
            recounting.show(Some(&mut nowhere), &Asking::anything(), AtMost::ONE_SITTING),
            Recounts::Refused(NotRecounted::Surface(SurfaceRefused::NothingToShowOn))
        );
        assert_eq!(nowhere.offered, 1);

        let strings = in_english();
        for said in [
            NotRecounted::NoCompositor.said(&strings),
            NotRecounted::Surface(SurfaceRefused::NothingToShowOn).said(&strings),
        ] {
            assert!(!said.is_a_bug(), "{said}");
        }

        // Neither of them read anything to nobody: a screen that arrives can
        // still be shown the same record.
        let mut screen = Screen::default();
        assert!(matches!(
            recounting.show(Some(&mut screen), &Asking::anything(), AtMost::ONE_SITTING),
            Recounts::Shown(_)
        ));
    }

    /// **Nowhere to show it is answered before the disk is read**, so a machine
    /// with no screen does not read a year of evidence to find that out.
    #[test]
    fn with_nowhere_to_show_it_the_record_is_not_even_read() {
        let missing = somewhere_of_our_own("not-read");
        let recounting = Recounting::kept_at(&missing);
        // There is no record at all here, so a call that read the disk would
        // refuse with `NotThere` rather than with the missing compositor.
        assert_eq!(
            recounting.show(None, &Asking::anything(), AtMost::ONE_SITTING),
            Recounts::Refused(NotRecounted::NoCompositor)
        );
    }

    /// The question is the record's own, so narrowing it narrows the account
    /// and nothing else — and a question nobody narrowed is answered with
    /// everything rather than with nothing.
    #[test]
    fn the_question_put_to_the_surface_is_the_records_own() {
        let kept_at = somewhere_of_our_own("asked");
        a_record_at(&kept_at, &an_afternoon());
        let recounting = Recounting::kept_at(&kept_at);

        let everything = recounting
            .about(&Asking::anything(), AtMost::ONE_SITTING)
            .unwrap();
        let refusals = recounting
            .about(
                &Asking::anything().only(Only::Refusals),
                AtMost::ONE_SITTING,
            )
            .unwrap();
        let left = recounting
            .about(&Asking::anything().only(Only::Egress), AtMost::ONE_SITTING)
            .unwrap();

        assert_eq!(everything.how_many(), an_afternoon().len());
        assert!(refusals.how_many() < everything.how_many());
        assert!(!refusals.is_empty());
        assert_eq!(
            left.how_many(),
            2,
            "an agent's departure, and the machine's"
        );
        assert_eq!(
            refusals.how_many_in_the_record(),
            everything.how_many_in_the_record(),
            "a question changed what the record holds"
        );
    }
}
