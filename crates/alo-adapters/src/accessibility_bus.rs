//! The session's accessibility tree, reached over its own bus.
//!
//! The one file in the fallback that touches the machine. Applications publish
//! their trees on the accessibility bus that at-spi2 runs beside the person's
//! session — the tree Orca reads (ADR 0011: rented, configured, never
//! patched). [`AccessibleSession`] asks it exactly the questions
//! [`AccessibilityTree`] lists, one method call each, and nothing else:
//!
//! - **no position, size or picture is ever asked for** — the `Component` and
//!   `Image` interfaces are not called, because nothing here has a use for
//!   where a thing is;
//! - **nothing is subscribed to** — no event is registered with the registry
//!   and no match rule is added, so nothing arrives that was not asked for, and
//!   every call is a plain method call rather than a proxy, which would ask the
//!   bus for signals;
//! - **text is asked only when the walk asks it**, which is never of a password
//!   field (`crate::walking`).
//!
//! It is made for one turn and dropped with it: the connection closes when it
//! does, so nothing stays attached to a person's applications between turns.

use std::time::Duration;

use alo_portals::Sandboxes;
use zbus::blocking::Connection;
use zbus::blocking::connection::Builder;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

use crate::accessibility_tree::{AccessibilityTree, Facts, NodeAt, Running, TreeFault};
use crate::whose_connection::application_of;

/// How long an application has to answer one question.
///
/// A tree is read with many small questions, so each is short: an application
/// that takes this long over one is not going to be read in a time a person
/// would wait for.
pub const HOW_LONG: Duration = Duration::from_secs(5);

/// The registry every application's root is listed under.
const REGISTRY: &str = "org.a11y.atspi.Registry";

/// The object an application's tree starts at.
const ROOT: &str = "/org/a11y/atspi/accessible/root";

/// The interface every thing in a tree answers.
const ACCESSIBLE: &str = "org.a11y.atspi.Accessible";

/// The interface a thing with text answers.
const TEXT: &str = "org.a11y.atspi.Text";

/// The interface a thing that can be acted on answers.
const ACTION: &str = "org.a11y.atspi.Action";

/// The accessibility tree of one session, for one turn.
#[derive(Debug)]
pub struct AccessibleSession {
    /// The connection to the accessibility bus.
    bus: Connection,
    /// Where each process's sandbox is read.
    sandboxes: Sandboxes,
}

impl AccessibleSession {
    /// The signed-in person's own session's tree.
    ///
    /// # Errors
    /// [`TreeFault::NotThere`] when the session has no accessibility bus.
    pub fn of_this_person() -> Result<Self, TreeFault> {
        let session = Builder::session()
            .map(|builder| builder.method_timeout(HOW_LONG))
            .and_then(Builder::build)
            .map_err(|_| TreeFault::NotThere)?;
        let address = Self::address_on(&session)?;
        Self::at(&address, Sandboxes::of_this_machine())
    }

    /// The accessibility bus the session bus at `session` announces — for a
    /// test with a session of its own.
    ///
    /// # Errors
    /// [`TreeFault::NotThere`] when nothing announces one.
    pub fn announced_on(session: &str) -> Result<String, TreeFault> {
        let session = Builder::address(session)
            .map(|builder| builder.method_timeout(HOW_LONG))
            .and_then(Builder::build)
            .map_err(|_| TreeFault::NotThere)?;
        Self::address_on(&session)
    }

    /// The tree on the accessibility bus at `address`, with sandboxes read
    /// under `sandboxes`.
    ///
    /// # Errors
    /// [`TreeFault::NotThere`] when nothing answers there.
    pub fn at(address: &str, sandboxes: Sandboxes) -> Result<Self, TreeFault> {
        let bus = Builder::address(address)
            .map(|builder| builder.method_timeout(HOW_LONG))
            .and_then(Builder::build)
            .map_err(|_| TreeFault::NotThere)?;
        Ok(Self { bus, sandboxes })
    }

    /// The name this reader holds on the accessibility bus — what a person
    /// auditing the bus sees every question here come from.
    #[must_use]
    pub fn asks_as(&self) -> Option<String> {
        self.bus.unique_name().map(ToString::to_string)
    }

    /// The accessibility bus a session bus announces.
    fn address_on(session: &Connection) -> Result<String, TreeFault> {
        session
            .call_method(
                Some("org.a11y.Bus"),
                "/org/a11y/bus",
                Some("org.a11y.Bus"),
                "GetAddress",
                &(),
            )
            .map_err(|_| TreeFault::NotThere)?
            .body()
            .deserialize::<String>()
            .map_err(|_| TreeFault::NotThere)
    }

    /// One method call, answered with `T`.
    fn ask<T, B>(
        &self,
        at: &NodeAt,
        interface: &str,
        method: &str,
        body: &B,
    ) -> Result<T, TreeFault>
    where
        T: for<'d> zbus::export::serde::Deserialize<'d> + zbus::zvariant::Type,
        B: zbus::export::serde::Serialize + zbus::zvariant::DynamicType,
    {
        self.bus
            .call_method(
                Some(at.holder.as_str()),
                at.object.as_str(),
                Some(interface),
                method,
                body,
            )
            .map_err(|why| heard(&why))?
            .body()
            .deserialize::<T>()
            .map_err(|_| TreeFault::Vanished)
    }
}

impl AccessibilityTree for AccessibleSession {
    fn applications(&self) -> Result<Vec<Running>, TreeFault> {
        let registry = NodeAt {
            holder: REGISTRY.to_owned(),
            object: ROOT.to_owned(),
        };
        let listed: Vec<(String, OwnedObjectPath)> = self
            .ask(&registry, ACCESSIBLE, "GetChildren", &())
            .map_err(|fault| match fault {
                TreeFault::Vanished => TreeFault::NotThere,
                other => other,
            })?;
        Ok(listed
            .into_iter()
            .map(|(holder, object)| Running {
                is: application_of(&self.bus, &holder, &self.sandboxes),
                at: NodeAt {
                    holder,
                    object: object.to_string(),
                },
            })
            .collect())
    }

    fn facts(&self, at: &NodeAt) -> Result<Facts, TreeFault> {
        let role: u32 = self.ask(at, ACCESSIBLE, "GetRole", &())?;
        let name: OwnedValue = self.ask(
            at,
            "org.freedesktop.DBus.Properties",
            "Get",
            &(ACCESSIBLE, "Name"),
        )?;
        let name = String::try_from(name).unwrap_or_default();
        let states: Vec<u32> = self.ask(at, ACCESSIBLE, "GetState", &())?;
        let interfaces: Vec<String> = self.ask(at, ACCESSIBLE, "GetInterfaces", &())?;
        let count: OwnedValue = self.ask(
            at,
            "org.freedesktop.DBus.Properties",
            "Get",
            &(ACCESSIBLE, "ChildCount"),
        )?;
        let count = i32::try_from(count).unwrap_or(0).max(0);
        // A list of a hundred thousand rows is not asked for row by row: past
        // what a walk reads at once, the rest are said to be there and unread.
        let most = i32::try_from(crate::walking::MOST_THINGS).unwrap_or(i32::MAX);
        let unlisted_children = count > most;
        let count = count.min(most);
        let mut children = Vec::new();
        for index in 0..count {
            let (holder, object): (String, OwnedObjectPath) =
                match self.ask(at, ACCESSIBLE, "GetChildAtIndex", &(index,)) {
                    Ok(child) => child,
                    Err(TreeFault::Vanished) => continue,
                    Err(fault) => return Err(fault),
                };
            // A tree answers a child it does not have with an empty holder
            // and the null object.
            if holder.is_empty() || object.as_str() == "/org/a11y/atspi/null" {
                continue;
            }
            children.push(NodeAt {
                holder,
                object: object.to_string(),
            });
        }
        Ok(Facts {
            role,
            name,
            states,
            children,
            has_text: interfaces.iter().any(|interface| interface == TEXT),
            unlisted_children,
        })
    }

    fn text(&self, at: &NodeAt) -> Result<String, TreeFault> {
        self.ask(at, TEXT, "GetText", &(0_i32, -1_i32))
    }

    fn actions(&self, at: &NodeAt) -> Result<Vec<String>, TreeFault> {
        let actions: Vec<(String, String, String)> = self.ask(at, ACTION, "GetActions", &())?;
        Ok(actions.into_iter().map(|(name, _, _)| name).collect())
    }

    fn act(&self, at: &NodeAt, action: usize) -> Result<bool, TreeFault> {
        let index = i32::try_from(action).map_err(|_| TreeFault::Vanished)?;
        self.ask(at, ACTION, "DoAction", &(index,))
    }
}

/// What an error from the bus means for one question.
fn heard(why: &zbus::Error) -> TreeFault {
    match why {
        zbus::Error::MethodError(name, _, _) => {
            let name = name.as_str();
            let bus = |suffix: &str| name == format!("org.freedesktop.DBus.Error.{suffix}");
            if bus("NoReply") || bus("Timeout") || bus("TimedOut") {
                TreeFault::DidNotAnswer
            } else {
                TreeFault::Vanished
            }
        }
        zbus::Error::InputOutput(io) if io.kind() == std::io::ErrorKind::TimedOut => {
            TreeFault::DidNotAnswer
        }
        _ => TreeFault::NotThere,
    }
}
