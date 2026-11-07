//! One verb an adapter declares — the verb contract's six fields, and the two
//! the adapter contract adds: how a person does it by hand, and how it is
//! carried out.

use alo_capability::Effect;
use alo_strings::Word;

use crate::adapter_arg::AdapterArg;
use crate::invocation::Invocation;

/// What a verb reaches beyond its application.
///
/// **Every adapter verb reaches its application**, and that grant is asked
/// before anything is sent (`crate::driving`). This says what else it reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reaches {
    /// A grant over each of these path arguments, every one of them.
    Over(&'static [&'static str]),
    /// Nothing but the application — for a verb that takes no path.
    OnlyItsApplication,
}

/// Why an adapter verb that reaches only its application requires no grant of
/// its own, carried in the declaration as `docs/contracts/agent-verbs.md` rule
/// 5 asks. `docs/contracts/app-adapters.md` carries the same sentence.
pub const ONLY_ITS_APPLICATION: &str = "an adapter's verb that takes no path reaches only its own \
     application, and the grant over that application is asked before anything is sent to it";

/// Whether carrying a verb out sends anything off this machine.
///
/// **What a person reads before they type it** (`docs/contracts/terminal-commands.md`,
/// section 4, *Saying so before, not after*): `alo-app show` prints *this
/// cannot be undone once it has been sent* beside every verb that does not say
/// [`Sends::Never`], because Put back restores files and reaches nobody else's
/// computer. Declared by the adapter's author, honestly, as its effect is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sends {
    /// It never sends anything off this machine.
    Never,
    /// It always does: a message, a post, an upload.
    Always,
    /// It may, depending on what it is given or what the application does.
    May,
}

impl Sends {
    /// What the contract calls it.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Always => "always",
            Self::May => "may",
        }
    }
}

/// One verb of an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterVerb {
    /// Its name within the adapter; the machine knows it as `adapter.name`.
    pub name: &'static str,
    /// What it does, in one sentence a person would use.
    pub purpose: Word,
    /// Read or change — honestly (the adapter contract's rule 1).
    pub effect: Effect,
    /// Whether it sends anything off this machine — honestly, as its effect.
    pub sends: Sends,
    /// What it takes. All of them are required.
    pub args: &'static [AdapterArg],
    /// What it reaches beyond its application.
    pub reaches: Reaches,
    /// The sentence a person approves, with a gap for every argument.
    pub sentence: Word,
    /// How a person does the same thing in the application without the agent
    /// (ADR 0009). A verb with none is refused.
    pub by_hand: Option<Word>,
    /// How it is carried out.
    pub carried_out: Invocation,
}
