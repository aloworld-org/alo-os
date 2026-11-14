//! **`alo-reading-the-sign-in-screen`: the first start and the sign-in screen,
//! read aloud** ([ADR 0155](../../../docs/decisions/0155-the-screens-before-sign-in-speak.md)).
//!
//! Run as the sign-in login, `alo-greeter`, by that login's own manager
//! (`image/usr/lib/systemd/user/alo-reading-the-sign-in-screen.service`), from
//! boot. It knocks at the sign-in screen's door until the screen is there,
//! serves it until it goes, and knocks again — so a compositor that restarts,
//! or a person who signs out, is met by the same reader.
//!
//! Everything it decides is `crate::the_sign_in_screen_read_aloud`'s; this
//! file is only what a process is: which voice, which bus, which door.
//!
//! It lives in this package because the voice is the desktop's own —
//! `crate::starting_the_screen_reader` and `crate::watching_the_screen_reader`
//! start, stop and watch Orca the same way for a person and for the sign-in
//! screen, and two copies of that would be two answers to one question.

#[cfg(target_os = "linux")]
#[path = "starting_the_screen_reader.rs"]
mod starting_the_screen_reader;

#[cfg(target_os = "linux")]
#[path = "watching_the_screen_reader.rs"]
mod watching_the_screen_reader;

#[cfg(target_os = "linux")]
#[path = "the_sign_in_screen_read_aloud.rs"]
mod the_sign_in_screen_read_aloud;

/// What this process does on the machine alo OS is for.
#[cfg(target_os = "linux")]
mod running {
    use std::os::unix::net::UnixStream;
    use std::process::ExitCode;
    use std::sync::{Arc, Mutex};

    use alo_shell::{ReadAloudTree, THE_READING_DOOR, TheReaderIsTold, TheScreenReaderAnswered};

    use crate::starting_the_screen_reader::TheScreenReader;
    use crate::the_sign_in_screen_read_aloud::{
        BETWEEN_KNOCKS, THE_SCREENS_UID, THE_VOICE, TheBus, TheRelay, TheVoice,
        is_the_sign_in_screen, said_by, served, the_voice_for,
    };

    /// **Orca, in this login's manager**: the desktop's own switch worker, and
    /// the language it is to speak, read again at every start.
    struct OrcaHere {
        /// The worker that starts, stops and watches it.
        reader: TheScreenReader,
        /// The language the next start speaks in.
        language: Arc<Mutex<Option<String>>>,
        /// Every sentence this machine can say, for the one the voice says
        /// alone; [`None`] where the bundle would not read.
        words: Option<alo_strings::Strings>,
    }

    impl TheVoice for OrcaHere {
        fn is(&mut self, on: bool, language: Option<String>) {
            if on && let Ok(mut held) = self.language.lock() {
                *held = language;
            }
            self.reader.is(on);
        }

        fn answered(&mut self) -> Option<TheScreenReaderAnswered> {
            self.reader.answered()
        }

        /// **The sentence, in the screen's language, said by eSpeak NG on a
        /// thread of its own**, so the door is still answered while it speaks.
        fn says_how_to_hear_it(&mut self, language: Option<String>) {
            let Some(mut words) = self.words.clone() else {
                eprintln!(
                    "alo-reading-the-sign-in-screen: this machine's words would not read, so how \
                     to hear the screen is not said"
                );
                return;
            };
            let chosen: Vec<alo_strings::Language> = language
                .as_deref()
                .and_then(|tag| alo_strings::Language::written(tag).ok())
                .into_iter()
                .collect();
            words.prefers(&chosen);
            let sentence = words
                .say(
                    &alo_shell::sign_in_words::HEAR_THE_SCREEN.key(),
                    &alo_strings::Filling::nothing(),
                )
                .into_text();
            let voice = the_voice_for(language.as_deref());
            let spoken = std::thread::Builder::new()
                .name("how-to-hear-the-screen".to_owned())
                .spawn(move || {
                    if let Err(why) = said_by(std::path::Path::new(THE_VOICE), voice, &sentence) {
                        eprintln!(
                            "alo-reading-the-sign-in-screen: how to hear the screen was not \
                             heard: {why}"
                        );
                    }
                });
            if let Err(why) = spoken {
                eprintln!(
                    "alo-reading-the-sign-in-screen: how to hear the screen was not said, \
                     because no thread could be made for it: {why}"
                );
            }
        }
    }

    /// **This login's accessibility bus**, opened when the first screen
    /// arrives and again after any screen that could not be put on it.
    #[derive(Default)]
    struct TheLoginsBus {
        /// The screen on the bus, once it is.
        told: Option<TheReaderIsTold>,
    }

    impl TheBus for TheLoginsBus {
        fn shows(&mut self, screen: &ReadAloudTree) -> Result<(), String> {
            if let Some(told) = self.told.as_mut() {
                let shown = told
                    .following(screen)
                    .map(|_| ())
                    .map_err(|why| why.to_string());
                if shown.is_err() {
                    self.told = None;
                }
                return shown;
            }
            self.told = Some(
                TheReaderIsTold::before_sign_in(screen.clone()).map_err(|why| why.to_string())?,
            );
            Ok(())
        }
    }

    /// Knock, serve, and knock again, for as long as the machine runs.
    pub fn main() -> ExitCode {
        let language = Arc::new(Mutex::new(None::<String>));
        let speaking = Arc::clone(&language);
        let reader = TheScreenReader::acted_on_by(
            move |on| {
                let language = speaking.lock().ok().and_then(|held| held.clone());
                crate::starting_the_screen_reader::on_this_machine(on, language)
            },
            crate::watching_the_screen_reader::OnThisMachine::default(),
            crate::watching_the_screen_reader::BETWEEN_WATCHES,
        );
        let words = match alo_shell::what_this_machine_can_say() {
            Ok(words) => Some(words),
            Err(why) => {
                eprintln!(
                    "alo-reading-the-sign-in-screen: this machine's words would not read: {why}"
                );
                None
            }
        };
        let mut relay = TheRelay::of(
            OrcaHere {
                reader,
                language,
                words,
            },
            TheLoginsBus::default(),
        );
        eprintln!(
            "alo-reading-the-sign-in-screen: waiting for the sign-in screen at {THE_READING_DOOR}"
        );
        loop {
            // A door nobody has opened yet is the ordinary case at boot, and
            // is not said once a second.
            if let Ok(stream) = UnixStream::connect(THE_READING_DOOR)
                && is_the_sign_in_screen(&stream, THE_SCREENS_UID)
            {
                eprintln!("alo-reading-the-sign-in-screen: the sign-in screen is here");
                if let Err(why) = served(&mut relay, stream) {
                    eprintln!(
                        "alo-reading-the-sign-in-screen: the connection to the sign-in screen \
                         failed: {why}"
                    );
                }
                relay.the_screen_went_away();
                eprintln!("alo-reading-the-sign-in-screen: the sign-in screen went away");
            }
            std::thread::sleep(BETWEEN_KNOCKS);
        }
    }
}

#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    running::main()
}

/// Nothing but Linux has a sign-in screen of alo's to read.
#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    eprintln!("alo-reading-the-sign-in-screen: this runs on alo OS, which is Linux");
    std::process::ExitCode::FAILURE
}
