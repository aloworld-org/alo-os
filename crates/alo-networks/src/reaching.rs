//! **How far this machine reaches, and whether the way out is somebody's
//! meter** — the network manager's own two answers, and nothing inferred from
//! them.
//!
//! [`crate::reported::TheNetworks`] is what there is to join: the networks the
//! radio can see, the ones this machine has saved, and which of them it is
//! sending through. That is a different question from **whether anything is
//! reached by sending through it**, which is the one an application asks the
//! network monitor portal, and the one a status area shows. A machine joined to
//! a café's access point that has not been paid for has a primary connection
//! and reaches nothing.
//!
//! # Neither answer is worked out here
//!
//! NetworkManager already decides both, by walking its own connectivity check
//! and by reading what the connection said about itself, and ADR 0011 rents it
//! rather than repeating it. So both are read as it reports them and mapped
//! into words, and **no third state is calculated from the pair**: a caller
//! that wants *connected* asks [`HowFar::reaches_anything`] and gets this
//! crate's one answer to it, rather than each caller inventing its own.
//!
//! # What is not known is said, never rounded to what is convenient
//!
//! Both of NetworkManager's properties have a value meaning *it has not worked
//! that out*, and both keep it here — [`HowFar::NotSaid`] and
//! [`Metered::NotSaid`]. Rounding either to the cheerful end would be this
//! crate telling a person they are online, or that their connection is free,
//! on no evidence. A machine whose network manager is still starting up reports
//! exactly that, and a surface above can say so or wait.

/// How far this machine reaches through the connection it is sending on.
///
/// `NM_CONNECTIVITY_*`, which NetworkManager decides by its own connectivity
/// check and publishes as `Connectivity` on its own object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HowFar {
    /// The network manager has not worked out how far this machine reaches —
    /// `NM_CONNECTIVITY_UNKNOWN`, which is what it says while it is still
    /// starting up and when its connectivity check is switched off.
    NotSaid,
    /// Nothing is reached: the machine is not sending anywhere —
    /// `NM_CONNECTIVITY_NONE`.
    Nowhere,
    /// Something answered, but it was a sign-in page rather than what was asked
    /// for — `NM_CONNECTIVITY_PORTAL`. The café, the hotel and the airport.
    APageInTheWay,
    /// The machine reaches the network it is on, and not past it —
    /// `NM_CONNECTIVITY_LIMITED`.
    OnlyThisNetwork,
    /// The connectivity check was answered by what it asked —
    /// `NM_CONNECTIVITY_FULL`.
    AllOfIt,
}

impl HowFar {
    /// Every answer, in the order NetworkManager numbers them.
    pub const EVERY: [Self; 5] = [
        Self::NotSaid,
        Self::Nowhere,
        Self::APageInTheWay,
        Self::OnlyThisNetwork,
        Self::AllOfIt,
    ];

    /// How far the network manager said, from the number it published.
    ///
    /// A number it has never published is [`HowFar::NotSaid`], because a
    /// service that grew a sixth answer has told us something we do not
    /// understand, and the honest reading of what is not understood is that
    /// nothing was said — never the reachable end of the range.
    #[must_use]
    pub const fn reported(connectivity: u32) -> Self {
        match connectivity {
            1 => Self::Nowhere,
            2 => Self::APageInTheWay,
            3 => Self::OnlyThisNetwork,
            4 => Self::AllOfIt,
            _ => Self::NotSaid,
        }
    }

    /// **Whether anything at all is reached through this connection.**
    ///
    /// The one place that question is answered, so that a status area, a
    /// portal and whatever asks next cannot hold three different views of what
    /// *connected* means.
    ///
    /// A sign-in page in the way counts as reaching something: an application
    /// that wants to show one, and a person who wants to get past it, both
    /// need the connection to be used. What is not reached past that page is
    /// [`HowFar::APageInTheWay`]'s own business to say, and a caller that
    /// cares asks for it by name.
    ///
    /// Nothing worked out yet is **not** something reached. A machine that has
    /// not been told is not a machine that is online.
    #[must_use]
    pub const fn reaches_anything(self) -> bool {
        matches!(
            self,
            Self::APageInTheWay | Self::OnlyThisNetwork | Self::AllOfIt
        )
    }
}

/// Whether what this machine is sending through is somebody's meter.
///
/// `NM_METERED_*`, published as `Metered` on the network manager's own object.
/// NetworkManager knows this two ways — because a person marked the connection,
/// or because it guessed from what kind of connection it is — and keeps the two
/// apart. So does this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metered {
    /// The network manager has not worked it out — `NM_METERED_UNKNOWN`.
    NotSaid,
    /// It is metered, and somebody said so — `NM_METERED_YES`.
    Metered,
    /// It is not metered, and somebody said so — `NM_METERED_NO`.
    Unmetered,
    /// The network manager guessed it is metered — `NM_METERED_GUESS_YES`.
    ProbablyMetered,
    /// The network manager guessed it is not — `NM_METERED_GUESS_NO`.
    ProbablyUnmetered,
}

impl Metered {
    /// Every answer, in the order NetworkManager numbers them.
    pub const EVERY: [Self; 5] = [
        Self::NotSaid,
        Self::Metered,
        Self::Unmetered,
        Self::ProbablyMetered,
        Self::ProbablyUnmetered,
    ];

    /// What the network manager said, from the number it published.
    ///
    /// A number it has never published is [`Metered::NotSaid`], for the reason
    /// [`HowFar::reported`] gives.
    #[must_use]
    pub const fn reported(metered: u32) -> Self {
        match metered {
            1 => Self::Metered,
            2 => Self::Unmetered,
            3 => Self::ProbablyMetered,
            4 => Self::ProbablyUnmetered,
            _ => Self::NotSaid,
        }
    }

    /// **Whether something about to spend somebody's data should hold off.**
    ///
    /// A guess that it is metered counts, and so does nothing said. This is the
    /// one question in this crate answered the careful way round, and
    /// deliberately: being wrong here costs a person money on their own
    /// connection, and being wrong the other way costs an update that waits
    /// until the machine is somewhere cheaper.
    #[must_use]
    pub const fn should_hold_off(self) -> bool {
        matches!(self, Self::Metered | Self::ProbablyMetered | Self::NotSaid)
    }
}

/// **What the network manager says about the way out, at one moment.**
///
/// The two answers together, because they are read together and a caller that
/// had to ask twice could be told about two different moments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reaching {
    /// How far this machine reaches.
    pub how_far: HowFar,
    /// Whether the way out is somebody's meter.
    pub metered: Metered,
}

impl Reaching {
    /// What was reported at one moment.
    #[must_use]
    pub const fn reported(how_far: HowFar, metered: Metered) -> Self {
        Self { how_far, metered }
    }

    /// What a machine whose network manager has said nothing reaches: nothing
    /// known, either way.
    ///
    /// Not a default anybody falls back to — a reading that could not be taken
    /// is [`crate::NotAnswering`], never this — but the honest reading of a
    /// service that is up and has not decided yet.
    #[must_use]
    pub const fn nothing_said() -> Self {
        Self::reported(HowFar::NotSaid, Metered::NotSaid)
    }
}

/// **What is reached now**, asked of the network manager.
///
/// Apart from [`crate::Networks`] because it is a different question and a
/// different caller: what there is to join is the person's Settings and the
/// broker's business, and how far the machine reaches is the network monitor
/// portal's and the status area's. A machine that only shows a network icon
/// implements this and never the other.
pub trait WhatIsReached {
    /// How far this machine reaches at this moment, and whether it is metered.
    ///
    /// # Errors
    /// [`crate::NotAnswering`] where the network manager could not be asked.
    /// Never a cheerful reading standing in for a question that got no answer.
    fn reaching_now(&self) -> Result<Reaching, crate::NotAnswering>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every number the network manager publishes is the answer it means**,
    /// and each is its own.
    #[test]
    fn each_number_the_network_manager_publishes_is_its_own_answer() {
        let read: Vec<HowFar> = (0..5).map(HowFar::reported).collect();
        assert_eq!(read, HowFar::EVERY);
        let metered: Vec<Metered> = (0..5).map(Metered::reported).collect();
        assert_eq!(metered, Metered::EVERY);
    }

    /// **A number this machine has never seen is nothing said**, at both ends
    /// and in both properties — never rounded to the reachable or the free one.
    #[test]
    fn a_number_we_do_not_understand_is_nothing_said() {
        for number in [5, 6, 99, u32::MAX] {
            assert_eq!(HowFar::reported(number), HowFar::NotSaid, "{number}");
            assert_eq!(Metered::reported(number), Metered::NotSaid, "{number}");
        }
        assert!(!HowFar::reported(u32::MAX).reaches_anything());
    }

    /// **A machine that has not been told is not a machine that is online**,
    /// and neither is one reaching nowhere.
    #[test]
    fn nothing_worked_out_yet_is_not_something_reached() {
        assert!(!HowFar::NotSaid.reaches_anything());
        assert!(!HowFar::Nowhere.reaches_anything());
    }

    /// **A sign-in page in the way is still something reached**, because an
    /// application showing one and a person getting past it both need the
    /// connection used.
    #[test]
    fn a_sign_in_page_in_the_way_is_still_something_reached() {
        assert!(HowFar::APageInTheWay.reaches_anything());
        assert!(HowFar::OnlyThisNetwork.reaches_anything());
        assert!(HowFar::AllOfIt.reaches_anything());
    }

    /// **Holding off is answered the careful way round**: a guess that the
    /// connection is metered, and nothing said at all, both hold off, because
    /// being wrong here spends a person's own money.
    #[test]
    fn what_might_be_metered_holds_off() {
        assert!(Metered::Metered.should_hold_off());
        assert!(Metered::ProbablyMetered.should_hold_off());
        assert!(Metered::NotSaid.should_hold_off());
        assert!(!Metered::Unmetered.should_hold_off());
        assert!(!Metered::ProbablyUnmetered.should_hold_off());
    }

    /// **A machine whose network manager has decided nothing says so in both
    /// answers**, rather than looking like a machine that is offline and free.
    #[test]
    fn a_network_manager_that_has_decided_nothing_says_so_in_both_answers() {
        let nothing = Reaching::nothing_said();
        assert_eq!(nothing.how_far, HowFar::NotSaid);
        assert_eq!(nothing.metered, Metered::NotSaid);
        assert!(!nothing.how_far.reaches_anything());
        assert!(nothing.metered.should_hold_off());
    }
}
