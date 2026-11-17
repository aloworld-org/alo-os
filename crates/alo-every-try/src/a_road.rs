//! **A request and the road it is expected to take**: the verbs, in order,
//! the arguments whose values matter, and what the machine answers at each.

use alo_adapters::places_for_text::APlaceForText;

/// What the machine answers when a step is taken — as a booted machine at full
/// access would, written by [`crate::the_machine_answers`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answered {
    /// The file was opened in the application.
    AFileOpened,
    /// The file was opened, and the machine read its window and says where
    /// typing into it goes, as `alo-agentd` does once the window is in front
    /// (`Done::of_a_file_opened`).
    AFileOpenedTypingInto {
        /// The name the typing verb takes for it.
        name: String,
        /// The window's title.
        window: String,
    },
    /// The application was started.
    AnApplicationOpened,
    /// The application's window was read: its title, and one empty field or
    /// document, by the name the application shows it under.
    AWindowShowing {
        /// The window's title.
        title: String,
        /// The field's name.
        field: String,
    },
    /// The text was typed, read back exactly, and saved.
    TypedAndSaved,
    /// The folder was published to the site of the account GitHub's own `gh`
    /// is signed in as, which the machine fills in and the model never gives.
    Published {
        /// The account.
        account: String,
    },
    /// What the catalogue holds under the kind of work the call chose, read
    /// from the decided catalogue (`catalogue.toml`) as the machine reads it,
    /// so a model that looked under another kind is shown what that kind holds.
    TheCatalogue(alo_software::Catalogue),
    /// What the verb did, as the protocol carries it: told as done without
    /// asking where the step changes something (the call's own sentence
    /// inside the full-access words), and as the answer to a read otherwise.
    ///
    /// For the verbs whose answer has no shorter kind above: what a folder
    /// holds, a file says, a search found, the catalogue offers, the record
    /// holds, or what was renamed, moved, archived, installed or pressed.
    With(alo_protocol::Done),
}

/// One step of a road: the verb the model is expected to ask for, the values
/// some of its arguments must have, and what the machine answers.
///
/// **Every argument is checked**, not only the ones named here: a step is
/// matched through `Verbs::call`, which refuses an argument the verb does not
/// take, one left out and one given twice. The ones named here are those whose
/// *value* the road decides — the application, the path, the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// The verb the step expects.
    verb: &'static str,
    /// The arguments whose values the road decides, and those values.
    expecting: Vec<(&'static str, String)>,
    /// The arguments that name a place for text, and the place as the window
    /// shows it: any name the machine's typing takes for it will do.
    fields: Vec<(&'static str, APlaceForText)>,
    /// What the machine answers when the step is taken.
    answered: Answered,
    /// Whether this is a look the model may skip, or take more than once.
    look: bool,
    /// Whether the model may take this step again, as often as it asks.
    again: bool,
}

impl Step {
    /// A step asking for `verb`, answered as `answered`.
    #[must_use]
    pub fn of(verb: &'static str, answered: Answered) -> Self {
        Self {
            verb,
            expecting: Vec::new(),
            fields: Vec::new(),
            answered,
            look: false,
            again: false,
        }
    }

    /// **And the model may take it again**, as often as it asks, without the
    /// road moving on — a page and then another page. Unlike a look it may be
    /// a change, and it is never skipped: it is taken at least once. Each time
    /// is answered as the level answers a change, so at ask each time each
    /// page is one card.
    #[must_use]
    pub const fn as_often_as_asked(mut self) -> Self {
        self.again = true;
        self
    }

    /// Whether the model may take this step again.
    #[must_use]
    pub const fn is_taken_as_often_as_asked(&self) -> bool {
        self.again
    }

    /// **And it is a look**: a read the model may skip when it already knows
    /// what the read would tell it, or take again — so a road holds the model
    /// to the result the machine needs, not to one way of getting there. A
    /// look is never a road's last step, because a road that ended on one
    /// could not tell skipping it from stopping short; `the_set`'s tests hold
    /// every look to being a read and not last.
    #[must_use]
    pub const fn a_look(mut self) -> Self {
        self.look = true;
        self
    }

    /// Whether this step is a look.
    #[must_use]
    pub const fn is_a_look(&self) -> bool {
        self.look
    }

    /// And `argument` must be given as exactly `is`.
    #[must_use]
    pub fn expecting(mut self, argument: &'static str, is: impl Into<String>) -> Self {
        self.expecting.push((argument, is.into()));
        self
    }

    /// And `argument` must name `place` as the machine's typing reads a name
    /// (`alo_adapters::places_for_text::is_meant_by`): its own name, or the
    /// title of its window where it has the keyboard or no name.
    #[must_use]
    pub fn expecting_the_field(mut self, argument: &'static str, place: APlaceForText) -> Self {
        self.fields.push((argument, place));
        self
    }

    /// The arguments that name a place for text, and those places.
    #[must_use]
    pub fn fields(&self) -> &[(&'static str, APlaceForText)] {
        &self.fields
    }

    /// The verb this step expects.
    #[must_use]
    pub const fn verb(&self) -> &'static str {
        self.verb
    }

    /// The arguments whose values this step decides, and those values.
    #[must_use]
    pub fn expected(&self) -> &[(&'static str, String)] {
        &self.expecting
    }

    /// What the machine answers when this step is taken.
    #[must_use]
    pub const fn answered(&self) -> &Answered {
        &self.answered
    }
}

/// One request in a person's words, and the road it is expected to take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ARequest {
    /// What the request is called in a summary.
    named: &'static str,
    /// The request, as the person types it.
    words: String,
    /// The steps, in order.
    road: Vec<Step>,
    /// Reads the machine would answer at any point of the road, which a try
    /// may take aside of it ([`ARequest::or_looking_at`]).
    aside: Vec<Step>,
    /// **Other roads to the same place** ([`ARequest::or_by_another_road`]),
    /// which a try may take instead from where they part.
    other: Vec<Vec<Step>>,
}

impl ARequest {
    /// A request called `named`, typed as `words`, expected to take `road` and
    /// then answer the person in words.
    #[must_use]
    pub fn asking(named: &'static str, words: impl Into<String>, road: Vec<Step>) -> Self {
        Self {
            named,
            words: words.into(),
            road,
            aside: Vec::new(),
            other: Vec::new(),
        }
    }

    /// **And another road that ends where the person asked to be** — a page at
    /// a time, say, where the road goes in one step. A try that asks, where
    /// the two roads part, for the other road's step is held to the other road
    /// from there, and a pass by it is a pass, counted apart in the summary so
    /// the longer way is seen. The roads share their steps up to where they
    /// part, and a try may go on from one to another while every step it has
    /// taken is both roads' (the first declared, where two would do); a road
    /// that ends with a read of what the person asked to see holds the try to
    /// having seen it.
    #[must_use]
    pub fn or_by_another_road(mut self, road: Vec<Step>) -> Self {
        self.other.push(road);
        self
    }

    /// The other roads, in the order they were declared.
    #[must_use]
    pub fn other_roads(&self) -> &[Vec<Step>] {
        &self.other
    }

    /// **And a read the machine would answer at any point of the road**,
    /// which a try may take aside of it — at most
    /// `crate::a_try::THE_MOST_LOOKS_ASIDE` a try — without leaving the road.
    /// The try must still take every step after it. Only a read: a change is
    /// never aside of the road.
    #[must_use]
    pub fn or_looking_at(mut self, read: Step) -> Self {
        self.aside.push(read);
        self
    }

    /// The reads a try may take aside of the road.
    #[must_use]
    pub fn aside(&self) -> &[Step] {
        &self.aside
    }

    /// What the request is called in a summary.
    #[must_use]
    pub const fn named(&self) -> &'static str {
        self.named
    }

    /// The request, as the person types it.
    #[must_use]
    pub fn words(&self) -> &str {
        &self.words
    }

    /// The road, in order.
    #[must_use]
    pub fn road(&self) -> &[Step] {
        &self.road
    }

    /// The road as a person reads it: `open_file_in_application → …`.
    #[must_use]
    pub fn the_road_written(&self) -> String {
        self.road
            .iter()
            .map(Step::verb)
            .chain(std::iter::once("an answer in words"))
            .collect::<Vec<_>>()
            .join(" → ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_road_is_written_in_order_and_ends_in_words() {
        let request = ARequest::asking(
            "two steps",
            "do two things",
            vec![
                Step::of("first", Answered::AFileOpened),
                Step::of("second", Answered::TypedAndSaved),
            ],
        );
        assert_eq!(
            request.the_road_written(),
            "first → second → an answer in words"
        );
    }

    #[test]
    fn a_step_keeps_every_value_it_was_given_in_order() {
        let step = Step::of("verb", Answered::AFileOpened)
            .expecting("one", "1")
            .expecting("two", "2");
        assert_eq!(
            step.expected(),
            [("one", "1".to_owned()), ("two", "2".to_owned())]
        );
    }
}
