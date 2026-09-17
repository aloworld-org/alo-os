# An organisation's permitted places, read from the machine's description

- Date: 2026-09-16
- Workstream: v0.5 software and the web (`docs/autonomy/v0-5-software-and-the-web-plan.md`, task 8)
- Contributor: Claude Code
- Status: **ready for integration.** Every acceptance criterion has a test; what
  is not shown is named under *Limitations*.

## What a person outside this repository needs to know

An organisation that manages a machine can now say which places applications may
be installed from, in the one file that already describes the machine,
`/etc/alo/agentd.toml`:

```toml
format = 3

[applications]
may-come-from = ["acme-apps", "flathub"]
```

When a person on that machine tries to install from a place the list does not
name, they are told *the organisation that manages this machine does not permit
installing applications from flathub* — the rule, and who made it. On a machine
whose description has no such section — every personal machine — nothing changes
and there is no rule. A person who writes the same list into a description they
own is bounded identically and is told no organisation set it.

An alo OS too old to enforce the list refuses the description and does not start,
rather than quietly installing from anywhere. A list that cannot be read — a
misspelt key, a name no place could have, a place named twice — also stops the
service rather than becoming no rule.

## What changed

| | |
|---|---|
| `crates/alo-agentd/src/permitted_places.rs` (new) | `[applications]` as typed, and the one door from it to `alo_software::Bound`, attributed by who owns the file |
| `crates/alo-agentd/src/describing.rs` | the optional section on the description; `THE_FORMAT` is `3`, `ALSO_READ` is `[1, 2]`; `QUESTIONS_SINCE` and `APPLICATIONS_SINCE` name the shape each section arrived in |
| `crates/alo-agentd/src/described.rs` | `Bounds` — the two bounds together, with no `Default` — and `Described::applications` |
| `crates/alo-agentd/src/refusing.rs` | `NotDescribed::PlacesNeedANewerShape`, `NotAPlaceName`, `APlaceNamedTwice` |
| `crates/alo-agentd/src/installing_under_the_description.rs` (new, test only) | the acceptance: an install refused in the organisation's words, beside the same install with no section |
| `crates/alo-agentd/tests/what_a_machine_says_about_itself.rs` | the section read off a real disk, attributed by the file's real owner; its absence; its refusals |
| `crates/alo-agentd/src/lib.rs`, `starting.rs`, `Cargo.toml`, `Cargo.lock` | registration; `alo-software` as a dependency, `alo-applications` as a dev-dependency |
| `docs/contracts/machine-description.md` | the `[applications]` section, its refusals, and its `format` rule |
| `docs/autonomy/v0-5-software-and-the-web-plan.md` | task 8 marked done; task 9 written |

**Nothing in `alo-software` was edited.** What a permitted place may do, the order a
place is refused in (not set up → outside the bound → not verified → nowhere to
reach) and the words are all that crate's; this task only produces the value it
already takes.

## Decisions

**The section is `[applications]` with `may-come-from`.** It pairs with
`[questions]` / `may-go`: a section named for what is bounded, one key saying where
it may come from. `[software]` was the other candidate; it names the crate rather than
what a person or an administrator thinks they are restricting.

**It needs `format = 3`, not a reuse of `2`.** The plan's rule is `[questions]`'s: an
older service must refuse a description whose bound it cannot enforce. A format-2
service does already refuse an unknown section through `deny_unknown_fields`, but it
refuses it as a typo, which sends whoever is on call to the wrong line; the format
number is what the contract says carries a shape change, so it carries this one.
`[questions]` stays readable from `2` onward, and its older-shape refusal now names
`QUESTIONS_SINCE` (2) rather than `THE_FORMAT`, so a `1` carrying `[questions]` is still
told the truth about which shape it needs.

**An empty list is a rule that keeps everything out.** `alo_software::Bound` already
decided this (*an organisation that wrote an empty list wrote a rule*), and the plan
forbids re-deciding what `alo-software` decides.

**A place named twice is refused.** It permits nothing a single mention would not,
so refusing it is strict — but a duplicated line is how a copied-and-not-edited entry
looks, with the place that was meant missing. The service stops and names the place;
the fix is one line.

**A name is checked by `alo_software::SourceName::checked`**, the same check a place's
name from the rented tool passes, so the two sides of `Bound::keeps_out` are
normalised identically. Matching is exact: `Flathub` is not `flathub`.

**`Described::of` takes a `Bounds` rather than an eighth argument.** Two reasons:
clippy's argument limit, and the better one — both bounds arrive from the same file
with the same owner, and a struct with no `Default` means a machine cannot be
assembled with one bound written and the other forgotten.

**Attribution is the file's owner, exactly as `[questions]`.** Root →
`SetBy::AnAdministrator`; the person → `SetBy::ThisMachine`. A strict list is never
on its own evidence that an organisation wrote it.

## Acceptance criteria and their tests

| Criterion | Test |
|---|---|
| an optional section, documented, with the `format` rule — an older shape carrying it is refused | `describing::tests::places_in_a_shape_that_could_not_carry_them_are_refused`; `what_a_machine_says_about_itself::places_in_an_older_shape_are_refused_off_the_disk` |
| read into `alo_software::Bound`, who set it decided by who owns the file | `describing::tests::places_an_administrator_wrote_are_an_organisations_rule`; `describing::tests::the_same_places_the_person_wrote_name_no_organisation`; `what_a_machine_says_about_itself::places_on_a_disk_are_read_and_attributed_to_whoever_wrote_them` |
| absence is `Bound::Nobodys`, never a permissive list | `describing::tests::a_machine_with_no_applications_section_has_nobodys_rule`; `what_a_machine_says_about_itself::a_description_with_no_places_on_a_disk_has_nobodys_rule` |
| an install from an unnamed place is refused in words naming the organisation, beside the same install with no section | `installing_under_the_description::a_place_the_organisations_description_does_not_name_is_refused_in_its_words` (and `…the_same_list_in_the_persons_own_description_names_no_organisation`) |
| a section that does not hold stops the service | `describing::tests::an_applications_section_that_does_not_hold_is_refused`; `permitted_places::tests::a_name_that_could_never_be_a_place_is_refused`; `permitted_places::tests::a_place_named_twice_is_refused`; `what_a_machine_says_about_itself::places_that_cannot_be_read_stop_the_machine` |

The acceptance test reads the description through the production `describing::read`
as root wrote it, hands `Described::applications()` to `alo_software::Enabled::read`
unchanged, goes through `installing`/`install` around a real `alo_egress::Indicator`,
and renders the refusal in the machine's whole vocabulary from
`starting::what_this_machine_says`. It checks that the refused install left the
indicator quiet and reached the tool not at all, that the permitted place on the same
machine installs, and that the same install succeeds with no section.

## Verification

Run in WSL Ubuntu (kernel 6.18.33.2-microsoft-standard-WSL2, as root) from
`/mnt/c/dev/alo-os-2`, with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-2-72aa7fda7f7de151`. The Windows host cannot
build this workspace (see `one-proxy-machine-wide.md`).

- `cargo fmt --all` — clean.
- `cargo clippy --all-targets -p alo-agentd -- -D warnings` — clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-agentd` — clean.
- `cargo test -p alo-agentd` — exit 0: 424 unit tests passed (11 ignored, as
  before), and every integration target passed, including
  `what_a_machine_says_about_itself` at 16.
- Each evidence test above run on its own with `--exact --include-ignored` — one
  passed each.

Because the suite ran as root, the off-disk attribution test exercised the
organisation's case; run as another user it exercises the person's. The unit tests
exercise both regardless.

Not run: the full workspace suite (the supervisor's), and `alo-software`'s suite,
since nothing in that crate changed.

### Recovered from a parked branch, and gated again

The first handoff for this task was finished and never gated: the supervisor was
restarted while checking it, chose task 5 first on restarting, found a handoff for
task 8, and parked the tree on `parked/task-5-1789560442` unrefused by any gate. The
next worker restored that tree onto `main` (the branch sat directly on `4a887d1`, so
nothing had to be merged), read every line of it, and ran the gates again from
scratch on 2026-09-16, same machine and build directory:

- `cargo fmt --all` — no file changed.
- `cargo clippy --all-targets -p alo-agentd -- -D warnings` — exit 0.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-agentd` — exit 0.
- `cargo test -p alo-agentd --lib` — 424 passed, 11 ignored; every `--test` target —
  `a_machine_on_two_networks` 1 (2 ignored), `a_question_is_bounded_by_the_kernel` 5,
  `a_turn_is_bounded_by_the_kernel` 1, `a_turn_is_refused_when_the_boundary_is_gone` 1,
  `two_doors_on_one_socket` 4, `what_a_machine_says_about_itself` 16; bins and doctests 0.
- Because `alo-agentd` now names `alo-software`, the other crates' tests that read the
  daemon's dependencies were run too: `alo-secrets` `nothing_ships_the_fixture` 2,
  `alo-changing` `no_agents_door_reaches_this_writer` 3, `alo-choosing`
  `no_agents_door_reaches_these_settings` 3, `alo-setting-up`
  `nothing_here_writes_anywhere_else` 4 — all passed.
- Each of the thirteen evidence tests run alone with `--exact` — one passed each.

## Limitations

- **Nothing on a machine installs yet.** The surface a person installs from is the
  shell's, and the daemon does not serve installation. `Described::applications()` is
  the value that surface hands to `alo_software::Enabled::read`; wiring it is the
  shell's work, not this plan's (*nothing in `crates/alo-shell`*).
- **`alo-image` reads only format 1.** The image ships an unmanaged machine with
  format 1 and no sections, which is correct; a built image carrying a policy would
  need that reader taught `2` and `3` first, as the contract already says of
  `[questions]`.

## Proposed updates for the integration owner

- `CHANGELOG.md`: *A machine an organisation manages can now say which places
  applications may be installed from, in its machine description. A place the list
  does not name is refused, and the person is told their organisation set the rule; a
  machine without the list is unchanged. An alo OS too old to enforce the list refuses
  to start rather than ignoring it.*
- `ROADMAP.md` v0.5 *Software*: an organisation's source list is read from the machine
  description (code and tests); installing on a machine is still outstanding.
- `docs/autonomy/QUEUE.md`: task 8 done; task 9 (*An organisation's proxy, read from
  the machine's description*) written and ready, depending on 4 and 8 — the section
  `one-proxy-machine-wide.md` said task 8 was the natural place for, kept as its own
  task so this one stayed inside its acceptance.
