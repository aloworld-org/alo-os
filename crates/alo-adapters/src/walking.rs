//! Walking one application's windows, once, at the moment it is asked.
//!
//! The walk is the only thing in this crate that asks an
//! [`AccessibilityTree`] anything. It is reached from an authorised call
//! (`crate::reading_windows`, `crate::activating`) and from a person's own
//! command (`crate::by_their_hand`), and **both take this one walk**, so what a
//! person can reach and what alo can reach are found the same way. What it
//! keeps to:
//!
//! 1. **Only this application.** The applications the tree lists are matched by
//!    the identifier their sandbox names ([`crate::accessibility_tree::Running::is`]),
//!    never by the name an application gives itself; a child held on another
//!    connection is another program's and is not followed, only said. **alo's
//!    own surfaces** ([`Whose::AlosOwn`]) are matched by the unique name the
//!    shell's tree answers as on the accessibility bus, which the compositor
//!    tells a person's command and no client can take — never by a name or a
//!    toolkit a client chooses for itself.
//! 1. **Only the window named, when one is** (`--in`): a window whose title is
//!    exactly that, or on alo's own tree the surface whose typed name it is.
//!    None, or more than one, is refused rather than guessed.
//! 2. **Only what is on the screen.** A thing that is not showing is not read,
//!    and neither is anything inside it.
//! 3. **Never a password field's contents.** Its text is not asked for. The
//!    question is not put, so there is no answer to drop.
//! 4. **Bounded.** At most [`MOST_THINGS`] things, [`DEEPEST`] levels and
//!    [`LONGEST_TEXT`] characters of any one text; past that the answer says
//!    not everything was read, rather than stopping silently.

use crate::accessibility_tree::{AccessibilityTree, Facts, NodeAt, Running, TreeFault};
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
    /// The name it had before, on alo's own tree, where the shell says it had
    /// one ([`crate::alos_own_tree::THE_OLD_NAME_ATTRIBUTE`]). Never read from an
    /// application's tree: only alo's own names are promised to keep working.
    pub(crate) formerly: Option<String>,
}

/// What one walk found.
#[derive(Debug)]
pub(crate) struct Walked {
    /// What the windows show.
    pub(crate) shown: Shown,
    /// Every named control that can be pressed.
    pub(crate) pressable: Vec<Found>,
    /// The names of the settings no page of Settings draws yet, as the windows
    /// walked say them — empty on every tree but alo's own.
    pub(crate) not_drawn_yet: Vec<String>,
}

/// Whose windows a walk reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Whose<'a> {
    /// An installed application, by the identifier its sandbox gives it.
    Application(&'a str),
    /// **alo OS's own surfaces**: the tree the shell publishes, by the unique
    /// name it answers as on the accessibility bus.
    ///
    /// The name comes from the compositor, which serves the tree, and a unique
    /// name — one beginning `:` — is the bus's own and cannot be claimed by
    /// another connection. A name that is not unique is never read as alo's:
    /// anybody can ask for a well-known name.
    AlosOwn {
        /// The unique name the shell's tree answers as.
        answers_as: &'a str,
    },
}

impl<'a> Whose<'a> {
    /// What a person is told this is: the identifier, or `alo`.
    #[must_use]
    pub const fn named(self) -> &'a str {
        match self {
            Self::Application(application) => application,
            Self::AlosOwn { .. } => "alo",
        }
    }

    /// Whether a root the tree lists is this one's.
    fn is(self, running: &Running) -> bool {
        match self {
            Self::Application(application) => running.is.as_deref() == Some(application),
            Self::AlosOwn { answers_as } => {
                answers_as.starts_with(':') && running.at.holder == answers_as
            }
        }
    }
}

/// Walk every window of `whose` on the screen — or, where `within` names one,
/// that one window alone.
pub(crate) fn walk(
    tree: &dyn AccessibilityTree,
    whose: Whose<'_>,
    within: Option<&str>,
) -> Result<Walked, NotWalked> {
    let roots: Vec<NodeAt> = tree
        .applications()
        .map_err(NotWalked::Tree)?
        .into_iter()
        .filter(|running| whose.is(running))
        .map(|running| running.at)
        .collect();
    let mut walker = Walker {
        tree,
        read: 0,
        not_all_read: false,
        pressable: Vec::new(),
        alos_own: matches!(whose, Whose::AlosOwn { .. }),
    };
    let mut showing = Vec::new();
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
            if States::of_the_tree(&facts.states).showing() {
                showing.push((window, facts));
            }
        }
    }
    if showing.is_empty() {
        return Err(NotWalked::NoWindow);
    }
    if let Some(named) = within {
        let mut matching = Vec::new();
        for (window, facts) in showing {
            if walker.is_called(whose, &window, &facts, named)? {
                matching.push((window, facts));
            }
        }
        if matching.len() > 1 {
            return Err(NotWalked::MoreThanOneWindow);
        }
        if matching.is_empty() {
            return Err(NotWalked::NoSuchWindow);
        }
        showing = matching;
    }
    let mut windows = Vec::new();
    let mut not_drawn_yet = Vec::new();
    for (window, facts) in showing {
        not_drawn_yet.extend(walker.not_drawn_yet_on(&window)?);
        let mut seen = Vec::new();
        walker.inside(&window, &facts, 0, &mut seen)?;
        windows.push(ShownWindow::titled(facts.name, seen));
    }
    Ok(Walked {
        shown: Shown::of(whose.named().to_owned(), windows, walker.not_all_read),
        pressable: walker.pressable,
        not_drawn_yet,
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
    /// Whether this is alo's own tree, the one tree whose old names are
    /// promised to keep working (`alo_access::old_names`).
    alos_own: bool,
}

impl Walker<'_> {
    /// Whether this window is the one a person named: by its exact title, or
    /// on alo's own tree by the surface's typed name, which the shell publishes
    /// as an attribute of the surface's own node.
    fn is_called(
        &self,
        whose: Whose<'_>,
        window: &NodeAt,
        facts: &Facts,
        named: &str,
    ) -> Result<bool, NotWalked> {
        match whose {
            Whose::Application(_) => Ok(facts.name == named),
            Whose::AlosOwn { .. } => match self.tree.attributes(window) {
                Ok(attributes) => Ok(attributes.iter().any(|(key, value)| {
                    key == crate::alos_own_tree::THE_SURFACE_ATTRIBUTE && value == named
                })),
                Err(TreeFault::Vanished) => Ok(false),
                Err(fault) => Err(NotWalked::Tree(fault)),
            },
        }
    }

    /// The attributes of one thing on alo's own tree, or none on any other —
    /// and none for a thing that vanished while being read.
    fn alos_own_attributes(&self, at: &NodeAt) -> Result<Vec<(String, String)>, NotWalked> {
        if !self.alos_own {
            return Ok(Vec::new());
        }
        match self.tree.attributes(at) {
            Ok(attributes) => Ok(attributes),
            Err(TreeFault::Vanished) => Ok(Vec::new()),
            Err(fault) => Err(NotWalked::Tree(fault)),
        }
    }

    /// The names a window says belong to settings no page draws yet.
    fn not_drawn_yet_on(&self, window: &NodeAt) -> Result<Vec<String>, NotWalked> {
        Ok(self
            .alos_own_attributes(window)?
            .into_iter()
            .filter(|(key, _)| crate::alos_own_tree::the_setting_not_drawn_yet(key).is_some())
            .map(|(_, name)| name)
            .collect())
    }

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
                        let formerly =
                            self.alos_own_attributes(at)?
                                .into_iter()
                                .find_map(|(key, value)| {
                                    (key == crate::alos_own_tree::THE_OLD_NAME_ATTRIBUTE)
                                        .then_some(value)
                                });
                        self.pressable.push(Found {
                            kind,
                            name,
                            at: at.clone(),
                            usable: states.usable(),
                            formerly,
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
