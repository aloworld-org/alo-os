//! **A command alo runs, counted by the kernel in a control group of its own**
//! — the machine's half of what a turn asks for
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md),
//! as amended 2026-11-13).
//!
//! The owner, 2026-11-13: *"Yes, count alo's commands"* — bytes only, no
//! addresses, no contents, only for the seconds alo's command runs, and only
//! alo's command, never the person's. In order, for one command:
//!
//! ```text
//! 1. a control group made for it, beside this service's turns   (Turns::a_group_for_a_command)
//! 2. a count begun on that group, from nothing                  (Counter::begun)
//! 3. this thread steps in, the command is started, the thread
//!    steps back out — so the command is born inside the group   (Turns::from_inside)
//! 4. the count read on every look while it runs                 (Counter::so_far)
//! 5. when it ends: read once more, the count ended, the group
//!    taken away                                                 (Counter::ended, removed)
//! ```
//!
//! **Only this group is ever named to the counter.** The person's own programs
//! are in their own session's groups, and the applications their own commands
//! count are entries the desktop makes; nothing here touches either, and the
//! group is this service's alone.
//!
//! **A step that fails before the command starts is a machine that cannot
//! count it**, so the command still runs, uncounted and shown as uncounted for
//! the whole run (`alo_turn::Bounding::a_command_counted`), and the reason is
//! said once on the service log. What is never done is start the command
//! outside its group while saying it is counted.
//!
//! **What it cannot see**, said so it is not mistaken for proof: anything the
//! command hands to another service to send for it — a name the machine's
//! resolver looks up, a download a package manager's own daemon makes — is in
//! that service's group, not this one. And something the command leaves
//! running after it ends is not counted after it ends, because the count ends
//! with the command; its group is then left in place, and said so.

use std::io;
use std::process::Child;

use alo_bounding::{ACountedGroup, Counter, Turns};
use alo_turn::ACommandCounted;

/// One command's group, and its count, from before it starts until it ends.
pub struct ItsOwnGroup<'k> {
    /// Where this service's turns are made, and the way back out.
    turns: &'k Turns,
    /// The kernel's counter, opened from its pin when the service started.
    counter: &'k mut Counter,
    /// The group made for this command.
    group: ACountedGroup,
    /// The number the counter knows the group by.
    id: u64,
}

/// A group for the next command, named from `named`, with its count begun —
/// or [`None`], said on the service log, if any step would not be taken.
pub fn a_group_counted<'k>(
    turns: &'k Turns,
    counter: &'k mut Counter,
    named: &mut u64,
) -> Option<ItsOwnGroup<'k>> {
    *named = named.saturating_add(1);
    let name = format!("command-{}-{named}", std::process::id());
    let group = match turns.a_group_for_a_command(&name) {
        Ok(group) => group,
        Err(why) => {
            uncounted(&format!("no control group of its own could be made: {why}"));
            return None;
        }
    };
    let id = match group.id() {
        Ok(id) => id,
        Err(why) => {
            uncounted(&format!("its control group could not be looked at: {why}"));
            taken_away(group);
            return None;
        }
    };
    if let Err(why) = counter.begun(id) {
        uncounted(&format!("the kernel would not begin counting it: {why}"));
        taken_away(group);
        return None;
    }
    Some(ItsOwnGroup {
        turns,
        counter,
        group,
        id,
    })
}

impl ACommandCounted for ItsOwnGroup<'_> {
    fn starting(&mut self, start: &mut dyn FnMut() -> io::Result<Child>) -> io::Result<Child> {
        let (started, not_back) = self
            .turns
            .from_inside(&self.group, start)
            .map_err(|why| io::Error::other(format!("not started inside its own group: {why}")))?;
        if let Some(why) = not_back {
            // The command is running and counted either way; what is wrong is
            // this service's own thread, which is now counted beside it until
            // the group goes. Said, because nothing else would say it.
            eprintln!(
                "alo-agentd: a thread of this service could not come back from the control \
                 group of a command alo runs, so what it sends is counted with the command: {why}"
            );
        }
        started
    }

    fn so_far(&self) -> Option<u64> {
        self.counter.so_far(self.id).ok().flatten()
    }

    fn ended(self: Box<Self>) -> Option<u64> {
        let ItsOwnGroup {
            counter, group, id, ..
        } = *self;
        let sent = counter.so_far(id).ok().flatten();
        if let Err(why) = counter.ended(id) {
            // Nothing here asks again, and the entry is the kernel's until the
            // boundary is loaded afresh at the next boot — said plainly,
            // because a count that outlives its command is the one thing the
            // owner's answer forbids, and it must be findable.
            eprintln!(
                "alo-agentd: the count of a command alo ran could not be ended, so the kernel \
                 keeps counting control group {id} until this machine restarts: {why}"
            );
        }
        taken_away(group);
        sent
    }
}

/// Take a command's group away, or say why it stays.
fn taken_away(group: ACountedGroup) {
    let at = group.at().display().to_string();
    if let Err(why) = group.removed() {
        eprintln!(
            "alo-agentd: the control group of a command alo ran is left at {at}, because \
             something it started is still running there; it is no longer counted: {why}"
        );
    }
}

/// Say, once per command, that it runs uncounted and why.
fn uncounted(why: &str) {
    eprintln!(
        "alo-agentd: a command alo runs will run uncounted, and the indicator says so for the \
         whole run, because {why}"
    );
}
