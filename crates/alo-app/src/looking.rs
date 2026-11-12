//! **`controls` and `read`**: what an application's windows show, read once at
//! the moment of the command and kept nowhere — and **never written in the
//! record**, because a record of what a person looked at is a record of the
//! person (the owner's answer 5).
//!
//! The walk is `alo_adapters::read_by_hand`'s, the one alo's own road takes:
//! the same bounds, and a password field's contents are never read.
//!
//! On a terminal it is sentences in the person's language. On a pipe it is one
//! value per line, for a program, and never translated:
//!
//! ```text
//! controls:  <kind>\t<name>\t<what is true of it, comma-separated, or nothing>
//! read:      each piece of text a window shows, one per line
//! ```
//!
//! where `<kind>` is a kind `press` takes (`button`, `check_box`, …), or
//! `field` or `password_field`, and what is true of it is any of `greyed-out`,
//! `on` and `off`.

use alo_adapters::{Contents, Limit, NotWalked, Pressable, Role, Seen, Shown, read_by_hand};
use alo_strings::{Filling, Said, Strings};

use crate::acting::Telling;
use crate::ended::Ended;
use crate::machine::Machine;
use crate::subject::Subject;
use crate::words;

/// Which of the two looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    /// Every control.
    Controls,
    /// The text.
    Read,
}

/// Look, and tell what was seen.
pub fn looked(
    look: Look,
    subject: &Subject,
    telling: Telling,
    strings: &Strings,
    machine: &mut dyn Machine,
) -> Ended {
    let within = subject.within.as_deref();
    let shown = match machine.tree() {
        Ok(tree) => read_by_hand(tree, subject.whose(), within),
        Err(fault) => Err(NotWalked::Tree(fault)),
    };
    let shown = match shown {
        Ok(shown) => shown,
        Err(why) => {
            machine.warned(why.said(subject.said_as().named(), within, strings).text());
            return match why {
                NotWalked::Tree(alo_adapters::TreeFault::NotThere) => Ended::NotOffered,
                NotWalked::NoWindow
                | NotWalked::NoSuchWindow
                | NotWalked::MoreThanOneWindow
                | NotWalked::Tree(_) => Ended::Refused,
            };
        }
    };
    let lines = match (look, telling) {
        (Look::Controls, Telling::Sentences) => controls_said(&shown, strings),
        (Look::Controls, Telling::Values) => controls_as_values(&shown),
        (Look::Read, _) => text_of(&shown),
    };
    // **A read that finds nothing says so**, window by window, on the warning
    // side so a program reading the values still reads none — never a silent
    // success that looks like an empty document.
    if look == Look::Read && lines.is_empty() {
        // **Which switch is the likeliest reason.** VSCodium's document is
        // readable only in its screen reader mode, which alo turns on for it
        // only where the person let alo work fully inside apps like it
        // (`docs/quirks/vscodium-is-pressed-and-read-only-when-described-completely.md`);
        // for any other app, the likeliest reason is that it is not describing
        // itself at all.
        let access = machine.access();
        let until_it_works_fully = alo_launching::is_drawn_by_chromium(subject.identifier())
            && access.apps_describe_themselves()
            && !access.alo_works_fully_inside_apps();
        for window in shown.windows() {
            let titled = Filling::of(words::WINDOW, window.title().to_owned());
            let (word, filling) = if until_it_works_fully {
                (
                    words::NOTHING_TO_READ_UNTIL_ALO_WORKS_FULLY,
                    titled.and(words::APPLICATION, subject.called.clone()),
                )
            } else {
                (words::NOTHING_TO_READ, titled)
            };
            machine.warned(
                strings
                    .say(&word.key(), &filling)
                    .text(),
            );
        }
    }
    for line in lines {
        machine.told(&line);
    }
    if let Some(said) = shown.not_all_read_said(strings) {
        machine.warned(said.text());
    }
    Ended::Done
}

/// Whether a thing is a control: something a person presses or types into.
fn is_a_control(seen: &Seen) -> bool {
    Pressable::of(seen.role()).is_some()
        || matches!(seen.role(), Role::TextField | Role::PasswordField)
}

/// A control's kind, in words.
fn kind_said(role: Role, strings: &Strings) -> Option<Said> {
    let word = match (Pressable::of(role), role) {
        (Some(kind), _) => kind.words(),
        (None, Role::TextField) => words::A_FIELD,
        (None, Role::PasswordField) => words::A_PASSWORD_FIELD,
        (None, _) => return None,
    };
    Some(strings.say(&word.key(), &Filling::nothing()))
}

/// A control's kind, as a program reads it.
fn kind_named(role: Role) -> Option<&'static str> {
    match (Pressable::of(role), role) {
        (Some(kind), _) => Some(kind.named()),
        (None, Role::TextField) => Some("field"),
        (None, Role::PasswordField) => Some("password_field"),
        (None, _) => None,
    }
}

/// What is true of a control now, in words.
fn states_said(seen: &Seen, strings: &Strings) -> Vec<String> {
    let say = |word: words::Word| strings.say(&word.key(), &Filling::nothing()).into_text();
    let mut states = Vec::new();
    if !seen.is_usable() {
        states.push(say(words::GREYED_OUT));
    }
    match seen.is_on() {
        Some(true) => states.push(say(words::STATE_ON)),
        Some(false) => states.push(say(words::STATE_OFF)),
        None => {}
    }
    match seen.contents() {
        Contents::Text(text) => states.push(
            strings
                .say(&words::HOLDS.key(), &Filling::of(words::SAID, text.clone()))
                .into_text(),
        ),
        Contents::Withheld => states.push(say(words::WITHHELD)),
        Contents::Nothing => {}
    }
    states
}

/// What could not be read about a thing, in words.
fn limit_said(seen: &Seen, strings: &Strings) -> Option<String> {
    let said = match seen.limit()? {
        Limit::NoName => {
            let kind = kind_said(seen.role(), strings)?;
            strings.say(
                &words::UNNAMED_CONTROL.key(),
                &Filling::of(words::KIND, kind.into_text()),
            )
        }
        Limit::NotDescribed => strings.say(&words::NOT_DESCRIBED.key(), &Filling::nothing()),
        Limit::ShownByAnotherProgram => {
            strings.say(&words::ANOTHER_PROGRAMS.key(), &Filling::nothing())
        }
    };
    Some(said.into_text())
}

/// Every control, window by window, in sentences.
fn controls_said(shown: &Shown, strings: &Strings) -> Vec<String> {
    let mut lines = Vec::new();
    for window in shown.windows() {
        lines.push(
            strings
                .say(
                    &words::IN_THE_WINDOW.key(),
                    &Filling::of(words::WINDOW, window.title().to_owned()),
                )
                .into_text(),
        );
        let before = lines.len();
        for seen in window.seen() {
            if let Some(limit) = limit_said(seen, strings) {
                lines.push(format!("  {limit}"));
                continue;
            }
            if !is_a_control(seen) {
                continue;
            }
            let Some(kind) = kind_said(seen.role(), strings) else {
                continue;
            };
            let states = states_said(seen, strings);
            let filling =
                Filling::of(words::CONTROL, seen.name().to_owned()).and_said(words::KIND, &kind);
            let said = if states.is_empty() {
                strings.say(&words::A_CONTROL.key(), &filling)
            } else {
                strings.say(
                    &words::A_CONTROL_WITH.key(),
                    &filling.and(words::LIST, states.join(", ")),
                )
            };
            lines.push(format!("  {said}"));
        }
        if lines.len() == before {
            lines.push(format!(
                "  {}",
                strings.say(&words::NO_CONTROLS.key(), &Filling::nothing())
            ));
        }
    }
    lines
}

/// Every named control, one per line, for a program.
fn controls_as_values(shown: &Shown) -> Vec<String> {
    shown
        .everything()
        .filter(|seen| seen.limit().is_none() && is_a_control(seen))
        .filter_map(|seen| {
            let kind = kind_named(seen.role())?;
            let mut states = Vec::new();
            if !seen.is_usable() {
                states.push("greyed-out");
            }
            match seen.is_on() {
                Some(true) => states.push("on"),
                Some(false) => states.push("off"),
                None => {}
            }
            Some(format!("{kind}\t{}\t{}", seen.name(), states.join(",")))
        })
        .collect()
}

/// Every piece of text the windows show, in reading order.
fn text_of(shown: &Shown) -> Vec<String> {
    shown
        .everything()
        .filter_map(|seen| match seen.contents() {
            Contents::Text(text) => Some(text.clone()),
            Contents::Withheld => None,
            Contents::Nothing if seen.role().text_is_read() && !seen.name().is_empty() => {
                Some(seen.name().to_owned())
            }
            Contents::Nothing => None,
        })
        .collect()
}
