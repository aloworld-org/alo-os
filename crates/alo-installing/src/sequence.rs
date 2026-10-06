//! The installation, from the first line on the screen to the restart.
//!
//! In order, and each step said before it begins:
//!
//! 1. read which disk was chosen, from the kernel command line (`crate::told`);
//! 2. wait for that disk to appear, by its own name (`crate::disk`);
//! 3. ask what it holds, and refuse what no consent could have been about
//!    (`crate::disks`) — the four things a disk taken whole is refused for, or,
//!    on the road that keeps what is there, anything other than the two
//!    partitions the installer on Windows made this road for;
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

use alo_strings::{Filling, Said, Strings, Word};

use crate::disk::{DiskName, PartitionName};
use crate::disks::{Disks, Replacing, Unsuitable};
use crate::ended::{Ended, Refusal, the_disk};
use crate::environment::Environment;
use crate::machine::{BEFORE_RESTARTING, STILL_EVERY, THE_DISK_APPEARS_WITHIN, TheMachine};
use crate::program::Program;
use crate::told::{NotTold, Told};
use crate::verifying::{Verified, Verifying};
use crate::words;
use crate::writing::{Onto, Writing};

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
        Ok((writing, disk)) => {
            // **The disk the person chose, not the one the write names.** On the
            // road that keeps what is already there the write names a partition
            // and no disk at all — `Writing::disk` is `None` there — while what
            // this sequence says, tidies and ends with is still the disk. Asking
            // the write for it would have been asking it the wrong question.
            say(
                machine,
                strings,
                words::INSTALLING,
                &the_disk(disk.as_str()),
            );
            let still = strings.say(&words::STILL_INSTALLING.key(), &Filling::nothing());
            // **On the road that keeps what is there, the root is made and
            // mounted here**, because the writer is handed a root rather than a
            // disk. Each step is checked: a write into a root that was never
            // mounted would install alo OS into this environment's own memory
            // and report success.
            if let Onto::BesideWhatIsThere { root, esp } = writing.onto().clone()
                && !made_the_root(machine, &still, &root, &esp)
            {
                return Ended::NotInstalled(disk);
            }
            let program = Program::Writing(writing);
            match machine.run(&program, &still, STILL_EVERY) {
                Ok(ran) if ran.succeeded => {
                    // Step 6: what the install leaves behind, put right. alo OS
                    // is installed by this line whatever happens next, and
                    // `crate::tidying` says so in every road out of it.
                    let _tidied = crate::tidying::tidy_up(machine, strings, &disk);
                    Ended::Installed(disk)
                }
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

/// Make the root alo OS goes into, and mount it with the EFI partition under it.
///
/// Only on the road that keeps what is already on the disk. Three programs, in
/// this order and no other: the file system is made, the root is mounted, and
/// the EFI partition is mounted **beneath** it, because the writer looks for the
/// loader's home under the root it is given.
///
/// **Every one is checked.** A `mount` that failed and was not noticed leaves
/// the writer installing into an empty directory in this environment's own
/// memory — which succeeds, says so, and leaves a disk with nothing on it. What
/// each program complained of is noted where a technician reads it.
///
/// The sentence is the write's own, because to the person watching this *is*
/// the install going on. Three sentences about machinery they were never shown
/// would be three sentences about something they did not ask for.
fn made_the_root(
    machine: &mut impl TheMachine,
    still: &Said,
    root: &PartitionName,
    esp: &PartitionName,
) -> bool {
    let steps = [
        Program::MakingTheRoot {
            partition: root.clone(),
        },
        Program::Mounting {
            partition: root.clone(),
            at: Writing::THE_ROOT,
        },
        Program::Mounting {
            partition: esp.clone(),
            at: Writing::THE_ESP,
        },
    ];
    for program in steps {
        match machine.run(&program, still, STILL_EVERY) {
            Ok(ran) if ran.succeeded => {}
            Ok(ran) => {
                noted(machine, &program, &ran.complained);
                return false;
            }
            Err(why) => {
                noted(machine, &program, &why.to_string());
                return false;
            }
        }
    }
    true
}

/// Steps 1 to 4, which only read, and the write they lead to.
///
/// Gives back the disk beside the write because the two are no longer the same
/// question: a write can name a partition, and everything after it — what is
/// said, what is tidied, what the environment ends with — is about the disk the
/// person chose.
fn before_writing(
    machine: &mut impl TheMachine,
    strings: &Strings,
    environment: &Environment,
) -> Result<(Writing, DiskName), Refusal> {
    say(
        machine,
        strings,
        words::READING_THE_CHOICE,
        &Filling::nothing(),
    );
    let line = machine
        .command_line()
        .map_err(|_| Refusal::ChoiceNotUnderstood)?;
    let told = match Told::from_the_command_line(&line) {
        Ok(told) => told,
        Err(why) => {
            // **Which word was wrong is kept, and kept off the screen.** The
            // person is told in the vocabulary, which never names a kernel
            // word; whoever helps them afterwards needs to know exactly which
            // one. Three of these can only happen if the installer that staged
            // this line staged it wrong — a half-named road, or a name that is
            // not a partition's — and that is a thing to be able to read
            // afterwards rather than to guess at.
            machine.note(&format!("the command line: {why}"));
            return Err(match why {
                NotTold::NothingChosen => Refusal::NoDiskChosen,
                NotTold::MoreThanOne
                | NotTold::NotADisk(_)
                | NotTold::NoEfiPartition
                | NotTold::NoPartitionToInstallInto
                | NotTold::NotAPartition(_)
                | NotTold::BothRoads => Refusal::ChoiceNotUnderstood,
                NotTold::APartition(named) => Refusal::NotAWholeDisk(named),
            });
        }
    };
    let disk = told.disk();

    say(
        machine,
        strings,
        words::LOOKING_FOR_THE_DISK,
        &the_disk(disk.as_str()),
    );
    let device = machine
        .wait_for(&disk.path(), THE_DISK_APPEARS_WITHIN)
        .ok_or_else(|| Refusal::DiskNotConnected(disk.clone()))?;

    let checking = strings.say(&words::CHECKING_THE_DISK.key(), &the_disk(disk.as_str()));
    machine.say(&checking);
    let listed = machine
        .run(&Program::ListingTheDisks, &checking, STILL_EVERY)
        .ok()
        .filter(|ran| ran.succeeded)
        .and_then(|ran| Disks::read(&ran.printed).ok())
        .ok_or(Refusal::DisksNotRead)?;
    // **The two roads ask the disk different questions, and only one of them
    // each.** A disk taken whole must hold nothing anybody would miss; a disk
    // being kept is *meant* to hold Windows and the installer's own staging
    // area, and what matters there is whether the two partitions named are the
    // two the installer made this road for. Neither check is a weakened version
    // of the other, and a line claiming both roads never reaches here
    // (`NotTold::BothRoads`).
    match told.beside() {
        None => {
            if told.replaces_what_is_there() {
                // Said here, where the disk has been read and nothing has been
                // written yet: the last sentence before the road that has no
                // way back.
                machine.say(&strings.say(
                    &words::REPLACING_WHAT_IS_THERE.key(),
                    &the_disk(disk.as_str()),
                ));
            }
            let replacing = if told.replaces_what_is_there() {
                Replacing::TheSystemOnTheDisk
            } else {
                Replacing::Nothing
            };
            listed
                .may_receive(&device, replacing)
                .map_err(|why| match why {
                    Unsuitable::NotListed => Refusal::DiskNotConnected(disk.clone()),
                    Unsuitable::NotAWholeDisk => Refusal::NotAWholeDisk(disk.as_str().to_owned()),
                    Unsuitable::HoldsThisInstaller => Refusal::HoldsThisInstaller(disk.clone()),
                    Unsuitable::HoldsAnotherSystem => Refusal::HoldsAnotherSystem(disk.clone()),
                    Unsuitable::CannotBeWritten => Refusal::CannotBeWritten(disk.clone()),
                })?;
        }
        Some(beside) => {
            let waited = machine.wait_for(&beside.root().path(), THE_DISK_APPEARS_WITHIN);
            let Some(root) = waited else {
                machine.note(&format!(
                    "the space for alo OS: {} never appeared",
                    beside.root().as_str()
                ));
                return Err(Refusal::NotTheSpaceTheInstallerMade(disk.clone()));
            };
            let waited = machine.wait_for(&beside.efi().path(), THE_DISK_APPEARS_WITHIN);
            let Some(esp) = waited else {
                machine.note(&format!(
                    "the start-up area: {} never appeared",
                    beside.efi().as_str()
                ));
                return Err(Refusal::NotTheSpaceTheInstallerMade(disk.clone()));
            };
            if let Err(why) = listed.may_keep_what_is_there(&device, &root, &esp) {
                machine.note(&format!("keeping what is on {}: {why}", disk.as_str()));
                return Err(Refusal::NotTheSpaceTheInstallerMade(disk.clone()));
            }
        }
    }

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

    // **The road is the one the line named, and nothing here chooses it.** The
    // person chose on the machine they still had, before the restart. This reads
    // what they chose and decides nothing they did not.
    let writing = match told.beside() {
        None => Writing::of(environment.pin(), disk),
        Some(beside) => {
            Writing::beside_what_is_there(environment.pin(), beside.root(), beside.efi())
        }
    };
    Ok((writing, disk.clone()))
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
