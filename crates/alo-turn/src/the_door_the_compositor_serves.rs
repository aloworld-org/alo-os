//! The one thing a turn needs from the compositor, as a trait.
//!
//! [ADR 0108](../../../docs/decisions/0108-the-compositor-carries-an-application-verb-out-and-needs-no-token-to.md):
//! the capability model is here and the windows are in the compositor, and
//! between them is one socket. `docs/contracts/application-door.md` is that
//! socket's published surface and `alo-acting` is its vocabulary.
//!
//! # Why a trait and not a socket
//!
//! This crate is portable and holds no path. The socket is
//! `/run/alo/<uid>/shell.sock` on a Linux machine with a compositor running,
//! and `alo-agentd` is the thing that knows both of those — which is the same
//! division `Machine::installing_with` already makes for the rented tool, and
//! `alo_shell::an_application_is_started`'s `AsksAnApplicationToStart` makes
//! from the other side.
//!
//! It also makes the executor testable without a compositor, which is the
//! whole of what a unit test can hold here.
//!
//! # A door that is not there is not an error in the machine
//!
//! A person may be signed in with no compositor running — a machine at the
//! sign-in screen, a daemon started before the shell, a shell that crashed. The
//! verbs are still declared and a turn may still reach this, so *there is no
//! door* has to be an answer rather than a panic or a hang.
//!
//! It answers [`NotAtTheDoor::NoCompositor`], which the executor turns into the
//! person's own *this machine could not reach {application}* — the same
//! sentence as a compositor that answered `could-not-ask`, because from the
//! person's side they are the same fact: nothing was changed.

use alo_acting::{
    Answer, AnsweredTheChooser, Opened, Pictured, Request, ToAnswerAFileChooser, ToOpenAFile,
    ToPicture, ToType, ToWorkIn, Typed, Worked, Working,
};

/// Whatever can put one request to the compositor and read one answer.
///
/// One line in, one line out, and **nothing about a session**: ADR 0108 is
/// explicit that one line is one act, because nothing about this door is an
/// authorisation in the first place — the authority was the turn's and it was
/// spent before the line was written.
pub trait TheDoorTheCompositorServes {
    /// Put this to the compositor and wait for its one-word answer.
    ///
    /// # Errors
    /// [`NotAtTheDoor`] when there was nothing to ask or the asking failed.
    /// **Never a refusal of the capability model** — *may this run* was decided
    /// before this was called, and a door that could answer it would be a
    /// second place deciding it.
    fn asking(&mut self, request: &Request) -> Result<Answer, NotAtTheDoor>;

    /// Put one typing to the compositor — `type-into` with its text,
    /// `save-in`, or `may-type-into` (ADR 0153) — and wait for its answer.
    /// `text` is sent only for [`ToType::Text`].
    ///
    /// **No compositor by default**, so a door that does not know typing types
    /// nothing.
    ///
    /// # Errors
    /// [`NotAtTheDoor`], as for [`Self::asking`].
    fn typing(&mut self, asked: &ToType, text: &str) -> Result<Typed, NotAtTheDoor> {
        let _ = (asked, text);
        Err(NotAtTheDoor::NoCompositor)
    }

    /// Put `open-file` to the compositor — open this file in this application
    /// (`open_file_in_application`, 2026-11-14) — and wait for its answer,
    /// which says where it opened or why it did not.
    ///
    /// **No compositor by default**, as for [`Self::typing`], so a door that
    /// does not know this act opens nothing.
    ///
    /// # Errors
    /// [`NotAtTheDoor`], as for [`Self::asking`].
    fn opening_a_file(&mut self, asked: &ToOpenAFile) -> Result<Opened, NotAtTheDoor> {
        let _ = asked;
        Err(NotAtTheDoor::NoCompositor)
    }

    /// Put `alo-chooses-file` to the compositor — answer the file chooser
    /// that is up with this file (`choose_on_the_file_chooser`, 2026-11-17) —
    /// and wait for its answer: whom it was chosen for, no sheet, or a file
    /// the sheet could not have been answered with.
    ///
    /// **No compositor by default**, as for [`Self::typing`], so a door that
    /// does not know this act answers nothing.
    ///
    /// # Errors
    /// [`NotAtTheDoor`], as for [`Self::asking`].
    fn answering_the_file_chooser(
        &mut self,
        asked: &ToAnswerAFileChooser,
    ) -> Result<AnsweredTheChooser, NotAtTheDoor> {
        let _ = asked;
        Err(NotAtTheDoor::NoCompositor)
    }

    /// Put `picture-of` to the compositor — one picture of the window in
    /// front, of that window alone (ADR 0147 §1) — and wait for its answer and,
    /// for a picture, its PNG's bytes. Every refusal carries no bytes.
    ///
    /// **No compositor by default**, as for [`Self::typing`], so a door that
    /// does not know this act takes no picture.
    ///
    /// # Errors
    /// [`NotAtTheDoor`], as for [`Self::asking`] — and for a picture cut short
    /// of the length its head said, which is never a smaller picture.
    fn picturing(&mut self, asked: &ToPicture) -> Result<(Pictured, Vec<u8>), NotAtTheDoor> {
        let _ = asked;
        Err(NotAtTheDoor::NoCompositor)
    }

    /// Put `click-at` to the compositor — one click at one point of the
    /// window in front, given as thousandths of it (ADR 0147, amended
    /// 2026-11-17) — and wait for its one word.
    ///
    /// **No compositor by default**, as for [`Self::typing`], so a door that
    /// does not know this act clicks nothing.
    ///
    /// # Errors
    /// [`NotAtTheDoor`], as for [`Self::asking`].
    /// Begin `work-in-their-window` or `work-in-alos-copy` on the door
    /// (`alo_acting::working_in`) and read its first answer. **The one act
    /// on this door that holds its connection**, for as long as alo works,
    /// because the line it puts up is up for exactly that long; an adapter's
    /// verb holds it while its one message goes
    /// (`crate::acting_through_an_adapter`).
    ///
    /// **No compositor by default**, as for [`Self::typing`], so a door that
    /// does not know this act lets alo work nowhere.
    ///
    /// # Errors
    /// [`NotAtTheDoor`], as for [`Self::asking`].
    fn working_in(&mut self, asked: &ToWorkIn) -> Result<Box<dyn WhileAloWorks>, NotAtTheDoor> {
        let _ = asked;
        Err(NotAtTheDoor::NoCompositor)
    }

    fn clicking(
        &mut self,
        asked: &alo_acting::ToClick,
    ) -> Result<alo_acting::Clicked, NotAtTheDoor> {
        let _ = asked;
        Err(NotAtTheDoor::NoCompositor)
    }
}

/// **alo at work in one application**, as the door answered it: the first
/// answer, and `done` when the work is over.
pub trait WhileAloWorks {
    /// What the door answered when the work was asked for.
    fn answered(&self) -> Working;

    /// The work is over: say `done` and read what was sent while it lasted.
    ///
    /// # Errors
    /// [`NotAtTheDoor`] when the door went away first.
    fn done(self: Box<Self>) -> Result<Worked, NotAtTheDoor>;
}

/// Why the compositor was not asked.
///
/// Every one of these means **nothing was changed**, which is what the person
/// is told. They are told apart here rather than in the sentence because only
/// one of them is worth somebody reading a log about.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAtTheDoor {
    /// No compositor is serving a door on this machine.
    ///
    /// Ordinary rather than exceptional: a machine at the sign-in screen is in
    /// exactly this state.
    #[error("no compositor is serving an application door on this machine")]
    NoCompositor,
    /// The door was there and the asking did not finish.
    ///
    /// The socket went away mid-conversation, the compositor stopped, or it
    /// answered something this machine's vocabulary does not know — which
    /// `alo_acting::Answer::read` deliberately never reads as success.
    #[error("the compositor was asked and did not answer: {why}")]
    NotAnswered {
        /// What went wrong, for a log and never for a person.
        why: String,
    },
}
