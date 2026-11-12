//! An `OpenURI` request for a link, answered from *what opens what* and from
//! nothing else — and with **no grant**, by the owner's decision.
//!
//! The owner, 2026-11-12, in a question dialog: *"No — open in my default, but
//! record it (Recommended)"*. An `http` or `https` link opens in the person's
//! default browser and a `mailto:` link in their default mail application,
//! with no grant over either; every link opened is recorded, naming the
//! application that asked, the address, and the application that opened it.
//! Following a link is the person's own act, so it is not a question for the
//! grants the way an application reaching a file or the camera is.
//!
//! **What this withdraws, said plainly.** Until that day the decided order
//! (`alo-software`'s `opening_the_web`, now retired) asked the grants first so
//! that an application granted nothing could not learn, from how it was
//! answered, whether this machine has a browser and which. The owner withdrew
//! that protection for links: an application asking to open one now learns
//! whether something opened it. It never learns *which* application — the
//! response is `0` or `2` and `results` is empty — but it can tell a machine
//! with a browser from one without.
//!
//! # In this order
//!
//! 1. **The asking application is named** by its sandbox, as every portal
//!    request's is — a program outside one is refused before this is reached.
//! 2. **The link is a link** ([`alo_applications::Link`]): `http`, `https` or
//!    `mailto`, one line, not too long, no password in a web address. Refused
//!    otherwise, naming the scheme where there was one, and the address is not
//!    kept in the record.
//! 3. **What opens it**: the person's default for the kind, or the first
//!    installed application declaring the kind, never a terminal
//!    ([`alo_applications::WhatOpensWhat::what_opens_links`]).
//!
//! # `ask`
//!
//! An application may ask for the person to be shown a choice of
//! applications: the *Open with* sheet the owner approved on 2026-11-12. The
//! decision here is then a proposal — the default, which the sheet starts on —
//! and what opens the link is what the person chose
//! ([`LinkOpened::chosen_on_the_sheet`]), held to what was offered
//! ([`LinkOpened::offered`]). A link the person did not choose for is
//! [`crate::LinkNotChosen`], and nothing opens.
//!
//! # It opens nothing
//!
//! [`LinkOpened`] is the decision; the desktop starts the application
//! (`crate::opening_links`), and only after the record has kept it.

use alo_applications::{Application, Link, LinkOpener, NotALink, NothingOpensLinks, WhatOpensWhat};
use alo_capability::Applicant;
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// A link that is opened, what opens it, and who asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkOpened {
    /// The application that asked.
    from: Applicant,
    /// The link, checked.
    link: Link,
    /// What opens it, and why that application.
    opener: LinkOpener,
    /// Whether the application asked for the person to choose.
    asked_to_choose: bool,
    /// What the person chose on the *Open with* sheet, when they did.
    on_the_sheet: Option<Application>,
    /// What the sheet offers, the default first: every installed handler of
    /// the kind, as `alo_applications::WhatOpensWhat::what_could_open_links`
    /// answers. Empty unless the application asked to choose.
    offered: Vec<String>,
}

impl LinkOpened {
    /// The application that asked.
    #[must_use]
    pub const fn from(&self) -> &Applicant {
        &self.from
    }

    /// The link.
    #[must_use]
    pub const fn link(&self) -> &Link {
        &self.link
    }

    /// What opens it.
    #[must_use]
    pub const fn opener(&self) -> &LinkOpener {
        &self.opener
    }

    /// Whether the application asked for the person to choose what opens it.
    #[must_use]
    pub const fn asked_to_choose(&self) -> bool {
        self.asked_to_choose
    }

    /// What the *Open with* sheet offers, the default first — empty unless the
    /// application asked to choose.
    #[must_use]
    pub fn offered(&self) -> &[String] {
        &self.offered
    }

    /// What the person chose on the sheet, when they did.
    #[must_use]
    pub const fn on_the_sheet(&self) -> Option<&Application> {
        self.on_the_sheet.as_ref()
    }

    /// The identifier of the application the link is handed to: what the
    /// person chose on the sheet, or the default.
    #[must_use]
    pub fn opens_in(&self) -> &str {
        self.on_the_sheet.as_ref().map_or_else(
            || self.opener.application().identifier(),
            Application::identifier,
        )
    }

    /// The person chose `identifier` on the sheet — or [`None`] when it is not
    /// one the sheet offered, which is never believed.
    #[must_use]
    pub fn chosen_on_the_sheet(&self, identifier: &str) -> Option<Self> {
        if !self.offered.iter().any(|one| one == identifier) {
            return None;
        }
        let application = Application::identified(identifier).ok()?;
        Some(Self {
            on_the_sheet: Some(application),
            ..self.clone()
        })
    }

    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        said_as_opened(
            Filling::of("application", self.from.as_str()),
            self.link.as_str(),
            self.opens_in(),
            (self.asked_to_choose, self.on_the_sheet.is_some()),
            strings,
        )
    }
}

/// The sentence for a link handed to `opener`, from what was decided or from
/// what the record kept — one function, so the two cannot be worded apart.
/// `asked` fills `{application}` with who asked; `chose` is whether the
/// application asked to choose, and whether the person then chose on the
/// sheet.
pub(crate) fn said_as_opened(
    asked: Filling,
    address: &str,
    opener: &str,
    (asked_to_choose, on_the_sheet): (bool, bool),
    strings: &Strings,
) -> Said {
    let word = if on_the_sheet {
        words::LINK_OPENED_ON_THE_SHEET
    } else if asked_to_choose {
        words::LINK_OPENED_INSTEAD_OF_ASKING
    } else {
        words::LINK_OPENED
    };
    strings.say(
        &word.key(),
        &asked.and("address", address).and("opener", opener),
    )
}

/// Why a link was not opened, before anything was started.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkNotOpened {
    /// What arrived is not a link this machine opens.
    NotALink(NotALink),
    /// Nothing on this machine opens that kind of link.
    NothingOpensLinks(NothingOpensLinks),
}

/// What opens `address` for the application `from`, which asked with `ask`
/// set as `asked_to_choose`.
///
/// # Errors
///
/// [`LinkNotOpened`], in the order at the top of this file.
pub fn link_answered(
    from: Applicant,
    address: &str,
    asked_to_choose: bool,
    what_opens: &WhatOpensWhat<'_>,
) -> Result<LinkOpened, LinkNotOpened> {
    let link = Link::checked(address).map_err(LinkNotOpened::NotALink)?;
    let opener = what_opens
        .what_opens_links(link.kind())
        .map_err(LinkNotOpened::NothingOpensLinks)?;
    // **The default first, then every other installed handler**, the list
    // Settings → Default apps draws — so the sheet offers what Settings does.
    let offered = if asked_to_choose {
        let default = opener.application().identifier().to_owned();
        std::iter::once(default.clone())
            .chain(
                what_opens
                    .what_could_open_links(link.kind())
                    .into_iter()
                    .map(|one| one.identifier().to_owned())
                    .filter(|one| *one != default),
            )
            .collect()
    } else {
        Vec::new()
    };
    Ok(LinkOpened {
        from,
        link,
        opener,
        asked_to_choose,
        on_the_sheet: None,
        offered,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_applications::{Application, Chosen, Declared, Installed, LinkKind};
    use alo_strings::Vocabulary;

    const MAIL: &str = "org.gnome.Geary";
    const FIREFOX: &str = "org.mozilla.firefox";

    fn in_english() -> Strings {
        let mut vocabulary = Vocabulary::empty();
        crate::declare_into(&mut vocabulary).unwrap();
        alo_applications::words::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    fn a_machine_with_a_browser() -> (Installed, Declared) {
        let firefox = Application::identified(FIREFOX).unwrap();
        let installed =
            Installed::holding([firefox.clone(), Application::identified(MAIL).unwrap()]);
        let mut declared = Declared::nothing();
        declared.declares_media_types(&firefox, ["x-scheme-handler/https"]);
        (installed, declared)
    }

    /// **A web link opens in the default browser with no grant at all**, and
    /// the sentence names who asked, the address and what opened it.
    #[test]
    fn a_web_link_opens_in_the_default_with_nothing_granted() {
        let (installed, declared) = a_machine_with_a_browser();
        let chosen = Chosen::untouched();
        let what_opens = WhatOpensWhat::on(&installed, &declared, &chosen);
        let opened = link_answered(
            Applicant::named(MAIL),
            "https://example.com/invoice",
            false,
            &what_opens,
        )
        .unwrap();
        assert_eq!(opened.opener().application().identifier(), FIREFOX);
        assert_eq!(opened.link().kind(), LinkKind::Web);
        let said = opened.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        for named in [MAIL, FIREFOX, "https://example.com/invoice"] {
            assert!(said.text().contains(named), "{said}");
        }
    }

    /// **`ask` offers the default first and every other handler after it, and
    /// only one of those can be chosen** — and a choice is said as the
    /// person's.
    #[test]
    fn asking_offers_the_default_first_and_only_an_offered_one_is_chosen() {
        let (installed, mut declared) = a_machine_with_a_browser();
        let librewolf = Application::identified("io.gitlab.librewolf-community").unwrap();
        declared.declares_media_types(&librewolf, ["x-scheme-handler/https"]);
        let installed = Installed::holding(installed.all().cloned().chain([librewolf]));
        let chosen = Chosen::untouched();
        let what_opens = WhatOpensWhat::on(&installed, &declared, &chosen);
        let opened = link_answered(
            Applicant::named(MAIL),
            "https://example.com/",
            true,
            &what_opens,
        )
        .unwrap();
        assert_eq!(
            opened.offered(),
            [
                FIREFOX.to_owned(),
                "io.gitlab.librewolf-community".to_owned()
            ]
        );
        assert!(opened.chosen_on_the_sheet("org.example.Evil").is_none());
        assert!(opened.chosen_on_the_sheet(MAIL).is_none(), "not offered");
        let picked = opened
            .chosen_on_the_sheet("io.gitlab.librewolf-community")
            .unwrap();
        assert_eq!(picked.opens_in(), "io.gitlab.librewolf-community");
        let said = picked.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("You chose"), "{said}");
        assert!(
            said.text().contains("io.gitlab.librewolf-community"),
            "{said}"
        );
        // Without `ask`, nothing is offered.
        let plain = link_answered(
            Applicant::named(MAIL),
            "https://example.com/",
            false,
            &what_opens,
        )
        .unwrap();
        assert!(plain.offered().is_empty());
        assert!(plain.chosen_on_the_sheet(FIREFOX).is_none());
    }

    /// **The sentence for a default used without a sheet** still says so —
    /// kept for records written before the sheet was built.
    #[test]
    fn asking_to_choose_uses_the_default_and_says_so() {
        let (installed, declared) = a_machine_with_a_browser();
        let chosen = Chosen::untouched();
        let what_opens = WhatOpensWhat::on(&installed, &declared, &chosen);
        let opened = link_answered(
            Applicant::named(MAIL),
            "https://example.com/",
            true,
            &what_opens,
        )
        .unwrap();
        assert!(opened.asked_to_choose());
        let strings = in_english();
        let said = said_as_opened(
            Filling::of("application", MAIL),
            "https://example.com/",
            FIREFOX,
            (true, false),
            &strings,
        );
        assert!(said.text().contains("your default"), "{said}");
        assert_ne!(
            said,
            said_as_opened(
                Filling::of("application", MAIL),
                "https://example.com/",
                FIREFOX,
                (false, false),
                &strings
            )
        );
    }

    /// **What is not a link is refused before what opens it is asked**, and a
    /// `mailto:` link with no mail application installed opens in nothing.
    #[test]
    fn what_is_not_a_link_and_a_kind_nothing_opens_are_refused() {
        let (installed, declared) = a_machine_with_a_browser();
        let chosen = Chosen::untouched();
        let what_opens = WhatOpensWhat::on(&installed, &declared, &chosen);
        assert_eq!(
            link_answered(
                Applicant::named(MAIL),
                "file:///etc/shadow",
                false,
                &what_opens
            )
            .unwrap_err(),
            LinkNotOpened::NotALink(NotALink::NotOpenedHere {
                scheme: "file".to_owned()
            })
        );
        let Err(LinkNotOpened::NothingOpensLinks(nothing)) = link_answered(
            Applicant::named(MAIL),
            "mailto:anna@example.com",
            false,
            &what_opens,
        ) else {
            panic!("a mailto: link opened with no mail application installed");
        };
        assert_eq!(nothing.kind(), LinkKind::Mail);
    }
}
