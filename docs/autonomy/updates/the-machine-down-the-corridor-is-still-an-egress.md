# The machine down the corridor is still an egress

- Date: 2026-09-13
- Workstream: v0.5 the local network, task 3 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, by hand — no worker could run
- Status: **the promise, and the exception that would have ended it**

`docs/features.md` promises, at v0.5: *a machine without a GPU discovers the one
with it, and the agents just work. The inference never leaves the building; it
moves down the corridor.*

And ADR 0003's condition on it, which is the whole reason the promise is
keepable: **it is still egress, and the indicator still fires.**

## The sentence this task exists to refuse

*"It only went to the machine down the hall."*

That is the shape every guarantee dies in — not a decision to break it, but an
exception that sounds too small to matter. The corridor is short, the machine
belongs to the same office, the question never touches the internet. None of
that makes the question not have left, and a person who cannot see the short
journeys has no way to check any claim made about the long ones.

`alo_models::InferenceSource::causes_egress` has been true for `PairedMachine`
since item 18a. This change is the first thing that could have quietly
disagreed with it.

## What changed

**There was no path to a machine on this network.** Three doors refused one —
`to_a_provider`, `to_this_machine`, `to_a_service_on_this_machine` — each with
`Miswired::NoPathToAPairedMachine`, whose sentence said *there is no path to one
in this repository yet*. There is now, and the sentence would have gone on
saying otherwise with nothing failing. It is `Miswired::BelongsDownTheCorridor`,
and it names the door: **a refusal that describes the world rather than the
caller's mistake goes stale without anything catching it.**

The new door is `crates/alo-asking/src/corridor.rs` and
`Asking::to_a_paired_machine`. It is built exactly like the provider door and
deliberately not at all like the two local ones: it reaches the wire holding an
`alo_egress::Departing`, which only `Indicator::beginning` makes, so the
question cannot leave without a person having been shown it leaving. The doors
that skip the indicator are the doors where nothing goes anywhere, and this is
not one of them however short the corridor is.

## The question travels; the grant does not

`DownTheCorridor::paired` decides one thing and one thing only: whether a
pairing made by two people permits **this** machine to ask **that** machine's
models, at the moment of asking. Nothing constructed here travels with the
question.

An unpaired machine offering inference reaches that constructor and is refused,
however convenient it would be — the machine is right there, it has the GPU, its
answer would be better. None of that is a pairing, and there is no constructor
that skips one. So is a pairing that has expired, and one that was revoked, and
one made for the workspace rather than for the models: all four are the same
fact about this machine, with no arm distinguishing them, because telling a
caller *which kind* of not-paired a machine is would be telling them how to
become paired.

**And it is never a fallback.** A question reaches this door because the
person's permission named that machine. A permission naming a provider is
refused here even with the corridor paired, open and waiting —
`the_machine_down_the_corridor_is_not_a_fallback_for_anything`. A paired machine
is the most tempting substitution in the whole document: the answer would be
better, the wait shorter, and the person would never know their question left.

## The other end of the corridor

The acceptance's last clause is the answering machine's, and it needed a new
kind of record entry: `Happened::AnsweredForAnotherMachine`, additive with
`format` staying `1` as `docs/contracts/record-file.md` requires.

**It names the machine and not the agent**, which is the decision in it. There
*was* an agent — one on the machine that asked. But its name is a name on
somebody else's machine, and this one has no way to check it; writing it here
would put somebody else's claim into a record that people read as a statement of
fact. What this machine can stand behind is which machine it paired with,
because its own person agreed to that. `Happened::agent` therefore answers
`None` for this entry, and the module says why it is a different `None` from the
errand's.

What was asked is not kept, for the reason every other entry gives, and with
more force: the question is somebody else's.

## What the compiler made me answer

Adding a `Happened` arm stopped five `match`es compiling at once — *is there an
agent, is this an errand, what ran, where was it stopped, which approval* — and
each had to be answered rather than defaulted. That is the boundary working:
the record cannot gain a kind of event whose relationship to the questions
people ask of a record is left undecided.

Two stale counts came out of it. `happened.rs` opened *Seven things can happen
on this machine, and the record keeps all seven*, and `alo-recounting`'s word
list opened with *Eighteen* and *Ten clauses* while holding nineteen and twelve
— **an off-by-one that predates this change** and that nothing was checking.
Both now say what is true.

## Verified

Windows, this development machine:

| Target | Result |
|---|---|
| `the_machine_down_the_corridor_is_still_an_egress` | 7 passed, 0 failed |
| `alo-asking` | 98 unit + every integration test green |
| `alo-record`, `alo-saying`, `alo-collected` | green |
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -D warnings` | clean |
| the rest of the workspace | green but for `alo-recounting` |

`alo-recounting` fails **identically to before this change** — 6, 5, 5, 3 and 2
in the same five binaries — which is the Windows-only platform split in
`docs/quirks.md`, not anything here.

**What no test shows is two machines.** Both ends are on this host over a real
socket, which is what shows the request is right, the indicator fires, the
answer comes back and the record is written. Whether an office machine reaches
the studio machine across a switch is owed to two machines, and so is everything
about the connection between them: **there is no cryptography here and none is
claimed.** Proving that the machine which paired is the machine that later asks
is not built, and nothing in this change implies it is.

**CHANGELOG.md** — nothing user-visible: no surface calls this door yet.
**ROADMAP.md** — *one GPU box serves the office* is built; the box stays
unticked until a machine does it.
