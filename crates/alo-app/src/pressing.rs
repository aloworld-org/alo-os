//! **`press <kind> <name>`**: one control, pressed by its own action, through
//! the accessibility tree (`docs/contracts/terminal-commands.md`, section 2).
//!
//! The walk, the lookup and the press are `alo_adapters::press_by_hand`'s —
//! the one press alo's own road makes too — so a person's command is refused
//! for exactly the reasons alo's would be: no such control, more than one of
//! it, too much showing to be sure, greyed out, nothing to press it with, or no
//! window at all. Nothing is found by position and nothing is guessed.

use alo_adapters::{NotPressed, NotWalked, Pressable, TreeFault, Unpressed, press_by_hand};
use alo_record::{Answered, Road};
use alo_strings::{Filling, Strings};

use crate::ended::Ended;
use crate::machine::Machine;
use crate::outcome::Outcome;
use crate::pipe::readable;
use crate::subject::Subject;
use crate::words;

/// A press, checked before anything is asked of the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToPress {
    /// The kind of control.
    pub kind: Pressable,
    /// Its name, as it shows it.
    pub control: String,
}

/// Check a kind and a name as the person typed them.
///
/// # Errors
/// An [`Outcome::NotAsked`]: a kind `press` does not take, or a name a
/// sentence cannot hold.
pub fn to_press(kind: &str, control: &str, strings: &Strings) -> Result<ToPress, Box<Outcome>> {
    let Some(kind) = Pressable::called(kind) else {
        let kinds: Vec<&str> = Pressable::ALL.iter().map(|kind| kind.named()).collect();
        let said = strings.say(
            &words::NO_SUCH_KIND.key(),
            &Filling::of(words::TYPED, kind.to_owned()).and(words::LIST, kinds.join(", ")),
        );
        return Err(Outcome::refusing(Ended::ValueRefused, said));
    };
    if !readable(control) {
        let said = strings.say(&words::UNREADABLE_VALUE.key(), &Filling::nothing());
        return Err(Outcome::refusing(Ended::ValueRefused, said));
    }
    Ok(ToPress {
        kind,
        control: control.to_owned(),
    })
}

/// Press it, once.
pub fn pressed(
    asked: &ToPress,
    subject: &Subject,
    strings: &Strings,
    machine: &mut dyn Machine,
) -> Outcome {
    let within = subject.within.as_deref();
    let mut arguments = vec![
        ("kind".to_owned(), asked.kind.named().to_owned()),
        ("name".to_owned(), asked.control.clone()),
    ];
    if let Some(window) = within {
        arguments.push(("window".to_owned(), window.to_owned()));
    }
    let answered = match machine.tree() {
        Ok(tree) => press_by_hand(tree, subject.whose(), within, asked.kind, &asked.control),
        Err(fault) => Err(Unpressed::NotWalked(NotWalked::Tree(fault))),
    };
    let kind = strings.say(&asked.kind.words().key(), &Filling::nothing());
    let filling = Filling::of(words::APPLICATION, subject.called.clone())
        .and_said(words::KIND, &kind)
        .and(words::CONTROL, asked.control.clone());
    match answered {
        Ok(pressed) if pressed.unanswered() => Outcome::Carried {
            road: Road::Tree,
            arguments,
            answered: Answered::NoAnswer,
            ended: Ended::NotKnown,
            said: strings.say(&words::PRESS_UNANSWERED.key(), &filling),
        },
        Ok(_) => Outcome::Carried {
            road: Road::Tree,
            arguments,
            answered: Answered::Done,
            ended: Ended::Done,
            said: strings.say(&words::PRESSED.key(), &filling),
        },
        Err(why) => {
            let said = why.said(
                subject.said_as(),
                within,
                asked.kind,
                &asked.control,
                strings,
            );
            match why {
                // **The application was asked and said no**: that reached it.
                Unpressed::NotPressed(NotPressed::DidNotPress) => {
                    Outcome::refused_by_it(Road::Tree, arguments, Ended::Refused, said)
                }
                Unpressed::NotWalked(NotWalked::Tree(TreeFault::NotThere)) => {
                    Outcome::not_asked(Ended::NotOffered, said)
                }
                Unpressed::NotPressed(_) | Unpressed::NotWalked(_) => {
                    Outcome::not_asked(Ended::Refused, said)
                }
            }
        }
    }
}
