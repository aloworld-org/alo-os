//! **Where a person left their canvas, between one sign-in and the next.**
//!
//! Task 9 of `docs/autonomy/the-smallest-canvas-worth-showing.md`: *frame
//! positions and the camera survive a session ending and starting again.* Until
//! this crate nothing kept either, so every session began at the plane's origin at
//! life size with every window placed as if it had never been opened.
//!
//! One file, read by the shell when it stands the desktop up and written when it
//! comes down. That is the whole of it, and what it is **not** is most of the
//! design.
//!
//! # This crate decides nothing about a canvas
//!
//! What a place on the plane is, how far out a camera may look and how big a frame
//! may be are all `alo-canvas`'s. Every value read back off the disk is rebuilt by
//! [`alo_canvas::At::checked`], [`alo_canvas::Size::checked`] and
//! [`alo_canvas::Zoom::of`] — the same road a live canvas takes — so a file
//! hand-edited to put a window a million units off the plane, or to a zoom no
//! canvas has, is refused by the crate that owns that rule. There is no
//! `Deserialize` for a `Camera` or a `Frame` here and there must not be one: a
//! position read back without being checked is a position nothing validated.
//!
//! # A window is remembered by what it is, not by which one it was
//!
//! A `wl_surface` does not survive a session, so the key is the application's own
//! `app_id` — the thing that is the same when it opens again tomorrow. That has a
//! consequence worth stating rather than discovering: **two windows of one
//! application share one remembered place**, and the second to open gets it. A
//! per-window identity would need something the application does not give us, and
//! inventing one would mean remembering a window by a title a person can change.
//!
//! # An application that did not come back leaves nothing behind
//!
//! The acceptance says *a frame whose application did not come back is absent
//! rather than drawn empty*, and this is where that is kept: the file is a list of
//! places waiting to be claimed, and a place nothing claims is simply never used.
//! Nothing here draws, reserves room, or tells anybody that a window is missing.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use alo_canvas::{At, Camera, Place, Size, Zoom};

/// Why an arrangement could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotArranged {
    /// The file is not the shape this crate writes.
    #[error("the arrangement could not be read: {0}")]
    Unreadable(String),
    /// The file says a version this one does not write or read.
    ///
    /// **Its own variant rather than [`Self::Unreadable`]**, because the two are
    /// different situations for the person in front of the machine: a file that
    /// is not this shape is damaged or is somebody else's, and a file from
    /// another version is theirs and this binary is the wrong one to open it
    /// with. Carries both numbers, so the message can say which way round it is.
    #[error("the arrangement says version {said}, and this reads version {reads}")]
    AnotherVersion {
        /// What the file claimed.
        said: u32,
        /// What this crate reads.
        reads: u32,
    },
    /// A value in it is not one `alo-canvas` allows.
    ///
    /// Carries what was refused, because a hand-edited file is the case this
    /// exists for and *which line* is the only useful thing to say about it.
    #[error("the arrangement holds a place no canvas has: {0}")]
    NotOnThePlane(String),
}

/// One window's place, as it is written down.
///
/// The four numbers are its **ordinary** geometry, not the geometry it was last
/// drawn at — see [`AWindowWas::normal`] for why that distinction is the whole
/// point of remembering a window at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct Written {
    /// Plane units across.
    x: i32,
    /// Plane units down.
    y: i32,
    /// How wide it was.
    width: u32,
    /// How tall it was.
    height: u32,
    /// How it was showing over that geometry.
    ///
    /// **Defaulted on read, and the default is the ordinary case.** A window
    /// written by a release that knew only a place and a size was an ordinary
    /// window, because there was no other state it could have been in — so the
    /// absent key reads as [`HowItWasShowing::Ordinary`] rather than refusing
    /// the file. The version above is what refuses a shape this does not know;
    /// a missing field inside a shape it does know is a different question.
    #[serde(default)]
    showing: HowItWasShowing,
}

/// What version of this file this crate writes, and the only one it reads.
///
/// # Why this exists before anything needs it
///
/// **Because today it is free and tomorrow it is not.** `TheFile` carries
/// `deny_unknown_fields`, and `arranging_tests.rs` asserts that refusal on
/// purpose: *an unknown key is a file written by something else, or by a later
/// version of this one. Either way it is not read half-way.* So the first key
/// ever added to this file makes every new file unreadable to every older
/// binary — and with no version there is nothing to negotiate that through, only
/// a parse error that says `unknown field`.
///
/// Measured on 2026-09-30: **nothing in this repository writes this file to
/// disk.** `Arrangement::written` has no production caller, `read` none outside
/// this crate's own tests, and the file has no name and no path anywhere in the
/// tree. So there are no files in the world to break, and there will be from the
/// first machine that saves one.
///
/// **That stopped being true on 2026-10-03, and this is the sentence to read
/// before bumping this number again.** `keeping::keep` writes
/// `canvas-layout.toml` and `alo-desktop` calls it on every meaningful layout
/// change, so **version 3 files exist on real machines**. The paragraph above is
/// kept because it is why 1 and 2 cost nothing; it is no longer why 4 would.
///
/// `docs/autonomy/the-canvas-and-its-places.md` task 5 is what needs it: an
/// arrangement per Place is a **whole-file reshape** rather than a key, because
/// two cameras cannot live in a file with one. That change is affordable only
/// while this number exists to say which shape a file is.
///
/// **It is not a compatibility promise.** A later version may refuse this one
/// outright; what it may not do is read it and be wrong about what it means.
///
/// # Strict equality is the owner's decision, not an unexamined one
///
/// The panel lane put the contrary case while reading the change that made this
/// file real: `read` refuses a version that is not exactly this one — a floor
/// would accept older ones — and **a file written by an older version of our own
/// format is not wrong, it is old**, while this check treats the two
/// identically. It is a good argument and it is the one somebody will make again.
///
/// The owner settled it on 2026-10-04, in these words:
///
/// > **old rules that were modified are wrong**
///
/// So an older version is not merely old: it was written under a rule that has
/// since been modified, and reading it means interpreting bytes under rules they
/// were not written to. That is the one thing the paragraph above forbids. **The
/// check stays strict.**
///
/// What it costs, stated so nobody discovers it: at a bump, every existing file
/// is refused rather than migrated, and a person loses their remembered
/// arrangement **once**, at that upgrade. The cost is bounded and visible —
/// `keeping::at_sign_in` answers with a fresh arrangement and the reason,
/// `alo-desktop` prints it, and the next layout change replaces the file. A
/// person gets the canvas their applications put up, not a refusal to start.
///
/// # Two, and why there is nothing to migrate
///
/// Version 1 held **one** camera beside a flat map of windows. Task 5 of
/// `docs/autonomy/the-canvas-and-its-places.md` asks that *two Places with
/// different cameras both come back as they were*, and two cameras cannot live in
/// a file with one — so this is a whole-file reshape rather than a key addition,
/// which is exactly what the version existed to make affordable.
///
/// **No file of version 1 or 2 was ever written to disk**, and that sentence
/// has a date on it. `written` had no production caller when 1 was defined,
/// none when 2 was, and none when 3 was — so versions 1 and 2 were spent
/// without a migration, and an older file is refused by name like any other
/// shape this does not read.
///
/// **Version 3 is not in that position and has not been since `alo-desktop`
/// gained its two calls.** Measured 2026-10-04:
/// `grep -rn 'arranging::keep\|at_sign_in' crates --exclude-dir=target` finds
/// `crates/alo-desktop/src/main.rs:294` reading one at sign-in and `:318`
/// keeping one when the layout moves, neither under `cfg(test)`. **Files of
/// this shape are on real disks**, and alo OS 0.0.1 is public.
///
/// So the reasoning above is spent too. **A bump to 4 costs a person the
/// canvas they left** unless it reads 3 as well — `at_sign_in` answers with a
/// fresh arrangement and a printed reason, which is the right behaviour for a
/// damaged file and the wrong one for an upgrade. Whoever moves this number
/// owes the first migration this crate has needed, and
/// `docs/misreadings/nothing-is-on-a-disk-is-a-fact-with-a-date-on-it.md` is
/// why the sentence above did not say so for two days.
///
/// **3 is how a window was showing**, which the owner's ruling of 2026-10-03
/// requires alongside its place: a window that was minimised, compacted or full
/// screen comes back as its ordinary self in that state rather than frozen at
/// the size it was last drawn. That is a new field inside each window rather
/// than a reshape of the file, and it reads as the ordinary case when absent —
/// so strictly it need not have moved the version. It moved anyway, because the
/// first release to write one of these files should write the shape it means,
/// and nothing is on a disk to be migrated.
pub const FORMAT: u32 = 4;

/// How many earlier states one Place remembers.
///
/// **A ribbon a person drags has to be scannable, and a file a session writes
/// on every rearrangement has to stay small.** Twenty is where those two meet:
/// a fortnight of ordinary rearranging on one Place, and a file that grows by
/// a camera and a window list per entry rather than without bound.
///
/// It is a cap rather than an age. *The canvas as it was on Tuesday* is what a
/// person asks for, but pruning by date would forget a Place nobody touched
/// for a month — and a Place nobody touched is exactly the one whose earlier
/// states are still worth having.
pub const HOW_MANY_A_PLACE_REMEMBERS: usize = 20;

/// Every version [`Arrangement::read`] takes, newest first.
///
/// **Three is here because files of it are on disks.** A version this does not
/// list is refused by name, which is what versions 1 and 2 got and deserved:
/// nothing had ever written one. The day a version on this list stops being
/// readable is the day somebody loses a canvas, so a number leaves it only
/// with a reason written beside its leaving.
const READS: [u32; 2] = [FORMAT, 3];

/// One Place's own arrangement, as it is written down.
///
/// **A camera per Place, which is the whole of task 5.** Version 1 had one camera
/// for the session; a person who left two Places at two zooms had one of them
/// restored wrongly and nothing said so.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
struct APlacesOwn {
    /// Where the camera was looking on this Place, in plane units.
    looking_at: (i32, i32),
    /// How far in, in thousandths.
    zoom: u32,
    /// Each application's window on this Place, by `app_id`.
    ///
    /// A map rather than a list, so the file says plainly that one application has
    /// one remembered place **per Place** and a second row for it cannot be
    /// written. One application may now appear once on each of several Places,
    /// which is what `one_application_has_one_place` asserted was impossible when
    /// there was only one.
    windows: BTreeMap<String, Written>,
    /// What this Place was before, oldest first.
    ///
    /// **Absent in a version-3 file and absent in a Place nobody has changed
    /// twice**, which is the same shape: a Place with no earlier states is a
    /// Place whose ribbon has one stop on it.
    earlier: Vec<HeldInFile>,
}

/// One earlier state of one Place, as it is written down.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
struct HeldInFile {
    /// When this state stopped being the current one, in seconds since the
    /// epoch.
    ///
    /// **Seconds rather than a formatted time**, for the boundary rule this
    /// crate already states about positions: across a disk hold the raw number
    /// and validate on read. A clock that went backwards writes a smaller
    /// number and nothing here pretends otherwise — the series is in the order
    /// it was held, which is the order a person made it in, and sorting it by
    /// its own timestamps would reorder a person's history to flatter a clock.
    when: u64,
    /// Where the camera was looking, in plane units.
    looking_at: (i32, i32),
    /// How far in, in thousandths.
    zoom: u32,
    /// Each application's window on this Place then.
    windows: BTreeMap<String, Written>,
}

/// The whole file, as it is written down.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
struct TheFile {
    /// Which shape this file is, as [`FORMAT`] numbers them.
    ///
    /// **Zero is the absent case and is refused by name.** `#[serde(default)]` on
    /// the container takes every missing field from this struct's derived
    /// `Default`, so a file with no `version` arrives here as zero — and
    /// [`Arrangement::read`] answers [`NotArranged::AnotherVersion`] rather than
    /// letting a missing version mean the current one. **A file that does not say
    /// its version is not from a version that wrote one.**
    ///
    /// Which is why the default is *derived* and [`Arrangement::written`] sets
    /// [`FORMAT`] explicitly: a hand-written `Default` giving the current version
    /// would be read back through this same path and make the refusal
    /// unreachable. That was written the wrong way round first, and the doc
    /// comment describing the refusal sat above an implementation that prevented
    /// it.
    version: u32,
    /// Each Place's own arrangement, keyed by the Place's number.
    ///
    /// **The key is the raw number as text, and it is validated on read.** TOML
    /// keys are strings, so the boundary cannot carry an `alo_canvas::Place` — and
    /// the rule this crate already states about positions applies to identities:
    /// *across a disk boundary hold the raw number and validate on read, because
    /// serde will invent one; inside a process hold the checked type, because
    /// nothing will invent one for you.* So a key that is not a number, or is
    /// zero, is a file this does not read rather than a Place nobody has.
    places: BTreeMap<String, APlacesOwn>,
}

/// **One stop on a Place's ribbon**: when it was, and enough to draw it.
///
/// What a ribbon needs and no more. The windows themselves are not handed out
/// here: a caller that wants them asks for the state by its moment through
/// [`Arrangement::as_it_was`], which is the one road back and the one that
/// holds what is being left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsItWas {
    /// When this state stopped being the current one.
    when: SystemTime,
    /// Where the camera was looking then.
    camera: Camera,
    /// How many windows were on the Place then.
    how_many: usize,
}

impl AsItWas {
    /// When this state stopped being the current one.
    #[must_use]
    pub const fn when(&self) -> SystemTime {
        self.when
    }

    /// Where the camera was looking then.
    #[must_use]
    pub const fn camera(&self) -> Camera {
        self.camera
    }

    /// How many windows were on the Place then.
    #[must_use]
    pub const fn how_many(&self) -> usize {
        self.how_many
    }
}

/// **Where a person left their canvas.**
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Arrangement {
    /// Each Place's own camera and windows.
    ///
    /// **Keyed by the checked type**, not by the number — the raw number lives
    /// only in the file. Empty for a canvas nobody has arranged, which is not the
    /// same as a Place with nothing on it: the first is *no answer* and the second
    /// is *an answer that is empty*, and a restore must be able to tell them
    /// apart.
    places: BTreeMap<Place, OnePlace>,
}

/// How a window was showing when the session ended.
///
/// **This crate's own vocabulary, deliberately not the shell's.** The states a
/// window can be in live in `alo-shell`, which is the crate that *reads* this
/// one — so naming them from there would invert the dependency, and
/// `alo-arranging` declares only `alo-canvas`. The shell maps its own mode into
/// this when it saves, the same way `alo-reported` mirrors a scope rather than
/// importing one.
///
/// **It carries no geometry.** The geometry a window is remembered at is its
/// ordinary one, in [`AWindowWas::normal`], whatever state it was in — see
/// there for why.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HowItWasShowing {
    /// Its ordinary self, at the geometry remembered for it.
    #[default]
    Ordinary,
    /// Filling the output, with the shell's own furniture still drawn over it.
    Maximised,
    /// Filling the screen, with the Dock and the panel giving way to it.
    FillingTheScreen,
    /// Sitting in a share a division gave it.
    ///
    /// **The share itself is not remembered here**, and that is not an
    /// omission. A share is a rectangle a division decided, and the division is
    /// another crate's state with its own file; a window put back into a share
    /// this file invented would be a second layout decider. So the fact is
    /// kept and the rectangle is asked for again.
    InAShare,
    /// A live tile on the canvas, shrunk but still showing its work.
    Compacted,
    /// A preview in the panel at the edge of the screen.
    PutAside,
}

/// Where one window was, and how it was showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AWindowWas {
    /// Where it sat and how big it was **as its ordinary self**.
    ///
    /// **Always the ordinary geometry, never the geometry it was last drawn
    /// at.** A window that was full screen when the session ended is one
    /// rectangle the size of an output; remembering that would restore it as a
    /// window the size of a screen that nothing could make smaller again,
    /// because the size it used to be would be gone. The shell already holds
    /// this distinction in memory — its own mode record keeps the initial
    /// committed normal geometry and says that later maximise requests never
    /// replace it — so this file keeps the same thing rather than a second
    /// answer.
    normal: (At, Size),
    /// How it was showing over that geometry.
    showing: HowItWasShowing,
}

impl AWindowWas {
    /// A window at this ordinary geometry, showing this way.
    #[must_use]
    pub const fn at(normal: (At, Size), showing: HowItWasShowing) -> Self {
        Self { normal, showing }
    }

    /// Where it sat and how big it was as its ordinary self.
    #[must_use]
    pub const fn normal(self) -> (At, Size) {
        self.normal
    }

    /// How it was showing over that geometry.
    #[must_use]
    pub const fn showing(self) -> HowItWasShowing {
        self.showing
    }
}

/// One Place's camera and windows, in memory.
#[derive(Debug, Clone, PartialEq, Eq)]
struct OnePlace {
    /// Where they were looking on this Place.
    camera: Camera,
    /// Where each application's window was on it, how big, and how it was
    /// showing.
    windows: BTreeMap<String, AWindowWas>,
    /// What this Place was before, oldest first, capped at
    /// [`HOW_MANY_A_PLACE_REMEMBERS`].
    earlier: Vec<Held>,
}

/// One earlier state of one Place, inside a process.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Held {
    /// When it stopped being the current one.
    when: SystemTime,
    /// Where the camera was looking then.
    camera: Camera,
    /// What was on it then.
    windows: BTreeMap<String, AWindowWas>,
}

impl OnePlace {
    /// Forget the oldest states beyond the cap.
    ///
    /// **The oldest go**, because a ribbon a person drags is reached from the
    /// end they are standing at: the canvas ten minutes ago is asked for far
    /// more often than the canvas a fortnight ago, and a cap that dropped the
    /// recent ones would empty the part of the ribbon that is used.
    fn remember_no_more_than_the_cap(&mut self) {
        let too_many = self
            .earlier
            .len()
            .saturating_sub(HOW_MANY_A_PLACE_REMEMBERS);
        self.earlier.drain(..too_many);
    }

    /// A Place nobody has arranged yet: the origin, at life size, nothing
    /// placed, and nothing remembered.
    fn fresh() -> Self {
        Self {
            camera: Camera::new(),
            windows: BTreeMap::new(),
            earlier: Vec::new(),
        }
    }
}

impl Arrangement {
    /// A canvas nobody has arranged yet: no Places remembered at all.
    #[must_use]
    pub fn fresh() -> Self {
        Self {
            places: BTreeMap::new(),
        }
    }

    /// The camera this Place was left with, or [`None`] if it was never arranged.
    ///
    /// **[`None`] rather than the origin at life size**, because *this Place was
    /// never left anywhere* and *this Place was left at the origin* are different
    /// answers and only one of them should move a camera. Version 1 could not tell
    /// them apart: it had one camera and it always had a value.
    #[must_use]
    pub fn camera_on(&self, place: Place) -> Option<Camera> {
        self.places.get(&place).map(|one| one.camera)
    }

    /// Remember where this application's window was, on this Place.
    ///
    /// One place per `app_id` **per Place**: writing a second for the same pair
    /// replaces the first. One application may now be remembered once on each of
    /// several Places, which is the thing version 1 made impossible.
    ///
    /// **Takes the whole answer at once**, which is the same reason
    /// `alo_put_aside::AtWork::on` does: a window remembered field by field is
    /// a window that can be written down half-described, and a restore reading
    /// a place with no state beside it would have to guess.
    pub fn window_was(&mut self, place: Place, app_id: impl Into<String>, was: AWindowWas) {
        self.places
            .entry(place)
            .or_insert_with(OnePlace::fresh)
            .windows
            .insert(app_id.into(), was);
    }

    /// Remember where the camera was on this Place.
    pub fn looking(&mut self, place: Place, camera: Camera) {
        self.places
            .entry(place)
            .or_insert_with(OnePlace::fresh)
            .camera = camera;
    }

    /// Where this application's window was on this Place, if it was there at all.
    #[must_use]
    pub fn where_it_was(&self, place: Place, app_id: &str) -> Option<AWindowWas> {
        self.places.get(&place)?.windows.get(app_id).copied()
    }

    /// How many windows are remembered on this Place.
    #[must_use]
    pub fn how_many_on(&self, place: Place) -> usize {
        self.places.get(&place).map_or(0, |one| one.windows.len())
    }

    /// How many windows are remembered across every Place.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.places.values().map(|one| one.windows.len()).sum()
    }

    /// Every Place this arrangement remembers, lowest number first.
    pub fn each_place(&self) -> impl Iterator<Item = Place> + '_ {
        self.places.keys().copied()
    }

    /// **What this Place was before, oldest first** — what a ribbon is drawn
    /// from.
    ///
    /// Empty for a Place nobody has rearranged twice, and for every Place in a
    /// file written before this crate remembered anything. A Place with one
    /// stop on its ribbon is not a fault.
    #[must_use]
    pub fn earlier_on(&self, place: Place) -> Vec<AsItWas> {
        self.places.get(&place).map_or_else(Vec::new, |one| {
            one.earlier
                .iter()
                .map(|held| AsItWas {
                    when: held.when,
                    camera: held.camera,
                    how_many: held.windows.len(),
                })
                .collect()
        })
    }

    /// **Put this Place back as it was at `when`, and keep where it is now.**
    ///
    /// Answers `false` and changes nothing when this Place has no state held
    /// at that moment — a ribbon drawn from [`Self::earlier_on`] cannot ask for
    /// one that is not there, and a caller that asks anyway is told rather than
    /// silently given the nearest.
    ///
    /// **The state being left is held first**, which is the acceptance's own
    /// words: *the person is never shown a state they cannot get back from.*
    /// Going back is itself a rearrangement, so the canvas a person is leaving
    /// joins the ribbon rather than being dropped — and going back twice walks
    /// backwards rather than losing the middle.
    pub fn as_it_was(&mut self, place: Place, when: SystemTime, now: SystemTime) -> bool {
        let Some(one) = self.places.get_mut(&place) else {
            return false;
        };
        let Some(at) = one.earlier.iter().position(|held| held.when == when) else {
            return false;
        };
        let held = one.earlier.remove(at);
        one.earlier.push(Held {
            when: now,
            camera: one.camera,
            windows: std::mem::take(&mut one.windows),
        });
        one.camera = held.camera;
        one.windows = held.windows;
        one.remember_no_more_than_the_cap();
        true
    }

    /// **This arrangement, carrying forward what `previous` remembered** — and
    /// holding `previous`'s current state for every Place that moved.
    ///
    /// The shell builds an arrangement from the live canvas each time
    /// (`the_arrangement_now`), so it has no memory of what came before: the
    /// series can only be assembled where both are in hand, which is at the
    /// moment one is kept over the other. A Place whose camera and windows are
    /// unchanged holds nothing — a file rewritten by a save that changed
    /// another Place does not fill this one's ribbon with repeats.
    #[must_use]
    pub fn following(&self, previous: &Self, when: SystemTime) -> Self {
        let mut carried = self.clone();
        for (place, was) in &previous.places {
            let entry = carried.places.entry(*place).or_insert_with(OnePlace::fresh);
            entry.earlier.clone_from(&was.earlier);
            if entry.camera != was.camera || entry.windows != was.windows {
                entry.earlier.push(Held {
                    when,
                    camera: was.camera,
                    windows: was.windows.clone(),
                });
            }
            entry.remember_no_more_than_the_cap();
        }
        carried
    }

    /// This arrangement, as the file says it.
    #[must_use]
    pub fn written(&self) -> String {
        let file = TheFile {
            // **Set here rather than defaulted**, so that the absent case stays
            // reachable in `read`. See the field's own note.
            version: FORMAT,
            places: self
                .places
                .iter()
                .map(|(place, one)| {
                    (
                        place.number().to_string(),
                        APlacesOwn {
                            looking_at: (one.camera.at().x, one.camera.at().y),
                            zoom: one.camera.zoom().thousandths(),
                            earlier: one
                                .earlier
                                .iter()
                                .map(|held| HeldInFile {
                                    // A moment before the epoch is not one this
                                    // crate held, so it writes as the epoch
                                    // rather than refusing a whole layout over
                                    // a clock. `read` takes it back as the
                                    // epoch and the ribbon has a stop with an
                                    // odd date on it, which is visible and
                                    // recoverable; a refused file is neither.
                                    when: held
                                        .when
                                        .duration_since(UNIX_EPOCH)
                                        .map_or(0, |since| since.as_secs()),
                                    looking_at: (held.camera.at().x, held.camera.at().y),
                                    zoom: held.camera.zoom().thousandths(),
                                    windows: held
                                        .windows
                                        .iter()
                                        .map(|(app_id, was)| {
                                            let (at, size) = was.normal();
                                            (
                                                app_id.clone(),
                                                Written {
                                                    x: at.x,
                                                    y: at.y,
                                                    width: size.width(),
                                                    height: size.height(),
                                                    showing: was.showing(),
                                                },
                                            )
                                        })
                                        .collect(),
                                })
                                .collect(),
                            windows: one
                                .windows
                                .iter()
                                .map(|(app_id, was)| {
                                    let (at, size) = was.normal();
                                    (
                                        app_id.clone(),
                                        Written {
                                            x: at.x,
                                            y: at.y,
                                            width: size.width(),
                                            height: size.height(),
                                            showing: was.showing(),
                                        },
                                    )
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
        };
        // A file this crate wrote is always writable: every value in it came out
        // of a checked one. An unwritable one would be a bug here rather than
        // anything a person did, so it answers with nothing arranged rather than
        // taking the session down over a layout.
        toml::to_string_pretty(&file).unwrap_or_default()
    }

    /// An arrangement read back, with every value rebuilt by `alo-canvas`.
    ///
    /// # Errors
    /// [`NotArranged::Unreadable`] where the file is not this shape, and
    /// [`NotArranged::NotOnThePlane`] where a value in it is not one a canvas
    /// allows — a window a million units out, a zoom further in than any canvas
    /// goes, a window of no width. Refused rather than clamped: a person whose
    /// file was quietly corrected would have a canvas that disagreed with what
    /// they could read.
    pub fn read(written: &str) -> Result<Self, NotArranged> {
        let file: TheFile =
            toml::from_str(written).map_err(|why| NotArranged::Unreadable(why.to_string()))?;
        // **Before any value in it is trusted.** A file from another shape may
        // parse cleanly and mean something else entirely — version 1 had these
        // same keys one level higher — so the version is checked before the
        // cameras rather than after the windows.
        //
        // **Three is read as well as four, and this is the first migration this
        // crate has had.** Versions 1 and 2 were spent without one because
        // nothing had ever written them; 3 is on real disks, written by
        // `alo-desktop` since before 0.0.1 went public, so refusing it would
        // cost a person the canvas they left on the day they upgraded
        // (`docs/misreadings/nothing-is-on-a-disk-is-a-fact-with-a-date-on-it.md`).
        //
        // It needs no conversion code: a Place's series is `#[serde(default)]`,
        // so a version-3 Place arrives with an empty ribbon, which is exactly
        // what it is — a Place whose earlier states were never kept. The first
        // rearrangement after an upgrade puts the first stop on it.
        if !READS.contains(&file.version) {
            return Err(NotArranged::AnotherVersion {
                said: file.version,
                reads: FORMAT,
            });
        }
        let mut places = BTreeMap::new();
        for (key, was) in file.places {
            // The boundary rule, applied to an identity rather than a position:
            // the raw number crossed the disk and is validated here. A key that is
            // not a number, or is zero, is a file this does not read.
            let place = key
                .parse::<u64>()
                .ok()
                .and_then(Place::numbered)
                .ok_or_else(|| NotArranged::NotOnThePlane(format!("{key}: not a Place")))?;
            let at = At::checked(was.looking_at.0, was.looking_at.1)
                .ok_or_else(|| NotArranged::NotOnThePlane(format!("{key}: looking-at")))?;
            let zoom = Zoom::of(was.zoom)
                .map_err(|why| NotArranged::NotOnThePlane(format!("{key}: zoom: {why}")))?;
            let camera = Camera::new()
                .looking_at(at)
                .and_then(|camera| camera.zoomed_to(zoom, (0, 0)))
                .ok_or_else(|| NotArranged::NotOnThePlane(format!("{key}: its camera")))?;
            let mut windows = BTreeMap::new();
            for (app_id, where_it_was) in was.windows {
                let at = At::checked(where_it_was.x, where_it_was.y).ok_or_else(|| {
                    NotArranged::NotOnThePlane(format!("{key}/{app_id}: its place"))
                })?;
                let size =
                    Size::checked(where_it_was.width, where_it_was.height).ok_or_else(|| {
                        NotArranged::NotOnThePlane(format!("{key}/{app_id}: its size"))
                    })?;
                windows.insert(app_id, AWindowWas::at((at, size), where_it_was.showing));
            }
            let mut earlier = Vec::new();
            for held in was.earlier {
                let at = At::checked(held.looking_at.0, held.looking_at.1).ok_or_else(|| {
                    NotArranged::NotOnThePlane(format!("{key}: an earlier looking-at"))
                })?;
                let zoom = Zoom::of(held.zoom).map_err(|why| {
                    NotArranged::NotOnThePlane(format!("{key}: an earlier zoom: {why}"))
                })?;
                let camera = Camera::new()
                    .looking_at(at)
                    .and_then(|camera| camera.zoomed_to(zoom, (0, 0)))
                    .ok_or_else(|| {
                        NotArranged::NotOnThePlane(format!("{key}: an earlier camera"))
                    })?;
                let mut windows = BTreeMap::new();
                for (app_id, where_it_was) in held.windows {
                    let at = At::checked(where_it_was.x, where_it_was.y).ok_or_else(|| {
                        NotArranged::NotOnThePlane(format!("{key}/{app_id}: an earlier place"))
                    })?;
                    let size = Size::checked(where_it_was.width, where_it_was.height).ok_or_else(
                        || NotArranged::NotOnThePlane(format!("{key}/{app_id}: an earlier size")),
                    )?;
                    windows.insert(app_id, AWindowWas::at((at, size), where_it_was.showing));
                }
                earlier.push(Held {
                    when: UNIX_EPOCH + std::time::Duration::from_secs(held.when),
                    camera,
                    windows,
                });
            }
            // **A file may arrive holding more than the cap** — written by a
            // later release whose cap is larger, or by a hand. It is read and
            // then trimmed rather than refused: the states beyond the cap are
            // this machine's to forget, not a reason to tell a person their
            // layout is unreadable.
            let mut one = OnePlace {
                camera,
                windows,
                earlier,
            };
            one.remember_no_more_than_the_cap();
            places.insert(place, one);
        }
        Ok(Self { places })
    }
}

pub mod keeping;

#[cfg(test)]
#[path = "arranging_tests.rs"]
mod tests;
