# The sign-in screen, drawn on the nested compositor

**Date:** 2026-09-14
**Workstream:** v0.5 — the shell (`docs/autonomy/v0-5-the-shell-plan.md`, task 1)
**Responsible contributor:** Claude worker in `C:\dev\alo-os-shell`, for the owner
**Status:** ready for integration — the code. Not seen on a certified machine.

## What changed

A person at alo OS can now be shown a sign-in screen: a name field and a
password field on the compositor `alo-shell` already is, typed at through the
seat it already has, which lends what was typed to `alo_greeting::Greeting` and
either shows the sentence that crate hands back or hands over to the session
that opened. Until now `alo-greeting` did everything a greeter does except draw.

| File | What it is |
|---|---|
| `crates/alo-shell/src/sign_in_password.rs` | The password being typed: bounded, reserved once, zeroed before it is shortened or dropped, no `Debug`/`Clone`/`Display`/`PartialEq` |
| `crates/alo-shell/src/sign_in_entry.rs` | The name, the password and which field is waiting |
| `crates/alo-shell/src/sign_in_keys.rs` | What one symbol means: a letter, erase, the other field, Enter, or nothing |
| `crates/alo-shell/src/sign_in_seat.rs` | `Server::sign_in_key` — keys through the seat's XKB state, intercepted, never forwarded |
| `crates/alo-shell/src/sign_in_screen.rs` | `SignInScreen`, `Signing`, `SignInShows` — the screen, its one call to `Greeting::signs_in`, the handover |
| `crates/alo-shell/src/sign_in_raster.rs` | Layout and rasterisation with `alo-appearance` tokens and the bundled font |
| `crates/alo-shell/src/sign_in_paint.rs` | Painting a laid-out screen into a GLES frame |
| `crates/alo-shell/src/nested_sign_in.rs` | `Nested::pump_sign_in` and `Nested::submit_sign_in` |
| `crates/alo-shell/examples/sign_in_check.rs` | The WSLg measurement |
| `crates/alo-shell/tests/sign_in_source.rs` | The promises that are about shape, read from the shipped source |
| `docs/contracts/native-sign-in-screen.md` | The new public surface |

Touched to make room: `scene_native.rs` (a `SignIn` scene), `presentation.rs`
(`RenderError::SignInScene`), `keyboard.rs` (two fields `pub(crate)`),
`nested.rs` (two methods `pub(crate)`), `lib.rs`, `Cargo.toml` (`alo-greeting`
and `alo-accounts` as dependencies; `alo-saying` and `alo-sessiond` as
dev-dependencies only), `Cargo.lock`.

**User-readable change description:** *alo OS has a sign-in screen. It asks for
a name and a password, never shows the password or how long it is, says exactly
what the machine's own sign-in said when it refuses — the same sentence for a
wrong password and for a name that does not exist — and gets out of the way the
moment a session opens.*

## Decisions, and why

- **The screen is consumed by every key press.** `pressed(self)` returns
  `Signing::Still(Box<SignInScreen>)` or `Signing::HandedOver(Session)`, and the
  second holds no screen. *Two things never own the screen at once* is then a
  property of the types: after a handover there is no value to pass to
  `submit_sign_in`. Boxed because clippy's `large_enum_variant` is right that
  the two variants differ by 400 bytes.
- **The password field draws one fixed band, not a dot per letter.** The plan
  says the password is never drawn; a row of dots is its length on a screen
  anyone behind the person can read. The band shows that keystrokes arrived and
  nothing else, and a test holds two screens with different passwords to be the
  same picture pixel for pixel.
- **Name and password are both forgotten after every attempt.** A refused name
  left in the field is a hint to the next person at the machine — the argument
  `alo-greeting` makes for holding no memory. The cost is retyping a name.
- **Enter in the name field moves to the password field** rather than signing
  in with an empty password, which would only ever be refused.
- **Keys are intercepted at the seat and keyboard focus is cleared first**, so
  even a client that was somehow mapped cannot hear a password. Chords
  (Control, Alt, logo) type nothing. An out-of-range key code is ignored by the
  pump rather than ending the screen; `sign_in_key` itself still refuses it.
- **A handover is returned even when the parent window closed in the same
  pump**: the session is already open, and dropping it would leave a person
  signed in to something nothing shows.
- **Painting reuses the crate's existing run-length solid painter** (the path
  the control labels and reader already take) rather than adding a texture
  upload path in this change.
- **No offscreen or DRM variant.** The plan says the screen is measured under
  the nested compositor; a direct-display path is a later task and is not
  pretended here.

## Finding: the fields have no labels

No crate declares a word for *name* or *password* as field labels. The plan
forbids writing a word in this crate, and the crates that could own one
(`alo-greeting`, `alo-accounts`) are read-only to this plan. The screen
therefore draws its fields unlabelled, told apart by order and by which one is
waiting (edge thickness and caret, not colour). That is usable but is not good
enough for a screen reader or for somebody who does not already know the
convention, and EN 301 549 expects programmatic labels.

**Proposed follow-up (outside this plan's crates):** `alo-greeting` declares two
label words (e.g. `greeting.name-label`, `greeting.password-label`), `alo-saying`
already collects that crate, and this screen draws them above the fields. That
is a small change, but it is a decision about words and belongs to the crate
that decides them.

## Acceptance criteria and evidence

| Criterion (plan, task 1) | Test |
|---|---|
| Draws a name and a password field on the compositor | `alo-shell lib sign_in_raster::tests::the_name_is_drawn`; measured: `examples/sign_in_check.rs` |
| Takes keystrokes through the seat it already has | `alo-shell lib sign_in_seat::tests::keys_arrive_through_the_seats_layout_and_modifiers` |
| Calls `Greeting`, never `alo-accounts` or `alo-sessiond` directly | `alo-shell sign_in_source the_sign_in_screen_calls_the_greeting_and_never_what_it_composes` |
| Every sentence is `Greeting`'s, in the vocabulary, none written here | `alo-shell lib sign_in_screen::tests::every_sentence_the_screen_can_show_is_collected_and_none_is_written_here`; `alo-shell sign_in_source the_sign_in_screen_writes_no_words_of_its_own` |
| A wrong password says what `alo-greeting` says, never attempts or whether the name exists | `alo-shell lib sign_in_screen::tests::a_wrong_password_says_what_the_greeting_says_and_never_how_many_or_whether_the_name_exists` |
| The password is never drawn | `alo-shell lib sign_in_raster::tests::the_password_is_never_drawn_not_even_its_length` |
| Never in a `Debug` (source read) | `alo-shell sign_in_source no_type_that_carries_a_password_has_a_debug_that_could_print_it`; `alo-shell sign_in_source the_password_holder_has_no_road_out` |
| Held only as long as needed | `alo-shell lib sign_in_screen::tests::what_was_typed_is_forgotten_once_it_is_asked_about`; `alo-shell lib sign_in_password::tests::what_is_taken_back_is_zeroed_before_it_is_let_go` |
| Hands over when a session opens | `alo-shell lib sign_in_screen::tests::the_right_password_opens_a_session_and_the_screen_hands_over` |
| No account is created from this screen | `alo-shell lib sign_in_screen::tests::a_machine_with_nobody_on_it_takes_no_keys_and_makes_no_account` |

Refusal paths beside those: door refusals in the door's words, a door nobody is
behind, a machine described for somebody else, accounts that would not be read,
chords, repeated presses and unmatched releases, out-of-range codes, a display
with no keyboard, an output too small or too large, terracotta never on screen.

Both source tests were checked against deliberate mutations (a hand-written
`Debug` for `SignInScreen`; a new struct holding a `SignInEntry`) and failed on
each, naming the file and line; the mutations were removed.

## Verification

Executed on 2026-09-14, Windows 11 host, Ubuntu under WSL 2, Rust 1.98.0,
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-shell-cd217193b5311c25`:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy -p alo-shell --all-targets -- -D warnings` — clean.
- `cargo test -p alo-shell` — lib 201 passed, `client_lifecycle` 262 passed,
  `sign_in_source` 4 passed, `socket_ownership` 3 passed, doc tests 4 passed;
  0 failed. (The `error in client communication` lines in that output are the
  existing protocol-refusal tests.)
- `WAYLAND_DISPLAY=wayland-0 timeout 60s …/examples/sign_in_check` under WSLg —
  exit 0: all three standings drawn in light and dark through the parent's EGL,
  pumped with real parent input and nothing proceeding on silence; 10 frames
  submitted. `--interactive SECONDS` exists for a person to type at it; it was
  not run, since nobody was at the window.

Not run: the full workspace suite (the supervisor runs it). Only `alo-shell` was
changed.

## Limitations

- **Not seen on a certified machine**, and no direct-display (DRM/KMS)
  submission exists for it yet. `ROADMAP.md`'s *firmware to sign-in* line
  cannot be ticked on the strength of this.
- **Debug-build frame rate is low**: 10 submissions in the check's 1.8 s of
  drawing, including pumping and a 16 ms sleep per frame, on WSLg. Text is
  painted as one solid per run of equal pixels, as the crate's labels already
  are. A release build and a texture upload path are the obvious measures if a
  certified machine shows typing lag.
- **No field labels** — see the finding above.
- **No pointer input**; the two fields are reached with Tab and Enter.
- Nothing launches the greeter at boot; that is the session and image work.

## Proposed changes to shared documents

**CHANGELOG.md** — *alo OS has a sign-in screen: a name and a password, the
password never shown nor its length, the machine's own sentence when sign-in is
refused, and a handover to the session when it opens. Measured under a nested
compositor; not yet on a certified machine.*

**ROADMAP.md** — under v0.01 *firmware to sign-in*: `- [x] The code.` for the
sign-in screen; nothing on the machine.

**docs/autonomy/QUEUE.md** — the shell plan's task 1 done; proposed follow-up
for `alo-greeting` field-label words; a direct-display submission of the
sign-in screen.

**docs/autonomy/STATE.md** — reference this report.
