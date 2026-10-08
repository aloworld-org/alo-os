//! **The recovery recheck is called by the draw**, read from the source this
//! crate ships because nothing can run that draw.
//!
//! This repository's recurring fault is a rule that is written, tested and never
//! asked. `crate::canvas_never_lost` held *a frame keeps a usable part of its name
//! outside every fixed control* for weeks with unit tests, integration tests and
//! **no caller on any path a person could take**. Its detector and its mover —
//! `frames_the_controls_now_hide` and `bring_back_frames_the_controls_hide` — then
//! repeated it one layer up: both tested, both called by tests alone, so on a
//! running machine the Dock could grow over a frame and nothing noticed.
//!
//! # Why this is read rather than run
//!
//! The caller is `direct_desktop`'s `desktop`, and it **opens a graphics card**:
//! `with_active_device`, `discover_atomic_output`, an atomic commit. There is no
//! fixture for it and there is not going to be one — `direct_desktop_tests` builds
//! its pictures by calling `frame_pictures` directly for exactly this reason, and
//! the integration fixtures reach `Server::render`, which is a different path and
//! never touches a desktop frame. So every test that can run drives the recheck by
//! handing the bounds over itself, **as a draw would**, and not one of them would
//! fail if the draw stopped calling it.
//!
//! That is the hole this file closes, and it is the only tool that fits: the
//! question *is this symbol called from production* is answerable by reading, and
//! answerable by nothing else here.
//!
//! # The test proves its own reading
//!
//! An assertion that a name appears in a file passes just as well when the name
//! appears **in a comment**, and this file's subject is a comment-heavy one — the
//! call site carries a paragraph that names the function it calls. An empty or
//! careless search would therefore report success about prose. So the stripping is
//! measured rather than trusted: the name is counted before and after comments are
//! removed, and **the count must fall**. If it does not, the stripper did nothing
//! and every other assertion here is about raw text.
#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "an unexpected None or Err here is the failure this test reports, and a \
              formatted panic is how it names which control it was reading"
)]

use std::path::{Path, PathBuf};

/// The draw that lays the fixed controls out and must act on them moving.
const THE_DRAW: &str = "direct_desktop.rs";

/// Where the draw records the controls it laid out.
const RECORDS_THEM: &str = "the_fixed_controls_were_drawn";

/// What it must call once they have moved.
const ACTS_ON_THEM: &str = "bring_back_frames_the_moved_controls_hide";

/// Where the file holding the set lives, so its fields can be read.
const HOLDS_THE_SET: &str = "canvas_fixed_controls.rs";

/// The value every control must be read from.
///
/// Narrower than *not a literal* on purpose. `what_is_leaving: self.the_band()`
/// is not a literal and is still wrong: it would hand over a rectangle the draw is
/// storing rather than the one this frame laid out, which is the staleness the
/// whole module exists to prevent. If a control ever stops coming straight off the
/// pictures, this guard refuses it — correctly — and the repair is to say where it
/// does come from, never to loosen the pattern.
const FROM_THIS_FRAME: &str = "pictures.";

/// This crate's source directory.
fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// One file with its comment lines and trailing comments taken out.
///
/// Line comments only, which is all this file needs and all it should claim:
/// `direct_desktop.rs` has no block comments and no string literal containing
/// either symbol. A general Rust stripper here would be a second, untested
/// implementation of something the assertions below check directly.
fn the_code_of(written: &str) -> String {
    written
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| match line.split_once("//") {
            Some((before, _)) => before,
            None => line,
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// How many times `name` appears in `text`.
fn how_often(text: &str, name: &str) -> usize {
    text.matches(name).count()
}

/// Every `pub` field of a struct in this crate's source, in the order written.
///
/// **This is what makes the guard below grow with the set rather than with
/// somebody's memory.** The list it replaced was three strings maintained by hand,
/// which is the shape of the fault the guard exists to catch: it covered what was
/// known when it was written, and the status area joined the set on 2026-10-02
/// without joining the guard.
fn the_fields_of(struct_named: &str) -> Vec<String> {
    let at = src().join(HOLDS_THE_SET);
    let written = std::fs::read_to_string(&at)
        .expect("canvas_fixed_controls.rs holds the set and must be readable");
    let code = the_code_of(&written);
    let opens = format!("pub struct {struct_named} {{");
    let from = code
        .find(&opens)
        .unwrap_or_else(|| panic!("{struct_named} is not declared in {HOLDS_THE_SET}"));
    let body = code.get(from..).unwrap_or_default();
    // The first line that is a lone `}` closes it. Every field here is one line
    // and none opens a brace, so this needs no nesting count — and a field that
    // did would show up as a missing field rather than as a silent pass, because
    // the guard refuses a set it could not read.
    let upto = body.find("\n}").unwrap_or(body.len());
    body.get(..upto)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("pub ")
                .and_then(|rest| rest.split_once(':'))
                .map(|(named, _)| named.trim().to_owned())
        })
        .collect()
}

/// One string with every run of whitespace reduced to a single space.
///
/// So that an assertion about *what this field is read from* survives `rustfmt`
/// deciding to break the line differently. Without it the guard below would be a
/// test about formatting wearing the name of a test about wiring.
fn one_space(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// **The draw acts on the controls having moved, in code rather than in a
/// comment.**
#[test]
fn the_draw_brings_back_frames_the_moved_controls_hide() {
    let at = src().join(THE_DRAW);
    // `expect` rather than a formatted `panic!` because there is nothing to
    // interpolate here — the path is a constant this file already names. The
    // guard below does use one, which is what the file-level `expect` of
    // `clippy::panic` is for; this comment claimed that lint forbade it outright
    // until the guard needed it.
    let written = std::fs::read_to_string(&at)
        .expect("direct_desktop.rs is this crate's desktop draw and must be readable");
    let code = the_code_of(&written);

    // The anchor first: a search that cannot find what is certainly there is
    // measuring its own mistake rather than the file. If the draw has stopped
    // recording the controls at all then this file is asking the wrong question
    // and must say so, rather than reporting the absence of the other symbol.
    assert!(
        how_often(&code, RECORDS_THEM) > 0,
        "{THE_DRAW} does not call {RECORDS_THEM}, so either the draw no longer \
         records where the fixed controls are — which breaks the rule outright — \
         or this test is reading the wrong file"
    );

    // The stripper is measured, not trusted. The call site names the function it
    // calls in its own comment, so a stripper that did nothing would make the
    // assertion below pass on prose.
    let in_prose_too = how_often(&written, ACTS_ON_THEM);
    let in_code = how_often(&code, ACTS_ON_THEM);
    assert!(
        in_code < in_prose_too,
        "comments were not removed: {ACTS_ON_THEM} appears {in_prose_too} times in \
         {THE_DRAW} and {in_code} times after stripping, so this test cannot tell \
         a call from a sentence about one"
    );

    assert!(
        in_code > 0,
        "{THE_DRAW} never calls {ACTS_ON_THEM}, so the controls are recorded on \
         every frame and a frame they have moved over is never brought back. The \
         detector and the mover are both written and both tested; without this \
         call they are reachable from tests alone, which is the fault this whole \
         module exists because of"
    );
}

/// **The controls handed over are the ones this frame laid out, not constants.**
///
/// # Measured, and the measurement is why this test exists
///
/// Substituting `Rectangle::default()` for the panel's reserved column at the
/// wiring site — one line in the draw — left **seventeen tests passing**: the
/// source test above, all nine in the dragging module, and seven in
/// `desktop_raster`. *The panel's column is compared against* and *an empty
/// rectangle is compared against* were indistinguishable to every test in this
/// crate.
///
/// The reason is the same one the panel lane found on 2026-10-02 by mutating its
/// own `panel_is_revealed` flag in three places and watching two of the three
/// survive the whole suite: **every fixture supplies its own rectangles.** The
/// tests that drive the rule call `the_fixed_controls_were_drawn` directly and
/// invent the bounds, correctly and deliberately — there is no draw in a headless
/// fixture. So they prove the rule, the trigger and the recovery, and they are
/// *silent by construction* about whether the draw gives the rule the real
/// rectangles or made-up ones. A whole suite of honest fixtures adds up to a
/// wiring nobody has watched do anything.
///
/// # Why reading is the only tool again
///
/// The same reason as above, one step further in: the values come from `pictures`,
/// which exists only inside a draw that opens a graphics card. Nothing can observe
/// them. What *can* be checked is that the draw reads them from the pictures it
/// just laid out rather than from a literal, and that is a fact about the text.
///
/// # The set is read from the struct, not from a list kept here
///
/// **This guard was three strings maintained by hand until 2026-10-02, and it had
/// the fault it was written to catch.** It named the Dock's band and the panel's
/// column; the status area joined the set the same day and would have landed
/// unguarded, because `status_area: Rectangle::default()` satisfies a guard that
/// never heard of `status_area`. A guard covering what was known when it was
/// written is the fault, not a lesser version of it — and the laptop lane found it
/// in mine within the hour, having just had its own 950 tests survive the field
/// being removed from the rule.
///
/// So the controls are now **whatever `FixedControlsDrawn` declares**, read from
/// source. `FixedControlsDrawn`'s own header says a control joins *by being
/// pushed* and that a caller who adds a surface to the picture and not to this call
/// has a compiler error rather than a silent hole. That is half true, and the half
/// that is missing is this guard's whole subject: **the compiler enforces that the
/// field is set, never that it is set from the pictures.**
///
/// # What the compiler already catches, and the one thing it does not
///
/// Measured rather than argued, and the first draft of this note claimed the wrong
/// win. Three mutations:
///
/// ```text
/// A  status_area: None                       test FAILED, naming status_area
/// B  a fourth field added, never set         error[E0063]: missing field …
/// C  a fourth field added AND set to None    test FAILED, naming a_fourth_control
/// ```
///
/// *The field in mutation A is called `what_is_leaving` since 2026-10-04. The table
/// keeps the name it was measured under, because a measurement with a value swapped
/// inside it is silently re-dated — and the mutations were run on 2026-10-02, when
/// the field was `status_area`. The rename is in that field's own note.*
///
/// **B is the compiler's, not this guard's.** A struct literal missing a field does
/// not build, which is exactly what `FixedControlsDrawn`'s header promises. This
/// note first cited B as the case a hand-written list could not reach; it is the
/// case *nothing needs to reach*, and claiming it would have been this guard taking
/// credit for `rustc`.
///
/// **C is the gap, and it is the only one.** A control added to the set and wired to
/// a constant **compiles** — the compiler is satisfied that the field is set and has
/// no opinion about where from — and a guard naming its controls by hand has never
/// heard of it. That is precisely how `status_area` would have landed on 2026-10-02:
/// the compiler was happy, three tests named two fields, and the third was free.
///
/// So the compiler enforces *a control is handed over*, and this enforces *what is
/// handed over came from this frame*. Neither is the other, and only together do
/// they make the header's promise true.
///
/// # The compiler does not merely miss C — it is what produces C
///
/// **The next reader's instinct is *the compiler covers this*, and they are nearly
/// right, which is the most dangerous distance from correct.** The desktop lane
/// supplied the argument that settles it, and it is stronger than *a case `rustc`
/// cannot see*.
///
/// `FixedControlsDrawn` was constructed in **seven** places on 2026-10-02 — six test
/// fixtures and the draw — with **no spread and no `Default::default()` at any of
/// them**, so `E0063` does fire everywhere.
///
/// The **count** is a figure about a day: fixtures come and go and the ratio moves
/// with them, so it is dated here rather than guarded, because a test that failed
/// when somebody added a fixture would be a maintained count wearing a check's
/// clothes.
///
/// **What is worth asserting is what keeps `E0063` able to fire at all**, and
/// finding it took one more correction. This note first called *no spread in the
/// draw* the invariant. It is not: `..Default::default()` does not compile today
/// because `FixedControlsDrawn` does not derive `Default` — measured, `E0277: the
/// trait bound FixedControlsDrawn: Default is not satisfied` — so that hole is held
/// shut by the type system and the assertion for it is a belt rather than the gate.
///
/// **The gate is the derive.** Add `Default` to that struct and the spread becomes
/// writable, a control added to the set arrives as an empty rectangle, there is no
/// compile error because every field is accounted for, and this guard has nothing to
/// read. One derive, in another file, and both halves of the promise go at once. So
/// the test asserts the derive's absence, and the spread check sits beside it for the
/// form nobody would reach for first.
///
/// So adding a control hands an author seven compile errors that ask one question —
/// *supply a value* — and the answers are not equally available. At the draw the
/// honest answer is the rectangle this frame laid out. **At the six fixtures there
/// is nothing laid out**, because there is no draw in a headless fixture, so the
/// cheapest answer is a rectangle somebody typed. Six easy wrong answers and one
/// hard right one, in a single editing session, with nothing distinguishing them.
///
/// That is why this crate's suite came to be *silent by construction* about whether
/// the draw hands the rule anything real: not six independent lapses, but **one
/// prompt answered six times**, with the compiler supplying the prompt. It is the
/// same shape as *the careful version of a check is written in the same idiom as the
/// careless one*, one layer up, with `rustc` supplying the idiom.
///
/// The six fixtures are **right** to invent their rectangles — they are testing the
/// rule, and the rule must hold for any rectangle. Nothing here asks them to change.
/// What it says is that their correctness is not evidence about the seventh site, and
/// only this guard looks there.
///
/// *Counted here rather than quoted: `grep -rn "FixedControlsDrawn {" | grep -v "pub
/// struct"`. The figure was first reported as eight by counting grep's lines, which
/// include the struct's own declaration — a declaration is not a construction, and
/// `pub struct FixedControlsDrawn {` matches the same pattern.*
/// # The draw has two sites since 2026-10-08, and that moved this guard
///
/// `more-than-one-display-plan.md` task 9 made the draw record **every**
/// display's controls rather than only the one its own loop holds, so there are
/// now two `FixedControlsDrawn` in `direct_desktop.rs`: one per other display,
/// and one for the display the draw itself is on. The count above is dated and
/// deliberately unguarded, so it is not the point. **What moved is the guard.**
///
/// It read `code.find`, which was the same thing as *the site* while there was
/// one. With two it answers about whichever is written first and is silent about
/// the other — and the whole argument of this file is that the draw's wiring is
/// watched by nothing else. A second production site nothing looks at is this
/// file's own subject, reappearing inside this file's own apparatus, three days
/// after it was written. So both halves now iterate every literal.
///
/// The guard asks that each value is read from a `pictures` binding. The
/// per-display loop's binding is named `its_pictures` for that reason, and the
/// draw says so where somebody would rename it.
#[test]
fn the_draw_hands_over_the_controls_it_laid_out() {
    let at = src().join(THE_DRAW);
    let written = std::fs::read_to_string(&at)
        .expect("direct_desktop.rs is this crate's desktop draw and must be readable");
    let code = one_space(&the_code_of(&written));

    // **The compiler's half of the promise, which a spread would silently end.**
    // `E0063` is what forces a new control to be handed over at all, and it only
    // fires where every field is named. `..Default::default()` here would let a
    // control join the set and arrive as a default rectangle with nothing refusing
    // — and this guard would not catch it either, because a field absent from the
    // literal cannot be read from the pictures or from anything else.
    // Scoped to the literal rather than the file, because a spread is written
    // *after* the fields — `{ dock_band: …, ..Default::default() }` — so looking for
    // `FixedControlsDrawn { ..` would miss every real one. That was this
    // assertion's first form.
    // **Every construction site in the draw, not the first of them.** Task 9 of
    // `more-than-one-display-plan.md` made there be two: one inside the
    // per-display loop, for each display that loop does not itself hold, and one
    // for the display it does. Until then `code.find` and *the* site were the
    // same thing. They are not any more, and a guard that answered about
    // whichever came first in the file would have said nothing about the other
    // — while the note above this test says this guard is the only thing that
    // looks at the draw at all. **So the site it skipped would have been watched
    // by nothing**, which is this file's own subject arriving in its own
    // apparatus. Both halves below now ask every site.
    let opens = "FixedControlsDrawn {";
    let literals: Vec<&str> = code
        .match_indices(opens)
        .map(|(at, _)| {
            let rest = code.get(at + opens.len()..).unwrap_or_default();
            rest.get(..rest.find('}').unwrap_or(rest.len()))
                .unwrap_or("")
        })
        .collect();
    assert!(
        !literals.is_empty(),
        "{THE_DRAW} does not construct a FixedControlsDrawn at all"
    );
    for literal in &literals {
        assert!(
            !literal.contains(".."),
            "{THE_DRAW} spreads into a `FixedControlsDrawn` — `{}` — so a control added \
             to the set would arrive from somewhere else instead of being a compile \
             error. A field absent from the literal cannot be read from the pictures \
             either, so the set would lose both halves of its promise at once",
            literal.trim()
        );
    }

    // **And the gate the above rests on.** `..Default::default()` is the form
    // somebody would actually reach for, and it does not compile today for one
    // reason only: `FixedControlsDrawn` does not derive `Default`. Add that derive
    // and the spread becomes writable, `E0063` stops forcing anybody to hand a new
    // control over, and the compiler's half of the promise is gone — without one
    // line of the draw changing. So the derive is the invariant and the check above
    // is the belt beside it.
    //
    // Measured: with `..Default::default()` added to the draw as it stands, the
    // build fails with `E0277: the trait bound FixedControlsDrawn: Default is not
    // satisfied`. That is the compiler refusing it, not this test — the same
    // division as mutation B, found the same way, by reading *why* the exit status
    // was 101.
    let holds = std::fs::read_to_string(src().join(HOLDS_THE_SET))
        .expect("canvas_fixed_controls.rs holds the set and must be readable");
    let declares = the_code_of(&holds);
    const OPENS: &str = "#[derive(";
    let derives = (|| {
        // The `#[derive(..)]` nearest above the declaration is its own: nothing
        // else in this file sits between a derive and the struct it decorates.
        let declared = declares.find("pub struct FixedControlsDrawn")?;
        let above = declares.get(..declared)?;
        let from = declares.get(above.rfind(OPENS)? + OPENS.len()..)?;
        from.get(..from.find(')')?)
    })()
    .unwrap_or_else(|| {
        panic!(
            "could not read the derives above FixedControlsDrawn in {HOLDS_THE_SET}. \
             An unreadable answer is not a safe one here: the check below would pass \
             on an empty string and say nothing about the derive it exists to refuse"
        )
    });
    assert!(
        !derives.contains("Default"),
        "FixedControlsDrawn derives Default (`{derives}`), which makes \
         `..Default::default()` writable in {THE_DRAW}. A control added to the set \
         would then arrive as an empty rectangle with nothing refusing it: no \
         compile error, because every field is accounted for, and nothing for this \
         guard to read. The set is what a frame's name must stay clear of, and a \
         defaulted member of it is a promise about a rectangle nobody drew"
    );

    let controls = the_fields_of("FixedControlsDrawn");
    // A set read as empty would make every assertion below vacuous — the loop
    // would not run and the test would pass having checked nothing, which is the
    // exact shape this file exists to refuse.
    assert!(
        controls.len() > 1,
        "read {} field(s) from FixedControlsDrawn in {HOLDS_THE_SET}: {controls:?}. \
         The set has never had fewer than two, so this is the reader failing rather \
         than the set shrinking, and every check below would pass vacuously",
        controls.len()
    );

    // Each site, and within it each control: the value is read out of the
    // literal it belongs to rather than out of the file, so a second site
    // cannot be answered for by the first one's text.
    for literal in &literals {
        for control in &controls {
            let from = format!("{control}:");
            let at = literal.find(&from).unwrap_or_else(|| {
                panic!(
                    "a `FixedControlsDrawn` in {THE_DRAW} — `{}` — never sets `{control}`, \
                     which {HOLDS_THE_SET} declares as part of the set a frame's name must \
                     stay clear of. A control the draw does not hand over is a control the \
                     rule is not in force against, which is how the Dock was the only \
                     member for weeks",
                    literal.trim()
                )
            });
            // From the field's name to the end of its value: the next comma at
            // this depth. No value in this literal contains a comma, and one
            // that did would be truncated rather than mis-read — a truncated
            // value still has to contain `pictures.` to pass.
            let rest = literal.get(at + from.len()..).unwrap_or_default();
            let value = rest
                .get(..rest.find(',').unwrap_or(rest.len()))
                .unwrap_or("");
            assert!(
                value.contains(FROM_THIS_FRAME),
                "{THE_DRAW} sets `{control}` to `{}`, which is not read from the \
                 pictures this frame laid out. A constant there passes every test in \
                 this crate — measured, not feared: it was tried with \
                 `panel_reserved`, and seventeen tests did not notice. If the picture's \
                 own path moved, this failing is the guard working; say where the \
                 rectangle now comes from rather than loosening what counts",
                value.trim()
            );
        }
    }
}
