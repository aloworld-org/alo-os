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

use crate::checking;
use crate::consent;
use crate::deciding::decide;
use crate::ended::{Ended, Refusal};
use crate::environment::{NotStaged, Released, TheEnvironment};
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

    staging::stage(machine, strings, &offer, chosen, &the_environment)?;

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

/// One sentence, looked up and put in front of the person.
fn say(machine: &mut impl TheMachine, strings: &Strings, word: Word, filling: &Filling) {
    machine.say(&strings.say(&word.key(), filling));
}
