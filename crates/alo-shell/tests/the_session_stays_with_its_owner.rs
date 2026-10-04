//! **The boundaries [ADR 0087](../../../docs/decisions/0087-the-signed-in-session-is-kept-by-one-owner-and-notifications-ask-it-twice.md)
//! drew, held by reading the source.**
//!
//! The owner's ruling of 2026-10-04 put the authenticated session with one owner
//! and kept it out of everything downstream:
//!
//! > Retain the signed-in session, but keep it with the trusted session owner. **Do
//! > not pass the full session object throughout the drawing code.** … Drawing
//! > layer: receives presentation data approved for the current session, **not
//! > authentication authority.**
//!
//! Both halves fail silently. A sign-in that reduces the session to a number
//! compiles and runs, and so does a raster that learns who is signed in — the
//! first cost this repository its entire notification system without one failing
//! test, and the second would cost it the rule that makes a notification safe to
//! draw.
//!
//! # Why the source rather than behaviour
//!
//! Neither boundary has a wrong answer at runtime to assert against. A raster
//! handed a session would draw exactly the same pixels; a `Stood` carrying a uid
//! answered every question anybody asked it for four days. What changes is **what
//! a later change is able to do**, and that is a property of the shape.
//!
//! The same reason `the_recheck_has_a_caller.rs` and `the_camera_has_one_home.rs`
//! read source. This is the third of that family and the first about authority
//! rather than about state.
#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "an unreadable source file here is the failure this test reports, and a formatted \
              panic names the file it was reading"
)]

use std::path::Path;

/// One of this crate's source files, read from the repository.
fn source(named: &str) -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(named);
    std::fs::read_to_string(&at).unwrap_or_else(|why| panic!("{named} must be readable: {why}"))
}

/// **The sign-in boundary hands on the session's owner, never a number.**
///
/// `booting.rs` built a real `alo_accounts::Session` and kept `session.uid()`,
/// which is why nothing in this system could show a notification: `arrives` wants
/// a `Seat`, a seat wants a `Session`, and the only one ever built was dropped on
/// the line that made it.
///
/// **A uid is not a smaller session.** The owner's sentence is the general form —
/// *a UID identifies an account; it does not establish that its session is
/// currently unlocked* — so a boundary carrying the number is not a lesser version
/// of this one, it is a different fact that cannot answer the question a drawing
/// path has to ask.
#[test]
fn the_sign_in_boundary_carries_the_seat_and_not_a_uid() {
    let booting = source("booting.rs");

    let variant = booting
        .split_once("SomebodySignedIn {")
        .map(|(_, rest)| rest)
        .expect("booting.rs declares Stood::SomebodySignedIn");
    let body = variant
        .split_once("},")
        .map(|(it, _)| it)
        .expect("the variant's body ends");
    // **Code only.** The first version of this test failed on its own subject's
    // doc comment, which quotes `person: u32` to record what the boundary used to
    // carry — a guard that reads prose as code refuses the explanation of the
    // thing it is guarding. The same mistake `the_camera_has_one_home` made by
    // matching a parameter as if it were a field, one file over.
    let declaration: String = body
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        declaration.contains("seat: Seat<Notification>"),
        "Stood::SomebodySignedIn no longer carries the session's owner. ADR 0087 keeps the \
         authenticated session with one owner; a boundary that carries anything less cannot \
         answer whether this session is unlocked, and nothing downstream can show a \
         notification. Found: {declaration:?}"
    );
    assert!(
        !declaration.contains("person: u32"),
        "Stood::SomebodySignedIn carries a bare uid again. That is the shape this boundary had \
         until 2026-10-04 and it is the reason no notification was shown anywhere: {declaration:?}"
    );

    // **And the session is not reduced on the way through.** The variant could
    // carry a seat while the constructor still threw the session away and rebuilt
    // a lesser one, which would compile and would be the same loss.
    let code: String = booting
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        code.contains("Seat::opened(session)"),
        "booting.rs no longer builds the owner from the session the sign-in screen handed over. \
         `Seat::opened` is the one constructor for it, and this is the moment the session exists."
    );
    assert!(
        !code.contains("person: session.uid()"),
        "booting.rs reduces the session to its uid again at the sign-in boundary"
    );
}

/// **The drawing layer is given no authentication authority.**
///
/// ADR 0087: *the drawing layer receives presentation data approved for the
/// current session, not authentication authority.* `notification_raster` takes
/// `&[Shown]` and `alo_notifying::deciding::arrives` is the only thing that makes
/// a `Shown` — so *never while locked, shared or recorded* is a rule **the type
/// carries**, as that file's own note says, rather than one it implements and
/// could get wrong.
///
/// A raster that learned who was signed in would be able to decide for itself, and
/// a second place that decides is a second place that can decide differently. This
/// is the named signal from that record's last paragraph.
#[test]
fn the_drawing_layer_names_no_session_and_no_seat() {
    /// What a file in the drawing path may not name.
    const AUTHORITY: [&str; 4] = ["Session", "Seat", "SignedIn", "uid"];
    /// The files that paint what a notification says.
    const THE_DRAWING_PATH: [&str; 2] = ["notification_raster.rs", "notification_paint.rs"];

    for file in THE_DRAWING_PATH {
        let text = source(file);
        for (line, said) in text.lines().enumerate() {
            // Prose may discuss the rule — this file's own note does. Only code
            // naming the type is the crossing.
            let code = said.trim_start();
            if code.starts_with("//") {
                continue;
            }
            for named in AUTHORITY {
                assert!(
                    !code.contains(named),
                    "{file}:{} names `{named}` in code. ADR 0087 gives the drawing layer \
                     presentation data approved for this session and no authority of its own: a \
                     raster that can ask who is signed in is a second place that decides whether \
                     to draw, and the first is `deciding::arrives`. The line: {code:?}",
                    line + 1
                );
            }
        }
    }
}
