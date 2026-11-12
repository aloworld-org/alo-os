//! **A person's own command, reaching an application's windows** — what
//! `alo-app <application> controls`, `read` and `press` stand on
//! (`docs/contracts/terminal-commands.md`, section 2, *The accessibility tree*).
//!
//! The person typed the command, so nothing here asks for a grant or an
//! approval: law 2 binds the agent, not the person. **Everything else is the
//! agent's road exactly.** There is one walk (`crate::walking`) and one press
//! (`pressed`, which `crate::activating` calls too), so the same bounds, the
//! same password rule and the same refusals hold whoever asked: a control
//! missing, more than one of it, too much showing to be sure, greyed out,
//! nothing to press it with, an application that said it did not, or no window
//! at all. A person is never told something alo would not be, and alo never
//! reaches a control a person's command could not.
//!
//! What this file adds is what a person can say and alo cannot yet:
//! [`Whose::AlosOwn`] for alo OS's own surfaces, and `within` for `--in`.

use alo_strings::{Said, Strings};

use crate::accessibility_tree::{AccessibilityTree, TreeFault};
use crate::activating::PRESSING;
use crate::not_pressed::{NotPressed, WorkingFully};
use crate::not_walked::NotWalked;
use crate::pressable::Pressable;
use crate::shown::Shown;
use crate::walking::{Found, Walked, Whose, walk};

/// What the windows show — every control with its kind, name and state, and
/// their text — read once, now, and kept nowhere.
///
/// `within` is the window a person named with `--in`, if they named one.
///
/// # Errors
/// [`NotWalked`]: nothing was read.
pub fn read_by_hand(
    tree: &dyn AccessibilityTree,
    whose: Whose<'_>,
    within: Option<&str>,
) -> Result<Shown, NotWalked> {
    walk(tree, whose, within).map(|walked| walked.shown)
}

/// Press the `kind` called `name`, once, by the control's own action.
///
/// # Errors
/// [`Unpressed`]: nothing was pressed.
pub fn press_by_hand(
    tree: &dyn AccessibilityTree,
    whose: Whose<'_>,
    within: Option<&str>,
    kind: Pressable,
    name: &str,
) -> Result<PressedByHand, Unpressed> {
    pressed(tree, whose, within, kind, name).map(|reached| PressedByHand {
        unanswered: reached.unanswered,
        now_called: reached.now_called,
    })
}

/// A control a person's command pressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PressedByHand {
    /// Whether the application did not answer in time — the press was sent and
    /// whether it happened is not known (`terminal-commands.md`, exit 75).
    unanswered: bool,
    /// What the control is called now, where the person named it by the name
    /// it had before (`alo_access::old_names`) — which the command says will
    /// change.
    now_called: Option<String>,
}

impl PressedByHand {
    /// Whether the press was sent and the application did not answer.
    #[must_use]
    pub const fn unanswered(&self) -> bool {
        self.unanswered
    }

    /// What the control is called now, where it was pressed by the name it had
    /// before on alo OS's own surfaces; [`None`] where it was pressed by the
    /// name it shows.
    #[must_use]
    pub fn now_called(&self) -> Option<&str> {
        self.now_called.as_deref()
    }
}

/// What the one press reached.
#[derive(Debug)]
pub(crate) struct Reached {
    /// Whether the application did not answer in time.
    pub(crate) unanswered: bool,
    /// What the control is called now, where it was found by its old name.
    pub(crate) now_called: Option<String>,
}

/// Why nothing was pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unpressed {
    /// The windows could not be walked, so no control was looked for.
    NotWalked(NotWalked),
    /// The control was looked for and not pressed.
    NotPressed(NotPressed),
}

impl Unpressed {
    /// What a person is told — the same sentence the agent's road says.
    #[must_use]
    pub fn said(
        self,
        whose: Whose<'_>,
        within: Option<&str>,
        kind: Pressable,
        name: &str,
        fully: WorkingFully,
        strings: &Strings,
    ) -> Said {
        match self {
            Self::NotWalked(why) => why.said(whose.named(), within, strings),
            Self::NotPressed(why) => why.said_of(whose.named(), kind, name, fully, strings),
        }
    }
}

/// **The one press**, for the agent's road and a person's alike: walk, find
/// exactly one usable control of that kind and name, and do its press action.
///
/// Answers whether the application did not answer in time, and what the
/// control is called now where it was found by the name it had before.
///
/// # The name it had before
///
/// **Only on alo OS's own surfaces, and only where no control shows the name
/// typed.** The shell marks the row that is an accessibility setting with the
/// name the fixed list gave it (`alo_access::old_names`), so `press switch
/// "stop things sliding and fading" --in settings` still reaches *Reduced
/// motion*; a name a control shows always wins over an old one. A setting no
/// page draws yet is refused by its own sentence rather than as *no such
/// control*, because the person named something real that is not there.
pub(crate) fn pressed(
    tree: &dyn AccessibilityTree,
    whose: Whose<'_>,
    within: Option<&str>,
    kind: Pressable,
    name: &str,
) -> Result<Reached, Unpressed> {
    let not_pressed = Unpressed::NotPressed;
    let Walked {
        shown,
        pressable,
        not_drawn_yet,
    } = walk(tree, whose, within).map_err(Unpressed::NotWalked)?;
    let (by_its_name, by_its_old_name): (Vec<Found>, Vec<Found>) = pressable
        .into_iter()
        .filter(|found| found.kind == kind)
        .filter(|found| found.name == name || found.formerly.as_deref() == Some(name))
        .partition(|found| found.name == name);
    let renamed = by_its_name.is_empty();
    let mut matching = if renamed {
        by_its_old_name
    } else {
        by_its_name
    }
    .into_iter();
    let found: Found = match (matching.next(), matching.next(), shown.not_all_read()) {
        (_, Some(_), _) => return Err(not_pressed(NotPressed::MoreThanOne)),
        (_, None, true) => return Err(not_pressed(NotPressed::TooMuchToBeSure)),
        (None, None, false) if not_drawn_yet.iter().any(|named| named == name) => {
            return Err(not_pressed(NotPressed::NotDrawnYet));
        }
        (None, None, false) => return Err(not_pressed(NotPressed::NoSuchControl)),
        (Some(found), None, false) => found,
    };
    let now_called = renamed.then(|| found.name.clone());
    let reached = |unanswered| Reached {
        unanswered,
        now_called: now_called.clone(),
    };
    if !found.usable {
        return Err(not_pressed(NotPressed::CannotBeUsedNow));
    }
    let actions = match tree.actions(&found.at) {
        Ok(actions) => actions,
        Err(TreeFault::Vanished) => return Err(not_pressed(NotPressed::NoSuchControl)),
        Err(fault) => return Err(Unpressed::NotWalked(NotWalked::Tree(fault))),
    };
    let Some(action) = PRESSING.iter().find_map(|pressing| {
        actions
            .iter()
            .position(|action| action.eq_ignore_ascii_case(pressing))
    }) else {
        return Err(not_pressed(NotPressed::CannotBePressed));
    };
    match tree.act(&found.at, action) {
        Ok(true) => Ok(reached(false)),
        Ok(false) => Err(not_pressed(NotPressed::DidNotPress)),
        Err(TreeFault::DidNotAnswer) => Ok(reached(true)),
        Err(TreeFault::Vanished) => Err(not_pressed(NotPressed::NoSuchControl)),
        Err(fault @ TreeFault::NotThere) => Err(Unpressed::NotWalked(NotWalked::Tree(fault))),
    }
}
