//! **What sticky, slow and bounce keys do to a key press**, as a machine
//! anybody can test.
//!
//! [`crate::key_filter`] holds what a person set. This holds what the keyboard
//! then does with it, which is the half EN 301 549 clause 5 is about: a key held
//! down that must not repeat, a key struck twice that must count once, and a
//! machine that must never need two keys at one time.
//!
//! # Time arrives, it is never read
//!
//! Every press carries how long since the keyboard started, so a test is a list
//! of presses rather than a person with a stopwatch. Nothing here reads a clock,
//! which is the same rule `alo-choosing` states about the environment and for
//! the same reason: a machine that reads the world cannot be described.
//!
//! # What each one is, in one line
//!
//! - **Sticky**: a modifier stays held after it is let go, so `Ctrl` then `C` is
//!   the chord `Ctrl+C` — the clause is *no action requires two keys at once*.
//! - **Slow**: a key counts only once it has been held for as long as a person
//!   asked, so a hand resting on a keyboard types nothing.
//! - **Bounce**: the same key struck again too soon does not count twice, which
//!   is a tremor rather than a person.
//!
//! **One filter, not three.** A person who needs two of these has one keyboard,
//! and the order they are applied in decides what they get: slow keys decide
//! whether a press happened at all, bounce keys decide whether a press that
//! happened counts, and sticky keys decide what it counts *as*. Applying them in
//! any other order gives a keyboard that swallows the second half of everything.

use std::time::Duration;

use crate::key_filter::KeyFilter;

/// **A key on the keyboard**, as this file needs to tell them apart.
///
/// A modifier or not, and which — nothing here cares which letter it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pressed {
    /// A key that means something on its own.
    AKey(char),
    /// A key that changes what another key means.
    AModifier(Modifier),
}

/// The four keys that change what another key means.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Modifier {
    /// The one alo OS's own chords use.
    Super,
    /// Control.
    Ctrl,
    /// Alt.
    Alt,
    /// Shift.
    Shift,
}

/// **One thing the keyboard reported**, with when it happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reported {
    /// Which key.
    pub key: Pressed,
    /// Whether it went down or came up.
    pub down: bool,
    /// How long since the keyboard started.
    pub at: Duration,
}

impl Reported {
    /// A key going down.
    #[must_use]
    pub const fn down(key: Pressed, at: Duration) -> Self {
        Self {
            key,
            down: true,
            at,
        }
    }

    /// A key coming up.
    #[must_use]
    pub const fn up(key: Pressed, at: Duration) -> Self {
        Self {
            key,
            down: false,
            at,
        }
    }
}

/// **What the machine is told a person typed**, once the filter has had it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Typed {
    /// The key.
    pub key: char,
    /// What was held with it — by a hand, or by sticky keys.
    pub with: Vec<Modifier>,
}

/// **A keyboard reading keys under one person's settings.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filtering {
    /// What the person asked for.
    filter: KeyFilter,
    /// Modifiers a hand is holding now.
    held: Vec<Modifier>,
    /// Modifiers sticky keys is holding for the next key.
    stuck: Vec<Modifier>,
    /// A key that is down and has not counted yet, and when it went down.
    waiting: Option<(char, Duration)>,
    /// The last key that counted, and when — for bounce keys.
    last: Option<(char, Duration)>,
}

impl Filtering {
    /// A keyboard under these settings.
    #[must_use]
    pub const fn under(filter: KeyFilter) -> Self {
        Self {
            filter,
            held: Vec::new(),
            stuck: Vec::new(),
            waiting: None,
            last: None,
        }
    }

    /// **What the machine is told about this report**, or [`None`] where the
    /// filter swallowed it.
    pub fn reads(&mut self, reported: Reported) -> Option<Typed> {
        match (reported.key, reported.down) {
            (Pressed::AModifier(modifier), true) => {
                if !self.held.contains(&modifier) {
                    self.held.push(modifier);
                }
                None
            }
            (Pressed::AModifier(modifier), false) => {
                self.held.retain(|held| *held != modifier);
                // Sticky keys: a modifier let go of is held for the next key
                // instead, so nothing needs two keys at one time.
                if self.filter.sticky && !self.stuck.contains(&modifier) {
                    self.stuck.push(modifier);
                }
                None
            }
            (Pressed::AKey(key), true) => {
                self.waiting = Some((key, reported.at));
                // Without slow keys a press counts when it goes down.
                match self.filter.held_before_it_counts {
                    Some(_) => None,
                    None => self.counted(key, reported.at),
                }
            }
            (Pressed::AKey(key), false) => {
                let waiting = self.waiting.take();
                // Slow keys: it counts on the way up, and only if it was held
                // for as long as the person asked. Without them it counted on
                // the way down and there is nothing to do here.
                let how_long = self.filter.held_before_it_counts?;
                let (waited, since) = waiting.filter(|(waited, _)| *waited == key)?;
                if reported.at.saturating_sub(since) < how_long {
                    return None;
                }
                self.counted(waited, reported.at)
            }
        }
    }

    /// A press that got past the filters, with whatever is held on it.
    fn counted(&mut self, key: char, at: Duration) -> Option<Typed> {
        // Bounce keys: the same key again, too soon, is one hand shaking.
        if let Some(ignored_for) = self.filter.ignored_after_the_same_key
            && let Some((last, when)) = self.last
            && last == key
            && at.saturating_sub(when) < ignored_for
        {
            self.waiting = None;
            return None;
        }
        self.last = Some((key, at));
        self.waiting = None;
        let mut with = self.held.clone();
        with.append(&mut self.stuck);
        with.sort_unstable();
        with.dedup();
        Some(Typed { key, with })
    }

    /// What a hand is holding now, for whoever draws it.
    #[must_use]
    pub fn held(&self) -> &[Modifier] {
        &self.held
    }

    /// What sticky keys is holding for the next key, which a person is shown so
    /// that a latched modifier is never a surprise.
    #[must_use]
    pub fn stuck(&self) -> &[Modifier] {
        &self.stuck
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn at(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    /// **Sticky keys: a chord is typed one key at a time.**
    ///
    /// EN 301 549 clause 5.9: nothing must need two keys at once. `Ctrl` is
    /// pressed and let go, then `c` — and the machine is told `Ctrl+c`.
    #[test]
    fn sticky_keys_make_a_chord_out_of_two_presses_one_after_the_other() {
        let mut keyboard = Filtering::under(KeyFilter {
            sticky: true,
            ..KeyFilter::none()
        });
        assert_eq!(
            keyboard.reads(Reported::down(Pressed::AModifier(Modifier::Ctrl), at(0))),
            None
        );
        assert_eq!(
            keyboard.reads(Reported::up(Pressed::AModifier(Modifier::Ctrl), at(50))),
            None
        );
        assert_eq!(keyboard.stuck(), [Modifier::Ctrl]);

        let typed = keyboard
            .reads(Reported::down(Pressed::AKey('c'), at(900)))
            .expect("the key counts");
        assert_eq!(typed.key, 'c');
        assert_eq!(typed.with, [Modifier::Ctrl]);

        // And it is let go of afterwards: the next key is a plain key.
        assert!(keyboard.stuck().is_empty());
        let after = keyboard
            .reads(Reported::down(Pressed::AKey('d'), at(1_000)))
            .expect("the key counts");
        assert!(
            after.with.is_empty(),
            "the modifier stayed held after the key it belonged to"
        );
    }

    /// **Slow keys: a key counts once it has been held long enough**, and a
    /// hand resting on the keyboard types nothing.
    #[test]
    fn slow_keys_ignore_a_key_that_was_not_held_for_as_long_as_the_person_asked() {
        let mut keyboard = Filtering::under(KeyFilter {
            held_before_it_counts: Some(Duration::from_millis(300)),
            ..KeyFilter::none()
        });

        assert_eq!(
            keyboard.reads(Reported::down(Pressed::AKey('a'), at(0))),
            None,
            "a key counted the moment it went down, which is what slow keys are for"
        );
        assert_eq!(
            keyboard.reads(Reported::up(Pressed::AKey('a'), at(100))),
            None,
            "a key held for a tenth of a second counted"
        );

        keyboard.reads(Reported::down(Pressed::AKey('b'), at(1_000)));
        let typed = keyboard
            .reads(Reported::up(Pressed::AKey('b'), at(1_400)))
            .expect("held for four tenths of a second");
        assert_eq!(typed.key, 'b');
    }

    /// **Bounce keys: the same key again, too soon, counts once.**
    #[test]
    fn bounce_keys_count_one_press_where_a_hand_shook_twice() {
        let mut keyboard = Filtering::under(KeyFilter {
            ignored_after_the_same_key: Some(Duration::from_millis(200)),
            ..KeyFilter::none()
        });

        assert!(
            keyboard
                .reads(Reported::down(Pressed::AKey('e'), at(0)))
                .is_some()
        );
        assert_eq!(
            keyboard.reads(Reported::down(Pressed::AKey('e'), at(120))),
            None,
            "the same key a tenth of a second later counted twice"
        );
        // A different key is not a bounce, however fast.
        assert!(
            keyboard
                .reads(Reported::down(Pressed::AKey('f'), at(130)))
                .is_some()
        );
        // And the same key once the moment has passed is a person typing.
        assert!(
            keyboard
                .reads(Reported::down(Pressed::AKey('e'), at(400)))
                .is_some()
        );
    }

    /// **All three at once**, which is the argument `crate::key_filter` makes
    /// for one filter rather than three settings: slow keys decide whether a
    /// press happened, bounce keys whether it counts, sticky keys what it
    /// counts as.
    #[test]
    fn the_three_together_read_as_one_keyboard() {
        let mut keyboard = Filtering::under(KeyFilter {
            sticky: true,
            held_before_it_counts: Some(Duration::from_millis(200)),
            ignored_after_the_same_key: Some(Duration::from_millis(500)),
        });

        keyboard.reads(Reported::down(Pressed::AModifier(Modifier::Super), at(0)));
        keyboard.reads(Reported::up(Pressed::AModifier(Modifier::Super), at(10)));
        keyboard.reads(Reported::down(Pressed::AKey('a'), at(100)));
        let typed = keyboard
            .reads(Reported::up(Pressed::AKey('a'), at(400)))
            .expect("held long enough");
        assert_eq!(
            typed.with,
            [Modifier::Super],
            "the latched modifier was lost"
        );

        // The same key again, inside the bounce, is swallowed even though it
        // was held long enough.
        keyboard.reads(Reported::down(Pressed::AKey('a'), at(500)));
        assert_eq!(
            keyboard.reads(Reported::up(Pressed::AKey('a'), at(800))),
            None,
            "a bounce got through because it was held long enough"
        );
    }

    /// **A keyboard nobody has changed is a keyboard**, and every key counts
    /// the moment it goes down.
    #[test]
    fn with_nothing_turned_on_a_key_counts_when_it_goes_down() {
        let mut keyboard = Filtering::under(KeyFilter::none());
        let typed = keyboard
            .reads(Reported::down(Pressed::AKey('x'), at(0)))
            .expect("a plain keyboard");
        assert_eq!(
            typed,
            Typed {
                key: 'x',
                with: Vec::new()
            }
        );
        assert_eq!(
            keyboard.reads(Reported::up(Pressed::AKey('x'), at(5))),
            None
        );

        // A modifier held by a hand is on the key, with nothing sticky about it.
        keyboard.reads(Reported::down(Pressed::AModifier(Modifier::Shift), at(10)));
        let shifted = keyboard
            .reads(Reported::down(Pressed::AKey('y'), at(20)))
            .expect("a plain keyboard");
        assert_eq!(shifted.with, [Modifier::Shift]);
        keyboard.reads(Reported::up(Pressed::AModifier(Modifier::Shift), at(30)));
        assert!(keyboard.stuck().is_empty(), "sticky keys were off");
    }
}
