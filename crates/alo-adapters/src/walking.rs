//! Walking one application's windows, once, at the moment it is asked.
//!
//! The walk is the only thing in this crate that asks an
//! [`AccessibilityTree`] anything, and it is reached only from an authorised
//! call (`crate::reading_windows`, `crate::activating`). What it keeps to:
//!
//! 1. **Only this application.** The applications the tree lists are matched by
//!    the identifier their sandbox names ([`crate::accessibility_tree::Running::is`]),
//!    never by the name an application gives itself; a child held on another
//!    connection is another program's and is not followed, only said.
//! 2. **Only what is on the screen.** A thing that is not showing is not read,
//!    and neither is anything inside it.
//! 3. **Never a password field's contents.** Its text is not asked for. The
//!    question is not put, so there is no answer to drop.
//! 4. **Bounded.** At most [`MOST_THINGS`] things, [`DEEPEST`] levels and
//!    [`LONGEST_TEXT`] characters of any one text; past that the answer says
//!    not everything was read, rather than stopping silently.

use crate::accessibility_tree::{AccessibilityTree, Facts, NodeAt, TreeFault};
use crate::not_walked::NotWalked;
use crate::pressable::Pressable;
use crate::role::Role;
use crate::shown::{Limit, Seen, Shown, ShownWindow};
use crate::states::States;

/// The most things read from one application at once.
pub const MOST_THINGS: usize = 2_000;

/// The deepest a thing is read inside its window.
pub const DEEPEST: usize = 48;

/// The most characters of one thing's text that are read.
pub const LONGEST_TEXT: usize = 4_000;

/// A control that can be pressed, as the walk found it.
#[derive(Debug)]
pub(crate) struct Found {
    /// What kind it is.
    pub(crate) kind: Pressable,
    /// What it is called.
    pub(crate) name: String,
    /// Where it is, for pressing it in the same act.
    pub(crate) at: NodeAt,
    /// Whether a person could use it now.
    pub(crate) usable: bool,
}

/// What one walk found.
#[derive(Debug)]
pub(crate) struct Walked {
    /// What the windows show.
    pub(crate) shown: Shown,
    /// Every named control that can be pressed.
    pub(crate) pressable: Vec<Found>,
}

/// Walk every window of `application` on the screen.
pub(crate) fn walk(tree: &dyn AccessibilityTree, application: &str) -> Result<Walked, NotWalked> {
    let roots: Vec<NodeAt> = tree
        .applications()
        .map_err(NotWalked::Tree)?
        .into_iter()
        .filter(|running| running.is.as_deref() == Some(application))
        .map(|running| running.at)
        .collect();
    let mut walker = Walker {
        tree,
        read: 0,
        not_all_read: false,
        pressable: Vec::new(),
    };
    let mut windows = Vec::new();
    for root in roots {
        let Some(facts) = walker.facts(&root)? else {
            continue;
        };
        for window in facts.children {
            if window.holder != root.holder {
                continue;
            }
            let Some(facts) = walker.facts(&window)? else {
                continue;
            };
            if !States::of_the_tree(&facts.states).showing() {
                continue;
            }
            let mut seen = Vec::new();
            walker.inside(&window, &facts, 0, &mut seen)?;
            windows.push(ShownWindow::titled(facts.name, seen));
        }
    }
    if windows.is_empty() {
        return Err(NotWalked::NoWindow);
    }
    Ok(Walked {
        shown: Shown::of(application.to_owned(), windows, walker.not_all_read),
        pressable: walker.pressable,
    })
}

/// One walk in progress.
struct Walker<'t> {
    /// The tree.
    tree: &'t dyn AccessibilityTree,
    /// How many things have been read.
    read: usize,
    /// Whether something was left unread.
    not_all_read: bool,
    /// The named controls found so far.
    pressable: Vec<Found>,
}

impl Walker<'_> {
    /// The facts about one thing, or [`None`] when it vanished while being
    /// read — a window changes while it is read, and what left it is not
    /// there to describe.
    fn facts(&self, at: &NodeAt) -> Result<Option<Facts>, NotWalked> {
        match self.tree.facts(at) {
            Ok(facts) => Ok(Some(facts)),
            Err(TreeFault::Vanished) => Ok(None),
            Err(fault) => Err(NotWalked::Tree(fault)),
        }
    }

    /// Everything inside `parent`, in reading order.
    fn inside(
        &mut self,
        parent: &NodeAt,
        facts: &Facts,
        depth: usize,
        seen: &mut Vec<Seen>,
    ) -> Result<(), NotWalked> {
        if depth >= DEEPEST {
            self.not_all_read |= !facts.children.is_empty();
            return Ok(());
        }
        self.not_all_read |= facts.unlisted_children;
        for child in &facts.children {
            if self.read >= MOST_THINGS {
                self.not_all_read = true;
                return Ok(());
            }
            if child.holder != parent.holder {
                seen.push(
                    Seen::of(Role::Part, String::new(), None, depth)
                        .limited(Limit::ShownByAnotherProgram),
                );
                continue;
            }
            let Some(facts) = self.facts(child)? else {
                continue;
            };
            self.read += 1;
            let states = States::of_the_tree(&facts.states);
            if !states.showing() {
                continue;
            }
            self.one(child, &facts, states, depth, seen)?;
            self.inside(child, &facts, depth + 1, seen)?;
        }
        Ok(())
    }

    /// One thing on the screen, described if it says anything.
    fn one(
        &mut self,
        at: &NodeAt,
        facts: &Facts,
        states: States,
        depth: usize,
        seen: &mut Vec<Seen>,
    ) -> Result<(), NotWalked> {
        let role = Role::of_the_tree(facts.role);
        let name = facts.name.trim().to_owned();
        // The one question never put to a password field.
        let text = if role.text_is_read() && role != Role::PasswordField && facts.has_text {
            match self.tree.text(at) {
                Ok(text) => Some(text.chars().take(LONGEST_TEXT).collect()),
                Err(TreeFault::Vanished) => None,
                Err(fault) => return Err(NotWalked::Tree(fault)),
            }
        } else {
            None
        };
        let described = Seen::of(role, name.clone(), text, depth)
            .on(states.on(role))
            .usable(states.usable());
        let kept = match role {
            Role::Button
            | Role::CheckBox
            | Role::RadioButton
            | Role::Switch
            | Role::MenuItem
            | Role::Tab
            | Role::Link => {
                if name.is_empty() {
                    Some(described.limited(Limit::NoName))
                } else {
                    if let Some(kind) = Pressable::of(role) {
                        self.pressable.push(Found {
                            kind,
                            name,
                            at: at.clone(),
                            usable: states.usable(),
                        });
                    }
                    Some(described)
                }
            }
            Role::Canvas if facts.children.is_empty() => {
                Some(described.limited(Limit::NotDescribed))
            }
            Role::TextField | Role::PasswordField | Role::Window | Role::Canvas => Some(described),
            Role::Label | Role::Document | Role::Image | Role::Part => {
                let says_something = !name.is_empty()
                    || matches!(described.contents(), crate::shown::Contents::Text(_));
                says_something.then_some(described)
            }
        };
        seen.extend(kept);
        Ok(())
    }
}
