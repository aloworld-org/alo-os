//! Typing through the rented table: a dead key, then the letter it is put on.
//!
//! This is the whole of what alo OS adds to [`crate::Compose`]: a person does
//! not press a sequence, they press one key and then another, and something
//! has to hold the first one while it waits for the second. That waiting is
//! what a dead key *is*, and it is the same machinery the compose key uses —
//! `<dead_diaeresis> <u>` and `<Multi_key> <quotedbl> <u>` are two rows of one
//! table.
//!
//! # Four answers, and one of them is *this key means itself*
//!
//! A compositor asks this about every key that goes down, so the common answer
//! has to be *nothing to do with me*: [`Typed::MeansItself`] is `a` on an
//! ordinary keystroke, and the key goes to the window untouched. The other
//! three are the ones this file exists for — waiting, written, and a sequence
//! that went nowhere.
//!
//! **A sequence that goes nowhere writes nothing.** Not the letters pressed on
//! the way, not a best guess at what was meant. `<dead_diaeresis> <q>` is not
//! in the table, and inventing a `q̈` for it would be this crate writing
//! keyboard data — and doing it in the one place where a person would never
//! think to look for the bug.

use crate::compose::Compose;
use crate::keysym::Keysym;

/// What a key press did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Typed<'a> {
    /// Nothing in the table begins with this key, so it means what it always
    /// means and the window gets it.
    MeansItself,
    /// This key begins a sequence: nothing is written yet, and the next key
    /// decides.
    Waiting,
    /// The sequence is complete and writes this.
    Wrote(&'a str),
    /// The keys held so far go nowhere in the table. Nothing is written, and
    /// the next key starts again.
    NothingInTheTable,
}

impl<'a> Typed<'a> {
    /// What was written, if anything was.
    #[must_use]
    pub const fn written(self) -> Option<&'a str> {
        match self {
            Self::Wrote(written) => Some(written),
            Self::MeansItself | Self::Waiting | Self::NothingInTheTable => None,
        }
    }
}

/// Keys pressed so far, against one rented table.
///
/// One of these belongs to one keyboard focus: what is half-typed is a person's
/// own and is not carried across windows.
#[derive(Debug, Clone)]
pub struct Composing<'a> {
    /// The table, which this never changes.
    table: &'a Compose,
    /// The keys pressed since the last one that finished or gave up.
    held: Vec<Keysym>,
}

impl<'a> Composing<'a> {
    /// Nothing pressed yet, against this table.
    #[must_use]
    pub fn on(table: &'a Compose) -> Self {
        Self {
            table,
            held: Vec::new(),
        }
    }

    /// The keys pressed and not yet written, for a surface that shows a person
    /// that a dead key is waiting.
    #[must_use]
    pub fn waiting(&self) -> &[Keysym] {
        &self.held
    }

    /// One key went down.
    pub fn press(&mut self, key: Keysym) -> Typed<'a> {
        let table = self.table;
        self.held.push(key);
        if let Some(written) = table.writes(&self.held) {
            self.held.clear();
            return Typed::Wrote(written);
        }
        if table.begins_a_sequence(&self.held) {
            return Typed::Waiting;
        }
        let alone = self.held.len() == 1;
        self.held.clear();
        if alone {
            Typed::MeansItself
        } else {
            Typed::NothingInTheTable
        }
    }

    /// These keys went down, one after another: what the last one did.
    ///
    /// What a test writes, and what a person does.
    ///
    /// # Errors
    /// [`crate::KeysymError`] when one of the names is not a key's.
    pub fn typing(&mut self, keys: &[&str]) -> Result<Typed<'a>, crate::KeysymError> {
        let mut last = Typed::MeansItself;
        for key in keys {
            last = self.press(Keysym::named(key)?);
        }
        Ok(last)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_small_table;

    /// A dead key waits, and the letter after it is written.
    #[test]
    fn a_dead_key_waits_and_the_next_key_writes() {
        let table = a_small_table();
        let mut composing = Composing::on(&table);
        assert_eq!(
            composing.press(Keysym::named("dead_diaeresis").unwrap()),
            Typed::Waiting
        );
        assert_eq!(composing.waiting().len(), 1);
        assert_eq!(
            composing.press(Keysym::named("u").unwrap()),
            Typed::Wrote("ü")
        );
        assert!(composing.waiting().is_empty());
    }

    /// The compose key is the same machinery, over three keys rather than two.
    #[test]
    fn the_compose_key_is_the_same_machinery() {
        let table = a_small_table();
        let mut composing = Composing::on(&table);
        assert_eq!(
            composing.typing(&["Multi_key", "s"]).unwrap(),
            Typed::Waiting
        );
        assert_eq!(composing.typing(&["s"]).unwrap(), Typed::Wrote("ß"));
    }

    /// **An ordinary key means itself**, which is the answer a compositor gets
    /// for almost every key a person presses.
    #[test]
    fn an_ordinary_key_means_itself() {
        let table = a_small_table();
        let mut composing = Composing::on(&table);
        assert_eq!(composing.typing(&["q"]).unwrap(), Typed::MeansItself);
        assert!(composing.waiting().is_empty());
    }

    /// **A sequence that goes nowhere writes nothing at all**, and the next key
    /// starts again rather than carrying a wrong dead key forward.
    #[test]
    fn a_sequence_that_goes_nowhere_writes_nothing() {
        let table = a_small_table();
        let mut composing = Composing::on(&table);
        let typed = composing.typing(&["dead_diaeresis", "q"]).unwrap();
        assert_eq!(typed, Typed::NothingInTheTable);
        assert_eq!(typed.written(), None);
        assert!(composing.waiting().is_empty());
        assert_eq!(
            composing.typing(&["dead_diaeresis", "u"]).unwrap(),
            Typed::Wrote("ü")
        );
    }

    /// A name that is not a key's is refused rather than pressed.
    #[test]
    fn a_key_that_is_not_a_key_is_refused() {
        let table = a_small_table();
        let mut composing = Composing::on(&table);
        assert!(composing.typing(&["dead acute"]).is_err());
    }
}
