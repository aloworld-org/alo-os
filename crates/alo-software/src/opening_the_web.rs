//! An application's request to open a web address, judged before anything opens.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 3: *the browser from
//! task 2 opens web addresses from any application through the open-with portal.*
//! This is that request's decision — the open-with question with a web address in
//! place of a file — and it is here rather than in `alo-portals` for one reason:
//! which application opens a web address is decided from
//! [`crate::Shipped`], and a decision cannot live above the crate that holds it.
//! The backend that answers `org.freedesktop.portal.OpenURI` calls this the way
//! it calls `alo_portals::open_with::answered` for a file.
//!
//! # In this order, and the order is the point
//!
//! 1. **The asking application is an application** — an identifier, checked by
//!    the rules an `application` argument is checked by ([`NotOpened::NotAnApplication`]).
//! 2. **The address is a web address** ([`crate::WebAddress`]) — refused before
//!    anything about this machine is read, so nothing shapeless learns anything.
//! 3. **The asking application holds some grant at all**, or it is refused here
//!    ([`NotOpened::NothingGranted`]). This is where the order matters: an
//!    application nobody has granted anything must not learn, from the shape of
//!    its refusal, whether this machine has a browser and which one — that is a
//!    fingerprint of the person in front of it. So the list is asked before
//!    [`crate::WhatOpensWebAddresses`] is.
//! 4. **Which application opens web addresses** ([`crate::NothingOpensThem`] when
//!    none does).
//! 5. **The asking application may reach that application** — ADR 0040's row for
//!    the open-with portal, which for a file is *the file, and the application to
//!    open it*. A web address is no file, so what is left is the second half: the
//!    asking application has to have been granted the browser
//!    ([`NotOpened::NotAllowed`]).
//!
//! # Any application may ask, and none of them may assume
//!
//! *From any application* is about the road, not about what it costs: there is no
//! list of applications privileged to open links and no application that is
//! refused for being the wrong one. What each of them needs is the same grant —
//! the browser — made by the person and revocable like every other. Nothing here
//! holds `&mut Grants`, so answering a request can never be how one gets made,
//! and a revoked grant is refused at the next request.
//!
//! # It opens nothing
//!
//! [`Opened`] is a name, a reason, an address and the grant it was allowed by. A
//! record owes an answer to *against which grant*, and this is the one moment
//! that answer exists. Asking the application to open the address needs a running
//! session and is not this crate's.

use std::time::SystemTime;

use alo_applications::Application;
use alo_capability::{Applicant, Ask, GrantId, Grants, NotAllowed};
use alo_strings::{Filling, Said, Strings};

use crate::browsing::{NothingOpensThem, TheBrowser, WhatOpensWebAddresses};
use crate::web_address::{NotAWebAddress, WebAddress};
use crate::words;

/// A request to open a web address that the grants allowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    /// The application that asked.
    from: Applicant,
    /// The address it asked to have opened.
    address: WebAddress,
    /// The application that opens it, and why it is the one.
    browser: TheBrowser,
    /// The grant that allowed the asking application to reach it — the handle a
    /// person revokes by.
    against: GrantId,
}

impl Opened {
    /// The application that asked.
    #[must_use]
    pub const fn from(&self) -> &Applicant {
        &self.from
    }

    /// The address to open.
    #[must_use]
    pub const fn address(&self) -> &WebAddress {
        &self.address
    }

    /// The application that opens it, and why it is the one.
    #[must_use]
    pub const fn browser(&self) -> &TheBrowser {
        &self.browser
    }

    /// The grant the asking application was allowed by.
    #[must_use]
    pub const fn against(&self) -> GrantId {
        self.against
    }
}

/// Why a web address was not opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotOpened {
    /// What asked did not name itself as an application.
    NotAnApplication,
    /// What arrived was not a web address.
    NotAWebAddress(NotAWebAddress),
    /// The asking application holds no grant at all.
    NothingGranted {
        /// The application that asked.
        from: Applicant,
    },
    /// No grant of the asking application's covers the application that opens
    /// web addresses.
    NotAllowed(NotAllowed),
    /// No application on this machine opens web addresses.
    NothingOpensThem(NothingOpensThem),
}

impl NotOpened {
    /// What this says, in the language the person reads — each the sentence of
    /// the crate that decided it.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotAnApplication => {
                strings.say(&words::WEB_NOT_AN_APPLICATION.key(), &Filling::nothing())
            }
            Self::NotAWebAddress(not) => {
                let word = match not {
                    NotAWebAddress::CarriesAPassword => words::WEB_CARRIES_A_PASSWORD,
                    NotAWebAddress::Nothing
                    | NotAWebAddress::NoScheme
                    | NotAWebAddress::NotAWebScheme { .. }
                    | NotAWebAddress::NotAHost
                    | NotAWebAddress::NotOneLine
                    | NotAWebAddress::TooLong => words::NOT_A_WEB_ADDRESS,
                };
                strings.say(&word.key(), &Filling::nothing())
            }
            Self::NothingGranted { from } => strings.say(
                &words::WEB_NOTHING_GRANTED.key(),
                &Filling::of(words::APPLICATION, from.as_str().to_owned()),
            ),
            Self::NotAllowed(not) => not.said(strings),
            Self::NothingOpensThem(nothing) => nothing.said(strings),
        }
    }
}

/// Which application opens `address` for the application `from`, and whether it
/// may.
///
/// # Errors
///
/// [`NotOpened`], in the order at the top of this file.
pub fn opened(
    from: &str,
    address: &str,
    what_opens: &WhatOpensWebAddresses<'_>,
    grants: &Grants,
    now: SystemTime,
) -> Result<Opened, NotOpened> {
    let from = Application::identified(from)
        .map(|application| Applicant::named(application.identifier()))
        .map_err(|_| NotOpened::NotAnApplication)?;
    let address = WebAddress::checked(address).map_err(NotOpened::NotAWebAddress)?;
    if !grants.allows_anything(&from, now) {
        return Err(NotOpened::NothingGranted { from });
    }
    let browser = what_opens
        .what_opens_them()
        .map_err(NotOpened::NothingOpensThem)?;
    let against = grants
        .allowing(
            &from,
            &Ask::application(browser.application().identifier()),
            now,
        )
        .map_err(NotOpened::NotAllowed)?;
    Ok(Opened {
        from,
        address,
        browser,
        against,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::browsing::Because;
    use crate::shipped::Shipped;
    use crate::testing::in_english;
    use alo_applications::Installed;
    use alo_capability::{Grant, Reach};
    use std::time::Duration;

    const READER: &str = "org.gnome.Papers";
    const FIREFOX: &str = "org.mozilla.firefox";

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    fn installed() -> Installed {
        Installed::holding([
            Application::called(FIREFOX, "Firefox").unwrap(),
            Application::identified(READER).unwrap(),
        ])
    }

    /// A machine where `who` has been granted `what`, for an hour from noon.
    fn granting(who: &str, what: Reach) -> Grants {
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked_for(&Applicant::named(who).grantee(), what, noon(), hour()).unwrap(),
        );
        grants
    }

    /// **An ordinary application opens a web address in the browser**, against
    /// the one grant the person made it.
    #[test]
    fn an_application_granted_the_browser_opens_a_web_address_in_it() {
        let installed = installed();
        let shipped = Shipped::decided().unwrap();
        let what_opens = WhatOpensWebAddresses::on(&installed, &shipped, None);
        let grants = granting(READER, Reach::Application(FIREFOX.to_owned()));

        let opened = opened(
            READER,
            "https://example.com/report",
            &what_opens,
            &grants,
            noon(),
        )
        .unwrap();
        assert_eq!(opened.from(), &Applicant::named(READER));
        assert_eq!(opened.address().as_str(), "https://example.com/report");
        assert_eq!(opened.address().host(), "example.com");
        assert_eq!(opened.browser().application().identifier(), FIREFOX);
        assert_eq!(opened.browser().because(), &Because::ThisMachineShipsIt);
        assert_eq!(
            grants.active_at(noon()).next().unwrap().id,
            opened.against()
        );
    }

    /// **An application granted nothing is refused before this machine is
    /// read** — it learns neither whether there is a browser nor which one.
    #[test]
    fn an_application_granted_nothing_learns_nothing_about_this_machine() {
        let shipped = Shipped::decided().unwrap();
        let grants = Grants::default();
        let here = installed();
        let empty = Installed::nothing();

        for installed in [&here, &empty] {
            let what_opens = WhatOpensWebAddresses::on(installed, &shipped, None);
            assert_eq!(
                opened(READER, "https://example.com/", &what_opens, &grants, noon()).unwrap_err(),
                NotOpened::NothingGranted {
                    from: Applicant::named(READER)
                },
                "a machine with a browser and one without refused differently"
            );
        }
    }

    /// **A grant over something else is not a grant over the browser**, and a
    /// grant that has run out is refused at the next request — as the list now
    /// is, with nothing remembered from the last one.
    #[test]
    fn a_grant_over_something_else_or_a_lapsed_one_opens_nothing() {
        let installed = installed();
        let shipped = Shipped::decided().unwrap();
        let what_opens = WhatOpensWebAddresses::on(&installed, &shipped, None);

        let elsewhere = granting(READER, Reach::Application("org.gnome.Loupe".to_owned()));
        assert!(matches!(
            opened(
                READER,
                "https://example.com/",
                &what_opens,
                &elsewhere,
                noon()
            ),
            Err(NotOpened::NotAllowed(NotAllowed::Never { .. }))
        ));

        // The browser was granted for an hour, and the hour is over. Nothing
        // else was granted, so this is refused where an application holding
        // nothing is: before this machine is read at all.
        let lapsed = granting(READER, Reach::Application(FIREFOX.to_owned()));
        let later = noon() + hour() + hour();
        assert_eq!(
            opened(READER, "https://example.com/", &what_opens, &lapsed, later).unwrap_err(),
            NotOpened::NothingGranted {
                from: Applicant::named(READER)
            }
        );

        // With something else still running, the lapsed grant over the browser
        // is what the refusal names, and it names the grant that ran out.
        let mut both = lapsed;
        both.grant(
            Grant::checked_for(
                &Applicant::named(READER).grantee(),
                Reach::Application("org.gnome.Loupe".to_owned()),
                later,
                hour(),
            )
            .unwrap(),
        );
        assert!(matches!(
            opened(READER, "https://example.com/", &what_opens, &both, later),
            Err(NotOpened::NotAllowed(NotAllowed::Lapsed { .. }))
        ));
    }

    /// **What is not a web address and what is not an application are refused
    /// first**, before the grants are read at all.
    #[test]
    fn what_is_not_a_web_address_is_refused_before_the_grants_are_read() {
        let installed = installed();
        let shipped = Shipped::decided().unwrap();
        let what_opens = WhatOpensWebAddresses::on(&installed, &shipped, None);
        let nothing = Grants::default();

        for address in ["file:///etc/shadow", "javascript:alert(1)", "", "  "] {
            assert!(
                matches!(
                    opened(READER, address, &what_opens, &nothing, noon()),
                    Err(NotOpened::NotAWebAddress(_))
                ),
                "{address}"
            );
        }
        assert_eq!(
            opened(
                READER,
                "https://anna:hunter2@example.com/",
                &what_opens,
                &nothing,
                noon()
            )
            .unwrap_err(),
            NotOpened::NotAWebAddress(NotAWebAddress::CarriesAPassword)
        );
        for from in ["", "   ", "org.gnome.Papers extra", "org.gnome\nPapers"] {
            assert_eq!(
                opened(from, "https://example.com/", &what_opens, &nothing, noon()).unwrap_err(),
                NotOpened::NotAnApplication,
                "{from:?}"
            );
        }
    }

    /// **A machine whose browser was removed refuses in words**, to an
    /// application that was granted it while it was there.
    #[test]
    fn a_machine_with_no_browser_refuses_in_words() {
        let installed = Installed::holding([Application::identified(READER).unwrap()]);
        let shipped = Shipped::decided().unwrap();
        let what_opens = WhatOpensWebAddresses::on(&installed, &shipped, None);
        let grants = granting(READER, Reach::Application(FIREFOX.to_owned()));
        let refused =
            opened(READER, "https://example.com/", &what_opens, &grants, noon()).unwrap_err();
        assert!(matches!(refused, NotOpened::NothingOpensThem(_)));
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
    }

    /// **Every refusal is a sentence**, and no two of them say the same thing.
    #[test]
    fn every_refusal_is_a_sentence_of_its_own() {
        let strings = in_english();
        let mut seen = Vec::new();
        for refused in [
            NotOpened::NotAnApplication,
            NotOpened::NotAWebAddress(NotAWebAddress::NoScheme),
            NotOpened::NotAWebAddress(NotAWebAddress::CarriesAPassword),
            NotOpened::NothingGranted {
                from: Applicant::named(READER),
            },
            NotOpened::NotAllowed(NotAllowed::Never {
                application: Applicant::named(READER),
                wanted: Ask::application(FIREFOX),
            }),
        ] {
            let said = refused.said(&strings);
            assert!(!said.is_a_bug(), "{refused:?}: {said}");
            assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
            seen.push(said.into_text());
        }
        assert!(seen.iter().any(|said| said.contains(READER)));
        seen.dedup();
        assert_eq!(seen.len(), 5);
    }
}
