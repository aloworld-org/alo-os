//! **One thing in the tree, answering for itself** — `org.a11y.atspi.Accessible`
//! for one node of [`crate::access_nodes::ReadAloudTree`].
//!
//! Split out of `crate::access_bus` on 2026-11-13, when the tree gained a
//! second way of being read — `org.a11y.atspi.Cache`, `crate::access_cache` —
//! which has to say exactly what each node says here. Both read [`ReadAloud`],
//! so the bulk answer and the one-at-a-time answers cannot drift apart.
//!
//! # A reference is one value, not two
//!
//! Every answer that names another thing in the tree — `GetChildAtIndex`,
//! `GetApplication`, the `Parent` property, each entry of `GetChildren` — is a
//! **reference**: a bus name and an object path *together*, the D-Bus
//! structure `(so)`. at-spi2-core's own `xml/Accessible.xml` gives
//! `GetChildAtIndex` and `GetApplication` exactly one out argument of type
//! `(so)`, and libatspi refuses anything else
//! (`_atspi_dbus_return_accessible_from_message`: *strange signature*).
//!
//! **A Rust tuple returned from a zbus method is not that.** zbus makes a
//! returned tuple the message's argument list, so `(String, OwnedObjectPath)`
//! goes out as two loose arguments, `so`. That is what this file answered
//! until 2026-11-13, and Orca on a booted machine logged *strange signature so*
//! for every child it asked for. So a reference is returned wrapped once more —
//! `((name, path),)`, one argument that is a structure — and
//! `access_answers_tests` reads the signature back off a real bus.

use std::collections::HashMap;

use zbus::zvariant::{ObjectPath, OwnedObjectPath};

use crate::access_action::ACTION;
use crate::access_nodes::{Node, ROOT, ReadAloudTree};

/// What an application that is not embedded answers with instead of a child.
pub(crate) const NOTHING: &str = "/org/a11y/atspi/null";

/// The registry every application's root is embedded in.
pub(crate) const REGISTRY: &str = "org.a11y.atspi.Registry";

/// The interface every thing in a tree answers.
pub(crate) const ACCESSIBLE: &str = "org.a11y.atspi.Accessible";

/// A thing in a tree as at-spi2 names one: the bus name that holds it and its
/// path there. On the wire this is the structure `(so)`.
pub(crate) type Reference = (String, OwnedObjectPath);

/// One thing in the tree, answering for itself.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReadAloud {
    /// Where it is on the bus, and who holds it.
    pub(crate) itself: Reference,
    /// The shell's own root, which every node belongs to.
    pub(crate) application: Reference,
    /// What kind of thing it is.
    pub(crate) role: u32,
    /// What it is called, in the person's language.
    pub(crate) name: String,
    /// The line drawn under it, where there is one.
    pub(crate) description: String,
    /// Where it is among what its parent holds, `-1` for the shell itself.
    pub(crate) index_in_parent: i32,
    /// The two words `GetState` answers with, low word first.
    pub(crate) states: Vec<u32>,
    /// What a reader is told besides the name — the `live` attribute, on a
    /// surface's own node the surface's typed name
    /// ([`alo_access::THE_SURFACE_ATTRIBUTE`]), and what the node itself
    /// carries ([`ReadAloudTree::attributes_of`]).
    pub(crate) attributes: HashMap<String, String>,
    /// What hangs under it, in reading order.
    pub(crate) children: Vec<Reference>,
    /// What it hangs under. For the shell itself that is the registry's own
    /// root — the desktop it was embedded in — which is what a toolkit's bridge
    /// answers too. **Until 2026-11-13 the shell said it hung under itself.**
    pub(crate) parent: Reference,
    /// Whether a reader can press it, which is whether `Action` is served
    /// beside this.
    pub(crate) pressable: bool,
}

impl ReadAloud {
    /// One node of `tree`, ready to answer as `answers_as`.
    pub(crate) fn of(node: &Node, tree: &ReadAloudTree, answers_as: &str) -> Self {
        let at = |index: usize| {
            (
                answers_as.to_owned(),
                tree.nodes()
                    .get(index)
                    .and_then(|node| OwnedObjectPath::try_from(node.path.as_str()).ok())
                    .unwrap_or_else(nothing),
            )
        };
        let attributes = tree.attributes_of(node);
        let is_the_shell = node.path == ROOT;
        let index_in_parent = if is_the_shell {
            -1
        } else {
            tree.nodes()
                .get(node.parent)
                .and_then(|parent| {
                    parent.children.iter().position(|child| {
                        tree.nodes()
                            .get(*child)
                            .is_some_and(|it| it.path == node.path)
                    })
                })
                .and_then(|at| i32::try_from(at).ok())
                .unwrap_or(-1)
        };
        let parent = if is_the_shell {
            (REGISTRY.to_owned(), root())
        } else {
            at(node.parent)
        };
        Self {
            itself: (
                answers_as.to_owned(),
                OwnedObjectPath::try_from(node.path.as_str()).unwrap_or_else(|_| nothing()),
            ),
            application: (answers_as.to_owned(), root()),
            role: node.role,
            name: node.name.clone(),
            description: node.description.clone(),
            index_in_parent,
            states: node.states.to_vec(),
            attributes,
            children: node.children.iter().map(|child| at(*child)).collect(),
            parent,
            pressable: node.press.is_some(),
        }
    }

    /// The interfaces this answers: it is read, and where a reader can press
    /// it, it is pressed. Nothing here is a document, a picture or a place on a
    /// screen.
    pub(crate) fn interfaces(&self) -> Vec<String> {
        let mut answers = vec![ACCESSIBLE.to_owned()];
        if self.pressable {
            answers.push(ACTION.to_owned());
        }
        answers
    }

    /// How many things are inside this.
    pub(crate) fn how_many_inside(&self) -> i32 {
        i32::try_from(self.children.len()).unwrap_or(i32::MAX)
    }
}

/// The object an application answers with where there is no thing at all.
pub(crate) fn nothing() -> OwnedObjectPath {
    ObjectPath::try_from(NOTHING)
        .map(Into::into)
        .unwrap_or_default()
}

/// Where every application's tree starts.
fn root() -> OwnedObjectPath {
    ObjectPath::try_from(ROOT)
        .map(Into::into)
        .unwrap_or_else(|_| nothing())
}

#[zbus::interface(name = "org.a11y.atspi.Accessible")]
impl ReadAloud {
    /// What kind of thing this is.
    fn get_role(&self) -> u32 {
        self.role
    }

    /// The states, in the two words the interface answers with.
    fn get_state(&self) -> Vec<u32> {
        self.states.clone()
    }

    /// What this answers.
    fn get_interfaces(&self) -> Vec<String> {
        self.interfaces()
    }

    /// What is inside this, in reading order: `a(so)`.
    fn get_children(&self) -> Vec<Reference> {
        self.children.clone()
    }

    /// One of them, or nothing at all where there is no such child — which is
    /// how the interface says *not there*, rather than an error.
    ///
    /// **One argument, `(so)`**, which is why it is wrapped in a tuple of one:
    /// see this file's own documentation.
    fn get_child_at_index(&self, index: i32) -> (Reference,) {
        (usize::try_from(index)
            .ok()
            .and_then(|index| self.children.get(index))
            .cloned()
            .unwrap_or_else(|| (String::new(), nothing())),)
    }

    /// The application this belongs to, which is the shell's own root: `(so)`.
    fn get_application(&self) -> (Reference,) {
        (self.application.clone(),)
    }

    /// How this relates to other things — *labelled by*, *controls* — which is
    /// nothing: every control here carries its own name. `a(ua(so))`, empty,
    /// rather than an error a reader would log for every control it reads.
    fn get_relation_set(&self) -> Vec<(u32, Vec<Reference>)> {
        Vec::new()
    }

    /// What a reader is told besides the name.
    fn get_attributes(&self) -> HashMap<String, String> {
        self.attributes.clone()
    }

    /// What it is called.
    #[zbus(property)]
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The line a surface draws under a control's name, where it draws one —
    /// *Installed* under an application on the *Open with* sheet — and
    /// nothing anywhere else: a description repeating the name would be read
    /// twice.
    #[zbus(property)]
    fn description(&self) -> String {
        self.description.clone()
    }

    /// Where this is among what its parent holds, which is how a reader says
    /// *2 of 3* on a list.
    fn get_index_in_parent(&self) -> i32 {
        self.index_in_parent
    }

    /// How many things are inside this.
    #[zbus(property)]
    fn child_count(&self) -> i32 {
        self.how_many_inside()
    }

    /// What this is inside. A property is one variant, so this crosses the bus
    /// as `v` holding `(so)` without being wrapped again.
    #[zbus(property)]
    fn parent(&self) -> Reference {
        self.parent.clone()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "access_answers_tests.rs"]
mod tests;
