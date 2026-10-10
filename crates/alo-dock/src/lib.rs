//! How thick the dock is, how far it runs, and when a name gives way to an icon.
//!
//! **The dock is along the bottom edge of the screen.** [ADR
//! 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
//! fixed it there and withdrew the v0.01 promise that a person chooses which edge
//! — along with the per-display exception, both orientations, and the status area,
//! which is not the Dock's job. What is left of that promise is the clause worth
//! keeping: *labels give way to icons where the short edge demands it*, which this
//! crate turns into arithmetic.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`measures`] | The numbers this crate is built out of, and what each answers to |
//! | [`room`] | How much room something takes, and the arithmetic that says so |
//! | [`screen`] | The screen it is laid out on |
//! | [`labels`] | What became of the names |
//! | [`hiding`] | Whether it gives way when a window needs the room |
//! | [`layout`] | The whole answer, worked out |
//! | [`on_the_canvas`] | Where a window is on the plane, and what is being looked at |
//! | [`window`] | One window: its application, its place, and how it sits |
//! | [`windows`] | Every window open, in the order they were last used |
//! | [`holding`] | Which applications are on the Dock, and which of them fit |
//! | [`places`] | Where each application's icon sits on the bar |
//! | [`clicking`] | What one click on an application's icon does |
//! | [`previews`] | An application's windows, for choosing one that is not the last |
//! | [`travelling`] | Going somewhere, looking without going, and coming back |
//! | [`offering`] | What a drop would do, said before it is done |
//! | [`menu`] | What an icon offers when asked, and what it never offers unasked |
//! | [`announcing`] | What a screen reader reads out for an icon |
//! | [`revealing`] | Reaching a surface at a screen edge when a window fills the screen |
//! | [`shipped`] | What the dock is before anybody changes anything |
//! | [`changes`] | What a person changed, which is all that is written down |
//! | [`dock`] | The two resolved, and every question asked of them |
//! | [`words`] | Every string this crate can say, and the English beside each |
//! | [`keeping`] | `dock.toml` in the person's own folder, read and written here |
//! | [`unkept`] | What a person is told when that file did not read, or was not written |
//!
//! ```
//! use alo_appearance::TextScale;
//! use alo_dock::{Dock, Hiding, Labels, Screen, Showing, TheRoom, dock_words};
//! use alo_strings::Strings;
//!
//! // What this machine reads. Nothing is translated here, so every answer
//! // below is English and says so.
//! let strings = Strings::of(dock_words()?);
//!
//! let mut dock = Dock::shipped();
//!
//! // The smallest screen alo OS lays out for, with the text at the size
//! // EN 301 549 requires a layout to survive.
//! let laptop = Screen::the_smallest();
//! let standard = TextScale::percent(200).expect("200% is the standard's floor");
//!
//! // It still has room for its names at that size.
//! assert_eq!(dock.layout_on(laptop, standard).labels(), Labels::Under);
//!
//! // Above it, on that screen, they give way — and the sentence a person is
//! // shown says where the names went, not only that they are gone.
//! let large = TextScale::percent(300).expect("300% is a size this shell draws");
//! let crowded = dock.layout_on(laptop, large);
//! assert!(!crowded.labels().are_shown());
//! assert!(crowded.labels().said(&strings).text().contains("screen reader"));
//!
//! // A fresh dock stays on the screen whatever the windows want, and the one
//! // thing a person can change about it is that.
//! assert_eq!(dock.showing(TheRoom::AWindowNeedsIt), Showing::Shown);
//! dock.set_hiding(Hiding::WhenAWindowNeedsTheRoom);
//! assert_eq!(dock.showing(TheRoom::AWindowNeedsIt), Showing::Hidden);
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
//! - a dock may take **one part in six** of the height of the screen it sits on,
//!   because it is on the screen all day and what it takes it takes from the
//!   person's work;
//! - a name under an icon needs a **line of text**;
//! - names are drawn when a dock with them fits under the ceiling, and give way
//!   when it does not.
//!
//! That share is not taste. It is as generous as EN 301 549's requirement that
//! text reach **200% without loss of content** allows, on the smallest screen alo
//! OS lays out for ([`Screen::the_smallest`]) — and [`layout`]'s tests are that
//! requirement, including the one that asserts a tighter share would fail it.
//!
//! **It was two numbers until ADR 0076.** The second was how much width a name
//! needed *beside* an icon, on a dock down the side of the screen. Removing an
//! orientation could have left the share loose, since it was justified against
//! both; it did not, and the test says which way it is fixed.
//!
//! **And giving way is not taking away.** A name that is not drawn is still
//! announced by a screen reader and still shown when somebody rests on the icon.
//! The reassurance is inside the string a person is shown rather than beside it,
//! so a translator is handed it and a checked translation cannot lose it
//! quietly.
//!
//! # Four edges, with bottom the default — and this section argued the opposite
//!
//! **It said a second edge was a decision to undo rather than a gap to fill, and
//! it outlived the decision it was defending by four days.** Kept rather than
//! deleted, because a reader who finds the old argument elsewhere should find its
//! correction here:
//!
//! > This crate held four edges, two orientations and a per-display exception to
//! > the edge. A **default** would have kept every one of them in the code … for a
//! > choice ADR 0076 says nobody should make. The cost of *offered but
//! > discouraged* is paid on every screen, at every scale, forever …
//! >
//! > Anybody restoring a second edge is undoing a decision rather than filling a
//! > gap, and that record is the reason.
//!
//! The owner reversed the bottom-only ruling on **2026-09-30**, within the hour of
//! the record that took it: *bottom should be the default; the person can choose
//! bottom, left, right, or top*, and *restore the Dock's edge choice in v0.01*.
//! So the paragraph above was arguing against a decision that had already been
//! withdrawn, and *anybody restoring a second edge* describes the owner.
//!
//! **Its reasoning was not wrong, only its subject.** The cost of *offered but
//! discouraged* is real, and it is why the restoration is being done in the order
//! the owner set on 2026-10-04 — structure first, and **nonfunctional edge
//! choices are not exposed as finished settings**. So [`Edge`] exists with all
//! four, and [`Layout::along`] lays out every one of them — it refused two by
//! name until 2026-10-10, when the owner supplied the measurement that was the
//! whole obstacle ([`measures::A_NAME_BESIDE_AN_ICON`], a name beside a side
//! dock's icon rather than under it). **The structure is finished and the
//! setting is still withheld**, which is the order the owner set: [`Edge`] sits
//! on [`Shipped`] and deliberately not on [`changes::Changes`], and
//! [`words`] declares no name for an edge, so nothing here offers a person a
//! choice at all — working or otherwise. What
//! the old paragraph got right is that a choice in the code and not on the screen
//! is a cost; what it got wrong is that the answer to that is to delete the
//! choice rather than to finish it.
//!
//! # Three things this crate is deliberately not
//!
//! **It does not draw anything.** Nothing here opens a window, measures a font,
//! knows what an application is or knows what a pixel is on this particular
//! screen. It answers *how thick is the dock*, *how far does it run*, *are the
//! names drawn* and *is it on the screen at all*; doing any of it is the
//! compositor's.
//!
//! **It does not read anything but its own file.** The screen and the text size
//! are passed in — the rule `alo-capability` set in item 1 and `alo-appearance`
//! kept — so a settings panel previewing a change asks exactly the question the
//! compositor asks, and neither has to wait for anything to find out. The one
//! file it reads and writes is `dock.toml` ([`keeping`]), at a path it is handed
//! and at no other.
//!
//! **It is not a capability.** There is no connection between this crate and
//! `alo-capability`, and that is not an omission: a person changing their own
//! dock in Settings is not an agent doing something to their machine, so there is
//! no verb, no grant and no approval.
//!
//! # Nothing here says anything in English by itself
//!
//! Every string a person reads — the two answers about whether it gives way, what
//! became of the names, the two refusals and the eight sentences about the
//! person's own file — is declared in [`words`] and answered through
//! `alo-strings`. No type in this crate has a `Display` that would put English on
//! a screen: what replaces it is `said`, which answers with an
//! `alo_strings::Said` that says whether anybody translated it.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod announcing;
pub mod changes;
pub mod clicking;
pub mod dock;
pub mod edge;
pub mod hiding;
pub mod holding;
pub mod keeping;
pub mod labels;
pub mod layout;
pub mod measures;
pub mod menu;
pub mod offering;
pub mod on_the_canvas;
pub mod places;
pub mod previews;
pub mod revealing;
pub mod room;
pub mod screen;
pub mod shipped;
pub mod travelling;
pub mod unkept;
pub mod window;
pub mod windows;
pub mod words;

#[cfg(test)]
mod testing;

pub use announcing::{Announced, Focused, where_it_sits};
pub use changes::{Changes, Setting};
pub use clicking::{WhatAClickDoes, what_a_click_does};
pub use dock::Dock;
pub use edge::Edge;
pub use hiding::{Hiding, Showing, TheRoom};
pub use holding::{Fitted, Holding, OnTheDock, Pinned, fit};
pub use labels::Labels;
pub use layout::Layout;
pub use menu::{AWindowsState, What};
pub use offering::{Offer, WhatWouldHappen, dropped_at, near_the_view};
pub use on_the_canvas::{NotAPatch, Patch, Spot, TheView};
pub use places::{APlace, Places};
pub use previews::{Opened, Preview, Previews};
pub use revealing::{ADrag, AMenu, Dismissed, FocusGoes, Revealing, TheKeyboard, ThePointer};
pub use room::Room;
pub use screen::{Screen, ScreenError};
pub use shipped::Shipped;
pub use travelling::{GoingBack, PeekEnded, Peeking, WhatHappenedNext};
pub use unkept::{FileNotRead, FileNotWritten};
pub use window::{AppId, HowItSits, NotAnApp, Window, WindowId};
pub use windows::Windows;
pub use words::{Word, WordsError, declare_into, dock_words};
