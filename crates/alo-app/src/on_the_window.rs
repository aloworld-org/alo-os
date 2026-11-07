//! **`open`, `focus` and `close`**: what every installed application with a
//! desktop entry offers, carried out by the desktop through its door
//! (`docs/contracts/application-door.md`).
//!
//! `acted` means *asked*, never *done* — the door's own promise. So *Asked Text
//! Editor to close* is what a person is told, because an application asked to
//! close may ask them about unsaved work first, and that answer is theirs.

use alo_acting::{Acting, Answer, NotActed, Request};
use alo_applications::Application;
use alo_record::{Answered, Road};
use alo_strings::{Filling, Strings};

use crate::door::Unanswered;
use crate::ended::Ended;
use crate::machine::Machine;
use crate::outcome::Outcome;
use crate::typed::OnTheWindow;
use crate::words;

/// Ask the desktop to do this to the application's windows, once.
pub fn on_the_window(
    act: OnTheWindow,
    application: &Application,
    called: &str,
    strings: &Strings,
    machine: &mut dyn Machine,
) -> Outcome {
    let acting = match act {
        OnTheWindow::Open => Acting::Open(application.clone()),
        OnTheWindow::Focus => Acting::Focus(application.clone()),
        OnTheWindow::Close => Acting::Close(application.clone()),
    };
    let answered = machine.door().act(&Request::to(acting));
    let filling = Filling::of(words::APPLICATION, called.to_owned());
    let say = |word: &words::Word| strings.say(&word.key(), &filling);
    match answered {
        Ok(Answer::Acted) => Outcome::Carried {
            road: Road::Window,
            arguments: Vec::new(),
            answered: Answered::Done,
            ended: Ended::Done,
            said: say(match act {
                OnTheWindow::Open => &words::ASKED_TO_OPEN,
                OnTheWindow::Focus => &words::ASKED_TO_FOCUS,
                OnTheWindow::Close => &words::ASKED_TO_CLOSE,
            }),
        },
        Ok(Answer::NotActed(NotActed::ItDidNot)) => Outcome::refused_by_it(
            Road::Window,
            Vec::new(),
            Ended::Refused,
            say(&words::IT_DID_NOT),
        ),
        Ok(Answer::NotActed(NotActed::NoWindow)) => {
            Outcome::not_asked(Ended::Refused, say(&words::NO_WINDOW_TO_ACT_ON))
        }
        Ok(Answer::NotActed(NotActed::NotInstalled)) => {
            Outcome::not_asked(Ended::NotOffered, say(&words::DESKTOP_SAYS_NOT_INSTALLED))
        }
        Ok(Answer::NotActed(NotActed::CouldNotAsk)) => {
            Outcome::not_asked(Ended::NotOffered, say(&words::COULD_NOT_ASK))
        }
        Err(Unanswered::NotReached) => {
            Outcome::not_asked(Ended::NotOffered, say(&words::DESKTOP_SILENT))
        }
        // **Sent, and the answer is not one this command knows**: never read as
        // success, and written down as having run, because it may have.
        Err(Unanswered::NotUnderstood) => Outcome::Carried {
            road: Road::Window,
            arguments: Vec::new(),
            answered: Answered::NoAnswer,
            ended: Ended::NotKnown,
            said: say(&words::WINDOW_UNANSWERED),
        },
    }
}
