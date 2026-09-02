//! One call of one verb: what arrived, what it means, and whether it may run.
//!
//! A [`Call`] is what a set of arguments becomes once every one of them has
//! been validated against the verb that declared them. Nothing downstream sees
//! a [`crate::arg::Given`] again — an executor is handed a `Call` or it is
//! handed a refusal, and there is no third thing.
//!
//! A call carries three answers, and they are separate on purpose:
//!
//! - **what it may touch** ([`Call::asks`]) — the questions to put to the
//!   grants, one per argument the verb said its grant is over;
//! - **what a person would be approving** ([`Call::sentence`]) — the verb's
//!   sentence with this call's validated values in it, in the language the
//!   person reads;
//! - **whether it waits** ([`Call::waits_for_approval`]) — a read answers
//!   inside the turn, a change waits for one approval (ADR 0001 §5).
//!
//! **A call carries a key and its values, never a rendered sentence** (item
//! 9g). It used to render one in the language the verb was declared in and keep
//! the string, so a shell showed a translated sentence while the approval and
//! the record kept an English one — two accounts of the same moment, which is
//! exactly what item 9e refused for refusals. Now there is one value, and the
//! screen and the record render it with the same vocabulary.
//!
//! **Being permitted and being approved are different questions.** This file
//! answers only the first, and a permitted change still has not run: it becomes
//! a [`crate::Proposal`], and one approval of that turns into one execution.
//! Building the two into one method is how "one approval, one execution" would
//! quietly become "one approval, whatever the grant allows".

use std::collections::BTreeMap;
use std::time::SystemTime;

use alo_strings::{Filling, Key, Said, Strings};
use serde::Serialize;

use crate::arg::{ArgError, Given, Value};
use crate::grant::Grantee;
use crate::grants::{GrantId, Grants};
use crate::reach::Ask;
use crate::refusing::NotGranted;
use crate::verb::{Effect, Requires, Verb};
use crate::words;

/// Why a call could not be made.
///
/// These are refusals at the boundary: the call never became a call, so nothing
/// was executed and no grant was consulted. Each is recorded, because a refusal
/// is exactly the thing a security review asks about (ADR 0001 §7).
///
/// **No `Display`**, like the two refusals it can carry. [`CallError::said`] is
/// the road to words, and for an argument that did not survive validation it is
/// [`ArgError::said`] — one sentence, said once, wherever the refusal came
/// from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallError {
    /// A verb that is not on the list.
    NoSuchVerb {
        /// The name that was asked for.
        name: String,
    },
    /// An argument the verb does not take.
    NoSuchArgument {
        /// The verb that was called.
        verb: String,
        /// The argument that was given.
        argument: String,
    },
    /// An argument the verb takes, that nothing gave.
    Missing {
        /// The verb that was called.
        verb: String,
        /// The argument that was not given.
        argument: String,
        /// What names what that argument is for, so the answer is in the
        /// question — and in the reader's language, which is what item 9g
        /// carrying the key rather than the words is for.
        purpose: Key,
    },
    /// The same argument given twice.
    SameArgumentTwice {
        /// The argument that was given twice.
        argument: String,
    },
    /// An argument that did not survive validation.
    Argument(ArgError),
}

impl From<ArgError> for CallError {
    fn from(why: ArgError) -> Self {
        Self::Argument(why)
    }
}

impl CallError {
    /// What this says, in the language the person reads.
    ///
    /// An argument's refusal is that argument's own sentence rather than a
    /// second one wrapped around it: a person who sent a relative path needs to
    /// be told to send a full one, not that a call could not be made.
    ///
    /// The one that quotes a verb's own words looks them up here, with the same
    /// vocabulary as the sentence around them — so *what this argument is for*
    /// is not a clause in the language the verb happened to be declared in.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::Argument(why) => why.said(strings),
            Self::NoSuchVerb { name } => strings.say(
                &words::NO_SUCH_VERB.key(),
                &Filling::of("verb", name.clone()),
            ),
            Self::NoSuchArgument { verb, argument } => strings.say(
                &words::NO_SUCH_ARGUMENT.key(),
                &Filling::of("verb", verb.clone()).and("argument", argument.clone()),
            ),
            Self::Missing {
                verb,
                argument,
                purpose,
            } => strings.say(
                &words::ARGUMENT_MISSING.key(),
                &Filling::of("verb", verb.clone())
                    .and("argument", argument.clone())
                    .and(
                        "purpose",
                        strings.say(purpose, &Filling::nothing()).into_text(),
                    ),
            ),
            Self::SameArgumentTwice { argument } => strings.say(
                &words::SAME_ARGUMENT_TWICE.key(),
                &Filling::of("argument", argument.clone()),
            ),
        }
    }
}

/// A call that has been validated, and is now a thing that could be run.
///
/// Serialises so that the record can keep what ran, or what was refused. It
/// does not deserialise: a call exists because a [`Verb`] validated it, and one
/// read back from a file would be a call nothing had checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Call {
    /// The verb, by name.
    verb: String,
    /// Read or change.
    effect: Effect,
    /// Every argument, validated, by name — sorted, so the record of a call is
    /// the same however the arguments arrived.
    values: BTreeMap<String, Value>,
    /// What has to be granted for this to run, in the order the verb named it.
    asks: Vec<Ask>,
    /// What names the sentence a person approves. The words are looked up
    /// wherever somebody reads them; the gaps are filled from the values above
    /// and from nothing else.
    sentence: Key,
}

impl Call {
    /// Validate a set of arguments against a verb.
    ///
    /// Every argument the verb declares is required, so a call is complete or
    /// it is a refusal. What that gives the sentence is the thing worth
    /// noticing: [`Verb::checked`] refuses a sentence that names an argument
    /// the verb does not take or leaves one out, and every argument it does
    /// take has a value here — so the filling covers every gap, in the source
    /// and in any translation, because `alo_strings::Vocabulary` refuses a
    /// translation that invents a gap or drops one.
    ///
    /// # Errors
    /// [`CallError`], saying what to send instead.
    pub fn of(verb: &Verb, given: &[(&str, Given)]) -> Result<Self, CallError> {
        let mut values = BTreeMap::new();
        for (name, value) in given {
            let name = name.trim();
            let arg = verb.arg(name).ok_or_else(|| CallError::NoSuchArgument {
                verb: verb.name().to_owned(),
                argument: name.to_owned(),
            })?;
            if values.contains_key(name) {
                return Err(CallError::SameArgumentTwice {
                    argument: name.to_owned(),
                });
            }
            values.insert(name.to_owned(), arg.validate(value)?);
        }
        for arg in verb.args() {
            if !values.contains_key(arg.name()) {
                return Err(CallError::Missing {
                    verb: verb.name().to_owned(),
                    argument: arg.name().to_owned(),
                    purpose: arg.purpose_key(),
                });
            }
        }
        let asks = asks_of(verb, &values);
        Ok(Self {
            verb: verb.name().to_owned(),
            effect: verb.effect(),
            values,
            asks,
            sentence: verb.sentence().key().clone(),
        })
    }

    /// The verb this is a call of.
    #[must_use]
    pub fn verb(&self) -> &str {
        &self.verb
    }

    /// Whether it answers or changes something.
    #[must_use]
    pub fn effect(&self) -> Effect {
        self.effect
    }

    /// Whether it has to be approved before it can run.
    #[must_use]
    pub fn waits_for_approval(&self) -> bool {
        self.effect.waits_for_approval()
    }

    /// **What a person would be approving, in the language they read.**
    ///
    /// The words are the verb's and the values are this call's, which is the
    /// whole point of the sentence: nothing the model wrote appears in it, and
    /// translating moves the words and never the values. The answer says where
    /// it came from, so a machine showing English because nobody translated the
    /// verb is a fact something can see.
    #[must_use]
    pub fn sentence(&self, strings: &Strings) -> Said {
        strings.say(&self.sentence, &self.filling())
    }

    /// What names the sentence a person approves.
    ///
    /// For anything that has to keep the sentence as a value rather than as
    /// words — a record, a refusal that quotes it — and render it where
    /// somebody reads it.
    #[must_use]
    pub fn sentence_key(&self) -> &Key {
        &self.sentence
    }

    /// What goes into the sentence's gaps: each validated value, under the name
    /// of the argument that carried it.
    #[must_use]
    pub fn filling(&self) -> Filling {
        let mut filling = Filling::nothing();
        for (argument, value) in &self.values {
            filling = filling.and(argument.clone(), value.describe());
        }
        filling
    }

    /// What has to be granted for this call to run.
    ///
    /// Empty only when the verb declared [`Requires::Nothing`] with its written
    /// reason. Every path here is the path as it was given: whatever executes
    /// the verb resolves symbolic links and asks about the real path, for the
    /// reason set out in [`crate::path`].
    #[must_use]
    pub fn asks(&self) -> &[Ask] {
        &self.asks
    }

    /// One validated argument, by name.
    #[must_use]
    pub fn value(&self, argument: &str) -> Option<&Value> {
        self.values.get(argument)
    }

    /// Every validated argument.
    #[must_use]
    pub fn values(&self) -> &BTreeMap<String, Value> {
        &self.values
    }

    /// Which grants permit this call — one for each thing it would touch, in
    /// the order the verb named them.
    ///
    /// Every ask, not one of them: a call that may touch one of two folders is
    /// refused, because half of a move is not a smaller move. The join lives
    /// here rather than in the daemon so that no caller can check some of the
    /// asks and believe it checked them all, and it stops at the first refusal
    /// so that a permitted answer is never half an answer.
    ///
    /// A verb that declared [`Requires::Nothing`] permits with an empty list.
    /// That is the honest answer to *against which grant*: none, for the reason
    /// its author wrote down.
    ///
    /// # Errors
    /// The first refusal, in the grants' own terms — because a person reading
    /// one needs to know about the grant rather than about the verb: that it
    /// expired, or that it was never made, and which folder it was over.
    pub fn permitting(
        &self,
        grants: &Grants,
        grantee: &Grantee,
        now: SystemTime,
    ) -> Result<Vec<GrantId>, NotGranted> {
        self.asks
            .iter()
            .map(|ask| grants.permitting(grantee, ask, now))
            .collect()
    }

    /// Whether this agent's grants permit every part of this call.
    #[must_use]
    pub fn permitted_by(&self, grants: &Grants, grantee: &Grantee, now: SystemTime) -> bool {
        self.permitting(grants, grantee, now).is_ok()
    }

    /// Why it is not permitted — `None` when it is.
    #[must_use]
    pub fn refusal(
        &self,
        grants: &Grants,
        grantee: &Grantee,
        now: SystemTime,
    ) -> Option<NotGranted> {
        self.permitting(grants, grantee, now).err()
    }
}

/// What a call has to have granted, from what its verb requires.
///
/// A verb that passed [`Verb::checked`] names only arguments it declares, and
/// only ones a grant can be over, so every name here resolves. A name that
/// somehow did not would silently reduce what is checked, which is why this
/// takes the values that are present rather than assuming an index into them.
fn asks_of(verb: &Verb, values: &BTreeMap<String, Value>) -> Vec<Ask> {
    match verb.requires() {
        Requires::Grants(over) => over
            .iter()
            .filter_map(|argument| values.get(argument))
            .filter_map(Value::as_ask)
            .collect(),
        Requires::Nothing { .. } => Vec::new(),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::test_calls::{
        MOVING_INTO, MOVING_SENTENCE, THE_WORDS, archiving_march as moving_march, files, granting,
        hour, list_folder, move_file, noon, reading,
    };
    use crate::testing::translating;
    use alo_strings::Word;
    use std::path::PathBuf;

    /// A call becomes a sentence and a set of questions for the grants, and
    /// nothing the model wrote appears in either.
    #[test]
    fn a_call_carries_its_sentence_and_what_it_would_touch() {
        let call = moving_march();
        assert_eq!(call.verb(), "move_file");
        assert!(call.waits_for_approval());
        assert_eq!(
            call.sentence(&reading()).text(),
            "move /home/anna/Invoices/march.pdf into /home/anna/Archive"
        );
        assert_eq!(
            call.asks(),
            &[
                Ask::path("/home/anna/Invoices/march.pdf"),
                Ask::path("/home/anna/Archive"),
            ]
        );
        assert_eq!(
            call.value("file"),
            Some(&Value::Path(PathBuf::from("/home/anna/Invoices/march.pdf")))
        );
    }

    /// The guarantee `CLAUDE.md` asks for by name: **a verb cannot reach
    /// outside its grant.** The call is well-formed, the sentence is
    /// generated, and it is still refused.
    #[test]
    fn a_verb_cannot_reach_outside_its_grant() {
        let call = Call::of(
            &list_folder(),
            &[("folder", Given::text("/home/anna/Taxes"))],
        )
        .unwrap();
        let grants = granting(&["/home/anna/Invoices"]);
        assert!(!call.permitted_by(&grants, &files(), noon()));
        let refusal = call
            .refusal(&grants, &files(), noon())
            .unwrap()
            .said(&crate::testing::in_english());
        assert!(refusal.text().contains("has not been granted"), "{refusal}");
        assert!(refusal.text().contains("/home/anna/Taxes"), "{refusal}");
    }

    /// Every part of a call has to be granted. Half of a move is not a smaller
    /// move — it is a file leaving a folder somebody granted and arriving in
    /// one they did not.
    #[test]
    fn a_call_is_refused_unless_every_part_of_it_is_granted() {
        let call = moving_march();
        let half = granting(&["/home/anna/Invoices"]);
        assert!(!call.permitted_by(&half, &files(), noon()));
        assert_eq!(
            call.refusal(&half, &files(), noon()).unwrap().wanted(),
            &Ask::path("/home/anna/Archive")
        );

        let both = granting(&["/home/anna/Invoices", "/home/anna/Archive"]);
        assert!(call.permitted_by(&both, &files(), noon()));
        assert!(call.refusal(&both, &files(), noon()).is_none());
    }

    /// A permitted call says which grants permitted it, one for each thing it
    /// would touch — the last of the four answers ADR 0001 §7 asks of a record.
    #[test]
    fn a_permitted_call_names_the_grants_that_permitted_it() {
        let grants = granting(&["/home/anna/Invoices", "/home/anna/Archive"]);
        let held: Vec<_> = grants.active_at(noon()).map(|held| held.id).collect();
        assert_eq!(
            moving_march().permitting(&grants, &files(), noon()),
            Ok(held)
        );

        // Half a move names no grant at all, rather than the one that did say
        // yes: a refused call touched nothing, so nothing permitted it.
        let half = granting(&["/home/anna/Invoices"]);
        assert_eq!(
            moving_march()
                .permitting(&half, &files(), noon())
                .unwrap_err()
                .wanted(),
            &Ask::path("/home/anna/Archive")
        );
    }

    /// A grant that has run out permits nothing, and the call that was
    /// permitted a moment ago is refused with the reason a person can act on.
    #[test]
    fn a_call_permitted_now_is_refused_once_the_grant_ends() {
        let call = moving_march();
        let grants = granting(&["/home/anna/Invoices", "/home/anna/Archive"]);
        assert!(call.permitted_by(&grants, &files(), noon()));
        assert!(!call.permitted_by(&grants, &files(), noon() + hour()));
        let refusal = call.refusal(&grants, &files(), noon() + hour()).unwrap();
        assert!(matches!(refusal, NotGranted::Lapsed { .. }), "{refusal:?}");
        assert!(
            refusal
                .said(&crate::testing::in_english())
                .text()
                .contains("has expired")
        );
    }

    /// One agent's grant is not another's, however well-formed the call is.
    #[test]
    fn a_call_by_another_agent_is_refused() {
        let call = moving_march();
        let grants = granting(&["/home/anna/Invoices", "/home/anna/Archive"]);
        assert!(!call.permitted_by(&grants, &Grantee::named("@mail"), noon()));
    }

    /// An argument that is missing, unknown, given twice or invalid never
    /// becomes a call at all — so nothing has to decide what to do with half of
    /// one.
    #[test]
    fn a_call_that_is_not_complete_is_not_a_call() {
        let verb = move_file();
        assert_eq!(
            Call::of(&verb, &[("file", Given::text("/home/anna/x.pdf"))]).unwrap_err(),
            CallError::Missing {
                verb: "move_file".to_owned(),
                argument: "into".to_owned(),
                purpose: MOVING_INTO.key(),
            }
        );
        assert_eq!(
            Call::of(
                &verb,
                &[
                    ("file", Given::text("/home/anna/x.pdf")),
                    ("into", Given::text("/home/anna/Archive")),
                    ("overwrite", Given::text("yes")),
                ],
            )
            .unwrap_err(),
            CallError::NoSuchArgument {
                verb: "move_file".to_owned(),
                argument: "overwrite".to_owned(),
            }
        );
        assert_eq!(
            Call::of(
                &verb,
                &[
                    ("file", Given::text("/home/anna/x.pdf")),
                    ("file", Given::text("/home/anna/y.pdf")),
                    ("into", Given::text("/home/anna/Archive")),
                ],
            )
            .unwrap_err(),
            CallError::SameArgumentTwice {
                argument: "file".to_owned(),
            }
        );
        assert!(matches!(
            Call::of(
                &verb,
                &[
                    ("file", Given::text("../../etc/shadow")),
                    ("into", Given::text("/home/anna/Archive")),
                ],
            ),
            Err(CallError::Argument(_))
        ));
    }

    /// A verb that requires nothing asks nothing of the grants, and that is a
    /// decision its author wrote down rather than an empty list nobody noticed.
    #[test]
    fn a_verb_that_requires_no_grant_asks_nothing() {
        const DISPLAYS: Word = Word::saying(
            "testing.verb.list-displays.purpose",
            "list the displays attached to this machine",
        );
        const DISPLAYS_SENTENCE: Word =
            Word::saying("testing.verb.list-displays.sentence", "list the displays");
        let verb = Verb::checked(
            "list_displays",
            DISPLAYS,
            Effect::Read,
            vec![],
            Requires::nothing_because(
                "a display is not a path, a file or an application, and naming one reaches nothing",
            ),
            DISPLAYS_SENTENCE,
        )
        .unwrap();
        let call = Call::of(&verb, &[]).unwrap();
        assert!(call.asks().is_empty());
        assert_eq!(
            call.sentence(&crate::testing::speaking(&[DISPLAYS, DISPLAYS_SENTENCE]))
                .text(),
            "list the displays"
        );
        assert!(call.filling().is_nothing());
        assert!(call.permitted_by(&Grants::default(), &files(), noon()));
        assert!(!call.waits_for_approval());
    }

    /// **The sentence a person approves is the one they read.** The words are
    /// the translation's, the values are this call's, and the two paths are in
    /// it because a translation that dropped one could not have been loaded.
    ///
    /// This is what item 9g bought: before it, a shell looked the key up and
    /// the approval and the record kept the English the call had rendered.
    #[test]
    fn the_sentence_is_read_in_the_readers_own_language() {
        let strings = translating(
            &THE_WORDS,
            &[(MOVING_SENTENCE, "{file} nach {into} verschieben")],
        );
        let said = moving_march().sentence(&strings);
        assert!(said.is_translated());
        assert!(said.unfilled().is_empty());
        assert_eq!(
            said.text(),
            "/home/anna/Invoices/march.pdf nach /home/anna/Archive verschieben"
        );
    }

    /// A verb nobody has translated says so, rather than looking like a
    /// sentence somebody wrote in the reader's language.
    #[test]
    fn an_untranslated_sentence_says_where_it_came_from() {
        let said = moving_march().sentence(&reading());
        assert!(!said.is_translated());
        assert!(!said.is_a_bug());
        assert_eq!(said.came_from(), &alo_strings::CameFrom::TheSource);
    }

    /// The record keeps what ran and what was refused, so a call has to survive
    /// being written down — **and what it carries is the key, not a rendering.**
    /// A record written from this is rendered where somebody reads it.
    #[test]
    fn a_call_can_be_written_down() {
        let call = moving_march();
        let written = serde_json::to_string(&call).unwrap();
        assert!(written.contains("move_file"), "{written}");
        assert!(written.contains("march.pdf"), "{written}");
        assert!(
            written.contains("testing.verb.move-file.sentence"),
            "{written}"
        );
        assert_eq!(call.sentence_key(), &MOVING_SENTENCE.key());
        assert!(
            !written.contains("move /home/anna/Invoices/march.pdf into"),
            "{written}"
        );
    }
}
