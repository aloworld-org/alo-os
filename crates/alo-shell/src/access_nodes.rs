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

use alo_access::{Control, Surface};
use alo_strings::{Filling, Strings};

use crate::access_roles::{self, APPLICATION, FILLER};

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
    pub fn of(strings: &Strings, showing: &[Surface]) -> Self {
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
                states: access_roles::words_of(alo_access::State::ReadOnly),
                announced: false,
                parent: 0,
                children: Vec::new(),
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
                    (this.push(0, of_control(strings, *first)), rest)
                }
                Some(_) => (
                    this.push(
                        0,
                        Node {
                            path: String::new(),
                            role: FILLER,
                            name: String::new(),
                            states: access_roles::words_of(alo_access::State::ReadOnly),
                            announced: false,
                            parent: 0,
                            children: Vec::new(),
                        },
                    ),
                    drawn.as_slice(),
                ),
            };
            for control in rest {
                this.push(at, of_control(strings, *control));
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
fn of_control(strings: &Strings, control: Control) -> Node {
    Node {
        path: String::new(),
        role: access_roles::number_of(control.role),
        name: said(strings, control.name),
        states: access_roles::words_of(control.state),
        announced: access_roles::is_announced(control.state),
        parent: 0,
        children: Vec::new(),
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
