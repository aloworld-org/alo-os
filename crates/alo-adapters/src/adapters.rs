//! The adapters this machine has loaded, and the verbs they put on its list.
//!
//! **One application, one adapter.** Two adapters for the text editor would be
//! two sets of verbs an agent could reach it through, and a person reading the
//! list of what an agent may do would have to know which was which.
//!
//! **What an agent is offered is what it can carry out.** [`Adapters::offered_to`]
//! holds only the verbs of adapters whose application the agent holds a grant
//! over at that moment, so a person is never asked to approve something that
//! would be refused the moment it ran for want of a grant nobody made. The
//! grant is asked again when it runs (`crate::driving`), because it can be
//! revoked in between.

use std::time::SystemTime;

use alo_capability::{Ask, Grantee, Grants, Verbs};

use crate::adapter::Adapter;
use crate::adapter_verb::AdapterVerb;
use crate::loading::Loaded;
use crate::not_loaded::NotLoaded;

/// Every adapter this machine loaded.
#[derive(Debug, Clone, Default)]
pub struct Adapters {
    /// In the order they were added.
    loaded: Vec<Loaded>,
}

impl Adapters {
    /// Add a loaded adapter.
    ///
    /// # Errors
    /// [`NotLoaded::Twice`] for a second adapter of one name or for one
    /// application.
    pub fn add(&mut self, loaded: Loaded) -> Result<(), NotLoaded> {
        let adding = loaded.adapter();
        if self.loaded.iter().any(|already| {
            already.adapter().name == adding.name
                || already.adapter().application == adding.application
        }) {
            return Err(NotLoaded::Twice {
                adapter: adding.name.to_owned(),
                application: adding.application.to_owned(),
            });
        }
        self.loaded.push(loaded);
        Ok(())
    }

    /// Every adapter, in the order it was added.
    pub fn all(&self) -> impl Iterator<Item = &'static Adapter> + '_ {
        self.loaded.iter().map(Loaded::adapter)
    }

    /// Put every adapter's verbs on the machine's list.
    ///
    /// # Errors
    /// [`NotLoaded::Taken`] if a name is on the list already — nothing replaces
    /// a verb.
    pub fn declare_into(&self, verbs: &mut Verbs) -> Result<(), NotLoaded> {
        for loaded in &self.loaded {
            for verb in loaded.verbs() {
                verbs.declare(verb.clone())?;
            }
        }
        Ok(())
    }

    /// The verbs this agent can be offered now: those of adapters whose
    /// application it holds a grant over.
    ///
    /// # Errors
    /// [`NotLoaded::Taken`], which adapters added through [`Adapters::add`]
    /// cannot cause.
    pub fn offered_to(
        &self,
        agent: &Grantee,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Verbs, NotLoaded> {
        let mut verbs = Verbs::default();
        for loaded in &self.loaded {
            let application = Ask::Application(loaded.adapter().application.to_owned());
            if grants.permits(agent, &application, now) {
                for verb in loaded.verbs() {
                    verbs.declare(verb.clone())?;
                }
            }
        }
        Ok(verbs)
    }

    /// The adapter and the declared verb the machine knows by this name.
    #[must_use]
    pub fn carrying_out(&self, named: &str) -> Option<(&'static Adapter, &'static AdapterVerb)> {
        self.loaded
            .iter()
            .find_map(|loaded| loaded.declared(named).map(|verb| (loaded.adapter(), verb)))
    }
}
