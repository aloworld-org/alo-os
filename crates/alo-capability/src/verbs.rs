//! The verbs this machine has, which is the list of everything an agent can do.
//!
//! `docs/contracts/agent-verbs.md` opens by saying the list is closed: if a
//! capability is not on it, `alo-agentd` does not have it. This type is that
//! sentence. [`Verbs::call`] is the only way in, and a name that is not
//! declared comes back as [`CallError::NoSuchVerb`] rather than as anything
//! resembling an attempt.
//!
//! **The registry cannot hold a verb that breaks the contract, and it manages
//! that by not being able to receive one.** [`crate::verb::Verb::checked`] is
//! the only constructor there is, so every rule in that file has already been
//! applied to anything that reaches [`Verbs::declare`]. What is left for the
//! registry itself is the rule it is the only place that can enforce: a name is
//! declared once and never means a second thing.
//!
//! The list starts empty. What goes on it is decided verb by verb, in
//! `docs/features.md` and in this contract, and nothing is smuggled in here as
//! a default: the file verbs are declared in the `alo-files` crate and put on a
//! list by whoever wants them, so a registry that was never given one has no
//! capabilities at all.

use crate::arg::Given;
use crate::call::{Call, CallError};
use crate::verb::Verb;

/// Why a verb could not be added to the list.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VerbsError {
    /// A name that is already on the list.
    #[error("there is already a verb called {name} — a name is never reused for a second meaning")]
    AlreadyDeclared {
        /// The name that was declared twice.
        name: String,
    },
}

/// Every verb an agent may ask for on this machine.
#[derive(Debug, Clone, Default)]
pub struct Verbs {
    /// In the order they were declared, which is the order the list reads in.
    declared: Vec<Verb>,
}

impl Verbs {
    /// Put a verb on the list.
    ///
    /// # Errors
    /// [`VerbsError::AlreadyDeclared`] if the name is taken. Replacing a verb
    /// silently would let an adapter shadow a system verb with something of the
    /// same name and a different meaning, and the agent would have no way of
    /// telling which one it had asked for.
    pub fn declare(&mut self, verb: Verb) -> Result<(), VerbsError> {
        if self.of(verb.name()).is_some() {
            return Err(VerbsError::AlreadyDeclared {
                name: verb.name().to_owned(),
            });
        }
        self.declared.push(verb);
        Ok(())
    }

    /// The verb of this name. Matched exactly, like every other identity here.
    #[must_use]
    pub fn of(&self, name: &str) -> Option<&Verb> {
        self.declared.iter().find(|verb| verb.name() == name)
    }

    /// Every verb, in the order it was declared.
    ///
    /// This is the list a person is entitled to read: what an agent on this
    /// machine can do, in one place, with each verb's purpose in its own words.
    pub fn all(&self) -> impl Iterator<Item = &Verb> {
        self.declared.iter()
    }

    /// How many verbs there are.
    #[must_use]
    pub fn len(&self) -> usize {
        self.declared.len()
    }

    /// Whether nothing has been declared at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.declared.is_empty()
    }

    /// Make a call: find the verb, and validate what arrived against it.
    ///
    /// This does not consult the grants and does not run anything. What comes
    /// back is a [`Call`] that could be permitted and then, if it changes
    /// something, approved — three separate questions, kept separate.
    ///
    /// # Errors
    /// [`CallError`], starting with [`CallError::NoSuchVerb`] for a name that
    /// is not on the list.
    pub fn call(&self, name: &str, given: &[(&str, Given)]) -> Result<Call, CallError> {
        let name = name.trim();
        let verb = self.of(name).ok_or_else(|| CallError::NoSuchVerb {
            name: name.to_owned(),
        })?;
        Call::of(verb, given)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::arg::{Arg, Takes};
    use crate::verb::{Effect, Requires, Verb};

    fn list_folder() -> Verb {
        Verb::checked(
            "list_folder",
            "list what is in a folder",
            Effect::Read,
            vec![Arg::taking("folder", "the folder to list", Takes::Path)],
            Requires::grants_over(["folder"]),
            "list what is in {folder}",
        )
        .unwrap()
    }

    fn one_verb() -> Verbs {
        let mut verbs = Verbs::default();
        verbs.declare(list_folder()).unwrap();
        verbs
    }

    /// The closed list, as a test. Everything not declared does not exist, and
    /// the refusal says so rather than describing a failure to find something.
    #[test]
    fn a_verb_that_is_not_on_the_list_does_not_exist() {
        let verbs = one_verb();
        let err = verbs
            .call("run_shell", &[("folder", Given::text("/home/anna"))])
            .unwrap_err();
        assert_eq!(
            err,
            CallError::NoSuchVerb {
                name: "run_shell".to_owned()
            }
        );
        assert!(err.to_string().contains("the list is closed"), "{err}");
        assert!(verbs.of("List_Folder").is_none());
    }

    /// Nothing is on the list until somebody put it there. A registry that
    /// arrived with defaults would be a set of capabilities nobody chose.
    #[test]
    fn the_list_starts_empty() {
        let verbs = Verbs::default();
        assert!(verbs.is_empty());
        assert_eq!(verbs.len(), 0);
        assert_eq!(verbs.all().count(), 0);
        assert!(matches!(
            verbs.call("list_folder", &[]),
            Err(CallError::NoSuchVerb { .. })
        ));
    }

    /// A name means one thing. An adapter cannot declare `move_file` over the
    /// top of the system's and have calls quietly arrive at its own.
    #[test]
    fn a_name_is_never_reused() {
        let mut verbs = one_verb();
        let shadow = Verb::checked(
            "list_folder",
            "list what is in a folder, differently",
            Effect::Change,
            vec![Arg::taking("folder", "the folder", Takes::Path)],
            Requires::grants_over(["folder"]),
            "list {folder}",
        )
        .unwrap();
        assert_eq!(
            verbs.declare(shadow).unwrap_err(),
            VerbsError::AlreadyDeclared {
                name: "list_folder".to_owned()
            }
        );
        assert_eq!(verbs.len(), 1);
        assert_eq!(verbs.of("list_folder").unwrap().effect(), Effect::Read);
    }

    /// The list a person reads: what each verb is for, in its own words.
    #[test]
    fn the_list_says_what_each_verb_is_for() {
        let verbs = one_verb();
        let listed: Vec<_> = verbs.all().collect();
        assert_eq!(listed.len(), 1);
        let first = listed.first().unwrap();
        assert_eq!(first.name(), "list_folder");
        assert_eq!(first.purpose(), "list what is in a folder");
        assert!(!first.effect().waits_for_approval());
    }

    /// A call through the registry is a call: validated, with its sentence
    /// already generated and its questions for the grants ready to ask.
    #[test]
    fn a_call_through_the_list_arrives_validated() {
        let verbs = one_verb();
        let call = verbs
            .call(
                " list_folder ",
                &[("folder", Given::text("/home/anna/Invoices"))],
            )
            .unwrap();
        assert_eq!(call.verb(), "list_folder");
        assert_eq!(call.sentence(), "list what is in /home/anna/Invoices");
        assert_eq!(call.asks().len(), 1);
        assert!(!call.waits_for_approval());
    }
}
