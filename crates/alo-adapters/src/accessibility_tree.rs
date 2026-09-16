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
//! the screen. There is no method that subscribes to anything either: every
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
}
