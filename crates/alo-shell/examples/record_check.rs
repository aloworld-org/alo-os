//! The record window on a real nested compositor under a Wayland parent.
//!
//! Explicit developer fixture, never installed as anybody's session. It keeps a
//! record on a temporary disk the way the daemon keeps one — a verb turned
//! away, a question answered here, one that left for a provider, one a rule
//! held back, a model fetched by alo OS on its own, and forty more refusals so
//! the view has somewhere to move — and draws: nothing while the window is
//! closed, the account opened by hand, the view moved to the oldest entry, the
//! account read again after the machine kept one more entry, the same account
//! reached by asking the agent, a refusal once the record is gone, and nothing
//! again — in light and dark, read left to right and right to left, with the
//! egress indicator carried by every frame. A virtual output proves the drawing
//! path, not a panel: a certified machine has not seen this window.

/// Main-thread graphics initialisation is required by winit.
fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("record window check failed: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Native graphics are unavailable on non-Linux hosts.
#[cfg(not(target_os = "linux"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    Err("requires Linux and a Wayland parent".into())
}

/// Draw every standing of the window in both schemes and both directions.
#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use alo_appearance::{Scheme, TextScale};
    use alo_capability::Grantee;
    use alo_dock::Dock;
    use alo_egress::{Destination, EgressPolicy, Errand, Indicator, Leaving, OnItsOwn, Why};
    use alo_indicator::{Drew, Indicating};
    use alo_keeping::Writing;
    use alo_models::Region;
    use alo_record::Entry;
    use alo_recounting::Recounting;
    use alo_shell::{
        Cursor, EgressStatus, EgressStatusFrame, EgressStatusLook, Nested, RecordFrame, RecordKey,
        RecordLook, RecordShows, RecordWindow, WindowControlLabels,
    };
    use alo_strings::{Direction, Strings};
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant, SystemTime};

    let strings = Strings::of(alo_saying::everything_this_machine_can_say()?);
    let held = tempfile::tempdir()?;
    let kept_at = held.path().join("record.jsonl");
    let now = SystemTime::now();
    let files = Grantee::named("@files");
    let mail = Grantee::named("@mail");
    let a_provider = Destination::provider("alo", Region::Declared("the EU".to_owned()))
        .map_err(|error| format!("{error:?}"))?;

    let mut writing = Writing::opening(&kept_at).map_err(|error| format!("{error:?}"))?;
    let mut keep = |entry: Entry| writing.keep(&entry).map_err(|error| format!("{error:?}"));
    for which in 0..40 {
        keep(Entry::turned_away(
            "tidy_everything",
            "there is no verb called tidy_everything",
            &files,
            now + Duration::from_secs(which),
        ))?;
    }
    keep(Entry::answered_here(&mail, now + Duration::from_secs(60)))?;
    let departing = Indicator::default()
        .beginning(
            &EgressPolicy::Anywhere,
            Leaving::because(&mail, Why::Asking, a_provider.clone()),
            now + Duration::from_secs(61),
        )
        .map_err(|error| format!("{error:?}"))?;
    keep(Entry::left(&departing))?;
    let refused = Indicator::default()
        .beginning(
            &EgressPolicy::NothingLeaves,
            Leaving::because(&mail, Why::Asking, a_provider),
            now + Duration::from_secs(62),
        )
        .err()
        .ok_or("a machine set to let nothing leave let something leave")?;
    keep(Entry::held_back(
        &refused,
        &strings,
        now + Duration::from_secs(62),
    ))?;
    let underway = Indicator::default().beginning_on_its_own(
        OnItsOwn::for_(
            Errand::FetchingAModel,
            Destination::at("models.alo.example").map_err(|error| format!("{error:?}"))?,
        ),
        now + Duration::from_secs(63),
    );
    keep(Entry::left_on_its_own(&underway))?;
    drop(writing);
    std::fs::set_permissions(&kept_at, std::fs::Permissions::from_mode(0o600))?;
    let recounting = Recounting::kept_at(&kept_at);

    let mut nested = Nested::new("alo record window check", (1366, 768))?;
    nested.pump()?;
    let mut labels = WindowControlLabels::new()?;
    let dock = Dock::shipped();
    let mut status = EgressStatus::on_an_output();
    if let Drew::Refused(refused) =
        Indicating::nowhere().show(Some(&mut status), &Indicator::default())
    {
        return Err(refused.said(&strings).into_text().into());
    }

    let mut submitted = 0;
    let mut draw = |what: &str,
                    window: &RecordWindow,
                    nested: &mut Nested|
     -> Result<(), Box<dyn std::error::Error>> {
        for scheme in [Scheme::Light, Scheme::Dark] {
            for reading in [Direction::LeftToRight, Direction::RightToLeft] {
                let until = Instant::now() + Duration::from_millis(150);
                while Instant::now() < until {
                    nested.pump()?;
                    nested.submit_with_record(
                        &[],
                        &[],
                        &Cursor::Default,
                        None,
                        &mut labels,
                        EgressStatusFrame {
                            status: &status,
                            strings: &strings,
                            dock: &dock,
                            look: EgressStatusLook {
                                scheme,
                                scale: TextScale::ordinary(),
                                reading,
                            },
                        },
                        RecordFrame {
                            window,
                            strings: &strings,
                            look: RecordLook {
                                scheme,
                                scale: TextScale::ordinary(),
                                reading,
                            },
                        },
                        None,
                    )?;
                    submitted += 1;
                    std::thread::sleep(Duration::from_millis(16));
                }
            }
        }
        match window.shows() {
            RecordShows::Nothing => println!("Drew {what}: nothing"),
            RecordShows::Account {
                account,
                moved_past,
            } => println!(
                "Drew {what}: {} entries, {moved_past} newer out of view, the newest \"{}\"",
                account.how_many(),
                account
                    .told()
                    .last()
                    .map(|told| told.outcome().said(&strings).into_text())
                    .unwrap_or_default()
            ),
            RecordShows::Refusal(why) => println!("Drew {what}: \"{}\"", why.said(&strings).text()),
        }
        Ok(())
    };

    let mut window = RecordWindow::on_an_output();
    draw("a closed window", &window, &mut nested)?;
    println!("Opened by hand: {:?}", window.opened_by_hand(&recounting));
    draw("the account opened by hand", &window, &mut nested)?;
    window.pressed(RecordKey::Oldest, &recounting);
    draw("the view at the oldest entry", &window, &mut nested)?;

    let mut writing = Writing::opening(&kept_at).map_err(|error| format!("{error:?}"))?;
    writing
        .keep(&Entry::answered_here(&mail, now + Duration::from_secs(64)))
        .map_err(|error| format!("{error:?}"))?;
    drop(writing);
    println!(
        "Read again: {:?}",
        window.pressed(RecordKey::ReadAgain, &recounting)
    );
    draw("the account read again", &window, &mut nested)?;

    let mut asked = RecordWindow::on_an_output();
    println!(
        "Asked what it did: {:?}",
        asked.asked_what_it_did(&recounting)
    );
    let same = matches!(
        (window.shows(), asked.shows()),
        (RecordShows::Account { account: a, .. }, RecordShows::Account { account: b, .. }) if a == b
    );
    if !same {
        return Err("asking the agent reached a different account".into());
    }
    draw("the same account, asked of the agent", &asked, &mut nested)?;

    std::fs::remove_file(&kept_at)?;
    window.pressed(RecordKey::ReadAgain, &recounting);
    draw("a record that is gone", &window, &mut nested)?;
    window.pressed(RecordKey::Close, &recounting);
    draw("the window closed", &window, &mut nested)?;

    println!("{submitted} frames submitted");
    Ok(())
}
