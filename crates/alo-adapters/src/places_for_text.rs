//! **Every place an application shows that text could go into**, with the
//! window it is in — and which of them a name the person approved means.
//!
//! One walk, read by three callers: typing, which goes into the one place a
//! name means ([`crate::typing_text::Typing::the_field`]); a refusal for a name
//! that means none, which says what is there; and the answer after alo opens a
//! file, which says the name typing into that file takes
//! ([`where_a_file_is_typed_into`]). **The rule for what a name means is
//! written once, in [`is_meant_by`]**, so the name the opening answer offers is
//! by construction one the typing verb accepts.

use crate::accessibility_tree::{AccessibilityTree, NodeAt, TreeFault};
use crate::role::Role;
use crate::walking::{DEEPEST, MOST_THINGS};

/// `ATSPI_STATE_EDITABLE`.
pub(crate) const EDITABLE: u32 = 7;

/// `ATSPI_STATE_FOCUSED`.
pub(crate) const FOCUSED: u32 = 12;

/// `ATSPI_STATE_SHOWING`.
pub(crate) const SHOWING: u32 = 25;

/// `ATSPI_ROLE_TERMINAL`.
pub(crate) const TERMINAL: u32 = 60;

/// The most names a refusal lists, so a window with hundreds of fields is
/// said in a sentence.
pub const THE_MOST_NAMES_SAID: usize = 8;

/// Whether the tree's two words of states hold this one.
pub(crate) fn holds(states: &[u32], state: u32) -> bool {
    let word = usize::try_from(state / 32).unwrap_or(usize::MAX);
    states
        .get(word)
        .is_some_and(|bits| bits & (1_u32 << (state % 32)) != 0)
}

/// One place text could go into, showing now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct APlaceForText {
    /// Where it is.
    pub at: NodeAt,
    /// Its role, as `AtspiRole` numbers it.
    pub role: u32,
    /// Its own name — empty for one with none.
    pub name: String,
    /// Its states, in the tree's two words.
    pub states: Vec<u32>,
    /// The title of the window it is in — empty where none was seen above it.
    pub window: String,
}

impl APlaceForText {
    /// Whether it has the keyboard.
    #[must_use]
    pub fn has_the_keyboard(&self) -> bool {
        holds(&self.states, FOCUSED)
    }
}

/// **Whether `name` means this place**: named exactly so, or — where an
/// application names its document nothing, as GNOME Text Editor does, or
/// something other than the file, as a code editor may — an unnamed one, or
/// the one with the keyboard, in a window whose title holds the name. A model
/// asked to write in hello.txt names it hello.txt, which is what the window
/// says (the walk of 2026-11-14 saw one guess "Document" instead).
#[must_use]
pub fn is_meant_by(place: &APlaceForText, name: &str) -> bool {
    let in_its_window = !place.window.is_empty() && place.window.contains(name);
    place.name == name || (in_its_window && (place.name.is_empty() || place.has_the_keyboard()))
}

/// Every place for text `application` shows now, in the order the walk met
/// them, at most [`MOST_THINGS`] things looked at and [`DEEPEST`] deep.
///
/// # Errors
/// [`TreeFault`], other than a thing that vanished while it was read, which is
/// skipped.
pub fn the_places_for_text(
    tree: &dyn AccessibilityTree,
    application: &str,
) -> Result<Vec<APlaceForText>, TreeFault> {
    let roots: Vec<NodeAt> = tree
        .applications()?
        .into_iter()
        .filter(|running| running.is.as_deref() == Some(application))
        .map(|running| running.at)
        .collect();
    let mut found = Vec::new();
    let mut waiting: Vec<(NodeAt, usize, String)> =
        roots.into_iter().map(|at| (at, 0, String::new())).collect();
    let mut seen = 0_usize;
    while let Some((at, depth, window)) = waiting.pop() {
        seen += 1;
        if seen > MOST_THINGS {
            break;
        }
        let facts = match tree.facts(&at) {
            Ok(facts) => facts,
            Err(TreeFault::Vanished) => continue,
            Err(fault) => return Err(fault),
        };
        let role = Role::of_the_tree(facts.role);
        let a_place_for_text =
            matches!(role, Role::TextField | Role::Document | Role::PasswordField)
                || facts.role == TERMINAL;
        if a_place_for_text && holds(&facts.states, SHOWING) {
            found.push(APlaceForText {
                at: at.clone(),
                role: facts.role,
                name: facts.name.clone(),
                states: facts.states.clone(),
                window: window.clone(),
            });
        }
        let window = if role == Role::Window {
            facts.name.clone()
        } else {
            window
        };
        if depth < DEEPEST {
            waiting.extend(
                facts
                    .children
                    .into_iter()
                    .map(|child| (child, depth + 1, window.clone())),
            );
        }
    }
    Ok(found)
}

/// **The places `name` means**, and of several the one with the keyboard
/// where exactly one has it: with the person's own VSCodium open beside alo's
/// copy, both show an editor of the same name, and the one alo may type into
/// is the one the keyboard is in. Two that both have it, or none of several,
/// is left as several: nothing is guessed.
#[must_use]
pub fn the_ones_meant<'p>(places: &'p [APlaceForText], name: &str) -> Vec<&'p APlaceForText> {
    let meant: Vec<&APlaceForText> = places
        .iter()
        .filter(|place| is_meant_by(place, name))
        .collect();
    if meant.len() > 1 {
        let focused: Vec<&APlaceForText> = meant
            .iter()
            .copied()
            .filter(|place| place.has_the_keyboard())
            .collect();
        if focused.len() == 1 {
            return focused;
        }
    }
    meant
}

/// **What a refusal says is there**: the names of the places for text, and
/// the titles of their windows for those with none, once each, at most
/// [`THE_MOST_NAMES_SAID`], each in quotes.
#[must_use]
pub fn what_is_there(places: &[APlaceForText]) -> Vec<String> {
    let mut said: Vec<String> = Vec::new();
    for place in places {
        let named = if place.name.trim().is_empty() {
            place.window.trim()
        } else {
            place.name.trim()
        };
        if named.is_empty() || said.iter().any(|already| already == named) {
            continue;
        }
        said.push(named.to_owned());
        if said.len() == THE_MOST_NAMES_SAID {
            break;
        }
    }
    said
}

/// **Where text typed into the file `file_name` just opened in `application`
/// goes**: the name to give the typing verb, and the title of the window —
/// for the answer after opening, so a model is told the name rather than
/// guessing one.
///
/// The name is the file's own, and only where it means exactly one place
/// ([`the_ones_meant`]): so typing with it reaches this place, by the same
/// rule typing uses. [`None`] where no one place is meant, or the tree could
/// not be read: the answer then says no more than it did before.
#[must_use]
pub fn where_a_file_is_typed_into(
    tree: &dyn AccessibilityTree,
    application: &str,
    file_name: &str,
) -> Option<(String, String)> {
    let places = the_places_for_text(tree, application).ok()?;
    let meant = the_ones_meant(&places, file_name);
    let [place] = meant.as_slice() else {
        return None;
    };
    // **And only one typing would go into**: never a password field or a
    // terminal, and one that takes text — the checks `Typing::the_field`
    // makes after it finds the place.
    let refused_by_typing = Role::of_the_tree(place.role) == Role::PasswordField
        || place.role == TERMINAL
        || !holds(&place.states, EDITABLE);
    if refused_by_typing {
        return None;
    }
    Some((file_name.to_owned(), place.window.clone()))
}
