//! **An application named in a call is the one this machine has by that
//! name**, put into the call by its identifier before anybody is asked.
//!
//! The walk of 2026-11-14: VSCodium was installed as `com.vscodium.codium`, a
//! model asked to open it sent `vscodium`, and the call was refused as *nothing
//! installed on this machine is vscodium — install it*; asked again, alo
//! installed something from Flathub. An identifier is matched exactly
//! everywhere below this file — by the grants, by
//! `alo_applications::Reaching`, by the sentence a person approves — and that
//! stays: `alo_applications::Installed::answering_to` says which installed
//! application a name means, and this puts **that application's identifier**
//! into the call, so what the person approves names the application exactly
//! and the grant is asked about the identifier it was made over.
//!
//! # Only where the grants already permit it
//!
//! A name is put into the call as an identifier **only when the grants permit
//! that identifier** at this moment. Otherwise the call keeps what the model
//! sent, and is refused in those words — so a name for an application nobody
//! granted is refused exactly as a name for one that is not installed, and an
//! agent cannot learn from a refusal what this person has
//! (`alo_applications::Reaching`'s header is why that matters). A name that
//! means several applications, or none, is left as it was too: nothing is
//! guessed.
//!
//! # On the roads that ask or act on this machine
//!
//! A read and a change on this machine come through here
//! ([`Turning::reading`], [`Turning::proposing`], [`Turning::changing`]). A
//! request from a paired machine does not: it acts only under a grant made here
//! by identifier, and is held to that identifier as it was sent.

use std::time::SystemTime;

use alo_applications::{AnsweringTo, Installed};
use alo_capability::{Ask, Call, Given, Grantee, Grants, Takes, Verbs};

use crate::refusing::NotDone;
use crate::turning::Turning;

impl Turning<'_, '_> {
    /// [`Turning::calling`], with every application the call names given by
    /// the identifier of the installed application it means, where the grants
    /// permit that one.
    ///
    /// # Errors
    /// What `Turning::calling` answers.
    pub(crate) fn calling_by_name(
        &mut self,
        verb: &str,
        given: &[(&str, Given)],
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Call, NotDone> {
        let named = self.as_this_machine_names_them(verb, given, grants, now);
        let named: Vec<(&str, Given)> = named
            .iter()
            .map(|(argument, value)| (*argument, value.clone()))
            .collect();
        // **A refusal here is written down**, as `Turning::calling` writes the
        // ones it makes: every refusal leaves a record.
        let named = match as_the_catalogue_names_it(verb, named) {
            Ok(named) => named,
            Err(refused) => {
                let said = refused.said(self.machine().strings()).into_text();
                let entry = alo_record::Entry::turned_away(verb, &said, self.grantee(), now);
                self.writing_down(entry)?;
                return Err(refused);
            }
        };
        self.calling(verb, &named, now)
    }

    /// What the model gave, with each application argument that names one
    /// installed, permitted application replaced by its identifier
    /// ([`as_this_machine_names_them`], with this turn's machine and grantee).
    fn as_this_machine_names_them<'g>(
        &mut self,
        verb: &str,
        given: &[(&'g str, Given)],
        grants: &Grants,
        now: SystemTime,
    ) -> Vec<(&'g str, Given)> {
        let grantee = self.grantee().clone();
        let machine = self.machine();
        // **Nothing is read for a verb that names no application**, as before
        // this was public: the installed list is asked only when it can matter.
        if the_applications_named_in(machine.verbs(), verb).is_empty() {
            return given.to_vec();
        }
        let installed = machine.the_applications_installed_now();
        let named = as_this_machine_names_them(
            machine.verbs(),
            &installed,
            grants,
            &grantee,
            verb,
            given,
            now,
        );
        // **What the call wanted and was not given is noted for the person**,
        // and only for them: the agent is told exactly the refusal it is told
        // without this, so it learns nothing about what is installed.
        let wanted = the_applications_wanted(
            self.machine().verbs(),
            &installed,
            grants,
            &grantee,
            verb,
            given,
            now,
        );
        self.wanting(wanted);
        named
    }
}

/// **What `given` is, as this machine names an application**: each argument
/// `verb` declares as taking an application ([`Takes::Application`]) that names
/// one application in `installed` (`Installed::answering_to`) is put as that
/// application's identifier — where `grants` permit `grantee` that
/// identifier at `now`. Everything else is left as it was given.
///
/// This is the turn's own matching, public so that a check holding a model to
/// what this machine accepts (`alo-every-try`) calls it rather than a copy.
#[must_use]
pub fn as_this_machine_names_them<'g>(
    verbs: &Verbs,
    installed: &Installed,
    grants: &Grants,
    grantee: &Grantee,
    verb: &str,
    given: &[(&'g str, Given)],
    now: SystemTime,
) -> Vec<(&'g str, Given)> {
    let applications = the_applications_named_in(verbs, verb);
    if applications.is_empty() {
        return given.to_vec();
    }
    given
        .iter()
        .map(|(argument, value)| {
            let Given::Text(said) = value else {
                return (*argument, value.clone());
            };
            if !applications.iter().any(|name| name == argument) {
                return (*argument, value.clone());
            }
            let AnsweringTo::One(application) = installed.answering_to(said) else {
                return (*argument, value.clone());
            };
            let identifier = application.identifier();
            let permitted = grants
                .permitted(grantee, &Ask::Application(identifier.to_owned()), now)
                .is_ok();
            if permitted {
                (*argument, Given::text(identifier))
            } else {
                (*argument, value.clone())
            }
        })
        .collect()
}

/// **The installed applications `given` names that nothing grants `grantee` at
/// `now`**, by identifier, once each: each argument `verb` declares as taking
/// an application whose text names exactly one installed application
/// (`Installed::answering_to`) that the grants do not permit.
///
/// What the person's desktop offers on the sheet that asks whether to give alo
/// one (the owner's decision of 2026-11-16). Nothing for a name that means
/// several applications or none, which the sheet could not name, and nothing
/// at full access, where every application is permitted already.
#[must_use]
pub fn the_applications_wanted(
    verbs: &Verbs,
    installed: &Installed,
    grants: &Grants,
    grantee: &Grantee,
    verb: &str,
    given: &[(&str, Given)],
    now: SystemTime,
) -> Vec<String> {
    let applications = the_applications_named_in(verbs, verb);
    let mut wanted: Vec<String> = Vec::new();
    for (argument, value) in given {
        let Given::Text(said) = value else {
            continue;
        };
        if !applications.iter().any(|name| name == argument) {
            continue;
        }
        let AnsweringTo::One(application) = installed.answering_to(said) else {
            continue;
        };
        let identifier = application.identifier();
        let permitted = grants
            .permitted(grantee, &Ask::Application(identifier.to_owned()), now)
            .is_ok();
        if !permitted && !wanted.iter().any(|it| it == identifier) {
            wanted.push(identifier.to_owned());
        }
    }
    wanted
}

/// The arguments `verb` declares as taking an application, by name; none for
/// a verb nobody declared.
fn the_applications_named_in<'v>(verbs: &'v Verbs, verb: &str) -> Vec<&'v str> {
    verbs.of(verb).map_or_else(Vec::new, |declared| {
        declared
            .args()
            .iter()
            .filter(|arg| matches!(arg.takes(), Takes::Application))
            .map(|arg| arg.name())
            .collect()
    })
}

/// What the decided catalogue offers, as applications a name can be matched
/// against — or nothing, where the catalogue would not read, which a test of
/// `alo-software` makes impossible on a machine that shipped.
fn the_catalogue_as_installable() -> Option<Installed> {
    alo_software::Catalogue::decided().ok().map(|catalogue| {
        Installed::holding(
            catalogue
                .every()
                .iter()
                .map(|listed| listed.application().clone()),
        )
    })
}

/// **An application to install, named by its name, is the one the catalogue
/// gives that name** (the owner, 2026-11-15, relayed): `given` with
/// `install_application`'s application put as the catalogue's identifier
/// where it was a name the catalogue answers to, matched as a name of an
/// installed application is (`Installed::answering_to`).
///
/// A dotted identifier the catalogue does not hold is left as it was — a
/// person may install any application by its identifier, and the place says
/// whether it offers it. **That matching is case-insensitive on an
/// identifier**, so `org.gimp.gimp` is the catalogue's `org.gimp.GIMP`; a
/// different identifier, such as `org.gnome.GIMP`, is never rewritten. A name
/// with no dots that several answer to, or none, is refused here, before
/// anything is proposed: there is no one identifier to put in the sentence the
/// person would approve.
///
/// # Errors
/// [`NotDone::CouldNotInstall`]: the place not offering what was named, or the
/// name meaning more than one application.
pub fn as_the_catalogue_names_it<'g>(
    verb: &str,
    given: Vec<(&'g str, Given)>,
) -> Result<Vec<(&'g str, Given)>, NotDone> {
    as_these_name_it(the_catalogue_as_installable().as_ref(), verb, given)
}

/// [`as_the_catalogue_names_it`], against `installable` rather than the
/// decided catalogue — so a test can hold the rule to a list it wrote.
pub(crate) fn as_these_name_it<'g>(
    installable: Option<&Installed>,
    verb: &str,
    given: Vec<(&'g str, Given)>,
) -> Result<Vec<(&'g str, Given)>, NotDone> {
    if verb.trim() != alo_software::INSTALL_APPLICATION {
        return Ok(given);
    }
    let source = given
        .iter()
        .find(|(argument, _)| argument.trim() == alo_software::words::SOURCE)
        .and_then(|(_, value)| match value {
            Given::Text(text) => Some(text.clone()),
            Given::Number(_) => None,
        })
        .unwrap_or_default();
    given
        .into_iter()
        .map(|(argument, value)| {
            let Given::Text(said) = &value else {
                return Ok((argument, value));
            };
            if argument.trim() != alo_software::words::APPLICATION {
                return Ok((argument, value));
            }
            let answered = installable.map(|installable| installable.answering_to(said));
            match answered {
                Some(AnsweringTo::One(application)) => {
                    return Ok((argument, Given::text(application.identifier())));
                }
                Some(AnsweringTo::Several(_)) if !said.contains('.') => {
                    return Err(NotDone::CouldNotInstall(
                        alo_software::NotDone::NamesMoreThanOne {
                            application: said.trim().to_owned(),
                        },
                    ));
                }
                Some(AnsweringTo::Several(_) | AnsweringTo::Nothing) | None => {}
            }
            if said.contains('.') {
                return Ok((argument, value));
            }
            Err(NotDone::CouldNotInstall(
                alo_software::NotDone::NotOffered {
                    source: source.clone(),
                    application: said.trim().to_owned(),
                },
            ))
        })
        .collect()
}

#[cfg(test)]
#[path = "naming_an_application_tests.rs"]
mod tests;
