//! Scanning for networks and joining the one a person chose, with each failure
//! said apart from the others.
//!
//! Item 6 of `docs/autonomy/the-environment-in-full-plan.md`. The parts a
//! person reads are the desktop lane's; this is the reading and the asking.
//!
//! # The promise this has to keep by construction, not by accident
//!
//! [ADR 0128](../../../docs/decisions/0128-whether-a-wi-fi-password-may-be-written-to-the-staging-partition.md)
//! decided the environment **asks** for the network and that **no pre-shared
//! key is written to the staging partition**, and `S3 · Choose a network` says
//! it to a person in as many words: *your Wi-Fi password is never written to
//! this computer's disk.*
//!
//! **NetworkManager writes one by default.** `nmcli device wifi connect` makes
//! a lasting connection profile, and a profile with a `psk` in it lands in
//! `/etc/NetworkManager/system-connections/` — a directory the image has.
//! Measured 2026-10-20.
//!
//! In this environment that happens to be harmless: the root is the initramfs,
//! which is memory, so nothing reaches a disk. **But that is the filesystem
//! being kind rather than the promise being kept**, and a promise that holds
//! because of where it happens to be running is one that breaks the first time
//! something else changes. So the profile is made with [`TEMPORARY`], which
//! keeps it in memory and writes no keyfile at all — the promise true by
//! construction, on any root.
//!
//! # Why two commands rather than one
//!
//! `nmcli device wifi connect …` is one call and has no way to say *do not
//! write this down*. `connection add --temporary` then `connection up` is two,
//! and the first of them carries the flag. One call that keeps a secret is
//! worth less than two that do not.
//!
//! # Five failures, because *no networks found* was standing in for all of them
//!
//! A person told *no networks found* when the truth is *your password was
//! refused* cannot act on it, and nor can a person whose card has no firmware.
//! Each of the five below is a different thing to go and do, which is the whole
//! reason this file is not one `bool`.
//!
//! # What is measured and what is not
//!
//! **The parsing and the classification are tested; no `nmcli` has been run.**
//! This build machine has no `nmcli` and no wireless card. The flags and the
//! exit codes below are from `nmcli`'s own documentation and its stable exit
//! codes, **not from a run** — and the words matched in its errors are matched
//! case-insensitively and loosely for that reason. The certified machine's boot
//! is the measurement this file is owed.

/// What keeps a connection profile out of any keyfile.
///
/// `nmcli connection add --temporary` holds the profile in memory for the life
/// of the daemon and writes nothing. See this module's header for why that is
/// a promise rather than a preference.
pub const TEMPORARY: &str = "--temporary";

/// A network the card can see.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ANetwork {
    /// What it calls itself. Empty where it does not say — a hidden network.
    name: String,
    /// How strong, nought to a hundred.
    strength: u8,
    /// Whether it wants a password.
    locked: bool,
}

impl ANetwork {
    /// What it calls itself.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// How strong, nought to a hundred.
    #[must_use]
    pub const fn strength(&self) -> u8 {
        self.strength
    }

    /// Whether it wants a password.
    #[must_use]
    pub const fn locked(&self) -> bool {
        self.locked
    }
}

/// Every network a scan found, strongest first.
///
/// **Strongest first because a person picks the one they are standing next
/// to**, and a list in the order a card happened to answer in is a list they
/// have to read all of.
///
/// `nmcli -t` separates fields with colons and escapes a colon inside a value
/// as `\:` — so a network called `Flat 2:Upstairs` arrives as
/// `Flat 2\:Upstairs`. **Splitting on every colon would make two networks out
/// of one and give the second one a nonsense strength.** This splits on
/// unescaped colons only, and that is the one thing about this format worth a
/// test.
#[must_use]
pub fn every_network(scanned: &str) -> Vec<ANetwork> {
    let mut found: Vec<ANetwork> = scanned
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let fields = unescaped_fields(line);
            let [name, strength, security] = fields.as_slice() else {
                return None;
            };
            Some(ANetwork {
                name: name.clone(),
                strength: strength.trim().parse().unwrap_or(0),
                // `--` is nmcli's own word for *nothing*, which here means an
                // open network. Anything else named is a kind of security.
                locked: !matches!(security.trim(), "--" | ""),
            })
        })
        .collect();
    found.sort_by(|one, two| {
        two.strength
            .cmp(&one.strength)
            .then_with(|| one.name.cmp(&two.name))
    });
    found
}

/// One terse line's fields, split on unescaped colons, with `\:` and `\\`
/// turned back into what they stand for.
fn unescaped_fields(line: &str) -> Vec<String> {
    let mut fields = vec![String::new()];
    let mut escaped = false;
    for at in line.chars() {
        if escaped {
            if let Some(last) = fields.last_mut() {
                last.push(at);
            }
            escaped = false;
        } else if at == '\\' {
            escaped = true;
        } else if at == ':' {
            fields.push(String::new());
        } else if let Some(last) = fields.last_mut() {
            last.push(at);
        }
    }
    fields
}

/// Why a network was not joined.
///
/// Five, because one sentence was standing in for all of them and a person
/// cannot act on the wrong one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotJoined {
    /// There is no wireless card the system can see.
    ///
    /// On the certified machine this is a driver that did not load or firmware
    /// that is not there, and it is the case where *check your password* and
    /// *move closer* are both impossible.
    NoCard,
    /// The card looked and found nothing at all.
    NothingToJoin,
    /// The password was refused.
    ThePasswordWasRefused,
    /// It tried and the network never answered.
    ItNeverAnswered,
    /// It joined and nothing gave it an address, so there is no road out even
    /// though the radio is associated.
    NoAddressCame,
}

/// Why the join failed, from what `nmcli` exited with and said.
///
/// **The exit code first, because it is the part that is not prose.** `nmcli`'s
/// codes are documented and stable: `3` is a timeout, `4` is an activation that
/// failed, `8` is the daemon not running, `10` is something not found. Its
/// English is matched only to tell two cases apart that share a code, and
/// loosely, because the words are a version's and the codes are an interface's.
#[must_use]
pub fn why_not_joined(exit: i32, said: &str, there_is_a_card: bool) -> NotJoined {
    if !there_is_a_card {
        return NotJoined::NoCard;
    }
    let said = said.to_ascii_lowercase();
    // A refused password is an activation failure that asked for a secret and
    // did not get one it liked. nmcli says so in words; the code alone cannot
    // tell it from a network that vanished mid-association.
    if said.contains("secrets were required") || said.contains("no secrets provided") {
        return NotJoined::ThePasswordWasRefused;
    }
    if said.contains("no network with ssid") {
        return NotJoined::NothingToJoin;
    }
    match exit {
        3 => NotJoined::ItNeverAnswered,
        10 => NotJoined::NothingToJoin,
        _ => NotJoined::NoAddressCame,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A scan is read, and the strongest comes first.**
    #[test]
    fn a_scan_is_read_strongest_first() {
        let found = every_network("Cafe:41:WPA2\nFRITZ!Box 7590:72:WPA2\nBTHub:55:WPA3\n");
        assert_eq!(
            found.iter().map(ANetwork::name).collect::<Vec<_>>(),
            ["FRITZ!Box 7590", "BTHub", "Cafe"]
        );
        assert_eq!(found.first().map(ANetwork::strength), Some(72));
    }

    /// **A colon inside a name does not make two networks.**
    ///
    /// The one thing about `nmcli -t` worth a test: it escapes a colon in a
    /// value as `\:`, so splitting on every colon turns `Flat 2:Upstairs` into
    /// a network called `Flat 2` with a strength parsed from `Upstairs`.
    #[test]
    fn a_colon_in_a_name_does_not_make_two_networks() {
        let found = every_network(r"Flat 2\:Upstairs:66:WPA2");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found.first().map(ANetwork::name), Some("Flat 2:Upstairs"));
        assert_eq!(found.first().map(ANetwork::strength), Some(66));
    }

    /// **A backslash in a name survives too.**
    #[test]
    fn a_backslash_in_a_name_survives() {
        let found = every_network(r"Up\\Down:50:WPA2");
        assert_eq!(found.first().map(ANetwork::name), Some(r"Up\Down"));
    }

    /// **An open network is not locked, and `--` is nmcli's word for it.**
    #[test]
    fn an_open_network_is_not_locked() {
        let found = every_network("Library:60:--\nHome:60:WPA2\n");
        let open = found.iter().find(|one| one.name() == "Library");
        assert_eq!(open.map(ANetwork::locked), Some(false));
        let locked = found.iter().find(|one| one.name() == "Home");
        assert_eq!(locked.map(ANetwork::locked), Some(true));
    }

    /// **A hidden network is a network with no name, not a missing line.**
    #[test]
    fn a_hidden_network_has_no_name() {
        let found = every_network(":30:WPA2\n");
        assert_eq!(found.len(), 1);
        assert_eq!(found.first().map(ANetwork::name), Some(""));
    }

    /// **Nothing that is not a scan line becomes a network.**
    #[test]
    fn nothing_that_is_not_a_scan_line_becomes_a_network() {
        assert!(every_network("").is_empty());
        assert!(every_network("\n\n  \n").is_empty());
        // Too few fields and too many are both not this format.
        assert!(every_network("JustAName\n").is_empty());
        assert!(every_network("Name:70\n").is_empty());
        assert!(every_network("Name:70:WPA2:extra\n").is_empty());
    }

    /// **A strength that is not a number is nought, not a refusal.**
    ///
    /// A card that answered with a strength this cannot read has still found a
    /// network, and dropping it would hide a network a person can see with
    /// their own eyes. It sorts last, which is where an unknown belongs.
    #[test]
    fn a_strength_that_is_not_a_number_sorts_last() {
        let found = every_network("Odd:nonsense:WPA2\nGood:10:WPA2\n");
        assert_eq!(
            found.iter().map(ANetwork::name).collect::<Vec<_>>(),
            ["Good", "Odd"]
        );
    }

    /// **No card beats every other reading.**
    ///
    /// It is checked before the exit code and before the words, because a
    /// machine with no wireless card can produce any of them and *check your
    /// password* is impossible on all of them.
    #[test]
    fn no_card_beats_everything_else() {
        for (exit, said) in [
            (4, "Secrets were required, but not provided"),
            (3, "Timeout expired"),
            (10, "No network with SSID 'Home' found"),
            (0, ""),
        ] {
            assert_eq!(why_not_joined(exit, said, false), NotJoined::NoCard);
        }
    }

    /// **A refused password is said as one**, and not as a timeout.
    #[test]
    fn a_refused_password_is_said_as_one() {
        assert_eq!(
            why_not_joined(
                4,
                "Error: Connection activation failed: Secrets were required, but not provided",
                true
            ),
            NotJoined::ThePasswordWasRefused
        );
        // However it is capitalised, because the words are a version's.
        assert_eq!(
            why_not_joined(4, "secrets were required", true),
            NotJoined::ThePasswordWasRefused
        );
    }

    /// **A network that is not there is not a wrong password.**
    #[test]
    fn a_network_that_is_not_there_says_so() {
        assert_eq!(
            why_not_joined(10, "Error: No network with SSID 'Home' found.", true),
            NotJoined::NothingToJoin
        );
        // And the code alone says it, where the words are something else.
        assert_eq!(why_not_joined(10, "", true), NotJoined::NothingToJoin);
    }

    /// **A timeout is its own sentence.**
    #[test]
    fn a_timeout_is_its_own_sentence() {
        assert_eq!(
            why_not_joined(3, "Error: Timeout expired", true),
            NotJoined::ItNeverAnswered
        );
    }

    /// **Each of the five is reachable and no two readings collide.**
    ///
    /// The test that would have caught the fault this file exists to remove:
    /// one sentence standing in for five. If two of these ever became one
    /// answer, this fails rather than a person being told the wrong thing.
    #[test]
    fn each_of_the_five_is_its_own_reading() {
        let readings = [
            why_not_joined(4, "", false),
            why_not_joined(10, "", true),
            why_not_joined(4, "Secrets were required", true),
            why_not_joined(3, "Timeout expired", true),
            why_not_joined(4, "Connection activation failed", true),
        ];
        for (one, first) in readings.iter().enumerate() {
            for (two, second) in readings.iter().enumerate() {
                assert_eq!(
                    one == two,
                    first == second,
                    "readings {one} and {two} are the same sentence: {first:?}"
                );
            }
        }
    }
}

/// Join this network, with this password, and say why if it did not.
///
/// # The two commands, and why the order is not a preference
///
/// The profile is **made first and brought up second**, because the flag that
/// keeps the password out of any keyfile belongs to the making. A single
/// `nmcli device wifi connect` would be one call and would write the key down;
/// see [`crate::Program::AddingANetwork`] for the whole of that argument.
///
/// If the making fails there is nothing to bring up, so the second command is
/// not run — a failure that reported *the network never answered* when the
/// profile was never made would send a person to move closer to a router that
/// was never contacted.
///
/// # Errors
///
/// [`NotJoined`], which is one of five and never a sixth: a person told *no
/// networks found* when the truth is *your password was refused* cannot act on
/// it, and nor can a person whose card has no firmware.
///
/// `there_is_a_card` is asked of the caller rather than worked out here,
/// because it is `why_there_is_no_network`'s question and that module answers
/// it off the kernel's own words rather than off a translated string.
///
/// # What is measured and what is not
///
/// **The arguments and the classification are tested; no `nmcli` has been run
/// against a real card.** This build machine has no wireless at all. The
/// certified machine's own boot is the measurement this owes, and it is the
/// same debt the rest of this file already carries.
pub fn join(
    machine: &mut impl crate::TheMachine,
    still: &alo_strings::Said,
    ssid: &str,
    password: &str,
    there_is_a_card: bool,
) -> Result<(), NotJoined> {
    let making = crate::Program::AddingANetwork {
        ssid: ssid.to_owned(),
        password: password.to_owned(),
    };
    let made = machine
        .run(&making, still, crate::STILL_EVERY)
        .map_err(|_| NotJoined::NoAddressCame)?;
    if !made.succeeded {
        // **What it complained of is noted and the arguments are not.** The
        // password is in the arguments of the command above and in nothing
        // else: `sequence::noted` writes a program's path and the lines it
        // complained of, and a test below holds that the password is in
        // neither.
        return Err(why_not_joined(1, &made.complained, there_is_a_card));
    }

    let bringing = crate::Program::BringingItUp {
        ssid: ssid.to_owned(),
    };
    let up = machine
        .run(&bringing, still, crate::STILL_EVERY)
        .map_err(|_| NotJoined::NoAddressCame)?;
    if up.succeeded {
        Ok(())
    } else {
        Err(why_not_joined(
            up.exit_code.unwrap_or(1),
            &up.complained,
            there_is_a_card,
        ))
    }
}

#[cfg(test)]
mod joining {
    use crate::Program;

    const SSID: &str = "Fritz!Box 7590";
    const SECRET: &str = "a-real-password";

    /// Every argument of the two commands, as one string, for asking whether
    /// something is in them.
    fn arguments_of(program: &Program) -> String {
        program.arguments().join(" ")
    }

    #[test]
    fn the_profile_is_made_temporary_so_no_key_is_ever_written() {
        // **The promise S3 makes to a person in as many words**: *your Wi-Fi
        // password is never written to this computer's disk.* Without this
        // flag `nmcli` writes a keyfile into
        // `/etc/NetworkManager/system-connections/`, which the image has — so
        // the flag is the whole of the promise, and this is the test that it
        // is there.
        let making = Program::AddingANetwork {
            ssid: SSID.to_owned(),
            password: SECRET.to_owned(),
        };
        assert!(
            arguments_of(&making).contains(super::TEMPORARY),
            "the profile is not temporary, so the password would be written down: {}",
            arguments_of(&making)
        );
    }

    #[test]
    fn an_open_network_is_given_no_security_at_all() {
        // An open network with `wpa-psk` and an empty key is refused by the
        // network manager — and refused in a way a person would read as their
        // password being wrong, on a network that asked for none.
        let open = Program::AddingANetwork {
            ssid: "A cafe, open".to_owned(),
            password: String::new(),
        };
        let said = arguments_of(&open);
        assert!(
            !said.contains("wifi-sec"),
            "an open network was given security: {said}"
        );
        assert!(
            said.contains("--temporary"),
            "and it is still temporary: {said}"
        );

        // The control: a locked one does get it, so the absence above is about
        // the password being empty and not about the arguments never carrying
        // security at all.
        let locked = Program::AddingANetwork {
            ssid: SSID.to_owned(),
            password: SECRET.to_owned(),
        };
        assert!(arguments_of(&locked).contains("wifi-sec.psk"));
    }

    #[test]
    fn the_password_is_in_the_arguments_and_in_nothing_else_this_program_can_say() {
        // **Where a secret may be and where it may not.** It reaches the one
        // command's arguments, because that is how the network manager is
        // told. It must reach nothing that is written down: `Program::path`
        // is what `sequence::noted` logs beside a complaint, and neither it
        // nor the bringing-up carries the key.
        let making = Program::AddingANetwork {
            ssid: SSID.to_owned(),
            password: SECRET.to_owned(),
        };
        let bringing = Program::BringingItUp {
            ssid: SSID.to_owned(),
        };

        assert!(
            arguments_of(&making).contains(SECRET),
            "the network manager is never told the password"
        );
        assert!(
            !making.path().contains(SECRET),
            "the password is in the path that gets logged"
        );
        assert!(
            !arguments_of(&bringing).contains(SECRET),
            "the second command carries the password, and it has no need of it"
        );
        assert!(!bringing.path().contains(SECRET));
    }

    #[test]
    fn neither_command_writes_to_a_disk_and_both_are_read() {
        // Answered deliberately rather than inherited: a temporary profile is
        // held in the daemon's memory and writes no keyfile, which is the
        // whole point of the flag. And what they say is read, because
        // `why_not_joined` classifies the failure from it.
        for program in [
            Program::AddingANetwork {
                ssid: SSID.to_owned(),
                password: SECRET.to_owned(),
            },
            Program::BringingItUp {
                ssid: SSID.to_owned(),
            },
        ] {
            assert!(!program.writes(), "{program:?} claims to write to a disk");
            assert!(
                program.is_read(),
                "{program:?} is not read, so a failure could not be told apart"
            );
        }
    }

    #[test]
    fn the_network_is_named_as_both_the_profile_and_the_ssid() {
        // A profile named something else would be a profile the second
        // command cannot find, and the failure would read as the network
        // never answering.
        let making = Program::AddingANetwork {
            ssid: SSID.to_owned(),
            password: SECRET.to_owned(),
        };
        let said = arguments_of(&making);
        assert!(said.contains(&format!("con-name {SSID}")), "{said}");
        assert!(said.contains(&format!("ssid {SSID}")), "{said}");
        assert!(
            arguments_of(&Program::BringingItUp {
                ssid: SSID.to_owned()
            })
            .contains(SSID)
        );
    }
}
