//! The grants as they are written down, and read back believed or refused.
//!
//! TOML, because the two other files a machine says something about itself in —
//! `/etc/alo/agentd.toml` and `/etc/alo/accounts.toml` — are TOML, and the one
//! file on this machine that says *what an agent may reach* is the last one
//! that should need a notation of its own to read.
//!
//! ```toml
//! format = 2
//! next = 2
//!
//! [[grant]]
//! handle = 0
//! agent = "alo"
//! folder = "/home/ada/Invoices"
//! granted = 1760000000
//! expires = 1760003600
//!
//! [[grant]]
//! handle = 1
//! applicant = "org.gnome.Cheese"
//! facility = "camera"
//! granted = 1760000000
//! expires = 1760086400
//! ```
//!
//! `docs/contracts/grants-file.md` is the contract.
//!
//! # Two formats, and which one is written
//!
//! **Format 1** is a list of agents' grants over a folder, a file or an
//! application. **Format 2** is ADR 0040's: a grant may also be to an
//! application (`applicant`) and over a facility (`facility`). A format-1
//! reader refuses a whole file that says `format = 2`, which is the point of
//! the number — it would otherwise refuse the file anyway, over the first key
//! it did not know, and say something less useful.
//!
//! Both are read. **The lowest format that holds the list is written**: a
//! machine whose grants are all agents' over paths and applications writes
//! format 1, byte for byte what it wrote before, so rolling an update back does
//! not cost the person every folder they granted. Format 2 is written only
//! when a grant on the list needs it. And a file that says format 1 and names
//! `applicant` or `facility` is refused: it is a file no alo OS wrote.
//!
//! # Every grant is made again on the way in
//!
//! Nothing here deserialises an [`alo_capability::Grant`]. Each line is handed
//! to [`Grant::checked`], which is the same road a person's pick takes, so
//! every rule that crate enforces about a grant is enforced about a grant read
//! off a disk: no grant to the whole machine, no relative path, nothing with
//! `..` in it, nobody granted to, and **an end**. A file hand-edited into
//! `folder = "/"` is refused by the crate that owns that rule rather than by a
//! second opinion written here.
//!
//! # Moments are whole seconds, and the rounding reaches less
//!
//! `granted` and `expires` are seconds since 1970. A grant argued about in
//! nanoseconds is a grant nobody can read in the file it is kept in, and the
//! truncation is downwards on both — so a grant read back off the disk ends at
//! or before the moment it was made to end, never after. `alo-capability` makes
//! the same choice about the boundary itself: it falls on the side that reaches
//! less.
//!
//! # What is written is what is granted
//!
//! [`written`] takes the moment it is called at and writes down what is granted
//! **then**. An expired grant is not kept: it permits nothing, it is dropped
//! the moment the file is read anyway, and a file that accumulated them would
//! be a list a person opens to find things they cannot revoke because they are
//! already gone.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_capability::{Applicant, Facility, Grant, GrantId, Grantee, Grants, Held, Reach};
use serde::{Deserialize, Serialize};

use crate::refusing::NotRemembered;

/// The newest shape of grants file this alo OS writes and reads.
///
/// Format 2 adds grants to applications and over facilities (ADR 0040). The
/// older [`THE_FIRST_FORMAT`] is read as well, and written whenever the list
/// fits in it.
pub const THE_FORMAT: u32 = 2;

/// The first shape of grants file, which holds only agents' grants over a
/// folder, a file or an application.
pub const THE_FIRST_FORMAT: u32 = 1;

/// The file's shape, as serde sees it.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Kept {
    /// Which shape this file is in.
    format: u32,
    /// The handle the next grant made on this machine is given.
    #[serde(default)]
    next: u64,
    /// The grants, one table each.
    #[serde(rename = "grant", default)]
    grants: Vec<KeptGrant>,
}

/// One grant's table in the file.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KeptGrant {
    /// What the person revokes it by.
    handle: u64,
    /// The agent it is for, by the name the system knows it by — this or
    /// `applicant`, and exactly one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    agent: Option<String>,
    /// The application it is for, by its identifier. Format 2 only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    applicant: Option<String>,
    /// A folder and everything in it — one of the four, and exactly one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    folder: Option<PathBuf>,
    /// Exactly one file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    file: Option<PathBuf>,
    /// One installed application, by its identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    application: Option<String>,
    /// Something this machine has that is not a path, by
    /// `alo_capability::Facility::named`. Format 2 only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    facility: Option<String>,
    /// When it was made, in seconds since 1970.
    granted: u64,
    /// When it stops, in seconds since 1970.
    expires: u64,
}

impl KeptGrant {
    /// What this grant is over, or a refusal naming the handle.
    ///
    /// Exactly one of the four: none of them is a grant over nothing, and two
    /// of them is a grant a person believes covers something it does not.
    fn reach(&self) -> Result<Reach, NotRemembered> {
        let facility = match &self.facility {
            Some(name) => {
                Some(
                    Facility::by_name(name).ok_or_else(|| NotRemembered::NoSuchFacility {
                        handle: self.handle,
                        named: name.clone(),
                    })?,
                )
            }
            None => None,
        };
        let named = [
            self.folder.clone().map(Reach::Folder),
            self.file.clone().map(Reach::File),
            self.application.clone().map(Reach::Application),
            facility.map(Reach::Facility),
        ];
        let mut over = None;
        for one in named.into_iter().flatten() {
            if over.is_some() {
                return Err(NotRemembered::ReachesTwoThings {
                    handle: self.handle,
                });
            }
            over = Some(one);
        }
        over.ok_or(NotRemembered::ReachesNothing {
            handle: self.handle,
        })
    }

    /// Who this grant is for, or a refusal naming the handle.
    ///
    /// Exactly one of `agent` and `applicant`: a grant to nobody grants
    /// nothing, and a grant to both is a grant a person made to one of them.
    fn grantee(&self) -> Result<Grantee, NotRemembered> {
        match (&self.agent, &self.applicant) {
            (Some(agent), None) => Ok(Grantee::named(agent)),
            (None, Some(applicant)) => Ok(Applicant::named(applicant).grantee()),
            (None, None) => Err(NotRemembered::GrantedToNobody {
                handle: self.handle,
            }),
            (Some(_), Some(_)) => Err(NotRemembered::GrantedToTwo {
                handle: self.handle,
            }),
        }
    }

    /// The first key this line names that its file's format does not have.
    fn newer_than(&self, format: u32) -> Option<&'static str> {
        if format >= THE_FORMAT {
            return None;
        }
        if self.applicant.is_some() {
            Some("applicant")
        } else if self.facility.is_some() {
            Some("facility")
        } else {
            None
        }
    }

    /// This line as the grant it means, checked the way a pick is checked.
    fn as_a_grant(&self, format: u32) -> Result<Held, NotRemembered> {
        if let Some(key) = self.newer_than(format) {
            return Err(NotRemembered::NewerThanItsFormat {
                handle: self.handle,
                key,
            });
        }
        let grantee = self.grantee()?;
        let reach = self.reach()?;
        let granted = at_second(self.granted);
        // Saturating rather than checked, because a file saying a grant ended
        // before it began is a file `Grant::checked` refuses in its own words —
        // zero lasts no time — and that is the sentence worth reading.
        let lasting = Duration::from_secs(self.expires.saturating_sub(self.granted));
        let grant = Grant::checked_for(&grantee, reach, granted, lasting).map_err(|why| {
            NotRemembered::NotAGrant {
                handle: self.handle,
                said: why.word().says(),
            }
        })?;
        Ok(Held {
            id: GrantId::numbered(self.handle),
            grant,
        })
    }
}

/// The moment this many seconds after 1970.
fn at_second(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

/// The whole second this moment is in, or a refusal naming the grant.
///
/// Downwards, which is the rounding this file's header argues for: a grant read
/// back ends at or before the moment it was made to end.
fn as_seconds(moment: SystemTime, handle: GrantId) -> Result<u64, NotRemembered> {
    moment
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|since| since.as_secs())
        .map_err(|_before_the_epoch| NotRemembered::NotAMoment {
            handle: handle.as_u64(),
        })
}

/// The grants this text holds, believed only whole, with the expired ones
/// already gone.
///
/// `now` is what "expired" is measured against, and dropping them here is the
/// point rather than housekeeping: what comes back is the grants, not the
/// grants and a filter somebody has to remember to apply.
///
/// # Errors
///
/// [`NotRemembered::NotTheShape`] for text that is not this file,
/// [`NotRemembered::AnotherFormat`] for grants from an alo OS this is not —
/// answered before any grant is looked at — [`NotRemembered::NewerThanItsFormat`]
/// for a format-1 file naming a format-2 key, [`NotRemembered::GrantedToNobody`]
/// and [`NotRemembered::GrantedToTwo`] for a grant that is not for exactly one
/// agent or application, [`NotRemembered::ReachesNothing`],
/// [`NotRemembered::ReachesTwoThings`] and [`NotRemembered::NoSuchFacility`]
/// for a grant that is not over exactly one thing this machine has,
/// [`NotRemembered::NotAGrant`] for one `alo-capability` would not make, and
/// [`NotRemembered::NotOneList`] for handles that would collide.
pub fn read(text: &str, now: SystemTime) -> Result<Grants, NotRemembered> {
    let kept: Kept = toml::from_str(text).map_err(|why| NotRemembered::NotTheShape {
        why: why.to_string(),
    })?;
    if !(THE_FIRST_FORMAT..=THE_FORMAT).contains(&kept.format) {
        return Err(NotRemembered::AnotherFormat {
            format: kept.format,
        });
    }
    let mut held = Vec::with_capacity(kept.grants.len());
    for one in &kept.grants {
        held.push(one.as_a_grant(kept.format)?);
    }
    let mut grants = Grants::remembered(held, kept.next)?;
    // Before anybody has it. An expired grant that came back and was filtered
    // afterwards would be a grant on this machine for however long it took
    // somebody to remember the filter.
    let _expired = grants.forget_expired(now);
    Ok(grants)
}

/// What is granted at this moment, as the file that keeps it.
///
/// # Errors
///
/// [`NotRemembered::NotAMoment`] for a grant timed before 1970, and
/// [`NotRemembered::NotTheShape`] if the value would not serialise — which a
/// path that is not text is the one way to cause.
pub fn written(grants: &Grants, now: SystemTime) -> Result<String, NotRemembered> {
    let mut lines = Vec::new();
    for held in grants.active_at(now) {
        let granted = as_seconds(held.grant.granted_at, held.id)?;
        let expires = as_seconds(held.grant.expires, held.id)?;
        // A grant whose whole-second end is not after its whole-second start
        // ends inside the second this is being written in. It is over by the
        // time anybody reads the file, and writing it would produce a line the
        // reader refuses — a grant that lasts no time is not a grant.
        if expires <= granted {
            continue;
        }
        let (mut folder, mut file, mut application, mut facility) = (None, None, None, None);
        match &held.grant.reach {
            Reach::Folder(path) => folder = Some(path.clone()),
            Reach::File(path) => file = Some(path.clone()),
            Reach::Application(id) => application = Some(id.clone()),
            Reach::Facility(one) => facility = Some(one.named().to_owned()),
        }
        let name = held.grant.grantee.as_str().to_owned();
        let (agent, applicant) = if held.grant.grantee.is_an_application() {
            (None, Some(name))
        } else {
            (Some(name), None)
        };
        lines.push(KeptGrant {
            handle: held.id.as_u64(),
            agent,
            applicant,
            folder,
            file,
            application,
            facility,
            granted,
            expires,
        });
    }
    // The lowest format that holds what is written, so a machine with no
    // application's grant writes exactly what it wrote before format 2.
    let format = if lines
        .iter()
        .any(|line| line.newer_than(THE_FIRST_FORMAT).is_some())
    {
        THE_FORMAT
    } else {
        THE_FIRST_FORMAT
    };
    let kept = Kept {
        format,
        next: grants.next_handle().as_u64(),
        grants: lines,
    };
    toml::to_string(&kept).map_err(|why| NotRemembered::NotTheShape {
        why: why.to_string(),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{HERS, an_hour as hour, noon};
    use alo_capability::Ask;

    /// A machine on which one folder has been granted for an hour.
    fn one_folder_granted() -> Grants {
        crate::testing::granted_in("/home/ada/Invoices")
    }

    /// **What was written reads back and still permits what it permitted** —
    /// the round trip a restart is, in one file.
    #[test]
    fn what_was_written_reads_back_and_permits_what_it_did() {
        let text = written(&one_folder_granted(), noon()).unwrap();
        assert!(text.contains("format = 1"), "{text}");
        assert!(text.contains("[[grant]]"), "{text}");
        assert!(text.contains("/home/ada/Invoices"), "{text}");

        let read = read(&text, noon()).unwrap();
        let ask = Ask::path("/home/ada/Invoices/march.pdf");
        assert!(read.permits(&Grantee::named(HERS), &ask, noon()));
        assert!(!read.permits(&Grantee::named(HERS), &ask, noon() + hour()));
    }

    /// **A machine that has granted nothing keeps an empty list**, and reads it
    /// back as one rather than as a failure.
    #[test]
    fn a_machine_that_has_granted_nothing_round_trips() {
        let text = written(&Grants::default(), noon()).unwrap();
        let read = read(&text, noon()).unwrap();
        assert!(read.is_empty());
    }

    /// **An expired grant is gone when the list is read**, not read and then
    /// filtered: it is not on the list at all, which is the only version of
    /// this promise that cannot be forgotten by a later caller.
    #[test]
    fn an_expired_grant_is_not_on_the_list_that_comes_back() {
        let text = written(&one_folder_granted(), noon()).unwrap();
        let read = read(&text, noon() + hour()).unwrap();
        assert_eq!(read.len(), 0, "an expired grant came back on the list");
        assert!(!read.permits(
            &Grantee::named(HERS),
            &Ask::path("/home/ada/Invoices/march.pdf"),
            noon() + hour()
        ));
    }

    /// **And an expired grant is not written down either.** A person opening
    /// their list should not find things they cannot revoke because they are
    /// already over.
    #[test]
    fn an_expired_grant_is_not_written_down() {
        let text = written(&one_folder_granted(), noon() + hour()).unwrap();
        assert!(!text.contains("[[grant]]"), "{text}");
        // And the next handle survives, so the grant that is gone does not hand
        // its number to the next one made.
        assert!(text.contains("next = 1"), "{text}");
    }

    /// **The three reaches all survive the trip**, because a grant over an
    /// application is a grant a person made as much as a folder is.
    #[test]
    fn a_file_and_an_application_are_kept_as_themselves() {
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                HERS,
                Reach::File(PathBuf::from("/home/ada/Invoices/march.pdf")),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        grants.grant(
            Grant::checked(
                "@blender",
                Reach::Application("org.blender.Blender".to_owned()),
                noon(),
                hour(),
            )
            .unwrap(),
        );

        let read = read(&written(&grants, noon()).unwrap(), noon()).unwrap();
        assert!(read.permits(
            &Grantee::named(HERS),
            &Ask::path("/home/ada/Invoices/march.pdf"),
            noon()
        ));
        assert!(read.permits(
            &Grantee::named("@blender"),
            &Ask::application("org.blender.Blender"),
            noon()
        ));
    }

    /// **Text that is not this file is refused**, in the parser's words.
    #[test]
    fn what_is_not_a_list_of_grants_is_refused() {
        for wrong in ["=", "format = \"one\"", "just some text"] {
            assert!(
                matches!(read(wrong, noon()), Err(NotRemembered::NotTheShape { .. })),
                "`{wrong}` was read as a machine's grants"
            );
        }
    }

    /// **A key nobody declared is refused**, not skipped — the machine
    /// description's rule, because a line quietly passed over is a line
    /// somebody believes is doing something.
    #[test]
    fn a_key_nobody_declared_is_refused() {
        let with_extra = written(&one_folder_granted(), noon()).unwrap() + "\nforever = true\n";
        assert!(matches!(
            read(&with_extra, noon()),
            Err(NotRemembered::NotTheShape { .. })
        ));
    }

    /// **Grants from an alo OS this is not are refused as that**, before any
    /// grant in the file is looked at.
    #[test]
    fn grants_from_an_alo_os_this_is_not_are_refused() {
        for (format, refused) in [("format = 3", 3), ("format = 0", 0)] {
            let other = written(&one_folder_granted(), noon())
                .unwrap()
                .replace("format = 1", format);
            assert!(matches!(
                read(&other, noon()),
                Err(NotRemembered::AnotherFormat { format }) if format == refused
            ));
        }
        // And the refusal says which formats this alo OS does read.
        let said = NotRemembered::AnotherFormat { format: 3 }.to_string();
        assert!(said.contains("formats 1 to 2"), "{said}");
    }

    /// The camera granted to Cheese beside a folder granted to the agent.
    fn an_applications_grant_beside_an_agents() -> Grants {
        let mut grants = one_folder_granted();
        grants.grant(
            Grant::checked_for(
                &Applicant::named("org.gnome.Cheese").grantee(),
                Reach::Facility(Facility::Camera),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        grants
    }

    /// **An application's grant over a facility is kept as itself, in format
    /// 2**, and reads back allowing what it allowed and nothing an agent could
    /// use.
    #[test]
    fn an_applications_grant_over_a_facility_is_kept_in_format_two() {
        let text = written(&an_applications_grant_beside_an_agents(), noon()).unwrap();
        assert!(text.contains("format = 2"), "{text}");
        assert!(text.contains(r#"applicant = "org.gnome.Cheese""#), "{text}");
        assert!(text.contains(r#"facility = "camera""#), "{text}");

        let read = read(&text, noon()).unwrap();
        let cheese = Applicant::named("org.gnome.Cheese");
        let camera = Ask::facility(Facility::Camera);
        assert!(read.allowing(&cheese, &camera, noon()).is_ok());
        assert!(!read.permits(&Grantee::named("org.gnome.Cheese"), &camera, noon()));
        assert!(
            read.allowing(&cheese, &Ask::facility(Facility::Microphone), noon())
                .is_err()
        );
        assert!(read.permits(
            &Grantee::named(HERS),
            &Ask::path("/home/ada/Invoices/march.pdf"),
            noon()
        ));
        // Written again, it is the same file.
        assert_eq!(written(&read, noon()).unwrap(), text);
    }

    /// **A list that fits in format 1 is written as format 1**, so rolling an
    /// update back does not cost a person the folders they granted.
    #[test]
    fn a_list_with_no_applications_grant_is_written_as_it_was_before() {
        let text = written(&one_folder_granted(), noon()).unwrap();
        assert!(text.contains("format = 1"), "{text}");
        assert!(!text.contains("applicant"), "{text}");
        assert!(!text.contains("facility"), "{text}");

        // And once the application's grant has expired, the file goes back.
        let mut grants = one_folder_granted();
        grants.grant(
            Grant::checked_for(
                &Applicant::named("org.gnome.Cheese").grantee(),
                Reach::Facility(Facility::Camera),
                noon(),
                Duration::from_secs(60),
            )
            .unwrap(),
        );
        let later = noon() + Duration::from_secs(120);
        assert!(written(&grants, later).unwrap().contains("format = 1"));
    }

    /// **A format-1 file naming a format-2 key is refused**, because no alo OS
    /// wrote it.
    #[test]
    fn a_first_format_file_naming_a_later_key_is_refused() {
        let text = written(&an_applications_grant_beside_an_agents(), noon())
            .unwrap()
            .replace("format = 2", "format = 1");
        let refused = read(&text, noon()).unwrap_err();
        assert!(
            matches!(
                refused,
                NotRemembered::NewerThanItsFormat {
                    handle: 1,
                    key: "applicant"
                }
            ),
            "{refused}"
        );
    }

    /// **A grant for nobody, and a grant for both an agent and an
    /// application, are both refused.**
    #[test]
    fn a_grant_must_be_for_exactly_one_grantee() {
        let text = written(&an_applications_grant_beside_an_agents(), noon()).unwrap();
        let nobody = text.replace("applicant = \"org.gnome.Cheese\"\n", "");
        assert!(matches!(
            read(&nobody, noon()),
            Err(NotRemembered::GrantedToNobody { handle: 1 })
        ));
        let both = text.replace(
            "applicant = \"org.gnome.Cheese\"",
            "agent = \"@files\"\napplicant = \"org.gnome.Cheese\"",
        );
        assert!(matches!(
            read(&both, noon()),
            Err(NotRemembered::GrantedToTwo { handle: 1 })
        ));
    }

    /// **A facility this machine does not have is refused by name**, and the
    /// v1 ones are not on the list.
    #[test]
    fn a_facility_this_machine_does_not_have_is_refused() {
        let text = written(&an_applications_grant_beside_an_agents(), noon()).unwrap();
        for unknown in ["usb", "Camera", "/dev/video0", "remote-desktop"] {
            let edited = text.replace(
                "facility = \"camera\"",
                &format!("facility = \"{unknown}\""),
            );
            let refused = read(&edited, noon()).unwrap_err();
            assert!(
                matches!(&refused, NotRemembered::NoSuchFacility { handle: 1, named } if named == unknown),
                "{refused}"
            );
        }
    }

    /// **A hand-edit cannot give an agent the camera**: the file is read
    /// through `Grant::checked_for`, which refuses it in its own words.
    #[test]
    fn a_file_cannot_grant_an_agent_a_facility() {
        let text = written(&an_applications_grant_beside_an_agents(), noon())
            .unwrap()
            .replace(
                "applicant = \"org.gnome.Cheese\"",
                "agent = \"org.gnome.Cheese\"",
            );
        let refused = read(&text, noon()).unwrap_err();
        assert!(
            matches!(refused, NotRemembered::NotAGrant { handle: 1, .. }),
            "{refused}"
        );
        assert!(
            refused.to_string().contains("only to applications"),
            "{refused}"
        );
    }

    /// **A hand-edited grant to the whole machine is refused**, in the words of
    /// the crate that owns that rule — which is what building every line
    /// through `Grant::checked` is for. A file is the only road this can arrive
    /// by; no pick can make one.
    #[test]
    fn a_grant_to_the_whole_machine_is_refused_however_it_arrives() {
        let widened = written(&one_folder_granted(), noon())
            .unwrap()
            .replace("/home/ada/Invoices", "/");
        let refused = read(&widened, noon()).unwrap_err();
        assert!(
            matches!(refused, NotRemembered::NotAGrant { handle: 0, .. }),
            "{refused}"
        );
        assert!(
            refused.to_string().contains("the whole machine"),
            "{refused}"
        );

        // And the same for a path that could lead somewhere else.
        let sideways = written(&one_folder_granted(), noon())
            .unwrap()
            .replace("/home/ada/Invoices", "/home/ada/../root");
        assert!(matches!(
            read(&sideways, noon()),
            Err(NotRemembered::NotAGrant { .. })
        ));
    }

    /// **A grant that ends before it begins is refused** — as a grant that
    /// lasts no time, which is `alo-capability`'s own sentence about it.
    #[test]
    fn a_grant_that_ends_before_it_begins_is_refused() {
        let backwards = written(&one_folder_granted(), noon())
            .unwrap()
            .replace("expires = 1760003600", "expires = 1759000000");
        assert!(matches!(
            read(&backwards, noon()),
            Err(NotRemembered::NotAGrant { .. })
        ));
    }

    /// **A grant over nothing, and a grant over two things, are both
    /// refused** — the first has nothing to be a grant over and the second
    /// covers something a person did not pick.
    #[test]
    fn a_grant_must_be_over_exactly_one_thing() {
        let nothing = written(&one_folder_granted(), noon())
            .unwrap()
            .replace("folder = \"/home/ada/Invoices\"\n", "");
        assert!(matches!(
            read(&nothing, noon()),
            Err(NotRemembered::ReachesNothing { handle: 0 })
        ));

        let two = written(&one_folder_granted(), noon()).unwrap().replace(
            "folder = \"/home/ada/Invoices\"",
            "folder = \"/home/ada/Invoices\"\napplication = \"org.blender.Blender\"",
        );
        assert!(matches!(
            read(&two, noon()),
            Err(NotRemembered::ReachesTwoThings { handle: 0 })
        ));
    }

    /// **A file naming one handle twice is refused whole**, in
    /// `alo-capability`'s words: revoking the grant a person can see would take
    /// away whichever of the two came first.
    #[test]
    fn a_file_naming_one_handle_twice_is_refused() {
        let text = written(&one_folder_granted(), noon()).unwrap();
        let table = text
            .split("[[grant]]")
            .nth(1)
            .map(|rest| format!("[[grant]]{rest}"))
            .unwrap();
        let doubled = text + "\n" + &table;
        assert!(matches!(
            read(&doubled, noon()),
            Err(NotRemembered::NotOneList(_))
        ));
    }

    /// **The next handle is the file's, so a restart does not hand out one it
    /// has already used.** The `next` key is what carries it: a list rebuilt
    /// from what is left would begin again at the handle of a grant that
    /// expired.
    #[test]
    fn a_handle_is_not_handed_out_twice_across_a_restart() {
        let mut grants = one_folder_granted();
        let taxes = grants.grant(
            Grant::checked(
                HERS,
                Reach::Folder(PathBuf::from("/home/ada/Taxes")),
                noon(),
                Duration::from_secs(60),
            )
            .unwrap(),
        );
        // The second grant is over; the first is not. What comes back knows
        // both numbers are spent.
        let later = noon() + Duration::from_secs(120);
        let mut read = read(&written(&grants, later).unwrap(), later).unwrap();
        assert_eq!(read.len(), 1);

        let next = read.grant(
            Grant::checked(
                HERS,
                Reach::Folder(PathBuf::from("/home/ada/Photos")),
                later,
                hour(),
            )
            .unwrap(),
        );
        assert_ne!(
            next, taxes,
            "an expired grant's handle was handed out again"
        );
        assert_eq!(next.as_u64(), 2);
    }
}
