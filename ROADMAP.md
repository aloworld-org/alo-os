# alo OS — ROADMAP.md

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
1,128 passing tests sat underneath it — fourteen crates and 1,183 now. That is not honesty; it is a different
inaccuracy, and somebody reading it would conclude nothing had been built. Two
boxes tell the truth twice: what is finished, and what is still owed to a
machine nobody has yet plugged in.

**A code box is a claim, so its clause names the crate.** A clause that cannot
name one is decoration, and decoration here is how a roadmap starts lying
slowly.

---

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
- **Eight need the compositor or the certified machine. None of the eight is started.**

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
        actually run* is asked of the machine rather than assumed
  - [ ] **On the machine.**
        the setup screen that asks it on somebody's behalf, and a real
        measurement — *comfortable* is a judgement in a table until a model
        has run on a machine without a GPU

- [ ] ★ **Or use an API instead** (ADR 0008) — an answer may come from this
      machine, from a machine on your network, or from a provider you named, and
      the choice is the person's
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
        stopped from leaving — before the answer reaches anybody. 73 tests
        in `alo-asking` and 25 more around it, most of them against a stub on
        a real socket or a stub of the runtime trait
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
        was asked once, the place that was offered was asked nothing at all
  - [ ] **On the machine.**
        something that points, which is `alo-agentd`. *Something that asks*
        is no longer owed here at all: `alo-asking` puts a question to a
        hosted provider **and** to the model on this machine, and hands back
        a failure whose only door onward is an offer a person answered — so
        the fallback is carried by the code that would have had to contain
        it, in both directions. What is left of the machine half is the
        daemon that points at the local model, and a real runtime answering
        a real question, which needs Ollama installed

- [ ] **Add your own provider in Settings** — name, address, key to the keyring;
      the region stated rather than guessed; https required off this machine
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
        asked

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
        to all of that: five requests and no sixth, none of them able to
        carry a command, and two doors rather than one — so the side that
        proposes a change cannot be the side that approves it. What is not on
        the wire is as much of it as what is: no moment, no context, no turn
        and no place a question goes, because each of those would be a caller
        helping itself to something the machine is supposed to know
  - [ ] **On the machine.**
        the daemon itself — the socket, its peer credentials and a
        long-lived process (queue 21c), plus what it answers with (21b) — the
        acting half of the application verbs, which is Wayland and D-Bus and
        is the whole of what makes any of these move a window, and the half
        of the context that **reads** a screen, which is Wayland and AT-SPI
        and is where *with no invocation, no context calls at all* becomes
        something anybody can test

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
        not write one stops doing anything at all
  - [ ] **On the machine.**
        queue 20 — the path it is written to and the timer that shortens it,
        both `alo-agentd`'s, and the daemon does not exist

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
  - [x] **The code.**
        `alo-shortcuts` — the shortcuts, rebindable, nothing quietly taking
        one away, and every row and key of the panel said in the language
        the person reads
  - [ ] **On the machine.**
        a shell to press them in

### Everything that needs the compositor or the certified machine

Eight capabilities, and **not one of them is started**. That is not eight things
skipped over — it is the same dividing line read from the other side. The
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

- [ ] **Sign-in**: alo identity, and a local account that needs no tenant

- [ ] **The agent overlay**: one key, from anywhere

- [ ] Launcher and window management: move, resize, snap, tile

- [ ] Copy, cut and paste across applications; switching between windows

- [ ] The workspace client runs as an application on the shell

- [ ] **Image**: a **bootable container** (`bootc`) on a rented Fedora-derived
      base (ADR 0011), booting on the certified machine, firmware to sign-in.
      The base is rented deliberately: everything that makes alo OS *alo* is in
      layers we already own, and building our own base would buy no capability
      that is not already free

**Exit gate.** On the certified machine, from a cold boot: sign in, press the
key, ask an agent to do something to a file in a granted folder, approve the
sentence, see it happen, and afterwards ask what it did and get an answer from
the record — with the egress indicator having stayed dark throughout.

---

## v0.5 — a person can work on it all day

Everything that turns a demonstration into a machine somebody uses on a Tuesday.

**Two lines here already have code**, which is why a tick appears in a list that
is otherwise untouched: *Making it yours* (`alo-appearance`) and *Language*
(`alo-strings`). Both were reached early because v0.01 work ran through them —
appearance carries the accent set, and every crate's English moved onto the
strings layer. Nothing else in v0.5 is started.

Unlike v0.01, this list is **not** ordered by what was built. It is a plan, and
it is grouped by subject so it can be read; when work begins here it will be
sorted the same way v0.01 now is.

- [ ] Lock screen, suspend and resume
- [ ] Multi-monitor, scaling, hotplug
- [ ] Recovery and rollback screen
- [ ] **Settings, as one place**: network, display, sound, printers, storage,
      keyboard, accounts, privacy, updates
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
        nobody's to translate and never count a line as unfinished
  - [ ] **On the machine.**
        a shell to translate, and every translation — there are none yet
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
