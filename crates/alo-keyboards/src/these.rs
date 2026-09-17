//! A person's keyboards as a list that is never empty.
//!
//! A person always has a keyboard. That is not a rule somebody remembers to
//! check — it is this type: [`These`] is one keyboard and then any others, so
//! *which keyboard am I typing on* is a question with an answer rather than an
//! `Option` every caller has to decide what to do with, and there is no
//! arithmetic anywhere that can index past the end of it.
//!
//! [`These::of`] is the one door a list from outside comes through — a file a
//! person edited by hand — and it refuses an empty one there, once.

use crate::layout::Layout;

/// One keyboard, and any others after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct These {
    /// The first, which always exists.
    first: Layout,
    /// The rest, in the order they are switched through.
    then: Vec<Layout>,
}

impl These {
    /// Just this one.
    #[must_use]
    pub fn just(first: Layout) -> Self {
        Self {
            first,
            then: Vec::new(),
        }
    }

    /// These, in this order, or `None` when there are none.
    #[must_use]
    pub fn of(layouts: Vec<Layout>) -> Option<Self> {
        let mut layouts = layouts.into_iter();
        let first = layouts.next()?;
        Some(Self {
            first,
            then: layouts.collect(),
        })
    }

    /// How many there are, which is never nought.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.then.len().saturating_add(1)
    }

    /// The one at this place, and the first for anything past the end — so a
    /// place that has gone stale answers the keyboard a person certainly has
    /// rather than nothing at all.
    #[must_use]
    pub fn at(&self, place: usize) -> &Layout {
        match place.checked_sub(1).and_then(|past| self.then.get(past)) {
            Some(layout) => layout,
            None => &self.first,
        }
    }

    /// Each of them, in order.
    pub fn each(&self) -> impl Iterator<Item = &Layout> {
        std::iter::once(&self.first).chain(self.then.iter())
    }

    /// All of them, in order.
    #[must_use]
    pub fn to_vec(&self) -> Vec<Layout> {
        self.each().cloned().collect()
    }

    /// Whether this one is among them.
    #[must_use]
    pub fn holds(&self, layout: &Layout) -> bool {
        self.each().any(|one| one == layout)
    }

    /// Where this one is among them, if it is.
    #[must_use]
    pub fn where_is(&self, layout: &Layout) -> Option<usize> {
        self.each().position(|one| one == layout)
    }

    /// Add one at the end. Whether it is already there is the caller's
    /// question, because what a person is told about it is a refusal with
    /// words.
    pub fn add(&mut self, layout: Layout) {
        self.then.push(layout);
    }

    /// Take the one at this place out, unless it is the only one there is.
    ///
    /// Answers whether it went.
    pub fn take_out(&mut self, place: usize) -> bool {
        if self.then.is_empty() {
            return false;
        }
        match place.checked_sub(1) {
            None => {
                self.first = self.then.remove(0);
                true
            }
            Some(past) if past < self.then.len() => {
                self.then.remove(past);
                true
            }
            Some(_) => false,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn layout(named: &str) -> Layout {
        Layout::named(named).unwrap()
    }

    /// There is always a keyboard, and a place past the end still answers one.
    #[test]
    fn there_is_always_a_keyboard() {
        let these = These::just(layout("de"));
        assert_eq!(these.how_many(), 1);
        assert_eq!(these.at(0), &layout("de"));
        assert_eq!(these.at(9), &layout("de"));
    }

    /// **An empty list is refused at the one door it could come through.**
    #[test]
    fn an_empty_list_is_not_a_list_of_keyboards() {
        assert_eq!(These::of(Vec::new()), None);
        let these = These::of(vec![layout("de"), layout("gr")]).unwrap();
        assert_eq!(these.to_vec(), vec![layout("de"), layout("gr")]);
        assert_eq!(these.at(1), &layout("gr"));
        assert_eq!(these.where_is(&layout("gr")), Some(1));
        assert_eq!(these.where_is(&layout("fr")), None);
        assert!(these.holds(&layout("de")));
    }

    /// **The last one never goes**, wherever in the list it happens to be.
    #[test]
    fn the_last_one_never_goes() {
        let mut these = These::just(layout("de"));
        assert!(!these.take_out(0));
        assert_eq!(these.how_many(), 1);

        these.add(layout("gr"));
        assert!(these.take_out(0));
        assert_eq!(these.to_vec(), vec![layout("gr")]);

        these.add(layout("fr"));
        assert!(!these.take_out(7));
        assert!(these.take_out(1));
        assert_eq!(these.to_vec(), vec![layout("gr")]);
    }
}
