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
use alo_capability::{Ask, Call, Given, Grants, Takes};

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
        let named = as_the_catalogue_names_it(verb, named)?;
        self.calling(verb, &named, now)
    }

    /// What the model gave, with each application argument that names one
    /// installed, permitted application replaced by its identifier.
    fn as_this_machine_names_them<'g>(
        &mut self,
        verb: &str,
        given: &[(&'g str, Given)],
        grants: &Grants,
        now: SystemTime,
    ) -> Vec<(&'g str, Given)> {
        let grantee = self.grantee().clone();
        let machine = self.machine();
        let applications: Vec<String> =
            machine.verbs().of(verb).map_or_else(Vec::new, |declared| {
                declared
                    .args()
                    .iter()
                    .filter(|arg| matches!(arg.takes(), Takes::Application))
                    .map(|arg| arg.name().to_owned())
                    .collect()
            });
        if applications.is_empty() {
            return given.to_vec();
        }
        let installed = machine.the_applications_installed_now();
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
                    .permitted(&grantee, &Ask::Application(identifier.to_owned()), now)
                    .is_ok();
                if permitted {
                    (*argument, Given::text(identifier))
                } else {
                    (*argument, value.clone())
                }
            })
            .collect()
    }
}

/// **The identifier of the one application in `installable` that `said`
/// names**, matched as a name of an installed application is
/// (`Installed::answering_to`): an identifier exactly, or a short name or a
/// name with no dots that only one answers to. Several, or none, is [`None`]:
/// nothing is guessed.
pub(crate) fn installable_answering_to(installable: &Installed, said: &str) -> Option<String> {
    match installable.answering_to(said) {
        AnsweringTo::One(application) => Some(application.identifier().to_owned()),
        AnsweringTo::Several(_) | AnsweringTo::Nothing => None,
    }
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
/// where it was a name the catalogue answers to.
///
/// A dotted identifier the catalogue does not hold is left as it was — a
/// person may install any application by its identifier, and the place says
/// whether it offers it. A name with no dots that no one application answers
/// to is refused here, before anything is proposed: there is no identifier to
/// put in the sentence the person would approve.
///
/// # Errors
/// [`NotDone::CouldNotInstall`], as the place not offering what was named.
pub fn as_the_catalogue_names_it<'g>(
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
    let installable = the_catalogue_as_installable();
    given
        .into_iter()
        .map(|(argument, value)| {
            let Given::Text(said) = &value else {
                return Ok((argument, value));
            };
            if argument.trim() != alo_software::words::APPLICATION {
                return Ok((argument, value));
            }
            if let Some(identifier) = installable
                .as_ref()
                .and_then(|installable| installable_answering_to(installable, said))
            {
                return Ok((argument, Given::text(identifier)));
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
