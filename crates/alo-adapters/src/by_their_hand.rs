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
use crate::not_pressed::NotPressed;
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
    pressed(tree, whose, within, kind, name).map(|unanswered| PressedByHand { unanswered })
}

/// A control a person's command pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PressedByHand {
    /// Whether the application did not answer in time — the press was sent and
    /// whether it happened is not known (`terminal-commands.md`, exit 75).
    unanswered: bool,
}

impl PressedByHand {
    /// Whether the press was sent and the application did not answer.
    #[must_use]
    pub const fn unanswered(self) -> bool {
        self.unanswered
    }
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
        strings: &Strings,
    ) -> Said {
        match self {
            Self::NotWalked(why) => why.said(whose.named(), within, strings),
            Self::NotPressed(why) => why.said_of(whose.named(), kind, name, strings),
        }
    }
}

/// **The one press**, for the agent's road and a person's alike: walk, find
/// exactly one usable control of that kind and name, and do its press action.
///
/// Answers whether the application did not answer in time.
pub(crate) fn pressed(
    tree: &dyn AccessibilityTree,
    whose: Whose<'_>,
    within: Option<&str>,
    kind: Pressable,
    name: &str,
) -> Result<bool, Unpressed> {
    let not_pressed = Unpressed::NotPressed;
    let Walked { shown, pressable } = walk(tree, whose, within).map_err(Unpressed::NotWalked)?;
    let mut matching = pressable
        .into_iter()
        .filter(|found| found.kind == kind && found.name == name);
    let found: Found = match (matching.next(), matching.next(), shown.not_all_read()) {
        (_, Some(_), _) => return Err(not_pressed(NotPressed::MoreThanOne)),
        (_, None, true) => return Err(not_pressed(NotPressed::TooMuchToBeSure)),
        (None, None, false) => return Err(not_pressed(NotPressed::NoSuchControl)),
        (Some(found), None, false) => found,
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
        Ok(true) => Ok(false),
        Ok(false) => Err(not_pressed(NotPressed::DidNotPress)),
        Err(TreeFault::DidNotAnswer) => Ok(true),
        Err(TreeFault::Vanished) => Err(not_pressed(NotPressed::NoSuchControl)),
        Err(fault @ TreeFault::NotThere) => Err(Unpressed::NotWalked(NotWalked::Tree(fault))),
    }
}
