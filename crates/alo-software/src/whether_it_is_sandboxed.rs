//! Whether an application was installed sandboxed, which is what it is unless a person
//! deliberately said otherwise.
//!
//! Task 15 of `docs/autonomy/software-and-the-web-plan.md`, against
//! `docs/features.md`: *Unsandboxed installation as a deliberate, clearly-marked act —
//! never the default, and forbiddable by policy on a managed machine.*
//!
//! # Sandboxed is what a caller gets by saying nothing, and that is the guarantee
//!
//! [`Wanted::by_hand`](crate::installing::Wanted::by_hand) means
//! [`Sandboxing::Sandboxed`]. An unsandboxed install is only reachable through
//! [`Wanted::unsandboxed_by_hand`](crate::installing::Wanted::unsandboxed_by_hand), a
//! constructor a caller has to name.
//! So *never the default* is held by there being no way to express it by omission, rather
//! than by a check somewhere that could be forgotten.
//!
//! # Why this is not `alo_put_aside::whether_it_is_private::Privacy`'s shape
//!
//! That enum has **no default and is a required argument**, and its own header gives the
//! reason: *a default of ordinary is the wrong default for privacy — the caller who forgets
//! gets a leaked title, and forgot is the commonest thing a caller does.*
//!
//! **The direction of danger is inverted here, and that is the whole of the difference.**
//! Forgetting `Privacy` produces the unsafe value. Forgetting this one produces the **safe**
//! value, because the dangerous state is the one a caller can only reach by writing its name.
//! So forcing every one of fifteen call sites in five files — one of them in another crate
//! (`alo-agentd`) —
//! to restate *sandboxed* would buy no guarantee that this does not already have.
//!
//! Written down because somebody meeting both enums will reasonably ask why they disagree,
//! and *the other crate did it differently* is the answer that gets one of them changed to
//! match the wrong model.
//!
//! # It is carried, never derived
//!
//! [`Wanted`](crate::installing::Wanted) holds it and
//! [`Installed`](crate::installing::Installed) keeps it, so a surface marking an unsandboxed
//! application is **handed the fact**. Nothing infers it from the source, the identifier or a
//! setting read at install time: an application is unsandboxed because a person said so on
//! this installation, and that sentence has one place it can come from.

/// Whether an installation is sandboxed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sandboxing {
    /// Sandboxed: what every installation is unless a person asked otherwise.
    Sandboxed,
    /// Unsandboxed, because a person deliberately chose it for this installation.
    ///
    /// A surface showing an application in this state marks it. The words are
    /// [`crate::words::INSTALLED_UNSANDBOXED`], so the marking is one sentence in every
    /// language rather than a flag each surface words for itself.
    Unsandboxed,
}

impl Sandboxing {
    /// Whether this one needs marking to a person.
    ///
    /// A question rather than a comparison, so a surface does not hold its own copy of
    /// *which of the two is the one worth saying* — the same reason
    /// `alo_put_aside::a_place_groups_its_windows` hands a drawer `Headings` instead of a
    /// count.
    #[must_use]
    pub const fn is_worth_marking(self) -> bool {
        matches!(self, Self::Unsandboxed)
    }
}
