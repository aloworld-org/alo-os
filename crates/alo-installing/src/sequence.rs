//! The installation, from the first line on the screen to the restart.
//!
//! In order, and each step said before it begins:
//!
//! 1. read which disk was chosen, from the kernel command line (`crate::told`);
//! 2. wait for that disk to appear, by its own name (`crate::disk`);
//! 3. ask what it holds, and refuse the four things no consent could have been
//!    about (`crate::disks`);
//! 4. wait for a wired connection, then check the pinned release is signed by
//!    the pinned key, over the network,
//!    and read the answer back (`crate::verifying`);
//! 5. write it onto the disk (`crate::writing`), saying every minute that it is
//!    still going;
//! 6. say it is installed, give the person a moment to read that, and restart.
//!
//! **Nothing is written before step 5**, and every road out of steps 1–4 is a
//! [`Refusal`] said in its own sentence followed by *you can turn this computer
//! off or restart it now*. The environment does not restart after a refusal:
//! the person has to be able to read why.
//!
//! **The disk named is the only disk touched.** Steps 2–4 read; step 5 writes the
//! one path step 1 produced. There is no step that looks for a disk to use.
//!
//! **Why a program failed is kept, and kept off the screen.** When the check or
//! the write fails, what it complained of is noted line by line on the machine's
//! log and its serial lines (`TheMachine::note`), so the reason exists
//! somewhere a person helping, or a test, can read it. The person watching is
//! told in the vocabulary, which never names the machinery.

use alo_strings::{Filling, Strings, Word};

use crate::disks::{Disks, Unsuitable};
use crate::ended::{Ended, Refusal, the_disk};
use crate::environment::Environment;
use crate::machine::{BEFORE_RESTARTING, STILL_EVERY, THE_DISK_APPEARS_WITHIN, TheMachine};
use crate::program::Program;
use crate::told::{NotTold, Told};
use crate::verifying::{Verified, Verifying};
use crate::words;
use crate::writing::Writing;

/// Install, or refuse, on this machine, with this environment — and say every
/// step on the way.
pub fn install(
    machine: &mut impl TheMachine,
    strings: &Strings,
    environment: Result<&Environment, Refusal>,
) -> Ended {
    say(machine, strings, words::STARTING, &Filling::nothing());
    let ended = match environment {
        Ok(environment) => installing(machine, strings, environment),
        Err(refusal) => Ended::Refused(refusal),
    };

    let (word, filling) = ended.said_as();
    say(machine, strings, word, &filling);
    match &ended {
        Ended::Installed(_) => {
            machine.pause(BEFORE_RESTARTING);
            // A restart that does not happen leaves a machine that says it is
            // installed and is: there is nothing further to say about it.
            let _restarting = machine.run(
                &Program::Restarting,
                &strings.say(&words::INSTALLED.key(), &Filling::nothing()),
                STILL_EVERY,
            );
        }
        Ended::Refused(_) | Ended::NotInstalled(_) => {
            say(
                machine,
                strings,
                words::RESTART_WHEN_READY,
                &Filling::nothing(),
            );
        }
    }
    ended
}

/// Steps 1 to 5.
fn installing(
    machine: &mut impl TheMachine,
    strings: &Strings,
    environment: &Environment,
) -> Ended {
    match before_writing(machine, strings, environment) {
        Err(refusal) => Ended::Refused(refusal),
        Ok(writing) => {
            say(
                machine,
                strings,
                words::INSTALLING,
                &the_disk(writing.disk().as_str()),
            );
            let still = strings.say(&words::STILL_INSTALLING.key(), &Filling::nothing());
            let disk = writing.disk().clone();
            let program = Program::Writing(writing);
            match machine.run(&program, &still, STILL_EVERY) {
                Ok(ran) if ran.succeeded => Ended::Installed(disk),
                Ok(ran) => {
                    noted(machine, &program, &ran.complained);
                    Ended::NotInstalled(disk)
                }
                Err(why) => {
                    noted(machine, &program, &why.to_string());
                    Ended::NotInstalled(disk)
                }
            }
        }
    }
}

/// Steps 1 to 4, which only read, and the write they lead to.
fn before_writing(
    machine: &mut impl TheMachine,
    strings: &Strings,
    environment: &Environment,
) -> Result<Writing, Refusal> {
    say(
        machine,
        strings,
        words::READING_THE_CHOICE,
        &Filling::nothing(),
    );
    let line = machine
        .command_line()
        .map_err(|_| Refusal::ChoiceNotUnderstood)?;
    let told = Told::from_the_command_line(&line).map_err(|why| match why {
        NotTold::NothingChosen => Refusal::NoDiskChosen,
        NotTold::MoreThanOne | NotTold::NotADisk(_) => Refusal::ChoiceNotUnderstood,
        NotTold::APartition(named) => Refusal::NotAWholeDisk(named),
    })?;
    let disk = told.disk();

    say(
        machine,
        strings,
        words::LOOKING_FOR_THE_DISK,
        &the_disk(disk.as_str()),
    );
    let device = machine
        .wait_for(disk, THE_DISK_APPEARS_WITHIN)
        .ok_or_else(|| Refusal::DiskNotConnected(disk.clone()))?;

    let checking = strings.say(&words::CHECKING_THE_DISK.key(), &the_disk(disk.as_str()));
    machine.say(&checking);
    let listed = machine
        .run(&Program::ListingTheDisks, &checking, STILL_EVERY)
        .ok()
        .filter(|ran| ran.succeeded)
        .and_then(|ran| Disks::read(&ran.printed).ok())
        .ok_or(Refusal::DisksNotRead)?;
    listed.may_receive(&device).map_err(|why| match why {
        Unsuitable::NotListed => Refusal::DiskNotConnected(disk.clone()),
        Unsuitable::NotAWholeDisk => Refusal::NotAWholeDisk(disk.as_str().to_owned()),
        Unsuitable::HoldsThisInstaller => Refusal::HoldsThisInstaller(disk.clone()),
        Unsuitable::HoldsAnotherSystem => Refusal::HoldsAnotherSystem(disk.clone()),
        Unsuitable::CannotBeWritten => Refusal::CannotBeWritten(disk.clone()),
    })?;

    let connecting = strings.say(&words::CONNECTING.key(), &Filling::nothing());
    machine.say(&connecting);
    let connected = machine
        .run(&Program::WaitingForTheNetwork, &connecting, STILL_EVERY)
        .map_err(|_| Refusal::Damaged)?;
    if !connected.succeeded {
        return Err(Refusal::NotReachable);
    }

    let asking = strings.say(&words::CHECKING_IT_IS_GENUINE.key(), &Filling::nothing());
    machine.say(&asking);
    let verifying = Verifying::of(environment.pin(), environment.key());
    let ran = machine
        .run(&Program::Verifying(verifying.clone()), &asking, STILL_EVERY)
        .map_err(|_| Refusal::Damaged)?;
    let answer = verifying.answer(&ran);
    if answer != Verified::Genuine && !ran.succeeded {
        noted(machine, &Program::Verifying(verifying), &ran.complained);
    }
    match answer {
        Verified::Genuine => say(machine, strings, words::GENUINE, &Filling::nothing()),
        Verified::NotGenuine => return Err(Refusal::NotGenuine),
        Verified::NotReachable => return Err(Refusal::NotReachable),
    }

    Ok(Writing::of(environment.pin(), disk))
}

/// What a program that failed complained of, a line at a time, where a
/// technician or a test reads it and never where the person does.
///
/// Every line is prefixed with the program's path, so a serial line that holds
/// the environment's own sentences as well says which of it is the machinery's.
/// A failure that complained of nothing is noted as that, rather than left as a
/// silence nobody can tell from a lost line.
fn noted(machine: &mut impl TheMachine, program: &Program, complained: &str) {
    let mut lines = complained
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .peekable();
    if lines.peek().is_none() {
        machine.note(&format!("{}: failed, and said nothing", program.path()));
    }
    for line in lines {
        machine.note(&format!("{}: {line}", program.path()));
    }
}

/// One sentence, looked up and put in front of the person.
fn say(machine: &mut impl TheMachine, strings: &Strings, word: Word, filling: &Filling) {
    machine.say(&strings.say(&word.key(), filling));
}
