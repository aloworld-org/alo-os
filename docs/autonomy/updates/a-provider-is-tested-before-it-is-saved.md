# A provider is tested before it is saved, on the indicator

**Date:** 2026-09-12
**Workstream:** v0.5, lane B — providers and models, task 2 (*Test a provider
before saving it*)
**Contributor:** Claude (`C:\dev\alo-os-b`, kernel-loop worker)
**Status:** ready for integration

## What was wrong

`docs/features.md`, v0.5: *test a provider before saving it, so a mistyped key
is found now rather than in the middle of a question.*

Half of it existed. `alo_models::Trying` has made the one request — `GET
{endpoint}/v1/models`, the key as a bearer token, ten seconds, no redirect
followed, nothing of the person's in the body — since the provider type did,
and `alo_models::Tried`/`NotTried` already say what came back in words
`alo-saying` collects. What was missing:

- **The request was off the indicator, on purpose.** `trying.rs` argued that a
  button somebody has just pressed is not something they need to be shown.
  The plan decides otherwise, and the reason is the key: a test request is an
  egress somebody asked for, a credential travels with it, and the indicator
  says so. Nothing could put it there — `alo-egress` depends on `alo-models`,
  so the wire cannot reach the indicator.
- **Seven outcomes where the plan asks for three.** `NotTried` tells a person
  seven different things, which is right for the sentence and wrong for the
  decision: a settings panel branches on *save it*, *fix the key or save it
  anyway*, *fix the address or save it anyway*, and nothing said which of the
  seven was which.
- **No account of a provider that cannot be tested**, and no guarantee that
  the key typed into the test never reached a sentence, a rendering or a log.

## What changed

All in `crates/alo-asking`, which already holds `alo-egress`, `alo-models` and
the one convention the wire speaks:

- `src/vetting.rs` — new. `Vetting::provider(&provider, Some(&key)).under(&by,
  &policy, &mut indicator, now)`. Step one: a provider this crate cannot turn
  into somewhere to connect is not guessed at. Step two: a provider somewhere
  else goes on the indicator through `alo_egress::Indicator::beginning` —
  `Leaving::because(by, Why::Fetching, Destination::of(&source))` — which asks
  the rule and shows the line in one call; a provider on this machine
  (`Provider::source()` says `ThisMachine`) leaves nothing and gets no line,
  which is `alo-egress`' own refusal to draw a departure that did not happen.
  Step three: `alo_models::Trying` makes the request, once.
- `src/vetted.rs` — new. `Vetted`: what the test found, and the `Departing`
  it was found with, handed back rather than ended inside — `asked.rs`'s rule,
  so `alo_record::Entry::left` can be written from it. `None` for a provider
  on this machine, where nothing left.
- `src/found.rs` — new. `Found`, exactly three values: `Answered(Tried)`,
  `RefusedTheKey(NotTried)`, `CouldNotBeReached(NotTried)`. The precise reason
  travels inside, so the sentence stays `alo-models`' own. `NotVetted`, the
  four reasons no request was made: `CannotBeTestedFromHere` (this crate's one
  new sentence), `CannotBeShown` (`alo-egress`'), `HeldBack` (the rule, heard
  by the indicator), `Forbidden` (the same rule, heard by the wire — the arm a
  reporter does not get to assume away; nothing left and the line is ended).
- `src/words.rs` — one string, `asking.vetting.cannot-be-tested-from-here`,
  with its translator's note. Every other sentence a test can produce was
  already somebody's.
- `src/lib.rs` — modules, re-exports, and a paragraph in the crate's argument.
- `Cargo.toml` — two dev-dependencies, `alo-choosing` and `alo-saying`, for
  the journey test. Not dependencies: a crate that puts requests on a socket
  has no business holding a door to somebody's settings file. `Cargo.lock`
  follows.
- `tests/a_provider_is_tested_before_it_is_saved.rs` — new. The whole journey
  the way a settings panel will walk it, one test per clause of the
  acceptance, against a counting listener so a retry is a failure rather than
  a thing nobody noticed; plus the two checks that read source: no loop and
  no wait of the door's own, and nothing that ships reaches
  `alo_models::Trying` except the wire itself and this door.
- `tests/nothing_here_keeps_the_key.rs` — new. Reads the three files the way
  `alo-greeting`'s `nothing_here_keeps_the_password` reads that crate: the key
  is named in shipped code exactly five times, all in `vetting.rs` — the field,
  the parameter, the constructor, and the two hand-overs to `alo-models` — and
  no line naming it copies, prints, logs or renders it; `found.rs` and
  `vetted.rs` never name it and hold no `Secret`.
- `tests/what_this_crate_says.rs` — the third sentence, in Greek.
- `crates/alo-models/src/trying.rs` — doc only. The paragraph that argued the
  request belonged off the indicator now says the plan decided otherwise,
  names the door, and why `Trying` stays public.
- `docs/contracts/person-settings.md` — *Writing it* gained one paragraph,
  additively: the writer does not know about the test, and that is the shape.
- `docs/autonomy/v0-5-lane-b-plan.md` — task 2 marked done. Task 3 was
  already written after it.

### User-readable change description

Before a provider is saved, it can be tested with the key you just typed: one
request for its model list, made while the egress indicator shows *@settings
is fetching something from Mistral, in the EU*, under the same rule your
organisation set for questions. It comes back as one of three things — it
answered, it refused the key, or no working provider could be reached there —
each in a sentence that says what to do. A provider that answered is saved; one
that did not has written nothing, and you may still save it if you say so,
because a provider that is down today is not a wrong provider. A provider alo
OS does not know how to reach is not guessed at: you are told it cannot be
tested from here, and saving it works exactly as before. The key is never
logged, never in a sentence and never in an error. A provider on this machine
can be tested under the strictest rule there is, and puts nothing on the
indicator, because nothing leaves.

## Decisions, and why

**The door is in `alo-asking`, not `alo-choosing`.** Two facts decided it.
`alo-egress` is lane A's (ADR 0028), so a third kind of egress — a person's own
— could not be added to the indicator; the test had to be expressed with the
indicator's existing shapes, and `alo-asking` is the crate that already holds
them. And `alo-choosing`'s own manifest test,
`the_persons_writer_has_nothing_to_knock_with`, forbids it a dependency on
`alo-asking` or `alo-capability`, so the settings crate cannot hold a door that
makes a request. The consequence is stated rather than hidden: **the save and
the test are two calls, sequenced by the settings surface**, and the journey
test walks that sequence so the pieces are shown to compose.

**`Why::Fetching`, under the authority the caller names.** `Why` is a closed
list whose widening belongs in ADR 0001, and a test request genuinely fetches
the provider's model list and sends nothing of the person's. Whose authority it
is under is the caller's to say, as `Asking::by` already takes its agent from
its caller: a person pressing *Test* is not an agent, the surface that made the
request on their instruction names itself, and this crate invents no name. The
tests use `@settings`.

**Three values carrying the precise reason, not three sentences.** A bucket
sentence would send a person to the wrong place: *nothing answered — check the
address* for a provider that answered 503 is wrong advice. The three values
are what a panel branches on; `alo-models` keeps the sentence, and it is
already collected.

**`NotVetted` is the test not happening, not a fourth outcome.** Nothing was
sent in any of its four cases, and in all four the save proceeds as it does
today — a test that did not happen found nothing wrong. `HeldBack` and
`Forbidden` are one rule heard twice; the indicator's hearing is the primary
(its refusal is the recordable one), the wire's is the arm a reporter does not
get to assume passes, and if it is ever taken the line comes off first.

**A provider on this machine is tested without a line.** `Destination::of`
refuses `ThisMachine`, and `alo_models::trying` already guarantees a local
provider can be tested under `ThisMachineOnly`. Branching on
`Provider::source()` — the one rule about what an address is — keeps both.

**The wire stays in `alo-models`, unchanged, and the second road is read off
the repository.** `Trying` has to stay public for this door to use it; a
workspace-reading test holds that nothing else that ships names it, so a
future surface reaching for the wire directly fails a test that names
`Vetting`.

**"A provider this crate does not know" is an address with no host or a
scheme alo OS does not open.** Today every provider is OpenAI-compatible by
construction, so the case is reachable only through a by-hand `Provider`;
the constraint is still held by a test, because the plan names it and because
a second provider kind is the likely future.

## Verification

Run inside WSL Ubuntu against this checkout, `CARGO_TARGET_DIR` the
supervisor's own directory for it (`/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`),
after touching changed files (the `/mnt/c` mtime quirk):

| Command | Result |
|---|---|
| `cargo fmt --all` | clean, no diff |
| `cargo clippy -p alo-asking -p alo-models --all-targets -- -D warnings` | clean |
| `cargo test -p alo-asking` | 92 unit + 27 integration + doctests pass |
| `cargo test -p alo-models` | 152 unit + 46 integration pass |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-asking -p alo-models --no-deps` | clean |
| `cargo test -p alo-saying -p alo-collected -p alo-choosing` | pass — the collector, the word-collection check and the unchanged writer |
| each evidence test with `--exact --include-ignored` | 1 passed, each |

The full workspace suite was not run here, per the task's instruction; the
supervisor runs it. Workspace-wide clippy was not run either, for the same
reason; the two touched crates and everything that depends on `alo-asking`'s
words were.

**Hardware:** nothing here touches a device. Every listener is the test's own,
on loopback; the hosted provider that is never reached is `https://0.0.0.0:1`.
No external provider is contacted.

### Evidence, one line per acceptance criterion

| Criterion | Test |
|---|---|
| One request, for the model list, with the key — and a working provider is then saved | `a_provider_is_tested_before_it_is_saved::one_request_is_made_with_the_key_and_a_provider_that_answers_is_saved` |
| The outcome is one of exactly three values | `found::tests::every_answer_from_the_wire_is_one_of_three_things_to_do` |
| Each is a sentence `alo-saying` collects | `a_provider_is_tested_before_it_is_saved::every_sentence_a_test_can_say_is_one_the_machine_can_say` |
| The key is never logged, in a sentence or in an error — read off the source | `nothing_here_keeps_the_key::the_key_is_a_borrow_a_parameter_a_field_and_two_hand_overs_and_nothing_else`, `nothing_on_a_line_naming_the_key_copies_prints_or_keeps_it`, `what_the_test_found_is_made_from_nothing_that_holds_the_key` |
| … and never in a rendering, behaviourally | `vetting::tests::a_key_the_provider_refuses_is_found_now_and_the_sentence_does_not_carry_it` |
| A refused key writes nothing; the person may still save it | `a_provider_is_tested_before_it_is_saved::a_key_the_provider_refuses_writes_nothing_and_the_person_may_still_save_it` |
| An unreachable provider writes nothing; the person may still save it | `a_provider_is_tested_before_it_is_saved::a_provider_that_could_not_be_reached_is_shown_leaving_writes_nothing_and_may_be_saved` |
| The request goes through `alo-egress`' accounting and the indicator says so | `vetting::tests::a_test_request_that_leaves_is_shown_leaving_and_the_departure_comes_back` |
| … and a rule that forbids it sends nothing, the indicator quiet | `vetting::tests::a_rule_that_forbids_it_sends_nothing_and_the_indicator_stays_quiet`, `a_provider_is_tested_before_it_is_saved::a_rule_that_forbids_the_test_sends_nothing_and_the_save_proceeds_as_it_does_today` |
| A provider on this machine is tested and puts nothing on the indicator | `vetting::tests::a_provider_on_this_machine_answers_with_what_it_offers_and_leaves_nothing` |
| No endpoint is guessed for a provider this crate does not know; *cannot be tested from here*; the save proceeds | `vetting::tests::a_provider_this_crate_does_not_know_cannot_be_tested_from_here_and_nothing_is_sent`, `a_provider_is_tested_before_it_is_saved::a_provider_this_crate_does_not_know_is_not_guessed_at_and_the_save_proceeds` |
| No retry loop, no timeout longer than a person would wait | `a_provider_is_tested_before_it_is_saved::the_test_is_one_request_with_no_retry_and_no_long_wait` |
| No second road to the wire | `a_provider_is_tested_before_it_is_saved::nothing_that_ships_reaches_the_wire_except_through_the_door_that_shows_it` |
| The new sentence is read in the person's language | `what_this_crate_says::every_sentence_is_read_in_the_language_the_person_reads` |

## Remaining limitations

- **The answered-and-shown combination is not exercised over a socket.** A
  line goes on the indicator only for an address that is not this machine, and
  a plain-HTTP listener a test can own is on this machine; a TLS listener at
  another address is `alo-agentd`'s fixture (rcgen, rustls, the
  `trust-a-test-authority` feature) and was not copied here. The order —
  departure before request — is one function and is exercised on the path
  where nothing answered. Copying that fixture into `alo-asking` is a
  reasonable follow-up if a reviewer wants the socket.
- **The save is two calls.** `Vetting::under` then `Choosing::adding`,
  sequenced by the settings surface (lane A's `alo-shell`). The alternative —
  a door that tests and saves — would need `alo-choosing` to depend on
  `alo-asking`, which its own manifest test forbids for a reason this report
  agrees with.
- **The key does not go to a keyring yet**, because nothing in this repository
  stores one: `alo-secrets` looks a key up and has no door to keep one. The
  plan's *before the key goes to the keyring* is therefore satisfied trivially
  today and will need the same ordering when a storing door exists.
- **A provider's model list still cannot be saved.** `Found::Answered` carries
  `Tried::into_models()`, and `alo-choosing`'s writer refuses a provider with
  models on it (`NotExpressible`), as it did before this task. Not in scope;
  noted so the settings surface does not try.
- **`alo_models::Trying` resolves the name itself.** It runs from a settings
  panel outside any kernel boundary, so ADR 0020's resolve-first shape does not
  apply; stated so nobody reads the door as bounded.

## Proposed shared-document updates

**CHANGELOG.md**, under Unreleased / Settings:

> A provider can be tested before it is saved: one request for its model list
> with the key you typed, shown on the egress indicator while it happens and
> under your organisation's rule. It comes back as one of three things — it
> answered, it refused the key, or no working provider could be reached — and a
> provider that did not work writes nothing unless you say to save it anyway.
> A provider alo OS cannot reach is not guessed at, and the key is never
> logged, never in a sentence and never in an error.

**QUEUE.md / STATE.md:** lane B task 2 done; tasks 3 and 4 are ready and
depend on nothing.
