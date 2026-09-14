# A question from a paired machine is answered by this machine's own model, and the person's door reaches a remote turn

- Date: 2026-09-14
- Workstream: v0.5 the local network, task 11 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, as a development worker in `C:\dev\alo-os-claude`
- Status: **ready for integration**

Task 10 bound the port and left two things it could not decide as side
effects: a proven, permitted question on `/v1/chat/completions` was answered
`not-answered-here`, and a change a paired machine proposed waited for this
machine's person with no request on the person's door that reached it. This
task decides both, in the crates that own them.

## What changed, in a sentence a person can read

A machine running alo OS now answers a question from a machine it is paired
with, using the model its own person chose and nothing else: never a
provider, never a third machine. The answer leaves under the machine's egress
indicator, and the record says a question was answered for that machine and
that the answer left. A machine whose person chose a provider for their own
questions refuses to answer for anybody, in words. And a change an agent on a
paired machine proposed is now listed for the person here with that machine
named, approved or declined from their own shell, and runs exactly once when
approved — with the pairing asked again at the moment of approval, so one
revoked in between stops the change.

## What changed, by crate

**`crates/alo-turn`** — the door that answers for another machine.

- `src/answering_for.rs` (new): `Machine::answering_for` puts a question
  from a proven `Origin` to a runtime on this machine — the only door it
  reaches is `alo_asking::Asking::to_this_machine`, so a permission for
  anywhere else is refused as miswired before anything is asked — writes
  `Entry::answered_for` with the origin named, and puts the answer going
  back on the indicator under the origin's principal as `Why::Sending` to
  the machine by its name; `Machine::answer_returned` writes `Entry::left`
  stamped with the origin and takes the line off. A rule that refuses is
  written down as `held back`, stamped with the origin. `AnsweredFor` holds
  the answer and the departure. `Arriving` gains the same two doors,
  forwarding to the machine it holds, so a question is answered while a
  remote turn holds the machine without touching the turn.
- `src/arriving.rs`: `pub(crate) fn machine`, for the file above.

**`crates/alo-asking`** — the receiving side of the one shape.

- `src/openai.rs`: `Sent` and `Spoke` are now read and written both ways;
  `a_question_off_the_wire(body, of_model)` reads exactly what `put` sends
  and refuses anything else — a field the shape has no place for, a stream,
  no message, two, one in another role, one that asks nothing — and makes
  the question for the model the answering machine's person chose;
  `an_answer_on_the_wire(&Answer)` writes the reply `put` reads. `NotHeard`
  is the closed list of refusals.

**`crates/alo-nearby`** — `http::a_json_reply`, the same framing as
`a_reply` saying `application/json`, for the one reply whose body is not a
line of text.

**`crates/alo-protocol`** — `Standing::from_a_machine` and `Standing::from`,
an additive `from` field naming the machine a change was proposed from;
`ToAPerson::waiting_from_a_machine`. Absent, not empty, for a local change,
so every message written before reads back as written.

**`crates/alo-agentd`** — the daemon answers, and the person's door reaches
the remote turn.

- `src/questioned.rs`: the question door now answers. The order is the
  proof, the pairing's list, what the person here chose (looked for at every
  question), the body, and this machine's own model through whatever holds
  the machine — the remote turn or the machine. Six words on the wire:
  `not-permitted` (403), `not-a-question` (400), `not-answered-here` (503),
  `answers-elsewhere` (503), `nothing-answered-here` (503), `no-model-here`
  (404), and `200` with the answer as JSON. A held-back answer and an origin
  that cannot be shown close the connection with nothing on it. `Replied` is
  what became of the reply; `NotServed::NothingIsWrittenDown` when an answer
  left with no record of it.
- `src/reaching.rs` (new): the three requests about a turn, answered against
  `alo_turn::Arriving` — `waiting` stamped with the origin machine's name,
  `approve` under the network's lock so the pairing is asked at the moment,
  `decline` as a whole answer — with a number nothing is waiting under refused
  in the same words as on a local turn.
- `src/holding.rs`: `Holding::TheNetwork` carries the doorway, the network
  lock and the `Questions`; `Holding::remote` hands out the remote turn and
  the lock together.
- `src/answering.rs`: dispatches to `reaching` when a remote turn holds the
  machine; `nothing_is_waiting` shared.
- `src/hearing.rs`: `Judging` carries the `Questions`.
- `src/serving.rs`: the network loop hands the `Questions` and the lock into
  the holding; the fixture takes what answers questions as an argument.
- `src/questions.rs`: `a_new_turn` is also called before every remote
  question; the `already_found` test seam survives it, as a runtime on a
  real machine does.

**`docs/contracts/local-network-wire.md`**: the question path in full — the
order, the words, the `200` answer and its shape, additively.
**`docs/contracts/daemon-protocol.md`**: `from` on a waiting change, and
what `approve` asks at the moment for one. **The plan**: task 11 done, task
12 written.

## Decisions

**Which model answers for another machine is the person's own choice for
their own questions.** The plan said the setting is the person's and no
default may decide it; task 10's report expected a setting of its own. There
is one already: what the person chose under `[answers]` in their settings
file (ADR 0016, `docs/contracts/person-settings.md`), and a pairing they
confirmed with *may ask this machine's models* on its list is the second half
of the consent — two things the person did, and nothing this repository
decided for them. A second key naming a model *for others* would be a second
settings system for one owner, which `alo-choosing` refuses in its own
header, and nothing in `docs/features.md` promises one. So the daemon reads
the person's choice at every remote question through the same `Questions` it
reads for a local turn, and:

- a machine where nothing is chosen, nothing is running, or the file does
  not hold answers *not answered here*, the word it already had;
- a machine whose person chose a **provider** answers *answers elsewhere*
  and puts the question nowhere. ADR 0008 runs both ways and ADR 0003 makes
  this the sharp case: a question that travelled one corridor must not
  travel a second, and a machine forwards for nobody. This is the refusal the
  plan asked for in words, and it is tested against a real settings file
  naming Mistral.

**The model named in the question is checked and not used.** The corridor
puts a question with the asking person's model name in it, because that is
the shape; what runs on the answering machine is that machine's person's
setting, so the question is put to *their* model and the reply names it.
`a_question_off_the_wire` holds the body's model to the rule that a question
names one, then makes the question for the chosen model — which also keeps
`Question::text` private to `alo-asking`, where it always was. The asking
side's `Answer` still names the model it asked for, by
`alo_asking::Answer::new`'s own rule (nothing about an answer's origin is read
off a reply), so what reception reads is what reception asked; what really
answered is in the studio's reply and record.

**The door is on `Machine`, and `Arriving` forwards to it.** `arriving.rs`
says a remote turn puts no question, and the plan says the question is put
inside no turn of the asking machine's. A question is not a verb: it asks no
grant, spends no approval and begins no turn, so it is the machine's, the way
`a_pairing_was_kept` is. But a question can arrive while a remote turn —
another machine's, or the same machine's — holds the machine, and the record
and indicator are behind that turn; so `Arriving` has the same two doors,
forwarding, and the entries are the machine's own rather than stamped with
that turn's origin. The daemon dispatches on what the doorway holds.

**Answered first, then the departure; two entries.** The model is asked, then
`answered for another machine` is written, then the egress rule is asked and
the answer put on the indicator, then the bytes go, then `left` is written and
the line comes off — the order `Arriving::departing` and `returned` keep for a
verb's answer. A rule that says nothing leaves therefore leaves `answered for`
beside `held back`, both true: the model answered here and the answer did not
go. A model that did not answer writes nothing, `alo_turn::unanswered`'s
argument unchanged. The departure is spent whatever the socket write said, as
the verb wire spends it: enough of it went that the indicator cannot say it
did not.

**The reply is JSON under `application/json`.** `alo_nearby::http::a_reply`
says `text/plain` for every word on the port; an answer is the one body that
is not a word, and a reader of the convention is told the truth about it.
The framing is byte-identical, held by a test.

**`from` on a waiting change is a stamp, not a second shape.** What the
person approves is the sentence, and a sentence for a change from the
machine down the corridor has to say so. `Standing::from_a_machine` follows
`alo_record::Entry::from_another_machine`: absent for a local change, so a
reader that has never heard of it reads an old message as written. An older
reader meeting the field on a remote change refuses that one message in
words, which is the contract's rule for anything additive.

**The lock is taken for the approval.** `Arriving::approving` asks the
pairings at the moment; the person's door holds `TheNetwork`'s lock for
exactly that call, so a revocation from the person's surface between the
proposal and the approval stops the change at the moment it would have run,
and the refusal names the machine.

**The `already_found` seam survives a new turn.** Calling `a_new_turn` before
every remote question is what makes a model chosen this morning answer this
afternoon; a test seam that forgot its runtime on that call would be testing
a machine that finds nothing on its second question. It is `cfg(test)` and
pinned, and a test that wants the look itself writes a settings file — one
does, for the provider refusal.

## Acceptance, and the test behind each line

| The plan says | The test |
|---|---|
| a proven question from a pairing that permits asking this machine's models is put to this machine's **own** model and to nothing else, tested by a machine whose person chose a provider refusing the question in words | `alo-agentd` · `questioned::tests::a_machine_whose_person_chose_a_provider_refuses_the_question_in_words`; `alo-turn` · `answering_for::tests::a_permission_for_anywhere_else_asks_this_machines_runtime_nothing` |
| inside no turn of the asking machine's, recorded as `Entry::answered_for` with the origin named, its answer leaving under a departure the indicator shows and the record keeps, answered on the wire in the OpenAI-compatible shape the corridor reads | `alo-agentd` · `questioned::tests::a_proven_question_is_answered_by_this_machines_own_model_and_leaves_under_a_departure`; `alo-turn` · `answering_for::tests::a_question_from_a_paired_machine_is_answered_here_and_its_answer_leaves_under_a_departure`; end to end through the corridor, `alo-agentd` · `serving::tests::a_question_from_a_paired_machine_is_answered_on_the_port_by_this_machines_own_model`; the shape both ways, `alo-asking` · `openai::tests::what_is_written_as_an_answer_is_read_back_by_the_asking_side` |
| a question from a pairing that does not permit it is still refused before anybody knows what it asked | `alo-agentd` · `questioned::tests::a_pairing_for_something_else_does_not_open_this_door` |
| a machine where nothing has been chosen to answer a question says so in the word it already has | `alo-agentd` · `questioned::tests::a_question_is_proven_before_anything_else_and_a_replay_is_refused` |
| the person's door approves, declines and lists a change a paired machine proposed, through `alo_turn::Arriving`, with one approval causing exactly one execution there — the refusal paths beside the road | `alo-agentd` · `reaching::tests::a_change_from_a_paired_machine_is_listed_approved_and_runs_once`, `reaching::tests::a_change_from_a_paired_machine_the_person_declined_runs_nothing`, `reaching::tests::an_approval_after_the_pairing_was_revoked_runs_nothing`, `reaching::tests::a_number_nobody_proposed_answers_nothing_on_a_remote_turn_or_without_one`; on the running service, `serving::tests::a_change_a_paired_machine_proposed_is_approved_from_the_persons_shell`; the shape, `alo-protocol` · `to_a_person::tests::what_a_paired_machine_proposed_names_the_machine_on_every_change` |
| the proof is judged before anything else through the same `Seen` | `alo-agentd` · `questioned::tests::a_question_is_proven_before_anything_else_and_a_replay_is_refused`, `questioned::tests::an_unpaired_machines_question_is_refused_at_the_proof` |

And beside them: a body that proved itself and is not a question is refused
as such (`questioned::tests::a_proven_body_that_is_not_a_question_is_refused_as_not_a_question`,
`alo-asking` · `openai::tests::a_body_that_is_not_one_question_from_the_person_is_refused`);
a model that does not answer says so and writes nothing
(`questioned::tests::a_model_here_that_does_not_answer_says_so_and_writes_nothing`,
`alo-turn` · `answering_for::tests::a_model_that_does_not_answer_writes_nothing_and_shows_nothing`);
a rule that says nothing leaves holds the answer back and writes it down
(`questioned::tests::a_rule_that_says_nothing_leaves_holds_the_answer_back`,
`alo-turn` · `answering_for::tests::a_rule_that_says_nothing_leaves_holds_the_answer_back_and_writes_it_down`);
the same door through a remote turn touches nothing on the turn
(`alo-turn` · `answering_for::tests::a_question_is_answered_through_a_remote_turn_without_touching_the_turn`);
a JSON reply is the same framing (`alo-nearby` ·
`http::tests::a_json_reply_is_the_same_framing_and_says_what_its_body_is`);
and a local change carries no `from` while an old message reads back
(`alo-protocol` · `standing::tests::a_change_from_a_paired_machine_says_which_and_a_local_one_says_nothing`).

## Verified

Ubuntu under WSL, this development machine, from the checkout, each run in
the foreground and waited on, building in
`/root/alo-builds/alo-os-claude-bd192ccccbc3745b`:

| Gate | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, zero warnings, exit 0 |
| `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps` on the five crates touched | clean, exit 0 |
| `cargo test -p alo-agentd` | **268 unit** (was 257), 5 + 1 + 1 + 4 + 12 integration, all green |
| `cargo test -p alo-turn` | 89 unit (was 84), 2 + 4 + 12 + 6 + 1, 2 doctests, all green |
| `cargo test -p alo-asking` | 104 unit (was 101), every integration target green |
| `cargo test -p alo-nearby` | 125 unit (was 124), 9 + 12 + 5 + 9, 1 doctest, all green |
| `cargo test -p alo-protocol` | 94 unit (was 93), 8 + 7 + 6 + 1, all green |

The full workspace suite was not run here, as instructed; the supervisor
runs it. Every crate that names `alo-protocol`, `alo-asking`, `alo-nearby`
or `alo-turn` compiles under the workspace clippy above; no public signature
any of them already had changed, `Standing` gained a defaulted field, and
`Questioned` is `non_exhaustive` and gained arms.

**What no test shows is two chassis.** Both machines are one process on one
host over real sockets, as in every report of this plan; the end-to-end test
puts the question through `alo-asking`'s own corridor exactly as reception's
machine would, to the daemon's real port, and reads the answer back through
the same reader a provider's answer goes through.

## Second pass: why the first handoff was refused, and what was changed

The first handoff of this task passed every gate on its own tree and on the
combined tree, twice over, and its evidence stood up both times. On the third
evidence run, after a push race rebased the tree once more, the supervisor
refused `reaching::tests::a_change_from_a_paired_machine_is_listed_approved_and_runs_once`
with *cargo printed 0 test results*. What cargo had printed was nothing:
`wsl.exe` had not reached the distribution, and said so —
`Wsl/Service/0x8007274c`, a connection that timed out before `bash` ran.

The supervisor has a classifier for exactly that sentence
(`gates::the_machine_rather_than_the_work`), and it did not fire, because
`wsl.exe` writes its own words in UTF-16LE and the supervisor read the pipe as
UTF-8: what reached the classifier was `W s l / S e r v i c e /`, a NUL between
every letter. Its own test held the sentence in UTF-8, which is not how the
bridge ever prints it. That is visible in the refusal quoted to this worker,
where the message is spaced out character by character.

Nothing in the product changed on this pass; the code of task 11 is as the
first worker left it, and every gate was run again on it. What changed is the
supervisor, in the smallest way that makes the refusal read what was said:

- `tools/kernel-loop/src/what_it_printed.rs` (new): everything a bridged
  process printed goes through one reader that tells a run of UTF-16LE from
  UTF-8 by the byte UTF-8 output never contains, decodes each as what it is,
  drops a byte-order mark, and lets the two follow one another in one stream
  (the bridge's warning, then cargo's output). A character outside Latin-1 in
  the bridge's own text degrades to a replacement rather than taking the
  error code beside it with it.
- `tools/kernel-loop/src/gates.rs` and `src/evidence.rs` read what a gate or
  an evidence run printed through it. The classification was not touched: a
  WSL timeout under an evidence test is now `NOT_READY_TO_BE_GATED`, and the
  loop runs the gates again rather than sending a worker at a test that
  passed, which is what the supervisor's own rustdoc already promised. No
  gate is weaker: a build error, a failed test, a name that matches nothing
  are refused in the same words as before, by the same tests.

| What is shown | The test |
|---|---|
| the bytes `wsl.exe` printed on 2026-09-14 are read as the sentence | `tools/kernel-loop` · `what_it_printed::tests::what_wsl_prints_in_utf16_is_read_as_the_words` |
| the same bytes, through the evidence reader, are the machine's fault and not the evidence's, and the tail quoted is words | `tools/kernel-loop` · `evidence::tests::a_distribution_that_did_not_answer_is_the_machine_in_the_bytes_wsl_prints` |
| cargo's UTF-8 is left as it is, and a bridge warning ahead of it does not hide the one result line | `what_it_printed::tests::what_the_linux_side_prints_in_utf8_is_left_as_it_is`, `what_it_printed::tests::a_warning_from_the_bridge_before_cargos_output_reads_as_both` |

Verified on this pass, Ubuntu under WSL, each run in the foreground and
waited on: `cargo fmt --all` clean at the root and in `tools/kernel-loop`;
`cargo clippy --workspace --all-targets -- -D warnings` clean; the
supervisor's `cargo clippy --all-targets -- -D warnings` clean and its
`cargo test` 113 passed (was 109); `cargo test -p` for `alo-agentd` (268
unit, every integration target), `alo-turn` (89), `alo-asking` (104),
`alo-nearby` (125) and `alo-protocol` (94), all green; and each of the three
new tests run alone by `--exact` as the supervisor runs evidence, one result
line each. The full workspace suite is the supervisor's.

## Remaining limitations, honestly

- **The person's surface still confirms a pairing outside the loop**, and
  pairings are not kept between restarts; task 12 is written for both.
- **No shell shows a change from a paired machine or keeps a name**, so
  `from` carries a machine's identity until a shell keeps names (`NoNameYet`).
- **A question is answered by the person's own choice**, not by a setting
  of its own; the decision above says why, and a setting can be added
  additively if a person ever wants to answer for others with a different
  model than they use themselves.
- **The asking side names the model it asked for**, not the one that
  answered; the record on the answering machine and the reply both name the
  latter. Reading it off the reply would be the asking side letting the far
  end say what a person is told, which `alo_asking::Answer` refuses.
- **Proven, not private**, unchanged from ADR 0031.

## Proposed updates to the shared documents

- **CHANGELOG.md** — under the next release: *A machine paired with another
  now answers its questions with the model its own person chose, and with
  nothing else — never a provider, never a third machine; a machine set to a
  provider says so and answers for nobody. The answer leaves under the
  egress indicator and is written down as answered for that machine and as
  having left. A change an agent on a paired machine proposes is listed for
  the person here with the machine named, approved or declined from their own
  shell, runs once, and is stopped if the pairing was revoked in between.*
- **ROADMAP.md** — no box moves; *one GPU box serves the office* now has an
  answering side, and stays unticked until two machines do it.
- **QUEUE.md** — task 11 of the v0.5 local-network plan done; task 12 (*the
  person's door proposes, confirms and revokes a pairing, and a pairing
  outlives a restart*) ready, depending on 10 and 11.
