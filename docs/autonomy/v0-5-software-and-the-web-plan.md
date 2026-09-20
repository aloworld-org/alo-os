# v0.5 — software and the web: installing applications, the browser, the proxy, and adapters

**Workstream:** four `ROADMAP.md` v0.5 lines — the half of *Software* that is not
portals or grants (*install sandboxed applications, update and remove them;
corporate proxy support*); *A web browser for the open web*; ★ *Application
adapters, and the accessibility fallback for applications without one*; and the
fresh-machine half of *The ordinary desktop* (*a text editor and an image viewer, so
a fresh machine is not helpless*, and *a terminal*). They belong together because
they are **the software a person did not write and alo OS did not write either**,
arriving on the machine and being reachable — by the person, and through adapters by
the agent.
**Why it exists:** written 2026-09-15 so that no v0.5 line is without a plan. The
applications plan decided what an application may be granted; this plan is how the
application gets there.

**Crates this plan owns, all new:** `crates/alo-software` (installing, updating and
removing a sandboxed application, from which sources, and the applications a fresh
machine ships), `crates/alo-proxy` (one machine-wide proxy, honoured by applications),
and `crates/alo-adapters` (loading an adapter against
`docs/contracts/app-adapters.md`, and the accessibility fallback) — and, from task
10, `crates/alo-declared` (the one list of the crates whose verbs alo OS ships,
held to this workspace's members), which belongs to no workstream and which every
plan that adds a verb-declaring crate registers in. **It reads and
never edits** `alo-portals`, `alo-granted`, `alo-applications` and `alo-secrets` (the
applications plan's — an installed application arrives with no grants), `alo-capability`
(an adapter's verbs are verbs; if registering them needs a change there, that is a
finding and a decision, not an edit), `alo-egress` (installing and updating are
errands that leave; the indicator fires), `alo-keeping-up` (the machine-keeps-itself
plan's — an application update is not a system update), `alo-access` (the access
plan's accessibility tree is the fallback's tree), `image/` and `alo-image` (the
installer plan's — the fresh-machine applications are named there by the installer
lane, with this plan's list), and `alo-saying`. **Nothing in `crates/alo-shell`.**

**What this plan may not do:** tick anything *on the machine*; build a package
manager, a sandbox, a browser engine or a terminal emulator (Flatpak, its portals, the
browser and the terminal are rented, configured and never patched — ADR 0005, ADR
0011, and `docs/features.md`: *a pinned upstream one, since our own engine is not
scheduled*); name any of them to a person (*they install an application — not a
Flatpak*); accept an adapter that takes model-written code (the contract's one rule);
or install anything unsandboxed (v1, and deliberate). Before writing the next task,
`git pull` and read the plan as published.

**Owner-authorized broker contribution, 2026-09-18:** the third PC owns this
software workstream and was authorized to clear its broker producer blockers.
Broker task 2 may register its printer verbs through these two existing shared
registration files. No other task or file is released; this plan retains its
remaining work and the supervisor's ownership checks remain unchanged.

```owner-release
plan = docs/autonomy/v0-5-the-broker-and-the-disk-plan.md
task = 2
files =
  crates/alo-declared/Cargo.toml
  crates/alo-declared/src/shipped.rs
```

## Tasks

### 1. Installing, updating and removing an application

**Status:** **Done, 2026-09-15.** Built in `crates/alo-software` on
`docs/decisions/0042-installing-an-application-is-an-errand-and-an-agent-only-proposes-it.md`;
the report is `docs/autonomy/updates/installing-updating-and-removing-an-application.md`.
`alo_egress::Errand` gained installing, checking for application updates and
updating (ADR 0042 part 1, the change named there); removing reaches no network
and puts nothing on the indicator (part 2); `install_application` is a change
needing no grant, with its reason (part 3). Reading an organisation's list of
permitted places out of the machine description is task 8. What is not yet
shown is the rented tool on a machine: no machine this task ran on has it, and
the report says what the on-machine acceptance is. **Depends on:** nothing.

- **Acceptance:** `alo-software` installs an application from the sources a person or
  an organisation enabled — Flathub, or a repository the organisation runs — through
  the rented tool, and **the application arrives with no grants**, held by a test that
  reads the one list after installing and finds nothing new; installing, updating and
  removing are errands that leave the machine and appear on the egress indicator
  (`alo-egress`), named as what they are; an update is **offered, never applied while
  the application runs**, and an application update never restarts the machine;
  removing an application ends its grants in the one list in the same act, held by a
  test; the signature of every source is verified by the rented tool and a source that
  fails verification is refused in words, never installed from; and an agent may
  propose installing an application through a verb a person approves, never install
  one by itself.
- **Constraint:** no package manager of ours. Sources are the rented tool's
  configuration; an organisation's source list is a bound in the machine's description
  (ADR 0016), never a person's setting it overrides silently.

### 2. What a fresh machine has, so it is not helpless

**Status:** **Done, 2026-09-15.** The decided list is
`crates/alo-software/shipped.toml`, read and held by `alo_software::Shipped` —
Firefox, Dolphin with Ark for archives, GNOME Text Editor, Loupe, Papers and
Ptyxis, each at its identifier, release and licence on the place a fresh machine
installs from — and the installer plan reads that same file
(`alo_software::shipped::WHERE_IT_IS`) rather than a copy in `image/`. Each goes
through task 1's two steps, so a person updates and removes any of them, the
browser included. The terminal is a person's own: on
`docs/decisions/0043-the-terminal-is-a-persons-and-never-an-agents.md` (accepted
2026-09-16, option C) this plan made **that ADR's additive change to
`alo-capability` and nothing else in that crate** — `persons_own.rs`,
`GrantError::APersonsOwn` with its word, and the three refusals — taken from the
held branch `held/software-task-2-awaits-capability` and rebased onto current
`main`. The rented tool on a machine is still outstanding, as it is for task 1;
the report says what that acceptance is. The report is
`docs/autonomy/updates/what-a-fresh-machine-has.md`. **Depends on:** 1.

- **Acceptance:** a decided list — a web browser, a file manager with trash and archives
  that open, a text editor, an image viewer, a document viewer, a terminal — each a
  **pinned upstream application**
  named by its source identifier and version, chosen for licence, maintenance and
  accessibility with the reasons in the report, installed the same way a person
  installs one (task 1) so that updating and removing it is no different; a person
  may remove any of them, including the browser; **the terminal is a person's and not
  an agent's** — a test holds that no verb reaches it (ADR 0001 §1: no arbitrary
  command); and the list is handed to the installer plan as data it reads, not
  edited into `image/` here.
- **Constraint:** no application is patched or rebranded. Which browser is a decision
  with its alternatives recorded in the report; it is not a partnership.

### 3. The web browser, and what it may take from the machine

**Status:** **Done, 2026-09-15.** Built in `crates/alo-software` and nowhere else:
`web_address.rs` is what a web address is, checked at the boundary — `http` and
`https` only, and a name-and-password in one refused with its own sentence;
`browsing.rs` is which application opens them (the person's choice, then the one
`Shipped` names for `Role::WebBrowser`, then nothing — and never a person's own
application, ADR 0043's reasoning about a value nobody here wrote);
`opening_the_web.rs` is an application's request judged in the open-with portal's
own order, asking the grants **before** reading this machine so an application
holding nothing learns nothing from its refusal; and `browser_settings.rs` with
`browser_configuration.rs` and `browser.json` are the shipped configuration —
three settings, telemetry and experiments off and *ask where each download goes*,
with the ten keys that would take the person's home page, search engine, proxy,
bookmarks or download folder refused by name. The decision is in this crate and
not in `alo-portals` because it reads `Shipped`, and a crate cannot read a crate
that reads it; wiring `OpenURI` to `alo_software::opened` is the applications
plan's, named in the report. What is not yet shown is the browser on a machine
reading that configuration, and whether the sandboxed build sees the path it is
put at — the report says so plainly rather than guessing. The report is
`docs/autonomy/updates/the-web-browser.md`. **Depends on:** 1, 2.

- **Acceptance:** the browser from task 2 opens web addresses from any application
  through the open-with portal; **it is a sandboxed application like any other**, with
  no grants beyond what a person gives it through the portals; its downloads go to the
  folder a person chose through the file chooser portal; the machine-wide proxy (task 4)
  reaches it; and nothing alo OS ships sets its home page, search engine or telemetry —
  a test reads the shipped configuration and finds the upstream's own defaults, with
  its telemetry **off** where the upstream allows a policy to turn it off, and the
  report names what could not be turned off.
- **Constraint:** no engine of our own and no fork. *No telemetry* (v0.01) is alo OS's
  promise; a rented browser's own behaviour is stated honestly, not claimed.

### 4. One proxy, machine-wide, honoured

**Status:** **Done, 2026-09-15.** Built in the new `crates/alo-proxy`, which decides
and reaches nothing: `setting.rs` is the one setting in its three shapes, `kept.rs`
is whose it is (ADR 0016, with a person refused in words on a machine an
organisation manages), `road.rs` is the closed list of every road out and
`deciding.rs` the one door each takes, `carried.rs` is what a road is then given —
variables for a program, a proxy for a request, and the one function in the crate
that turns a password into text — `published.rs` and `portal.rs` are the two ways an
application already honours a proxy, and `automatic.rs` with `evaluator.rs` is a
network's own configuration, fetched and evaluated by a **separate program with a
cleared environment**, with a machine that has none **refusing** rather than going
straight out. A password is a `WhereThePasswordIs` in the setting and never a
credential (ADR 0022). The three roads named in the acceptance each ask it in their
own crate — `alo-software`'s rented tool, `alo-updating`'s base, and `alo-models`'
`Trying::taking`, which now says *straight out* explicitly so no process environment
can point alo OS's own road anywhere. What is not here is reading an organisation's
proxy out of `/etc/alo/agentd.toml`: that file is `alo-agentd`'s, exactly as
`alo_software::Bound`'s is, and the report says what that section would be. The
report is `docs/autonomy/updates/one-proxy-machine-wide.md`. **Depends on:** nothing.

*A great many company networks have no other route out.*

- **Acceptance:** `alo-proxy` holds one proxy setting — none, a manual address per
  scheme with exceptions, or an automatic configuration address — kept machine-wide as
  a bound an organisation sets or a person sets on a personal machine (ADR 0016); it
  reaches **every road out alo OS itself uses** — installing, updates, providers — held
  by a test per road, and is published to applications through the environment and the
  proxy portal they already read; a proxy's password goes to the keyring (ADR 0022),
  never to a file; **the egress indicator still names the real destination**, not the
  proxy, because *it went to the proxy* is not what a person needs to know; and an
  automatic configuration script is fetched and evaluated **without running it in any
  process that holds a grant**.
- **Constraint:** no proxy server of ours. A policy that cannot be evaluated refuses,
  as `alo-egress` already does.

### 5. An adapter, loaded against the contract

**Status:** **Done, 2026-09-16.** Built in the new `crates/alo-adapters`. An
adapter is declared data (`Adapter`, written as Rust constants because a verb's
words are `alo_strings::Word`s); `load` refuses a verb taking a script, a command
or free text, a parameter the application interprets, a method named for running
something, an action an argument chooses, screenshots and synthetic input, a verb
with no by-hand road, a path outside a grant, and an adapter for a person's own
application (ADR 0043), and then declares what passes through
`alo_capability::Verb::checked`. **Registering adapter verbs needed no change to
`alo-capability`**: a verb is named `adapter.verb`, its grant is over its path
arguments, and the grant over its application is asked in `Driving::of` before
anything is sent. The reference adapter is GNOME Text Editor (`text_editor.open_document`,
`text_editor.new_window`) through `org.freedesktop.Application`, its own interface,
shown end to end on a real private bus against a service holding its name; the
application itself on a machine is still outstanding, and the report says what
that acceptance is. `api` and `accessibility` are declared and refused until this
machine carries them out. The report is
`docs/autonomy/updates/an-adapter-loaded-against-the-contract.md`. **Depends on:** 1.

★ *Installed applications become agents with typed verbs (`@blender`, `@resolve`,
`@gimp`).*

- **Acceptance:** `alo-adapters` loads an adapter as declared data — its application,
  its typed verbs with validated arguments, and its mechanism (automation API,
  accessibility tree, D-Bus) — **and refuses any adapter whose verb takes a script, a
  command or free text that becomes code**, with a test that submits one and finds it
  refused; each adapter verb is approved and recorded like any verb (ADR 0001) and has a
  by-hand road (ADR 0009) or is refused for lacking one; one reference adapter for an
  application task 2 ships is built end to end as the proof, with the application's
  own automation interface; and **if registering adapter verbs needs `alo-capability`
  to change, the task stops at a decision record** naming the change and who makes it,
  the way ADR 0040 did.
- **Constraint:** the contract is additive and published; nothing here changes it
  without versioning. No adapter uses screenshots and synthetic input (v1, last resort,
  policy-disabled).

### 6. The accessibility fallback, for applications without an adapter

**Status:** **Done, 2026-09-16.** Built in `crates/alo-adapters`, beside the
adapters and not as one: `accessible.read_window` (a read over a granted
application) and `accessible.activate_control` (a change naming a kind of control
from a closed list of seven, the name it shows and the application, approved as
*press the button named “Sign in” in org.example.Mail*). Both reach only a granted,
installed application **with no adapter of its own** (`fallback_reach.rs`), identify
an application by its sandbox through `alo_portals::Sandboxes` and never by the name
it gives itself, and walk its windows at that moment and keep nothing
(`walking.rs`); a press finds its control again then, and refuses none, more than
one, a window too large to be sure, a greyed-out control and one that cannot be
pressed, in words. **A password field is never asked for its text** — held against
GTK 3's own `gtk-builder-tool` on at-spi2's own bus, read off a monitor of that bus —
and nothing is ever asked about position (`AccessibilityTree` has no such
question). The read is a read, as ADR 0001 §5 makes every read; *approved like any
change* binds the press, and the report says why. What is not shown is a GTK 4 or Qt
application on a certified machine's Wayland session; the report says what that
acceptance is. The report is `docs/autonomy/updates/the-accessibility-fallback.md`.
**Depends on:** 5.

- **Acceptance:** an application with no adapter is readable and operable through its
  accessibility tree — the agent may **read** what a window shows and **activate** a
  control by its role and name, through two typed verbs, each proposed as a sentence
  that names the control and the application and approved like any change; the tree is
  read at invocation for that turn only, never watched; a password field's contents are
  **never** read, held by a test against a real application's password field; and what
  the fallback cannot do — a control with no name, a canvas — is said in words, never
  guessed at with coordinates.
- **Constraint:** no screenshot, no synthetic pointer event, no coordinates. The tree
  is the rented AT-SPI's.

### 7. Every sentence, and the walk from nothing to a working application

**Status:** **Done, 2026-09-16.** Held in `crates/alo-software/tests`, the one
crate that already reaches the other two:
`every_sentence_of_software_and_the_web.rs` asks every word `alo-software`,
`alo-proxy` and `alo-adapters` declare to be in the machine's collected
vocabulary with its note, on one line, naming no packaging, bus, accessibility
interface or browser engine; `from_nothing_to_a_working_application.rs` walks
set a proxy → install the text editor → open a web link → the agent opens a
document through the reference adapter → remove it, and a second walk stopped at
each step, each against its table, with every row's key measured rather than
written down. Reading them fixed four things: two proxy sentences carrying a line
break, the automatic configuration's address and the places reached directly
refused only in English (`NotAConfiguration` and `NotAnException` now have words
and no `Display`), a managed machine told to set its proxy by hand, and removal
not saying that permission to use the application ended. What
`capability.refused.never-granted` says about an application's grant is named for
that crate's owner. The report, with both tables, is
`docs/autonomy/updates/every-sentence-and-the-walk-to-a-working-application.md`.
**Depends on:** 1, 2, 3, 4, 5, 6.

- **Acceptance:** every sentence these crates can say is in the vocabulary with a
  translator's note; one walk — set a proxy, install an application, open a web link
  in the browser, ask the agent to use the reference adapter, remove the application —
  produces the exact sequence a person meets, recorded as a table and held by one test;
  no sentence names Flatpak, Flathub by its tooling, AT-SPI, D-Bus or a browser engine.
- **Constraint:** nothing here re-decides what the sentences describe.

### 8. An organisation's permitted places, read from the machine's description

**Status:** **Done, 2026-09-16.** `/etc/alo/agentd.toml` gained an optional
`[applications]` section with one key, `may-come-from`, a list of place names;
it requires `format = 3` (a `1` or `2` carrying it is refused, as a `1` carrying
`[questions]` is), and `1`, `2` and `3` without it are the same machine. Read in
`crates/alo-agentd/src/permitted_places.rs` into `alo_software::Bound`, with who
set it decided by who owns the file, and handed out as
`alo_agentd::Described::applications`; absence is `Bound::Nobodys`. A name that
could never be a place, a place named twice, a missing key or an unknown one stops
the service. Nothing in `alo-software` was edited. The acceptance test is
`crates/alo-agentd/src/installing_under_the_description.rs`; the contract is
`docs/contracts/machine-description.md`; the report is
`docs/autonomy/updates/an-organisations-permitted-places-from-the-machine-description.md`.
What is not here is the surface that installs — the shell's — handing this value
to `alo_software::Enabled::read`; nothing on a machine installs yet. **Depends on:** 1.

*The organisation bounds; the person chooses* (ADR 0016), for where applications come from.

- **Acceptance:** `/etc/alo/agentd.toml` carries an optional section naming the places an
  organisation permits applications to come from, documented in
  `docs/contracts/machine-description.md` with the `format` rule `[questions]` follows (an
  older service must refuse a description whose software bound it cannot enforce, never
  ignore it); `alo-agentd` reads it into `alo_software::Bound` with who set it decided by
  who owns the file, exactly as `[questions]`; its absence is `Bound::Nobodys` and never
  a permissive list; and a test installs from a place the file does not name and finds it
  refused in the words naming the organisation, beside the same install on a machine with
  no section.
- **Constraint:** nothing here edits what `alo-software` decides; a section that does not
  hold stops the service, as `[questions]` does.

### 9. An organisation's proxy, read from the machine's description

**Status:** **Done, 2026-09-18.** `/etc/alo/agentd.toml` gained an optional
`[proxy]` section in the shape `[applications]` established: it requires
`format = 4` (a `1`, `2` or `3` carrying it is refused, as a `2` carrying
`[applications]` is), and `1`, `2`, `3` and `4` without it are the same machine.
Which way out is **written rather than worked out** — `goes-through` is
`"nothing"`, `"an-address"` or `"a-configuration"` — because a section whose
meaning depended on which line somebody had commented out is a section where
deleting a line moves the machine's traffic; a key the way out would ignore is
refused, as a `region` beside the wrong `may-go` is. Read in
`crates/alo-agentd/src/machine_wide_proxy.rs` into `alo_proxy::Kept`, with who
set it decided by who owns the file; **absence is `Option::None`, nobody having
set one**, and never `TheProxy::None` written on a person's behalf. The section
names the keyring entry the password is kept under (ADR 0022), and both ways a
password would reach `/etc` — a `password` key declared in order to be refused,
and a credential pasted into an address or a name — are refused by name in
sentences that never repeat what was written. Nothing in `alo-proxy` was edited:
this crate's own spelling goes through that crate's constructors, which is why a
proxy address in a description is a checked host and not a `Deserialize` derive.
The acceptance test is
`crates/alo-agentd/src/changing_the_proxy_under_the_description.rs`; the contract
is `docs/contracts/machine-description.md`; the report is
`docs/autonomy/updates/an-organisations-proxy-from-the-machine-description.md`.
What is not here is the daemon **handing** that setting to the roads a turn
takes — `alo_proxy::the_way` answers it in this task's own test, and no caller in
`alo-agentd` carries it to `alo-asking` yet; the report says so and names what
that would be. **Depends on:** 4, 8.

*A great many company networks have no other route out* (task 4), and on a managed
machine the organisation is who knows the route.

- **Acceptance:** `/etc/alo/agentd.toml` carries an optional section stating the
  machine-wide proxy an organisation sets — none, a manual address per scheme with
  exceptions, or an automatic configuration address, in `alo_proxy::Setting`'s three
  shapes — documented in `docs/contracts/machine-description.md` beside
  `[questions]` and `[applications]`, with the `format` rule both follow (a new shape
  number; an older service refuses a description whose proxy it cannot honour, and an
  older shape carrying the section is refused); `alo-agentd` reads it into
  `alo_proxy::Kept` with who set it decided by who owns the file, exactly as task 8;
  its absence is a proxy nobody set, never "straight out" written on a person's
  behalf; **a proxy password is never in the file** — the section names where the
  password is in the keyring (ADR 0022), and a password written inline is refused by
  name; and a test shows a person's own proxy refused in words on a machine whose
  root-owned description sets one, beside the same change accepted on a machine with
  no section.
- **Constraint:** nothing here edits what `alo-proxy` decides; a section that does not
  hold stops the service, as `[questions]` and `[applications]` do.

### 10. A list of crates that cannot drift from the workspace it describes

**Status:** **Done, 2026-09-18.** Recovered task 10; the one list is the new `crates/alo-declared`:
`WHO_DECLARES_THEM` and the `declare_into` calls in one file
(`src/shipped.rs`), so neither half can move without the other, and
`alo_declared::held` holds that list to `Cargo.toml`'s members through
`alo_by_hand::whoever_declares_verbs` — naming the crate and naming
`crates/alo-declared/src/shipped.rs` when one is missing. `alo-by-hand`'s test
and `alo-software`'s terminal test are both handed it and keep no copy;
`alo-saying`'s `EVERY_LIST` and `ONE_STRING_EACH` are slices with no length
written on them. Nothing got weaker: `alo-by-hand`'s `AVerbListNobodyHandedIn`
still runs against the real workspace inside its own check, and the crates it
covered are the crates it covers. `tools/kernel-loop`'s `REGISTRATIONS` names the
new file, so a plan adding a verb-declaring crate is not refused for registering
it. What the report says plainly is that the list does not write itself and
cannot: a test cannot call a crate it does not depend on, so the calls are Cargo
dependencies. The registry derives names and calls from one entry per crate;
the check derives their agreement with the workspace. The report
is `docs/autonomy/updates/one-list-of-crates-that-cannot-drift.md`.
**Depends on:** nothing.

**Why, 2026-09-17.** Three times in one day a lane was refused for a change it
never made, each time by a hand-kept list that has to move in step with the
workspace and has nothing tying it there:
- `alo-software`'s `the_terminal_is_a_persons_and_no_verb_reaches_it` spells out
  the crates that declare verbs. It broke when `alo-converting` was added, and
  again when `alo-capturing` was (f9054b0).
- `alo-by-hand`'s `every_verb_can_be_done_by_hand` keeps `WHO_DECLARES_THEM` and
  its own copy of the calls. It broke for `alo-capturing` too (38ccba8).
- `alo-saying`'s `EVERY_LIST` and `ONE_STRING_EACH` are arrays with a written
  length. Two lanes each adding a crate merge cleanly and then fail to compile,
  five times on 2026-09-16 alone.
Each broke every lane on every machine until somebody fixed a list their task
never touched.

This plan owns `alo-software`, so it takes the fix. It crosses into
`alo-by-hand`'s test and `alo-saying`'s `collecting.rs`. No plan owns either, and
every plan that adds a crate already edits both. The change there is only this
one.

- **Acceptance:**
  - **No length is written anywhere:** `EVERY_LIST` and `ONE_STRING_EACH` are
    slices, so two parallel additions merge and compile.
  - **One list of verb-declaring crates:** there is **one** list of the crates
    whose verbs alo OS ships, with the calls that declare them, in one place both
    tests use. A test holds that list to what the workspace says:
    `alo_by_hand::whoever_declares_verbs` over `Cargo.toml`'s members.
  - **A missing crate is named, with where to add it:** when a crate is missing,
    the failure names it and names the one file to add it in.
  - **Only the check reads the workspace:** the words and the verbs still reach a
    test through real Cargo dependencies, because a test cannot call a crate it
    does not depend on. So the crates cannot be derived at build time, and only
    their agreement with the workspace is. The report says so plainly rather than
    claiming a list that writes itself.
  - **Proof it holds:** a test adds a crate that declares verbs to a fixture
    workspace, and finds the check naming it and the file.
- **Constraint:** no test gets weaker. Every crate that is checked today is still
  checked, and a verb or a word that escaped the list is still a failure, just in
  one place.
- **Not this task:** the release number written in six places. It belongs to the
  installer plan's `image/` and `alo-image`, and is left to that plan.

### 11. The proxy a machine was told about, carried to the question a turn puts

**Status:** **Done, 2026-09-20.** The setting now reaches the socket.
`crates/alo-agentd/src/the_road_out.rs` is the one place that asks —
`alo_proxy::the_way` for `Road::AskingAProvider`, over the `alo_proxy::Kept`
`Described::proxy` answers with — and it decides nothing itself; `starting.rs`
hands it over, `questions.rs` carries it as it already carries `[questions]`'s
bound, and `doing.rs` asks it **after the organisation's rule and before the
keyring**, so a question with no road to go on never reaches for a credential.
`alo_asking::Hosted::taking` takes the answer as an `alo_proxy::Carried`, and
`Hosted::where_it_would_connect` then names **the proxy** — which is what ADR
0020 registers with the boundary, because that is where the socket really opens.
Two findings are in the report and worth the line here: the provider road was
**inheriting `HTTP_PROXY` from whatever process the daemon was in**
(`ureq::Config::default` reads it), which is now said explicitly on every
request and held by a test that runs the question in a second process with those
variables set; and a test that watched addresses alone could not have caught
either half, because a road out is handed its addresses and resolves nothing, so
the acceptance reads the first line on the wire. Carrying the proxy needed
`alo-asking`'s hosted door — the models-measured plan's, **finished 2026-09-15**
— so this plan took it under *a machine unblocks itself*
(`docs/autonomy/a-new-machine-becomes-a-lane.md`) rather than stopping at a
decision record; that file records the transfer. The acceptance test is
`crates/alo-agentd/src/the_proxy_on_the_road_a_question_takes.rs`; the report is
`docs/autonomy/updates/the-proxy-on-the-road-a-question-takes.md`. What is not
here is a proxy that asks who you are: no road in this workspace reads the
password `[proxy]` names, and that is task 12. **Depends on:** 4, 9.

*Machine-wide, and honoured* (`docs/features.md`, v0.5) — a proxy that is read
and not taken is neither.

- **Acceptance:** the proxy `alo_agentd::Described::proxy` answers with reaches
  the road a turn's question takes, decided by `alo_proxy::the_way` for
  `Road::AskingAProvider` and carried by `alo_proxy::Carried`, with a test that
  puts a question on a machine whose description states a proxy and finds the
  request configured with it — and the same question on a machine whose
  description states none going straight out, **said explicitly** so that no
  process environment can point alo OS's own road anywhere
  (`alo_models::Trying::taking`'s rule, asked of this road); a machine told to
  ask an automatic configuration and unable to **refuses the question** rather
  than asking the provider directly, in the words `alo-proxy` already has; the
  **egress indicator still names the provider**, never the proxy, held by a test
  that reads the line; and a question answered by a runtime on this machine is
  never sent through a proxy at all, which is `alo_proxy::the_way`'s answer about
  this machine and not a second rule here.
- **Constraint:** nothing here re-decides which way a road goes — `alo-proxy`
  decides, and this is the crate that asks it. If carrying a proxy to that road
  needs `alo-asking` to change, and that crate turns out to be another plan's,
  **the task stops at a decision record** naming the change and who makes it, the
  way task 5 was told to about `alo-capability`.

### 12. A proxy that asks who you are, signed in to on every road

**Status:** **Done, 2026-09-20.** The decision came first and is
`docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md` (accepted
2026-09-20): a machine-wide proxy password lives in **the machine's own
credentials**, provisioned to each unit by systemd and read at
`/run/credentials/<unit>/<entry>`, where the entry is the name
`password-in-keyring` already gives — **not**
ADR 0022's store, because that is a *provider's* key on the person's session bus
and two of the four roads run before anybody has signed in. `alo_proxy::signed_in`
(`crates/alo-proxy/src/signing_in.rs`) is the one door a credential travels
through, and `alo_proxy::TheMachinesPasswords`
(`crates/alo-proxy/src/provisioned.rs`) is the reader, which believes nothing it
finds: it checks the name is one thing to look up, that what is there is a
regular file, that nobody but its owner may read it, and that what it holds is a
password. Seven states are told apart under three sentences — what an
administrator *does* about them is three things — and on every one of them the
road is **not taken**: not to the proxy without the credential it asked for, and
not around the proxy either. The unit is **named, never worked out**
(`TheRoadOut::THE_UNIT`, `alo_looking_once::road::THE_UNIT`), so
`$CREDENTIALS_DIRECTORY` is not read and there is no code path that could —
`alo_secrets::TheBus::of`'s rule. Held by a test per road, each with its
refusals beside it, and two of them read the credential **off the wire**:
`alo-agentd`'s and `alo-models`' watch a socket and find
`Proxy-Authorization: Basic …` in the `CONNECT` this machine sends. A test in
`signing_in.rs` reads the workspace and finds `Carried::with_the_password` has
one caller that ships. What is **not** here, and is named rather than implied:
the `LoadCredentialEncrypted=` lines in `image/`'s unit files, which are the
installer lane's as every line there is — until they land, a machine whose proxy
asks for a name is refused *by alo OS, in words* rather than by the proxy, which
is better than what it did and is not the finish; and a door for a person on
their own machine to write one, which is task 13. The report is
`docs/autonomy/updates/signing-in-to-a-proxy-that-asks-who-you-are.md`.
**Depends on:** 4, 11.

**Originally written 2026-09-20 by task 11, which found it:** `[proxy]`
names `sign-in-as` and `password-in-keyring`, `alo_proxy::ProxyAddress` carries
both, and `alo_proxy::Carried::with_the_password` is the one door a proxy
credential becomes text through — and **no road in this workspace calls it**.
Not the rented tool's (`alo-software`), not the base's (`alo-updating`), not a
provider's list (`alo-models`), and not a turn's question (task 11). Every one
of them builds its `Carried` from `the_way` alone, so a machine whose company
proxy wants a name reaches it as somebody with no password and is refused by the
proxy — on a network task 4's own header says has no other route out. Today that
refusal is at least a sentence somebody can act on, which is why task 11 left it
rather than taking the decision underneath it quietly for four roads at once.

*A great many company networks have no other route out* — and a good many of
those proxies ask who you are.

**The decision this waited on, and it was small. Taken, as ADR 0059.**
ADR 0022 settles where a
**provider's** key lives: the Secret Service, over the person's own session bus
at `/run/user/<uid>/bus`, derived from the daemon's uid. A machine-wide proxy
password is not that, and the difference is not a detail — `alo-agentd` reads
`/etc/alo/agentd.toml` as root before anybody has signed in, and the rented tool
and the base take their roads with no session at all. So *which store*, *whose*,
and *what a road does when nobody has signed in yet* were the three questions,
and this task wrote the ADR rather than answering them in a crate. Its answers,
in order: the machine's own credentials; the machine's, set by whoever
administers it (ADR 0016's split, unchanged); and **it takes the road**, because
a credential delivered to a unit at its start does not wait for a person — which
is the property the store was chosen for.

- **Acceptance:** a proxy an organisation's description says wants a name is
  signed in to on **every road out alo OS itself uses** — installing and
  application updates, the system's own update, a provider's list and a turn's
  question — through `alo_proxy::Carried::with_the_password` and through nothing
  else, held by a test per road, so the one function that turns a password into
  text stays the one function `grep` finds every caller of; the password is
  read from where ADR 0022's successor says it lives and **never from the
  description** (`crate::machine_wide_proxy` already refuses one written there,
  and that refusal does not get weaker); a machine that cannot read it
  **refuses the road in words** rather than reaching the proxy as somebody with
  no password or going straight out around it, with the refusal naming what to
  do and quoting nothing that was stored; and no record, no service log, no
  indicator line and no `Debug` anywhere carries the password, held by a test
  that formats everything on the road and finds it absent.
- **Constraint:** the decision above is written first, as an ADR under
  `docs/decisions/`, and it is this task's first deliverable — a store chosen
  inside a crate would be a store four roads then have to agree with. Nothing
  here widens what `[proxy]` may contain: a password in the file stays refused.

### 13. A person's own proxy password, set on the machine that is theirs

**Status:** **Done, 2026-09-20.** The decision came first and is
`docs/decisions/0060-a-persons-own-proxy-password-is-set-with-the-proxy-in-one-act.md`
(accepted 2026-09-20), because the broker's own door
(`crates/alo-broker`) is the broker-and-disk plan's and still has unfinished
tasks — so the answer had to be *no twelfth verb*, which is also the better
answer: `network.set-proxy` carries the proxy **and the password it signs in
with**, one act under one approval, the credential written **first** and the
machine's proxy file only if that succeeded. A machine where the write fails is
therefore left exactly as it was rather than holding a proxy it cannot sign in
to, which two verbs could not have promised without a transaction across two
approvals (ADR 0001 §5). The useful corollary is that the proxy file naming
where its password is kept **is** the machine's statement that one is set, so a
settings panel needs no second place to look and nothing reads a credential
back. `alo_proxy::TheMachinesCredentials` (`crates/alo-proxy/src/provisioning.rs`)
is the writer beside task 12's reader — `systemd-creds`, started directly with a
cleared environment and the password on its **standard input**, written beside
its place, `0600`, and renamed over. Two decisions are worth the line here:
alo OS reserves **one** name, `alo_proxy::THE_PERSONS_PROXY_PASSWORD`
(`the-proxy-on-this-machine`), because `LoadCredentialEncrypted=` is a static
line in a unit file and a name a person typed would reach no unit — and the
broker refuses every other name, which is also what stops a root process writing
a file somebody else named into the credential store; and the bytes handed over
carry **thirty-two bytes of the kernel's randomness** in front of the password,
because the identity that crosses the door is written into the broker's record
and a digest of a password is a password. A person on a machine an organisation
manages is refused **before anything is handed over**, in
`alo_proxy::NotChanged::AnOrganisationSetIt`'s existing sentence, so the password
they typed never leaves the process they typed it into. What is not here, and is
named rather than implied: the `LoadCredentialEncrypted=` lines in `image/`,
which are the installer lane's — ADR 0060 §2 fixes the name so they can be
written once; and the settings panel itself, which is `alo-shell`'s and which
this plan may not touch. The report is
`docs/autonomy/updates/a-persons-own-proxy-password.md`. **Depends on:** 12.

**Originally written 2026-09-20 by task 12, which found it:** ADR 0059
settled where a machine-wide proxy password lives — the machine's own
credentials, provisioned to a unit — and settled it for the case that pays for
it: an organisation writes the credential with the machine, alongside the
`[proxy]` section in the description. **On a personal machine there is nobody
else to write it.** ADR 0016 says the person is their own machine's
organisation, and ADR 0049 §3 says the proxy is set by a person through the
broker rather than by any agent — so a person on their own machine may already
set a proxy in Settings and, today, may not give it the password it asks for
without opening a terminal and running `systemd-creds` by hand. That is a
settings panel with a field that works and a field beside it that does not.

*Machine-wide, and honoured* (`docs/features.md`, v0.5) — on the person's own
machine as much as on a company's.

- **Acceptance:** a person on a machine they own may give their proxy the
  password it asks for, from the same place they set the proxy — carried to the
  broker **by identity and never as text**, in the bytes-and-digest shape
  `alo_networks::proxy_file` already uses for the proxy itself (ADR 0001 §2,
  ADR 0049 §1), so nothing free-form crosses the broker's door; the broker
  writes it where ADR 0059 says it lives and **nowhere else**, through the tool
  the base already has rather than an encryption of ours (ADR 0011), and the
  bytes the person handed over are removed in the same act; a person on a
  machine **an organisation manages is refused in words** naming who can change
  it, which is `alo_proxy::NotChanged`'s existing sentence and not a second one;
  the password never reaches a file the person's own programs can read, never
  reaches the record, the journal or any `Debug`, and is never read back out —
  a settings panel shows *a password is set* and offers to replace it, held by a
  test; **no verb reaches any of this** (ADR 0049 §3: no agent proposes a proxy
  in v0.5), held by a test; and a machine where the write fails says so and
  changes nothing, rather than leaving a proxy set that cannot be signed in to.
- **Constraint:** no encryption of ours, no second store, and no credential-
  transfer protocol — ADR 0022's approval excluded one and ADR 0059 did not add
  one. If carrying this to the broker needs a change to the broker's own door,
  and that door turns out to be another plan's, **the task stops at a decision
  record** naming the change and who makes it, the way task 5 was told to about
  `alo-capability`.
- **Not this task:** the `LoadCredentialEncrypted=` lines in `image/`'s unit
  files. ADR 0059 names them exactly and they are the installer lane's, as every
  line in `image/` is; this plan hands them over as data rather than editing
  them, the way task 2 handed over `shipped.toml`.

### 14. The proxy a person set, read from the machine's file by the roads that install

**Status:** **Done, 2026-09-20.** Written 2026-09-20 by task 13, which found it — and the
contract has named it since the network task: *no road out reads this file yet;
`alo-software`, `alo-updating` and `alo-models` are handed a `TheProxy` by their
callers* (`docs/contracts/machine-proxy-file.md`, **What is not here**). Task 4
decided the setting, task 11 carried it to a turn's question and task 13 gave it
a password a person can set — and **the two roads this plan owns still take
whichever proxy their caller happens to pass them.**
`alo_software::TheRentedTool::taking` is handed an `alo_proxy::Carried`, and
nothing in the workspace builds one for it from `/etc/alo-proxy/proxy.json`. On
a company network that is a machine whose person set a proxy in Settings and
whose applications still install straight out — or do not install at all, which
is the same bug read from the other end. `alo-looking-once` already reads that
file (`crates/alo-looking-once/src/road.rs`) and is the shape to follow.
**Depends on:** 4, 12.

**Done, 2026-09-20.** Report:
[The proxy a person set, on the road that installs](updates/the-proxy-a-person-set-on-the-road-that-installs.md).
`crates/alo-software/src/road.rs` reads the machine's own file through
`alo_networks::proxy_file::kept_on_this_machine` — the same one reader and the
same one path `alo-looking-once` uses, and no second spelling of either — asks
`alo_proxy::the_way` for the road, and signs in through `alo_proxy::signed_in`
with the password the machine gave the unit that carries the errand out.
`the_tool_for` hands back a `TheRentedTool` already carrying it, which is what
closes the gap: a caller asks by errand rather than by proxy, and cannot pass
the wrong one.

**Three states of the machine, each measured.** A file naming a proxy is taken
on all three roads; **no file at all goes straight out**; and a file that is
there and is **not one this machine wrote is refused** — never read as *no
proxy*, which on a company network is the difference between *this did not work*
and *this left the building without permission*. Only one of those can be
undone.

**And the half that earns the task**: an errand on a machine whose file names a
proxy produces a tool whose **started program really receives it**, read out of
a real child process rather than out of a list — including the credential, for a
proxy that asks who this machine is. A proxy that is set and cannot be worked
out **refuses rather than falling back to straight out**, and there is no branch
that could do otherwise. The egress indicator still names where the errand is
really going and never the proxy, held by a test that reads the line.

**What this leaves owed**, named the way the network task named this one:
`alo-updating` and `alo-models` are still handed a `TheProxy` by their callers
and are **not this plan's crates**. `docs/contracts/machine-proxy-file.md` is
corrected to say so — its *no road out reads this file yet* had been stale since
`alo-looking-once` landed.

*Machine-wide, and honoured by applications* (`docs/features.md`, v0.5) — a
setting nothing reads is neither.

- **Acceptance:** installing an application and updating one take the way
  `alo_proxy::the_way` decides from the `alo_proxy::Kept` the **machine's own
  file** holds, read through `alo_networks::proxy_file::kept_on_this_machine`
  and signed in to through `alo_proxy::signed_in` and nothing else, with a test
  that puts each errand on a machine whose file names a proxy and finds the
  rented tool configured with it — and the same errand on a machine with no
  file going straight out; a machine whose file **is not one this machine
  wrote** refuses the errand in words rather than reading it as *no proxy*,
  which on a company network is a machine that silently goes nowhere; a proxy
  that asks who this machine is is signed in to with the password task 13 lets a
  person set, held by a test that reads the first line on the wire rather than
  an address; and the **egress indicator still names where the errand is really
  going**, never the proxy, held by a test that reads the line.
- **Constraint:** nothing here re-decides which way a road goes — `alo-proxy`
  decides and this is the crate that asks it — and nothing here reads the
  machine's description: an organisation's proxy reaches the file through the
  broker, and how it gets there is not this task's. `alo-updating` and
  `alo-models` are **not** this plan's crates and are not touched: this task is
  `alo-software`'s two roads, and what it leaves owed for the other two is
  named in its report the way this one named it.
- **Not this task:** a road for an application's own traffic. What an installed
  application is given is `alo_proxy::Published` and the portal
  (`alo_proxy::looked_up`), which task 4 built; this is alo OS's own errands.
