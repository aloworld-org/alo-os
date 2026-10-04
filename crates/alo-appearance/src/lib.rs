//! What a machine looks like, and which of it a person chose.
//!
//! The first thing anybody does with a new machine is change the picture.
//! `docs/features.md` calls that the moment somebody decides whether the system
//! is theirs or the company's, which is why it is a model with rules in it
//! rather than five values in a configuration file: a background, a lock screen,
//! light and dark, and the size of the text — with what the release ships kept
//! apart from what the person changed, so that neither can quietly overwrite the
//! other.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`colour`] | One colour, and the one way it is written down |
//! | [`contrast`] | How far apart two colours are to look at, to the standard |
//! | [`role`] | What a colour is for — the twelve the design file names |
//! | [`token`] | The six colours a person can pick, by the names they are called |
//! | [`accent`] | The four a person can choose from, and the one they cannot |
//! | [`background`] | The surface a person works on, and what it is made of |
//! | [`display`] | Which screen, when there is more than one |
//! | [`time`] | A time of day, which is all a schedule needs |
//! | [`scheme`] | Light and dark, and the schedule that moves between them |
//! | [`text`] | How big the text is, which is an accessibility setting first |
//! | [`lock`] | What is on the screen when nobody is signed in |
//! | [`shipped`] | What a machine looks like before anybody changes anything |
//! | [`changes`] | What a person changed, which is all that is written down |
//! | [`appearance`] | The two resolved, and every question asked of them |
//! | [`words`] | Every string this crate can say, and the English beside each |
//! | [`unreadable`] | What a settings file that did not read says, where nobody can be asked for words |
//! | [`keeping`] | `appearance.toml` in the person's own folder, read and written here |
//! | [`unkept`] | What a person is told when that file did not read, or was not written |
//!
//! ```
//! use alo_appearance::{
//!     Accent, Appearance, Background, DisplayId, Following, Scheme, Shipped, TextScale,
//!     TimeOfDay, Token, appearance_words,
//! };
//! use alo_strings::Strings;
//!
//! // What this machine reads. Nothing is translated here, so every answer
//! // below is English and says so.
//! let strings = Strings::of(appearance_words()?);
//!
//! let mut appearance = Appearance::shipped();
//! let laptop = DisplayId::named("eDP-1").expect("a screen is named");
//! let projector = DisplayId::named("HDMI-1").expect("a screen is named");
//!
//! // Dark after six, answered at a time that is given rather than read.
//! appearance.follow(Following::from(Shipped::the_evening_schedule()));
//! let evening = TimeOfDay::checked(19, 30).expect("half past seven is a time");
//! let morning = TimeOfDay::checked(9, 0).expect("nine is a time");
//! assert_eq!(appearance.scheme_at(evening), Scheme::Dark);
//! assert_eq!(appearance.scheme_at(morning), Scheme::Light);
//!
//! // One background, and a display can be an exception to it — so a screen
//! // nobody has chosen for shows what the person chose, not what we chose.
//! let navy = Background::from(Token::Navy.colour());
//! appearance.set_background(navy.clone());
//! assert_eq!(appearance.background_on(&projector), navy);
//! assert_eq!(Token::Navy.said(&strings).text(), "Navy");
//!
//! // Text reaches the 200% EN 301 549 requires.
//! appearance.set_text(TextScale::percent(200).expect("200% is the standard's floor"));
//! assert_eq!(appearance.text().to_string(), "200%");
//!
//! // An accent follows light and dark, and deep teal is never one of them —
//! // refused in a sentence the person can read, in the language they read it.
//! appearance.set_accent(Accent::Rose);
//! assert_eq!(appearance.accent_at(evening), Accent::Rose.on(Scheme::Dark));
//! let refused = Accent::of_colour(Token::DeepTeal.colour()).unwrap_err();
//! let sentence = refused.said(&strings).text().to_lowercase();
//! for accent in Accent::ALL {
//!     assert!(sentence.contains(&accent.word().says().to_lowercase()));
//! }
//!
//! // Only the difference is written down.
//! assert!(!appearance.changes().is_untouched());
//! appearance.put_everything_back();
//! assert!(appearance.changes().is_untouched());
//! # let _ = laptop;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Three things this crate is deliberately not
//!
//! **It does not draw anything.** Nothing here opens a picture, reads a folder,
//! measures a screen or knows what a pixel is. It answers *what is behind the
//! windows on this display*, *what does the lock screen show*, *light or dark at
//! this hour* and *how big is the text*; doing any of it is the compositor's,
//! and the compositor does not exist yet.
//!
//! **It does not read the clock, or any disk but its own file.** A schedule is
//! answered at a time of day that is passed in, and a rotating folder is
//! answered by *how many pictures it holds* and *how long it has been running*
//! rather than by going and looking. The one file it reads and writes is
//! `appearance.toml` ([`keeping`]), at a path it is handed and at no other. That is the rule `alo-capability` set in item 1 and it is here
//! for the same reason: the answer is testable without a wait, and the settings
//! panel and the compositor cannot disagree about it.
//!
//! **Setting it is not a capability; asking for it is.** A person setting their
//! own wallpaper in Settings is not an agent doing something to their machine,
//! so that road has no verb, no grant and no approval, and nothing in
//! [`changes`] knows one exists.
//!
//! `docs/features.md` promises at **v0.5** — moved there from v1 by
//! [ADR 0084](../../../docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md)
//! on 2026-10-03 — that an agent can be *asked* for an appearance change: *use
//! dark after six*. That road is [`verbs`], two declarations handed to
//! `alo-capability` through `declare_into`, as thirteen other crates hand over
//! theirs. **The connection is a declaration and nothing more**: both verbs end
//! at [`changes::Changes::follow`], which is the road a settings panel already
//! writes through, so a person's settings and an agent's cannot drift apart into
//! two accounts of one machine.
//!
//! *This paragraph said there was no connection at all, and that it was not an
//! omission. It was true until the verbs landed, and it moved in the same change
//! that made it false rather than after it.*
//!
//! # What this crate does not answer
//!
//! **That the agent is never signalled by colour alone.** ADR 0010 has two
//! halves. The first is here: deep teal is reserved, the five accents are
//! [`accent::Accent`], and every one of them is measured against the grounds it
//! is drawn on. The second — that wherever the agent appears, its colour arrives
//! with a mark and a word — is true of screens rather than of colours, and
//! belongs where the drawing happens. Nothing in this crate can enforce it, and
//! [`contrast`] holds the measurements the rule was argued from.
//!
//! **The reason the mark is not optional has changed, and this paragraph used to
//! state the old one about the wrong colour.** It said *deep teal on the reading
//! ground measures 2.87:1, under what either a word or a shape needs*. 2.87:1 is
//! **terracotta**, which is what ADR 0010 argued from — its colour could not be
//! read at all. ADR 0067 made the agent's colour deep teal, and deep teal
//! **clears both thresholds** at 5.84:1 on the canvas (5.78:1 until ADR 0092
//! moved the canvas's value; terracotta is 2.89:1 against the same new ground
//! and still misses both).
//!
//! So the mark and the word survive their original argument on the durable half:
//! **a hue is never a signal.** Around one man in twelve cannot rely on one, and
//! a colour that passes contrast is still a colour somebody cannot distinguish
//! from `text/primary` — which is why `docs/design/who-is-acting.md` asks whether
//! a person can say who is acting **without naming the colour**.
//!
//! # Nothing here says anything in English by itself
//!
//! Every string a person reads — the eleven colour names, the seventeen
//! refusals and the seven sentences about the person's own file — is declared in [`words`] and answered through `alo-strings`. No
//! type in this crate has a `Display` that would put English on a screen: what
//! replaces it is `said`, which answers with an `alo_strings::Said` that says
//! whether anybody translated it.
//!
//! The two `Display` implementations that remain are deliberate, and neither is
//! a sentence: [`Colour`] writes itself as `#102A43` and [`TextScale`] as
//! `200%`, which is the spelling a settings file holds and a design tool uses.
//! How a *number* or a *time* is written for a person to read belongs to their
//! region rather than to their language — somebody reading Swedish in Finland
//! writes a time the Finnish way — so it is not a string in this list, and
//! [`TimeOfDay`] keeps its `18:00` for the same reason.
//!
//! **A refusal never depends on a string table.** A machine that was never given
//! [`appearance_words`] refuses exactly what it refused before and answers with
//! the key, marked, rather than with a sentence it invented. And a settings file
//! that did not read writes the *key* of the refusal — [`unreadable::NotRead`] —
//! because a deserialiser has no `Strings` to ask and never will.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod accent;
pub mod appearance;
pub mod background;
pub mod changes;
pub mod colour;
pub mod contrast;
pub mod display;
pub mod keeping;
pub mod lock;
pub mod role;
pub mod scheme;
pub mod shipped;
pub mod targets;
pub mod text;
pub mod time;
pub mod token;
pub mod unkept;
pub mod unreadable;
pub mod verbs;
pub mod words;

#[cfg(test)]
mod testing;

pub use accent::{Accent, AccentError};
pub use appearance::Appearance;
pub use background::Background;
pub use changes::{Changes, Setting};
pub use colour::{Colour, ColourError};
pub use contrast::{ENOUGH_FOR_A_SHAPE, ENOUGH_FOR_TEXT};
pub use display::{DisplayError, DisplayId};
pub use lock::Lock;
pub use role::Role;
pub use scheme::{Following, Schedule, ScheduleError, Scheme};
pub use shipped::{Shipped, THE_SURFACE};
pub use text::{TextError, TextScale};
pub use time::{TimeError, TimeOfDay};
pub use token::Token;
pub use unkept::{FileNotRead, FileNotWritten};
pub use unreadable::NotRead;
pub use words::{Word, WordsError, appearance_words, declare_into};
