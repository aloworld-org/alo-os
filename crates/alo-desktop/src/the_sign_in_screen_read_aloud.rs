//! **The sign-in screen, read aloud by the sign-in login**
//! ([ADR 0155](../../../docs/decisions/0155-the-screens-before-sign-in-speak.md)).
//!
//! The compositor draws the first start and the sign-in screen as root and
//! never touches this login's buses. This is the other side of its one door
//! (`alo_shell::THE_READING_DOOR`): it runs as `alo-greeter`, in that login's
//! own manager, and is the only thing that puts the screen on the login's
//! accessibility bus and starts or stops its voice.
//!
//! - **A screen that arrives is put on the accessibility bus**
//!   (`alo_shell::TheReaderIsTold::before_sign_in`), and each that follows
//!   replaces it, so Orca hears each new screen and where the keyboard moves.
//! - **Read on or off** starts or stops Orca in this login's manager — the same
//!   unit, the same two-second watch and the same late-failure watch a person's
//!   desktop uses (`crate::starting_the_screen_reader`,
//!   `crate::watching_the_screen_reader`) — and what that came to goes back to
//!   the screen.
//! - **A new language while reading** starts Orca again in it, because Orca
//!   reads its language once, when it starts.
//! - **Somebody signing in** stops the voice and says so, so the person's own
//!   reader does not start over this one.
//!
//! # Only the sign-in screen is listened to
//!
//! The kernel says who is on the other end of the door (`SO_PEERCRED`), and a
//! door whose other end is not root — the compositor — is closed at once.
//! **Nothing that arrives is run**: a message is a tree, a language tag, or one
//! of two words (`alo_shell::ToTheReader`), and anything else is refused whole.

use std::io::{self, BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use alo_shell::{FromTheReader, ReadAloudTree, TheScreenReaderAnswered, ToTheReader};

/// How long between two knocks at a door nobody answered.
pub const BETWEEN_KNOCKS: Duration = Duration::from_secs(1);

/// How often a connection with nothing to say is looked at for the voice's
/// answers.
pub const BETWEEN_LOOKS: Duration = Duration::from_millis(200);

/// How long the voice is given to stop when somebody signs in.
pub const PATIENCE_AT_SIGN_IN: Duration = Duration::from_millis(1500);

/// The sign-in screen's uid: the compositor runs as root.
pub const THE_SCREENS_UID: u32 = 0;

/// **The voice**: Orca in this login's manager, as the relay asks it.
pub trait TheVoice {
    /// Read aloud in `language`, or stop. Asked only when it is a change.
    fn is(&mut self, on: bool, language: Option<String>);
    /// The next thing starting or stopping came to, where one has come.
    fn answered(&mut self) -> Option<TheScreenReaderAnswered>;
    /// **Say one sentence with the voice alone**, in `language`, without the
    /// screen reader: how to hear the screen, or that the reader is off. A
    /// machine with nothing to play it on says nothing, and that is said on
    /// the journal, not a failure.
    fn says(&mut self, saying: Saying, language: Option<String>);
    /// Whether this computer has anything to play a voice on.
    fn has_somewhere_to_play_it(&self) -> bool;
}

/// **The sentences the voice says alone**, before sign-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Saying {
    /// *Press Control, Alt and S to hear the screen read aloud* — once, on a
    /// new machine's first screen (`alo_shell`'s `the_spoken_hint`).
    HowToHearIt,
    /// *Screen reader off* — as the person turns it off.
    TheReaderIsOff,
}

impl Saying {
    /// The word it is, in the sign-in screen's vocabulary.
    #[must_use]
    pub const fn word(self) -> alo_strings::Word {
        match self {
            Self::HowToHearIt => alo_shell::sign_in_words::HEAR_THE_SCREEN,
            Self::TheReaderIsOff => alo_shell::sign_in_words::SCREEN_READER_OFF,
        }
    }
}

/// Where the kernel lists this computer's sound devices.
pub const THE_SOUND_DEVICES: &str = "/dev/snd";

/// **Whether `devices` lists anything to play sound on**: a playback device,
/// which the kernel names `pcmC<card>D<device>p`. A folder that is not there
/// or cannot be read is nothing to play it on.
#[must_use]
pub fn there_is_somewhere_to_play_it(devices: &std::path::Path) -> bool {
    std::fs::read_dir(devices).is_ok_and(|listed| {
        listed.filter_map(Result::ok).any(|device| {
            let name = device.file_name();
            let name = name.to_string_lossy();
            name.starts_with("pcmC") && name.ends_with('p')
        })
    })
}

/// **The accessibility bus**, as the relay puts a screen on it.
pub trait TheBus {
    /// Put `screen` on the bus in place of what was there.
    ///
    /// # Errors
    /// Why it could not be, for the journal; the next screen tries again.
    fn shows(&mut self, screen: &ReadAloudTree) -> Result<(), String>;
}

/// **The relay between messages**: whether it was asked to read, and in which
/// language.
#[derive(Debug)]
pub struct TheRelay<V, B> {
    /// The voice.
    voice: V,
    /// The bus.
    bus: B,
    /// Whether it was last asked to read.
    reading: bool,
    /// The language it was last asked to read in.
    language: Option<String>,
    /// **Whether it has said how to hear the screen** — once for as long as
    /// this relay runs, which is the whole boot, so a sign-in screen started
    /// again does not say it twice.
    said_how_to_hear_it: bool,
}

impl<V: TheVoice, B: TheBus> TheRelay<V, B> {
    /// A relay with `voice` and `bus`, reading nothing.
    pub fn of(voice: V, bus: B) -> Self {
        Self {
            voice,
            bus,
            reading: false,
            language: None,
            said_how_to_hear_it: false,
        }
    }

    /// **One message from the sign-in screen**, and what to tell it back.
    pub fn heard(&mut self, message: ToTheReader) -> Vec<FromTheReader> {
        match message {
            ToTheReader::TheScreen(screen) => {
                if let Err(why) = self.bus.shows(&screen) {
                    eprintln!(
                        "alo-reading-the-sign-in-screen: the screen could not be put on the \
                         accessibility bus, and the next one will be tried: {why}"
                    );
                }
                Vec::new()
            }
            ToTheReader::Read { on, language } => {
                let mut told = Vec::new();
                match (self.reading, on) {
                    (true, true) if language == self.language => {}
                    (false, false) => {}
                    // **On, or a new language while on** — which starts the
                    // voice again in it, because Orca reads its language once,
                    // as it starts. **Not on a machine with nothing to play it
                    // on**: that is said, apart from *did not start*.
                    (_, true) => {
                        if self.reading {
                            self.voice.is(false, None);
                        }
                        if self.voice.has_somewhere_to_play_it() {
                            self.voice.is(true, language.clone());
                        } else {
                            eprintln!(
                                "alo-reading-the-sign-in-screen: the screen reader was turned on \
                                 and this computer has no sound device to play it on"
                            );
                            told.push(FromTheReader::Answered(
                                TheScreenReaderAnswered::NothingToPlayItOn,
                            ));
                        }
                    }
                    // **Off is said before the voice stops**, so the person
                    // hears that it worked.
                    (true, false) => {
                        self.voice
                            .says(Saying::TheReaderIsOff, self.language.clone());
                        self.voice.is(false, language.clone());
                    }
                }
                self.reading = on;
                self.language = language;
                told.extend(self.answers());
                told
            }
            ToTheReader::SomebodyIsSigningIn => self.steps_aside(PATIENCE_AT_SIGN_IN),
            // **Once a boot, and never over the reader.**
            ToTheReader::SayHowToHearIt { language } => {
                if !self.said_how_to_hear_it && !self.reading {
                    self.said_how_to_hear_it = true;
                    self.voice.says(Saying::HowToHearIt, language);
                }
                Vec::new()
            }
        }
    }

    /// What the voice answered since this was last asked.
    pub fn answers(&mut self) -> Vec<FromTheReader> {
        std::iter::from_fn(|| self.voice.answered())
            .map(FromTheReader::Answered)
            .collect()
    }

    /// **Somebody is signing in**: stop the voice, wait up to `patience` for
    /// it to say it has, and say this relay has stepped aside — whether it
    /// answered or not, because the session opens either way.
    pub fn steps_aside(&mut self, patience: Duration) -> Vec<FromTheReader> {
        let mut told = self.answers();
        if self.reading {
            self.voice.is(false, None);
            self.reading = false;
            let until = Instant::now() + patience;
            loop {
                let now = self.answers();
                let stopped = now.iter().any(|it| {
                    matches!(
                        it,
                        FromTheReader::Answered(
                            TheScreenReaderAnswered::Stopped | TheScreenReaderAnswered::DidNotStop
                        )
                    )
                });
                told.extend(now);
                if stopped {
                    break;
                }
                if Instant::now() >= until {
                    eprintln!(
                        "alo-reading-the-sign-in-screen: the voice did not say it had stopped \
                         within {patience:?}; stepping aside anyway"
                    );
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        told.push(FromTheReader::SteppedAside);
        told
    }

    /// **The sign-in screen went away** — signed in, restarted, or fell over:
    /// nothing is left to read, so the voice stops.
    pub fn the_screen_went_away(&mut self) {
        if self.reading {
            self.voice.is(false, None);
            self.reading = false;
        }
        let _ = self.answers();
    }
}

/// The voice program: eSpeak NG, ADR 0151's, which speaks all 24 languages on
/// this computer and nowhere else.
pub const THE_VOICE: &str = "/usr/bin/espeak-ng";

/// **The voice eSpeak NG speaks `language` in**: its own language, where the
/// engine has one (`alo_access::voices`), and English where it has none or
/// none was named — so a hint is always said in something.
#[must_use]
pub fn the_voice_for(language: Option<&str>) -> &'static str {
    let primary = language
        .and_then(|tag| tag.split(['-', '_']).next())
        .map(str::to_ascii_lowercase);
    alo_access::voices::A_VOICE_FOR_EACH
        .iter()
        .find(|(tag, voice)| Some(*tag) == primary.as_deref() && !voice.is_empty())
        .map_or("en", |(tag, _)| *tag)
}

/// **Say `sentence` with `program` in `voice`, and wait until it is said.**
///
/// # Errors
/// What went wrong, for the journal: the program is not there, or it said it
/// could not speak — which is what a machine with no sound device answers.
/// Neither is a failure of anything else: the sentence is simply not heard.
pub fn said_by(program: &std::path::Path, voice: &str, sentence: &str) -> Result<(), String> {
    let ran = std::process::Command::new(program)
        .args(["-v", voice, "--", sentence])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|why| format!("{} could not be started: {why}", program.display()))?;
    if ran.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} could not speak ({}): {}",
            program.display(),
            ran.status,
            String::from_utf8_lossy(&ran.stderr).trim()
        ))
    }
}

/// **Whether the other end of `stream` is the sign-in screen**: the kernel's
/// answer for who it is, held to `the_screens_uid`.
pub fn is_the_sign_in_screen(stream: &UnixStream, the_screens_uid: u32) -> bool {
    match rustix::net::sockopt::socket_peercred(stream) {
        Ok(peer) if peer.uid.as_raw() == the_screens_uid => true,
        Ok(peer) => {
            eprintln!(
                "alo-reading-the-sign-in-screen: the door's other end is uid {}, not the sign-in \
                 screen's {the_screens_uid}, so nothing it says is listened to",
                peer.uid.as_raw()
            );
            false
        }
        Err(why) => {
            eprintln!(
                "alo-reading-the-sign-in-screen: the kernel would not say who is at the door, so \
                 nothing it says is listened to: {why}"
            );
            false
        }
    }
}

/// **One connection to the sign-in screen, served until it ends**: each line
/// heard is answered, and what the voice says in between is passed on.
///
/// # Errors
/// What the connection could not do, for the journal; the caller knocks
/// again.
pub fn served<V: TheVoice, B: TheBus>(
    relay: &mut TheRelay<V, B>,
    stream: UnixStream,
) -> io::Result<()> {
    stream.set_read_timeout(Some(BETWEEN_LOOKS))?;
    let mut writing = stream.try_clone()?;
    let mut reading = BufReader::new(stream);
    let mut heard = Vec::new();
    loop {
        let told = match reading.read_until(b'\n', &mut heard) {
            Ok(0) => return Ok(()),
            Ok(_) if heard.ends_with(b"\n") => {
                let line = String::from_utf8_lossy(&heard).into_owned();
                heard.clear();
                match ToTheReader::read(&line) {
                    Ok(message) => relay.heard(message),
                    Err(why) => {
                        eprintln!(
                            "alo-reading-the-sign-in-screen: the sign-in screen said something \
                             that is not a message, and it was ignored: {why}"
                        );
                        Vec::new()
                    }
                }
            }
            Ok(_) => Vec::new(),
            Err(why)
                if matches!(
                    why.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) =>
            {
                relay.answers()
            }
            Err(why) => return Err(why),
        };
        if heard.len() > alo_shell::THE_LONGEST_LINE_A_READER_IS_SENT {
            return Err(io::Error::other(
                "the sign-in screen said more on one line than this reads",
            ));
        }
        for message in told {
            writing.write_all(message.written().as_bytes())?;
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_sign_in_screen_read_aloud_tests.rs"]
mod tests;
