//! *Which application opens this kind of file* — answered from the person's
//! choice, or the application's own declaration, and never from a guess.
//!
//! `docs/features.md`, v0.5: *file associations — what opens what, changeable
//! by a person.* [`WhatOpensWhat`] is that answer, and the open-with portal
//! (`alo_portals::open_with`) is answered from it and from nothing else.
//!
//! # The kind is the file's bytes
//!
//! A file's kind is read with [`alo_opening::decide`], from its own bytes and
//! never from its name: the name is not even handed over. A shell script named
//! `invoice.pdf` is a program, and a program is opened by nothing
//! ([`NothingOpens::TheFile`]); a scan saved as `.txt` is a JPEG image, and
//! opens where JPEG images do.
//!
//! # In this order
//!
//! 1. **What the person chose for the kind**, when that application is
//!    installed. It wins over every declaration.
//! 2. **The first installed application that declares the kind**
//!    ([`crate::Declared`] on why the first), saying — when the person's choice
//!    was passed over — that what they chose is not installed.
//! 3. **Nothing**, as a sentence ([`NothingOpens::NoApplication`]). There is no
//!    fallback application.
//!
//! # What it does not do
//!
//! **It launches nothing.** The answer is a name and a reason. **It changes
//! nothing**: it borrows the person's choices and the declarations to read, so
//! answering a request can never be how an association gets set.

use std::ffi::OsStr;
use std::io::{Read, Seek};

use alo_opening::{Cannot, Decided, Kind, Outcome, ThisMachine};

use crate::chosen::Chosen;
use crate::declared::Declared;
use crate::installed::Installed;
use crate::nothing_opens::NothingOpens;
use crate::opener::{Because, Opener};

/// What this machine has, what its applications declare, and what the person
/// chose — the three things *what opens this* is answered from.
#[derive(Debug, Clone, Copy)]
pub struct WhatOpensWhat<'a> {
    /// What is installed, which every answer has to be.
    installed: &'a Installed,
    /// What the applications say they open.
    declared: &'a Declared,
    /// What the person chose.
    chosen: &'a Chosen,
}

impl<'a> WhatOpensWhat<'a> {
    /// The answer on a machine with these applications, these declarations and
    /// these choices.
    #[must_use]
    pub const fn on(installed: &'a Installed, declared: &'a Declared, chosen: &'a Chosen) -> Self {
        Self {
            installed,
            declared,
            chosen,
        }
    }

    /// What opens this file, read from its own bytes.
    ///
    /// The file is handed over open, because which file may be read is a
    /// grant and the caller holds it; its position afterwards is wherever
    /// reading left it.
    ///
    /// # Errors
    ///
    /// [`NothingOpens`]: a kind nothing installed opens, a file that is no kind
    /// anything opens, or a file that would not be read.
    pub fn what_opens<F: Read + Seek>(&self, file: &mut F) -> Result<Opener, NothingOpens> {
        let kind = kind_of(file)?;
        self.what_opens_a(kind)
    }

    /// What opens a file of this kind.
    ///
    /// # Errors
    ///
    /// [`NothingOpens::NoApplication`] when the person chose nothing installed
    /// for the kind and no installed application declares it.
    pub fn what_opens_a(&self, kind: Kind) -> Result<Opener, NothingOpens> {
        let chosen = self.chosen.for_kind(kind);
        if let Some(application) = chosen.and_then(|chosen| self.installed.knows(chosen)) {
            return Ok(Opener::new(
                application.clone(),
                kind,
                Because::ThePersonChoseIt,
            ));
        }
        let declared = self
            .declared
            .declaring(kind)
            .find_map(|declaring| self.installed.knows(declaring));
        match (declared, chosen) {
            (Some(application), None) => Ok(Opener::new(
                application.clone(),
                kind,
                Because::ItDeclaresIt,
            )),
            (Some(application), Some(chosen)) => Ok(Opener::new(
                application.clone(),
                kind,
                Because::ItDeclaresItAndTheChoiceIsNotInstalled {
                    chosen: chosen.to_owned(),
                },
            )),
            (None, chosen) => Err(NothingOpens::NoApplication {
                kind,
                chosen: chosen.map(str::to_owned),
            }),
        }
    }
}

/// The kind of this file from its bytes, or why it is no kind anything opens.
///
/// Asked of a machine that opens nothing, so every file comes back as what it
/// is and never as what could be done with it here — which is this crate's
/// question, not `alo-opening`'s. No name is handed over, so no name can decide.
fn kind_of<F: Read + Seek>(file: &mut F) -> Result<Kind, NothingOpens> {
    let decided = alo_opening::decide(file, OsStr::new(""), &ThisMachine::with_nothing())
        .map_err(NothingOpens::Unreadable)?;
    let outcome = match decided {
        Decided::AsItIs(outcome) | Decided::NotWhatItsNameSays { outcome, .. } => outcome,
    };
    match outcome {
        Outcome::CannotOpen(Cannot::NothingHereOpens(kind))
        | Outcome::OpensAsItIs { kind, .. }
        | Outcome::Converts { from: kind, .. } => Ok(kind),
        Outcome::CannotOpen(cannot) => Err(NothingOpens::TheFile(cannot)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::application::Application;
    use std::io::Cursor;

    fn papers() -> Application {
        Application::called("org.gnome.Papers", "Papers").unwrap()
    }

    fn okular() -> Application {
        Application::identified("org.kde.okular").unwrap()
    }

    /// **The person's choice, then the first installed declaration, then
    /// nothing** — and a choice or declaration naming what is not installed is
    /// passed over and said.
    #[test]
    fn the_answer_is_taken_in_order() {
        let installed = Installed::holding([papers()]);
        let mut declared = Declared::nothing();
        declared.declares(&okular(), [Kind::Pdf]);
        declared.declares(&papers(), [Kind::Pdf]);

        // Okular declared first and is not installed: Papers answers.
        let nothing_chosen = Chosen::untouched();
        let opener = WhatOpensWhat::on(&installed, &declared, &nothing_chosen)
            .what_opens_a(Kind::Pdf)
            .unwrap();
        assert_eq!(opener.application(), &papers());
        assert_eq!(opener.because(), &Because::ItDeclaresIt);

        // The person chose Okular, which is not installed: said, and Papers.
        let mut chosen = Chosen::untouched();
        chosen.choose(Kind::Pdf, &okular());
        let opener = WhatOpensWhat::on(&installed, &declared, &chosen)
            .what_opens_a(Kind::Pdf)
            .unwrap();
        assert_eq!(
            opener.because(),
            &Because::ItDeclaresItAndTheChoiceIsNotInstalled {
                chosen: "org.kde.okular".to_owned()
            }
        );

        // And for a kind nobody declares, nothing — naming the choice.
        chosen.choose(Kind::WebpImage, &okular());
        assert_eq!(
            WhatOpensWhat::on(&installed, &declared, &chosen).what_opens_a(Kind::WebpImage),
            Err(NothingOpens::NoApplication {
                kind: Kind::WebpImage,
                chosen: Some("org.kde.okular".to_owned())
            })
        );
    }

    /// **What is not a kind anything opens is `alo-opening`'s finding**, carried
    /// whole.
    #[test]
    fn a_file_that_is_no_kind_is_the_finding_about_it() {
        let installed = Installed::holding([papers()]);
        let declared = Declared::nothing();
        let chosen = Chosen::untouched();
        let asking = WhatOpensWhat::on(&installed, &declared, &chosen);
        for (bytes, cannot) in [
            (&b"\x7fELF\x02\x01\x01\0"[..], Cannot::AProgram),
            (&b""[..], Cannot::Empty),
        ] {
            assert_eq!(
                asking.what_opens(&mut Cursor::new(bytes.to_vec())),
                Err(NothingOpens::TheFile(cannot))
            );
        }
    }
}
