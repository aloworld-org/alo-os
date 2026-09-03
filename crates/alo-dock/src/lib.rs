//! Where the dock sits, which way it runs, and when a name gives way to an icon.
//!
//! `docs/features.md` promises at v0.01: **the dock, and the person decides
//! where it goes** — bottom, left, right or top, chosen in Settings — and that
//! it *works in both orientations rather than being a horizontal bar someone
//! turned sideways: the status area reflows, and labels give way to icons where
//! the short edge demands it*. This crate is the portable half of that: the
//! layout model, with the last clause turned into arithmetic.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`edge`] | Which edge of the screen the dock is on — the whole of what a person chooses |
//! | [`along`] | Which way it runs, and the one thing that never turns with it |
//! | [`measures`] | The numbers this crate is built out of, and what each answers to |
//! | [`room`] | How much room something takes, and the arithmetic that says so |
//! | [`screen`] | The screen it is laid out on, and the side it takes from |
//! | [`labels`] | What became of the names |
//! | [`status`] | The status area: which end, and which way it runs |
//! | [`layout`] | The whole answer, worked out |
//! | [`shipped`] | Where the dock is before anybody moves it |
//! | [`changes`] | What a person changed, which is all that is written down |
//! | [`dock`] | The two resolved, and every question asked of them |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! ```
//! use alo_appearance::TextScale;
//! use alo_dock::{Dock, Edge, Labels, Screen, dock_words};
//! use alo_strings::{Direction, Strings};
//!
//! // What this machine reads. Nothing is translated here, so every answer
//! // below is English and says so.
//! let strings = Strings::of(dock_words()?);
//!
//! let mut dock = Dock::shipped();
//! assert_eq!(dock.edge(), Edge::Bottom);
//!
//! // The smallest screen alo OS lays out for, with the text at the size
//! // EN 301 549 requires a layout to survive.
//! let laptop = Screen::the_smallest();
//! let standard = TextScale::percent(200).expect("200% is the standard's floor");
//!
//! // A dock down the side still has room for its names at that size.
//! dock.set_edge(Edge::Left);
//! let layout = dock.layout_on(laptop, standard, Direction::LeftToRight);
//! assert_eq!(layout.labels(), Labels::Beside);
//!
//! // Above it, on that screen, they give way — and the sentence a person is
//! // shown says where the names went, not only that they are gone.
//! let large = TextScale::percent(300).expect("300% is a size this shell draws");
//! let crowded = dock.layout_on(laptop, large, Direction::LeftToRight);
//! assert!(!crowded.labels().are_shown());
//! assert!(crowded.labels().said(&strings).text().contains("screen reader"));
//!
//! // Only the difference is written down.
//! assert!(!dock.changes().is_untouched());
//! dock.put_everything_back();
//! assert!(dock.changes().is_untouched());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # The threshold is measured, not judged
//!
//! *Where the short edge demands it* is the clause the whole design turns on,
//! and a threshold picked by eye is a threshold nobody can test. So it is
//! arithmetic, and the arithmetic answers to a standard:
//!
//! - a dock may take **one part in six** of the side of the screen it sits on,
//!   because it is on the screen all day and what it takes it takes from the
//!   person's work;
//! - a name under an icon needs a **line of text**, and a name beside one needs
//!   **five ems** of width — an em being the one unit that scales with the text
//!   without a font in the room, since nothing in this crate can measure text;
//! - names are drawn when a dock with them fits under the ceiling, and give way
//!   when it does not.
//!
//! Those two numbers are not taste. They are as generous as EN 301 549's
//! requirement that text reach **200% without loss of content** allows, on the
//! smallest screen alo OS lays out for ([`Screen::the_smallest`]), on all four
//! edges — and [`layout`]'s tests are that requirement, including the one that
//! asserts a tighter share would fail it.
//!
//! **And giving way is not taking away.** A name that is not drawn is still
//! announced by a screen reader and still shown when somebody rests on the icon.
//! The reassurance is inside the string a person is shown rather than beside it,
//! so a translator is handed it and a checked translation cannot lose it
//! quietly.
//!
//! # Two orientations, not one rotated
//!
//! A dock down the side of the screen is not the bottom one turned ninety
//! degrees, and three things in here say so. Its **names sit beside** their
//! icons rather than under them, and still read the ordinary way round, because
//! rotated text is unreadable at a glance and no magnifier or screen reader
//! expects it. Its **thickness comes out of the width** rather than the height,
//! so a wide screen gives a side dock more room than it gives a bottom one. And
//! its **status area is a column**, at the bottom — where the far end of a *row*
//! follows which way the person reads, and the far end of a *column* does not,
//! because every script alo OS ships is read downwards.
//!
//! # Three things this crate is deliberately not
//!
//! **It does not draw anything.** Nothing here opens a window, measures a font,
//! knows what an application is or knows what a pixel is on this particular
//! screen. It answers *which way does the dock run*, *how thick is it*, *are the
//! names drawn and where*, and *where is the status area*; doing any of it is
//! the compositor's, and the compositor does not exist yet.
//!
//! **It does not read anything.** The screen, the text size and which way the
//! person reads are all passed in — the rule `alo-capability` set in item 1 and
//! `alo-appearance` kept — so a settings panel previewing a change asks exactly
//! the question the compositor asks, and neither has to wait for anything to
//! find out.
//!
//! **It is not a capability.** There is no connection between this crate and
//! `alo-capability`, and that is not an omission: a person moving their own dock
//! in Settings is not an agent doing something to their machine, so there is no
//! verb, no grant and no approval.
//!
//! # Nothing here says anything in English by itself
//!
//! Every string a person reads — the four edge names, the three things that can
//! become of the names, and the two refusals — is declared in [`words`] and
//! answered through `alo-strings`. No type in this crate has a `Display` that
//! would put English on a screen: what replaces it is `said`, which answers with
//! an `alo_strings::Said` that says whether anybody translated it. Nothing here
//! deserialises a screen, so there is no key-writing refusal of the kind
//! `alo-appearance` needed for its settings file.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod along;
pub mod changes;
pub mod dock;
pub mod edge;
pub mod labels;
pub mod layout;
pub mod measures;
pub mod room;
pub mod screen;
pub mod shipped;
pub mod status;
pub mod words;

#[cfg(test)]
mod testing;

pub use along::Along;
pub use changes::{Changes, Setting};
pub use dock::Dock;
pub use edge::Edge;
pub use labels::Labels;
pub use layout::Layout;
pub use room::Room;
pub use screen::{Screen, ScreenError};
pub use shipped::Shipped;
pub use status::{End, StatusArea};
pub use words::{Word, WordsError, declare_into, dock_words};
