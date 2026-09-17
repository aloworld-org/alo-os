//! **Where a person is asked what a device wants to know, and nothing is
//! answered for them.**
//!
//! While a pairing is going on, the rented service turns to whoever is running
//! the machine and asks one of four things: *show these six digits so they can
//! be typed on it*, *these six digits — does the device show the same?*, *what
//! is the code printed on it?*, or *may this device connect at all?*. Each of
//! them is a question for a person, and [`Asking`] is the one place they are put
//! to one.
//!
//! # Why this is not in the D-Bus object
//!
//! Because then it could only be tested on a machine with a radio in it, and
//! this is the part that must not be wrong. `crate::pairing_agent` is a shell
//! that hands the service's callbacks to this, and everything decided is decided
//! here, against [`ThePersonsOwnSurface`] — which a test can be.
//!
//! # What is never done
//!
//! - No answer is invented. A surface that does not answer is a *no*.
//! - No code is guessed. Older devices very often take `0000` or `1234`, every
//!   pairing tool on the internet tries them, and this one asks instead: a
//!   machine that guessed would be pairing with something on a person's behalf.
//! - **Nothing is remembered.** A person who confirmed one pairing has confirmed
//!   one pairing, and the next device asks again.

use crate::confirming::{Answer, Asked};
use crate::reported::Found;

/// Where the person is asked, in their own session.
pub trait ThePersonsOwnSurface {
    /// **Show them what the device asked for, and say what they answered.**
    ///
    /// A surface that cannot ask — nobody is signed in, the screen is locked —
    /// answers [`Answer::No`], which is what an unattended machine should say
    /// to a device that wants to pair with it.
    fn answer(&self, device: &Found, asked: &Asked) -> Answer;

    /// **The code a person read off the device**, where it has one printed on
    /// it, or [`None`] if they did not give one.
    fn code_from(&self, device: &Found) -> Option<String>;
}

/// **A person, asked.**
pub struct Asking<'a> {
    /// Where they are asked.
    surface: &'a dyn ThePersonsOwnSurface,
}

impl<'a> Asking<'a> {
    /// Ask through this surface.
    #[must_use]
    pub const fn through(surface: &'a dyn ThePersonsOwnSurface) -> Self {
        Self { surface }
    }

    /// **Six digits both ends show.** The person compares and says.
    pub fn the_same_on_both(&self, device: &Found, passkey: u32) -> Result<(), Refused> {
        self.said_yes(device, &Asked::TheSameOnBoth(passkey))
    }

    /// **Six digits to type on the device.** Shown, and the person says when
    /// they have done it — or that they will not.
    pub fn type_this_on_it(&self, device: &Found, passkey: u32) -> Result<(), Refused> {
        self.said_yes(device, &Asked::TypeThisOnIt(passkey))
    }

    /// **May this device connect at all**, which a device asks when it wants in
    /// without showing anything.
    pub fn pair_with_this(&self, device: &Found) -> Result<(), Refused> {
        self.said_yes(device, &Asked::Nothing)
    }

    /// **The code printed on the device**, asked of the person and never
    /// guessed.
    ///
    /// # Errors
    /// [`Refused`] where they gave none.
    pub fn its_own_code(&self, device: &Found) -> Result<String, Refused> {
        if !self.surface.answer(device, &Asked::ItsOwnCode).is_yes() {
            return Err(Refused);
        }
        let code = self.surface.code_from(device).unwrap_or_default();
        let code = code.trim().to_owned();
        if code.is_empty() {
            return Err(Refused);
        }
        Ok(code)
    }

    /// Whether the person said yes to what they were shown.
    fn said_yes(&self, device: &Found, asked: &Asked) -> Result<(), Refused> {
        if self.surface.answer(device, asked).is_yes() {
            return Ok(());
        }
        Err(Refused)
    }
}

/// **The person said no**, or was not there to say anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("nobody confirmed it")]
pub struct Refused;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reported::Kind;
    use crate::testing::a_device;
    use std::cell::RefCell;

    /// A person who answers the same way every time, and remembers what they
    /// were shown.
    struct APerson {
        /// What they say.
        says: Answer,
        /// The code they have, if any.
        code: Option<String>,
        /// What they were shown.
        shown: RefCell<Vec<Asked>>,
    }

    impl APerson {
        fn saying(says: Answer) -> Self {
            Self {
                says,
                code: None,
                shown: RefCell::new(Vec::new()),
            }
        }

        fn with_the_code(mut self, code: &str) -> Self {
            self.code = Some(code.to_owned());
            self
        }
    }

    impl ThePersonsOwnSurface for APerson {
        fn answer(&self, _device: &Found, asked: &Asked) -> Answer {
            self.shown.borrow_mut().push(asked.clone());
            self.says
        }

        fn code_from(&self, _device: &Found) -> Option<String> {
            self.code.clone()
        }
    }

    fn a_headset() -> Found {
        a_device("AA:BB:CC:DD:EE:FF", "A headset", Kind::Audio, false)
    }

    /// **Every question reaches the person**, with the digits in it.
    #[test]
    fn what_the_device_asked_is_what_the_person_is_shown() {
        let person = APerson::saying(Answer::Yes);
        let asking = Asking::through(&person);
        assert!(asking.the_same_on_both(&a_headset(), 1_234).is_ok());
        assert!(asking.type_this_on_it(&a_headset(), 12).is_ok());
        assert!(asking.pair_with_this(&a_headset()).is_ok());

        let shown = person.shown.borrow();
        assert_eq!(
            shown.as_slice(),
            [
                Asked::TheSameOnBoth(1_234),
                Asked::TypeThisOnIt(12),
                Asked::Nothing
            ]
        );
        assert_eq!(
            shown.first().and_then(Asked::shown).as_deref(),
            Some("001234")
        );
    }

    /// **A person who says no is a pairing that does not happen**, in every one
    /// of the four shapes.
    #[test]
    fn nothing_pairs_when_the_person_says_no() {
        let person = APerson::saying(Answer::No).with_the_code("0000");
        let asking = Asking::through(&person);
        assert_eq!(asking.the_same_on_both(&a_headset(), 1), Err(Refused));
        assert_eq!(asking.type_this_on_it(&a_headset(), 1), Err(Refused));
        assert_eq!(asking.pair_with_this(&a_headset()), Err(Refused));
        assert_eq!(asking.its_own_code(&a_headset()), Err(Refused));
    }

    /// **A code is the person's, never the machine's guess** — and a person who
    /// gives none has given none, rather than `0000`.
    #[test]
    fn a_code_nobody_typed_is_not_a_code_this_machine_invents() {
        let nobody_typed = APerson::saying(Answer::Yes);
        assert_eq!(
            Asking::through(&nobody_typed).its_own_code(&a_headset()),
            Err(Refused),
            "a machine that fell back to a common code would be pairing on somebody's behalf"
        );

        let blank = APerson::saying(Answer::Yes).with_the_code("   ");
        assert_eq!(
            Asking::through(&blank).its_own_code(&a_headset()),
            Err(Refused)
        );

        let typed = APerson::saying(Answer::Yes).with_the_code(" 4213 ");
        assert_eq!(
            Asking::through(&typed).its_own_code(&a_headset()),
            Ok("4213".to_owned())
        );
    }
}
