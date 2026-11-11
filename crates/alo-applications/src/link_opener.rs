//! *Which application opens this link* — the person's default, or the first
//! installed application that declares the kind, and never a guess.
//!
//! The owner's decision of 2026-11-12, given in a question dialog: *"No — open
//! in my default, but record it (Recommended)"*. A web link opens in the
//! person's default browser and a `mailto:` link in their default mail
//! application, **with no grant over either**: following a link is the
//! person's own act. Until the design for choosing a default arrives, the
//! default is what the person chose in `what-opens-what.toml` where they chose
//! one, and otherwise the first installed application that declares the kind
//! (`x-scheme-handler/http`, `…/https`, `…/mailto` in its desktop entry).
//!
//! # In this order
//!
//! 1. **What the person chose for the kind**, when it is installed and is not a
//!    terminal.
//! 2. **The first installed application that declares the kind** and is not a
//!    terminal, saying — when the person's choice was passed over — why.
//! 3. **Nothing**, as a sentence that names which kind of link has no
//!    application ([`NothingOpensLinks`]). There is no fallback application.
//!
//! # A link is never handed to a terminal
//!
//! A terminal is a person's own (`alo_capability::A_PERSONS_OWN`, ADR 0043)
//! and whatever is handed to one may run. A link is written outside this
//! machine — by a web page, by whoever sent the message it was in — so a
//! terminal is passed over even where the person chose it, and the reason
//! says so. This is the reasoning `alo-software`'s `browsing` already applies
//! to its own answer, carried to the one that is now used.
//!
//! # It launches nothing
//!
//! The answer is a name and a reason, as [`crate::Opener`] is. Starting the
//! application is the desktop's, through its door.

use alo_capability::is_a_persons_own;
use alo_strings::{Filling, Said, Strings};

use crate::application::Application;
use crate::link::LinkKind;
use crate::what_opens::WhatOpensWhat;
use crate::words;

/// Why this application is the one that opens the kind of link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkBecause {
    /// The person chose it.
    ThePersonChoseIt,
    /// It is the first installed application declaring the kind, and the
    /// person chose nothing.
    ItDeclaresIt,
    /// It declares the kind, and what the person chose is not installed.
    ItDeclaresItAndTheChoiceIsNotInstalled {
        /// The identifier the person chose.
        chosen: String,
    },
    /// It declares the kind, and what the person chose is a terminal.
    ItDeclaresItAndTheChoiceIsATerminal {
        /// The identifier the person chose.
        chosen: String,
    },
}

/// The application that opens a kind of link, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkOpener {
    /// The application, as this machine has it.
    application: Application,
    /// The kind of link.
    kind: LinkKind,
    /// Why it is the one.
    because: LinkBecause,
}

impl LinkOpener {
    /// The application that opens it.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// The kind of link.
    #[must_use]
    pub const fn kind(&self) -> LinkKind {
        self.kind
    }

    /// Why this application is the one.
    #[must_use]
    pub const fn because(&self) -> &LinkBecause {
        &self.because
    }

    /// Whether this was the person's own choice.
    #[must_use]
    pub const fn was_chosen(&self) -> bool {
        matches!(self.because, LinkBecause::ThePersonChoseIt)
    }
}

/// What the person chose, where it is part of why nothing opens a link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassedOver {
    /// It is not installed.
    NotInstalled(String),
    /// It is a terminal, which no link is handed to.
    ATerminal(String),
}

impl PassedOver {
    /// The identifier the person chose.
    #[must_use]
    pub fn chosen(&self) -> &str {
        match self {
            Self::NotInstalled(chosen) | Self::ATerminal(chosen) => chosen,
        }
    }
}

/// Why no application opens a kind of link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NothingOpensLinks {
    /// The kind of link.
    kind: LinkKind,
    /// What the person chose, where that was passed over.
    passed_over: Option<PassedOver>,
}

impl NothingOpensLinks {
    /// The kind of link nothing opens.
    #[must_use]
    pub const fn kind(&self) -> LinkKind {
        self.kind
    }

    /// What the person chose, where it was passed over.
    #[must_use]
    pub const fn passed_over(&self) -> Option<&PassedOver> {
        self.passed_over.as_ref()
    }

    /// What this says, in the language the person reads — naming which kind
    /// of link has no application.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = match (self.kind, &self.passed_over) {
            (LinkKind::Web, None) => words::NOTHING_OPENS_WEB_LINKS,
            (LinkKind::Mail, None) => words::NOTHING_OPENS_MAIL_LINKS,
            (LinkKind::Web, Some(PassedOver::NotInstalled(_))) => {
                words::NOTHING_OPENS_WEB_LINKS_CHOICE_GONE
            }
            (LinkKind::Mail, Some(PassedOver::NotInstalled(_))) => {
                words::NOTHING_OPENS_MAIL_LINKS_CHOICE_GONE
            }
            (LinkKind::Web, Some(PassedOver::ATerminal(_))) => {
                words::NOTHING_OPENS_WEB_LINKS_CHOICE_A_TERMINAL
            }
            (LinkKind::Mail, Some(PassedOver::ATerminal(_))) => {
                words::NOTHING_OPENS_MAIL_LINKS_CHOICE_A_TERMINAL
            }
        };
        let filling = match &self.passed_over {
            None => Filling::nothing(),
            Some(passed) => Filling::of("chosen", passed.chosen().to_owned()),
        };
        strings.say(&word.key(), &filling)
    }
}

impl WhatOpensWhat<'_> {
    /// What opens a link of this kind.
    ///
    /// # Errors
    ///
    /// [`NothingOpensLinks`] when the person chose nothing that can open it and
    /// no installed application other than a terminal declares it.
    pub fn what_opens_links(&self, kind: LinkKind) -> Result<LinkOpener, NothingOpensLinks> {
        let chosen = self.chosen().for_links(kind);
        let passed_over = match chosen {
            Some(chosen) if is_a_persons_own(chosen) => {
                Some(PassedOver::ATerminal(chosen.to_owned()))
            }
            Some(chosen) => match self.installed().knows(chosen) {
                Some(application) => {
                    return Ok(LinkOpener {
                        application: application.clone(),
                        kind,
                        because: LinkBecause::ThePersonChoseIt,
                    });
                }
                None => Some(PassedOver::NotInstalled(chosen.to_owned())),
            },
            None => None,
        };
        let declared = self
            .declared()
            .declaring_links(kind)
            .filter(|declaring| !is_a_persons_own(declaring))
            .find_map(|declaring| self.installed().knows(declaring));
        let Some(application) = declared else {
            return Err(NothingOpensLinks { kind, passed_over });
        };
        let because = match passed_over {
            None => LinkBecause::ItDeclaresIt,
            Some(PassedOver::NotInstalled(chosen)) => {
                LinkBecause::ItDeclaresItAndTheChoiceIsNotInstalled { chosen }
            }
            Some(PassedOver::ATerminal(chosen)) => {
                LinkBecause::ItDeclaresItAndTheChoiceIsATerminal { chosen }
            }
        };
        Ok(LinkOpener {
            application: application.clone(),
            kind,
            because,
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::chosen::Chosen;
    use crate::declared::Declared;
    use crate::installed::Installed;
    use crate::testing::in_english;

    const FIREFOX: &str = "org.mozilla.firefox";
    const LIBREWOLF: &str = "io.gitlab.librewolf-community";
    const THUNDERBIRD: &str = "org.mozilla.Thunderbird";
    const PTYXIS: &str = "app.devsuite.Ptyxis";

    fn called(identifier: &str) -> Application {
        Application::identified(identifier).unwrap()
    }

    /// Firefox and Librewolf declaring web links, Thunderbird mail, and a
    /// terminal declaring web links too, all installed.
    fn a_machine() -> (Installed, Declared) {
        let installed = Installed::holding([
            called(FIREFOX),
            called(LIBREWOLF),
            called(THUNDERBIRD),
            called(PTYXIS),
        ]);
        let mut declared = Declared::nothing();
        declared.declares_media_types(&called(PTYXIS), ["x-scheme-handler/http"]);
        declared.declares_media_types(
            &called(FIREFOX),
            ["x-scheme-handler/http", "x-scheme-handler/https"],
        );
        declared.declares_media_types(&called(LIBREWOLF), ["x-scheme-handler/https"]);
        declared.declares_media_types(&called(THUNDERBIRD), ["x-scheme-handler/mailto"]);
        (installed, declared)
    }

    /// **The person's default wins; with none, the first installed declaring
    /// the kind answers** — and a terminal is never it, even declared first.
    #[test]
    fn the_default_is_the_persons_choice_and_then_the_first_declared() {
        let (installed, declared) = a_machine();
        let nothing = Chosen::untouched();
        let asking = WhatOpensWhat::on(&installed, &declared, &nothing);
        let web = asking.what_opens_links(LinkKind::Web).unwrap();
        assert_eq!(web.application().identifier(), FIREFOX);
        assert_eq!(web.because(), &LinkBecause::ItDeclaresIt);
        let mail = asking.what_opens_links(LinkKind::Mail).unwrap();
        assert_eq!(mail.application().identifier(), THUNDERBIRD);

        let mut chosen = Chosen::untouched();
        chosen.choose_for_links(LinkKind::Web, &called(LIBREWOLF));
        let web = WhatOpensWhat::on(&installed, &declared, &chosen)
            .what_opens_links(LinkKind::Web)
            .unwrap();
        assert_eq!(web.application().identifier(), LIBREWOLF);
        assert!(web.was_chosen());
    }

    /// **A choice that is gone, or that is a terminal, is passed over and
    /// said.**
    #[test]
    fn a_choice_that_cannot_open_links_is_passed_over_and_said() {
        let (installed, declared) = a_machine();
        let mut chosen = Chosen::untouched();
        chosen.choose_for_links(LinkKind::Web, &called("com.example.Gone"));
        chosen.choose_for_links(LinkKind::Mail, &called(PTYXIS));
        let asking = WhatOpensWhat::on(&installed, &declared, &chosen);
        assert_eq!(
            asking.what_opens_links(LinkKind::Web).unwrap().because(),
            &LinkBecause::ItDeclaresItAndTheChoiceIsNotInstalled {
                chosen: "com.example.Gone".to_owned()
            }
        );
        assert_eq!(
            asking.what_opens_links(LinkKind::Mail).unwrap().because(),
            &LinkBecause::ItDeclaresItAndTheChoiceIsATerminal {
                chosen: PTYXIS.to_owned()
            }
        );
    }

    /// **With no mail application, a `mailto:` link opens in nothing**, and
    /// the sentence names the kind — and a different one names a choice that
    /// is gone or is a terminal.
    #[test]
    fn nothing_opens_a_kind_with_no_application_and_says_which() {
        let installed = Installed::holding([called(FIREFOX), called(PTYXIS)]);
        let mut declared = Declared::nothing();
        declared.declares_media_types(&called(FIREFOX), ["x-scheme-handler/https"]);
        declared.declares_media_types(&called(PTYXIS), ["x-scheme-handler/mailto"]);
        let strings = in_english();
        let mut seen = Vec::new();
        for (choice, passed) in [
            (None, None),
            (
                Some(THUNDERBIRD),
                Some(PassedOver::NotInstalled(THUNDERBIRD.to_owned())),
            ),
            (Some(PTYXIS), Some(PassedOver::ATerminal(PTYXIS.to_owned()))),
        ] {
            let mut chosen = Chosen::untouched();
            if let Some(choice) = choice {
                chosen.choose_for_links(LinkKind::Mail, &called(choice));
            }
            let nothing = WhatOpensWhat::on(&installed, &declared, &chosen)
                .what_opens_links(LinkKind::Mail)
                .unwrap_err();
            assert_eq!(nothing.kind(), LinkKind::Mail);
            assert_eq!(nothing.passed_over(), passed.as_ref());
            let said = nothing.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
            assert!(said.text().contains("email"), "{said}");
            if let Some(choice) = choice {
                assert!(said.text().contains(choice), "{said}");
            }
            seen.push(said.text().to_owned());
        }
        let web = WhatOpensWhat::on(&Installed::nothing(), &declared, &Chosen::untouched())
            .what_opens_links(LinkKind::Web)
            .unwrap_err()
            .said(&strings);
        assert!(web.text().contains("web browser"), "{web}");
        seen.push(web.text().to_owned());
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 4, "two refusals said the same thing");
    }
}
