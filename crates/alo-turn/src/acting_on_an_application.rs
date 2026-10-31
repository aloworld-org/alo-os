//! From *the person approved this* to *the window moved*.
//!
//! The third executor on this machine, after `crate::carrying` for the six file
//! verbs and `crate::installing_an_application` for the seventh. The pattern is
//! the same and so is the reason it exists: `alo-applications` had every step
//! built — the declarations, the typed arguments, the sentence a person reads,
//! and `Reaching`, whose own header says *whatever drives the compositor holds
//! one of these or it holds nothing* — and nothing handed it an authority.
//!
//! # The order, and what is not in it
//!
//! 1. **Is this authority an application act at all?** The verb's own name says
//!    which of the three, and a verb from another family is refused here.
//! 2. **Do the grants permit this application, and is it here?**
//!    `alo_applications::Reaching`, in that order — and *the order is the
//!    security property*, which its header argues: a refusal answering *that is
//!    not installed* about an application nobody granted would tell an agent
//!    which applications a person has, and that is a fingerprint of who they
//!    are.
//! 3. **For `open`, how it starts**, read off its entry
//!    (`crate::how_an_application_starts`): an entry that gives no way to start
//!    is refused by name here, and the record of one that does says which road
//!    — over the bus, or from its entry's own command. The owner's decision of
//!    2026-10-31 (ADR 0108): an approved `open` starts an application exactly
//!    as a Dock press does.
//! 4. **Ask the door**, and turn its one word into the person's own.
//!
//! **There is no indicator line**, and that is not an omission. Law 1 is about
//! egress, and nothing here leaves the machine: a Unix socket to a process
//! running as the same person is not a network boundary. `alo-software`'s
//! executor puts the line up because an installation fetches from somewhere
//! else; this one asks the compositor next door.
//!
//! **There is no boundary either.** ADR 0013's cgroup is around *our* work
//! reaching the person's disk and is made out of resolved paths. There are no
//! paths here. What the compositor does with its own windows is the
//! compositor's, running as the person, and it was doing that before an agent
//! existed.
//!
//! # `arrange_application` cannot arrive here, and is refused rather than
//! assumed away
//!
//! The daemon declares three of the four
//! (`alo_applications::verbs::declare_the_three_with_a_road_into`), so a call
//! for the fourth is never authorised and never reaches this. It is still
//! refused by name rather than falling into an `open`: a list and an executor
//! are two edits and somebody will one day make only one of them.

use alo_acting::{Acting, Answer, NotActed, Request};
use alo_applications::{Reaching, words};
use alo_capability::{Authorised, Grants, Refused};
use alo_record::Entry;
use alo_strings::{Filling, Said, Strings, Word};

use crate::how_an_application_starts::{NoRoad, TheRoad};
use crate::machine::Machine;
use crate::refusing::NotDone;
use crate::the_door_the_compositor_serves::NotAtTheDoor;

/// What an application act did, which is one of three things asked for.
///
/// It carries the act rather than a word, so the record and the wire both say
/// *which* application had *what* asked of it. There is nothing else to
/// carry: a door that answered `acted` answered one word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reached {
    /// What was asked, and of which application.
    acting: Acting,
}

impl Reached {
    /// What was asked, and of which application.
    #[must_use]
    pub const fn acting(&self) -> &Acting {
        &self.acting
    }
}

/// Carry out one approved application act.
///
/// Answers the record's entry and what happened, as the other two executors do:
/// **a refusal is written down exactly as an execution is** (the gate's *every
/// execution and every refusal leaves a record*), so the entry comes back
/// whichever way this went.
///
/// # The refusals, and what each says to a person
///
/// - the grants' own words, for an application nobody granted;
/// - `applications.not-installed`, for one this machine does not have;
/// - `applications.no-window`, `applications.it-did-not` and
///   `applications.could-not-ask`, for the three things the compositor can
///   answer that are not *done*.
///
/// **Every one of them means nothing changed**, which is what the words say.
pub(crate) fn acting_on_an_application(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<Reached, NotDone>) {
    // Answers for as long as the words live rather than for as long as this
    // borrow does, which is what lets the machine be borrowed mutably below and
    // still be asked for a sentence afterwards.
    let strings = machine.strings();
    let at = authorised.at();
    let agent = authorised.under().clone();

    // Step 1. One of the three, by the verb's own name and nothing else — the
    // same rule `crate::carrying` routes by, for its reason: asking the
    // arguments what shape they are is a second opinion about what a call is.
    let Some(acting_for) = which_act(authorised.verb()) else {
        // No application is named in the sentence, because the fault is the
        // verb rather than any application: the gap takes the verb's own name,
        // which is what a person needs to see.
        let refused = Refused::worded_elsewhere(
            authorised.call().clone(),
            could_not_ask_about(strings, authorised.verb()),
        );
        let entry = Entry::refused(&refused, &agent, strings, at);
        return (entry, Err(NotDone::Refused(refused)));
    };

    // Step 2. Granted, and here — in that order, which is the security
    // property `Reaching`'s own header argues.
    let call = authorised.call().clone();
    let reaching = match Reaching::of(
        authorised,
        grants,
        machine.the_applications_installed(),
        strings,
    ) {
        Ok(reaching) => reaching,
        Err(refused) => {
            let entry = Entry::refused(&refused, &agent, strings, at);
            return (entry, Err(NotDone::Refused(refused)));
        }
    };
    let Some(application) = reaching.application("application") else {
        // The verb declares the argument and `Reaching` found every application
        // the call named, so this is a declaration and an executor having
        // drifted apart rather than anything a caller did. Said rather than
        // unwrapped: a library that panics inside the daemon takes the daemon.
        let refused =
            Refused::worded_elsewhere(call, could_not_ask_about(strings, "this application"));
        let entry = Entry::refused(&refused, &agent, strings, at);
        return (entry, Err(NotDone::Refused(refused)));
    };
    let identifier = application.identifier().to_owned();
    let acting = acting_for(application.clone());

    // Step 2½, for `open` alone. **How it starts, read off its entry**, so an
    // entry that gives no way to start is refused by name before anything is
    // asked, and the record of one that does names the road (the owner's
    // decision of 2026-10-31, ADR 0108). `focus` and `close` start nothing.
    let road = if matches!(acting, Acting::Open(_)) {
        let how = machine
            .how_applications_start()
            .map(|it| it.how_it_starts(&identifier));
        match how {
            Some(Ok(road)) => Some(road),
            Some(Err(NoRoad::NothingStartsIt)) => {
                let refused = Refused::worded_elsewhere(
                    call,
                    said_about(strings, words::NOTHING_STARTS_IT, &identifier),
                );
                let entry = Entry::refused(&refused, &agent, strings, at);
                return (entry, Err(NotDone::Refused(refused)));
            }
            Some(Err(NoRoad::NoEntry)) | None => {
                let refused =
                    Refused::worded_elsewhere(call, could_not_ask_about(strings, &identifier));
                let entry = Entry::refused(&refused, &agent, strings, at);
                return (entry, Err(NotDone::Refused(refused)));
            }
        }
    } else {
        None
    };

    // Step 3. Ask the door, and say what happened in the person's own words.
    let asked = match machine.at_the_compositors_door() {
        Some(door) => door.asking(&Request::to(acting.clone())),
        None => Err(NotAtTheDoor::NoCompositor),
    };
    let why = match asked {
        Ok(Answer::Acted) => {
            // The authorisation's own moment and agent, read back off the
            // `Reaching` that holds it — the same entry `crate::carrying`
            // writes for a file verb that ran.
            //
            // **With the road, for an `open`**: one approval, one entry, and
            // the entry says how the application starts.
            let entry = Entry::ran(reaching.authorised(), strings);
            let entry = match road {
                Some(road) => {
                    entry.telling(&[said_about(strings, the_word_for(road), &identifier)])
                }
                None => entry,
            };
            return (entry, Ok(Reached { acting }));
        }
        Ok(Answer::NotActed(why)) => as_the_person_reads_it(why),
        // **A door that is not there and a door that would not answer read the
        // same to a person**, because from their side they are one fact:
        // nothing was changed. Which it was belongs in a log.
        Err(_) => words::COULD_NOT_ASK,
    };
    let refused = Refused::worded_elsewhere(call, said_about(strings, why, &identifier));
    let entry = Entry::refused(&refused, &agent, strings, at);
    (entry, Err(NotDone::Refused(refused)))
}

/// Which of the three acts this verb's name is, and nothing for anything else.
///
/// Returns the constructor rather than a tag, so the act and the application
/// are put together in one place and a verb cannot end up naming one act and
/// carrying another.
fn which_act(verb: &str) -> Option<fn(alo_applications::Application) -> Acting> {
    match verb {
        "open_application" => Some(Acting::Open),
        "focus_application" => Some(Acting::Focus),
        "close_application" => Some(Acting::Close),
        // `arrange_application` lands here, deliberately. See this file's
        // header: the daemon does not declare it, and a refusal by name is
        // cheaper than discovering one day that it fell into `open`.
        _ => None,
    }
}

/// The compositor's one word, as the sentence a person reads.
///
/// `not-installed` cannot arrive — `Reaching` refused it at step 2, before the
/// door was asked — so it maps to the same *could not reach* as a machine that
/// could not ask. That is not a lost distinction: it is one that cannot occur,
/// and `words.rs` says so where the three are declared.
const fn as_the_person_reads_it(why: NotActed) -> alo_strings::Word {
    match why {
        NotActed::NoWindow => words::NO_WINDOW,
        NotActed::ItDidNot => words::IT_DID_NOT,
        NotActed::NotInstalled | NotActed::CouldNotAsk => words::COULD_NOT_ASK,
    }
}

/// What the record says about how an application starts.
const fn the_word_for(road: TheRoad) -> Word {
    match road {
        TheRoad::OverTheBus => words::STARTS_OVER_THE_BUS,
        TheRoad::FromItsCommand => words::STARTS_FROM_ITS_COMMAND,
    }
}

/// One of the three words, about this application.
fn said_about(strings: &Strings, why: Word, application: &str) -> Said {
    strings.say(
        &why.key(),
        &Filling::of("application", application.to_owned()),
    )
}

/// *This machine could not reach {application}*, about whatever is named.
fn could_not_ask_about(strings: &Strings, application: &str) -> Said {
    said_about(strings, words::COULD_NOT_ASK, application)
}

#[cfg(test)]
// Counted rather than guessed: 8 `unwrap`, 6 `panic!` and 1 `expect`, so all
// three are listed and nothing is listed that is not used — an unfulfilled
// expectation is an error in this workspace.
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use alo_acting::{Acting, Answer, NotActed, Request};
    use alo_applications::{Application, Installed};
    use alo_capability::{Given, Grant, Grants, Reach};
    use alo_egress::Indicator;
    use alo_files::OnThisMachine;
    use alo_record::{Asking, Record};
    use alo_software::Bound;

    use crate::Turning;
    use crate::how_an_application_starts::{NoRoad, SaysHowAnApplicationStarts, TheRoad};
    use crate::machine::{Machine, WhatASoftwareVerbNeeds, WhatAnApplicationVerbNeeds};
    use crate::refusing::NotDone;
    use crate::testing::{hour, in_english, noon, offering};
    use crate::the_door_the_compositor_serves::{NotAtTheDoor, TheDoorTheCompositorServes};

    /// The mail client, which every test here acts on.
    const THE_MAIL_CLIENT: &str = "org.gnome.Geary";

    /// A tool nothing here reaches.
    ///
    /// The constructor that offers the three acts also offers
    /// `install_application`, because the two lists are one list and these
    /// tests build a real machine rather than a reduced one. **Nothing in this
    /// file installs anything**, so every method refuses — a fixture that
    /// quietly succeeded would turn an executor reaching the wrong tool into a
    /// passing test.
    struct NoTool;

    impl alo_software::Tool for NoTool {
        fn sources(&self) -> Result<Vec<alo_software::Configured>, alo_software::Failed> {
            Ok(Vec::new())
        }

        fn installed(&self) -> Result<Vec<String>, alo_software::Failed> {
            Ok(Vec::new())
        }

        fn open(&self) -> Result<Vec<String>, alo_software::Failed> {
            Ok(Vec::new())
        }

        fn install(
            &self,
            _: &alo_software::SourceName,
            _: &Application,
        ) -> Result<(), alo_software::Failed> {
            panic!("nothing in this file installs anything")
        }

        fn updates(
            &self,
            _: &alo_software::SourceName,
        ) -> Result<Vec<String>, alo_software::Failed> {
            Ok(Vec::new())
        }

        fn update(&self, _: &Application) -> Result<(), alo_software::Failed> {
            panic!("nothing in this file updates anything")
        }

        fn remove(&self, _: &Application) -> Result<(), alo_software::Failed> {
            panic!("nothing in this file removes anything")
        }
    }

    /// A door that answers whatever it was told to, and remembers what it was
    /// asked.
    ///
    /// **It remembers, because *one approval causes exactly one execution* is
    /// one of the capability guarantees and a door that was asked twice is how
    /// that breaks here.** A double that only answered could not catch it.
    struct ADoor {
        /// What to answer.
        answering: Result<Answer, NotAtTheDoor>,
        /// Everything it was asked, in order.
        ///
        /// Shared rather than owned, so a test can read it **while** the
        /// machine still holds the door — which is what *nothing reached the
        /// compositor before the approval* has to be asserted in the middle of.
        asked: Rc<RefCell<Vec<Request>>>,
    }

    impl ADoor {
        fn answering(answering: Answer) -> Self {
            Self {
                answering: Ok(answering),
                asked: Rc::new(RefCell::new(Vec::new())),
            }
        }

        /// Not answering, because there is no compositor.
        fn with_no_compositor() -> Self {
            Self {
                answering: Err(NotAtTheDoor::NoCompositor),
                asked: Rc::new(RefCell::new(Vec::new())),
            }
        }

        /// What it has been asked, readable while the machine holds the door.
        fn a_tally(&self) -> Rc<RefCell<Vec<Request>>> {
            Rc::clone(&self.asked)
        }
    }

    impl TheDoorTheCompositorServes for ADoor {
        fn asking(&mut self, request: &Request) -> Result<Answer, NotAtTheDoor> {
            self.asked.borrow_mut().push(request.clone());
            self.answering.clone()
        }
    }

    /// A machine that offers the three acts, with this door behind them.
    ///
    /// A closure for `installing_an_application`'s reason: a [`Turning`]
    /// borrows the machine, which borrows the door, the record and the
    /// indicator, so all of them have to live in the caller's frame.
    ///
    /// `installed` is handed in so that *granted but not here* and *here but
    /// not granted* are both arrangeable — which is what the side-channel test
    /// below needs.
    fn on_a_machine_with_a_door<T>(
        door: &mut ADoor,
        installed: Installed,
        granting: bool,
        doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants) -> T,
    ) -> (T, Record) {
        on_a_machine_where_it_starts(
            door,
            installed,
            granting,
            &ItStarts(Ok(TheRoad::FromItsCommand)),
            doing,
        )
    }

    /// Whatever says how the mail client starts, answering one thing.
    struct ItStarts(Result<TheRoad, NoRoad>);

    impl SaysHowAnApplicationStarts for ItStarts {
        fn how_it_starts(&self, _: &str) -> Result<TheRoad, NoRoad> {
            self.0
        }
    }

    /// [`on_a_machine_with_a_door`], with how the application starts said.
    fn on_a_machine_where_it_starts<T>(
        door: &mut ADoor,
        installed: Installed,
        granting: bool,
        starting: &ItStarts,
        doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants) -> T,
    ) -> (T, Record) {
        let strings = in_english();
        let tool = NoTool;
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = crate::testing::NothingIsBounded;
        let folder = crate::testing::a_folder_of_our_own("acting");
        let there = folder.join("a-note.txt");
        std::fs::write(&there, "anything").unwrap();

        let answered = {
            let mut machine = Machine::carrying_out_file_software_and_application_verbs(
                &strings,
                &OnThisMachine,
                WhatASoftwareVerbNeeds {
                    tool: &tool,
                    bound: Bound::Nobodys,
                },
                WhatAnApplicationVerbNeeds {
                    installed,
                    door,
                    starting,
                },
                &mut bounding,
                &mut indicator,
                &mut record,
            )
            .unwrap();
            let mut grants = Grants::default();
            if granting {
                grants.grant(
                    Grant::checked(
                        "@windows",
                        Reach::Application(THE_MAIL_CLIENT.to_owned()),
                        noon(),
                        hour(),
                    )
                    .unwrap(),
                );
            }
            let mut turning = Turning::beginning(
                offering(&there),
                "@windows",
                hour(),
                &mut grants,
                &mut machine,
            )
            .unwrap();
            doing(&mut turning, &mut grants)
        };
        (answered, record)
    }

    /// The mail client, installed.
    fn the_mail_client_is_here() -> Installed {
        Installed::holding([Application::identified(THE_MAIL_CLIENT).unwrap()])
    }

    /// Propose this act on the mail client, approve it, and answer what the
    /// machine did — which is the road a real turn takes and not a shortcut
    /// into the executor.
    fn through_a_whole_turn(
        turning: &mut Turning<'_, '_>,
        grants: &Grants,
        verb: &str,
        tally: &Rc<RefCell<Vec<Request>>>,
    ) -> Result<crate::what_a_verb_did::WhatAVerbDid, NotDone> {
        let id = turning.proposing(
            verb,
            &[("application", Given::text(THE_MAIL_CLIENT))],
            grants,
            hour(),
            noon(),
        )?;
        // **Nothing has reached the compositor yet**, asserted on the road
        // every one of these tests takes rather than once: *reads answer,
        // changes wait*, and all three of these are changes. A proposal that
        // acted would be a change made without an approval, which is the
        // thing the whole model exists to prevent.
        assert!(
            tally.borrow().is_empty(),
            "the compositor was reached before anybody approved it"
        );
        turning.approving(id, grants, noon())
    }

    /// **The whole road: proposed, approved, asked, done** — and the door was
    /// asked exactly once.
    #[test]
    fn an_approved_act_reaches_the_door_once_and_says_what_it_did() {
        let mut door = ADoor::answering(Answer::Acted);
        let tally = door.a_tally();
        let (did, record) = on_a_machine_with_a_door(
            &mut door,
            the_mail_client_is_here(),
            true,
            |turning, grants| through_a_whole_turn(turning, grants, "focus_application", &tally),
        );

        let did = did.unwrap();
        let reached = did.reached().expect("an application was reached");
        assert_eq!(
            reached.acting(),
            &Acting::Focus(Application::identified(THE_MAIL_CLIENT).unwrap())
        );
        // **One approval, one execution.** A door asked twice is how that
        // guarantee breaks on this road.
        assert_eq!(door.asked.borrow().len(), 1);
        assert_eq!(
            door.asked.borrow().first().map(Request::written),
            Some(format!("focus {THE_MAIL_CLIENT}"))
        );
        // And it is written down.
        assert_eq!(record.answering(&Asking::anything()).count(), 1);
    }

    /// **Each of the three acts crosses as itself**, so the verb's name and the
    /// act on the wire cannot drift apart.
    #[test]
    fn each_of_the_three_verbs_crosses_as_its_own_act() {
        for (verb, expected) in [
            ("open_application", "open"),
            ("focus_application", "focus"),
            ("close_application", "close"),
        ] {
            let mut door = ADoor::answering(Answer::Acted);
            let tally = door.a_tally();
            let (did, _) = on_a_machine_with_a_door(
                &mut door,
                the_mail_client_is_here(),
                true,
                |turning, grants| through_a_whole_turn(turning, grants, verb, &tally),
            );
            assert!(did.is_ok(), "{verb} did not run: {did:?}");
            assert_eq!(
                door.asked.borrow().first().map(Request::written),
                Some(format!("{expected} {THE_MAIL_CLIENT}")),
                "{verb} crossed as something else"
            );
        }
    }

    /// **Each refusal the compositor can make becomes the person's own
    /// sentence, naming the application** — and every one of them is written
    /// down, which the gate asks for by name.
    #[test]
    fn each_refusal_becomes_a_sentence_about_this_application_and_is_recorded() {
        for (why, expected) in [
            (
                NotActed::NoWindow,
                "org.gnome.Geary has no window open here",
            ),
            (NotActed::ItDidNot, "org.gnome.Geary was asked and has not"),
            (
                NotActed::CouldNotAsk,
                "this machine could not reach org.gnome.Geary",
            ),
        ] {
            let mut door = ADoor::answering(Answer::NotActed(why));
            let tally = door.a_tally();
            let (did, record) = on_a_machine_with_a_door(
                &mut door,
                the_mail_client_is_here(),
                true,
                |turning, grants| {
                    through_a_whole_turn(turning, grants, "close_application", &tally)
                },
            );

            let Err(NotDone::Refused(refused)) = did else {
                panic!("{why:?} was not a refusal: {did:?}");
            };
            let said = refused.said(&in_english()).text().to_owned();
            assert!(said.contains(expected), "{why:?} said {said:?}");
            assert_eq!(
                record.answering(&Asking::anything()).count(),
                1,
                "{why:?} left no record"
            );
        }
    }

    /// **No compositor is an answer and not a fault**, and it reads to a person
    /// exactly as a compositor that could not be asked — because from their
    /// side they are one fact: nothing was changed.
    ///
    /// A machine at the sign-in screen is in exactly this state.
    #[test]
    fn a_machine_with_no_compositor_says_it_could_not_reach_it() {
        let mut door = ADoor::with_no_compositor();
        let tally = door.a_tally();
        let (did, record) = on_a_machine_with_a_door(
            &mut door,
            the_mail_client_is_here(),
            true,
            |turning, grants| through_a_whole_turn(turning, grants, "open_application", &tally),
        );

        let Err(NotDone::Refused(refused)) = did else {
            panic!("no compositor was not a refusal: {did:?}");
        };
        assert!(
            refused
                .said(&in_english())
                .text()
                .contains("this machine could not reach org.gnome.Geary"),
            "{:?}",
            refused.said(&in_english()).text()
        );
        assert_eq!(record.answering(&Asking::anything()).count(), 1);
    }

    /// **An application nobody granted is refused before a proposal is ever
    /// made, and the refusal says nothing about what is installed.**
    ///
    /// # This test was a tautology until a mutation caught it
    ///
    /// It was written to guard `alo_applications::Reaching`'s security
    /// property — *the order is the security property*, grants before the
    /// installed list, because a refusal answering *that is not installed*
    /// about something ungranted tells an agent which applications a person
    /// has, and that is a fingerprint of who they are and who they work for.
    ///
    /// The first version collapsed both arrangements to the string
    /// `"never asked"` and compared them, so it compared one literal with
    /// itself. **Swapping `Reaching`'s two questions left all seven tests in
    /// this file green**, which is how it was found.
    ///
    /// What is really true is better than what it was asserting: on this road
    /// an ungranted application never reaches `Reaching` at all. The grants
    /// refuse it at **propose time** — `NotDone::NeverAsked(NotGranted(..))` —
    /// so no sentence is ever put to a person and nothing is ever approved.
    /// `Reaching`'s own order is held by `alo-applications`' tests, where it
    /// belongs; this one holds that the road in front of it does not need it.
    ///
    /// So it asserts three things a mutation can reach: the refusal is the
    /// same whichever way, it **names no installation fact**, and the
    /// compositor is never touched.
    #[test]
    fn an_ungranted_application_is_refused_before_anything_is_proposed() {
        let mut said = Vec::new();
        for installed in [the_mail_client_is_here(), Installed::nothing()] {
            let mut door = ADoor::answering(Answer::Acted);
            let tally = door.a_tally();
            let (did, record) =
                on_a_machine_with_a_door(&mut door, installed, false, |turning, grants| {
                    // Not `through_a_whole_turn`: there is nothing to approve,
                    // because the proposal itself is refused.
                    turning.proposing(
                        "focus_application",
                        &[("application", Given::text(THE_MAIL_CLIENT))],
                        grants,
                        hour(),
                        noon(),
                    )
                });
            assert!(
                tally.borrow().is_empty(),
                "an ungranted application reached the compositor"
            );
            let Err(NotDone::NeverAsked(why)) = did else {
                panic!("an ungranted application was put to somebody: {did:?}");
            };
            // **And the refusal is written down**, which the gate asks for by
            // name: *every execution and every refusal leaves a record*.
            assert_eq!(
                record.answering(&Asking::anything()).count(),
                1,
                "a refused proposal left no record"
            );
            said.push(format!("{why:?}"));
        }

        let [here, not_here] = said.as_slice() else {
            unreachable!("two arrangements, two refusals")
        };
        assert_eq!(
            here, not_here,
            "the refusal differs depending on whether the application is installed"
        );
        // **The direct test of the side channel**, rather than only the
        // comparison: a refusal that mentioned installation at all would hand
        // over the fact even if it happened to word both cases alike.
        for way in [here, not_here] {
            assert!(
                !way.to_lowercase().contains("install"),
                "the refusal mentions what is installed: {way}"
            );
        }
    }

    /// **The fourth verb is not one of the three**, said as a test because a
    /// list and an executor are two edits and somebody will one day make only
    /// one of them.
    ///
    /// It cannot be proposed at all on a machine built by the constructor that
    /// offers these — `declare_the_three_with_a_road_into` leaves it out — so
    /// this holds the executor's own half of that agreement.
    #[test]
    fn the_fourth_verb_is_not_one_of_the_three_acts() {
        assert!(super::which_act("arrange_application").is_none());
        for verb in ["open_application", "focus_application", "close_application"] {
            assert!(super::which_act(verb).is_some(), "{verb}");
        }
        // And nothing from another family falls into one.
        for verb in ["rename_file", "install_application", "open"] {
            assert!(super::which_act(verb).is_none(), "{verb}");
        }
    }

    /// And the machine really does not offer it, which is the other half.
    ///
    /// Read off a real [`Machine`] rather than through a turn, because the list
    /// is the machine's: `Machine::verbs` is what `alo-instructing` shows a
    /// model as *these are the only verbs there are*.
    #[test]
    fn a_machine_that_offers_the_three_does_not_offer_the_fourth() {
        let strings = in_english();
        let tool = NoTool;
        let mut door = ADoor::answering(Answer::Acted);
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = crate::testing::NothingIsBounded;
        let machine = Machine::carrying_out_file_software_and_application_verbs(
            &strings,
            &OnThisMachine,
            WhatASoftwareVerbNeeds {
                tool: &tool,
                bound: Bound::Nobodys,
            },
            WhatAnApplicationVerbNeeds {
                installed: the_mail_client_is_here(),
                door: &mut door,
                starting: &ItStarts(Ok(TheRoad::OverTheBus)),
            },
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();

        let names: Vec<&str> = machine.verbs().all().map(|verb| verb.name()).collect();
        for offered in ["open_application", "focus_application", "close_application"] {
            assert!(
                names.contains(&offered),
                "{offered} is not offered: {names:?}"
            );
        }
        assert!(
            !names.contains(&"arrange_application"),
            "a verb with no road is offered to an agent: {names:?}"
        );
        // And the other two families are still all there, so adding a third
        // did not take anything away.
        assert!(names.contains(&"rename_file"), "{names:?}");
        assert!(names.contains(&"install_application"), "{names:?}");
    }

    /// The sentences an entry was written with.
    fn what_it_told(record: &Record) -> Vec<String> {
        record
            .answering(&Asking::anything())
            .flat_map(|entry| entry.told().iter().map(ToString::to_string))
            .collect()
    }

    /// **An approved `open` of an entry that is not D-Bus-activatable reaches
    /// the door once, and the one entry says it starts from its own command** —
    /// the owner's decision of 2026-10-31, that alo's approved open starts an
    /// application exactly as a Dock press does.
    #[test]
    fn an_approved_open_of_a_command_entry_is_asked_once_and_recorded_with_its_road() {
        let mut door = ADoor::answering(Answer::Acted);
        let tally = door.a_tally();
        let (did, record) = on_a_machine_where_it_starts(
            &mut door,
            the_mail_client_is_here(),
            true,
            &ItStarts(Ok(TheRoad::FromItsCommand)),
            |turning, grants| through_a_whole_turn(turning, grants, "open_application", &tally),
        );
        assert!(did.is_ok(), "{did:?}");
        assert_eq!(
            door.asked
                .borrow()
                .iter()
                .map(Request::written)
                .collect::<Vec<_>>(),
            [format!("open {THE_MAIL_CLIENT}")],
            "one approval, one ask"
        );
        assert_eq!(record.answering(&Asking::anything()).count(), 1);
        assert_eq!(
            what_it_told(&record),
            [format!(
                "{THE_MAIL_CLIENT} starts from the command its own entry names, never from one \
                 alo wrote"
            )]
        );
    }

    /// The other road, named as itself.
    #[test]
    fn an_approved_open_of_a_bus_entry_is_recorded_as_over_the_bus() {
        let mut door = ADoor::answering(Answer::Acted);
        let tally = door.a_tally();
        let (did, record) = on_a_machine_where_it_starts(
            &mut door,
            the_mail_client_is_here(),
            true,
            &ItStarts(Ok(TheRoad::OverTheBus)),
            |turning, grants| through_a_whole_turn(turning, grants, "open_application", &tally),
        );
        assert!(did.is_ok(), "{did:?}");
        assert_eq!(
            what_it_told(&record),
            [format!(
                "{THE_MAIL_CLIENT} starts by being asked over the session bus, as its entry says"
            )]
        );
    }

    /// **An entry with no way to start is refused by name, the door is never
    /// asked, and the refusal is written down.**
    #[test]
    fn an_open_with_nothing_to_start_it_is_refused_by_name_and_recorded() {
        for (no, expected) in [
            (
                NoRoad::NothingStartsIt,
                "org.gnome.Geary's entry gives this machine no way to start it",
            ),
            (
                NoRoad::NoEntry,
                "this machine could not reach org.gnome.Geary",
            ),
        ] {
            let mut door = ADoor::answering(Answer::Acted);
            let tally = door.a_tally();
            let (did, record) = on_a_machine_where_it_starts(
                &mut door,
                the_mail_client_is_here(),
                true,
                &ItStarts(Err(no)),
                |turning, grants| through_a_whole_turn(turning, grants, "open_application", &tally),
            );
            let Err(NotDone::Refused(refused)) = did else {
                panic!("{no:?} was not refused: {did:?}");
            };
            let said = refused.said(&in_english()).text().to_owned();
            assert!(said.contains(expected), "{no:?} said {said:?}");
            assert!(
                door.asked.borrow().is_empty(),
                "{no:?}: the door was asked to start something nothing starts"
            );
            assert_eq!(
                record.answering(&Asking::anything()).count(),
                1,
                "{no:?} left no record"
            );
        }
    }

    /// **An ungranted `open` starts nothing and is written down** — the road
    /// does not change who may ask.
    #[test]
    fn an_ungranted_open_starts_nothing_and_is_recorded() {
        let mut door = ADoor::answering(Answer::Acted);
        let tally = door.a_tally();
        let (did, record) = on_a_machine_where_it_starts(
            &mut door,
            the_mail_client_is_here(),
            false,
            &ItStarts(Ok(TheRoad::FromItsCommand)),
            |turning, grants| {
                turning.proposing(
                    "open_application",
                    &[("application", Given::text(THE_MAIL_CLIENT))],
                    grants,
                    hour(),
                    noon(),
                )
            },
        );
        assert!(matches!(did, Err(NotDone::NeverAsked(_))), "{did:?}");
        assert!(
            tally.borrow().is_empty(),
            "an ungranted open reached the door"
        );
        assert_eq!(record.answering(&Asking::anything()).count(), 1);
    }

    /// **An `open` the person declined starts nothing, and the refusal is
    /// written down**: the door is untouched until the one approval, and a
    /// proposal is not one.
    #[test]
    fn an_unapproved_open_starts_nothing_and_is_recorded() {
        let mut door = ADoor::answering(Answer::Acted);
        let tally = door.a_tally();
        let (declined, record) = on_a_machine_where_it_starts(
            &mut door,
            the_mail_client_is_here(),
            true,
            &ItStarts(Ok(TheRoad::FromItsCommand)),
            |turning, grants| {
                let id = turning.proposing(
                    "open_application",
                    &[("application", Given::text(THE_MAIL_CLIENT))],
                    grants,
                    hour(),
                    noon(),
                )?;
                turning.declining(id, noon())
            },
        );
        assert!(declined.is_ok(), "{declined:?}");
        assert!(
            tally.borrow().is_empty(),
            "a proposal nobody approved reached the door"
        );
        assert_eq!(
            record.answering(&Asking::anything()).count(),
            1,
            "a declined open left no record"
        );
    }
}
