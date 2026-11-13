//! **The verb: `publish_to_github_pages`** — a change whose sentence the
//! machine writes, allowed by one approval of that sentence.
//!
//! The owner's decision of 2026-11-13: *a typed action "publish a folder to
//! GitHub Pages" whose sentence the machine writes (destination +
//! cannot-be-undone), running git/gh underneath.* So a model names two things —
//! the folder and the GitHub account — and everything else is this crate's:
//! the sentence the person approves, the command that runs, and what they are
//! told afterwards.
//!
//! # What it reaches
//!
//! **The folder, with no grant over it** — the owner, 2026-11-13, asked
//! whether *ask me each time* should also need a folder grant: *"Approval
//! alone is enough."* So, as for any command at that level, one approval of
//! the sentence naming the folder and the address is the whole permission, and
//! the command runs as the person, bounded by their account's own permissions;
//! at full access it needs no approval (ADR 0149). The sealed box, a rule that
//! says nothing may leave, a paired machine, an empty folder and an address
//! GitHub could not give are still refused. **And GitHub**, counted on the
//! indicator as every command alo runs is (ADR 0148, as amended 2026-11-13).
//!
//! # It is not an `exec`
//!
//! Law 2 forbids a verb that runs what a model wrote. Nothing a model sends
//! reaches the command as a command: the folder is a checked path carried in
//! quotes, the account is letters, numbers and hyphens, and the command is
//! [`crate::the_line()`]'s, the same for every call.

use alo_capability::{
    Arg, Authorised, Call, Effect, Requires, Takes, Value, Verb, VerbError, Verbs, VerbsError,
};

use crate::account::{Account, THE_LONGEST};
use crate::not_published::NotPublished;
use crate::publication::Publication;
use crate::words;

/// The name the verb is asked for by.
pub const PUBLISH_TO_GITHUB_PAGES: &str = "publish_to_github_pages";

/// Why the verb could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// The verb, on a list of its own.
///
/// # Errors
/// [`Declaring`], which the declaration below cannot cause.
pub fn publishing_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the verb on `verbs`.
///
/// # Errors
/// [`Declaring::List`] for a list that already has the name.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    verbs.declare(publish_to_github_pages()?)?;
    Ok(())
}

/// The declaration.
fn publish_to_github_pages() -> Result<Verb, VerbError> {
    Verb::checked(
        PUBLISH_TO_GITHUB_PAGES,
        words::PURPOSE,
        Effect::Change,
        vec![
            Arg::taking(words::FOLDER, words::THE_FOLDER, Takes::Path),
            Arg::taking(words::ACCOUNT, words::THE_ACCOUNT, Takes::name(THE_LONGEST)),
        ],
        Requires::nothing_because(
            "the person approves this one publication by the sentence naming the folder and the \
             address, as they approve any command at that level, and at full access their own \
             account's permissions are the boundary (the owner, 2026-11-13: approval alone is \
             enough)",
        ),
        words::SENTENCE,
    )?
    .changing(words::CHANGES, Some(words::LEAVES))
}

/// **Whether a change of `verb` changes nothing on this computer** — so no
/// place to return to is taken before it, and *Put back* is never offered
/// beside it. True of a publication alone: it reads the folder and writes
/// nothing here (`crate::the_line`'s header), and what it changed is on GitHub,
/// which a snapshot of this computer cannot recall (ADR 0110 §2).
#[must_use]
pub fn changes_nothing_on_this_computer(verb: &str) -> bool {
    verb == PUBLISH_TO_GITHUB_PAGES
}

/// **What is asked of a publication before the person is asked about it**:
/// that the folder is a folder and the account a name GitHub gives — so the
/// sentence they approve names a folder and an address that can exist.
///
/// A call of any other verb is not this crate's, and passes.
///
/// # Errors
/// [`NotPublished::NotAFolder`] and [`NotPublished::NotAnAccount`].
pub fn checked_before_asking(call: &Call) -> Result<(), NotPublished> {
    if call.verb() != PUBLISH_TO_GITHUB_PAGES {
        return Ok(());
    }
    publication_in(call).map(|_| ())
}

/// The publication an authority carries: one of this verb, from an approval
/// or from full access.
///
/// # Errors
/// [`NotPublished::NotHere`] for an authority of another verb, or one that
/// came from neither; and [`checked_before_asking`]'s two refusals, asked
/// again here because the folder may have gone while the question stood.
pub fn approved(authorised: &Authorised) -> Result<Publication, NotPublished> {
    if authorised.verb() != PUBLISH_TO_GITHUB_PAGES {
        return Err(NotPublished::NotHere(format!(
            "{} is not {PUBLISH_TO_GITHUB_PAGES}, so it publishes nothing",
            authorised.verb()
        )));
    }
    if authorised.from_approval().is_none() && !authorised.without_the_ok() {
        return Err(NotPublished::NotHere(format!(
            "{PUBLISH_TO_GITHUB_PAGES} is a change, and this authority came from no approval \
             and not from full access"
        )));
    }
    publication_in(authorised.call())
}

/// The folder and the account a call names, each checked.
fn publication_in(call: &Call) -> Result<Publication, NotPublished> {
    let Some(Value::Path(folder)) = call.value(words::FOLDER) else {
        return Err(NotPublished::NotHere(format!(
            "{PUBLISH_TO_GITHUB_PAGES} arrived without a {}",
            words::FOLDER
        )));
    };
    let Some(Value::Name(account)) = call.value(words::ACCOUNT) else {
        return Err(NotPublished::NotHere(format!(
            "{PUBLISH_TO_GITHUB_PAGES} arrived without an {}",
            words::ACCOUNT
        )));
    };
    let account =
        Account::named(account).ok_or_else(|| NotPublished::NotAnAccount(account.clone()))?;
    if !folder.is_dir() {
        return Err(NotPublished::NotAFolder(folder.clone()));
    }
    // **Asked here rather than found out on GitHub**, so an empty folder never
    // makes a repository it then has nothing to put in.
    if !has_something_in_it(folder) {
        return Err(NotPublished::Empty);
    }
    Ok(Publication::of(folder, account))
}

/// Whether `folder` holds anything but `git`'s own folder — which is never
/// published, because `git` never sends a `.git`.
fn has_something_in_it(folder: &std::path::Path) -> bool {
    std::fs::read_dir(folder).is_ok_and(|entries| {
        entries
            .filter_map(Result::ok)
            .any(|entry| entry.file_name() != ".git")
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_capability::{Given, Grantee, Grants};
    use alo_strings::Strings;

    fn in_english() -> Strings {
        Strings::of(crate::words::publishing_words().unwrap())
    }

    fn a_call(folder: &str, account: &str) -> Call {
        publishing_verbs()
            .unwrap()
            .call(
                PUBLISH_TO_GITHUB_PAGES,
                &[
                    (words::FOLDER, Given::text(folder)),
                    (words::ACCOUNT, Given::text(account)),
                ],
            )
            .unwrap()
    }

    /// **One change, needing no folder grant — one approval of the sentence
    /// is enough** (the owner, 2026-11-13: *"Approval alone is enough"*) —
    /// that says what it changes and leaves alone.
    #[test]
    fn it_is_one_change_that_the_approval_alone_allows() {
        let verbs = publishing_verbs().unwrap();
        assert_eq!(verbs.len(), 1);
        let verb = verbs.of(PUBLISH_TO_GITHUB_PAGES).unwrap();
        assert_eq!(verb.effect(), Effect::Change);
        assert!(
            matches!(verb.requires(), Requires::Nothing { reason } if reason.contains("approves")),
            "{:?}",
            verb.requires()
        );
        assert!(verb.what_it_changes().is_some() && verb.what_it_leaves().is_some());
    }

    /// **The sentence the person approves names the folder and the address,
    /// and says it goes to the internet and Put back can't recall it** — ADR
    /// 0110 §2's *the surface says so at the moment of offering*.
    #[test]
    fn the_sentence_names_the_folder_the_address_and_that_it_cannot_be_recalled() {
        let call = a_call("/home/anna/site", "anna");
        assert!(call.waits_for_approval());
        assert_eq!(
            call.sentence(&in_english()).text(),
            "publish the folder /home/anna/site to the internet as your GitHub Pages site \
             anna.github.io, in place of what is there — anybody can read it, and Put back can't \
             recall it"
        );
    }

    /// **The refusal path**: a folder that is not one, and an account GitHub
    /// would not give, are refused before anybody is asked; a call of
    /// another verb passes untouched.
    #[test]
    fn what_is_not_a_folder_or_not_an_account_is_refused_before_anybody_is_asked() {
        let here = tempfile::tempdir().unwrap();
        let folder = here.path().display().to_string();
        let file = here.path().join("index.html");
        std::fs::write(&file, "<p>hello</p>").unwrap();

        assert_eq!(checked_before_asking(&a_call(&folder, "anna")), Ok(()));
        assert_eq!(
            checked_before_asking(&a_call(&file.display().to_string(), "anna")),
            Err(NotPublished::NotAFolder(file.clone()))
        );
        assert_eq!(
            checked_before_asking(&a_call(&folder, "anna smith")),
            Err(NotPublished::NotAnAccount("anna smith".to_owned()))
        );
        assert_eq!(
            checked_before_asking(&a_call(&folder, "$(reboot)")),
            Err(NotPublished::NotAnAccount("$(reboot)".to_owned()))
        );
        // **An empty folder, or one holding only `git`'s own**, is refused
        // before a repository is made for it.
        let empty = tempfile::tempdir().unwrap();
        std::fs::create_dir(empty.path().join(".git")).unwrap();
        assert_eq!(
            checked_before_asking(&a_call(&empty.path().display().to_string(), "anna")),
            Err(NotPublished::Empty)
        );
    }

    /// **Only an approval or full access makes a publication**, and only of
    /// this verb.
    #[test]
    fn only_an_approval_or_full_access_makes_a_publication() {
        let here = tempfile::tempdir().unwrap();
        std::fs::write(here.path().join("index.html"), "<p>hello</p>").unwrap();
        let call = a_call(&here.path().display().to_string(), "Anna");
        let agent = Grantee::named("@alo");
        let now = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_800_000_000);
        let at = Grants::default().at_full_access_for(&agent);
        let authorised = Authorised::at_full_access(&call, &agent, &at, now).unwrap();
        let publication = approved(&authorised).unwrap();
        assert_eq!(publication.account().name(), "anna");
        assert_eq!(publication.folder(), here.path());

        // Without full access no authority for it is made at all, and a read
        // never is.
        assert!(Authorised::at_full_access(&call, &agent, &Grants::default(), now).is_err());
        assert!(Authorised::read(&call, &agent, &at, now).is_err());
    }

    #[test]
    fn a_name_already_taken_is_not_replaced() {
        let mut verbs = publishing_verbs().unwrap();
        assert!(matches!(declare_into(&mut verbs), Err(Declaring::List(_))));
    }
}
