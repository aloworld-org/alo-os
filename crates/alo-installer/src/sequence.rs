//! The installer, from the first line on the screen to the restart.
//!
//! In order, each step said before it begins:
//!
//! 1. ask whether this program has an administrator's rights, and stop if not;
//! 2. hold the environment beside it to the list this release was built with
//!    (`crate::environment`), and stop if it is not genuine;
//! 3. check the computer with reads alone and say everything found
//!    (`crate::checking`, `crate::found`);
//! 4. decide whether to offer, and stop with the first reason not to
//!    (`crate::deciding`) — Secure Boot on is one;
//! 5. say exactly what will happen, and that nothing has changed yet;
//! 6. take the typed consent, which names the disk (`crate::consent`);
//! 7. prepare the computer, putting everything back if any step fails
//!    (`crate::staging`);
//! 8. say it is ready, give the person a moment to read that, and restart.
//!
//! **Nothing is changed before step 7**, and every road out of steps 1–6 is a
//! [`Refusal`] said in its own sentence, each of which says nothing was
//! changed. Step 8's restart is the point after which this program is no longer
//! running, and everything before it is reversible.

use alo_strings::{Filling, Strings, Word};

use crate::asking::{self, Answer};
use crate::checking;
use crate::consent;
use crate::deciding::decide;
use crate::ended::{Ended, Refusal};
use crate::environment::{NotStaged, Released, TheEnvironment};
use crate::fast_startup::FastStartup;
use crate::machine::{BEFORE_RESTARTING, TheMachine};
use crate::program::Program;
use crate::sizes;
use crate::staging;
use crate::words;

/// Check, say, consent, stage and restart — or refuse — on this machine, and
/// say every step on the way.
pub fn install(machine: &mut impl TheMachine, strings: &Strings, released: Released) -> Ended {
    say(machine, strings, words::STARTING, &Filling::nothing());
    let ended = installing(machine, strings, released).unwrap_or_else(|ended| ended);
    match &ended {
        Ended::Staged { .. } => {}
        Ended::Refused(refusal) => {
            let (word, filling) = refusal.said_as();
            say(machine, strings, word, &filling);
        }
        Ended::NotPutBack(remains) => {
            say(machine, strings, words::NOT_PUT_BACK, &Filling::nothing());
            for remaining in remains {
                let (word, filling) = remaining.said_as();
                say(machine, strings, word, &filling);
            }
        }
    }
    ended
}

/// Steps 1 to 8, with every refusal as an early return.
fn installing(
    machine: &mut impl TheMachine,
    strings: &Strings,
    released: Released,
) -> Result<Ended, Ended> {
    if !checking::is_an_administrator(machine) {
        return Err(Ended::Refused(Refusal::NotAnAdministrator));
    }

    say(
        machine,
        strings,
        words::CHECKING_THE_DOWNLOAD,
        &Filling::nothing(),
    );
    let the_environment = TheEnvironment::read(machine, released).map_err(|why| {
        Ended::Refused(match why {
            NotStaged::Incomplete => Refusal::Incomplete,
            NotStaged::NotGenuine => Refusal::NotGenuine,
        })
    })?;
    say(machine, strings, words::GENUINE, &Filling::nothing());

    say(
        machine,
        strings,
        words::CHECKING_THE_COMPUTER,
        &Filling::nothing(),
    );
    let found = checking::check(machine);
    for said in found.said(strings) {
        machine.say(&said);
    }
    let offer = decide(&found).map_err(Ended::Refused)?;

    let area = sizes::needed(sizes::THE_AREA);
    say(
        machine,
        strings,
        words::NOTHING_CHANGED_YET,
        &Filling::nothing(),
    );
    say(
        machine,
        strings,
        words::WILL_SHRINK_WINDOWS,
        &Filling::of("volume", offer.windows.letter.drive()).and("area", area.as_str()),
    );
    say(
        machine,
        strings,
        words::WILL_MAKE_THE_AREA,
        &Filling::of("area", area.as_str()).and("disk", offer.windows_disk.as_str()),
    );
    say(
        machine,
        strings,
        words::WILL_ADD_THE_ENTRY,
        &Filling::nothing(),
    );
    say(machine, strings, words::WILL_RESTART, &Filling::nothing());

    let typed = machine.ask(&strings.say(&words::TYPE_THE_DISKS_NAME.key(), &Filling::nothing()));
    let chosen = consent::chosen(&typed, &offer.disks_for_alo_os).map_err(Ended::Refused)?;

    // Asked after the consent and before anything is changed, and only when
    // Fast Startup is on (ADR 0064 term 9). Either answer goes on with the
    // install: this is a setting of the person's Windows, not a condition of
    // installing alo OS.
    let answer = ask_about_fast_startup(machine, strings, found.fast_startup);

    staging::stage(machine, strings, &offer, chosen, &the_environment, answer)?;

    say(machine, strings, words::RESTARTING, &Filling::nothing());
    machine.pause(BEFORE_RESTARTING);
    let restarted = machine
        .run(&Program::Restarting)
        .is_ok_and(|ran| ran.succeeded);
    if !restarted {
        say(
            machine,
            strings,
            words::RESTART_IT_YOURSELF,
            &Filling::nothing(),
        );
    }
    Ok(Ended::Staged { restarted })
}

/// How many times a question is asked again before it is left alone.
///
/// A question asked for ever is a computer a person cannot get out of, and the
/// answer that changes nothing is a safe one to end at.
const ASKED_AGAIN: usize = 3;

/// The person's answer about Fast Startup, asked only when it is on.
///
/// Anything that is not one of the two answers is asked again, [`ASKED_AGAIN`]
/// times; after that Fast Startup is left on and that is said. Neither answer
/// stops the install, and the value is only ever changed by *turn off*.
fn ask_about_fast_startup(
    machine: &mut impl TheMachine,
    strings: &Strings,
    fast_startup: FastStartup,
) -> Answer {
    if !fast_startup.is_asked_about() {
        return Answer::LeaveOn;
    }
    let answers = Filling::of(
        "off",
        strings
            .say(&words::ANSWER_TURN_OFF.key(), &Filling::nothing())
            .into_text(),
    )
    .and(
        "on",
        strings
            .say(&words::ANSWER_LEAVE_ON.key(), &Filling::nothing())
            .into_text(),
    );
    for _ in 0..ASKED_AGAIN {
        say(
            machine,
            strings,
            words::ASK_FAST_STARTUP,
            &Filling::nothing(),
        );
        let typed = machine.ask(&strings.say(&words::TYPE_ONE_OF_THESE_ANSWERS.key(), &answers));
        if let Some(answer) = asking::answered(&typed, strings) {
            return answer;
        }
    }
    Answer::LeaveOn
}

/// One sentence, looked up and put in front of the person.
fn say(machine: &mut impl TheMachine, strings: &Strings, word: Word, filling: &Filling) {
    machine.say(&strings.say(&word.key(), filling));
}
