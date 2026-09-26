//! What a person sees of what their machine is keeping: how far back the
//! agent's changes can be put back, and what that is costing in disk space.
//!
//! # A machine that keeps nothing is a different thing, not a zero
//!
//! [`WhatIsKept`] is an enum rather than a struct with a flag, because ADR
//! 0045's sixth term is that a machine installed without the means to hold an
//! earlier state of a folder *answers not yet on this machine, honestly*. On
//! such a machine there is no window to widen, nothing being held, and no act to
//! offer — and making that a variant means a pane cannot accidentally offer one:
//! [`WhatIsKept::may_forget`] has nothing to return, by the shape of the value
//! rather than by a check somebody has to remember to write.
//!
//! # The size is carried, never worked out
//!
//! ADR 0045's fourth accepted term asks that *what is filling the disk counts
//! snapshots, by name*, and [`Holding::measured`] is how that answer arrives
//! here: `alo_measuring::Node`'s own `size`, and its own account of whether that
//! size is the whole truth. This crate does not walk a folder, does not add up a
//! tree and does not know where what is kept lives. A pane that counted would be
//! a second answer to a question one crate already answers, and the day the two
//! disagreed a person would have no way to tell which was their machine's.
//!
//! That is also why the number goes out of here as the `u64` it came in as:
//! `tests/the_number_shown_is_the_number_measured.rs` holds the pane's number
//! and `alo-measuring`'s to be the same one.

use alo_keeping_up::HowFarBack;
use alo_measuring::{Counted, Node};
use alo_strings::{Filling, Said, Strings};

use crate::forgetting::Offered;
use crate::words;

/// How much room what can be put back is holding — `alo-measuring`'s answer,
/// carried and never recomputed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holding {
    /// The bytes that crate answered with.
    bytes: u64,
    /// Its own account of whether that is the whole truth.
    counted: Counted,
}

/// What a person sees of what their machine is keeping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatIsKept {
    /// This machine keeps what a person's files were, so the agent's changes
    /// can be put back — this far back, holding this much.
    Keeping {
        /// How far back it reaches, as the person's settings decided it.
        window: HowFarBack,
        /// What that is costing.
        holding: Holding,
    },

    /// It keeps nothing, so there is nothing to show and nothing to do.
    NotOnThisMachine,
}

impl Holding {
    /// What `alo-measuring` answered about the folder what is kept lives in.
    ///
    /// The node asked for is the one the caller measured; this crate never
    /// chooses a folder, because a pane that picked its own would be answering
    /// a different question from the one *what is filling the disk* answers.
    #[must_use]
    pub fn measured(node: &Node) -> Self {
        Self {
            bytes: node.size,
            counted: node.counted.clone(),
        }
    }

    /// The bytes it is holding, exactly as `alo-measuring` answered.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Whether that size is the whole truth about it, in that crate's words —
    /// or nothing, when it is.
    ///
    /// A folder the machine would not read answers here rather than passing a
    /// zero off as an amount, which is the difference between *nothing* and
    /// *not known* and is the reason this is carried at all.
    #[must_use]
    pub fn not_the_whole(&self, strings: &Strings) -> Option<Said> {
        self.counted.said(strings)
    }

    /// Whether the size is known to be the whole truth **and** none.
    ///
    /// Not merely zero: a folder that could not be read is zero bytes seen and
    /// an unknown amount held, and telling a person nothing is being kept on
    /// the strength of it would be a sentence the machine cannot stand behind.
    #[must_use]
    pub fn nothing_at_all(&self) -> bool {
        self.bytes == 0 && self.counted == Counted::Whole
    }
}

impl WhatIsKept {
    /// A machine that keeps, with the window its person's settings decided and
    /// what that is holding.
    #[must_use]
    pub const fn keeping(window: HowFarBack, holding: Holding) -> Self {
        Self::Keeping { window, holding }
    }

    /// A machine that keeps nothing of what a person's files were.
    #[must_use]
    pub const fn not_on_this_machine() -> Self {
        Self::NotOnThisMachine
    }

    /// How far back it reaches — or nothing, on a machine that keeps nothing.
    #[must_use]
    pub const fn window(&self) -> Option<HowFarBack> {
        match self {
            Self::Keeping { window, .. } => Some(*window),
            Self::NotOnThisMachine => None,
        }
    }

    /// What it is holding — or nothing, on a machine that keeps nothing.
    #[must_use]
    pub const fn holding(&self) -> Option<&Holding> {
        match self {
            Self::Keeping { holding, .. } => Some(holding),
            Self::NotOnThisMachine => None,
        }
    }

    /// **How far back this machine keeps**, in the language the person reads —
    /// or nothing, on a machine that keeps nothing, which has
    /// [`Self::keeps_nothing`] to say instead.
    #[must_use]
    pub fn how_far_back(&self, strings: &Strings) -> Option<Said> {
        let window = self.window()?;
        Some(
            strings.say(
                &words::HOW_FAR_BACK.key(),
                &Filling::of(words::DAYS, window.days().to_string())
                    .and(words::CHANGES, window.turns().to_string()),
            ),
        )
    }

    /// **How much room that is holding**, in the language the person reads,
    /// given the size already worded the way this language writes one.
    ///
    /// Nothing on a machine that keeps nothing. On a machine holding a known
    /// none, the sentence is that nothing has needed keeping yet — which is an
    /// ordinary state, and the `size` passed in is not used for it.
    #[must_use]
    pub fn holding_said(&self, strings: &Strings, size: &str) -> Option<Said> {
        let holding = self.holding()?;
        if holding.nothing_at_all() {
            return Some(strings.say(&words::HOLDING_NOTHING.key(), &Filling::nothing()));
        }
        Some(strings.say(
            &words::HOLDING.key(),
            &Filling::of(words::SIZE, size.to_owned()),
        ))
    }

    /// **This machine keeps nothing of what your files were** — in
    /// `alo-keeping-up`'s words, because it is the same fact an undo refuses
    /// with and a person should meet one sentence for it however they got
    /// there. Nothing on a machine that keeps.
    #[must_use]
    pub fn keeps_nothing(&self, strings: &Strings) -> Option<Said> {
        match self {
            Self::Keeping { .. } => None,
            Self::NotOnThisMachine => Some(strings.say(
                &alo_keeping_up::words::NOT_UNDONE_NOTHING_KEEPS_WHAT_WAS_THERE.key(),
                &Filling::nothing(),
            )),
        }
    }

    /// **The one act**, offered — or nothing at all on a machine that keeps
    /// nothing, which is the whole reason this returns an [`Option`].
    ///
    /// It is offered whenever this machine keeps, including when it is holding a
    /// known none: a person may reasonably want the machine to stop holding
    /// their past whether or not it is holding any of it this minute, and an act
    /// that appeared and disappeared with a number would be a control nobody
    /// could find twice. It is **never** withheld because the size could not be
    /// read, for the opposite reason — that is the case where a person most
    /// wants the space back.
    #[must_use]
    pub fn may_forget(&self) -> Option<Offered> {
        match self {
            Self::Keeping { .. } => Some(Offered::new()),
            Self::NotOnThisMachine => None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A node as `alo-measuring` would answer with one, of `bytes` and counted
    /// `counted`.
    fn a_node(bytes: u64, counted: Counted) -> Node {
        Node {
            name: "undo".to_owned(),
            at: std::path::PathBuf::from("/var/lib/alo/undo"),
            kind: alo_files::Kind::Folder,
            own: 0,
            size: bytes,
            counted,
            children: Vec::new(),
        }
    }

    /// Every sentence this crate and the two it says others' out of can say.
    fn strings() -> Strings {
        let mut vocabulary = alo_strings::Vocabulary::empty();
        crate::words::declare_into(&mut vocabulary).unwrap();
        alo_keeping_up::declare_into(&mut vocabulary).unwrap();
        alo_measuring::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// **The number the pane shows is the number that crate answered**, and not
    /// a rounding, a recount or a sum of its own.
    #[test]
    fn the_number_shown_is_the_number_measured() {
        let node = a_node(5_183_545_344, Counted::Whole);
        let kept = WhatIsKept::keeping(HowFarBack::AS_SHIPPED, Holding::measured(&node));
        assert_eq!(kept.holding().unwrap().bytes(), node.size);
    }

    /// **A machine that keeps nothing offers nothing**, and says the plain truth
    /// instead. Held on the value, because this is the promise ADR 0045's sixth
    /// term makes to a person whose disk cannot do it.
    #[test]
    fn a_machine_that_keeps_nothing_offers_no_act_and_says_so() {
        let kept = WhatIsKept::not_on_this_machine();
        let strings = strings();
        assert!(kept.may_forget().is_none());
        assert!(kept.window().is_none());
        assert!(kept.holding().is_none());
        assert!(kept.how_far_back(&strings).is_none());
        assert!(kept.keeps_nothing(&strings).is_some());
    }

    /// **A machine that keeps says how far back, with both numbers in it.**
    #[test]
    fn how_far_back_says_both_numbers() {
        let kept = WhatIsKept::keeping(
            HowFarBack::of(30, 200).unwrap(),
            Holding::measured(&a_node(1024, Counted::Whole)),
        );
        let said = kept.how_far_back(&strings()).unwrap();
        assert!(said.text().contains("30"), "{}", said.text());
        assert!(said.text().contains("200"), "{}", said.text());
    }

    /// **A known none says nothing has needed keeping yet** rather than an
    /// amount of no bytes.
    #[test]
    fn a_known_none_says_nothing_has_needed_keeping() {
        let kept = WhatIsKept::keeping(
            HowFarBack::AS_SHIPPED,
            Holding::measured(&a_node(0, Counted::Whole)),
        );
        let said = kept.holding_said(&strings(), "0 bytes").unwrap();
        assert!(!said.text().contains('0'), "{}", said.text());
    }

    /// **A size that could not be read is not a none**, and the act is still
    /// offered: that is exactly when a person wants the space back.
    #[test]
    fn a_size_that_could_not_be_read_is_not_nothing() {
        let holding = Holding::measured(&a_node(
            0,
            Counted::NotRead {
                why: "it belongs to somebody else".to_owned(),
            },
        ));
        assert!(!holding.nothing_at_all());
        let kept = WhatIsKept::keeping(HowFarBack::AS_SHIPPED, holding);
        assert!(kept.may_forget().is_some());
        assert!(kept.holding().unwrap().not_the_whole(&strings()).is_some());
    }

    /// **A whole count says nothing beside itself**, because there is nothing to
    /// say.
    #[test]
    fn a_whole_count_has_nothing_to_add() {
        let holding = Holding::measured(&a_node(4096, Counted::Whole));
        assert!(holding.not_the_whole(&strings()).is_none());
    }
}
