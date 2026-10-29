//! **Every surface this machine draws, as the things a tree is made of.**
//!
//! One node for the shell itself, one for each surface `alo_access::Surface`
//! names, and one for each control on it — in the order a reader reads them,
//! which is `alo_access::Surface::read_aloud`'s order and therefore
//! `alo_access::focus_order`'s.
//!
//! **Nothing is decided here.** The role, the name and the state of every
//! control are `alo-access`'s; the numbers are `crate::access_roles`'; and the
//! shape — what hangs under what — is the only thing this file chooses. It
//! chooses it from the roles rather than by hand: a surface whose first control
//! is one others are read inside **is** that control, with the rest inside it,
//! and a surface whose first control is not gets a filler with no name, because
//! inventing a name for a box nobody decided would be this crate wording
//! something.
//!
//! # A name is said, never spelled
//!
//! Every name crosses the bus already in the person's language: it is
//! `alo_strings::Strings::say` of the word `alo-access` named the control with,
//! so a machine being used in Latvian publishes a tree in Latvian. This is the
//! same vocabulary the surface is drawn from, so what a reader hears and what a
//! sighted person sees are one sentence and not two that can drift.

use alo_access::{Control, Surface, TurnedOn};
use alo_dock::places::WhatIsHere;
use alo_strings::{Filling, Strings};

use crate::access_action::Pressed;
use crate::access_roles::{self, APPLICATION, FILLER};
use crate::frame_name::FrameName;
use crate::the_dock_as_a_reader_meets_it::{ItsName, as_read};
use crate::the_tasks_a_person_opened::WhichTask;
use crate::window_edge::Decorations;
use crate::window_edge_reading::what_a_reader_is_told;
use crate::window_edge_who_draws::who_draws_a_frame;

/// Where every application's tree starts, as at-spi2 fixes it.
pub(crate) const ROOT: &str = "/org/a11y/atspi/accessible/root";

/// One thing in the tree, as a reader is told about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Node {
    /// Where it is, on the bus.
    pub(crate) path: String,
    /// What kind of thing it is, as `AtspiRole` numbers them.
    pub(crate) role: u32,
    /// What it is called, in the person's own language.
    pub(crate) name: String,
    /// The two words `GetState` answers with, low word first.
    pub(crate) states: [u32; 2],
    /// Whether a reader announces it the moment it changes.
    pub(crate) announced: bool,
    /// What it hangs under.
    pub(crate) parent: usize,
    /// What hangs under it, in reading order.
    pub(crate) children: Vec<usize>,
    /// What pressing it asks the shell to do, where a reader can press it.
    pub(crate) press: Option<Pressed>,
}

/// **The whole of what a screen reader is told about this machine.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadAloudTree {
    /// Every node, the shell itself first.
    nodes: Vec<Node>,
    /// Which node each surface's own is.
    surfaces: Vec<(Surface, usize)>,
}

impl ReadAloudTree {
    /// The tree for somebody reading in `strings`' language, with `showing`
    /// the surfaces that are up now.
    ///
    /// **Every surface is described and only some are shown.** A reader is
    /// told this machine has a recovery screen whether or not one is in front
    /// of the person; what says which are actually there is the `SHOWING`
    /// state, and a tree that claimed all nine at once would have a reader
    /// announcing screens nobody is looking at.
    #[must_use]
    /// `turned_on` is what the person has switched on, which is how a
    /// setting's control says which way it is set. **Handed in rather than
    /// read**: this file does not know where a person's settings live, and the
    /// caller is the only thing that does.
    pub fn of(strings: &Strings, showing: &[Surface], turned_on: &TurnedOn) -> Self {
        let mut this = Self {
            nodes: vec![Node {
                path: ROOT.to_owned(),
                role: APPLICATION,
                // **Nothing names the machine itself.** `alo-access` names
                // controls, and no crate words *this machine* — so the thing a
                // reader announces as the application has the name a filler
                // has, which is none, rather than one written here. Written
                // down as a finding in the access plan: it is a word that has
                // to be decided before it can be said.
                name: String::new(),
                states: access_roles::words_of(alo_access::State::ReadOnly, None),
                announced: false,
                parent: 0,
                children: Vec::new(),
                press: None,
            }],
            surfaces: Vec::new(),
        };
        for surface in Surface::ALL {
            let drawn = surface.read_aloud();
            let (at, rest) = match drawn.split_first() {
                // A surface `alo-access` names no control on cannot happen —
                // its own test holds every surface to being read as something —
                // and if it ever does, it is left out rather than published as
                // a box with nothing in it.
                None => continue,
                Some((first, rest)) if access_roles::holds_others(first.role) => {
                    (this.push(0, of_control(strings, *first, turned_on)), rest)
                }
                Some(_) => (
                    this.push(
                        0,
                        Node {
                            path: String::new(),
                            role: FILLER,
                            name: String::new(),
                            states: access_roles::words_of(alo_access::State::ReadOnly, None),
                            announced: false,
                            parent: 0,
                            children: Vec::new(),
                            press: None,
                        },
                    ),
                    drawn.as_slice(),
                ),
            };
            for control in rest {
                this.push(at, of_control(strings, *control, turned_on));
            }
            if !showing.contains(&surface) {
                let under: Vec<usize> = this
                    .nodes
                    .get(at)
                    .map(|node| node.children.clone())
                    .unwrap_or_default();
                for node in std::iter::once(at).chain(under) {
                    if let Some(node) = this.nodes.get_mut(node) {
                        node.states = access_roles::off_the_screen(node.states);
                    }
                }
            }
            this.surfaces.push((surface, at));
        }
        this
    }

    /// Every node, the shell itself first.
    pub(crate) fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Where each surface's own node is — what the tests hold the shape to.
    /// Nothing serving the tree needs it: the bus is answered from the nodes.
    #[cfg(test)]
    pub(crate) fn surfaces(&self) -> &[(Surface, usize)] {
        &self.surfaces
    }

    /// How many things a reader is told about, the shell itself included.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.nodes.len()
    }

    /// Put `node` under `parent`, and say where it went.
    /// The same tree, with the frames now open hanging under *the windows open*.
    ///
    /// **Task 7 of the canvas plan: every frame reached, focused and *named*.**
    /// `alo-access` lists `Surface::Desktop` as a window and a list called *the
    /// windows open*, and that list had no children — a reader was told the
    /// machine has a list of windows and never what was in it.
    ///
    /// # A frame's name is the only name here a translator never sees
    ///
    /// Every other name in this tree is an `alo_access::words::Word` put through
    /// the person's language. A frame's is whatever the application called
    /// itself, so it arrives already in whatever language that application
    /// chose, and no vocabulary can hold it. [`FrameName::its_own_words`]
    /// returns `None` for the one case that **is** ours to word — an application
    /// that set neither a title nor a class — and that falls back to
    /// `alo_access::words::AN_APPLICATION`, which is the phrase this tree
    /// already uses for a window nobody named.
    ///
    /// # Each window carries what can be done to it
    ///
    /// The owner's direction of 2026-10-09: *screen-reader and keyboard users
    /// must reach the new edge without first hovering*. So every window's
    /// controls hang under it here, where the tree is built from what is open
    /// rather than from where a pointer is. `turned_on` is carried for the
    /// same reason [`Self::of`] carries it - a control's node says which way
    /// its setting is set, and a window control sets nothing, so it is passed
    /// through rather than consulted.
    ///
    /// # Order is the caller's
    ///
    /// They are listed in the order given, which is `Server::mapped_surfaces`'
    /// order, which is the same ring `switch_window` walks. A reader hearing a
    /// different order from the one the keyboard moves in would be a second
    /// interface, which is the constraint task 7 carries.
    #[must_use]
    pub fn with_the_frames_open(
        mut self,
        strings: &Strings,
        frames: &[FrameName],
        turned_on: &TurnedOn,
    ) -> Self {
        let Some(list) = self.the_windows_open() else {
            return self;
        };
        for frame in frames {
            let name = frame.its_own_words().map_or_else(
                || said(strings, alo_access::words::AN_APPLICATION),
                str::to_owned,
            );
            let frame = self.push(
                list,
                Node {
                    path: String::new(),
                    role: access_roles::number_of(alo_access::Role::ListItem),
                    name,
                    states: access_roles::words_of(alo_access::State::CanBeUsed, None),
                    announced: false,
                    parent: list,
                    children: Vec::new(),
                    press: None,
                },
            );
            // **What can be done to this window, under the window itself.**
            // Until now the list held names and nothing else: a reader was
            // told a window was open and never that anything could be done to
            // it. The controls are the ones `crate::window_edge` draws, asked
            // of the same `who_draws_a_frame`, so what a person hears and what
            // is drawn cannot disagree - and they are here whether or not the
            // edge is revealed, because revealing is what a person sees and is
            // never what a person can reach.
            for control in what_a_reader_is_told(who_draws_a_frame()) {
                let node = of_control(strings, control, turned_on);
                self.push(frame, node);
            }
        }
        self
    }

    /// The same tree, with the task windows now open hanging under *the
    /// windows open* too, after the frames.
    ///
    /// **The owner's decision of 2026-10-28**: the Close on a task window's
    /// edge is in this tree, so somebody using a screen reader hears it on
    /// every task window and can press it. Until now a task window was not in
    /// the tree at all, so a reader was told nothing about the one window
    /// alo itself draws for a person's request.
    ///
    /// # Named as the window is titled
    ///
    /// `tasks` is each open task window and its title, which is the person's
    /// own request — the words the window draws at its top. Like a frame's
    /// title it is nobody's to translate, so it is used as it is; a request
    /// with no words falls back to `alo_tasks::words::A_TASK`, the word the
    /// window draws under its title.
    ///
    /// # Its controls are asked the same question as a frame's
    ///
    /// `what_a_reader_is_told(Decorations::ATaskWindow)`, which is Close and
    /// nothing else — the same answer `crate::a_task_windows_edge` draws from,
    /// so what is heard and what is drawn cannot disagree. Each control that
    /// closes a window carries [`Pressed::CloseTheTask`], so a reader's press
    /// reaches the window it is under and no other.
    ///
    /// # Order
    ///
    /// Oldest first, which is the order they are drawn in; after the frames,
    /// because task windows are drawn above every application window.
    #[must_use]
    pub fn with_the_task_windows_open(
        mut self,
        strings: &Strings,
        tasks: &[(WhichTask, String)],
        turned_on: &TurnedOn,
    ) -> Self {
        let Some(list) = self.the_windows_open() else {
            return self;
        };
        for (which, title) in tasks {
            let name = if title.trim().is_empty() {
                said(strings, alo_tasks::words::A_TASK)
            } else {
                title.clone()
            };
            let window = self.push(
                list,
                Node {
                    path: String::new(),
                    role: access_roles::number_of(alo_access::Role::ListItem),
                    name,
                    states: access_roles::words_of(alo_access::State::CanBeUsed, None),
                    announced: false,
                    parent: list,
                    children: Vec::new(),
                    press: None,
                },
            );
            for control in what_a_reader_is_told(Decorations::ATaskWindow) {
                let press = (control.does == Some(alo_shortcuts::Action::CloseWindow))
                    .then_some(Pressed::CloseTheTask(*which));
                let node = Node {
                    press,
                    ..of_control(strings, control, turned_on)
                };
                self.push(window, node);
            }
        }
        self
    }

    /// Everything a reader can press on this tree, in reading order.
    ///
    /// What `crate::access_serving` compares besides the names, so a task
    /// window that closed and another of the same title that opened are not
    /// mistaken for one window whose Close still reaches the first.
    #[must_use]
    pub fn what_can_be_pressed(&self) -> Vec<Pressed> {
        self.nodes
            .iter()
            .filter_map(|node| node.press.clone())
            .collect()
    }

    /// The same tree, with each control on the Dock in place of the Dock's
    /// description.
    ///
    /// **The owner's decision of 2026-10-28**: every Dock button has a name a
    /// screen reader announces. `alo-access` describes the Dock as a list with
    /// *an application* and *open something* in it, which is what a reader is
    /// told about a Dock in general; `slots` is this Dock, as it was drawn, and
    /// once there is one the description gives way to it. Each control is
    /// named and made pressable by `crate::the_dock_as_a_reader_meets_it`.
    ///
    /// **No slots, no change**: before a Dock has been drawn there is nothing
    /// to name, and the description is truer than an empty list would be.
    ///
    /// The new controls are shown exactly when the Dock is, so a Dock that is
    /// not up is not read as something in front of the person.
    #[must_use]
    pub fn with_the_docks_slots(mut self, strings: &Strings, slots: &[WhatIsHere]) -> Self {
        if slots.is_empty() {
            return self;
        }
        let Some(dock) = self
            .surfaces
            .iter()
            .find_map(|(surface, at)| (*surface == Surface::Dock).then_some(*at))
        else {
            return self;
        };
        let showing = self
            .nodes
            .get(dock)
            .is_some_and(|node| access_roles::is_showing(node.states));
        let dock = self.take_away_what_hangs_under(dock);
        for what in slots {
            let Some(slot) = as_read(what) else {
                continue;
            };
            let name = match slot.name {
                ItsName::ItsOwn(name) => name.to_owned(),
                ItsName::Said(word) => said(strings, word),
            };
            let states = access_roles::words_of(alo_access::State::CanBeUsed, None);
            self.push(
                dock,
                Node {
                    path: String::new(),
                    role: access_roles::number_of(slot.role),
                    name,
                    states: if showing {
                        states
                    } else {
                        access_roles::off_the_screen(states)
                    },
                    announced: false,
                    parent: dock,
                    children: Vec::new(),
                    press: slot.press,
                },
            );
        }
        self
    }

    /// What a reader is told is on the Dock, in the order it hears it.
    ///
    /// The Dock's twin of [`Self::the_windows_open_as_read`]: what somebody
    /// with their eyes shut would hear walking the Dock.
    #[must_use]
    pub fn the_dock_as_read(&self) -> Vec<&str> {
        self.surfaces
            .iter()
            .find_map(|(surface, at)| (*surface == Surface::Dock).then_some(*at))
            .and_then(|dock| self.nodes.get(dock))
            .map(|dock| {
                dock.children
                    .iter()
                    .filter_map(|at| self.nodes.get(*at).map(|node| node.name.as_str()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Take away everything under `at`, and say where `at` now is.
    ///
    /// **Every index is renumbered, and every path with it**, because a node's
    /// path is its position (see [`Self::push`]) and a gap would leave a path
    /// pointing at the wrong node. The shell's own node is never under
    /// anything, so it stays first and keeps [`ROOT`].
    fn take_away_what_hangs_under(&mut self, at: usize) -> usize {
        let mut gone: Vec<usize> = Vec::new();
        let mut looking: Vec<usize> = self
            .nodes
            .get(at)
            .map(|node| node.children.clone())
            .unwrap_or_default();
        while let Some(next) = looking.pop() {
            if let Some(node) = self.nodes.get(next) {
                looking.extend(node.children.iter().copied());
            }
            gone.push(next);
        }
        let mut moved_to: Vec<Option<usize>> = Vec::with_capacity(self.nodes.len());
        let mut kept = 0;
        for old in 0..self.nodes.len() {
            if gone.contains(&old) {
                moved_to.push(None);
            } else {
                moved_to.push(Some(kept));
                kept += 1;
            }
        }
        let moved = |old: usize| moved_to.get(old).copied().flatten();
        let nodes = std::mem::take(&mut self.nodes);
        self.nodes = nodes
            .into_iter()
            .enumerate()
            .filter_map(|(old, node)| {
                let new = moved(old)?;
                Some(Node {
                    path: if new == 0 {
                        node.path
                    } else {
                        format!("/org/a11y/atspi/accessible/{new}")
                    },
                    parent: moved(node.parent).unwrap_or(0),
                    children: node.children.iter().filter_map(|c| moved(*c)).collect(),
                    ..node
                })
            })
            .collect();
        self.surfaces = std::mem::take(&mut self.surfaces)
            .into_iter()
            .filter_map(|(surface, old)| moved(old).map(|new| (surface, new)))
            .collect();
        moved(at).unwrap_or(at)
    }

    /// The windows a reader is told are open, in the order it hears them.
    ///
    /// The one question worth asking this tree from outside: *what would somebody
    /// with their eyes shut be told is open right now*. Answering it does not
    /// mean handing out every node, so it does not.
    #[must_use]
    pub fn the_windows_open_as_read(&self) -> Vec<&str> {
        self.the_windows_open()
            .and_then(|list| self.nodes.get(list))
            .map(|list| {
                list.children
                    .iter()
                    .filter_map(|at| self.nodes.get(*at).map(|node| node.name.as_str()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Where a frame hangs: the list under the desktop, found by its role.
    ///
    /// By role rather than by position, because *the windows open* is the second
    /// control `Surface::Desktop` names today and a third arriving above it would
    /// silently start collecting windows.
    fn the_windows_open(&self) -> Option<usize> {
        let desktop = self
            .surfaces
            .iter()
            .find_map(|(surface, at)| (*surface == Surface::Desktop).then_some(*at))?;
        let list = access_roles::number_of(alo_access::Role::List);
        self.nodes
            .get(desktop)?
            .children
            .iter()
            .copied()
            .find(|at| self.nodes.get(*at).is_some_and(|node| node.role == list))
    }

    /// Put a node under `parent`, at the path its position gives it.
    fn push(&mut self, parent: usize, node: Node) -> usize {
        let at = self.nodes.len();
        self.nodes.push(Node {
            path: format!("/org/a11y/atspi/accessible/{at}"),
            parent,
            ..node
        });
        if let Some(above) = self.nodes.get_mut(parent) {
            above.children.push(at);
        }
        at
    }
}

/// One control, as a node.
fn of_control(strings: &Strings, control: Control, turned_on: &TurnedOn) -> Node {
    Node {
        path: String::new(),
        role: access_roles::number_of(control.role),
        name: said(strings, control.name),
        // **Which way it is set, for a control that is a setting.** Looked up
        // from the setting the control carries rather than matched by name, so
        // renaming a word cannot silently detach a switch from its value.
        states: access_roles::words_of(
            control.state,
            control.setting.map(|setting| turned_on.has(setting)),
        ),
        announced: access_roles::is_announced(control.state),
        parent: 0,
        children: Vec::new(),
        press: None,
    }
}

/// One word, in the person's language.
fn said(strings: &Strings, word: alo_access::words::Word) -> String {
    strings
        .say(&word.key(), &Filling::nothing())
        .text()
        .to_owned()
}

#[cfg(test)]
#[path = "access_nodes_tests.rs"]
mod tests;
