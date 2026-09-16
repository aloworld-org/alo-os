//! Which application opens a web address, answered from what is installed and
//! what the person chose — never from a guess.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 3: *the browser from
//! task 2 opens web addresses from any application.* `alo_applications::WhatOpensWhat`
//! answers *which application opens this file* from the file's own bytes; a web
//! address has no bytes and is no kind, so this is the same question asked of the
//! one thing that is not a file, and it is asked here because the browser that
//! answers it is [`crate::Shipped`]'s.
//!
//! # In this order, and there is no fallback
//!
//! 1. **What the person chose**, when that application is installed and is not
//!    one of their own (`alo_capability::A_PERSONS_OWN` — see below). A person's choice
//!    wins over what this machine ships.
//! 2. **The application this machine ships for [`Role::WebBrowser`]**, when it
//!    is installed — and saying, where the person's choice was passed over, why
//!    it was ([`Because`]).
//! 3. **Nothing**, as a sentence ([`NothingOpensThem`]). A person may remove the
//!    browser, and a machine with no browser opens web addresses in no
//!    application rather than in whatever happens to be installed.
//!
//! # A web address is never handed to a person's own application
//!
//! A terminal is a person's and never an agent's (ADR 0043), and whatever is
//! typed into one runs. A web address arrives from any application and is
//! written by whoever wrote the page it came from, so *the application the
//! person chose to open web addresses in* is refused when it is one of those:
//! the shipped browser answers instead, and the reason says so. This is not the
//! ADR's refusal — that one is about agents — it is the same reasoning about a
//! value nobody on this machine wrote.
//!
//! # It opens nothing
//!
//! The answer is a name and a reason, as `alo_applications::Opener` is. Asking
//! the application to open the address is the acting half, which needs a running
//! session and is not this crate's; [`crate::opening_the_web`] is where the
//! grants are asked first.

use alo_applications::{Application, Installed};
use alo_capability::is_a_persons_own;
use alo_strings::{Filling, Said, Strings};

use crate::role::Role;
use crate::shipped::Shipped;
use crate::words;

/// Why this application is the one that opens web addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Because {
    /// The person chose it.
    ThePersonChoseIt,
    /// It is the one this machine ships, and the person chose none.
    ThisMachineShipsIt,
    /// It is the one this machine ships, and what the person chose is not
    /// installed.
    ThisMachineShipsItAndTheChoiceIsNotInstalled {
        /// The identifier the person chose.
        chosen: String,
    },
    /// It is the one this machine ships, and what the person chose is one of
    /// their own applications, which no web address is handed to.
    ThisMachineShipsItAndTheChoiceIsAPersonsOwn {
        /// The identifier the person chose.
        chosen: String,
    },
}

impl Because {
    /// The string this crate declares for this reason.
    #[must_use]
    pub const fn word(&self) -> words::Word {
        match self {
            Self::ThePersonChoseIt => words::OPENS_THE_WEB_CHOSEN,
            Self::ThisMachineShipsIt => words::OPENS_THE_WEB_SHIPPED,
            Self::ThisMachineShipsItAndTheChoiceIsNotInstalled { .. } => {
                words::OPENS_THE_WEB_INSTEAD
            }
            Self::ThisMachineShipsItAndTheChoiceIsAPersonsOwn { .. } => {
                words::OPENS_THE_WEB_INSTEAD_OF_A_PERSONS_OWN
            }
        }
    }

    /// The identifier the person chose, where one was passed over.
    #[must_use]
    pub fn passed_over(&self) -> Option<&str> {
        match self {
            Self::ThePersonChoseIt | Self::ThisMachineShipsIt => None,
            Self::ThisMachineShipsItAndTheChoiceIsNotInstalled { chosen }
            | Self::ThisMachineShipsItAndTheChoiceIsAPersonsOwn { chosen } => Some(chosen),
        }
    }
}

/// The application that opens web addresses, and why it is the one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheBrowser {
    /// The application, as this machine has it.
    application: Application,
    /// Why it is the one.
    because: Because,
}

impl TheBrowser {
    /// The application that opens web addresses.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// Why it is the one.
    #[must_use]
    pub const fn because(&self) -> &Because {
        &self.because
    }

    /// Whether this was the person's own choice, rather than what this machine
    /// ships.
    #[must_use]
    pub const fn was_chosen(&self) -> bool {
        matches!(self.because, Because::ThePersonChoseIt)
    }

    /// What opens web addresses and why, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = Filling::of(words::APPLICATION, self.application.identifier().to_owned());
        let filling = match self.because.passed_over() {
            None => filling,
            Some(chosen) => filling.and(words::CHOSEN, chosen.to_owned()),
        };
        strings.say(&self.because.word().key(), &filling)
    }
}

/// Why no application on this machine opens web addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NothingOpensThem {
    /// What the person chose, when they chose something that cannot open them.
    chosen: Option<String>,
}

impl NothingOpensThem {
    /// What the person chose, where that is part of why.
    #[must_use]
    pub fn chosen(&self) -> Option<&str> {
        self.chosen.as_deref()
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> words::Word {
        match self.chosen {
            None => words::NOTHING_OPENS_THE_WEB,
            Some(_) => words::NOTHING_OPENS_THE_WEB_CHOICE_GONE,
        }
    }

    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match &self.chosen {
            None => strings.say(&self.word().key(), &Filling::nothing()),
            Some(chosen) => strings.say(
                &self.word().key(),
                &Filling::of(words::CHOSEN, chosen.clone()),
            ),
        }
    }
}

/// What this machine has, what it ships and what the person chose — the three
/// things *what opens a web address* is answered from.
#[derive(Debug, Clone, Copy)]
pub struct WhatOpensWebAddresses<'a> {
    /// What is installed, which every answer has to be.
    installed: &'a Installed,
    /// What this machine ships, whose [`Role::WebBrowser`] is the answer when
    /// the person chose nothing.
    shipped: &'a Shipped,
    /// The identifier the person chose to open web addresses in, where they
    /// chose one.
    chosen: Option<&'a str>,
}

impl<'a> WhatOpensWebAddresses<'a> {
    /// The answer on a machine with these applications and this choice.
    #[must_use]
    pub const fn on(
        installed: &'a Installed,
        shipped: &'a Shipped,
        chosen: Option<&'a str>,
    ) -> Self {
        Self {
            installed,
            shipped,
            chosen,
        }
    }

    /// Which application opens web addresses on this machine, and why.
    ///
    /// # Errors
    /// [`NothingOpensThem`] when neither the person's choice nor the
    /// application this machine ships for the web is installed.
    pub fn what_opens_them(&self) -> Result<TheBrowser, NothingOpensThem> {
        let passed_over = match self.chosen {
            None => None,
            Some(chosen) if is_a_persons_own(chosen) => {
                Some(Because::ThisMachineShipsItAndTheChoiceIsAPersonsOwn {
                    chosen: chosen.to_owned(),
                })
            }
            Some(chosen) => match self.installed.knows(chosen) {
                Some(application) => {
                    return Ok(TheBrowser {
                        application: application.clone(),
                        because: Because::ThePersonChoseIt,
                    });
                }
                None => Some(Because::ThisMachineShipsItAndTheChoiceIsNotInstalled {
                    chosen: chosen.to_owned(),
                }),
            },
        };
        let ships = self
            .shipped
            .the(Role::WebBrowser)
            .application()
            .identifier();
        match self.installed.knows(ships) {
            Some(application) => Ok(TheBrowser {
                application: application.clone(),
                because: passed_over.unwrap_or(Because::ThisMachineShipsIt),
            }),
            None => Err(NothingOpensThem {
                chosen: self.chosen.map(str::to_owned),
            }),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// The application this machine ships for the web, installed.
    fn firefox() -> Application {
        Application::called("org.mozilla.firefox", "Firefox").unwrap()
    }

    fn shipped() -> Shipped {
        Shipped::decided().unwrap()
    }

    /// **The person's choice wins, and what this machine ships answers when
    /// they chose nothing.**
    #[test]
    fn the_persons_choice_wins_and_the_shipped_browser_answers_otherwise() {
        let epiphany = Application::identified("org.gnome.Epiphany").unwrap();
        let installed = Installed::holding([firefox(), epiphany.clone()]);
        let shipped = shipped();

        let chose = WhatOpensWebAddresses::on(&installed, &shipped, Some(epiphany.identifier()))
            .what_opens_them()
            .unwrap();
        assert_eq!(chose.application(), &epiphany);
        assert_eq!(chose.because(), &Because::ThePersonChoseIt);
        assert!(chose.was_chosen());

        let ships = WhatOpensWebAddresses::on(&installed, &shipped, None)
            .what_opens_them()
            .unwrap();
        assert_eq!(ships.application(), &firefox());
        assert_eq!(ships.because(), &Because::ThisMachineShipsIt);
        assert!(!ships.was_chosen());
    }

    /// **A choice that is not installed is said, not swallowed**, and the
    /// shipped browser answers instead.
    #[test]
    fn a_choice_that_is_not_installed_is_said() {
        let installed = Installed::holding([firefox()]);
        let shipped = shipped();
        let answer = WhatOpensWebAddresses::on(&installed, &shipped, Some("org.gnome.Epiphany"))
            .what_opens_them()
            .unwrap();
        assert_eq!(answer.application(), &firefox());
        assert_eq!(
            answer.because(),
            &Because::ThisMachineShipsItAndTheChoiceIsNotInstalled {
                chosen: "org.gnome.Epiphany".to_owned()
            }
        );
        assert_eq!(answer.because().passed_over(), Some("org.gnome.Epiphany"));
    }

    /// **A person's own application is never what a web address is opened in**,
    /// even when it is installed and even when they chose it — the shipped
    /// browser answers, and the reason says which choice was passed over.
    #[test]
    fn a_persons_own_application_is_never_handed_a_web_address() {
        let terminal = Application::identified("app.devsuite.Ptyxis").unwrap();
        assert!(is_a_persons_own(terminal.identifier()));
        let installed = Installed::holding([firefox(), terminal.clone()]);
        let shipped = shipped();
        let answer = WhatOpensWebAddresses::on(&installed, &shipped, Some(terminal.identifier()))
            .what_opens_them()
            .unwrap();
        assert_eq!(answer.application(), &firefox());
        assert_eq!(
            answer.because(),
            &Because::ThisMachineShipsItAndTheChoiceIsAPersonsOwn {
                chosen: terminal.identifier().to_owned()
            }
        );

        // And with no browser installed, a person's own application is not the
        // fallback either: nothing opens them.
        let only_terminal = Installed::holding([terminal.clone()]);
        assert_eq!(
            WhatOpensWebAddresses::on(&only_terminal, &shipped, Some(terminal.identifier()))
                .what_opens_them()
                .unwrap_err()
                .chosen(),
            Some(terminal.identifier())
        );
    }

    /// **A machine whose browser was removed opens web addresses in nothing**,
    /// and says so rather than falling back.
    #[test]
    fn a_machine_with_no_browser_opens_them_in_nothing() {
        let shipped = shipped();
        let editor = Application::identified("org.gnome.TextEditor").unwrap();
        let installed = Installed::holding([editor]);
        let nothing = WhatOpensWebAddresses::on(&installed, &shipped, None)
            .what_opens_them()
            .unwrap_err();
        assert_eq!(nothing.chosen(), None);
        assert_eq!(nothing.word(), words::NOTHING_OPENS_THE_WEB);

        let gone = WhatOpensWebAddresses::on(&installed, &shipped, Some("org.gnome.Epiphany"))
            .what_opens_them()
            .unwrap_err();
        assert_eq!(gone.chosen(), Some("org.gnome.Epiphany"));
        assert_eq!(gone.word(), words::NOTHING_OPENS_THE_WEB_CHOICE_GONE);
    }

    /// **Every reason and both refusals are sentences**, each naming the
    /// application by its identifier and the passed-over choice by its own.
    #[test]
    fn every_reason_and_refusal_is_said() {
        let strings = in_english();
        let mut seen = Vec::new();
        for because in [
            Because::ThePersonChoseIt,
            Because::ThisMachineShipsIt,
            Because::ThisMachineShipsItAndTheChoiceIsNotInstalled {
                chosen: "org.gnome.Epiphany".to_owned(),
            },
            Because::ThisMachineShipsItAndTheChoiceIsAPersonsOwn {
                chosen: "app.devsuite.Ptyxis".to_owned(),
            },
        ] {
            let browser = TheBrowser {
                application: firefox(),
                because: because.clone(),
            };
            let said = browser.said(&strings);
            assert!(!said.is_a_bug(), "{because:?}: {said}");
            assert!(said.unfilled().is_empty(), "{because:?}: {said}");
            assert!(said.text().contains("org.mozilla.firefox"), "{said}");
            if let Some(chosen) = because.passed_over() {
                assert!(said.text().contains(chosen), "{said}");
            }
            seen.push(said.into_text());
        }
        for nothing in [
            NothingOpensThem { chosen: None },
            NothingOpensThem {
                chosen: Some("org.gnome.Epiphany".to_owned()),
            },
        ] {
            let said = nothing.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
            seen.push(said.into_text());
        }
        seen.dedup();
        assert_eq!(seen.len(), 6, "two of these say the same thing");
    }
}
