//! Installing, updating and removing an application.
//!
//! `docs/features.md` v0.5: **install applications, sandboxed, from Flathub or a
//! repository the organisation runs; update and remove them.** ADR 0005 rents
//! the tool that does it, and this crate is everything alo OS decides around
//! that tool — which places may be installed from, what a person is shown while
//! it happens, what an installed application is given (nothing), when an update
//! may be applied (never while the application is open), and what removing one
//! ends (every grant it held, and every grant over it).
//!
//! It is not a package manager. The places applications come from are the
//! rented tool's own configuration, read and never written ([`source`]); the
//! tool checks every signature, and this crate refuses a place set up not to
//! ([`Enabled::usable`]).
//!
//! # The four promises, and where each is kept
//!
//! | | |
//! |---|---|
//! | **It arrives with no grants** | [`installing`](mod@installing) takes nothing that could make one |
//! | **What leaves is on the indicator, named** | [`shown`] holds each act to its `alo_egress::Errand` |
//! | **An update is offered, never applied while it runs** | [`updating`] — [`Offer`] has no public constructor, and [`applying`] refuses an open application |
//! | **Removing ends its grants in the same act** | [`removing`] |
//!
//! And the fifth, for the agent: it may **propose** an installation through one
//! verb a person approves ([`verbs`]), and nothing else here is reachable from
//! a call.
//!
//! # Two steps around the indicator
//!
//! Every act that leaves the machine is two calls with the indicator between
//! them, in `alo-keeping-up`'s shape: the first asks everything that can be
//! asked without leaving and answers with the `alo_egress::OnItsOwn` to show;
//! the caller puts it on the machine's one `alo_egress::Indicator`; the second
//! takes the `alo_egress::Underway` that came back, asks again, and reaches the
//! tool while the line is still showing.
//!
//! ```
//! # use alo_software::*;
//! # use alo_applications::Application;
//! # use alo_egress::Indicator;
//! # use std::cell::RefCell;
//! # use std::time::{Duration, SystemTime};
//! # struct Tool(RefCell<Vec<String>>);
//! # impl alo_software::Tool for Tool {
//! #     fn sources(&self) -> Result<Vec<Configured>, Failed> {
//! #         Ok(vec![Configured {
//! #             name: "flathub".into(),
//! #             address: "https://dl.flathub.org/repo/".into(),
//! #             checks_signatures: true,
//! #             switched_off: false,
//! #         }])
//! #     }
//! #     fn installed(&self) -> Result<Vec<String>, Failed> { Ok(self.0.borrow().clone()) }
//! #     fn open(&self) -> Result<Vec<String>, Failed> { Ok(Vec::new()) }
//! #     fn install(&self, _: &SourceName, app: &Application) -> Result<(), Failed> {
//! #         self.0.borrow_mut().push(app.identifier().to_owned());
//! #         Ok(())
//! #     }
//! #     fn updates(&self, _: &SourceName) -> Result<Vec<String>, Failed> { Ok(Vec::new()) }
//! #     fn update(&self, _: &Application) -> Result<(), Failed> { Ok(()) }
//! #     fn remove(&self, _: &Application) -> Result<(), Failed> { Ok(()) }
//! # }
//! # let tool = Tool(RefCell::new(Vec::new()));
//! # let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
//! let enabled = Enabled::read(&tool, Bound::Nobodys).expect("the tool answers");
//! let wanted = Wanted::by_hand(
//!     Application::identified("org.gnome.TextEditor").expect("an identifier"),
//!     "flathub",
//! );
//!
//! let mut indicator = Indicator::default();
//! let errand = installing(&enabled, &wanted, &tool).expect("nothing refuses it");
//! let underway = indicator.beginning_on_its_own(errand, now);
//! assert!(!indicator.is_quiet());
//!
//! let installed = install(&underway, &enabled, wanted, &tool).expect("installed");
//! indicator.ended_on_its_own(underway);
//! assert_eq!(installed.application().identifier(), "org.gnome.TextEditor");
//! assert!(indicator.is_quiet());
//! ```
//!
//! # Nothing a person reads names the machinery
//!
//! Every sentence is declared in [`words`] and said through `alo-strings`; a
//! test there holds that none of them, and no note beside one, names the tool,
//! the service applications come from by its tooling, a repository, a remote or
//! a signature key. What the tool itself said when something failed is kept for
//! whoever fixes the machine ([`NotDone::DidNotAnswer`]) and never shown.
//!
//! # What is here, and what is not
//!
//! **The rented door** ([`rented`], with [`asked`] and [`heard`]) starts the
//! tool directly — no shell — with arguments built from checked values, and
//! reads its answer. **Not here:** reading an organisation's rule about which
//! places are permitted out of the machine's description; [`Bound`] is the
//! value that rule becomes, and `alo-agentd` reads that file. Nor the surface a
//! person installs from, which is the shell's.
//!
//! # What a fresh machine has
//!
//! [`Shipped`] is the decided list — a web browser, a file manager and what
//! opens its archives, a text editor, an image viewer, a document viewer and a
//! terminal — read from `shipped.toml` beside this crate's manifest, which the
//! installer plan reads too. Each is a [`Pinned`] upstream application whose
//! [`Pinned::wanted`] goes through the same two steps as any other, so a
//! person updates and removes it the same way, the browser included. The
//! terminal is a person's own: `alo_capability` refuses an agent any grant over
//! it and any call naming it (ADR 0043).

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod asked;
pub mod bound;
pub mod enabled;
pub mod heard;
pub mod installing;
pub mod refusing;
pub mod removing;
pub mod rented;
pub mod role;
pub mod shipped;
pub mod shown;
pub mod source;
pub mod tool;
pub mod updating;
pub mod verbs;
pub mod words;

#[cfg(test)]
mod testing;

pub use bound::{Bound, SetBy};
pub use enabled::Enabled;
pub use installing::{Installed, Wanted, install, installing};
pub use refusing::NotDone;
pub use removing::{Removed, remove};
pub use rented::TheRentedTool;
pub use role::Role;
pub use shipped::{NotShipped, Pinned, Shipped};
pub use shown::{NotShown, Stopped};
pub use source::{Configured, Source, SourceName};
pub use tool::{Failed, Tool};
pub use updating::{Offer, Updated, apply, applying, looking_for_updates, offered};
pub use verbs::{Declaring, INSTALL_APPLICATION, NotAnInstallation, approved, software_verbs};
pub use words::{Word, WordsError, declare_into, software_words};
