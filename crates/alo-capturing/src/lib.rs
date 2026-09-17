//! A picture of the screen: the whole of it, one window, or a part somebody
//! selected.
//!
//! `docs/features.md` promises at v0.5 *capture: screenshots, annotation,
//! screen recording with audio, screen sharing*, and
//! `docs/autonomy/v0-5-capture-and-the-room-plan.md` puts the screenshot second
//! — **after** the indicator, deliberately, so that nothing can be built that
//! captures without the indicator already existing to show it. This crate is
//! the screenshot, and it is built on top of `alo-in-use` rather than beside
//! it.
//!
//! - [`What`] — the whole screen, one window, or a selected region;
//! - [`Session`] — whose session the picture is taken in, and whether the lock
//!   screen is up, which is where two of the plan's refusals live;
//! - [`WhereItGoes`] — a file in a folder the person chose, the clipboard, or
//!   both because somebody asked for both;
//! - [`Screenshot`] — a picture that may be taken, and taking it;
//! - [`Asked`] — an application asking through the portal, judged by
//!   `alo-portals` against its grant before anything is taken;
//! - [`Grabs`] and [`TheScreenCast`] — the rented screen-capture mechanism, as
//!   one question and as the one thing on a machine that answers it;
//! - [`NotTaken`] — every way a picture does not happen, each with a sentence.
//!
//! ```
//! use alo_capturing::{
//!     Folder, NotTaken, Screen, Screenshot, Session, What, WhereItGoes, WhoseSession,
//!     capturing_words,
//! };
//! use alo_strings::Strings;
//! use std::path::Path;
//!
//! let anna = WhoseSession::of("anna");
//! let screen = Screen::measuring(1920, 1080)?;
//! let pictures = Folder::chosen(Path::new("/home/anna/Pictures"))?;
//!
//! // A picture of the whole screen, into the folder she chose and nowhere
//! // else: asking for a file does not put one on the clipboard as well.
//! let taking = Screenshot::of(
//!     What::TheWholeScreen,
//!     &Session::of(&anna),
//!     screen,
//!     WhereItGoes::a_file(&pictures),
//! )?;
//! assert!(taking.where_it_goes().writes_a_file());
//! assert!(!taking.where_it_goes().reaches_the_clipboard());
//!
//! // While the lock screen is up, nothing is captured at all — and a refusal
//! // is not a value that can reach the rented mechanism.
//! assert_eq!(
//!     Screenshot::of(
//!         What::TheWholeScreen,
//!         &Session::behind_the_lock_screen(&anna),
//!         screen,
//!         WhereItGoes::a_file(&pictures),
//!     ),
//!     Err(NotTaken::TheLockScreen),
//! );
//!
//! // And the person is told why, in the language they read.
//! let strings = Strings::of(capturing_words()?);
//! assert!(
//!     NotTaken::TheLockScreen
//!         .said(&strings)
//!         .text()
//!         .starts_with("A picture of the lock screen cannot be taken")
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`screen`] | The screen a picture is taken from, as a size |
//! | [`region`] | A rectangle of it |
//! | [`window`] | One window, and the two things about it that refuse a picture |
//! | [`what`] | What is being captured, and the rectangle it is |
//! | [`session`] | Whose session, and whether the lock screen is up |
//! | [`folder`] | The folder a person chose to keep pictures in |
//! | [`where_it_goes`] | A file, the clipboard, or both because somebody asked |
//! | [`on_this_day`] | The moment, as a calendar date |
//! | [`naming`] | What a saved picture is called: the date, and nothing else |
//! | [`picture`] | The bytes, and what form they are in |
//! | [`grabs`] | The rented mechanism, as one question |
//! | [`the_screen_cast`] | The one thing on a machine that answers it |
//! | [`announcing`] | How a capture of ours announces itself, so it is on the indicator |
//! | [`taking`] | The act, and everything refused before it |
//! | `writing` | Writing the file, without touching anything already there |
//! | [`taken`] | What happened to it |
//! | [`asked`] | An application asking through the portal |
//! | [`refusing`] | Every way it does not happen |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! # A screenshot is a use of the screen, and it is on the indicator
//!
//! Not because this crate tells the indicator so. `alo_in_use::InUse::read_from`
//! has one door and it is the machine's own media server, and a capture of alo
//! OS's own that announced itself to the indicator directly would be the first
//! exception to that — from inside the operating system, which is exactly where
//! `docs/features.md`'s ★ *by any application, including ours* is pointed.
//!
//! So [`TheScreenCast`] reads its frame through the media server, announcing
//! itself under the names `alo_in_use::heard` reads ([`announcing`]), and the
//! indicator lists *the screen, in use by alo OS itself* for the moment the
//! picture is being taken — from the server's record, exactly as it lists
//! anybody else.
//!
//! # Nothing is written that was not asked for, and nothing goes anywhere else
//!
//! **Never both unasked.** [`WhereItGoes`] has a *both*, and the only way to
//! obtain it is to ask for it. A picture bound for the clipboard writes no
//! file; a picture bound for a file leaves the clipboard exactly as it found
//! it, and [`Screenshot::take`] is handed the machine's one clipboard whatever
//! the destination so that a test can watch it come back untouched.
//!
//! **Nothing already in the folder is touched.** The file is created rather
//! than opened, so a name that is already anything — including a symbolic link
//! pointing at somebody else's file — fails and the next name is tried.
//!
//! **Nothing leaves.** There are two destinations and both are on this machine.
//! Nothing here opens a connection and nothing this crate depends on can;
//! `tests/nothing_leaves_when_a_picture_is_taken.rs` reads the manifests and
//! says so. The plan's constraint — *nothing is uploaded, shared or sent
//! anywhere by taking a screenshot* — needs no code to enforce it, because
//! there is no code that could break it.
//!
//! # The file's name says nothing about what was on the screen
//!
//! It is the date and the time, and it has no word in it at all — not even in
//! English. [`naming`] says why that is load-bearing rather than tidy: a folder
//! of pictures is something a person scrolls past, shows over their shoulder,
//! backs up and hands to a repair shop, and a window's title in a file name is
//! a sentence about somebody's appointment readable long after the picture
//! itself was deleted.
//!
//! # Two things cannot be captured, and they are refusals rather than checks
//!
//! **The lock screen**, whatever was asked for, and **a window in another
//! person's session**. [`Screenshot::of`] is the constructor and both refusals
//! are in it, so a picture that may not be taken is not a value that exists and
//! then declines to be taken: it is a value that cannot be built, and
//! [`Screenshot::take`] is the only thing that can reach [`Grabs`].
//!
//! # It decides nothing about grants
//!
//! An application asking for a picture through the portal is judged by
//! `alo_portals::Request::judged` against `alo_capability::Grants`, and
//! [`asked::ForAnApplication`] — the only value with a `take` on it for an
//! application — can only be built from an allowed judgement. There is no verb
//! in this crate and nothing an agent can call: the agent's picture of the
//! screen is the plan's task 6, through a verb whose proposal says *a picture
//! of your screen* and which is approved like any other change.
//!
//! # And nothing here draws
//!
//! No selection rectangle, no shutter, no notification. Choosing a region is a
//! person's act with a pointer and belongs to the shell plan's later tasks;
//! what is settled here is every value and every refusal that lane would
//! otherwise decide while wiring it, which is how a refusal gets decided by
//! accident.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod announcing;
pub mod asked;
pub mod folder;
pub mod grabs;
pub mod marking;
pub mod naming;
pub mod on_this_day;
pub mod picture;
pub mod refusing;
pub mod region;
pub mod screen;
pub mod session;
pub mod taken;
pub mod taking;
pub mod the_screen_cast;
pub mod what;
pub mod where_it_goes;
pub mod window;
pub mod words;

mod to_the_clipboard;
mod writing;

#[cfg(test)]
mod testing;

pub use asked::{Asked, ForAnApplication};
pub use folder::Folder;
pub use grabs::{Grabs, NotGrabbed};
pub use marking::{Mark, Marks};
pub use on_this_day::OnThisDay;
pub use picture::Picture;
pub use refusing::NotTaken;
pub use region::Region;
pub use screen::Screen;
pub use session::{Session, WhoseSession};
pub use taken::Taken;
pub use taking::Screenshot;
pub use the_screen_cast::TheScreenCast;
pub use what::What;
pub use where_it_goes::WhereItGoes;
pub use window::{Window, WindowId};
pub use words::{
    EVERY_REFUSAL, EVERY_TOLD, EVERY_WORD, Word, WordsError, capturing_words, declare_into,
};
