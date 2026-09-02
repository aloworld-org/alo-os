//! The only type that means *this may touch the disk* — and the paths it may
//! touch are the real ones.
//!
//! [`alo_capability::Authorised`] is the end of the journey ADR 0001 §5
//! describes: validated, permitted, approved if it changes anything, and
//! checked against the grants once more at the moment it would run. It is not
//! quite enough to open a file with, and the crate that produces it says so
//! itself — reach there is decided lexically, so a link inside a granted folder
//! can point outside it.
//!
//! A [`Touching`] is an `Authorised` whose paths have been made real and asked
//! about again. It has one constructor, it takes the authorisation **by
//! value**, and it is not `Clone` — so an executor that wants to open a file
//! holds one of these or it holds nothing, and the authority it was made from
//! is gone.
//!
//! # The three questions, in this order
//!
//! For every path a call names:
//!
//! 1. **do the grants permit the path as it was written?** Lexically, exactly
//!    as [`alo_capability`] already decided it. If not, that is the refusal,
//!    and the disk is never touched. This is what stops the file half becoming
//!    a way to ask whether a file exists: an agent that names a path nobody
//!    granted is told it was not granted and learns nothing about the machine;
//! 2. **where does it really lead?** [`crate::Resolving`], which is the only
//!    thing here that touches a filesystem;
//! 3. **do the grants permit *that*?** The same question as the first, asked
//!    about the answer to the second. A link out of a granted folder dies here,
//!    and it dies as a refusal by the grants, in their own words.
//!
//! Every path the call names goes through all three — not only the ones the
//! verb declared its grant is over. A verb that forgot to require a grant over
//! one of its paths is a mistake somebody will make one day, and it should not
//! be one that reaches a disk.
//!
//! # What it does not do
//!
//! It does not open anything, and it cannot close the gap between a path being
//! checked and being opened: a link swapped in afterwards, and a hard link,
//! which is a second real name for a file that also lives elsewhere and which
//! no amount of resolving will reveal. Both are in `docs/quirks.md`. Closing
//! them belongs to the code that opens the file, by holding on to what it
//! opened rather than by asking about the path twice.
//!
//! # Why a refusal needs the strings
//!
//! [`Touching::of`] takes the strings this machine reads, because two of the
//! three refusals above are worded here rather than by the grants, and
//! `alo_capability::Refused` carries words. The words it carries are the words
//! the person was shown — one rendering, in their language, which is then what
//! the record keeps. The alternative is an English record and a translated
//! screen, which is two accounts of one moment that nothing keeps equal.
//!
//! A `Strings` that was never given [`crate::file_words`] refuses just as
//! firmly and says so: the refusal carries the key, marked. What must never
//! depend on a string table is whether something is refused, and nothing here
//! does.

use std::collections::BTreeMap;

use alo_capability::{Ask, Authorised, Call, Grants, Refused, Value};
use alo_strings::{Filling, Strings};

use crate::real::Real;
use crate::resolving::Resolving;
use crate::words;

/// A file call that may touch the disk, and the real paths it may touch.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it: a thing that
/// means may-run and can be copied is a thing that can be run twice.
#[derive(Debug)]
pub struct Touching {
    /// What may run, and the authority it runs under.
    authorised: Authorised,
    /// Where each path argument really leads, by the argument that named it.
    real: BTreeMap<String, Real>,
}

impl Touching {
    /// Resolve everything this call names, and ask the grants about where it
    /// really leads.
    ///
    /// The moment is the authorisation's own ([`Authorised::at`]) rather than a
    /// fresh one: this is the last part of the same question the grants were
    /// asked when the call was authorised, and two moments would be two
    /// answers that could disagree.
    ///
    /// # Errors
    /// [`Refused`], carrying the call — the grants' own words when a path is
    /// not granted, and this crate's when the path is not there or leads
    /// somewhere the grants do not cover.
    pub fn of(
        authorised: Authorised,
        grants: &Grants,
        resolving: &dyn Resolving,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let at = authorised.at();
        let under = authorised.under().clone();
        let mut real = BTreeMap::new();
        for (argument, value) in authorised.call().values() {
            let Value::Path(given) = value else {
                continue;
            };
            // 1. As it was written. Refused here, nothing has been looked for.
            if let Err(why) = grants.permitting(&under, &Ask::Path(given.clone()), at) {
                return Err(Refused::not_granted(authorised.call().clone(), why));
            }
            // 2. Where it really leads.
            let resolved = match resolving.real(given) {
                Ok(resolved) => resolved,
                Err(why) => {
                    return Err(Refused::worded_elsewhere(
                        authorised.call().clone(),
                        why.said(strings),
                    ));
                }
            };
            // 3. And whether the grants cover that.
            if grants
                .permitting(&under, &Ask::Path(resolved.as_path().to_owned()), at)
                .is_err()
            {
                return Err(Refused::worded_elsewhere(
                    authorised.call().clone(),
                    strings.say(
                        &words::REALLY_LEADS_ELSEWHERE.key(),
                        &Filling::of("path", given.display().to_string())
                            .and("really", resolved.describe())
                            .and("who", under.as_str()),
                    ),
                ));
            }
            real.insert(argument.clone(), resolved);
        }
        Ok(Self { authorised, real })
    }

    /// What may run, and the authority it runs under.
    #[must_use]
    pub fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// What may run.
    #[must_use]
    pub fn call(&self) -> &Call {
        self.authorised.call()
    }

    /// The verb that may run.
    #[must_use]
    pub fn verb(&self) -> &str {
        self.authorised.verb()
    }

    /// Where this argument's path really leads.
    ///
    /// `None` for an argument that is not a path, and for one this verb does
    /// not take. Whatever opens a file opens **this**, and never the path the
    /// call arrived with — that path was checked, and this one is the one the
    /// check was about.
    #[must_use]
    pub fn real(&self, argument: &str) -> Option<&Real> {
        self.real.get(argument)
    }

    /// Every real path this call may touch, by the argument that named it.
    pub fn all(&self) -> impl Iterator<Item = (&str, &Real)> {
        self.real
            .iter()
            .map(|(argument, real)| (argument.as_str(), real))
    }

    /// Give back what ran, so it can be recorded.
    ///
    /// `alo-record` writes an entry from an [`Authorised`], and this is how the
    /// executor hands one over once the work is done. It consumes the token,
    /// because a thing that has run is not a thing that may run.
    #[must_use]
    pub fn into_authorised(self) -> Authorised {
        self.authorised
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::real::RealError;
    use crate::testing::{in_english, refusal};
    use crate::verbs::file_verbs;
    use alo_capability::{
        Approvals, Arg, Effect, Given, Grant, Grantee, NotAuthorised, Proposal, Reach, Requires,
        Takes, Verb, Verbs,
    };
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime};

    /// A fixed moment, so that expiry is arithmetic rather than a wait.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// How long the grants and the questions in these tests last.
    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    /// The agent the tests grant things to.
    fn files() -> Grantee {
        Grantee::named("@files")
    }

    /// Grants to `@files` over these folders, made at noon and lasting an hour.
    fn granting(folders: &[&str]) -> Grants {
        let mut grants = Grants::default();
        for folder in folders {
            grants.grant(
                Grant::checked(
                    "@files",
                    Reach::Folder(PathBuf::from(folder)),
                    noon(),
                    hour(),
                )
                .unwrap(),
            );
        }
        grants
    }

    /// A filesystem this test wrote down.
    ///
    /// The escape this whole crate exists to stop needs a link that leaves a
    /// granted folder, and making a real one needs a privilege a developer's
    /// machine may not have. So the *decision* is tested against a filesystem
    /// stated here, on every platform, and the one real implementation is
    /// tested against a real disk in [`crate::resolving`].
    struct Wherever {
        /// Where each path really leads. Anything not named is not there.
        leads: BTreeMap<PathBuf, PathBuf>,
        /// Every path anything asked about, in the order it was asked.
        asked: RefCell<Vec<PathBuf>>,
    }

    impl Wherever {
        /// A filesystem where these paths lead to these places.
        fn leading(pairs: &[(&str, &str)]) -> Self {
            Self {
                leads: pairs
                    .iter()
                    .map(|(from, to)| (PathBuf::from(from), PathBuf::from(to)))
                    .collect(),
                asked: RefCell::new(Vec::new()),
            }
        }

        /// A filesystem where every one of these paths is what it says it is.
        fn plain(paths: &[&str]) -> Self {
            Self::leading(&paths.iter().map(|path| (*path, *path)).collect::<Vec<_>>())
        }

        /// Everything anything asked about, in order.
        fn was_asked_about(&self) -> Vec<PathBuf> {
            self.asked.borrow().clone()
        }
    }

    impl Resolving for Wherever {
        fn real(&self, path: &Path) -> Result<Real, RealError> {
            self.asked.borrow_mut().push(path.to_owned());
            match self.leads.get(path) {
                Some(really) => Ok(Real::new(really.clone())),
                None => Err(RealError::Nothing {
                    path: path.display().to_string(),
                }),
            }
        }
    }

    /// A read of a folder that really is where it says it is.
    fn listing(folder: &str) -> Call {
        file_verbs()
            .unwrap()
            .call("list_folder", &[("folder", Given::text(folder))])
            .unwrap()
    }

    /// A read of a file that really is where it says it is.
    fn reading(file: &str) -> Call {
        file_verbs()
            .unwrap()
            .call("read_file", &[("file", Given::text(file))])
            .unwrap()
    }

    /// **The reason this crate exists.** A file inside a granted folder that is
    /// really a link to somewhere else is refused — and it is refused by the
    /// grants, which is what puts it in the record beside every other refusal.
    #[test]
    fn a_link_that_leads_outside_a_grant_is_refused() {
        let call = reading("/home/anna/Invoices/march.pdf");
        let grants = granting(&["/home/anna/Invoices"]);
        let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();

        // Lexically it is inside the grant, and the deciding crate said yes.
        assert!(call.permitted_by(&grants, &files(), noon()));

        let refused = Touching::of(
            authorised,
            &grants,
            &Wherever::leading(&[("/home/anna/Invoices/march.pdf", "/etc/shadow")]),
            &in_english(),
        )
        .unwrap_err();
        assert!(
            refusal(&refused).contains("/etc/shadow"),
            "{}",
            refusal(&refused)
        );
        assert!(
            refusal(&refused).contains("really leads to"),
            "{}",
            refusal(&refused)
        );
        assert!(
            refusal(&refused).contains("not where a link to it sits"),
            "{}",
            refusal(&refused)
        );
        // And it knows what it refused, because it is about to be recorded.
        assert_eq!(refused.call(), &call);
        // It is a refusal by the grants, worded here: the question *where does
        // this really lead* is one only this crate can ask, so this crate says
        // it, in the language the person reads, and the record keeps that
        // rendering rather than a second one.
        assert!(matches!(
            refused.why(),
            NotAuthorised::NotGrantedElsewhere(_)
        ));
    }

    /// What may be touched is the **real** path, not the one that arrived. A
    /// link that stays inside the grant is fine, and what comes back is where
    /// it leads.
    #[test]
    fn what_may_be_touched_is_the_real_path_and_not_the_one_that_was_asked_for() {
        let call = reading("/home/anna/Invoices/march.pdf");
        let grants = granting(&["/home/anna/Invoices"]);
        let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();

        let touching = Touching::of(
            authorised,
            &grants,
            &Wherever::leading(&[(
                "/home/anna/Invoices/march.pdf",
                "/home/anna/Invoices/2026/march.pdf",
            )]),
            &in_english(),
        )
        .unwrap();
        assert_eq!(touching.verb(), "read_file");
        assert_eq!(
            touching.real("file").map(Real::as_path),
            Some(Path::new("/home/anna/Invoices/2026/march.pdf"))
        );
        assert_eq!(touching.all().count(), 1);
        assert!(touching.real("folder").is_none());
        assert_eq!(touching.call(), &call);
        assert!(touching.into_authorised().from_approval().is_none());
    }

    /// **A path nobody granted is never looked for on the disk.** Otherwise a
    /// refusal would tell an agent whether a file it may not touch is there,
    /// and a capability model that answers that question has a side channel in
    /// it.
    ///
    /// The verb here requires a grant over one of its two paths, which the
    /// contract permits and the file verbs do not do — so this is also the test
    /// that every path a call names is asked about, not only the ones a verb
    /// remembered to name.
    #[test]
    fn a_path_that_is_not_granted_is_refused_before_the_disk_is_touched() {
        let forgetful = Verb::checked(
            "compare_folders",
            "say how two folders differ",
            Effect::Read,
            vec![
                Arg::taking("folder", "the folder to look at", Takes::Path),
                Arg::taking("with", "the folder to compare it with", Takes::Path),
            ],
            Requires::grants_over(["folder"]),
            "say how {folder} differs from {with}",
        )
        .unwrap();
        let mut verbs = Verbs::default();
        verbs.declare(forgetful).unwrap();
        let call = verbs
            .call(
                "compare_folders",
                &[
                    ("folder", Given::text("/home/anna/Invoices")),
                    ("with", Given::text("/home/anna/Taxes")),
                ],
            )
            .unwrap();

        let grants = granting(&["/home/anna/Invoices"]);
        // The deciding crate permits it: the verb only asked about one folder.
        let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();

        let wherever = Wherever::plain(&["/home/anna/Invoices", "/home/anna/Taxes"]);
        let refused = Touching::of(authorised, &grants, &wherever, &in_english()).unwrap_err();
        assert!(
            refusal(&refused).contains("/home/anna/Taxes"),
            "{}",
            refusal(&refused)
        );
        assert!(
            refusal(&refused).contains("has not been granted"),
            "{}",
            refusal(&refused)
        );
        assert_eq!(
            wherever.was_asked_about(),
            [PathBuf::from("/home/anna/Invoices")],
            "the disk was asked about a path nobody granted"
        );
    }

    /// The grants are asked again here, so a grant taken away between the
    /// authorisation and the file being opened still stops it — and the disk is
    /// not touched on the way to finding that out.
    #[test]
    fn a_grant_taken_away_after_the_authorisation_still_stops_it() {
        let call = listing("/home/anna/Invoices");
        let mut grants = granting(&["/home/anna/Invoices"]);
        let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();
        assert_eq!(grants.revoke_everything_for(&files()), 1);

        let wherever = Wherever::plain(&["/home/anna/Invoices"]);
        let refused = Touching::of(authorised, &grants, &wherever, &in_english()).unwrap_err();
        assert!(
            refusal(&refused).contains("has not been granted"),
            "{}",
            refusal(&refused)
        );
        assert!(wherever.was_asked_about().is_empty());
    }

    /// A path that is not there is a refusal rather than an empty answer: there
    /// is nothing to compare against a grant, so there is nothing to permit.
    #[test]
    fn a_path_that_is_not_there_is_refused_and_says_what_to_do() {
        let call = reading("/home/anna/Invoices/april.pdf");
        let grants = granting(&["/home/anna/Invoices"]);
        let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();

        let refused =
            Touching::of(authorised, &grants, &Wherever::plain(&[]), &in_english()).unwrap_err();
        assert!(
            refusal(&refused).contains("there is nothing at"),
            "{}",
            refusal(&refused)
        );
        assert!(
            refusal(&refused).contains("april.pdf"),
            "{}",
            refusal(&refused)
        );
        assert_eq!(refused.call(), &call);
    }

    /// **A change reaches here only after somebody approved it.** The journey
    /// is unchanged by this crate: propose, approve, redeem, and only then is
    /// there anything to resolve.
    #[test]
    fn a_change_reaches_the_disk_only_after_one_approval() {
        let verbs = file_verbs().unwrap();
        let call = verbs
            .call(
                "move_file",
                &[
                    ("file", Given::text("/home/anna/Invoices/march.pdf")),
                    ("into", Given::text("/home/anna/Archive")),
                ],
            )
            .unwrap();
        let grants = granting(&["/home/anna/Invoices", "/home/anna/Archive"]);

        // A change cannot take the read door, whatever it would touch.
        assert!(Authorised::read(&call, &files(), &grants, noon()).is_err());

        let mut approvals = Approvals::default();
        let id =
            approvals.propose(Proposal::checked(&call, &files(), &grants, noon(), hour()).unwrap());
        let approved = approvals.approve(id, noon()).unwrap();
        let authorised = approved.redeem(&grants, noon()).unwrap();

        let touching = Touching::of(
            authorised,
            &grants,
            &Wherever::plain(&["/home/anna/Invoices/march.pdf", "/home/anna/Archive"]),
            &in_english(),
        )
        .unwrap();
        assert_eq!(touching.verb(), "move_file");
        assert_eq!(touching.all().count(), 2);
        assert_eq!(touching.authorised().from_approval(), Some(id));
        assert_eq!(touching.authorised().against().len(), 2);
    }

    /// Half of a move is not a smaller move. A destination that really leads
    /// out of its grant stops the whole thing, and nothing has moved.
    #[test]
    fn a_destination_that_leads_outside_its_grant_stops_the_whole_move() {
        let verbs = file_verbs().unwrap();
        let call = verbs
            .call(
                "move_file",
                &[
                    ("file", Given::text("/home/anna/Invoices/march.pdf")),
                    ("into", Given::text("/home/anna/Archive")),
                ],
            )
            .unwrap();
        let grants = granting(&["/home/anna/Invoices", "/home/anna/Archive"]);
        let mut approvals = Approvals::default();
        let id =
            approvals.propose(Proposal::checked(&call, &files(), &grants, noon(), hour()).unwrap());
        let authorised = approvals
            .approve(id, noon())
            .unwrap()
            .redeem(&grants, noon())
            .unwrap();

        let refused = Touching::of(
            authorised,
            &grants,
            &Wherever::leading(&[
                (
                    "/home/anna/Invoices/march.pdf",
                    "/home/anna/Invoices/march.pdf",
                ),
                ("/home/anna/Archive", "/mnt/usb/Archive"),
            ]),
            &in_english(),
        )
        .unwrap_err();
        assert!(
            refusal(&refused).contains("/mnt/usb/Archive"),
            "{}",
            refusal(&refused)
        );
        assert_eq!(refused.call().verb(), "move_file");
    }

    /// A verb that names no path at all touches nothing, and says so — an empty
    /// answer rather than a refusal, because reaching nothing is a perfectly
    /// good thing to be allowed to do.
    #[test]
    fn a_call_that_names_no_path_touches_nothing() {
        let displays = Verb::checked(
            "list_displays",
            "list the displays attached to this machine",
            Effect::Read,
            vec![],
            Requires::nothing_because(
                "a display is not a path, a file or an application, and naming one reaches nothing",
            ),
            "list the displays",
        )
        .unwrap();
        let mut verbs = Verbs::default();
        verbs.declare(displays).unwrap();
        let call = verbs.call("list_displays", &[]).unwrap();
        let grants = Grants::default();
        let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();

        let wherever = Wherever::plain(&[]);
        let touching = Touching::of(authorised, &grants, &wherever, &in_english()).unwrap();
        assert_eq!(touching.all().count(), 0);
        assert!(touching.real("folder").is_none());
        assert!(wherever.was_asked_about().is_empty());
    }

    /// A name is not a place, so nothing tries to resolve one. Renaming
    /// resolves the file and stops.
    #[test]
    fn only_the_paths_are_resolved_and_a_name_is_not_one() {
        let verbs = file_verbs().unwrap();
        let call = verbs
            .call(
                "rename_file",
                &[
                    ("file", Given::text("/home/anna/Invoices/march.pdf")),
                    ("name", Given::text("march-2026.pdf")),
                ],
            )
            .unwrap();
        let grants = granting(&["/home/anna/Invoices"]);
        let mut approvals = Approvals::default();
        let id =
            approvals.propose(Proposal::checked(&call, &files(), &grants, noon(), hour()).unwrap());
        let authorised = approvals
            .approve(id, noon())
            .unwrap()
            .redeem(&grants, noon())
            .unwrap();

        let wherever = Wherever::plain(&["/home/anna/Invoices/march.pdf"]);
        let touching = Touching::of(authorised, &grants, &wherever, &in_english()).unwrap();
        assert_eq!(
            wherever.was_asked_about(),
            [PathBuf::from("/home/anna/Invoices/march.pdf")]
        );
        assert_eq!(touching.all().count(), 1);
        assert!(touching.real("name").is_none());
    }

    /// An expired grant permits nothing here either, and the moment asked
    /// about is the authorisation's own rather than a second reading of a
    /// clock.
    #[test]
    fn the_moment_asked_about_is_the_one_the_call_was_authorised_at() {
        let call = listing("/home/anna/Invoices");
        let grants = granting(&["/home/anna/Invoices"]);
        let authorised = Authorised::read(&call, &files(), &grants, noon()).unwrap();
        assert_eq!(authorised.at(), noon());

        let touching = Touching::of(
            authorised,
            &grants,
            &Wherever::plain(&["/home/anna/Invoices"]),
            &in_english(),
        )
        .unwrap();
        assert_eq!(touching.authorised().at(), noon());

        // An hour later there is no authorisation to resolve anything for.
        assert!(Authorised::read(&call, &files(), &grants, noon() + hour()).is_err());
    }
}
