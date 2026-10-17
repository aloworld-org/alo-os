//! What every turn on this machine happens against, and what it can carry out.
//!
//! Six things a turn needs that are not the turn's own: the verbs an agent may
//! ask for, the words the person in front of the machine reads, how a path is
//! resolved, what a verb's work runs inside, what is leaving right now, and
//! where what happened is written down. They are made once, when the daemon
//! starts, and every turn borrows them.
//!
//! # A machine cannot be made without a boundary
//!
//! [`Bounding`] is held here for the reason [`crate::Kept`] is: the promise is
//! only structural if a machine cannot exist without one. ADR 0015 says a turn
//! whose boundary cannot be applied does not run, so there is no honest
//! fallback — a machine that could carry a verb out with nothing around it
//! would be the guarantee turned off wherever somebody forgot, which is exactly
//! what ADR 0013 says is wrong with promising it in our own code.
//!
//! # The list an agent may ask from is the list this machine can carry out
//!
//! [`Machine::carrying_out_file_and_software_verbs`] builds the registry rather
//! than receiving one, and that is the decision this file exists for. A
//! registry handed in from outside could hold a verb nothing here can execute —
//! the four application verbs are declared and portable, and what actually
//! moves a window is Wayland and D-Bus in another process — and an agent asking
//! for one would be told *the machine could not*, which is a sentence about a
//! disk being full rather than about a capability this machine does not have.
//!
//! So the two lists are one list. A verb an agent can name is a verb something
//! here can do, and adding an executor and adding to the offered list are the
//! same edit. The name says which executors there are, so the day there is a
//! third one the name is wrong until somebody fixes it — which is what it is
//! for.
//!
//! **It said `carrying_out_file_verbs` until 2026-10-13, and the name was the
//! whole of what was wrong.** `alo_software::software_verbs` had been declared,
//! worded, tested and undoable since it was written, and
//! `docs/contracts/agent-verbs.md` listed `install_application` publicly for
//! third parties to build against. Nothing offered it: `software_verbs` was
//! called by its own tests and by nothing else, so on a real machine an agent
//! could install nothing, and the sentence above is what said so. The second
//! executor is `crate::installing_an_application`.
//!
//! Asking a model something is **not** one of those verbs, and the name of the
//! constructor stays right because of it: a question an agent puts to a model
//! is what an agent *is* rather than something it is granted, so it is a door
//! on [`crate::Turning`] and not a row in the registry. `alo-asking` never asks
//! the grants anything.
//!
//! # The indicator is the machine's, and the rule in force is not
//!
//! [`Indicator`] is held here for the reason the record is: **one machine has
//! one of each, and a second would be a second place to look.** Item 16 settled
//! that on the indicator itself — what alo OS does on its own goes on the same
//! list as what an agent causes, because the failure law 1 exists to prevent is
//! not *the policy was wrong* but *nobody could see it* — and a turn handed an
//! indicator of its own would put two turns on one machine on two lists. A
//! machine that could run a turn without one would make law 1's surface
//! optional equipment.
//!
//! What is **not** held here is `alo_models::SourcePolicy`. That is a rule an
//! organisation sets and can tighten while a turn is open, so it is passed to
//! the door at the moment a question is asked — exactly as the grants are, and
//! for item 3's reason: the rule that counts is the one in force now, not the
//! one that was in force when somebody pressed a key.
//!
//! # It holds no grants and no clock
//!
//! The grants are the machine's, and they are passed to each door rather than
//! held here: a grant revoked while a turn is open takes effect at the next
//! question asked of them (item 3), and a list borrowed once at the start of a
//! turn would be a list that could not change under it. Nothing here reads a
//! clock either, as everywhere else in this workspace.

use alo_applications::Installed;
use alo_capability::Verbs;
use alo_egress::Indicator;
use alo_files::{Resolving, file_verbs};
use alo_record::Entry;
use alo_software::{Bound, Tool};
use alo_strings::{Said, Strings};

use crate::bounding::Bounding;
use crate::kept::Kept;
use crate::shortening::{Shortened, Shortening};
use crate::the_door_the_compositor_serves::TheDoorTheCompositorServes;
use crate::the_verbs_would_not_declare::TheVerbsWouldNotDeclare;

/// The verbs, the words, the resolver, the boundary, the indicator and the
/// record: one machine's, shared by every turn on it.
///
/// Not `Clone`, and it holds the boundary, the indicator and the record by
/// exclusive borrow: two of these would be two turns writing into one record
/// with no order between them, and the record is a file whose lines are read as
/// a sequence.
pub struct Machine<'a> {
    /// What an agent may ask for, which is what this machine can carry out.
    verbs: Verbs,
    /// The words the person in front of this machine reads.
    strings: &'a Strings,
    /// Where a path really leads.
    resolving: &'a dyn Resolving,
    /// The rented tool an installation reaches, on a machine that can install.
    ///
    /// Taken rather than assumed for `resolving`'s reason: what installs an
    /// application is a program at a path, so a crate that named it could not
    /// be built or tested anywhere that program is not.
    ///
    /// **`None` and the install verb are absent together**, which is this
    /// file's rule rather than an exception to it. A machine built by
    /// [`Machine::carrying_out_file_verbs`] has no tool *and does not offer the
    /// verb*, so the two lists are still one list; a machine built by
    /// [`Machine::carrying_out_file_and_software_verbs`] has both. There is no
    /// constructor that produces one without the other, which is why the one
    /// unreachable case is the machine contradicting itself rather than
    /// something a caller can arrange.
    installing_with: Option<&'a dyn Tool>,
    /// Whether an organisation said which places this machine may install from.
    ///
    /// `Bound::Nobodys` on a machine that cannot install at all, which is what
    /// *nobody set a rule* means and is the only honest value where there is
    /// nothing to bound.
    ///
    /// Held here rather than passed to the door, which is the opposite of
    /// `alo_models::SourcePolicy` one paragraph down, and the difference is
    /// where it comes from: this is `[applications]` in the description a root
    /// wrote, read once when the daemon starts, and `alo-agentd` has no road
    /// that changes it while a turn is open. The *places* it bounds are asked
    /// of the tool at every installation, which is where a place switched off a
    /// moment ago takes effect.
    bound: Bound,
    /// What a verb's work runs inside.
    ///
    /// Exclusive, because imposing one is a thing that happens to a kernel: an
    /// entry is written into a map, a thread goes into a control group and
    /// comes back out of it, and two turns doing that at once through one
    /// boundary would be two threads in one turn's cgroup.
    bounding: &'a mut dyn Bounding,
    /// What is leaving this machine right now.
    indicator: &'a mut Indicator,
    /// Where what happened is written down, and the one thing that can shorten
    /// it.
    ///
    /// The larger of the two traits, because a machine has a record and a turn
    /// only writes to one: [`Machine::kept`] hands a turn the smaller one, and
    /// [`Machine::shorten`] is the door the other half goes through.
    kept: &'a mut dyn Shortening,
    /// The applications this machine has, for an application verb to be held to.
    ///
    /// **Empty on a machine that offers no application verb**, which is the
    /// same rule as `installing_with` above and not an exception to it: a list
    /// nothing can ask about is an honest empty list, and the three verbs and a
    /// real list arrive together or not at all.
    ///
    /// Read once when the daemon starts, like `bound`. An application installed
    /// a moment ago is not in it, and the answer to that is the compositor's
    /// own — `alo_applications::Reaching`'s header says so: *nor can it close
    /// the gap between an application being found and being reached*.
    installed: Installed,
    /// The compositor's door, on a machine where one is running.
    ///
    /// **`None` and the three verbs are absent together**, as with the tool.
    /// `None` on a machine built any other way, and a machine at the sign-in
    /// screen with no compositor is the ordinary case rather than an error —
    /// `crate::the_door_the_compositor_serves` carries that argument.
    reaching_the_compositor: Option<&'a mut dyn TheDoorTheCompositorServes>,
}

/// What a machine needs to carry out the installation verb.
///
/// **A pair rather than two arguments**, because
/// [`Machine::carrying_out_file_software_and_application_verbs`] would
/// otherwise take nine and clippy's bound is seven — and the bound is right
/// here rather than inconvenient: nine positional arguments of which four are
/// `&mut dyn` is a call nobody can read at the call site.
///
/// Grouped by *which family needs them* rather than by type, so the grouping
/// says something true. `Machine::carrying_out_file_and_software_verbs` still
/// takes its two separately and is unchanged: it has seven and does not need
/// this.
pub struct WhatASoftwareVerbNeeds<'a> {
    /// The rented tool an installation reaches.
    pub tool: &'a dyn Tool,
    /// Whether an organisation said which places this machine may install from.
    pub bound: Bound,
}

/// What a machine needs to carry out the three application acts.
///
/// [`WhatASoftwareVerbNeeds`]' twin, for the same reason and one more: the
/// door is `&mut`, because one line in and one line out on one socket is not
/// a thing two turns may do at once.
pub struct WhatAnApplicationVerbNeeds<'a> {
    /// The applications this machine has, read once when the daemon starts.
    pub installed: Installed,
    /// The compositor's door.
    ///
    /// Not an [`Option`]: a caller with no compositor builds a machine that
    /// offers no application verb, which is this file's rule that the list an
    /// agent may ask from is the list the machine can carry out. The `None`
    /// inside lives on the field, where it means *this machine does not offer
    /// them*.
    pub door: &'a mut dyn TheDoorTheCompositorServes,
}

/// Every verb a whole machine offers, as one list.
///
/// **One list with two readers**, and that is the point of it existing at all.
/// `Machine::carrying_out_file_software_and_application_verbs` builds the list a
/// daemon can carry out; an **agent in another process** needs the same list to
/// know what it may ask for, and it cannot ask the daemon — `FromAnAgent` has
/// three messages and none of them is *what may I ask for*, because
/// `docs/contracts/agent-verbs.md` **is** the answer: *a closed list; if a
/// capability is not written here, `alo-agentd` does not have it.*
///
/// So the list is read from a document by whoever writes an adapter, and from
/// this function by anything in this workspace. Two places *computing* it is the
/// fault this replaces: the three families could not drift in their contents,
/// because both would call the same declaring functions — but they could drift
/// in **which families are included**, and that drift has exactly one shape.
/// A verb the machine carries out and the agent is never told about is a
/// capability offered to nobody, which is the fault this repository has now met
/// four times.
///
/// The other direction is harmless and is why this is not a security boundary:
/// an agent told about a verb the machine does not have asks for it and is
/// refused, in words, with the refusal written down.
///
/// # Errors
/// [`TheVerbsWouldNotDeclare`], naming the family that refused — including two
/// families having declared one name, which is what this `Result` is really
/// for. The ten as they are written cannot cause it.
pub fn the_verbs_a_whole_machine_offers() -> Result<Verbs, TheVerbsWouldNotDeclare> {
    let mut verbs = file_verbs()?;
    alo_software::verbs::declare_into(&mut verbs)?;
    alo_applications::verbs::declare_the_three_with_a_road_into(&mut verbs)?;
    Ok(verbs)
}

impl<'a> Machine<'a> {
    /// A machine that can carry out the six file verbs, and offers those.
    ///
    /// No tool and no installation: a machine made this way answers an agent
    /// with six verbs, which is the whole list of what it can do. It is what
    /// every surface that has nothing to do with software is built from — a
    /// compositor drawing an approval, a corridor proving a verb crossed, a
    /// test about a boundary — and keeping it is what stops *this machine can
    /// install* from being a thing seventy-six callers have to answer.
    ///
    /// # Errors
    /// [`TheVerbsWouldNotDeclare`], which the six as they are written cannot cause. It is a
    /// `Result` rather than an unwrap for the reason `alo-files` gives: a
    /// library that panics inside the daemon takes the daemon with it.
    pub fn carrying_out_file_verbs(
        strings: &'a Strings,
        resolving: &'a dyn Resolving,
        bounding: &'a mut dyn Bounding,
        indicator: &'a mut Indicator,
        kept: &'a mut dyn Shortening,
    ) -> Result<Self, TheVerbsWouldNotDeclare> {
        Ok(Self {
            verbs: file_verbs()?,
            strings,
            resolving,
            installing_with: None,
            installed: Installed::default(),
            reaching_the_compositor: None,
            bound: Bound::Nobodys,
            bounding,
            indicator,
            kept,
        })
    }

    /// A machine that can carry out the six file verbs and one installation,
    /// and offers exactly those.
    ///
    /// The resolver is taken rather than assumed so that the decisions in this
    /// crate are testable on a machine where making a symbolic link needs a
    /// privilege — `alo_files::OnThisMachine` is the one that ships, and
    /// `alo_files::Resolving` says why there is only one. The tool is taken for
    /// the same reason one step further: installing reaches a program at a
    /// path, and `alo_software::TheRentedTool` is the one that ships.
    ///
    /// The bound is taken because it is somebody else's rule and this crate
    /// does not read files: `alo-agentd` reads `[applications]` out of the
    /// description a root wrote and hands it over as it comes back, which is
    /// what makes it an organisation's rather than ours.
    ///
    /// The boundary is taken for the reason the record is, and it is the
    /// stronger half of the same argument: what imposes one is a kernel, this
    /// crate is portable, and a machine that could be made without one would be
    /// a machine able to run a verb with nothing around it.
    ///
    /// The indicator is taken for a different reason again: it is the surface a
    /// person watches, so there is exactly one of it and the shell that draws
    /// it is the thing that owns it. A machine borrows it, and
    /// [`Machine::showing`] is how the shell reads it back.
    ///
    /// # Errors
    /// [`TheVerbsWouldNotDeclare`], naming the family that refused. The seven as
    /// they are written cannot cause it. It is a
    /// `Result` rather than an unwrap for the reason `alo-files` gives: a
    /// library that panics inside the daemon takes the daemon with it. It can
    /// also report two families having declared the same verb name, which is a
    /// thing that becomes possible the moment there is more than one of them.
    pub fn carrying_out_file_and_software_verbs(
        strings: &'a Strings,
        resolving: &'a dyn Resolving,
        installing_with: &'a dyn Tool,
        bound: Bound,
        bounding: &'a mut dyn Bounding,
        indicator: &'a mut Indicator,
        kept: &'a mut dyn Shortening,
    ) -> Result<Self, TheVerbsWouldNotDeclare> {
        let mut verbs = file_verbs()?;
        alo_software::verbs::declare_into(&mut verbs)?;
        Ok(Self {
            verbs,
            strings,
            resolving,
            installing_with: Some(installing_with),
            installed: Installed::default(),
            reaching_the_compositor: None,
            bound,
            bounding,
            indicator,
            kept,
        })
    }

    /// The tool, the organisation's rule and the indicator at once.
    ///
    /// `pub(crate)`, and the only caller is `installing_an_application.rs`.
    /// Three fields of one machine, handed out together for the reason
    /// [`Machine::bounding_and_indicator`] hands out two: an installation needs
    /// the tool to reach, the rule to be held to and the indicator to be shown
    /// on, and asking for them one at a time would be three borrows of one
    /// machine. This is the one place that knows they are different fields.
    ///
    /// The bound is cloned because `alo_software::Enabled::read` takes it by
    /// value, and it is a word or two rather than a list.
    pub(crate) fn installing_and_showing(
        &mut self,
    ) -> Option<(&'a dyn Tool, Bound, &mut Indicator)> {
        let tool = self.installing_with?;
        Some((tool, self.bound.clone(), self.indicator))
    }

    /// Every verb this machine can carry out: the six, the installation, and
    /// the three application acts that have a road.
    ///
    /// # Three of the four, and the fourth is not offered
    ///
    /// `alo_applications::verbs::declare_the_three_with_a_road_into` is used
    /// rather than `declare_into`, so `arrange_application` is **absent** from
    /// what an agent is shown. That function carries the whole argument; the
    /// short form is that `alo-instructing` shows a model *these are the only
    /// verbs there are* from exactly this list, and a verb on it with nothing
    /// behind it is one a model proposes, a person approves, and nothing then
    /// does. **An approval that does nothing is the worst outcome available
    /// here.**
    ///
    /// This file's rule therefore still holds: *the list an agent may ask from
    /// is the list this machine can carry out.*
    ///
    /// # What is taken, and why each one
    ///
    /// `installed` is the applications this machine has, read once when the
    /// daemon starts, which `alo_applications::Reaching` asks after the grants
    /// and never before. `reaching_the_compositor` is the door — exclusive,
    /// because one line in and one line out on one socket is not a thing two
    /// turns may do at once.
    ///
    /// # Errors
    /// [`TheVerbsWouldNotDeclare`], naming the family that refused. The ten as
    /// they are written cannot cause it. It can also report two families having
    /// declared one verb name, which is what this `Result` is really for.
    pub fn carrying_out_file_software_and_application_verbs(
        strings: &'a Strings,
        resolving: &'a dyn Resolving,
        software: WhatASoftwareVerbNeeds<'a>,
        applications: WhatAnApplicationVerbNeeds<'a>,
        bounding: &'a mut dyn Bounding,
        indicator: &'a mut Indicator,
        kept: &'a mut dyn Shortening,
    ) -> Result<Self, TheVerbsWouldNotDeclare> {
        // **The same list an agent is told about**, read from one place so the
        // two cannot drift in which families they include.
        let verbs = the_verbs_a_whole_machine_offers()?;
        Ok(Self {
            verbs,
            strings,
            resolving,
            installing_with: Some(software.tool),
            installed: applications.installed,
            reaching_the_compositor: Some(applications.door),
            bound: software.bound,
            bounding,
            indicator,
            kept,
        })
    }

    /// The applications this machine has.
    ///
    /// `pub(crate)`, and the only caller is `acting_on_an_application.rs`.
    /// Empty on a machine that offers no application verb, which is what the
    /// field says and is honest rather than absent: nothing can ask about it.
    pub(crate) fn the_applications_installed(&self) -> &Installed {
        &self.installed
    }

    /// The compositor's door, on a machine where one is running.
    ///
    /// `pub(crate)`, and the only caller is `acting_on_an_application.rs`.
    /// [`None`] is *no compositor is serving one*, which is an answer a person
    /// is told rather than a fault — a machine at the sign-in screen is in
    /// exactly that state.
    pub(crate) fn at_the_compositors_door(
        &mut self,
    ) -> Option<&mut dyn TheDoorTheCompositorServes> {
        self.reaching_the_compositor
            .as_deref_mut()
            .map(|door| door as &mut dyn TheDoorTheCompositorServes)
    }

    /// What an agent may ask for on this machine.
    ///
    /// A shell reads this to show a person what their agent can do, and it is
    /// the whole answer: there is no second list of things that are permitted
    /// but not offered.
    #[must_use]
    pub fn verbs(&self) -> &Verbs {
        &self.verbs
    }

    /// The words the person in front of this machine reads.
    ///
    /// Answers for as long as the words themselves live rather than for as
    /// long as this borrow does, so a caller can hold the words and then ask
    /// the machine for something else — which is what a turn does between
    /// wording a refusal and writing it down.
    #[must_use]
    pub fn strings(&self) -> &'a Strings {
        self.strings
    }

    /// What is leaving this machine right now.
    ///
    /// What a shell draws law 1's indicator from. Borrowed rather than lent
    /// mutably: only a turn, and alo OS on its own errands, put anything on it.
    #[must_use]
    pub fn showing(&self) -> &Indicator {
        self.indicator
    }

    /// Where a path really leads.
    ///
    /// Answers for as long as the resolver lives rather than for as long as
    /// this borrow does, as [`Machine::strings`] does: a turn resolves a call's
    /// paths and then asks the machine for the boundary to run it inside, and
    /// those are two things it holds at once.
    pub(crate) fn resolving(&self) -> &'a dyn Resolving {
        self.resolving
    }

    /// What a verb's work runs inside.
    ///
    /// `pub(crate)`, and the only caller is `carrying.rs`: a boundary is
    /// put around an execution and around nothing else, and a public door onto
    /// it would be a way to run something in a turn's cgroup that no
    /// authorisation reached.
    pub(crate) fn bounding(&mut self) -> &mut dyn Bounding {
        self.bounding
    }

    /// The boundary and the indicator at once.
    ///
    /// Two fields of one machine, handed out together because putting a
    /// question inside a boundary needs both: the boundary to enter, and the
    /// indicator for the work that runs in there. Asking for them one at a time
    /// would be two borrows of one machine, and this is the one place that
    /// knows they are different fields.
    pub(crate) fn bounding_and_indicator(&mut self) -> (&mut dyn Bounding, &mut Indicator) {
        (self.bounding, self.indicator)
    }

    /// What is leaving, to be shown something else.
    ///
    /// `pub(crate)`: the only callers are the doors that put a question
    /// somewhere, and they hand it straight to `alo_egress::Indicator::beginning`
    /// by way of `alo-asking`. A public one would be a way to take a line off
    /// the indicator without the connection it belongs to having ended.
    pub(crate) fn indicator(&mut self) -> &mut Indicator {
        self.indicator
    }

    /// Where what happened is written down.
    ///
    /// `pub(crate)`, and the smaller of the two traits on purpose: what a turn
    /// is handed can write an entry and has no way to remove one.
    pub(crate) fn kept(&mut self) -> &mut dyn Kept {
        self.kept
    }

    /// Write down that this machine was asked to read the person's grants
    /// again and did not.
    ///
    /// Public, and the caller is the service that holds the machine
    /// (`alo-agentd`) rather than anything inside a turn — a grant is made and
    /// revoked on the person's side, and the message that says so may arrive
    /// with no turn under way at all. [`crate::Turning`] has the same door for
    /// the rounds where one is.
    ///
    /// **Nothing a caller passes in can name an authority.** The one argument is
    /// the sentence the person was shown, already rendered, so what they read
    /// and what is written down are one value; there is no agent, no verb, no
    /// grant and no path, because none was involved. That is what makes a public
    /// door onto the record safe here where a general one would not be — see
    /// [`alo_record::Entry::the_grants_were_not_read_again`].
    ///
    /// # Errors
    ///
    /// [`alo_keeping::NotKept`] when the record could not be written. A service
    /// that meets this has stopped keeping evidence and stops, exactly as it
    /// does for a turn that could not write one.
    pub fn the_grants_were_not_read_again(
        &mut self,
        why: &Said,
        now: std::time::SystemTime,
    ) -> Result<(), alo_keeping::NotKept> {
        self.kept
            .keep(Entry::the_grants_were_not_read_again(why.text(), now))
    }

    /// Write down that a pairing with `with` was kept on this machine.
    ///
    /// Public, and the caller is the service that holds the machine and the
    /// pairings (`alo-agentd`) rather than anything inside a turn: a pairing
    /// is completed by the second of two people's confirmations arriving on
    /// the wire, which may happen with no turn under way at all.
    /// [`crate::Arriving`] has the same door for the rounds where a remote
    /// turn holds the machine.
    ///
    /// **Nothing a caller passes in can name an authority.** The one argument
    /// is the other machine's identity as the pairing spells it — not a
    /// grantee, not a person's name — and the entry names no agent, because
    /// two people made this and no agent did. That is what makes a public
    /// door onto the record safe here, as it is for
    /// [`Machine::the_grants_were_not_read_again`]; see
    /// [`alo_record::Entry::paired`].
    ///
    /// # Errors
    ///
    /// [`alo_keeping::NotKept`] when the record could not be written. A
    /// service that meets this has stopped keeping evidence and stops.
    pub fn a_pairing_was_kept(
        &mut self,
        with: &str,
        now: std::time::SystemTime,
    ) -> Result<(), alo_keeping::NotKept> {
        self.kept.keep(Entry::paired(with, now))
    }

    /// Write down that the person opened a workspace discovery found, with
    /// the identity they named and the address it answered from at that
    /// moment.
    ///
    /// Called by the service that holds this machine, from the person's own
    /// door, before it hands the address to their session — so no turn need
    /// be under way. [`crate::Turning`] and [`crate::Arriving`] have the same
    /// door for the rounds where a turn holds the machine.
    ///
    /// **Nothing a caller passes in can name an authority**, for
    /// [`Machine::a_pairing_was_kept`]'s reason: the arguments are an identity
    /// and an address, and the entry names no agent because no agent can
    /// send the request. See [`alo_record::Entry::a_workspace_was_opened`].
    ///
    /// # Errors
    ///
    /// [`alo_keeping::NotKept`] when the record could not be written. A
    /// service that meets this has stopped keeping evidence and stops.
    pub fn a_workspace_was_opened(
        &mut self,
        workspace: &str,
        answers_at: &str,
        now: std::time::SystemTime,
    ) -> Result<(), alo_keeping::NotKept> {
        self.kept
            .keep(Entry::a_workspace_was_opened(workspace, answers_at, now))
    }

    /// Shorten this machine's record under the rule it is kept by.
    ///
    /// Public, because the caller is the service that holds the machine and
    /// reads the clock (`alo-agentd`, queue item 20) rather than anything
    /// inside a turn. It is a door onto the machine and not onto a turn: a
    /// [`crate::Turning`] hands nobody a `&mut Machine`, so nothing an agent can
    /// ask for arrives here, and what is removed is decided by a rule and a
    /// moment either way.
    ///
    /// # Errors
    ///
    /// [`alo_keeping::NotKept`], in every one of which nothing has been
    /// removed — see [`crate::Shortening::shorten`].
    pub fn shorten(
        &mut self,
        keeping: alo_keeping::Keeping,
        now: std::time::SystemTime,
    ) -> Result<Shortened, alo_keeping::NotKept> {
        self.kept.shorten(keeping, now)
    }
}

impl std::fmt::Debug for Machine<'_> {
    /// Written by hand because neither the resolver nor the record is a value
    /// that can be printed, and because what a reader wants here is what this
    /// machine offers rather than the addresses of two trait objects.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Machine")
            .field("verbs", &self.verbs.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
// Counted: 3 `unwrap` and 2 `expect`, no `panic!` or `unreachable!`, so both are
// listed and nothing else is — an unfulfilled expectation is an error here.
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {

    use super::*;
    use crate::testing::{NothingIsBounded, in_english};
    use alo_capability::Verb;
    use alo_files::OnThisMachine;
    use alo_record::Record;

    /// **Every verb this machine offers, by name.**
    ///
    /// # This test compared the two readers with each other and proved nothing
    ///
    /// The first version built a `Machine` the ten-verb way, called
    /// [`the_verbs_a_whole_machine_offers`], and asserted the two lists matched
    /// by name and by count. It read well and it was **a tautology**: the
    /// constructor calls that same function, so both sides were one list
    /// compared with itself. Deleting the application family from the function
    /// left the test green — which is how it was found, and the second time in
    /// one day that a test of mine compared a thing with itself.
    ///
    /// **One function with two readers cannot drift, which is the point of it
    /// and the reason a test of their agreement is worthless.** What can still
    /// go wrong is the function itself losing a family — and the only thing
    /// that catches that is naming what must be there.
    ///
    /// # Why the names and not a count
    ///
    /// A count passes when one family replaces another. These are the ten in
    /// `docs/contracts/agent-verbs.md`, which is the published list a third
    /// party builds an adapter against: *a closed list; if a capability is not
    /// written here, `alo-agentd` does not have it.* So this is the contract
    /// held against the code, in the direction that matters — a verb the
    /// contract promises and the machine does not offer is a capability offered
    /// to nobody, which this repository has met four times.
    ///
    /// **`arrange_application` is absent on purpose** and is asserted absent:
    /// it has no road (ADR 0108), so it is not declared, and an approval for a
    /// verb with nothing behind it is worse than the capability missing.
    #[test]
    fn every_verb_this_machine_offers_is_named_here_and_no_other() {
        let offered = the_verbs_a_whole_machine_offers().expect("the eleven verbs declare");
        let told: Vec<&str> = offered.all().map(alo_capability::Verb::name).collect();

        let the_eleven = [
            // `alo-files`' six.
            "list_folder",
            "read_file",
            "find_in_folder",
            "rename_file",
            "move_file",
            "archive_folder",
            // `alo-software`'s two: the installation a person approves and the
            // read that answers inside the turn.
            "install_application",
            "what_can_be_installed",
            // `alo-applications`' three with a road.
            "open_application",
            "focus_application",
            "close_application",
        ];
        for name in the_eleven {
            assert!(
                told.contains(&name),
                "{name} is in docs/contracts/agent-verbs.md and this machine does not offer it, \
                 which is a capability offered to nobody: {told:?}"
            );
        }
        assert_eq!(
            told.len(),
            the_eleven.len(),
            "this machine offers {} verbs and {} are named here: {told:?}",
            told.len(),
            the_eleven.len()
        );
        assert!(
            !told.contains(&"arrange_application"),
            "a verb with no road is offered to an agent: {told:?}"
        );
    }

    /// **The offered list is the list this machine can carry out**, which is
    /// the six file verbs and nothing else. An application verb is declared,
    /// portable and unreachable here, because what moves a window is not in
    /// this repository.
    #[test]
    fn a_machine_offers_exactly_what_it_can_carry_out() {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();

        let names: Vec<_> = machine.verbs().all().map(Verb::name).collect();
        assert_eq!(
            names,
            [
                "list_folder",
                "read_file",
                "find_in_folder",
                "rename_file",
                "move_file",
                "archive_folder"
            ]
        );
        assert!(
            machine.verbs().of("open_application").is_none(),
            "a verb nothing here can carry out is on the list an agent asks from"
        );
    }

    /// What a reader of a debug line wants is what this machine offers, not two
    /// trait objects it cannot print.
    #[test]
    fn what_a_machine_prints_is_what_it_offers() {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        assert_eq!(format!("{machine:?}"), "Machine { verbs: 6, .. }");
    }

    /// **A machine that has done nothing is showing nothing**, and the shell
    /// reads that off the machine rather than keeping a second copy of law 1's
    /// surface beside it.
    #[test]
    fn a_machine_that_has_caused_nothing_to_leave_is_quiet() {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        assert!(machine.showing().is_quiet());
        assert!(machine.showing().showing().is_empty());
    }
}
