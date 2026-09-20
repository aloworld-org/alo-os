//! **The tree, on the accessibility bus a screen reader reads.**
//!
//! `org.a11y.atspi.Accessible`, answered for every node of
//! [`crate::access_nodes::ReadAloudTree`], on the bus at-spi2 runs beside the
//! person's session — the bus Orca reads (ADR 0011: rented, configured, never
//! patched, and **no reader of our own**).
//!
//! # What is answered, and what is not
//!
//! The questions a reader asks of a thing that is read and pressed:
//! `GetRole`, `GetState`, `GetInterfaces`, `GetChildren`, `GetChildAtIndex`,
//! `GetAttributes`, and the `Name`, `Description`, `ChildCount` and `Parent`
//! properties. **Nothing else is served**: no `Component`, so nothing here is
//! asked or answered about where a thing is on the screen, and no `Text`,
//! because none of these controls is a document a reader reads through — what
//! each says is its name, in the person's language, and a name is not text a
//! caret moves through.
//!
//! **Nothing is subscribed to and nothing is emitted.** A surface that changed
//! while nobody was looking would be an event, and this crate has no event to
//! send that the surfaces themselves do not already draw.
//!
//! # Embedding is asking, not announcing
//!
//! An application joins the tree by asking the registry to embed it —
//! `org.a11y.atspi.Socket.Embed` with its own name and root — which is how
//! every toolkit's bridge joins it. [`ReadAloudBus::embedded`] is that one
//! call. A machine with no registry running is a machine with no reader
//! running, and the shell serves its tree there just the same: what a reader
//! needs is for the tree to be there when it arrives.

use std::collections::HashMap;

use zbus::blocking::Connection;
use zbus::blocking::connection::Builder;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

use crate::access_nodes::{Node, ROOT, ReadAloudTree};
use crate::access_roles::LIVE;

/// What an application that is not embedded answers with instead of a child.
const NOTHING: &str = "/org/a11y/atspi/null";

/// The registry every application's root is embedded in.
const REGISTRY: &str = "org.a11y.atspi.Registry";

/// The interface an application is embedded through.
const SOCKET: &str = "org.a11y.atspi.Socket";

/// The interface every thing in a tree answers.
const ACCESSIBLE: &str = "org.a11y.atspi.Accessible";

/// Why the tree is not on the bus.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotRead {
    /// There is no accessibility bus at that address, or it refused.
    #[error("the accessibility bus did not answer")]
    NoBus,
    /// The bus gave this connection no name of its own, so nothing on the tree
    /// could be addressed.
    #[error("the accessibility bus named nothing")]
    NoName,
    /// One of the nodes could not be served.
    #[error("the tree could not be served")]
    NotServed,
    /// Nothing answered as the registry, so nothing was embedded.
    #[error("no registry answered")]
    NoRegistry,
}

/// **This machine's surfaces, answered on the accessibility bus.**
///
/// The tree lives as long as this does and goes when it goes: dropping it
/// closes the connection, and a reader that asks afterwards is told the
/// application is gone rather than reading a screen that is not there.
#[derive(Debug)]
pub struct ReadAloudBus {
    /// The connection the tree is answered on.
    connection: Connection,
    /// The name this connection answers as — what every child in the tree is
    /// addressed by.
    answers_as: String,
}

impl ReadAloudBus {
    /// Serve `tree` on the accessibility bus at `address`.
    ///
    /// # Errors
    /// [`NotRead::NoBus`] when nothing answers there, [`NotRead::NoName`] when
    /// the bus names this connection nothing, and [`NotRead::NotServed`] when a
    /// node could not be put on it.
    pub fn serving(address: &str, tree: &ReadAloudTree) -> Result<Self, NotRead> {
        let connection = Builder::address(address)
            .and_then(Builder::build)
            .map_err(|_| NotRead::NoBus)?;
        let answers_as = connection
            .unique_name()
            .map(ToString::to_string)
            .ok_or(NotRead::NoName)?;
        {
            let server = connection.object_server();
            server
                .at(ROOT, TheShellItself { id: 0 })
                .map_err(|_| NotRead::NotServed)
                .and_then(|put| put.then_some(()).ok_or(NotRead::NotServed))?;
            for node in tree.nodes() {
                let served = ReadAloud::of(node, tree, &answers_as);
                server
                    .at(node.path.as_str(), served)
                    .map_err(|_| NotRead::NotServed)
                    .and_then(|put| put.then_some(()).ok_or(NotRead::NotServed))?;
            }
        }
        Ok(Self {
            connection,
            answers_as,
        })
    }

    /// The name this machine's tree answers as.
    #[must_use]
    pub fn answers_as(&self) -> &str {
        &self.answers_as
    }

    /// Ask the registry to embed this tree, so a reader walking the session
    /// finds it.
    ///
    /// # Errors
    /// [`NotRead::NoRegistry`] when nothing answers as the registry — which is
    /// a session with no reader in it, not a fault in the tree.
    pub fn embedded(&self) -> Result<(), NotRead> {
        let root = ObjectPath::try_from(ROOT).map_err(|_| NotRead::NoRegistry)?;
        self.connection
            .call_method(
                Some(REGISTRY),
                ROOT,
                Some(SOCKET),
                "Embed",
                &((self.answers_as.as_str(), &root),),
            )
            .map(drop)
            .map_err(|_| NotRead::NoRegistry)
    }
}

/// The shell itself, as an application joined to the session's tree.
///
/// The registry gives each application it embeds a number and reads back what
/// it is: this is that, and nothing a person is ever told. **The toolkit is
/// named as ours** rather than as a rented one, because a reader that decides
/// how to read a window by its toolkit would otherwise be told a lie about
/// which one drew it.
struct TheShellItself {
    /// The number the registry gave this machine's tree.
    id: i32,
}

#[zbus::interface(name = "org.a11y.atspi.Application")]
impl TheShellItself {
    /// What drew the surfaces: alo OS's own shell.
    #[zbus(property)]
    fn toolkit_name(&self) -> String {
        "alo".to_owned()
    }

    /// Which alo OS this is.
    #[zbus(property)]
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    /// Which AT-SPI this speaks.
    #[zbus(property)]
    fn atspi_version(&self) -> String {
        "2.1".to_owned()
    }

    /// The number the registry gave this tree.
    #[zbus(property)]
    fn id(&self) -> i32 {
        self.id
    }

    /// The registry's own numbering, taken as given.
    #[zbus(property)]
    fn set_id(&mut self, id: i32) {
        self.id = id;
    }

    /// The language the tree is published in, which is the person's own and is
    /// answered by the strings the tree was built from rather than by a locale
    /// this crate reads.
    fn get_locale(&self, _category: u32) -> String {
        String::new()
    }
}

/// One thing in the tree, answering for itself.
struct ReadAloud {
    /// What kind of thing it is.
    role: u32,
    /// What it is called, in the person's language.
    name: String,
    /// The two words `GetState` answers with, low word first.
    states: Vec<u32>,
    /// What a reader is told besides the name — the `live` attribute, and
    /// nothing else.
    attributes: HashMap<String, String>,
    /// What hangs under it, in reading order.
    children: Vec<(String, OwnedObjectPath)>,
    /// What it hangs under.
    parent: (String, OwnedObjectPath),
}

impl ReadAloud {
    /// One node of `tree`, ready to answer as `answers_as`.
    fn of(node: &Node, tree: &ReadAloudTree, answers_as: &str) -> Self {
        let at = |index: usize| {
            (
                answers_as.to_owned(),
                tree.nodes()
                    .get(index)
                    .and_then(|node| OwnedObjectPath::try_from(node.path.as_str()).ok())
                    .unwrap_or_else(nothing),
            )
        };
        let mut attributes = HashMap::new();
        if node.announced {
            attributes.insert(LIVE.0.to_owned(), LIVE.1.to_owned());
        }
        Self {
            role: node.role,
            name: node.name.clone(),
            states: node.states.to_vec(),
            attributes,
            children: node.children.iter().map(|child| at(*child)).collect(),
            parent: at(node.parent),
        }
    }
}

/// The object an application answers with where there is no thing at all.
fn nothing() -> OwnedObjectPath {
    ObjectPath::try_from(NOTHING)
        .map(Into::into)
        .unwrap_or_default()
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

    /// What this answers. One interface: it is read, and nothing here is a
    /// document, a picture or a place on a screen.
    fn get_interfaces(&self) -> Vec<String> {
        vec![ACCESSIBLE.to_owned()]
    }

    /// What is inside this, in reading order.
    fn get_children(&self) -> Vec<(String, OwnedObjectPath)> {
        self.children.clone()
    }

    /// One of them, or nothing at all where there is no such child — which is
    /// how the interface says *not there*, rather than an error.
    fn get_child_at_index(&self, index: i32) -> (String, OwnedObjectPath) {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.children.get(index))
            .cloned()
            .unwrap_or_else(|| (String::new(), nothing()))
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

    /// Nothing: a control's name is the whole of what it is called, and a
    /// description repeating it would be read twice.
    #[zbus(property)]
    fn description(&self) -> String {
        String::new()
    }

    /// How many things are inside this.
    #[zbus(property)]
    fn child_count(&self) -> i32 {
        i32::try_from(self.children.len()).unwrap_or(i32::MAX)
    }

    /// What this is inside.
    #[zbus(property)]
    fn parent(&self) -> (String, OwnedObjectPath) {
        self.parent.clone()
    }
}
