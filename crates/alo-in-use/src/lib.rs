//! What is watching or listening, right now.
//!
//! Law 1 is about egress: *nothing leaves silently*, and `alo-egress` and
//! `alo-indicator` are that law as working code. `docs/features.md` makes the
//! same instinct a promise about the room a person is sitting in — ★ *a visible
//! indicator whenever the screen, camera or microphone is in use — by any
//! application, including ours* — and this crate is that promise's model.
//!
//! It comes **before** anything that captures, which is the plan's own order
//! and is the point of it: nothing can be built that captures without the
//! indicator already existing to show it.
//!
//! - [`Used`] — the three things a machine can watch or listen with;
//! - [`By`] — who is using one: an application, an agent, alo OS itself, or
//!   something this machine can see and cannot name;
//! - [`Use`] — one of them, as the media server records it;
//! - [`InUse`] — every current use, read from the machine's own media server
//!   and from nothing else;
//! - [`Line`] — what the indicator shows for one use: a [`Mark`], a sentence, a
//!   [`Position`], and only then a colour;
//! - [`Streams`] and [`TheMediaServer`] — the one question, and the one thing
//!   on a machine that answers it;
//! - [`NotHeard`] — the refusal in words when the machine cannot answer,
//!   because an indicator that quietly showed nothing would look exactly like a
//!   machine on which nothing is watching.
//!
//! ```
//! use alo_applications::Application;
//! use alo_appearance::Token;
//! use alo_capability::Grantee;
//! use alo_in_use::{By, InUse, NotHeard, Streams, Use, UseId, Used, in_use_words};
//! use alo_strings::Strings;
//!
//! /// A media server with a camera open and an agent looking at the screen.
//! struct AMachine;
//! impl Streams for AMachine {
//!     fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard> {
//!         Ok(vec![
//!             Use::of(
//!                 UseId::recorded(42),
//!                 Used::Camera,
//!                 By::an_application(
//!                     Application::called("com.example.VideoCall", "Video Call")
//!                         .expect("an identifier a verb could name"),
//!                 ),
//!             ),
//!             Use::of(
//!                 UseId::recorded(43),
//!                 Used::Screen,
//!                 By::the_agent(&Grantee::named("@alo")).expect("an agent"),
//!             ),
//!         ])
//!     }
//! }
//!
//! let strings = Strings::of(in_use_words()?);
//! let in_use = InUse::read_from(&mut AMachine)?;
//! assert!(!in_use.is_quiet());
//!
//! // The screen's line comes first, and it is the agent's: terracotta, with
//! // the agent's mark, and saying so in words as well (ADR 0010).
//! let lines = in_use.lines();
//! let screen = lines.first().expect("the screen is in use");
//! assert_eq!(screen.colour(), Token::Terracotta);
//! assert!(screen.the_agents_dot());
//! assert!(screen.said(&strings).text().contains("the agent"));
//!
//! // The camera's is not the agent's, and is not terracotta.
//! let camera = lines.get(1).expect("the camera is in use");
//! assert_eq!(camera.colour(), Token::Navy);
//! assert!(!camera.the_agents_dot());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`used`] | The screen, the camera, the microphone |
//! | [`by`] | Who is using one |
//! | [`uses`] | One current use |
//! | [`in_use`] | Every current use, read from the machine's media server |
//! | [`mod@line`] | What the indicator shows for one |
//! | [`mark`] | The shape beside a line |
//! | [`position`] | Where a line sits |
//! | [`streams`] | The one question a media server is asked |
//! | [`media_server`] | The one thing on a machine that answers it |
//! | [`heard`] | The server's own record, read |
//! | [`refusing`] | What a person reads when it cannot be answered |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! # There is one door, and it is the machine's own media server
//!
//! The plan's acceptance for this task is a sentence about provenance: every
//! current use is **read from the media server's own record of open streams
//! rather than from what applications say they are doing**. An application that
//! wanted its camera line to disappear would only have to stop announcing
//! itself, and a list kept from announcements would have nothing to say about
//! it. The server knows, because the stream is open through it.
//!
//! So [`InUse::read_from`] is the only constructor. There is no public field,
//! no `From`, no deserialiser, and no way to build one from a list somebody
//! assembled; the compile-fail examples on [`InUse`] are that argument as tests.
//! [`heard`] carries the other half — which properties of a record the server
//! itself vouches for, and which are only what a client said about itself.
//!
//! # Nothing is hidden, nothing is trusted, and nothing turns it off
//!
//! There is **no variant that hides a use**: [`InUse::read_from`] keeps
//! everything the server answered, a running source with nothing reading it is
//! still listed rather than dropped, and a client the machine cannot name is
//! listed as [`By::something_on_this_machine`] rather than left off.
//!
//! There is **no allow-list of trusted applications**. Nothing anywhere in this
//! crate takes a list of names, and alo OS's own captures appear like anybody
//! else's because nothing here can tell them apart in order to do otherwise.
//!
//! There is **no setting that turns the indicator off**, and there is nothing
//! to add one to: an indicator with an off switch is not an indicator, it is a
//! decoration.
//!
//! # It shows; it never decides
//!
//! Nothing here asks whether something *may* use the camera. That is
//! `alo-portals` judging an application's grant against `alo_capability::Facility`
//! (ADR 0040), and it happens before any of this. A [`Use`] is a fact about
//! what is happening, and a fact does not have a policy — so there is no verb
//! here, no grant, no approval, and no way for an agent to ask for any of it.
//!
//! # Two indicators, and neither is drawn as the other
//!
//! *Something left this machine* and *the microphone is on* are different
//! warnings and they are two lines. `alo-indicator` is the first;
//! this is the second; `alo-egress` and `alo-indicator` are dev-dependencies
//! here and nothing else, so nothing on this indicator can be drawn from what
//! is leaving and nothing there can be drawn from what is in use.
//! `tests/two_lines_and_neither_is_the_other.rs` is that held as arithmetic
//! over both vocabularies rather than as a paragraph.
//!
//! # Nothing here says anything in English by itself
//!
//! Every sentence a person can be shown is declared in [`words`] and answered
//! through a `said` in the language they read: the three lines, the sentence
//! for a quiet room, the three clauses that say who, and the three refusals.
//! The English beside each is what a machine with no translations shows, and
//! `alo-strings` marks it as English when it does.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod by;
pub mod heard;
pub mod in_use;
pub mod line;
pub mod mark;
pub mod media_server;
pub mod position;
pub mod refusing;
pub mod streams;
pub mod used;
pub mod uses;
pub mod words;

#[cfg(test)]
mod testing;

pub use by::By;
pub use in_use::InUse;
pub use line::Line;
pub use mark::Mark;
pub use media_server::TheMediaServer;
pub use position::Position;
pub use refusing::NotHeard;
pub use streams::Streams;
pub use used::Used;
pub use uses::{Use, UseId};
pub use words::{
    EVERY_LINE, EVERY_REFUSAL, EVERY_WHO, EVERY_WORD, Word, WordsError, declare_into, in_use_words,
};
