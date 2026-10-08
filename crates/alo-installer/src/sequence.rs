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

use alo_installing::Replacing;
use alo_strings::{Filling, Strings, Word};

use crate::a_test_build;
use crate::asking::{self, ASKED_AGAIN, Answer};
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
use crate::the_replacing_road::{Road, walk, which_road};
use crate::where_alo_os_goes::{self, Place, Places};
use crate::words;

/// Check, say, consent, stage and restart — or refuse — on this machine, and
/// say every step on the way.
pub fn install(machine: &mut impl TheMachine, strings: &Strings, released: Released) -> Ended {
    // **Before anything else, including the programme's own name.** ADR 0096
    // control five: a test build says so on its first line, because the person
    // it protects kept a candidate and is running it later, with no page to
    // consult and no network. A release says nothing, which is every release,
    // every local build and every test binary.
    if let Some((word, filling)) = a_test_build::this_build().said_as() {
        say(machine, strings, word, &filling);
    }
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

    say(
        machine,
        strings,
        words::NOTHING_CHANGED_YET,
        &Filling::nothing(),
    );

    // **The first question, which is the owner's own: two systems, or one.**
    //
    // Asked before anything is described, because what the installer will do
    // depends on the answer. It used to be the other way about - say what will
    // happen, then ask which road - which worked while both roads did the same
    // thing to Windows. They no longer do.
    //
    // Asked only where replacing can be walked: the offer carries a target for
    // it or it does not, and a road whose disk cannot be named is not a road to
    // offer. **Keeping Windows is what every unrecognised answer means**, so
    // nothing a person types by accident reaches the road that erases a disk.
    if let Some(the_windows_disk) = offer.the_windows_disk.clone()
        && which_road(machine, strings) == Road::ReplaceWindows
    {
        let staged = walk(
            machine,
            strings,
            &found,
            &offer,
            &the_windows_disk,
            |machine, replacing| {
                staging::stage(
                    machine,
                    strings,
                    &offer,
                    &the_windows_disk,
                    &the_environment,
                    Answer::LeaveOn,
                    replacing,
                )
            },
        )
        .map_err(Ended::Refused)?;
        staged?;
        return restart(machine, strings);
    }

    // **The second question, and only where this computer has both places.**
    //
    // One disk and no empty one: alo OS goes beside Windows, because that is
    // the only place it can go. An empty disk and no same-disk road: that disk.
    // Neither is a choice, and a question with one answer is an invitation to
    // type something that will be refused.
    //
    // `Places::None` is refused by `deciding::decide` before this is reached -
    // it returns `NoDiskForAloOs` when nothing can hold alo OS - and this says
    // so again rather than carrying on with no disk, because *refused
    // elsewhere* is a fact about another function that can stop being true.
    let (place, candidates) = match where_alo_os_goes::places(&offer) {
        Places::Both { same, other } => {
            let named = other.first().map_or("", |disk| disk.shown.as_str());
            match where_alo_os_goes::where_alo_os_goes(machine, strings, named) {
                Place::TheSameDisk => (Place::TheSameDisk, vec![same.clone()]),
                Place::TheOtherDisk => (
                    Place::TheOtherDisk,
                    other.into_iter().cloned().collect::<Vec<_>>(),
                ),
            }
        }
        Places::OnlyTheSameDisk(same) => (Place::TheSameDisk, vec![same.clone()]),
        Places::OnlyOtherDisks(other) => (
            Place::TheOtherDisk,
            other.into_iter().cloned().collect::<Vec<_>>(),
        ),
        Places::None => return Err(Ended::Refused(Refusal::NoDiskForAloOs)),
    };

    // **Now say what this road does**, with this road's numbers.
    //
    // What Windows gives up is read from the shrink that will happen. On the
    // same-disk road that is the area and alo OS's space together; on the other
    // road it is the area alone. The installer's own area is the same gibibyte
    // either way - and the sentence it fills says *an area for the installer*,
    // so filling it with the larger figure would tell a person the installer
    // wanted tens of gigabytes for itself.
    //
    // `words::WILL_GIVE_ALO_OS` says what the rest is for, and its own
    // documentation is the reason these are separate numbers: the sentences add
    // up, and a person who adds them should get the number they were told.
    let the_area = sizes::taken(sizes::THE_AREA);
    let beside = candidates.first().and_then(|disk| disk.beside);
    let gives_up = sizes::taken(
        offer
            .windows
            .size
            .saturating_sub(beside.map_or(offer.shrink.to, |beside| beside.to)),
    );
    say(
        machine,
        strings,
        words::WILL_SHRINK_WINDOWS,
        &Filling::of("volume", offer.windows.letter.drive()).and("area", gives_up.as_str()),
    );
    say(
        machine,
        strings,
        words::WILL_MAKE_THE_AREA,
        &Filling::of("area", the_area.as_str()).and("disk", offer.windows_disk.as_str()),
    );
    match (place, beside) {
        (Place::TheSameDisk, Some(beside)) => {
            say(
                machine,
                strings,
                words::WILL_GIVE_ALO_OS,
                &Filling::of("area", sizes::taken(beside.alo_os))
                    .and("disk", offer.windows_disk.as_str()),
            );
            // **The sentence the retired question could not contain.** Windows
            // gives this space up for as long as alo OS is there, by design and
            // not by failure, and it stops where the truth stops: it does not
            // say that removing alo OS later gives it back, because today
            // `crate::removing` refuses this road (the installer plan's task 4).
            say(
                machine,
                strings,
                words::WINDOWS_DOES_NOT_GET_IT_BACK,
                &Filling::nothing(),
            );
        }
        // Windows lends the area and gets it back when the installer has
        // finished, whether it finished or not.
        (Place::TheOtherDisk, _) | (Place::TheSameDisk, None) => say(
            machine,
            strings,
            words::WINDOWS_LENDS_THE_AREA,
            &Filling::of("area", the_area.as_str()),
        ),
    }
    say(
        machine,
        strings,
        words::WILL_ADD_THE_ENTRY,
        &Filling::nothing(),
    );
    // **Two restarts, and saying the wrong one is the worst mistake here.**
    // `WILL_RESTART` says the installer *replaces everything on the disk you
    // name below*, which is true of a disk alo OS takes whole and false of a
    // disk Windows is still on. Nothing would catch it, because each is correct
    // on its own road.
    say(
        machine,
        strings,
        match place {
            Place::TheSameDisk => words::WILL_RESTART_KEEPING_WINDOWS,
            Place::TheOtherDisk => words::WILL_RESTART,
        },
        &Filling::nothing(),
    );

    // The consent, over the disks this road offers and no others. A person who
    // chose the empty disk cannot reach the Windows disk by typing its name,
    // and the other way about.
    let typed = machine.ask(&strings.say(&words::TYPE_THE_DISKS_NAME.key(), &Filling::nothing()));
    let chosen = consent::chosen(&typed, &candidates).map_err(Ended::Refused)?;

    // Asked after the consent and before anything is changed, and only when
    // Fast Startup is on (ADR 0064 term 9). Either answer goes on with the
    // install: this is a setting of the person's Windows, not a condition of
    // installing alo OS.
    let answer = ask_about_fast_startup(machine, strings, found.fast_startup);

    staging::stage(
        machine,
        strings,
        &offer,
        chosen,
        &the_environment,
        answer,
        Replacing::Nothing,
    )?;

    restart(machine, strings)
}

/// Say it is ready, give the person a moment to read that, and restart.
///
/// Both roads end here, and the same way: the restart is step 8 either way, and a
/// machine that will not restart itself says so rather than looking finished.
fn restart(machine: &mut impl TheMachine, strings: &Strings) -> Result<Ended, Ended> {
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
pub(crate) fn say(machine: &mut impl TheMachine, strings: &Strings, word: Word, filling: &Filling) {
    machine.say(&strings.say(&word.key(), filling));
}
