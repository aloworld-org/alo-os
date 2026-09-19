//! A person's screens: which they are, where each sits, how large, and the
//! arrangement remembered for every set of them.
//!
//! `ROADMAP.md` v0.5: *multi-monitor, scaling, hotplug*. The failure everybody
//! knows is the laptop that forgets, every morning, that the external screen is
//! on the left — so this crate is organised around remembering the right thing,
//! and around the two hours of a working day when a cable goes in or comes out.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`identity`] | Which screen this is tomorrow: what it says about itself, or where it is plugged in |
//! | [`reported`] | What the machine says about a screen that is plugged in now |
//! | [`scale`] | How large everything is drawn, and what a screen nobody has seen is set to |
//! | [`placed`] | Where one screen sits, how large it draws, and whether it is the main one |
//! | [`arrangement`] | One set of screens laid out, and the set itself |
//! | [`changes`] | Everything a person has changed about their screens, which is all that is written down |
//! | [`attached`] | The screens plugged in at this moment, laid out |
//! | [`coming_and_going`] | A screen unplugged and plugged back in: where the windows belong |
//! | [`time_of_day`] | A time on the clock the person is looking at |
//! | [`between`] | A stretch of that clock, which nearly always runs through midnight |
//! | [`moment`] | What the machine's clock says, handed in rather than read |
//! | [`sun`] | When the sun sets and rises where the person says they are, worked out here |
//! | [`warmth`] | How warm a screen is drawn, and what that does to every colour on it |
//! | [`nightly`] | When night light is on: never, a schedule, or the sun |
//! | [`night_light`] | Night light as a person has it set, and what it is doing now |
//! | [`wearing`] | The background, the dock edge and the warmth each screen wears |
//! | [`notes`] | What a person is told about their screens |
//! | [`words`] | Every string this crate can say, and the English beside each |
//! | [`keeping`] | `displays.toml` in the person's own folder, read and written here |
//! | [`unkept`] | What a person is told when that file did not read, or was not written |
//! | [`unreadable`] | What a settings file that did not read writes, where nobody can be asked for words |
//!
//! ```
//! use alo_displays::{Attached, Changes, Reported, Scale, Socket, Support};
//!
//! // Two screens, as the machine reports them. The laptop's own panel says
//! // nothing about itself; the monitor says what it is and gives a serial.
//! let laptop = Reported::of(
//!     Socket::named("eDP-1").expect("a socket is named"),
//!     None,
//!     (1920, 1080),
//!     Some((294, 165)),
//! )
//! .expect("a screen with pixels is a screen");
//! let monitor = Reported::of(
//!     Socket::named("DP-1").expect("a socket is named"),
//!     Some(alo_displays::Panel::of("Dell", "U2720Q", Some("CN-0ABC")).expect("a screen says so")),
//!     (3840, 2160),
//!     Some((596, 336)),
//! )
//! .expect("a screen with pixels is a screen");
//!
//! // Nobody has arranged these two before, so they go side by side — and
//! // neither of them at 100%: each is sized from its own glass.
//! let mut remembered = Changes::untouched();
//! let mut attached = Attached::now(
//!     vec![laptop.clone(), monitor.clone()],
//!     &remembered,
//!     Support::Fractional,
//! )
//! .expect("two screens are an arrangement");
//! let office = attached.on(&attached.main_screen().clone()).expect("the main screen");
//! assert_eq!(office.placed().scale(), Scale::per_cent(175).expect("175% is a size"));
//!
//! // The person moves the monitor to the left and it is remembered, under
//! // this set of screens and no other.
//! remembered.remember(attached.arrangement().clone());
//! assert!(remembered.for_screens(&attached.screens()).is_some());
//!
//! // The monitor is unplugged. What was open on it belongs on the laptop.
//! let moved = attached
//!     .unplugged(monitor.socket(), &remembered)
//!     .expect("one screen remains");
//! assert_eq!(moved.onto(), attached.main_screen());
//!
//! // And it comes back, so what was open on it goes back to it.
//! let back = attached.plugged_in(monitor, &remembered).expect("it fits");
//! assert!(back.anything_goes_back());
//! ```
//!
//! And night light, which is the one thing here that depends on when it is:
//!
//! ```
//! use std::time::{Duration, UNIX_EPOCH};
//! use alo_displays::{Moment, NightLight, Nightly, Warmth, Whereabouts};
//!
//! // From sunset to sunrise in London, at 2700 K — worked out on this machine
//! // from two numbers somebody typed, and from nothing else.
//! let london = Whereabouts::typed(51.5074, -0.1278).expect("that is on the earth");
//! let night = NightLight::of(
//!     Nightly::FromSunsetAt(london),
//!     Warmth::kelvin(2700).expect("2700 K is a warmth"),
//! );
//!
//! // The longest evening of 2026, at nine o'clock: the sun is still up.
//! let midsummer = 20_625 * 24 * 60 * 60;
//! let nine = Moment::at(UNIX_EPOCH + Duration::from_secs(midsummer + 20 * 60 * 60), 60);
//! assert!(!night.at(nine).is_on());
//!
//! // An hour later it is not, and every screen is drawn warmer.
//! let ten = Moment::at(UNIX_EPOCH + Duration::from_secs(midsummer + 21 * 60 * 60), 60);
//! assert_eq!(night.at(ten).warmth(), Warmth::kelvin(2700).expect("2700 K"));
//! ```
//!
//! # The five decisions this crate makes, and nothing else
//!
//! 1. **Which screen this is.** What it says about itself, or — for a screen
//!    that says nothing, and for two screens that say the same thing — where it
//!    is plugged in, which is weaker and said to be ([`identity`]).
//! 2. **Where each one goes.** The arrangement remembered for exactly this set
//!    of screens, or side by side left to right when there is none
//!    ([`attached`]).
//! 3. **How large each draws.** The size last chosen for that screen anywhere,
//!    or one worked out from its own glass — never 100% by default — and then
//!    the nearest size the machine can actually draw ([`scale`]).
//! 4. **Where the windows of a screen that went belong**, and that they come
//!    back ([`coming_and_going`]).
//! 5. **How warm each one is drawn**, on a schedule a person set or from
//!    sunset to sunrise worked out on this machine from two numbers they typed
//!    ([`night_light`], [`sun`]).
//!
//! # What is not here
//!
//! **Modesetting.** The kernel's side of a graphics card and the compositor
//! library are rented and configured (ADR 0011). Nothing in this crate opens a
//! device, sets a mode or speaks a protocol; it is handed what the machine
//! reports and answers with what should happen.
//!
//! **Drawing, and moving a window.** Both are the shell's. This crate says
//! *onto which screen*; it holds no window identifier and will not.
//!
//! **A background or a dock.** [`wearing`] reads both from the crates that own
//! them and says which screen each belongs to. It writes to neither.
//!
//! **Anywhere a person is.** [`sun`] works a sunset out from two numbers
//! somebody typed into Settings, on this machine, with arithmetic. There is no
//! location service here, no address sent anywhere, and no network of any kind
//! — `tests/the_sun_is_worked_out_on_this_machine.rs` reads this crate's own
//! source and manifest and refuses every road to one. Absent a place the person
//! typed, there is no guess: the setting offers a schedule instead.
//!
//! **A clock.** [`moment`] is handed one. Nothing here reads the time, an
//! environment variable or a timezone database.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod arrangement;
pub mod attached;
pub mod between;
pub mod changes;
pub mod coming_and_going;
pub mod identity;
pub mod keeping;
pub mod moment;
pub mod night_light;
pub mod nightly;
pub mod notes;
pub mod placed;
pub mod reported;
pub mod scale;
pub mod sun;
pub mod time_of_day;
pub mod unkept;
pub mod unreadable;
pub mod warmth;
pub mod wearing;
pub mod words;

#[cfg(test)]
mod testing;

pub use arrangement::{Arrangement, NotArranged, Screens};
pub use attached::{Attached, OnScreen};
pub use between::{Between, NotAStretch};
pub use changes::Changes;
pub use coming_and_going::{CameBack, Moved, NotAttached};
pub use identity::{Identity, IdentityError, Panel, Socket, Stability, whoever_is_attached};
pub use moment::Moment;
pub use night_light::{NightLight, Tonight};
pub use nightly::{Nightly, NotNightly, Now};
pub use notes::Note;
pub use placed::{Placed, Position};
pub use reported::{Millimetres, NotAScreen, Reported, Resolution, which_screens_these_are};
pub use scale::{Rounded, Scale, ScaleError, Support};
pub use sun::{NowhereOnEarth, Sun, Whereabouts};
pub use time_of_day::{NotATime, TimeOfDay};
pub use unkept::{FileNotRead, FileNotWritten};
pub use unreadable::NotRead;
pub use warmth::{Warming, Warmth, WarmthError};
pub use wearing::Wearing;
pub use words::{EVERY_WORD, Word, WordsError, declare_into, display_words};
