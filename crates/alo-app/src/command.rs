//! **One command line, from what was typed to how it ends.**
//!
//! What the person typed is read before the machine is touched, so a person who
//! mistyped the shape is told that and not something about their machine. Then
//! the name in the application's place is read — and `front` and `alo` are
//! asked of the desktop, once — and then the command looks or acts.

use alo_strings::Filling;

use crate::acting::{Acting, Telling, refused_before_a_subject};
use crate::declared::the_ability;
use crate::ended::Ended;
use crate::listing::{Giving, listed};
use crate::looking::{Look, looked};
use crate::machine::{Knowing, Machine};
use crate::showing::{OnTheTree, on_the_tree, shown};
use crate::subject::the_subject;
use crate::typed::{Asked, Asking, Doing, NotTyped, Typed, read};
use crate::words;

/// Read what was typed and carry it out. `to_a_terminal` is whether what the
/// command gives back goes to a terminal, rather than to a pipe or a file.
pub fn run(
    typed: &[String],
    to_a_terminal: bool,
    knowing: Knowing<'_>,
    machine: &mut dyn Machine,
) -> Ended {
    let strings = knowing.strings;
    let typed = match read(typed) {
        Ok(typed) => typed,
        Err(why) => return not_typed(&why, knowing, machine),
    };
    let Typed { asking, asked } = typed;
    let giving = match (asking, to_a_terminal) {
        (Asking::Json, _) => Giving::Json,
        (Asking::Plain, _) | (Asking::AsTheOutputIs, true) => Giving::Sentences,
        (Asking::AsTheOutputIs, false) => Giving::Values,
    };
    let telling = match giving {
        Giving::Sentences => Telling::Sentences,
        Giving::Values | Giving::Json => Telling::Values,
    };
    match asked {
        Asked::List => {
            for line in listed(knowing.installed, knowing.adapters, giving, strings) {
                machine.told(&line);
            }
            Ended::Done
        }
        Asked::Show {
            application,
            ability,
        } => {
            let subject = match the_subject(&application, &Doing::Controls, None, knowing, machine)
            {
                Ok(subject) => subject,
                Err(why) => {
                    machine.warned(why.said.text());
                    return why.ended;
                }
            };
            let one = match (&ability, subject.application()) {
                (None, _) => None,
                (Some(ability), Some(found)) => {
                    match the_ability(knowing.adapters, found.identifier(), ability, strings) {
                        Ok(one) => Some(one),
                        Err(refused) => {
                            machine.warned(refused.said().text());
                            return refused.ended();
                        }
                    }
                }
                (Some(_), None) => {
                    machine.warned(
                        strings
                            .say(&words::ALO_TREE_ONLY.key(), &Filling::nothing())
                            .text(),
                    );
                    return Ended::NotOffered;
                }
            };
            let tree = match (subject.application(), one) {
                (Some(found), None) => on_the_tree(found.identifier(), machine),
                _ => OnTheTree::NotOpen,
            };
            for line in shown(&subject, one, tree, giving, knowing) {
                machine.told(&line);
            }
            Ended::Done
        }
        Asked::Of {
            application,
            doing,
            within,
        } => {
            let subject =
                match the_subject(&application, &doing, within.as_deref(), knowing, machine) {
                    Ok(subject) => subject,
                    Err(why) => {
                        if doing.acts() {
                            refused_before_a_subject(
                                &application,
                                doing.named(),
                                &why.said,
                                strings,
                                machine,
                            );
                        } else {
                            machine.warned(why.said.text());
                        }
                        return why.ended;
                    }
                };
            match doing {
                Doing::Controls => looked(Look::Controls, &subject, telling, strings, machine),
                Doing::Read => looked(Look::Read, &subject, telling, strings, machine),
                Doing::Press { .. } | Doing::Window(_) | Doing::Ability { .. } => Acting {
                    subject: &subject,
                    knowing,
                    telling,
                }
                .every_call(&doing, machine),
            }
        }
    }
}

/// A command line not in the shape: said, and written down where it was an
/// act the person had named.
fn not_typed(why: &NotTyped, knowing: Knowing<'_>, machine: &mut dyn Machine) -> Ended {
    let said = why.said(knowing.strings);
    match why.of_an_act() {
        Some((application, ability)) => {
            refused_before_a_subject(application, ability, &said, knowing.strings, machine);
        }
        None => machine.warned(said.text()),
    }
    Ended::NotThisShape
}
