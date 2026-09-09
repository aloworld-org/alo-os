# alo OS — ROADMAP.md

**Execution resumed 2026-09-07.** The full v0.01 release is now pursued by the
repository-owned Rust development loop (`tools/dev-loop`).

Native reader state (2026-09-09): whole-name page selection is now bound to one
live strip publication, with stale/foreign/retired readers refusing permanently.
This is a complete reader-state component; externalized position wording, input
navigation and transactional reader composition remain next. No feature tick.
Evidence: `docs/autonomy/updates/mapping-bound-native-name-readers.md`.

Contributor reconciliation (2026-09-09): machine-description format 2 now loads
inference policy and retains owner-based attribution, with invalid rules refusing
startup. This supersedes the earlier production-no-bound limitation below.
`docs/autonomy/updates/an-organisations-rule-off-a-disk.md` reports 134 passing
workspace binaries and targeted mutations without exact commands. Startup's bound
handoff remains inspection-only, person-owned disk evidence depends on runner UID,
and alo-image still reads only unmanaged format 1. No broader policy/release tick.
`docs/autonomy/updates/a-loop-that-chose-an-audit-heading.md` reports six selection
tests: only headings inside Tasks count, and scheduled work is skipped. No ready
backend plan entry is not a release-completion verdict.

`docs/autonomy/DELIVERY.md` orders the Linux, compositor, session, agent and image
work and the final physical acceptance. Each completed step is gated and pushed;
these checkpoints do not reduce release scope or discharge hardware obligations.
The owner's direct-to-main collaboration policy is in
`docs/autonomy/SHARED_MAIN.md`: separate checkouts, pull before each task,
integrate concurrent commits and recheck before pushing.
Concurrent development approved 2026-09-09: desktop and Claude may work in
separate checkouts at once; existing per-test kernel locks serialize shared
kernel fixtures. This supersedes the temporary single-workstream restriction.
A 12 GiB C: preflight reserve still guards new tasks and each gate phase.
Repairable desktop worker/gate failures now enter bounded diagnosis and repair
of the same unfinished task, then full verification again before publication.
Safety/authority handoffs and exhausted recovery preserve work. Details:
`docs/autonomy/updates/repairing-an-unfinished-desktop-task.md`. Backend supervisor
305cab1/6704741 also add live status, C: reserve and an OS-held Windows lock;
these development safeguards do not move release requirements.
Desktop recovery (2026-09-09): restored missing bpffs during a verified idle
handoff and integrated the preserved label-rendering commit with current main.
An owned WSL stdin lease now spans worker/Windows/Linux/publication phases;
no automatic mount, service or WSL restart is introduced. Combined-tree recovery
verification and clean-tree loop restart are tracked in
`docs/autonomy/updates/keeping-ubuntu-active-through-desktop-verification.md`.
Credential reports `connections-come-and-go.md` and
`a-key-over-a-verified-connection.md` under `docs/autonomy/updates/` now establish
fixture connection lifetime/concurrency and authenticated daemon HTTPS under
test trust. Real-session logout remains session-integration work; no release
checkbox moves. Subsequent `what-the-server-saw.md` (5e3a66b) supplies separate
untrusted-issuer/identity rejection with server-side handshake/application-data
instrumentation, a positive control and cross-thread test-trust isolation. This
passed in the complete combined-tree recovery gates. All Windows/Linux workspace,
rustdoc, BPF and label/control/nested graphical checks passed on 5e3a66b plus
the preserved labels and lease fix; normal publication and clean-tree restart
follow. No feature/release checkbox is promoted by that development evidence.
Contributor reconciliation (2026-09-09): policy-before-credential ordering and
explicit organisation/personal attribution are implemented; production supplied
no bound at that checkpoint (superseded by format 2 loading above). Pre-turn
refusal recording is now implemented, with persisted
response/record agreement and write-failure tests; see
`docs/autonomy/updates/recording-a-question-nobody-was-asked.md`. Contributor
mutation evidence has no exact command/results listing; settings/release remain
unchecked. `docs/autonomy/updates/an-administrator-set-that-rule.md`.
Company-managed cleanup belongs to the company administrator. Details:
`docs/autonomy/updates/storage-aware-desktop-loop.md`. Release scope is unchanged.
Recovery verified (2026-09-09): C: had recovered to 82.3 GiB at restart review;
the cause was not measured by this worker. The preserved painter, smaller build
profiles and updated supervisor passed all combined-tree publication gates, the
supervisor's 17 tests and full-frame graphical checks. Normal publication and
clean-tree restart follow; no further cleanup is requested. Historical
cleanup evidence: `docs/autonomy/updates/reclaiming-development-storage.md`.
Current coordination: `docs/autonomy/updates/concurrent-development-with-shared-kernel-tests.md`.
New work uses descriptive task names and individual reports. Only the integration
owner consolidates those reports into this roadmap and the shared progress files;
historical queue identifiers remain secondary cross-references.

Credential continuation reconciled 2026-09-09 from `8096910` and
`docs/autonomy/updates/a-real-keyring-answers.md`: an isolated Secret Service
fixture demonstrates DH retrieval, Missing and schema isolation (also passed in
combined-tree validation here). The later `9109875` and
`docs/autonomy/updates/the-four-refusals-against-a-real-store.md` add real
Locked/Denied evidence and correct Denied classification; its combined re-gate
passed. `c4e20c7` and `docs/autonomy/updates/a-bus-with-nothing-on-it.md` subsequently
add live empty-bus Unavailable evidence and avoid stacked absent-service timeouts;
that combined re-gate passed. The report
`docs/autonomy/updates/the-daemon-asks-for-the-key.md`, reconciled 2026-09-09,
adds production daemon lookup, four distinct refusals and hermetic test defaults.
It corrects the earlier environment assumption: a session keyring was activated
on the build host. The report lists no exact commands/results, so no additional
executed gate is claimed here. End-to-end daemon retrieval, authenticated HTTPS,
lifetime/concurrency and logout remain open. No keyring or provider-completion
checkbox is moved.

Shared credential fixture report reconciled 2026-09-09:
`docs/autonomy/updates/one-keyring-fixture-two-crates.md` moves the isolated
real-store fixture into a dev-only crate and adds dependency/image guards and
kernel-supervisor rename accounting. Contributor reports eight real-keyring and
nine unit tests passing, without exact commands; initial guard mistakes are
recorded in the report. No independent credential/tooling rerun is claimed here.
Production credential architecture is unchanged. Authenticated daemon HTTPS
remains Claude's next task; no feature or release checkbox changes.

### Credential contributor reconciliation, 2026-09-09

`docs/autonomy/updates/the-daemon-fetches-and-connects.md` records real-keyring
end-to-end daemon no-provider/no-fallback checks for four refusals and a connection
to only the chosen provider after key retrieval. The report supplies no exact
commands or numeric test results; these are contributor claims, not independent
checks in the native-label task. Authenticated HTTPS key delivery is unproved,
blocked on the report's trust-anchor decision. No machine-store or extra-root
change is authorized by this reconciliation. Claude retains connection lifetime,
concurrency/logout work; no provider/keyring or release acceptance box moves.

Credential reports reconciled 2026-09-08: ADR 0022 is accepted and amended to
`secret-service` over explicitly addressed `zbus`; the old libsecret dependency
and binding decision blockers are superseded. Synthetic endpoint routing,
no-connect refusal, settings redaction, eight bus/refusal tests, protocol shape
and intended/decoy listener checks are contributor evidence, not rerun here.
Real-store states, authenticated HTTPS through the daemon, connection lifetime,
concurrency and logout remain unverified. Claude reports needing a running Secret
Service fixture. Desktop sign-in/session integration and the image's Secret
Service package/unit remain scheduled in phases 4/7, with configuration in phase
5. Separate-user access still needs real login evidence; a compromised daemon
can retrieve keys. ADR 0017 permits outbound bus access; ADR 0021 remains proposed.
No store/portal tier is moved and no release gate closes. Exact source reports
and superseded claims: `docs/autonomy/updates/interactive-window-movement.md`.

Three-primary-source direction reconciled 2026-09-08 from
`docs/autonomy/updates/three-primary-model-choices.md`: Local models, Your own API
provider and Alo form the main choices; no-agent opt-out remains and paired-machine
placement is undecided. Advanced privacy settings are separate. Documentation-only
diff-check evidence; no runtime verification, enforcement change, release tick or
promotion of Alo hosting's later-release tier. ADR 0021 remains proposed.

Model-choice gap coverage reconciled 2026-09-08:
`docs/autonomy/updates/model-choice-and-what-alo-can-verify.md` records two
production Service-path gap tests and contributor-reported Linux/BPF gates.
The forwarding-service privacy gap remains open. ADR 0021 is still proposed;
its D1/D4 recommendation is not acceptance. A future verifiable restriction
requires runtime supervision regardless of ownership; that mechanism remains
unimplemented. Paired-machine configuration is still unreachable, and no scope,
runtime policy or release checkbox changes follow from this report.

The only order things get built in. Items are checked when they meet the
definition of done in `CLAUDE.md` law 3 — the full path, on real hardware — and
a release is done only when its **exit gate** is fully checked.

**A tick means done, not written.** Where something is built and tested but has
not yet met law 3 on real hardware, the item says so in its own words rather
than being ticked optimistically. A roadmap whose ticks are aspirations is worse
than no roadmap, because somebody plans around it.

Every item here appears in `docs/features.md` with a tier. If it is not there,
it is not built.

## And every promise there appears here

That rule only runs one way, and the missing direction cost something. **Only
`docs/features.md` is a list of what was promised.** This file and the queue are
lists of what somebody already knew was hard, which is not the same thing — the
build loop found **six v0.01 promises with no item and no line**, one at a time,
over seven iterations, and twice believed it had found the last of them.

So the rule runs both ways now: **every promise in `docs/features.md` at a given
release has a line here, or is named on the line that carries it.** A promise
with nowhere to be is a promise nobody is going to keep, and the ★ ones — the
things no other system offers — turned out to be the likeliest to go missing,
because they read as description rather than as work.

When an iteration cannot find the line a promise belongs to, that is the finding.
It goes in `STATE.md` and the line gets written, before the work does.

Filesystem report integrated 2026-09-07:
`docs/autonomy/updates/hard-linked-files-are-not-read.md`. Unix agent reads and
archives now refuse multiply named regular files using the open handle. The
contributor measured that the kernel permits the granted hard-link name while
refusing the private name; six alo-files tests independently pass here on Ubuntu.
Contributor full Linux gates and mutation evidence are retained as reported, not
claimed rerun. Windows exposure and untested macOS/physical acceptance remain.
The separate kernel rename gap (6d) is closed in the integrated report below.
No hardware or release box changes.

Kernel rename report integrated 2026-09-07:
`docs/autonomy/updates/kernel-enforcement-for-file-renames.md`. The second BPF
LSM hook validates source and destination and the loader requires both hooks.
Reviewed source and tests; six real-kernel tests, approved agentd move, read
regression and full Linux/BPF gates are contributor-reported evidence, not rerun
here. Remaining unhooked mutation operations, hard-link identity and conservative
exchange limits are retained; Windows/macOS are unaffected, physical acceptance
and supervisor gates remain owed. No roadmap capability box changes.

Kernel deletion/link report integrated 2026-09-07:
`docs/autonomy/updates/kernel-protection-for-deletion-and-links.md`. Four hooks
and six pins now cover open/rename/unlink/link; source/tests reviewed. Contributor
reports seven loaded-kernel tests, per-hook leftover-pin refusal, full teardown,
existing open/rename/agentd integration and full Linux/BPF gates passing. These
are retained contributor evidence, not rerun in the desktop iteration. Genuine
mid-attach failure was not induced. Symlink/directory/create/attribute hooks,
already-open descriptors and non-filesystem enforcement remain unimplemented;
pre-existing hard links still rely on alo-files refusal. No physical/release box
changes; supervisor independent gates remain owed.

Network/security reports reconciled 2026-09-07:
`docs/autonomy/updates/network-egress-enforcement-policy.md` and
`docs/autonomy/updates/network-egress-gap-reproduced.md`. Provider/region policy
stays in userspace; kernel enforcement will require a visible departure for a
bound turn. The contributor's loaded-kernel test refuses an ungranted file but
allows loopback connection, proving the socket gap is still open. Their Linux/
BPF gates are retained reported evidence, not rerun here; Windows was not run.
No enforcement or hardware box changes. Contributor tooling evidence is reconciled
in STATE from `docs/autonomy/updates/kernel-supervisor-runs-gates-through-wsl.md`; publication
and physical evidence must not be inferred from a successful local gate/commit.

## Three states, because two are not enough

Almost every item below is one capability spanning two halves: a crate that
decides, and a screen or daemon that shows and runs it. The crates are being
built first, deliberately — they need no compositor and no certified machine.
So an item is routinely *half done* for months, and a box that can only be empty
or ticked has no way to say that.

It said the wrong thing instead. For eight commits this file did not move while
eight crates and several hundred tests landed, and the two commits before that
moved it only by **adding** unticked lines. Read literally, it reported that
nothing had happened. That is a reporting failure, not a dishonest tick, but
somebody planning from it is misled either way.

So a part-done line **carries two boxes of its own**, and the parent line keeps
its empty one:

```
- [ ] Egress indicator, and no telemetry
  - [x] The code.        alo-egress — what counts as leaving, and …
  - [ ] On the machine.  the indicator itself, which is a compositor surface …
```

- The parent `- [ ]` stays empty until **both** halves are done. Half a
  capability is not a capability, and nobody planning around this file may read
  a ticked half as delivery.
- **The code** is ticked when that half is whole and gated — built, tested, no
  stubs. It is the half this repository can finish today, and every crate below
  was finished on an ordinary laptop.
- **On the machine** is ticked only under law 3: the certified machine, and the
  compositor once it exists. Nothing in this repository can tick it, and no loop
  may. *Not a GPU* — no machine half below needs one except the GPU line itself,
  which is acceleration rather than an entry price.

**Why this shape rather than one box.** The rule "a tick means done on real
hardware" is right and is not being weakened — but with one box per line it made
this file report **one done item out of eighty** while thirteen crates and
1,128 passing tests sat underneath it — twenty-one crates and 1,563 now. That is not honesty; it is a different
inaccuracy, and somebody reading it would conclude nothing had been built. Two
boxes tell the truth twice: what is finished, and what is still owed to a
machine nobody has yet plugged in.

**A code box is a claim, so its clause names the crate.** A clause that cannot
name one is decoration, and decoration here is how a roadmap starts lying
slowly.

---

Owner clarification reconciled 2026-09-08: model ownership does not establish
processing location or privacy. Existing local, paired, provider and no-agent
choices and release tiers stand; ADR 0021 remains proposed. Documentation-only
report: `docs/autonomy/updates/owner-model-choice-direction.md`; no runtime
verification or completion tick follows from that report.

## v0.01 — it boots and the agent acts

The point is to prove one sentence on real hardware: *an action a person would
take by hand can be proposed by an agent, approved in one click, and afterwards
explained — and the model that proposed it ran on the customer's own machine.*

**Two certified machines, and the ordinary one matters more** (ADR 0007,
`docs/hardware.md`): a business laptop with no discrete graphics and 16 GB of
memory, running its agents on the CPU, and a GPU workstation for alo OS AI. The
laptop is the machine that decides whether this project has a market — the
Windows 10 fleet it exists to catch has almost no discrete GPUs in it. **A
graphics card is acceleration, never an entry price**, and nothing in v0.01
requires one.

No installer, no fleet management, no compatibility list.

**Order, and why the ticks below are contiguous.** v0.01 divides on exactly one
question — *does this need a screen?* — and the answer sorts it perfectly:

- **Twelve capabilities need no screen. All twelve have their code finished.**
- **Eight need the compositor or the certified machine. Compositor protocol
  work has started; none of these capabilities has passed physical acceptance.**

There is no third group and nothing in between. The two bands used to be
interleaved in this list, so a completely consistent rule read as items being
skipped over at random; they are separated now, and the list is in the order the
work actually happened.

The AI stack leads the first band for the reason it always did: no kernel work,
no image pipeline, no certified hardware, and it is useful to `alo-workplace`
the day it lands.

### Everything that needs no screen — built first, and all of it is built

Twelve capabilities. Each has its code finished, tested and gated on an ordinary
laptop, and each is still owed the half only a machine can give it. **This band
was not chosen line by line**: it is simply everything in v0.01 that a
compositor is not required for, which is why it runs unbroken.

- [ ] **Model stack**: catalogue, pull, serve, unload, remove — over the pinned
      runtime (ADR 0006)
  - [x] **The code.**
        `alo-models` — the catalogue with its licence gate, `ModelRuntime`,
        and the Ollama adapter, the one file allowed to name it. Since item
        18a the *serve* in this line is real rather than a heading:
        `ModelRuntime::answers` puts a question to a model and the adapter
        carries it over the runtime's own chat API, so the trait manages
        models **and** uses one. 96 tests, several of them against a real
        socket rather than a mock
  - [ ] **On the machine.**
        it has never been run against a real Ollama — on any machine, with or
        without a card, because a CPU-only box is the one this has to work on
        first. *This line was ticked outright until the two boxes existed,
        while its own last sentence said law 3's "on real hardware" was owed —
        a tick and its own footnote contradicting each other, which is exactly
        what the parent box is now not allowed to do*

- [ ] ★ **It runs on the machine you already own** — no graphics card required.
      The catalogue carries models that work on a CPU, and says honestly which
      ones are comfortable there and which are merely possible
  - [x] **The code.**
        `alo-models`' catalogue — `OnCpu` as a stated property of each model
        rather than a footnote, `runnable_on_cpu(ram_gb)` and
        `runnable_with_vram`, so the answer to *what can this machine
        actually run* is asked of the machine rather than assumed. Since
        item 23 *what runs here* and *what gets the agent* are two methods
        rather than one: `default_for_cpu` is gone, because ADR 0007's own
        correction is that **"default" was the wrong word**, and the
        recommendation lives on the line below
  - [ ] **On the machine.**
        the setup screen that asks it on somebody's behalf, and a real
        measurement — *comfortable* is a judgement in a table until a model
        has run on a machine without a GPU

- [ ] ★ **The catalogue says whether a model can drive the verbs**, not only
      whether it will run — measured by us, never claimed by the publisher
      (ADR 0007, as corrected). And a machine is only offered agent work it can
      actually do: where nothing clears the bar, the honest answers are the ones
      ADR 0008 already provides, offered as a choice and never substituted
  - [x] **The code.**
        `alo-driving`, a new crate — the fixed set of ten requests, one per
        verb alo OS ships, each scored through `alo_protocol::FromAnAgent`
        and `alo_capability::Verbs::call`, which is the daemon's own door and
        the same validation a real turn does rather than a second reader
        written for a test. A run that skipped an exercise is refused, and the
        bar is nine attempts in ten. And `alo-models`, where the grade lives
        and is read: `Driving` as a stated property every entry must answer —
        `NotMeasured` is an answer and is `Region::Unknown` one file over —
        `Catalogue::agent_for_cpu`, and `NoAgentHere`, whose only road to a
        sentence hands back **three** lines so a machine cannot show somebody
        the refusal without every answer they still have. Since item 25a the
        middle one is weights they already have — the answer that needs no
        other machine and no account, added as a line of its own because
        ADR 0008's question is *where* and this one is *which model*, and
        ordered outward from the machine rather than by which is better.
        Since item 23a the method can also be **run**:
        `alo-driving`'s `against_a_model_on_this_machine` puts the fixed set
        to a real runtime through `alo-asking`'s local door, warms the model
        first so the exercise that loads the weights is not graded for the
        disk, and stops rather than scoring when a runtime fails — a model
        blamed for a machine is the one way a grade is worse than no grade.
        **Five entries have a measured grade because of it**, and they are
        every entry a machine with no graphics card can run: `phi-3-mini-
        instruct` since item 23a, and `llama-3.2-3b-instruct`,
        `qwen2.5-3b-instruct`, `gemma-2-2b-instruct` and
        `smollm2-1.7b-instruct` since 23c. A hundred attempts between them,
        three of which a machine would have acted on, `rarely` five times
  - [ ] **On the machine.**
        **Seven of the twelve entries have still never been run against**,
        because a measurement needs those weights on a disk and every one of
        the seven wants ten gigabytes of memory or more against the six the
        measuring box has; they say `not-measured`, and a grade is a data
        change rather than a release. The five that have been measured were
        measured on a development box, which is the right machine for the
        question they answer and the wrong one for the rest of the entry —
        `min_ram_gb` and `on_cpu` are what a model costs on the machine it
        runs on, and the certified one is where those are found. Plus the
        setup screen that shows the refusal and its three alternatives
        without choosing between them

- [ ] ★ **Or use an API instead** (ADR 0008) — an answer may come from this
      machine, from a machine on your network, or from a provider you named, and
      the choice is the person's. **An organisation bounds that choice and never
      makes it** (ADR 0016): two settings, two owners, two files, and a choice
      outside the bound refused in words rather than quietly replaced
  - [x] **The code.**
        `alo-models`' `InferenceSource` — this machine, a paired machine, a
        hosted provider with its region — and `SourcePolicy`, which can hold
        an answer in the building, on the machine, or inside a named region,
        and refuses in the policy's own words rather than silently routing
        elsewhere. **And `alo-asking`, which actually puts the question**: a
        provider is asked over https, the egress is on the indicator before
        the socket opens, the rule in force at that moment decides, and the
        answer comes back carrying where it came from. Since item 18a **all
        three of the ADR's places that this repository can reach are
        reachable**: `to_this_machine` puts the same question to the model
        alo OS ships, with no indicator, no departure and no rule to ask,
        because nothing on that path goes anywhere. Since item 18b a third
        door, `to_a_service_on_this_machine`, reaches an OpenAI-compatible
        service somebody runs here themselves — vLLM, llama.cpp's server,
        LM Studio — which is this machine and not a provider, and which
        **cannot be pointed anywhere else**: the door with no indicator on it
        takes a value that exists only for an address `alo-models` calls this
        machine. Three doors that divide on law 1 rather than on what speaks
        at the far end, and each refuses a permission another one is behind.
        Since item 19a **a turn is what reaches those doors**: `alo-turn`
        takes the place a person chose and the thing that answers there,
        routes to whichever of the three it is, holds the machine's one
        indicator while it happens, and writes what left — or what a rule
        stopped from leaving — before the answer reaches anybody. Since item
        22 **a provider whose account has run out says that and not
        something else**: `alo-answering`'s `RanOut` is the one failure here
        that is not a fault, and `alo-asking` reads the name inside a `403`
        or a `429` against a closed list to tell it apart from a refused key
        and from being asked to slow down — while opening no door a failure
        for any other reason would not, because *never a silent fallback*
        runs hardest in the direction where somebody's money is at the other
        end. 86 tests in `alo-asking`, most of them against a stub on a real
        socket or a stub of the runtime trait, and 57 in `alo-answering`
        behind them
  - [ ] **On the machine.**
        a provider somebody pays for, answering a real question with a real
        key, and a real model runtime answering one on this machine — neither
        has been run against the real thing. A question to **a machine on
        your network** is the third place and has no path in this repository
        at all: both doors refuse a permission naming one, in words that say
        so and offer neither of the other two instead

- [ ] ★ **Where the answer came from is said where the answer appears** — "on
      this machine", "on the studio workstation", "by a provider you added" —
      beside the answer, not buried in a setting
  - [x] **The code.**
        `alo-models` — every source can say itself (`shown`, `said`) in the
        language the reader reads, so provenance is a translated sentence
        rather than English a shell would have to reword — and `alo-asking`'s
        `Answer`, which **cannot be made without one**: the only constructor
        takes the source, so a shell holding an answer is holding the
        sentence about where it came from
  - [ ] **On the machine.**
        the surface that shows it beside an answer, which is the overlay,
        and therefore the compositor. **This was the one on this list most
        easily lost** — a sentence that must appear every time, with nothing
        forcing it to. The type now forces it as far as a type can: showing
        an answer without its provenance is a thing somebody has to decide to
        do rather than a thing they can forget

- [ ] **Agents point at the local model by default**, configured rather than coded
      — and, since the default is only a sovereignty guarantee if it does not
      quietly un-point itself, **★ never a silent fallback** (ADR 0008) is
      carried on this line too, having none of its own
  - [x] **The code.**
        `alo-models` — where an answer may come from, and the policy that
        keeps it in the building, on the machine, or in a region, both now
        said in the language the person reads rather than in English — and
        `alo-answering`, which is what happens when the place a person chose
        cannot answer: the failure named with the place it happened, the
        line saying nothing was sent and nothing will be, and asking
        somewhere else as one sentence a person approves for exactly one
        question rather than a setting anybody can leave on. Since item 18a
        **★ never a silent fallback runs both ways in code and not only in
        the ADR**: the local door is where *a local model that fails becomes
        an API call* would have been written as a convenience, and what it
        does instead is hand back the same failure, whose only way onward is
        an offer somebody answered. Since item 19a the same is true one
        level up, where a fallback would actually have been written: a turn
        that meets a failure **hands it back and stops**, and the only road
        from one to a second attempt is an offer a person took, which comes
        in at the same door and is shown and recorded like any other
        question. A test asks it as the thing it is — the place that failed
        was asked once, the place that was offered was asked nothing at all.
        Since item 21h **there is a place the pointing is configured from**:
        `alo-choosing` holds what a person chose — which model, and which of
        this machine's two lists it came from — in a file of their own under
        their home directory, and `Chosen::asking` is the one place that
        choice meets the rule an organisation set. *Configured rather than
        coded* is true of the setting now: there is no default and no
        `Default`, a machine nobody has configured has chosen nothing and
        says so, and a choice the organisation's bound forbids is refused in
        the rule's own words rather than swapped for a permitted one. Since
        item 21k **a choice resolves into something that can answer**: the two
        facts nothing on a machine stated are stated. `alo-models` finds the
        runtime — **found, never configured** (ADR 0019), at an address the
        adapter alone knows, with no key in any contract and no override for
        an operator to point elsewhere — and `alo-choosing` holds the weights
        somebody brought on the person's own list, so a choice from either of
        this machine's two lists resolves into the entry it names. Settings
        that could say *my questions are answered by weights that are not on
        my list* are refused where they are made. 71 tests, seven of them
        against a real file on a real disk. Since item 21n **the daemon
        walks it**: `alo-agentd` reads the person's own settings out of its
        own environment — it runs as them, one per login, which ADR 0019
        records as a condition rather than an assumption — finds the runtime
        once at the first question of each turn, and puts the question to
        what they chose. A turn that asks nothing opens no file and probes
        nothing, a model picked in Settings answers the next turn, and the
        three ways it can go wrong are three sentences rather than one: pick
        something, the runtime is not reachable, or your settings file says
        this and here is its path
  - [ ] **On the machine.**
        something that points, which is `alo-agentd`. *Something that asks*
        is no longer owed here at all: `alo-asking` puts a question to a
        hosted provider **and** to the model on this machine, and hands back
        a failure whose only door onward is an offer a person answered — so
        the fallback is carried by the code that would have had to contain
        it, in both directions. What is left of the machine half is the
        daemon that points at the local model, and a real runtime answering
        a real question, which needs Ollama installed. Since item 21d the
        daemon exists and holds a turn, and a question put to it is refused
        in words — *nothing on this machine has been chosen to answer
        questions* — because nothing yet reads what the person chose. Since
        item 21e a machine describes itself — two logins, two lengths of time
        and where the record goes — and which model or provider answers is not
        yet among the things it says. Since item 21f the process exists and runs
        all of that, so what is left is only the setting: `alo-asking`,
        `alo-models` and `alo-answering` are loaded into the machine's
        vocabulary and unused. Since item 21h **the setting exists and the
        daemon does not read it yet**: a person's choice has a file, a shape
        and eight refusals. Since item 21k **the road from that choice to
        something that answers is built and the daemon has not walked it
        yet**: a runtime is found by its adapter, weights somebody brought
        are on the person's own list, and both of the facts that were missing
        are stated. Since item 21n **the daemon reads the setting and puts
        the question**, so *nothing on this machine has been chosen to answer
        questions* is now what an unconfigured machine says rather than what
        every machine says. What is left here is what this loop cannot do: a
        real runtime with real weights, on a certified machine, answering a
        real question put through the socket — no test in this repository has
        ever had Ollama at the other end

- [ ] **Add your own provider in Settings** — name, address, key to the keyring;
      the region stated rather than guessed; https required off this machine
  - Provider configuration reconciliation (2026-09-08): format 2 settings now
    persist the person's provider list and selected model; format 1 compatibility
    and no local fallback are tested. Contributor-reported six choosing/two daemon
    tests and workspace/BPF gates: `docs/autonomy/updates/three-model-choices-in-the-backend.md`.
    Credential store and native controls remain delivery phase 5 work; providers
    requiring a key refuse, Alo has no endpoint, paired machines stay unreachable.
    Raw parse-error Debug may retain pasted credentials despite safe UI messages;
    follow-up remains open with Claude. ADR 0021 remains proposed; no release tick.
  - [x] **The code.**
        `alo-models` — the provider, the key held as a keyring handle and
        never in the record, testing it before it is saved, and every
        refusal about any of the three readable in the reader's own language.
        Since item 18b **"https required off this machine" is true rather
        than approximately true**: `address.rs` reads the host out of the
        address and matches it whole, so `localhost.attacker.example` and
        `127.0.0.1@attacker.example` are somewhere else — they were this
        machine to a prefix check, which meant an unencrypted connection
        carrying a key, and an answer that claimed never to have left
  - [ ] **On the machine.**
        the Settings panel to type it into

- [ ] **Egress indicator**, and no telemetry
  - Kernel coverage audit reconciled 2026-09-08: publication checks now fail
    closed through the shared kernel verify/publish path. Reproduced loopback
    proxy, unconnected datagram and established-socket gaps remain, not fixes.
    The production-reachable proxy gap still needs a decision; no enforcement or
    machine checkbox moves. See `docs/autonomy/updates/publication-hardening-and-egress-coverage.md`.
  - [x] **The code.**
        `alo-egress` — what counts as leaving, and the line said about it
        while it happens, now in the language the person reads rather than
        in English; the policy still decides before a socket opens and
        without needing a vocabulary to do it. Since item 16 the second half
        of this line has code too: egress with **no agent behind it** is a
        closed list of three reasons alo OS reaches the network — signing
        somebody in, fetching a model, checking for an update — with no
        member for measuring anything and no way to add one that is not an
        edit to a public enum. They go on the **same** indicator as an
        agent's egress, so *nothing has left this machine* stays one thing
        to look at, and the promise beside the list is a sentence a person
        reads in their own language rather than one this repository
        publishes. Since item 16a law 1's *and afterwards in a record*
        covers that second half as well: `alo-record` writes an errand down
        as `Happened::LeftOnItsOwn`, made only from the `Underway` the
        indicator showed, and it is the one entry in the record with **no
        agent field** — because nobody granted this machine permission to
        sign somebody in, and a name in that column would be an authority
        the record invented. *What left this machine* and *what did it do on
        its own* are two queries over one list, and
        `docs/contracts/record-file.md` now says what a new kind of entry
        means for a reader that predates it
  - [ ] **On the machine.**
        the indicator itself, which is a compositor surface; the daemon code
        that actually signs somebody in, fetches a model or checks for an
        update, none of which exists yet; and the enforcement at the network
        boundary, without which all of this describes only the code that
        asked. **Partial enforcement published and reconciled 2026-09-07:**
        Claude's `socket_connect` hook checks non-loopback IPv4/IPv6 address/port
        entries per turn, including next-connect withdrawal. Seven loaded-kernel
        cases and Linux workspace/BPF gates are contributor-reported evidence,
        not rerun by this desktop worker. Production wiring, UDP sendto,
        established/inherited sockets, loopback proxies and physical acceptance
        remain open; the enforcement requirement and machine box stay unchecked.
        Report: `docs/autonomy/updates/network-egress-enforcement.md`.
        **Security follow-ups reconciled 2026-09-08:** reports
        `docs/autonomy/updates/network-boundary-decisions-proposed.md` and
        `docs/autonomy/updates/one-kernel-two-checkouts.md` correct the relay door
        to Answers::Service (the provider door refuses ThisMachine), and serialize
        kernel tests across checkouts with a bounded abstract-socket lock. Five
        process tests, four unit tests and full gates are contributor-reported,
        not rerun here. ADR 0021 is still proposed; its revised C1 recommendation
        is not approval or implementation. Loopback-proxy, inherited-descriptor/
        socket and use-time gaps remain; no enforcement or machine box closes.
        **Production-path audit reconciled 2026-09-07:**
        `docs/autonomy/updates/end-to-end-network-enforcement.md` traces and tests
        that file verbs enter Bounding but provider requests do not. Destination
        enforcement therefore does not cover ordinary provider calls. Claude's
        boundary/destination-lifetime decision was subsequently approved in
        `docs/autonomy/updates/network-request-boundary-approval.md`, reconciled
        2026-09-07. Claude owns the scoped ADR and implementation, with explicit
        DNS and request-limited connections. Approval is not runtime evidence;
        reported Linux workspace/BPF gates are not rerun here. Enforcement stays
        unfinished, including retries, redirects, UDP, inherited sockets and proxies.

- [ ] **`alo-agentd`**: grants, file verbs, application verbs, context on invocation
  - [x] **The code.**
        `alo-capability` (the verbs, the grants, the approvals, every
        refusal of theirs said in the language the person reads, and — since
        9g — the sentence a person approves, carried as what names it and
        the values that fill it rather than as words in whichever language
        the verb was declared in), `alo-files` (the six file verbs, declared
        from the words a translator is handed) and `alo-applications` (all
        four application verbs — open, focus, ask-to-close and arrange — the
        list of what is installed they are checked against, and the rule
        that an ungranted application refuses identically whether or not it
        is here). Since 11a an argument that offers a choice offers a name a
        model sends beside a word a person reads, so an option cannot reach
        an approval sentence as untranslated English, and a sentence holding
        a word nobody has translated says the line is not translated. Since
        item 12 `alo-context` is what an agent is given when it is invoked:
        the window in front, the selection and the open document, with only
        the document making a grant — over that file, for that turn,
        revocable and visible in the same list as a folder somebody picked —
        so being told what is on a screen is finally distinct from being
        allowed to touch it. Since item 19 there is `alo-turn`, which is the
        four of them joined into one order that cannot be taken out of
        sequence: an invocation makes the turn, a name and typed values are
        made into a call **here** rather than accepted from a caller, a read
        answers inside the turn and a change waits for one approval, the
        grants are asked again at the moment it runs, and the machine offers
        exactly the verbs it can carry out. Since item 21a there is
        `alo-protocol`, which is what somebody else's code is allowed to say
        to all of that: six requests and no seventh, none of them able to
        carry a command, and two doors rather than one — so the side that
        proposes a change cannot be the side that approves it. What is not on
        the wire is as much of it as what is: no moment, no context, no turn
        and no place a question goes, because each of those would be a caller
        helping itself to something the machine is supposed to know. Since
        item 21b the same crate is what the daemon may say **back**: what a
        read found and what a change did, a change waiting with the sentence
        it waits on rather than only its number, a model's answer with where
        it came from beside it and no shape that carries one without the
        other, and every refusal in the workspace as the sentence whoever
        made it worded — each carrying whether anybody translated it, which
        is the one thing text alone would have lost at the last boundary
        before a person reads it. The answers divide by side as the requests
        do, so a daemon cannot put the person's own list onto an agent's
        connection. Since item 21c there is `alo-agentd`, which is the door
        those two lists arrive at: a Unix socket at a path this project's
        contract fixes, in a directory only the person and the agent's group
        can enter, and a side chosen from what the kernel says about the
        caller rather than from anything a caller says about itself. A
        machine on which the person and the agent are one login gets no
        socket at all, because on that machine both doors would be one —
        which is what turns 21a's division from a promise the code makes
        into one the operating system makes. It is the first crate here that
        is Linux rather than portable. Since item 21d it also holds the turn:
        one machine, two connections and one turn, answered as readiness in a
        single thread with no lock anywhere near the capability model — a turn
        is an agent's connection and ends with it, an approval arrives on the
        person's while it is open, and a second agent is refused in words
        rather than let into a grant another invocation made. A message that
        is not a request is answered and the caller stays; a line this machine
        will not read is answered and then closed; and a machine that could
        not write down what it did stops serving rather than going on without
        evidence. Since item 21e it reads what the machine is rather than
        being handed it: one file, whose shape and path are a contract because
        whoever installs or manages a machine writes it, holding the two
        logins, the agent's name as its grants know it, the two lengths of
        time and where the record goes and for how long. Nothing in it has a
        default, so a key left out is a machine that does not start rather
        than one running under a number nobody chose; a length of time longer
        than a day is refused rather than shortened, because an approval is
        never a session; and the file is checked before it is parsed — not a
        link, owned by root or by the person, writable by nobody else, and all
        of it asked of the open file rather than of the name, because the file
        that names which login is the agent is the file somebody would rewrite
        to become one. Since item 20 it is also the thing that removes what
        the machine no longer keeps: a machine an organisation set a retention
        rule on wakes once an hour to shorten its record, and a machine that
        keeps everything — which is what one ships with — sleeps in a single
        call until somebody says something, exactly as it did before there was
        a timer. It happens between turns and never inside one, which is not a
        rule anybody has to remember: while a turn is under way it holds the
        machine, so there is nothing there to ask. And since item 21f it is a
        **process** rather than a library nothing runs: a `main` that refuses to
        be root at all, loads the one vocabulary the whole machine says
        everything out of and its own three strings on top, opens the record
        before it opens the socket, and arranges for `SIGTERM` to stop it the
        way anything else does — one byte on a descriptor, from a handler that
        allocates nothing. It has been started, talked to and stopped as a real
        process with two real logins, which is what nothing above this sentence
        could previously claim. Since item 21j the agent's door is somewhere the
        agent could reach: running it as two real logins found that a `0750`
        directory inside `logind`'s `0700` session directory is a locked room
        inside a locked building, so ADR 0017 moved the socket to
        `/run/alo/<uid>/agentd.sock` — a root the image makes and the daemon
        refuses to invent, a per-person directory made for one session and taken
        away with the socket when it ends, and every check `place.rs` already
        made carried over unchanged. Since item 30 the daemon has been started
        by a real `systemd` from the image's own unit, which is what found that
        the per-person directory could not be the daemon's after all: `/run/alo`
        is root's, so the thing that starts the service makes the door and the
        service checks it. It runs there — the agent's login refused a folder
        nobody granted, in the grants' own words; the person's door answered;
        anybody else turned away; the refusal in the record; `SIGTERM` and the
        summary line — on a development box under systemd, which is not a boot.
        And since item 6b the file verbs no longer open anything **by name** on
        Linux: a path resolved and checked against the grants is opened in one
        call that refuses a symbolic link at every component of it, so a folder
        on the way cannot be exchanged for a link between the check and the
        open, and a move is one call that refuses or moves rather than a
        question followed by an act. Neither falls back — a kernel or a
        filesystem that cannot promise it refuses the work in its own words. The
        walk this was first written as had to be thrown away, and what threw it
        away was the boundary above: opening `/` and each folder under it is
        opening things the call never named, and a bounded turn was refused its
        own granted file. Since item 6c a move is held to the same promise: a
        rename cannot resolve its whole path in one call, so it **holds** its
        two folders instead — reached by a path with no link in it and then kept
        as handles — and neither the folder a file leaves nor the one it arrives
        in can be exchanged underneath the call. That was allowed to be built
        because it was measured rather than assumed: an `O_PATH` handle is
        invisible to the boundary *and confers no reading*, checked against a
        running kernel with the programme loaded, so nothing a turn may reach
        got wider and no ADR had to move. A granted move now runs inside a real
        boundary in `alo-agentd`'s own test, which is the guard that caught 6b's
        first design. Measuring also found missing rename enforcement (6d), now
        closed by the second BPF hook checking the source and destination parent.
        Claude's `docs/autonomy/updates/kernel-enforcement-for-file-renames.md`
        reports six kernel tests and the approved-move regression passing; those
        checks are retained as contributor evidence, not rerun by this iteration.
  - [ ] **On the machine.**
        the door being reached — the path moved in code and no connection from a
        second login has been made since, and it cannot be until an image exists
        with `/run/alo` in its `tmpfiles.d` (queue 28, ADR 0017,
        `docs/quirks.md`); which model or provider answers a
        question, which needs somewhere for a person's own choice to live
        (queue 21h); the acting half of the application
        verbs, which is Wayland and D-Bus and is the whole of what makes any
        of these move a window; and the half of the context that **reads** a
        screen, which is Wayland and AT-SPI and is where *with no invocation,
        no context calls at all* becomes something anybody can test

- [ ] Every execution recorded with its origin, approval and grant
  - [x] **The code.**
        `alo-record` — the record, including refusals, which are written
        down in the same words the person was shown rather than in a second
        rendering of their own; since 9g that covers what *ran* as well as
        what did not, so the sentence in the record is the sentence somebody
        approved — and `alo-keeping`, which puts it on a disk so it outlives
        the session: one line per thing that happened, synced as it happens,
        a retention rule that cannot be set to keep nothing, and a shortened
        record that says so permanently in the first line so an absence is
        never read as an innocence. Since 16a it also holds the one thing on
        the machine that no execution, approval or grant is behind — what
        alo OS did on its own — and holds it without naming anybody for it.
        Since item 19 *recorded* is structural rather than remembered:
        `alo-turn` cannot be made without somewhere to keep its record, every
        door writes its entry before it answers anybody, and a turn that could
        not write one stops doing anything at all. Since item 20 the timer is
        there too: `alo-agentd` shortens the record between turns, once an
        hour, and only on a machine somebody set a retention rule on — the
        first one at start-up, because a machine switched off for six months
        comes back with six months of a rule to catch up on. A shortening the
        machine refuses is counted and survived rather than stopping the
        service, because nothing is removed in one: that is a machine keeping
        **more** than its rule, which is the opposite failure from one that
        cannot write
  - [ ] **On the machine.**
        a certified machine showing a record surviving a restart and a
        shortening. The path, the retention and the timer are all code now, and
        since item 21f a real process really opens the record at the path its
        description named and writes the first line into it — what is left is
        that nothing has yet been started by systemd, and no shortening has
        fired on a machine nobody was watching

- [ ] **The dock on any edge** — bottom, left, right or top, the person's choice,
      built for both orientations rather than one rotated
  - [x] **The code.**
        `alo-dock` — the layout model, and the two orientations as two
        layouts rather than one turned sideways: names sit under an icon
        across the screen and beside it down the screen, the thickness comes
        off the side the dock actually sits on, and the status area is a
        column at the bottom of a vertical dock while the far end of a
        horizontal one follows which way the person reads. *Labels give way
        to icons where the short edge demands it* is arithmetic now rather
        than a designer's eye, and the threshold is held to EN 301 549's
        200% on the smallest screen alo OS lays out for, on all four edges
  - [ ] **On the machine.**
        the compositor that draws it, and with it everything about the dock
        that is a picture rather than a measurement — the icons, what is in
        the status area (v0.5), and the hover and screen-reader name the
        *gave way* sentence promises is still there

- [ ] **AI can be declined entirely** — setup's fourth choice, and a system that
      is complete without it (ADR 0009)
  - [x] **The code.**
        `alo-capability`'s `Agent` — the fourth answer as a value, and the
        half of ADR 0009 that would have been quietly got wrong. It is not a
        flag beside the grants; it is what holds them, so a machine where
        the person declined has no list at all rather than an empty one,
        nothing can be granted on it because there is no list to grant onto,
        and turning the agent off ends every grant on the machine — the
        folder picked in March and the document an invocation handed over
        five minutes ago alike — with the immediacy a single revoke has
        always had. Turning it on again brings back an agent and not the
        folders, which is the difference between *grants end* and *grants
        are suspended*, and the choice is written down so changing your mind
        is a setting rather than a reinstall. The record and the egress
        indicator are untouched, because neither is an AI feature: a machine
        with no agent still writes down its own errands, and `alo-record`'s
        `Only::ByAnAgent` is how somebody asks whether anything in their
        record has an agent's name on it at all
  - [ ] **On the machine.**
        everything about it that is a screen — setup's fourth choice as a
        question with the same weight as the other three, the hotkey doing
        nothing, the overlay not existing, and Grants, Models and providers
        being absent from Settings rather than greyed out. All of that is
        the compositor's and the settings panel's, and neither exists

- [ ] Keyboard shortcuts a person can change
  - Scheduling clarified 2026-09-08: finish underlying window operations before
    integrating phase 3 shortcuts. Physical acceptance remains phase 8, following
    phase 7 VM image checks. Documentation-only report
    `docs/autonomy/updates/deferred-desktop-and-hardware-acceptance.md` reconciled;
    no requirement is completed or moved to a later release.
  - Configurable window command dispatch (2026-09-08): current bindings drive
    native next/previous window and focused-root cooperative close, with conflict,
    unbound, unsupported-action and missing-target handling. Five real-client
    tests and configured GLES readback are recorded in
    `docs/autonomy/updates/configurable-window-command-dispatch.md`. This is the
    action bridge only: layout matching, consumed press/release isolation,
    nested/direct input wiring and settings UI/persistence remain owed. Full
    supervisor gates and physical acceptance remain; feature stays unchecked.
  - [x] **The code.**
        `alo-shortcuts` — the shortcuts, rebindable, nothing quietly taking
        one away, and every row and key of the panel said in the language
        the person reads
  - [ ] **On the machine.**
        a shell to press them in

### Everything that needs the compositor or the certified machine

Eight capabilities, with **the compositor protocol core now started**. The
remaining machine work follows `docs/autonomy/DELIVERY.md`. The
compositor is the one that matters most here, because sign-in, the overlay, the
launcher, copy and paste and the workspace client all wait on it; the image is
its own bring-up and waits on the certified machine.

**The GPU line is in this band but is not a requirement of it.** No graphics
card is needed anywhere in v0.01 — the whole product rests on running on the
machine somebody already owns, and the exit gate below neither mentions a GPU
nor may depend on one. That line is here only because *acceleration where a card
exists* still has to be verified on hardware.

They were interleaved with the band above until this ordering was corrected,
which made a completely consistent rule look like work being taken out of turn.

- [ ] ★ **The GPU works on first boot, where there is one** — drivers and runtime
      pinned together, no driver installation, no CUDA archaeology.
      **Acceleration, not an entry price** (`docs/features.md`): alo OS runs on
      the machine somebody already owns, and this line is what happens when that
      machine turns out to have a card in it. **It does not gate v0.01.** The
      exit gate below never mentions a GPU and must pass on a machine with none —
      if it ever cannot, the promise this release is built on has been broken and
      the fault is here rather than in the gate

- [ ] **The colours come from a source this repository can read** — and that is
      not CSS. **There is no CSS in alo OS and there will not be**: this
      repository is Rust, and the shell is native (ADR 0002). The problem is
      only that alo's palette currently *lives* in
      `alo-workplace/web/src/ds/tokens.css`, 327 lines calling themselves "the
      single source of visual truth", which a Rust compositor cannot read.
      The work is to move that source into something language-neutral — TOML,
      alongside the fifteen manifests already here — and **generate** from it:
      Rust constants for this shell, and the custom properties the workspace's
      web client still needs, because that client is a web application and its
      stylesheet is a fact about it rather than about this operating system.
      The point is to end CSS's authority over the palette, not to import it.
      *Found missing by an audit of the ADRs: a consequence of ADR 0002 with no
      line here and no entry in `docs/features.md`*
- [ ] **Compositor**: Wayland via Smithay, one display, keyboard and pointer
  - Client maximize/restore requests (2026-09-08): XDG requests share trusted
    transactions, pre-map intent is declined without premature configuration,
    unchanged/refused requests receive responses and only Maximize is advertised.
    Four real-client tests and five full-frame GLES boundaries pass. Verification
    is recorded in `docs/autonomy/updates/client-window-maximize-requests.md`.
    Next: minimization/restoration, then client minimize policy and tiling;
    this does not complete rendered controls or the compositor release gate.
  - Native window placement (2026-09-08): bounded mapped-root geometry origins
    feed drawing, input and popup output constraints, with committed-geometry and
    unmap semantics. Evidence: `docs/autonomy/updates/native-window-placement.md`.
    Corrected independent full-frame GLES expectations pass with a stationary
    second window, all-edge clipping and restoration. Interactive dragging/resizing,
    remaining operations, rendered controls and direct desktop entry remain.
    Publication gates are recorded in the task report. No feature/release tick.
  - Cooperative window sizing (2026-09-08): trusted mapped-root logical-size
    requests validate dimensions and committed client limits, suppress duplicate
    configures and preserve committed buffers and input until client response.
    Real-client negotiation/refusal and GLES evidence is recorded in
    `docs/autonomy/updates/cooperative-window-sizing.md`. Size-request primitive
    complete; interactive resize, placement, rendered controls and full native
    desktop entry remain unfinished. No feature or release gate is ticked.
  - Session-integrated direct input (2026-09-08): a new direct entry method creates
    SeatInput through the same libseat connection inside the active display scope.
    Idle/render dispatch uses latched authority polling; input suspension/reset
    and flush precede output retirement. Three new tests include real empty-seat
    libinput/calloop and descriptor EOF. Linux shell 221 tests and three doctests,
    affected Windows/Linux checks, Linux rustdoc/examples and WSLg regression
    (115 surfaces) pass. Report:
    `docs/autonomy/updates/session-integrated-direct-input.md`. Standalone safe
    GLES construction, real DRM/populated-seat acquisition, recovery and physical
    records remain owed. Full supervisor gates pending; compositor unchecked.
  - Libinput seat-event routing (2026-09-08): keyboard, relative/absolute pointer,
    button and modern scroll translation; first-press/last-release device counts
    and conservative whole-seat removal/reset cleanup. Nine new tests include
    client-wire refusal, popup/drag cleanup and real empty-seat reset delivery.
    Linux shell 218 tests and three doctests, affected Windows/Linux checks,
    Linux rustdoc/examples and WSLg regression (115 surfaces) pass. Report:
    `docs/autonomy/updates/libinput-seat-event-routing.md`. Next: live DirectSession
    polling/frame-loop wiring. Populated input acquisition/hotplug, safe standalone
    GLES, real DRM/seat, recovery and physical acceptance remain owed. Supervisor
    full publication gates pending; compositor and release remain unchecked.
  - Seat-checked libinput context lifetime (2026-09-08): udev assignment through
    SessionInput, seat checks before dispatch/delivery and terminal suspension
    before reset on failure. Eight new tests include real empty-seat integration
    and descriptor EOF/error retention. Linux shell 209 tests and three doctests,
    affected Windows/Linux fmt/clippy/tests, Linux rustdoc/examples and WSLg
    regression (115 surfaces) pass. Report:
    `docs/autonomy/updates/seat-checked-libinput-context-lifetime.md`. Event
    translation, device-removal cleanup and live DirectSession/frame-loop wiring
    remain next. Safe standalone GLES, real DRM/seat and physical acceptance
    remain owed; full supervisor publication gates pending, compositor unchecked.
  - Session-owned input descriptors (2026-09-08): restricted libinput opens/closes
    use the seat manager, refuse inactivity and latch acquisition/cleanup failures.
    Six new tests include real libinput refusal and Unix descriptor EOF evidence.
    Linux shell 201 tests and three doctests, affected Windows/Linux checks,
    warnings-denied Linux rustdoc/examples and WSLg regression (115 surfaces) pass.
    Report: `docs/autonomy/updates/session-owned-input-descriptors.md`. Context
    ownership, event dispatch, pause/device-removal cleanup and live DirectSession
    wiring remain next. Safe standalone GLES and real DRM/seat/physical acceptance
    remain owed; independent supervisor gates pending, compositor unchecked.
  - Direct keyboard and input retirement (2026-09-08): activity-checked evdev
    routing, pause focus/grab cleanup and both input capabilities cleared before
    direct-loop output retirement. Five new tests pass, including real client
    releases/modifiers/serial refusal and stop/pause cleanup before failed disable.
    Linux shell: 195 tests and three doctests; affected Windows/Linux checks,
    Linux rustdoc/examples and WSLg regression (115 surfaces) pass. Report:
    `docs/autonomy/updates/direct-keyboard-input-retirement.md`. Libinput acquisition,
    open/dispatch/device-removal handling and live session wiring remain next.
    Safe standalone GLES initialization and real DRM/seat/physical acceptance
    remain owed; full independent supervisor gates pending, compositor unchecked.
  - Bounded direct-pointer routing (2026-09-08): relative and normalized absolute
    motion map to scale-one output coordinates, with edge clamping, unchanged
    state on refusal and pause drag cleanup. Five focused tests pass; Linux shell
    190 tests plus three doctests, affected Windows/Linux clippy/fmt/tests, Linux
    rustdoc/examples and WSLg regression (115 surfaces) pass. Report:
    `docs/autonomy/updates/bounded-direct-pointer-routing.md`. Live libinput and
    keyboard pause wiring remain next. Standalone GLES construction currently
    requires unsafe Smithay APIs forbidden by workspace policy; no gate exception
    or engine patch was made. Safe construction, direct session wiring and real
    DRM/seat/physical acceptance remain owed. Compositor and release unchecked;
    full independent publication gates remain with the supervisor.
  - Session-driven direct frame loop (2026-09-08): fresh atomic discovery and
    the GLES target now run within the active descriptor scope. Polling surrounds
    pacing, idle dispatch remains active, and stop/failure retires and drops before
    close. Post-commit cleanup failure stops immediately; retirement flushes queued
    output events without more client dispatch. Four focused tests pass; Linux
    shell 185 tests plus three doctests, affected Windows/Linux fmt/clippy/tests,
    Linux rustdoc/examples and WSLg offscreen/nested checks pass (115 surfaces).
    Report: `docs/autonomy/updates/session-driven-direct-frame-loop.md`.
    Next: native GLES initialization, direct input and boot/session wiring.
    Caller still supplies the current renderer; real DRM/seat, recovery and
    physical acceptance remain owed. Supervisor full gates remain independent.
  - Scoped session pause polling (2026-09-08): `with_active_device` lends a device
    and nonblocking poll across a rendering lifetime. Pause/activate batches stay
    interrupted; descriptor close follows caller retirement/drop, including unwind.
    Six new calloop/real-descriptor happy/refusal tests and a compile-fail escape
    check pass. Linux shell: 181 tests and three doctests; affected clippy/rustdoc,
    Windows checks and WSLg offscreen/nested regressions pass (115 client surfaces).
    Report: `docs/autonomy/updates/scoped-session-pause-polling.md`.
    Next: connect direct renderer/input and server retirement to this scope.
    Upstream libseat acknowledgement ordering, physical DRM/GPU acceptance and
    full supervisor publication gates remain owed; no compositor/release tick.
  - Explicit output retirement (2026-09-08, recovered): target retirement and server
    withdrawal preserve callbacks, reject further direct submission and retain
    inert global bindings to avoid disconnecting delayed binders. Initial real
    wire test passed; owner-authorized recovery uses connector 4, distinct from
    CRTC 2 and plane 3, with an independently configured exact transport oracle.
    Eight focused tests pass, including aliased-route refusal, wrong-target
    refusal, delayed bind and no repeated disable. WSLg offscreen and nested
    regressions pass. Report: `docs/autonomy/updates/explicit-output-retirement.md`.
    Full Windows/Linux workspace, Linux rustdoc and pinned BPF gates pass.
    Automatic seat pause, direct renderer/input and physical records remain
    owed. Explicit retirement is code-ready; compositor and release are not complete.
  - Truthful output metadata (2026-09-07): backend identity, kernel connector
    millimetres and timing-derived millihertz replace shared nested placeholders.
    Validation and frozen identity refuse before submission; output globals/modes
    publish after successful frames. Three new tests and expanded direct protocol
    checks; 174 Linux shell checks, affected fmt/clippy/tests and rustdoc pass.
    WSLg nested wire-metadata/popup/cursor regression passes (115 client surfaces).
    Owner-authorized recovery (2026-09-08) fixes the offscreen client's initial
    output-global bind handshake without relaxing membership/callback assertions;
    normal offscreen passes twice, invalid EGL refuses as expected.
    Report: `docs/autonomy/updates/truthful-output-metadata.md`. Only afterwards:
    output retirement on session pause and direct renderer/input/session wiring.
    Full Windows/Linux workspace, Linux rustdoc and pinned BPF gates pass after
    recovery. Physical records remain owed; no compositor tick.
  - Compositor-owned default cursor (2026-09-07): positioned arrow snapshots,
    shared nested/offscreen GLES rendering, tip hotspot, clipping and hidden/
    client fallback. Four new tests; 171 Linux shell checks, affected Windows/
    Linux fmt/clippy/tests, Linux rustdoc/examples and WSLg golden pixels/refusal/
    nested regression (115 client surfaces) pass. Report:
    `docs/autonomy/updates/compositor-owned-default-cursor.md`. Next: truthful
    output metadata, then pause/direct input/session wiring. Direct scanout,
    asynchronous transport, GPU context-loss and physical acceptance remain owed;
    supervisor full publication gates pending. Compositor/release unchecked.
  - Synchronous direct frame target (2026-09-07): GLES preparation plus initial
    activation/replacement now implement FrameTarget. Committed identities alone
    publish callbacks/membership; cleanup failure halts further rendering and
    shutdown retains prior and disable errors. Five new tests and 167 Linux shell
    checks pass; affected Windows/Linux fmt/clippy/tests, Linux rustdoc/examples
    and WSLg public-target refusal/nested regression (125 client surfaces) pass.
    `docs/autonomy/updates/synchronous-direct-frame-target.md`. Successful DRM is
    injected. Default cursor, truthful direct-output metadata, pause/direct input/
    session wiring, asynchronous transport and physical acceptance remain owed.
    Supervisor full publication gates pending; compositor and release unchecked.
  - Synchronous scene replacement (2026-09-07): frozen-route allocation/upload/
    blocking commit preserves old ownership on refusal, retires it only after
    success and reports post-commit cleanup separately. Five new tests, including
    real Wayland resource identities, and 162 Linux shell checks pass; affected
    Windows/Linux fmt/clippy, Linux rustdoc/examples and WSLg regression pass
    (135 client surfaces). `docs/autonomy/updates/synchronous-scene-replacement.md`.
    Successful DRM is injected. Direct FrameTarget/callback scheduling, default
    cursor, pause/input/session wiring, asynchronous transport and physical
    acceptance remain; supervisor full gates remain owed. Compositor unchecked.
  - Prepared scene activation (2026-09-07): consuming full-mode validation,
    upload and blocking TEST_ONLY/enable retain identities with active ownership.
    Four new tests; 157 Linux shell checks, affected Windows/Linux fmt/clippy/tests,
    Linux rustdoc/examples and WSLg scene/nested checks pass (134 client surfaces).
    Evidence: `docs/autonomy/updates/prepared-scene-activation.md`. Successful
    DRM is fault-injected; actual non-DRM ioctl refusal preserves callbacks.
    Continuous direct FrameTarget, default cursor, replacement/retirement,
    pause/input/session wiring and physical acceptance remain. Supervisor full
    publication gates remain owed; compositor and release stay unchecked.
  - Offscreen scene rendering (2026-09-07): shared window/popup/cursor painter
    produces owned XRGB pixels and drawn identities without submission callbacks.
    Extent and real truncated-SHM refusal, all 1,056 scene pixels, clipping,
    stacking, callback timing and disconnect checked. 153 Linux shell checks,
    affected Windows/Linux fmt/clippy/tests, Linux rustdoc/examples pass; WSLg
    nested regression submits 137 client surfaces. Evidence:
    `docs/autonomy/updates/offscreen-scene-rendering.md`. Successful submission
    is fixture-only; direct FrameTarget/upload/scanout integration, default cursor,
    cookie transport, retirement, pause/input/session wiring and physical evidence
    remain. GPU context-loss/draw/readback fault evidence remains owed. Supervisor
    full publication gates pending; compositor and release remain unchecked.
  - GLES scanout readback (2026-09-07): full-target safe export, checked signed
    byte-count bounds, mapping metadata/length refusal, explicit row orientation
    and RGBA-to-XRGB conversion into immutable upload sources. Six new tests;
    151 Linux shell checks, affected Windows/Linux fmt/clippy/tests, Linux rustdoc
    and examples pass. WSLg verifies six exact offscreen pixels in each of two
    orientations; invalid EGL refuses; nested regression submits 130 client
    surfaces. Evidence: `docs/autonomy/updates/gles-scanout-readback.md`.
    Conversion/upload/enable/disable lifetime test uses fake DRM transport.
    Scene-to-offscreen rendering, direct presentation, safe cookie transport,
    retirement, pause/input/session integration and physical acceptance remain;
    compositor unchecked and supervisor full publication gates still owed.
  - Unbound scanout frame upload (2026-09-07): validated full-size XRGB CPU frames,
    independent strides, padding/tail clearing and consuming refusal with all
    cleanup errors retained. Six new tests; 145 Linux shell checks, affected
    Windows/Linux fmt/clippy/tests, Linux rustdoc/examples and WSLg regression
    (128 client surfaces) pass. Evidence:
    `docs/autonomy/updates/unbound-scanout-frame-upload.md`. Fault-injected
    allocation/upload/enable/disable integration is not successful DRM evidence.
    Renderer readback/conversion, cookie-bearing transport, buffer retirement,
    pause ordering, direct input, production entry and physical acceptance remain.
    Compositor unchecked; full supervisor publication gates still owed.
  - Session-scoped flip completion gate (2026-09-07): process-unique cookies,
    one pending submission and exactly-once cookie/CRTC matching, integrated with
    bounded event reads. Eight new happy/refusal tests; 139 Linux shell checks,
    affected Windows/Linux fmt/clippy/tests, Linux rustdoc/examples and WSLg
    regression (135 client surfaces) pass. Evidence:
    `docs/autonomy/updates/session-scoped-flip-completion.md`. This owns identity,
    not buffers. Safe cookie-bearing kernel transport, pending-buffer retirement,
    rendering, pause ordering, direct input and production entry remain open.
    No successful DRM flip or physical evidence; full supervisor gates still owed.
  - Cookie-preserving page-flip event reading (2026-09-07): bounded native ABI
    decoding and nonblocking borrowed-fd reads preserve full user_data and CRTC
    identity, refuse corrupt batches and retain kernel errno. This prerequisite
    avoids pinned drm-rs dropping the cookie, without changing upstream engines.
    Evidence: `docs/autonomy/updates/page-flip-event-reading.md`. Atomic cookie
    submission, pending-commit matching and buffer retirement remain unfinished;
    no successful DRM flip or physical acceptance is claimed. Supervisor full
    publication gates remain owed; compositor stays unchecked.
    Ten new tests pass (131 Linux shell checks total); affected Windows/Linux
    fmt/clippy/tests, Linux rustdoc/examples and WSLg regression (133 client-surface
    submissions) pass. Real descriptor integration uses synthetic DRM event bytes.
  - Blocking scanout ownership (2026-09-07): initialized resources transfer to
    an active owner after TEST_ONLY and synchronous enable. Synchronous disable
    precedes destruction; disable refusal quarantines handles until session-device
    retirement. Evidence: `docs/autonomy/updates/blocking-scanout-ownership.md`.
    Nonblocking page-flip retirement, rendered frames, pause ordering, direct input,
    production entry and successful DRM/physical checks remain owed. This is one
    ownership component; compositor and release remain unchecked.
    Six new unit tests plus an active-lifetime doctest pass (121 Linux shell
    checks). Affected Windows/Linux fmt/clippy/tests, Linux rustdoc/examples,
    real atomic enable/disable ENOTTY/fd survival, seat/usage refusal and WSLg
    125-surface regression pass. Supervisor full publication gates remain owed.
  - Scanout-buffer initialization (2026-09-07): full mapped allocation cleared
    to black and unmapped before framebuffer registration, including row padding
    and allocation tail. Mapping refusal unwinds buffer ownership and retains
    cleanup errors. Five new tests pass; 114 Linux shell checks including doctest.
    Evidence: `docs/autonomy/updates/scanout-buffer-initialization.md`.
    Successful DRM mapping/unmapping, active commit/page-flip retirement, renderer
    pause ordering, direct input and production entry remain owed. The pinned
    mapping destructor's unmap panic boundary is documented in quirks. All physical
    acceptance and full supervisor publication gates remain; compositor unchecked.
  - Atomic configuration validation (2026-09-07): immutable full-mode request,
    TEST_ONLY | ALLOW_MODESET and explicit cleanup on acceptance/refusal. Six new
    tests pass (109 Linux shell checks including doctest); Windows/Linux affected
    fmt/clippy/tests, Linux rustdoc/examples and WSLg 115-surface regression pass.
    Real atomic ioctl ENOTTY/fd survival and absent-card/usage refusals verified.
    Evidence: `docs/autonomy/updates/atomic-display-configuration-validation.md`.
    Successful DRM validation needs a DRM-equipped login/VM. Scanout/page flips,
    renderer pause ordering, direct input, production entry and all physical
    acceptance remain; full supervisor publication gates still owed.
  - Direct-display resource ownership (2026-09-07): borrowed-device full-mode
    XRGB8888 buffer/framebuffer and exact mode-blob lifetime, partial-allocation
    unwind and complete cleanup-error reporting. Eight new unit tests plus a
    lifetime doctest pass (103 Linux shell checks total). Windows/Linux affected
    fmt/clippy/tests, Linux rustdoc/examples and WSLg 115-surface regression pass;
    real CREATE_DUMB ENOTTY and absent-card ENOENT refusals verified. Evidence:
    `docs/autonomy/updates/direct-display-resource-ownership.md`. Successful DRM
    allocation, atomic TEST_ONLY, scanout/page flips, renderer pause ordering,
    direct input and all physical acceptance remain owed. No compositor tick;
    supervisor full publication gates pending.
  - Atomic display property discovery (2026-09-07): per-descriptor atomic and
    universal-plane negotiation, deterministic primary-plane selection and
    standard property schema checks. Framebuffer/blob ownership, kernel TEST_ONLY,
    scanout/page flips, direct input and renderer pause ordering remain unfinished.
    Evidence: `docs/autonomy/updates/atomic-display-property-discovery.md`.
    WSL has no DRM card; successful kernel discovery and all physical acceptance
    remain unmeasured. Eight new tests pass (94 Linux shell tests); Windows/Linux
    affected fmt/clippy/tests and Linux rustdoc/examples pass. Real ENOTTY/ENOENT
    refusals and WSLg 115-surface regression pass. Supervisor full publication
    gates remain owed. No compositor or release checkbox changes.
  - Direct-display session lifetime (2026-09-07): libseat device acquisition,
    pause retirement, lazy reacquisition and terminal failure handling implemented
    as a discovery-stage component. Nine new tests pass (86 Linux shell tests),
    including calloop handoff and actual descriptor EOF checks. Unavailable-seat
    diagnostic refuses ENOENT; WSLg submits 115 client surfaces. Exact commands:
    `docs/autonomy/updates/direct-display-session-lifetime.md`. Successful DRM
    acquisition and physical pause/resume remain unmeasured. Atomic modesetting,
    renderer lifecycle, scanout/page flips, direct input and production session
    still owed. Pinned Smithay has internal libseat panic/disable-order limits;
    see docs/quirks.md. Supervisor full gates remain owed; box unchanged.
  - Direct-display resource discovery (2026-09-07): read-only session-descriptor
    queries choose one usable internal panel or stable external port, a compatible
    CRTC and an exact advertised progressive mode. Seven new tests pass (77 Linux
    shell tests total); real non-DRM ioctl and absent-card diagnostics refuse.
    WSLg regression submits 115 client surfaces. Exact checks and limits:
    `docs/autonomy/updates/direct-display-resource-discovery.md`. WSL lacks
    `/dev/dri`; successful DRM queries and physical scanout are not measured.
    Next: session device ownership and pause/resume, then atomic modesetting,
    page flips and direct input. Parent leave/session UI and all physical
    acceptance remain; supervisor independent gates still owed. Box unchanged.
  - Reactive popup placement (2026-09-07): opted-in mapped menus reconstrain on
    output/committed parent changes, preserving configure/commit ordering and
    avoiding duplicate geometry events. Five new socket tests pass (70 Linux
    shell tests total); WSLg reactive popup/cursor submission passes. Exact checks
    and development corrections: `docs/autonomy/updates/native-popup-reactive-placement.md`.
    Parent leave, direct display/input, session and physical acceptance remain;
    compositor unchecked and supervisor full publication gates still owed.
  - Popup output constraints (2026-09-07): initial and explicit placement honor
    client flip/slide/resize flags against the single output, translated through
    committed parent window geometry. Four new socket tests pass (65 Linux shell
    tests total); WSLg constrained popup/cursor submission passes. Exact checks:
    `docs/autonomy/updates/native-popup-output-constraints.md`. Reactive placement,
    parent leave, direct display/input, session and physical acceptance remain.
    No compositor box changes; supervisor full publication gates remain owed.
  - Development prerequisite: `alo-graphics-check` pins Smithay 0.7.0 and
    compiles nested and direct-display dependencies. On 2026-09-07 it submitted
    a 320x200 GLES frame through WSLg and refused an unavailable EGL vendor.
    Ubuntu setup and exact verification are in `docs/autonomy/GRAPHICS.md`.
    This is delivery item 32's build fixture, not a compositor implementation.
  - Protocol component (2026-09-07): `alo-shell` owns a private Wayland socket
    and real XDG toplevel configure/buffer lifetimes. Eleven Linux integration
    tests cover map/unmap/remap, buffer release, client/server teardown, isolated
    refusal and socket ownership. A captured protocol trace verifies fd-backed
    SHM transfer and fresh configure serials after unmap; see
    `docs/autonomy/COMPOSITOR.md`.
  - Nested rendering component (2026-09-07): `Nested` and `Server::render`
    submit real SHM surface trees through GLES, advertise one resizable output
    and complete callbacks only after submission succeeds. Fifteen Linux tests
    pass, including failed-submission callback retention and output lifecycle.
    The WSLg fixture verifies root/child callbacks, offscreen-child withholding,
    unmap/remap, isolated refusal and disconnect; invalid EGL and missing display
    refuse startup. Focused checks and exact evidence are in `COMPOSITOR.md`.
    This remains backend code, not a usable desktop. Pointer routing,
    popups, direct display and physical keyboard/pointer/display evidence remain
    owed, along with the supervisor's independent full publication gates.
  - Keyboard component (2026-09-07): an explicitly configured XKB keyboard seat,
    mapped-root focus and nested parent key routing. Real clients verify keymap,
    repeat, modifier/key delivery and isolation, held-key cleanup on focus loss,
    unmap/destruction/disconnect, and stale/foreign focus refusal. Invalid layouts
    remove the failed display's socket. Item 33 remains
    incomplete. Exact focused checks and remaining machine evidence are recorded
    in `docs/autonomy/COMPOSITOR.md`.
  - Pointer core component (2026-09-07): optional pointer capability, mapped
    surface-tree/input-region hit testing, motion/buttons/scroll and implicit
    drag isolation. Four additional real-client tests cover child coordinates,
    refusal and leave/unmap/destruction/disconnect cancellation; 24 Linux tests
    pass. Wire trace and WSLg rendering regression passed. Nested pointer events,
    cursor presentation, popups and direct display remain unfinished; focused
    checks do not replace supervisor full gates or physical acceptance.

  - Nested pointer bridge (2026-09-07): `pump_seat` routes parent physical motion,
    evdev buttons and wheel/pixel scroll while active. Deactivation/close cancel
    drags; reactivation requires fresh motion. Four new tests bring Linux coverage
    to 28; real-client wire trace and WSLg seat/rendering regression pass.
    Smithay 0.7 drops parent cursor-leave events: leave notifications and client
    cursor presentation remain next, followed by popups and direct display.
    Full supervisor gates and physical input/display acceptance remain owed.

  - Client cursor component (2026-09-07): focused-client cursor surfaces render
    above windows with hotspot positioning, hidden requests and lifecycle cleanup.
    Three new real-client tests bring Linux coverage to 31; stale/unfocused requests,
    conflicting roles and failed submission are covered. WSLg submits a real cursor
    buffer, clips extreme hotspots and verifies callbacks/unmap; ordinary rendering
    regression also passes. Exact focused checks are in `COMPOSITOR.md`.
    Parent-leave notifications, popups, direct display, full supervisor gates and
    physical acceptance remain owed. Item 33 and this feature stay unchecked.

  - Popup protocol component (2026-09-07): opt-in popup protocol lifecycle tracking
    now provides configured-buffer snapshots, mapped-parent validation and terminal
    dismissal. Five new real-client tests pass (36 Linux shell tests total).
    WSLg verifies protocol coexistence with GLES and withheld callbacks for an
    unrendered popup; this is not popup presentation. See
    `docs/autonomy/updates/native-popup-protocol-lifetimes.md` for exact checks.
    Next: popup-aware rendering/input; nested chains, grabs, repositioning/output
    constraints, parent-leave backend support, direct display and physical acceptance
    remain unfinished. No feature box moves; supervisor full gates remain owed.

  - Popup presentation component (2026-09-07): the opt-in nested backend now draws
    non-grabbing popup trees and routes pointer input with shared XDG geometry
    placement and parent-relative stacking. Unsupported targets and failed swaps
    retain callbacks; dismissal/parent loss cancel implicit drags. Four new socket
    tests pass (40 Linux shell tests total). WSLg submits popup buffers and checks
    offscreen clipping, callback/output membership and dismissal. Exact commands
    and limits: `docs/autonomy/updates/native-popup-presentation.md`.
    Nested popup chains, grabs, output constraints/repositioning, parent-leave
    notifications, direct display and physical acceptance remain. The compositor
    stays unchecked; full independent publication gates belong to the supervisor.

  - Nested popup chains component (2026-09-07): mapped popup parents, accumulated
    geometry and descendant-first stacking/cleanup are implemented. Three new
    real-client tests pass (43 Linux shell tests); WSLg nested popup/cursor
    submission and callbacks pass. `docs/autonomy/updates/native-popup-chains.md`
    records checks and limits. Explicit grabs, repositioning/output constraints,
    parent leave, direct display and physical acceptance remain unfinished.
    No compositor box changes; supervisor publication gates remain pending.

  - Explicit popup repositioning (2026-09-07): validated positioners receive
    ordered token/configure responses; acknowledged commits update the shared
    render/input geometry and descendants. Three new real-client tests pass
    (61 Linux shell tests total); WSLg repositioned popup/child and cursor
    submission pass. Exact checks: `docs/autonomy/updates/native-popup-repositioning.md`.
    Output constraints/reactive placement, parent leave, direct display/input,
    session and physical acceptance remain. Compositor unchecked; full supervisor
    publication gates still owed.

  - Pointer-release popup initiation (2026-09-07): a matched real release
    authorizes one parent menu, with subsurface ownership and submenu inheritance.
    New button events, pointer recipient/lifetime loss and consumption invalidate
    it. Synthetic releases and final drag releases outside the recipient refuse.
    Three new socket tests pass (58 Linux shell tests total); WSLg pointer-release,
    pointer-press and keyboard-release checks pass. Exact checks and limits:
    `docs/autonomy/updates/native-popup-pointer-release-initiation.md`.
    Repositioning/output constraints, parent leave, direct display/input, session
    and physical acceptance remain unfinished; compositor stays unchecked and
    full independent supervisor publication gates remain owed.

  - Keyboard-release popup initiation (2026-09-07): the latest matched real key
    release authorizes one root grab on its focused parent; supersession,
    focus/lifetime loss and consumption invalidate it. Synthetic releases refuse.
    Three new socket tests pass (55 Linux shell tests); WSLg release initiation,
    key-press and pointer-grab regressions pass. Exact checks and limitations:
    `docs/autonomy/updates/native-popup-keyboard-release-initiation.md`.
    Pointer-release initiation, repositioning/output constraints, parent leave,
    direct display/input, session and physical acceptance remain unfinished.
    No compositor box changes; supervisor full publication gates remain owed.

  - Keyboard-triggered popup grabs (2026-09-07): latest held parent key press
    authorizes one root grab; submenus inherit the active chain serial. Release,
    supersession, focus/lifetime loss and consumption prevent stale reuse. Three
    new socket tests pass (52 Linux shell tests); WSLg keyboard-grab submission
    and pointer-grab regression pass. Exact focused checks and limits:
    `docs/autonomy/updates/native-popup-keyboard-grabs.md`. Release-triggered
    initiation, repositioning/output constraints, parent leave, direct display,
    session integration and physical acceptance remain. Compositor stays unchecked.

  - Pointer-triggered popup grabs (2026-09-07): active parent press/seat validation,
    topmost keyboard routing, owner-client pointer isolation, nested focus return
    and consumed outside clicks. Six new socket tests pass (49 Linux shell tests),
    including stale/foreign/late requests, serial replay, invalid parent/order,
    stationary hits and held-button cleanup. WSLg submitted a grabbed popup and
    verified callbacks, input and dismissal; popup/cursor regression also passed.
    Exact focused Windows/Linux checks and limits:
    `docs/autonomy/updates/native-popup-pointer-grabs.md`.
    Keyboard/release-triggered initiation, repositioning/output constraints, parent
    leave, direct display/input and physical acceptance remain. No feature box
    changes; supervisor full publication gates remain owed.

- [ ] **Sign-in**: alo identity, and a local account that needs no tenant

- [ ] **The agent overlay**: one key, from anywhere

- [ ] Launcher and window management: move, resize, snap, tile
  - Externalized reader navigation model (2026-09-09): complete position and
    previous/next/dismiss wording retains per-string provenance; vocabulary
    collisions, missing wording and invalid positions refuse. Live semantic
    navigation validates lifetime before boundary refusal, without wrapping or
    window dispatch. Three new private-client/vocabulary tests; Linux 162 unit,
    223 lifecycle, three socket and four doctests pass, affected Linux/Windows
    checks and existing 72 GLES page frames/nested controls pass. Chrome layout,
    raster, input ownership and reader transaction remain next; then native
    navigation/cursor selection and direct integration. No feature tick; full
    supervisor gates pending. `docs/autonomy/updates/externalized-native-reader-navigation.md`.
  - Paged native control label rendering (2026-09-09): immutable whole-line pages
    preserve full translation/source provenance and text scale. Foreign/clipped
    placement, lost ink and excessive page allocation refuse atomically; a page
    cannot masquerade as a complete scene label. Linux 162 unit/216 lifecycle/
    three socket/four doctests, affected Linux/Windows clippy/tests/fmt, Linux rustdoc,
    72 new and 72 existing full GLES frames and nested control transactions pass.
    Live mapping-bound reader navigation, page position wording, input ownership
    and submission/retirement remain next, then native navigation/cursor selection
    and direct integration. Full supervisor publication gates pending; no feature
    checkbox changes. `docs/autonomy/updates/paged-native-control-label-rendering.md`.
  - Adaptive native labels (2026-09-09), **graphical recovery verified**: fitting boxes
    stay; clipped names expand below/above controls at unchanged text scale and
    retain translation/source fallback. Exhausted space refuses. Linux 159 unit,
    216 lifecycle, three socket/three doctests and affected clippy/tests pass;
    examples compile. The earlier unexplained EGL timeout is preserved in the
    report; traced and normal nested runs now pass. Two explicit pre-submission
    scene stages preserve all pixel/callback assertions and deadlines; all 30
    offscreen stages pass. Full Windows/Linux/rustdoc/BPF and graphical gates
    passed on the combined tree. Paged names on
    smaller outputs remain next; no feature tick.
    `docs/autonomy/updates/repairing-an-unfinished-desktop-task.md`.
  - Transactional native label composition (2026-09-09): fresh hover/live native
    focus labels now submit with the strip; only success publishes opaque pointer
    exclusion. Covered clicks/scroll cannot reach clients, owned releases drain
    after removal/failure/leave and client grabs/typing remain intact. Five new
    real-client tests, affected tests/clippy/rustdoc/fmt, actual light/dark nested
    EGL labels/dismissals/refusal recovery and all 28 offscreen stages pass.
    Alternate full-text access, navigation/cursor selection and direct integration
    remain. No feature tick; independent supervisor gates pending. Evidence:
    `docs/autonomy/updates/transactional-native-label-composition.md`.
  - Transactional native control submission (2026-09-09): fresh explicit strips
    now share backend submission and callback publication, with authority retired
    on validation/submission failure, omitted targets and ordinary-frame removal.
    Three new real-client tests, six nested EGL strip submissions, two removals
    and two refusal/recovery sequences pass. All 28 offscreen stages pass. Initial
    nested deadline failure and evidence limits are retained in the report.
    Labels with overlay input policy are next, followed by full-text access,
    navigation/cursor selection and direct integration. No feature tick; full
    supervisor gates pending. `docs/autonomy/updates/transactional-native-control-submission.md`.
  - Native control scene composition (2026-09-09): nested submission and offscreen
    preparation share client/popup, strip, complete label, cursor ordering. Stale
    viewports, clipped labels and labels covering controls refuse before import.
    One new geometry test and twelve complete real-client GLES frames pass, with
    surface identity preservation, removal and refusal recovery. Affected checks
    and rustdoc pass; independent supervisor gates pending. Host presentation
    transactions, overlay input policy, alternate full-text access, navigation/
    native cursor selection and direct integration remain. No feature tick.
    Evidence: `docs/autonomy/updates/native-control-scene-composition.md`.
  - Nested native control event routing (2026-09-09): the seat pump now uses the
    published presentation and actual parent position through native grabs.
    Out-and-back motion and deactivation cancel execution; cancelled releases
    drain while inactive or before fresh motion. Client typing/buttons/scroll
    remain available. Four real-client tests, eight new complete GLES label
    frames and live minimization pass, alongside affected Linux/Windows checks
    and rustdoc. Composition, overlay policy, full clipped-name access, native
    navigation/cursor integration and direct integration remain unfinished.
    No feature tick; independent supervisor gates pending. Evidence/limits:
    `docs/autonomy/updates/nested-native-control-event-routing.md`.
  - Mapping-bound native control presentation (2026-09-09): server-owned explicit
    painted target, visibility identity, geometry/intent and native label focus
    now share a lifecycle. Replacement, invalid candidates, hide/remap, pointer
    deactivation and seat reset retire focus and disarm held execution without
    losing release ownership. Same-frame refresh preserves valid gestures. Five
    real-client tests and ten complete GLES lifetime frames pass; affected Linux
    and Windows checks, rustdoc and graphical regressions pass. Nested composition/
    event pumping, native navigation, overlay hit policy and full clipped-text
    access remain next, then direct integration. No feature tick or installed
    controls claim; independent supervisor gates pending. Exact evidence/limits:
    `docs/autonomy/updates/mapping-bound-native-control-presentation.md`.
  - Live native control label presentation (2026-09-09): read-only current hover
    or explicitly supplied native focus selection, disabled name access, bounded
    below/above/clamped placement and dismissal on absent/hidden/foreign targets
    or held/competing input. Three new real-client tests include prepared text,
    small/invalid outputs and ordinary typing isolation. Linux 156 unit/198 lifecycle/
    three socket/three doctests, affected clippy/examples/rustdoc/fmt and Windows
    affected checks pass. WSLg adds fourteen complete 5,760-pixel hide/reveal frames;
    28 nested stages, 72 label frames and 896 control frames pass. Production
    nested/direct composition, native focus dispatch and full clipped-text access
    remain integration work. No usable-controls or feature tick; independent
    supervisor gates pending. `docs/autonomy/updates/live-native-control-label-presentation.md`.
  - Native control label rendering (2026-09-09): externalized `Action::said` names,
    retained translation/fallback provenance, bundled Inter, advanced shaping,
    bounded wrapping/clipping and the existing 75..300% text scale. Disabled
    controls retain readable names. Font, glyph, vocabulary, length and geometry
    refusals are explicit. Live label placement/selection/dismissal, overlay input
    ownership and nested/direct production composition remain integration work;
    no window-management or release box is promoted. Component evidence and exact
    verification: `docs/autonomy/updates/native-control-label-rendering.md`.
    Four new tests; Linux 156 unit/195 lifecycle/three socket/three doctests and
    affected clippy, examples, rustdoc pass; Windows affected checks pass. WSLg
    passes 72 complete label frames, 896 control frames and 28 offscreen stages.
    Initial bearing-clipping defect and indexing lints corrected. Supervisor
    independent publication gates remain pending.
  - Native control pointer feedback (2026-09-09): immutable hover/armed-press
    presentation shares live mapping, hit and availability checks. Disabled,
    cancelled, foreign and stale gestures remain idle; client input is preserved.
    Two unit/three real-client tests added. Linux affected clippy, 152 unit/195
    lifecycle/three socket tests and three doctests, rustdoc/examples and fmt
    pass; Windows affected clippy/tests/fmt pass (zero Linux-only tests).
    WSLg passes 896 complete painter frames and 28 offscreen stages, including
    eight new live feedback frames and the existing twelve snapshot frames.
    Initial example dead-code lint corrected without exemptions. Native labels
    and production nested/direct composition follow; no usable-control or feature
    completion. Full supervisor gates pending. Evidence and machine limits:
    `docs/autonomy/updates/native-control-pointer-feedback.md`.
  - Native control pointer routing (2026-09-09): one trusted entry point consumes
    native primary gestures and motion, with ordinary client fallback. Current
    painted target replacement/removal cancels even at unchanged geometry.
    Four new real-client tests; Linux affected clippy, 150 unit/192 lifecycle/
    three socket tests and three doctests, rustdoc/examples and fmt pass.
    Windows affected clippy/tests/fmt pass (zero Linux-only tests). WSLg passes
    all 28 offscreen stages and 128 full painter frames; routed cancellation,
    minimize and restore retain complete 6,400-pixel scene assertions. No on-screen
    or direct-input evidence claimed. Labels/feedback and production nested/direct
    composition remain next; no feature tick. Full supervisor gates pending.
    `docs/autonomy/updates/native-control-pointer-routing.md`.
  - Native control motion cancellation (2026-09-09): out-and-back gestures,
    relayout and transient intent changes cannot rearm a press. Pointer leave and
    seat reset cancel even when pointer cleanup refuses, retaining release
    ownership. Four new real-client tests; Linux affected clippy, 150 unit/188
    lifecycle/three socket tests and three doctests, rustdoc/examples, Windows
    affected checks and both fmt checks pass. WSLg passes 28 offscreen stages,
    one new full 6,400-pixel cancellation frame and 128 painter regression frames.
    Primary event interception, labels/feedback and production composition remain;
    this completes cancellation only. Full supervisor gates pending; no feature
    tick. `docs/autonomy/updates/native-control-motion-cancellation.md`.
  - Mapping-bound control transactions (2026-09-09): disabled-hit ownership,
    duplicate isolation, explicit cancellation and exactly-once live release
    validation are code-ready. Unmap/remap and hide/reveal retire authority;
    existing client grabs refuse without theft. Seven new real-client tests;
    final Linux clippy, 150 unit/184 lifecycle/three socket tests and three
    doctests, rustdoc/examples, Windows affected checks and both fmt checks pass.
    WSLg's 28 offscreen stages include two additional full 6,400-pixel scene
    checks through native minimization/restoration; 128-frame painter passes.
    One new fixture assertion corrected to existing popup consumed-key behavior.
    Production event routing, motion/leave/reset handling, labels, feedback and
    nested/direct scene integration remain. Full supervisor gates pending;
    no feature tick. `docs/autonomy/updates/mapping-bound-window-control-transactions.md`.
  - Live window control snapshots (2026-09-09): explicit visible roots derive
    availability from the same read-only planner as execution. Latest requested
    mode selects maximize/restore; output, geometry, limits and busy refusals
    remain authoritative. Six new real-client tests, affected Linux clippy,
    150 unit/177 lifecycle/three socket tests plus three doctests, rustdoc/examples,
    Windows affected checks and Windows/Linux fmt pass. WSLg passes all 28
    offscreen stages, including 12 snapshot-derived full 5,760-pixel frames;
    the existing 128-frame painter regression also passes. Snapshots are frozen
    presentation data, not mapping-lifetime input authority. Next: mapping-bound
    press/release ownership, live revalidation/cancellation, native labels and
    production composition. Full supervisor gates pending; no feature tick.
    Evidence: `docs/autonomy/updates/live-window-control-snapshots.md`.
  - Native window control view (2026-09-08): immutable minimise/maximise-or-restore/
    close layout shares clipped paint/hit rectangles; disabled hits remain owned
    and carry a non-color mark. Light/dark appearance tokens and existing action
    labels avoid a second palette or vocabulary. Six new tests, Linux affected
    clippy, 150 unit/171 lifecycle/three socket tests plus three doctests,
    rustdoc/examples, Windows affected checks and both fmt checks pass. WSLg
    validates 128 full 5,760-pixel frames against independent glyph masks.
    Next: live target snapshots and availability, native label presentation,
    pointer ownership/cancellation and stale-target/typing/graphical interaction.
    This is the completed view component, not usable controls; no feature tick.
    Full supervisor gates pending. Evidence and limits:
    `docs/autonomy/updates/native-window-control-strip-painting.md`.
  - Focused layout command dispatch (2026-09-08): an additive native API resolves
    configured minimise, maximise/restore and left/right snap using actual keyboard
    ownership and latest requested mode. The original close/cycle API and exhaustive
    errors remain compatible. Seven new real-client tests, Linux affected clippy,
    144 unit/171 lifecycle/three socket tests plus three doctests, rustdoc/examples,
    Windows affected checks and both formatting checks pass. WSLg passes all 28
    offscreen stages, including configured minimise/snap full-frame boundaries,
    and the 115-surface nested regression. Full supervisor gates remain pending.
    Next: rendered native controls and pointer press/release dispatch with localized
    labels, refusal/disabled states and input isolation. Raw key/settings integration
    remains unfinished. Exact evidence and limits:
    `docs/autonomy/updates/focused-window-layout-command-dispatch.md`.
    No window-management or release checkbox is promoted.
  - Trusted tiling/restoration (2026-09-08), verified after authorized recovery:
    shared normal/maximized/tiled memory, latest acknowledged-commit placement,
    output/limit suspension and lifecycle isolation are tested. Nine transaction
    cases, missing-event and maximize compatibility regressions, 164 lifecycle
    tests, Windows/Linux workspace gates, rustdoc and pinned BPF checks pass.
    All 28 offscreen stages pass, including six full 6,400-pixel tile/restore
    boundaries; the nested regression passes 115 submitted client surfaces.
    Next: rendered native controls and operation dispatch. Evidence/limits:
    `docs/autonomy/updates/trusted-window-tiling-and-restoration.md`.
    No window-management or release checkbox is promoted.
  - Bounded tiling geometry (2026-09-08): immutable left/right half-output plans
    use successfully submitted extents, committed client limits and bounded
    actual-size outside-edge anchoring. Three unit/four real-client tests, Linux
    affected clippy, 300 tests plus three doctests, rustdoc/examples and WSLg
    offscreen/nested regressions pass. Both plans preserve all 6,400 GLES pixels.
    Evidence: `docs/autonomy/updates/bounded-window-tiling-geometry.md`.
    Supervisor gates passed and published as `08a1b62`. Trusted transactions are
    verified above; native controls remain. Planning and transactions do not
    complete window management or the release.
  - Client minimize requests (2026-09-08): verified after owner-authorized recovery.
    Mapped requests share native visibility policy; pre-map intent and duplicates
    are inert. Exact order-independent capabilities are checked on map/remap.
    Both new real-client tests, 149 lifecycle tests, Windows/Linux workspace
    gates, Linux rustdoc, BPF checks and all 22 offscreen stages pass, including
    hidden/restored full-frame pixels. Next: tiling, then native controls.
    Report: `docs/autonomy/updates/client-window-minimize-requests.md`.

  - Trusted minimization/restoration (2026-09-08): buffered mapping and visibility
    are separate; hiding retires input/popups/move/resize, preserves maximize
    geometry and withholds callbacks. Restoration does not steal focus; cycling
    retains hidden ring positions. Six real-client tests, affected Linux tests,
    clippy/rustdoc/examples and four full 6,400-pixel WSLg GLES frames pass;
    nested regression passes 115 submitted surfaces. Exact commands and limits:
    `docs/autonomy/updates/trusted-window-minimization-and-restoration.md`.
    Supervisor gates passed and published as `7e5e456`. Client minimize policy
    is verified above; tiling and native controls remain.
    This completes the trusted component, not window management or the release.
  - Trusted maximise/restore transactions (2026-09-08): normal geometry survives
    output changes and rapid toggles; only the newest acknowledged committed
    response places the actual buffer. Failed output submission/retirement and
    competing operations refuse without changing state; unmap forgets it.
    Seven real-client tests, affected Linux clippy/tests/rustdoc/examples/fmt,
    Windows affected checks and five complete WSLg GLES frames pass. Full
    supervisor publication gates remain pending. Exact commands and limits:
    `docs/autonomy/updates/trusted-window-maximize-and-restore.md`. Next: client
    XDG maximize/unmaximize policy, including pre-map handling, then remaining
    minimise/tile operations and controls. Transaction component complete;
    maximise feature, window management and release remain unchecked.
  - Interactive resize transactions (2026-09-08): shared XDG press authority,
    current-limit clamping, resizing configures and committed-response anchoring
    now connect all eight edges to native input. Last release retains the anchor
    through the final response; leave/unmap/disconnect retire ownership. Evidence
    and exact checks: `docs/autonomy/updates/interactive-resize-transactions.md`.
    Full supervisor publication gates remain pending. Next: trusted maximise
    and restore with remembered normal geometry, output-bound sizing and actual
    commit placement; then remaining minimise/tile operations and native controls.
    Window management and the release remain unchecked.
  - Constrained edge resize geometry (2026-09-08): immutable mapped-root
    snapshots use committed scene bounds/limits; all eight edges clamp moving
    dimensions and anchor using the actual client size. Three unit and four
    real-client tests, affected Linux clippy/tests/rustdoc/examples and WSLg
    all-edge 6,400-pixel preservation checks pass. Windows affected checks pass
    but run no Linux runtime cases. Exact commands and limits:
    `docs/autonomy/updates/constrained-edge-resize-geometry.md`. Next: XDG resize
    press authority, configure/ack/commit state and cancellation. Geometry is
    complete; interactive resize and window management remain unfinished.
    Full independent supervisor gates remain pending.
  - Interactive window movement (2026-09-08): XDG requests require the current
    seat's held press on the exact mapped root or its subsurface tree. Drag input
    is consumed; release, leave, unmap and disconnect cancel ownership. Six
    real-client tests and positive/negative full-frame GLES checks are the
    component acceptance; exact executed results are in
    `docs/autonomy/updates/interactive-window-movement.md`. Interactive resize,
    remaining operations and rendered shell controls remain; no feature tick.
  - Native placement implementation shares drawing/input/popup origins; graphical
    fixture validation is unfinished. Interactive move/resize follows that gate;
    no usable window-management completion claim.
  - Native window activation (2026-09-08): trusted selection focuses and raises
    mapped same-display roots. XDG activation follows actual keyboard ownership,
    including popup focus, explicit clearing and lifecycle cleanup. Four socket
    tests cover state, held keys, grabs, refusal, remap and parent restoration.
    Native switching controls/shortcuts and adapter wiring remain next; window
    management and release unchecked. Exact checks and machine limits:
    `docs/autonomy/updates/native-window-activation.md`.
  - Native window raising (2026-09-08): explicit mapped-root stacking feeds
    rendering and hit testing, preserving other roots, popup subtrees and grabs.
    Foreign, stale, unmapped and popup targets refuse. Activation is now available
    above; native switching controls remain next, window management unchecked.
    Checks and limits: `docs/autonomy/updates/native-window-raising.md`.
  - Native window close requests (2026-09-08): trusted per-window XDG close
    delivery preserves application choice and refuses stale, foreign, unmapped
    and non-toplevel targets. Real socket tests cover isolation, continued input/
    rendering and voluntary destruction. Native close controls/shortcuts and
    application-adapter integration remain; no window-management or release tick.
    Evidence: `docs/autonomy/updates/native-window-close-requests.md`.

- [ ] Copy, cut and paste across applications; switching between windows
  - Stable native window cycling (2026-09-08): trusted forward/backward selection
    traverses a mapped-root ring independent of raising, anchored on actual
    keyboard/popup ownership. Four socket tests cover three-client wrap/reverse,
    refusal, unmap/disconnect/remap and held-key isolation. Configurable shortcut
    dispatch, rendered controls, application grouping and clipboard remain owed.
    Component checks and graphics evidence are recorded in
    `docs/autonomy/updates/stable-native-window-cycling.md`. Full independent
    supervisor gates and physical evidence remain owed; no release checkbox changes.

- [ ] The workspace client runs as an application on the shell

- [ ] **Image**: a **bootable container** (`bootc`) on a rented Fedora-derived
      base (ADR 0011), booting on the certified machine, firmware to sign-in.
      The base is rented deliberately: everything that makes alo OS *alo* is in
      layers we already own, and building our own base would buy no capability
      that is not already free
  - [x] **The code.** `image/` — a `bootc` image on the Fedora-derived base,
        pinned by digest and unpatched. It compiles `alo-agentd` and
        `alo-boundaryd` in a builder stage and installs them; it ships the two
        `systemd` units in the order ADR 0018 requires, ADR 0017's `/run/alo`
        entry and the folder the record goes in, the two logins the machine has,
        and a machine description. `crates/alo-image` is the gate on it —
        fourteen promises from ADRs 0001, 0004, 0015, 0017 and 0018 read back out
        of those files, each with a twin that breaks one line and is caught,
        because a build cannot see a description naming a login the image never
        creates. **This line was the one holding the other seventeen**, and
        building it measured two things nobody had: the base's kernel really does
        start the BPF LSM (`docs/hardware.md`), and the login number every
        example in this repository used belongs to `systemd-resolve` there
        (`docs/quirks.md`). Booting it measured a third, which is item 30: the
        agent service could not make either of the two places it needs — a
        control group for a turn, and the person's own door — because it holds
        no authority at all and nothing had been asked to make them for it. Its
        unit now says `Delegate=` and `RuntimeDirectory=`, three more promises
        are read back out of these files, and the daemon has been run from them
        under a real `systemd`
  - [ ] **On the machine.** That it boots, and that the daemon is running when it
        comes up — a virtual machine answers most of it and the certified machine
        answers the rest. **An image that builds is not an image that boots**, so
        a green build never ticks this, and the build that produced this image
        did not. Two of `docs/hardware.md`'s five kernel checks are still
        unanswered for the same reason: they are questions about a kernel that is
        running

**Exit gate.** On the certified machine, from a cold boot: sign in, press the
key, ask an agent to do something to a file in a granted folder, approve the
sentence, see it happen, and afterwards ask what it did and get an answer from
the record — with the egress indicator having stayed dark throughout.

---

## v0.5 — a person can work on it all day

Everything that turns a demonstration into a machine somebody uses on a Tuesday.

**Three lines here already have code**, which is why a tick appears in a list
that is otherwise untouched: *Making it yours* (`alo-appearance`), *Language*
(`alo-strings`) and *Run a model we never catalogued* (`alo-models`). The first
two were reached early because v0.01 work ran through them — appearance carries
the accent set, and every crate's English moved onto the strings layer. The
third was taken on its own, because a catalogue with nothing beside it had
quietly become the only way a model could reach the machine, and that is a
walled garden nobody decided on. Nothing else in v0.5 is started.

Unlike v0.01, this list is **not** ordered by what was built. It is a plan, and
it is grouped by subject so it can be read; when work begins here it will be
sorted the same way v0.01 now is.

- [ ] Lock screen, suspend and resume
- [ ] Multi-monitor, scaling, hotplug
- [ ] Recovery and rollback screen
- [ ] **Settings, as one place**: network, display, sound, printers, storage,
      keyboard, accounts, privacy, updates
- [ ] ★ **Run a model we never catalogued** — point alo OS at weights you
      already have and it runs them; the catalogue recommends and does not
      gate. What you bring is yours, including its licence, and a model too
      large for this machine's memory is said so plainly once and then run
      anyway. *This line was missing entirely until iteration 34 went looking
      for it — three v0.5 promises in `docs/features.md` with nowhere to be,
      which is the seventh time that has happened and the reason the rule at
      the top of this file runs both ways*
  - [x] **The code.**
        `alo-models` — `Weights`, a set somebody brought, beside `Model`
        rather than inside it: **no licence field at all**, because one
        saying *unknown* is read downstream as an answer and a machine
        showing it would be implying alo OS went and looked. `Cost`, which
        warns and has nothing to refuse with — two answers rather than
        three, measured against what the weights take on disk, because that
        is the floor the machine actually knows and a middle band would be
        us inventing a threshold about somebody else's hardware. And
        `Brought`, the list beside the catalogue, whose only filter is the
        measurement — the one thing still held back, because it is whether
        an agent turn works rather than what somebody is allowed to run.
        The cost cannot be shown without the line saying whose licence
        these are: `Weights::lines` is the only road to either. Since item
        25a the refusal on the line above **says this list exists**: a
        machine whose catalogue offers nothing for the agent names weights
        somebody already has, first among the answers, because it is the one
        that needs no other machine and the one alo OS never advertised
  - [ ] **On the machine.**
        the Settings panel that lists what the runtime already holds and
        lets somebody take one of them, and the choice between a catalogued
        model and one they brought — which is the same unbuilt setting the
        line below waits on. Nothing here has been pointed at real weights
        on a real machine

- [ ] **alo's own hosted model, and a subscription to it** (ADR 0014) — built
      as a provider like any other, with a test that proves it: our address is
      not privileged in `alo-egress`, and a policy refusing hosted inference
      refuses ours. The account and the billing live outside this repository;
      what the machine knows is an address, a key in the keyring, a region,
      and whether the last request was accepted
- [ ] **Making it yours**: background from a file, folder or colour and per
      display; lock-screen image; light and dark; an accent from the five
      designed hues, terracotta reserved (ADR 0010); text scaling; wallpapers
      shipped in the image
  - [x] **The code.**
        `alo-appearance` — background per display, light and dark, text
        scaling, and the accent set as working code: five hues, each value
        measured against the ground it is drawn on, terracotta unreachable
        rather than refused; and every word of it readable in the reader's
        own language rather than in English
  - [ ] **On the machine.**
        the Settings panel, the wallpapers themselves, and the mark and word
        that must appear wherever the agent's colour does
- [ ] **The ordinary desktop**: notifications, status area, file manager, trash,
      archives, USB storage, file associations, a text editor, an image viewer,
      a terminal
- [ ] **The plain way to do what the agent does** (ADR 0009) — searching your own
      files by name, kind, date and contents; a window showing what is running
      and what it is using; what is filling the disk. Each is the non-agent
      answer to a ★ line elsewhere in this file, and all three were missing until
      the rule was applied to `docs/features.md`. **No surface is left out
      because an agent can do it instead**
- [ ] **Capture**: screenshots, annotation, screen recording with audio, screen
      sharing — and an indicator whenever screen, camera or microphone is in use
- [ ] ★ **Divide the screen**: halves and quarters by drag or keyboard, splits
      that hold while you work and are remembered, per display
- [ ] **Input**: drag and drop, context menus, gestures, virtual desktops,
      keyboard layouts with dead keys and a compose key, input methods
- [ ] **Software**: install sandboxed applications, update and remove them; the
      XDG portal backend against our own shell; one grant list covering agents
      and applications alike; secret storage; session management; corporate
      proxy support
- [ ] **Devices**: audio with mid-call switching, Bluetooth, camera, microphone,
      media playback, power management, night light
- [ ] **Language**: the shell in all 24 official EU languages, with regional
      formats, timezones and a keyboard layout offered alongside each; RTL-ready
      even though no official EU language needs it yet
  - [x] **The code.**
        `alo-strings` — every sentence named, translation checked against
        what the system says, the 24 languages listed each in its own
        language, English unable to hide, and a sentence that counts
        something counted with its reader's own plural rules, read from CLDR
        rather than recalled; and six crates whose own English has moved
        onto it — `alo-files` (every file refusal, the six verbs, and the
        sentence a person approves before a file is renamed, moved or
        archived), `alo-shortcuts` (every row of the shortcuts panel, and
        every key named the way the reader's own keyboard prints it rather
        than the way an English one does), `alo-appearance` (the eleven
        colour names a person picks from, each with the note a translator
        needs where the word does not travel, and every refusal about a
        value they chose), `alo-capability` (every way the capability model
        says no — the grant that could not be made, the argument that did
        not survive the boundary, the change nobody was asked about, and the
        grants themselves, which now refuse with a value that is worded in
        the reader's language wherever it is shown or written down),
        `alo-models` (where an answer is about to come from, which is read
        before somebody decides whether to send a document at all; the rule
        an organisation set and why it refused; and every refusal about a
        provider, a key or the runtime) and `alo-egress` (the line a person
        reads while something is leaving their machine — the visible half of
        law 1 — the place it names, and every refusal the egress policy
        makes) — and, since 9g, **the sentence a person approves is one
        string rather than two renderings of it**: a verb is declared from
        the words a translator is handed, a call carries what names its
        sentence and the values that fill it, and the screen, the approval
        and the record all ask the reader's own vocabulary for the words.
        Since 11a that holds for what goes *into* the sentence as well: an
        option a verb offers is a word somebody translates rather than the
        identifier a model sent, and **a sentence is only as translated as
        its least translated piece**, so a finished sentence with an
        unfinished word in it cannot pass for a translated line. Since
        **15** that rule is true of every sentence this system composes
        rather than only of the one a person approves: the place inside the
        line law 1 shows while something is leaving, the grant inside a
        refusal, the key inside a shortcut, the colour inside a settings
        refusal, and another crate's whole refusal inside `alo-files`' —
        each of them answers for itself, at any depth, so a half-translated
        line says it is half translated wherever the half is. What stayed
        data stayed data: a path, a hostname, a window's identifier, a
        colour somebody typed and the fifty-three keys that print a mark are
        nobody's to translate and never count a line as unfinished.
        Since **21g** there is somewhere for another language to come
        from: `alo-saying` collects every crate's words into **one
        vocabulary for the machine** — not one per program, because a
        translation is checked against the vocabulary it is loaded into
        and a program that declared only its own strings would read a
        translator's correct line for another part of the system as a
        mistake — and loads the translations an image ships with, in
        `docs/contracts/translations.md`. Nothing about a translation can
        stop a machine: one that is missing, half written or from a later
        alo OS leaves it speaking English with what went wrong in the
        service log, because a machine that would not start could not say
        why. And a line that would come out wrong is left out rather than
        costing the language, so a string renamed in a release cannot turn
        somebody's language off in the release that renamed it.
        Since **24** holding the whole list is what makes one more promise
        checkable rather than a habit: **no name of anything alo OS rents
        reaches a person.** Ollama, Flatpak, Wayland, systemd, Podman and
        nine more are none of them things the person who bought the machine
        chose, and a test walks every sentence, every note a translator
        works from and every key against that list. It finds nothing today,
        which is the point — it costs nothing now and catches the first one
        later, on the day somebody mid-refusal writes what the log in front
        of them said
  - [ ] **On the machine.**
        a shell to translate, and every translation — there are still
        none, and now there is a file for the first one to arrive in
- [ ] ★ The agent answers in the language it was asked in
- [ ] **Access**: screen reader, magnifier, high contrast, keyboard-only
      operation of everything
- [ ] **Printing**
- [ ] `.docx`, `.xlsx`, `.pptx` open
- [ ] A web browser for the open web
- [ ] **Application adapters**, and the accessibility fallback for applications without one
- [ ] **System verbs** through the privileged broker
- [ ] **Guided fine-tune**, with the dataset never leaving the machine
- [ ] Full-disk encryption
- [ ] **Atomic updates with rollback** — largely **inherited** rather than
      built, since a bootc image rolls back with one command (ADR 0011). What
      is ours is the policy around it, and *undo what the agent did*, which is
      the one agent capability the base rather than our code provides
- [ ] Installer
- [ ] Accessibility: EN 301 549 conformance on the shell
- [ ] ★ **"Where is that file?"** — local retrieval over granted paths, nothing uploaded
- [ ] ★ **"Why is it slow?"** and **"what is filling my disk?"**
- [ ] ★ **Printers, solved** — found, set up, and fixed when they stop
- [ ] ★ **"I can't open this file"** — converted, or plainly explained
- [ ] ★ **The grant enforced by the kernel** (ADR 0013) — Landlock, seccomp and an
      eBPF programme on the turn's cgroup, so a verb outside its grant fails at
      the syscall rather than being refused by our own code, and the record
      becomes what the kernel watched rather than what the daemon reported.
      Linux's own extension points from userspace: no kernel written, none
      patched. **Ordered behind `alo-agentd` and the turn** — there is nothing
      to enforce until a turn exists, and this is written down now so the turn
      is built with a boundary rather than retrofitted into one
  - [ ] The code. **Proved, and not yet in front of a turn — which is why this
        stays empty.** Three crates as of 2026-09-04: `alo-bounding-kernel` is
        the BPF LSM programme on `file_open`, written in Rust through `aya` with
        the places one turn may reach per cgroup; `alo-bounding` loads it, reads
        the running kernel's own type information to find out where that kernel
        keeps its fields, and holds the map; `alo-bounding-map` is the seventy-two
        bytes both halves read the same way. Gated on both hosts and for the BPF
        target. **The tests
        that show a kernel refusing a file now run and pass**, on
        `6.18.33.2-microsoft-standard-WSL2`: a turn granted a folder opens a file
        inside it, the same turn reaching for a private key beside it is refused
        `EACCES` by the kernel, the turn ends and the key opens, and a process
        that is not a turn is untouched throughout. `docs/autonomy/QUEUE.md` item
        26 is ticked. **Item 26a is ticked too, and it answered the question with
        law 2 in it**: nothing is started to fill the cgroup, because a system
        that starts a program to do an agent's work is a system with a program to
        choose. One thread of the service goes in, by writing a byte into
        `cgroup.threads`, and comes back out through a descriptor opened before
        it went in — measured on the same kernel, with a sibling thread of the
        same process opening the file the working thread is being refused.
        **Item 26b is ticked, and it answered which grant becomes the bound**:
        the places *this execution* named, not everywhere the turn's grants
        reach, which is the narrower of the two candidates and is least
        privilege. An entry now holds up to four places and a count, the walk
        asks all of them at each step, and it is measured — a turn bounded to two
        folders opens a file in each while the private key beside them is still
        refused, and a folder the execution did not name is outside the bound
        even when the turn was granted it. **Item 26c is ticked, and it answered
        which places one execution reaches**: the paths a call named, resolved,
        plus the folder above anything it would create — so a rename, a move and
        an archive can be bounded at all, and the archive that does not exist yet
        is never itself a place. The widest of the six needs two of the four an
        entry holds, and the six are now held to that by a test in the crate that
        has the list rather than by an argument in a crate that cannot see them.
        The one case where the boundary is wider than the grant — a rename under
        a grant over a single file needs the folder that file sits in — is
        written down rather than designed away, and the grants stay the deciding
        answer. **Item 26d is ticked, and it is the one that put the mechanism in
        front of a turn**: `alo-turn` now takes a boundary the way it takes a
        record — one interface, no constructor without one, and no
        implementation in any library here except the daemon's real one — and
        `alo-agentd` holds it, imposes it before it binds its socket, and gives
        it back when it stops. The order a turn keeps is resolve, reach, bound,
        do, come out, write: the resolving is outside the boundary because a
        thread cannot look up what it may not yet open, and **the record is
        outside it** because a thread bounded across its own evidence would be
        refused the writing of it. A turn that cannot be bounded does not run,
        nothing is written down about it because nothing happened, and the person
        is told in their own language; a thread that cannot be brought back out
        is a service that stops. Measured through the daemon on
        `6.18.33.2-microsoft-standard-WSL2`: a real turn reads a file and writes
        an archive inside a boundary a real kernel is holding.
        **Item 27 is ticked, and it is the discipline rather than the
        mechanism.** ADR 0015's one dangerous property is that a programme on the
        security hooks is called for every open on the machine by construction,
        so the same thing that stops an agent reaching a private key could be a
        record of somebody's whole day, and only the discipline differs. *The LSM
        decides and forgets* is now counted rather than asserted: ordinary
        programs — this process, its other threads and a second process, none of
        them a turn — open five hundred files under the loaded programme, and
        afterwards it still has two maps and they are the two the daemon fills,
        the map of turns holds nothing, the spare slots of the other are still
        zero, and this kernel's trace buffer has not been written a line. The
        same is asserted at the moment the programme does its whole job and
        refuses an open inside a turn, which is the instant it would have
        something worth recording. Every measurement was checked against a
        programme that broke it: a third map is caught by name, and a
        `bpf_printk` on the hook by the count.
        **Item 26e is ticked, and it is the one that made this runnable on a
        machine anybody can boot.** 26d had left the daemon refusing to start
        wherever it could not get `CAP_BPF` — ADR 0015 implemented faithfully,
        and not yet a machine. ADR 0018 answers it the way ADR 0001 §2 already
        prescribed for privileged work: a fourth crate, `alo-boundaryd`, runs
        once at boot as root, loads the one programme there is, pins it, and
        finishes. It takes no path, no name and no argument that selects what to
        load, so it has no verb to get wrong. `alo-agentd` gains **no
        capabilities at all** and opens one pinned map by path — writing a grant
        is now permission on a file rather than authority over a kernel — and the
        map of field offsets is deliberately not given to it, so a daemon can
        bind a turn and cannot change how the kernel reads a file. **alo OS now
        has one privileged component where it had none**, which is a real loss
        stated rather than buried; what makes it the right trade is that the
        alternative was giving kernel-wide observation to the largest and most
        network-exposed process in the system. Measured on
        `6.18.33.2-microsoft-standard-WSL2`: the loader lets go of every
        descriptor it held and the kernel still refuses an open outside a bound.
        **What is left of this half is the second sentence of this line** — *the
        record becomes what the kernel watched rather than what the daemon
        reported* — which nothing has yet done, and until then the record is
        still the daemon's account of itself with a kernel underneath it
  - [ ] On the machine. The certified machine, and a kernel requirement that is
        a **configuration** and not a patch — now **five** checks rather than
        one, each invisible to the one before it: `CONFIG_BPF_LSM=y`, `bpf`
        among the security modules that actually start, a kernel whose
        RCU-tasks grace periods complete so a programme can be attached at all,
        `CONFIG_DEBUG_INFO_BTF=y` so the kernel will say where its own fields
        are, and since ADR 0018 a `bpf` filesystem mounted at `/sys/fs/bpf` for
        the boundary to be pinned in. `docs/hardware.md` says how to ask all
        five; the middle two are the ones machines fail. It also needs the two
        units in the right order — `alo-boundaryd` before `alo-agentd` — which is
        the image's, and is queue item 28. The machine that proved the mechanism
        is a development one, and this half is about the certified machine
- [ ] ★ **Undo what the agent did**
- [ ] Updates that never interrupt
- [ ] **Machines find each other** on a local network, with pairing
- [ ] **One GPU box serves the office** — shared local inference over a pairing.
      **Still egress, and the indicator still fires** (ADR 0003): the pairing is
      what makes it wanted, not what makes it silent
- [ ] A self-hosted workspace on the network is discovered, not configured
- [ ] **Zero inference egress over a working day**, measured and published —
      *with a local model*, which is the claim `docs/features.md` makes and the
      only one that is true. A machine using the office GPU box or a hosted
      provider has non-zero inference egress by design, shown on the indicator,
      and the published test says which machine it measured

**Exit gate.** A person works a full day on alo OS — mail, documents, a video
call, printing something, driving one installed application through its agent —
and does not need another machine. An update lands and rolls back cleanly. And
the four questions everybody asks a computer are answerable out loud: where is
that file, why is it slow, what is filling my disk, and undo what the agent just
did.

---

## v1 — an organisation can buy it

Not only a fifty-seat firm. v1 is the release a security team, a compliance
officer and a procurement department can all say yes to, which is a much wider
bar and is where most of this list comes from. **ADR 0004** settles what a
managed machine means and what its person is told.

- [ ] **Sign in with the organisation's identity provider** (SAML/OIDC)
- [ ] **Smartcard and national eID sign-in**
- [ ] **Disk-encryption key escrow**; remote lock and wipe
- [ ] **Records export to their SIEM**
- [ ] **Update rings**, and an update mirror they host
- [ ] **Private model catalogue** and **signed adapter allowlist**
- [ ] **Agent policy by role**; agent retention policy
- [ ] **Inference accounting**
- [ ] **Egress attestation** — signed, printable, per period
- [ ] ★ **"Make this machine like my old one"** — configuration as a document, for a person
- [ ] A new colleague working on day one
- [ ] Configuration as a document; helpdesk assistance as a session a person ends
- [ ] Certification groundwork: ISO 27001, BSI Grundschutz, ANSSI
- [ ] **Published EN 301 549 accessibility conformance report**
- [ ] Remaining portals: USB, application-registered shortcuts, remote desktop;
      location services off by default; applications contribute to search
- [ ] Fleet enrollment by discovery — the machine asks, an administrator admits it
- [ ] Fleet policy and signed updates, for alo OS machines
- [ ] Files and printers shared between paired machines
- [ ] Cross-machine agent work, under grants made on the target machine
- [ ] Signed images verified before boot; Secure Boot with our key
- [ ] Backup and restore
- [ ] **Adapter SDK published**, with a conformance suite
- [ ] Multi-user on one machine, with per-person grants
- [ ] ★ Ask for appearance changes — "use dark after six" — under the same
      propose-then-approve as anything else; themes as a document for a fleet
- [ ] **Third-party security audit** of `alo-agentd` and the broker, published
- [ ] Compatibility list, grown outward from the certified machine
- [ ] Support and SLA definitions

**Exit gate.** An organisation enrols a fleet against its own identity
provider, sets policy by role, receives a staged signed update, recovers a
machine whose owner has left, sees agent actions in its own SIEM, and hands its
auditor an egress attestation — with a published third-party audit and somebody
to call. And a person signing in to one of those machines can say, in ten
seconds, who else has power over it.

---

## Not scheduled

The engine (`alo-engine`) replaces the workspace's rendering incrementally and
has its own roadmap.

**alo OS Desktop** — the packaging for the Windows 10 fleet at scale — follows
once alo OS AI has customers, and inherits everything built here. It used to be
described as "the non-GPU SKU", which ADR 0007 makes wrong twice over: the CPU
is the *default* of the product being built now, not a stripped variant of it
arriving later, and a card is never what a person pays to get in. What Desktop
adds is breadth — a compatibility list, fleet packaging, machine generations —
and none of that is a different engine, a different agent or a different
promise.
