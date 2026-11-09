//! **Full access reaches everything the account can, and nothing it was not
//! given** (ADR 0149) — the yes and every refusal around it.

use super::*;
use crate::agent::Agent;
use crate::approvals::Approvals;
use crate::authorised::{Authorised, NotAuthorised};
use crate::facility::Facility;
use crate::proposal::Proposal;
use crate::test_calls::{
    archiving_march, files, granting, granting_both, hour, listing_invoices, noon,
};

/// A path nobody granted anything over.
fn taxes() -> Ask {
    Ask::path("/home/anna/Taxes/2024.pdf")
}

/// **At full access a path no grant covers is reached, and says it was full
/// access that reached it** — not a handle nobody made.
#[test]
fn full_access_reaches_a_path_no_grant_covers() {
    let grants = granting(&["/home/anna/Invoices"]);
    assert!(grants.permitted(&files(), &taxes(), noon()).is_err());

    let at = grants.at_full_access_for(&files());
    assert_eq!(at.full_access_for(), Some(&files()));
    assert_eq!(
        at.permitted(&files(), &taxes(), noon()),
        Ok(Permitted::ByFullAccess)
    );
    assert_eq!(
        at.permitted_where_it_really_is(&files(), std::path::Path::new("/etc/hostname"), noon()),
        Ok(Permitted::ByFullAccess)
    );
    assert!(at.permits(&files(), &taxes(), noon()));
    assert!(at.refusal(&files(), &taxes(), noon()).is_none());
    assert_eq!(Permitted::ByFullAccess.grant(), None);
}

/// **Full access adds and never removes**: where a grant covers the ask, the
/// grant is still the answer, so the record still names it.
#[test]
fn a_grant_that_covers_it_is_still_named_at_full_access() {
    let grants = granting(&["/home/anna/Invoices"]);
    let march = Ask::path("/home/anna/Invoices/march.pdf");
    let id = grants.permitting(&files(), &march, noon()).unwrap();
    let at = grants.at_full_access_for(&files());
    assert_eq!(
        at.permitted(&files(), &march, noon()),
        Ok(Permitted::ByAGrant(id))
    );
    assert_eq!(Permitted::ByAGrant(id).grant(), Some(id));
}

/// A session bound to one choice cannot inherit account-wide agent access.
#[test]
fn a_single_grant_view_remains_narrow_even_when_derived_from_full_access() {
    let grants = granting(&["/home/anna/Invoices", "/home/anna/Taxes"]);
    let invoice = Ask::path("/home/anna/Invoices/march.pdf");
    let chosen = grants.permitting(&files(), &invoice, noon()).unwrap();
    let mut at = grants.at_full_access_for(&files());
    let limited = at.limited_to(chosen);
    assert!(limited.permits(&files(), &invoice, noon()));
    assert!(limited.permitted(&files(), &taxes(), noon()).is_err());
    assert_eq!(limited.full_access_for(), None);
    assert!(at.revoke(chosen));
    let revoked = at.limited_to(chosen);
    assert!(revoked.permitted(&files(), &invoice, noon()).is_err());
    assert_eq!(revoked.full_access_for(), None);
}

/// **The kept list never carries it**: the copy is for one moment, the list it
/// was made from is unchanged, and nothing written down reads back at full
/// access.
#[test]
fn full_access_is_never_kept_on_the_list_or_written_down() {
    let grants = granting(&["/home/anna/Invoices"]);
    let at = grants.at_full_access_for(&files());
    assert_eq!(grants.full_access_for(), None);
    assert!(grants.permitted(&files(), &taxes(), noon()).is_err());

    let written = serde_json::to_string(&at).unwrap();
    assert!(!written.contains("full"), "{written}");
    let read: Grants = serde_json::from_str(&written).unwrap();
    assert_eq!(read.full_access_for(), None);
    assert!(read.permitted(&files(), &taxes(), noon()).is_err());
}

/// **It is one agent's.** Another agent on the same list — a paired machine's
/// principal, say — is not widened, and an application's grantee is never
/// given full access at all.
#[test]
fn full_access_is_for_one_agent_and_never_for_an_application() {
    let at = Grants::default().at_full_access_for(&files());
    let someone_else = Grantee::named("@studio");
    assert!(matches!(
        at.permitted(&someone_else, &taxes(), noon()),
        Err(NotGranted::Never { .. })
    ));

    let cheese = crate::grantee::Applicant::named("org.gnome.Cheese").grantee();
    let for_an_application = Grants::default().at_full_access_for(&cheese);
    assert_eq!(for_an_application.full_access_for(), None);
    assert!(
        for_an_application
            .permitted(&cheese, &taxes(), noon())
            .is_err()
    );
}

/// **What ADR 0149 does not name stays a grant of its own**: the camera, the
/// microphone and the screen; and the person's terminal stays their separate
/// choice (ADR 0111).
#[test]
fn full_access_does_not_reach_a_facility_or_a_terminal_nobody_allowed() {
    let at = Grants::default().at_full_access_for(&files());
    for facility in [Facility::Camera, Facility::Microphone, Facility::ScreenOnce] {
        assert!(
            at.permitted(&files(), &Ask::facility(facility), noon())
                .is_err(),
            "{facility:?}"
        );
    }
    let terminal = Ask::application("app.devsuite.Ptyxis");
    assert!(at.permitted(&files(), &terminal, noon()).is_err());
    assert_eq!(
        at.permitted_when(
            &files(),
            &terminal,
            noon(),
            ATerminalForTheAgent::MayOpenOne
        ),
        Ok(Permitted::ByFullAccess)
    );
    // And any other application is reached.
    assert_eq!(
        at.permitted(&files(), &Ask::application("org.gnome.TextEditor"), noon()),
        Ok(Permitted::ByFullAccess)
    );
}

/// **A machine whose person declined the agent is not widened** (ADR 0009):
/// the refusal is the declined machine's own, whatever the list says.
#[test]
fn a_machine_that_declined_the_agent_is_not_widened_by_full_access() {
    let machine = Agent::Declined(Grants::default().at_full_access_for(&files()));
    assert!(matches!(
        machine.permitting(&files(), &taxes(), noon()),
        Err(NotGranted::NoAgent { .. })
    ));
    assert!(!machine.permits(&files(), &taxes(), noon()));
}

/// **A read at full access runs, and what may run says full access allowed
/// it** — with no grant named, because none was.
#[test]
fn a_read_at_full_access_is_authorised_by_full_access() {
    let call = listing_invoices();
    let nothing = Grants::default();
    assert!(Authorised::read(&call, &files(), &nothing, noon()).is_err());

    let at = nothing.at_full_access_for(&files());
    let authorised = Authorised::read(&call, &files(), &at, noon()).unwrap();
    assert!(authorised.by_full_access());
    assert!(authorised.against().is_empty());

    // And under a grant, the grant is named and full access is not.
    let granted = granting(&["/home/anna/Invoices"]).at_full_access_for(&files());
    let authorised = Authorised::read(&call, &files(), &granted, noon()).unwrap();
    assert!(!authorised.by_full_access());
    assert_eq!(authorised.against().len(), 1);
}

/// **Full access does not make a change skip its approval**: a change offered
/// as a read is refused at full access exactly as at every other level.
#[test]
fn a_change_still_waits_at_full_access() {
    let at = Grants::default().at_full_access_for(&files());
    let refused = Authorised::read(&archiving_march(), &files(), &at, noon()).unwrap_err();
    assert!(matches!(refused.why(), NotAuthorised::ChangeWaits { .. }));
}

/// **Switching full access off takes effect immediately**: a change proposed
/// at full access and approved after the person moved to *ask me each time*,
/// with no grant covering it, is refused at execution — as a revoked grant is.
#[test]
fn switching_full_access_off_stops_an_approved_change_at_execution() {
    let nothing = Grants::default();
    let at = nothing.at_full_access_for(&files());
    let mut approvals = Approvals::default();
    let id = approvals
        .propose(Proposal::checked(&archiving_march(), &files(), &at, noon(), hour()).unwrap());
    let approved = approvals.approve(id, noon()).unwrap();

    let refused = approved.redeem(&nothing, noon()).unwrap_err();
    assert!(
        matches!(
            refused.why(),
            NotAuthorised::NotGranted(NotGranted::Never { .. })
        ),
        "{refused:?}"
    );
}

/// The control for the test above: the same approval, redeemed while full
/// access still stands, runs — and so does one covered by grants once full
/// access is gone, because grants work at every level.
#[test]
fn an_approved_change_runs_at_full_access_and_under_grants_without_it() {
    let at = Grants::default().at_full_access_for(&files());
    let mut approvals = Approvals::default();
    let id = approvals
        .propose(Proposal::checked(&archiving_march(), &files(), &at, noon(), hour()).unwrap());
    let running = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&at, noon())
        .unwrap();
    assert!(running.by_full_access());

    let both = granting_both();
    let id = approvals
        .propose(Proposal::checked(&archiving_march(), &files(), &at, noon(), hour()).unwrap());
    let running = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&both, noon())
        .unwrap();
    assert!(!running.by_full_access());
    assert_eq!(running.against().len(), 2);
}

/// **Without full access nothing changed**: a proposal over a folder nobody
/// granted is still refused before anybody is asked.
#[test]
fn without_full_access_a_proposal_outside_the_grants_is_still_refused() {
    let grants = granting(&["/home/anna/Invoices"]);
    assert!(Proposal::checked(&archiving_march(), &files(), &grants, noon(), hour()).is_err());
}
