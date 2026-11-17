//! The rented accessibility tree, as this crate asks it — and everything it
//! can be asked.
//!
//! [`AccessibilityTree`] is the whole of what the fallback may ask an
//! application's tree: which applications are running and which each one is,
//! the facts about one thing, the text of one thing, the names of what can be
//! done to one thing, and to do one of those. On Linux it is carried by
//! `crate::accessibility_bus`; a test hands in a tree of its own.
//!
//! **What is not here is the guarantee.** There is no method that answers with
//! a position, a size or a picture, and none that takes one — so nothing built
//! on this trait can find a control by where it is, click a point, or look at
//! the screen. Scrolling ([`AccessibilityTree::scroll_to`], 2026-11-17) takes a
//! thing and an [`Edge`], never a point, and answers yes or no. There is no
//! method that subscribes to anything either: every
//! answer is to a question asked at that moment, and nothing arrives unasked.
//! A control with no name, and an area an application draws itself, are
//! therefore said in words (`crate::shown::Limit`) rather than guessed at.

/// Where one thing in the tree is: the connection that holds it, and its
/// object.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeAt {
    /// The connection the application holds it on.
    pub holder: String,
    /// The object.
    pub object: String,
}

/// One application the tree lists, and which application it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Running {
    /// Its root.
    pub at: NodeAt,
    /// The identifier of the installed application, **as its sandbox says** —
    /// never as the application names itself, which any program could write.
    /// [`None`] for a program nothing identifies.
    pub is: Option<String>,
}

/// What the tree says about one thing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    /// Its role, as `AtspiRole` numbers it.
    pub role: u32,
    /// Its name — empty for a thing with none.
    pub name: String,
    /// Its states, in the tree's two words.
    pub states: Vec<u32>,
    /// Where its children are, in order.
    pub children: Vec<NodeAt>,
    /// Whether it has text that can be asked for.
    pub has_text: bool,
    /// Whether it has more children than were listed, which a walk says it did
    /// not read.
    pub unlisted_children: bool,
}

/// Why the tree could not answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeFault {
    /// There is no accessibility tree on this machine to ask.
    NotThere,
    /// The thing asked about is no longer there — the window changed while it
    /// was read.
    Vanished,
    /// No answer came back in time.
    DidNotAnswer,
}

/// Everything the fallback may ask of an application's accessibility tree.
pub trait AccessibilityTree {
    /// The applications the tree lists, each with which application it is.
    ///
    /// # Errors
    /// [`TreeFault`].
    fn applications(&self) -> Result<Vec<Running>, TreeFault>;

    /// The facts about one thing.
    ///
    /// # Errors
    /// [`TreeFault`].
    fn facts(&self, at: &NodeAt) -> Result<Facts, TreeFault>;

    /// The text one thing shows.
    ///
    /// **Never asked of a password field** (`crate::walking`).
    ///
    /// # Errors
    /// [`TreeFault`].
    fn text(&self, at: &NodeAt) -> Result<String, TreeFault>;

    /// The names of what can be done to one thing, in the tree's order.
    ///
    /// # Errors
    /// [`TreeFault`].
    fn actions(&self, at: &NodeAt) -> Result<Vec<String>, TreeFault>;

    /// Do the action at this position of [`AccessibilityTree::actions`] —
    /// `true` when the application says it did.
    ///
    /// # Errors
    /// [`TreeFault`].
    fn act(&self, at: &NodeAt, action: usize) -> Result<bool, TreeFault>;

    /// What one thing says about itself besides its name, as `GetAttributes`
    /// answers — sorted by name.
    ///
    /// Read for one purpose: finding the surface of alo's own tree a person
    /// named (`crate::alos_own_tree::THE_SURFACE_ATTRIBUTE`). **Answers nothing by
    /// default**, which is what a tree that serves no attributes says; the bus
    /// asks the real one.
    ///
    /// # Errors
    /// [`TreeFault`].
    fn attributes(&self, at: &NodeAt) -> Result<Vec<(String, String)>, TreeFault> {
        let _ = at;
        Ok(Vec::new())
    }

    /// The number one thing holds — AT-SPI's `Value`: where a slider is, how
    /// far a progress bar has got — or [`None`] for a thing that holds none.
    /// Asked only of a [`crate::Role::Range`]. **None by default**, which is
    /// what a tree that offers no numbers says.
    ///
    /// # Errors
    /// [`TreeFault`].
    // The body is the answer itself and nothing else, so the mutation check
    // has no other version of it to try: `Ok(None)` is what it would put here.
    fn value(&self, _at: &NodeAt) -> Result<Option<f64>, TreeFault> {
        Ok(None)
    }

    /// Whether one thing takes text put into it through the tree — AT-SPI's
    /// `EditableText` (ADR 0153). **No by default**, which is what Chromium
    /// answers for every field it has; GTK answers yes.
    ///
    /// # Errors
    /// [`TreeFault`].
    fn takes_text(&self, at: &NodeAt) -> Result<bool, TreeFault> {
        let _ = at;
        Ok(false)
    }

    /// Where one thing's caret is, in characters from its start — or
    /// [`None`] when it has none to say.
    ///
    /// # Errors
    /// [`TreeFault`].
    fn caret(&self, at: &NodeAt) -> Result<Option<usize>, TreeFault> {
        let _ = at;
        Ok(None)
    }

    /// Put `text` into one thing at `position`, in characters — `true` when
    /// the application says it did. **The answer is not evidence the text
    /// arrived** (`docs/quirks/chromium-offers-no-editable-text-and-firefox-ignores-it.md`):
    /// a caller reads the text back.
    ///
    /// Never asked of a thing that does not [`AccessibilityTree::takes_text`];
    /// **no by default**.
    ///
    /// # Errors
    /// [`TreeFault`].
    fn insert_text(&self, at: &NodeAt, position: usize, text: &str) -> Result<bool, TreeFault> {
        let _ = (at, position, text);
        Ok(false)
    }

    /// Ask the application to scroll whatever holds one thing until that thing
    /// is on the screen, against `edge` of what it shows — `true` when the
    /// application says it did (`crate::scrolling`). **It takes a thing, never
    /// a point**: where the thing lands is the application's to work out, so
    /// nothing here learns or sends a position.
    ///
    /// **The answer is not evidence it moved**: a caller asks the thing's
    /// states again. **No by default**, so a tree that does not know scrolling
    /// scrolls nothing.
    ///
    /// # Errors
    /// [`TreeFault`].
    // The body is the answer itself and nothing else, so the mutation check
    // has no other version of it to try that means the same.
    fn scroll_to(&self, _at: &NodeAt, _edge: Edge) -> Result<bool, TreeFault> {
        Ok(false)
    }

    /// **The range a slider or a number box says it moves in** — AT-SPI's
    /// `Value`: its least, its most and its smallest step — or [`None`] for a
    /// thing that says none (`accessible.set_value`, 2026-11-17). **None by
    /// default**, which is what a tree that offers no numbers says.
    ///
    /// # Errors
    /// [`TreeFault`].
    // The body is the answer itself, so the mutation check has no other
    // version of it to try that means the same.
    fn range(&self, _at: &NodeAt) -> Result<Option<Bounds>, TreeFault> {
        Ok(None)
    }

    /// Move one thing to `value`, through AT-SPI's `Value` — `true` when the
    /// application took it. **The answer is not evidence it moved**: a caller
    /// reads [`AccessibilityTree::value`] back. Never asked of a thing whose
    /// [`AccessibilityTree::range`] is [`None`]; **no by default**.
    ///
    /// # Errors
    /// [`TreeFault`].
    fn set_value(&self, _at: &NodeAt, _value: f64) -> Result<bool, TreeFault> {
        Ok(false)
    }
}

/// The range a slider or a number box moves in, as the application says it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    /// The least it may be.
    pub least: f64,
    /// The most it may be.
    pub most: f64,
    /// Its smallest step — `0` for one that says none.
    pub step: f64,
}

impl Bounds {
    /// The range an application said, from its least, its most and its
    /// smallest step — or [`None`] where it said no least or no most, or a
    /// least above its most, which is no range at all. A step it did not say,
    /// or said below nothing, is `0`.
    #[must_use]
    pub fn said(least: Option<f64>, most: Option<f64>, step: Option<f64>) -> Option<Self> {
        let (least, most) = (least?, most?);
        (least <= most).then(|| Self {
            least,
            most,
            step: step.map_or(0.0, |step| step.max(0.0)),
        })
    }
}

/// Which edge of what a window shows a thing is scrolled to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// The thing's top against the top of what is shown — scrolling down.
    Top,
    /// The thing's bottom against the bottom of what is shown — scrolling up.
    Bottom,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A range is what the application said, or nothing**: both ends are
    /// needed, a least above the most is refused, and a step below nothing
    /// is none.
    #[test]
    fn a_range_is_both_ends_in_order_and_a_step_never_below_nothing() {
        let range = |least, most, step| Bounds::said(least, most, step);
        assert_eq!(
            range(Some(0.0), Some(100.0), Some(5.0)),
            Some(Bounds {
                least: 0.0,
                most: 100.0,
                step: 5.0
            })
        );
        assert_eq!(
            range(Some(3.0), Some(3.0), None),
            Some(Bounds {
                least: 3.0,
                most: 3.0,
                step: 0.0
            })
        );
        assert_eq!(
            range(Some(0.0), Some(1.0), Some(-1.0)).map(|it| it.step),
            Some(0.0)
        );
        assert_eq!(range(Some(5.0), Some(1.0), Some(1.0)), None);
        assert_eq!(range(None, Some(1.0), Some(1.0)), None);
        assert_eq!(range(Some(0.0), None, Some(1.0)), None);
    }

    /// A tree that answers only what every tree must.
    struct OnlyWhatIsRequired;

    impl AccessibilityTree for OnlyWhatIsRequired {
        fn applications(&self) -> Result<Vec<Running>, TreeFault> {
            Ok(Vec::new())
        }
        fn facts(&self, _: &NodeAt) -> Result<Facts, TreeFault> {
            Err(TreeFault::Vanished)
        }
        fn text(&self, _: &NodeAt) -> Result<String, TreeFault> {
            Ok(String::new())
        }
        fn actions(&self, _: &NodeAt) -> Result<Vec<String>, TreeFault> {
            Ok(Vec::new())
        }
        fn act(&self, _: &NodeAt, _: usize) -> Result<bool, TreeFault> {
            Ok(false)
        }
    }

    /// **A tree that offers no numbers says none**, rather than inventing
    /// one: a slider read from it holds no value.
    #[test]
    fn a_tree_that_offers_no_numbers_says_none() {
        let at = NodeAt {
            holder: ":1.7".to_owned(),
            object: "/slider".to_owned(),
        };
        assert_eq!(OnlyWhatIsRequired.value(&at), Ok(None));
        // Nor a range, and it moves nothing.
        assert_eq!(OnlyWhatIsRequired.range(&at), Ok(None));
        assert_eq!(OnlyWhatIsRequired.set_value(&at, 1.0), Ok(false));
    }
}
