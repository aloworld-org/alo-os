# The web browser, and what it may take from the machine

**Date:** 2026-09-15
**Workstream:** v0.5 — software and the web
(`docs/autonomy/v0-5-software-and-the-web-plan.md`, task 3)
**Contributor:** Claude Code worker in `C:\dev\alo-os-2`
**Status:** ready for integration. The browser itself reading the shipped
configuration on a real machine is still to do, as the rented tool is for tasks 1
and 2; *What was not run* says what that check involves and names the one thing
about it that is genuinely unknown.

## What changed, for a person

A web link now opens somewhere. When any application on the machine asks for a
web address to be opened, it opens in the web browser — the one the machine came
with, or another one the person chose. If the person removed the browser, the
machine says so plainly instead of opening the link in something that would show
them nonsense.

Only real web addresses are opened. A link that is really a file on the machine, a
piece of code, or a document the asking application wrote itself is refused; so is
a web address with a name and password written into it, which is both a way of
handing over a password and the usual shape of a link pretending to be a different
site. The person is told what happened and what to do, and the address itself is
never read back to them, because it was written outside their machine.

The browser is an ordinary application. It arrives allowed nothing — no folder, no
camera, no microphone — and it stays that way until the person allows it
something. That is why downloads work the way they do: the browser asks where each
download is to be saved, and the folder is the one the person picked. It cannot
write to a folder nobody gave it.

Nothing alo OS ships changes the browser's home page, its search engine, its
bookmarks or which road out of the machine it takes. The only thing set is its
reporting back to its own maker, which is turned off. What could not be turned off
is listed below rather than glossed over.

## What changed, in the code

All of it in **`crates/alo-software`**, which this plan owns. Nothing in
`alo-portals`, `alo-applications`, `alo-capability` or any other crate this plan
reads was edited — see *Decisions*, 1.

| File | What it holds |
|---|---|
| `src/web_address.rs` (new) | `WebAddress`, `NotAWebAddress`, `LONGEST`. What a web address is, checked at the boundary: `http`/`https` only, a host that is one, a port that is digits, one line, nothing over 2048 bytes, and no name-and-password before the host |
| `src/browsing.rs` (new) | `WhatOpensWebAddresses`, `TheBrowser`, `Because`, `NothingOpensThem`. Which application opens web addresses: the person's choice, then the one the machine ships, then nothing — never a fallback |
| `src/opening_the_web.rs` (new) | `opened`, `Opened`, `NotOpened`. An application's request to open one, judged against the grants in the open-with portal's own order |
| `src/browser_settings.rs` (new) | `Setting` (the three things alo OS sets, each with why) and `NeverSet` (the ten it does not, each with what setting it would take from the person) |
| `src/browser_configuration.rs` (new) | `Configuration`, `NotConfigured`, `THE_CONFIGURATION`, `WHERE_IT_IS`, `WHERE_THE_BROWSER_READS_IT`. The shipped configuration, read and held to being exactly the three settings |
| `browser.json` (new) | The configuration itself, as data, in the browser's own published policy notation |
| `src/words.rs` | Ten new sentences (`software.web.*`), the `{chosen}` gap, `THE_REFUSALS`, and two new tests — every refusal is one of the sentences, and no sentence about the web has a gap for the address |
| `src/lib.rs` | The five modules, their re-exports, and a new section of the crate documentation |
| `Cargo.toml` | `serde_json`, for the browser's own document notation. Already a workspace dependency, so it adds no crate to the tree |
| `tests/the_web_browser.rs` (new) | The acceptance tests, against a stand-in for the rented tool |

## Decisions, and why

### 1. The decision about web addresses lives in `alo-software`, not in `alo-portals`

The plan says this crate **reads and never edits** `alo-portals`. The acceptance
says the browser opens web addresses *through the open-with portal*. Both are
satisfiable at once, and the way they are is not a compromise — it is the direction
the dependency has to run.

`alo_portals::open_uri_portal` already says, in its own words, that `OpenURI` is
answered `2` because *nothing on this machine decides what opens a web link yet*.
That missing decision is what this task supplies. It cannot be supplied inside
`alo-portals`, because the answer is read out of `alo_software::Shipped` — the
list of what a fresh machine has — and a crate cannot read a crate that reads it.
If the decision were put in `alo-portals`, wiring the backend to it later would be
a dependency cycle, and the fix would be a second copy of the shipped list.

So `alo_software::opened` is to a web address what `alo_portals::open_with::answered`
is to a file: the same four questions in the same order, the same
`alo_capability::Grants` asked the same way, and the same refusals. The backend
that answers `OpenURI` calls it where it calls the other. That wiring is the
applications plan's lane — it owns `alo-portals` and the bus — and it is the only
part of this acceptance clause that is not here. It is named in *Proposed changes*
below so that lane sees it.

**What was considered and not taken.** Adding a `Request::over_a_web_address`
constructor to `alo-portals` would have let this crate build a real
`alo_portals::Request` and reuse `Request::judged`. It was not taken: it edits a
crate this plan may not edit, and it would still leave the decision — *which*
application — needing to come from here. The order and the refusals are reproduced
faithfully instead, and `opening_the_web.rs` says clause by clause which of
`open_with.rs`'s it is reproducing.

### 2. What a request for a web address has to have been granted

For a file, ADR 0040's row for this portal is *the file, and the application to
open it*: two grants. A web address is not a file and is not on this machine, so
the first has nothing to be about. What is left is the second — **the asking
application has to have been granted the browser**.

That is the honest reading, and it is also the only one that keeps *no grants
beyond what a person gives it through the portals* true in both directions. The
alternative — any application may make the browser open any address, no grant
needed — would mean an application the person granted nothing could make the
machine fetch a web address of its choosing, which is egress caused by something
nobody allowed anything.

*From any application* is about the road, not the price: there is no list of
applications privileged to open links, and no application refused for being the
wrong one. Every one of them needs the same grant, made by the person and
revocable like any other.

The order matters and is tested: **the grants are asked before this machine is
read.** An application holding nothing is refused identically whether the machine
has a browser or not, so a refusal never tells it which browser somebody uses —
the same reasoning `alo-applications` gives for asking the grants before the
installed list.

### 3. A person's own application is never handed a web address

A person may choose which application opens web addresses. They may not choose
one of their own applications for it (`alo_capability::A_PERSONS_OWN` — the
terminals, ADR 0043). A web address arrives from any application and is written by
whoever wrote the page it came from; handing attacker-written text to an
application where whatever arrives runs is the one thing ADR 0043 exists to
prevent, even though that ADR's own refusal is about agents and this is not an
agent.

It refuses by answering with the shipped browser instead and **saying which choice
was passed over** (`Because::ThisMachineShipsItAndTheChoiceIsAPersonsOwn`), rather
than failing: the person gets their link, and they get told why it did not go
where they said. With the browser removed as well, nothing opens web addresses —
the terminal is not the fallback either.

### 4. Only `http` and `https`, and every other scheme refused by name

`file:` would turn *open this link* into *read this path*, which is the grant model
gone. `javascript:` and `data:` are code and a document the asking application
wrote, wearing an address. `about:` reaches into the browser's own settings. None
is the open web. Each is refused naming the scheme it asked for — the scheme is
short, checked and the part worth writing into a record — and the person's sentence
does not repeat the address.

**A name and password in the address is its own refusal**, with its own sentence,
because the two things a person should do about it are different: *that is not a
web address* and *sign in on the site instead* are not the same advice.

**Nothing that arrived is read back to a person.** An address is attacker-written
text, and a sentence that quotes it is one whoever wrote it helped compose. A test
in `words.rs` holds that no `software.web.*` sentence has a gap for the address.

### 5. The configuration is data beside the crate, in the browser's own notation

A rented engine is configured and never patched (`CLAUDE.md`, ADR 0005), and the
one surface this browser offers for configuration is its published enterprise
policy document. So `browser.json` is that document, written in that notation,
sitting beside `shipped.toml` for the same reason: the installer plan can put a
file on a machine and cannot call a function in this crate.

The notation has nowhere to put a comment, which is why `Setting::why` exists — the
reason for each setting is in the crate beside the reader rather than in the file.
This is a real loss against `shipped.toml`, which is commented throughout, and it
was accepted rather than solved by inventing a commented format of ours that gets
rendered: a second representation is a second thing that can disagree with the
first, and the file the browser reads is the one that matters.

`Configuration::read` refuses a document that names a key that is not one of the
three, gives one of them a value other than `true`, or leaves one out. The
value check is not fussiness: a key whose value is text is where an address, a
search engine or a home page would arrive on a key that looked harmless.

### 6. Three settings, and the tenth key that would have opened the other nine

| Set | Key | Why |
|---|---|---|
| Telemetry off | `DisableTelemetry` | `docs/features.md` promises *no telemetry*, and this is how far that reaches into an application somebody else makes |
| Experiments off | `DisableFirefoxStudies` | An experiment changes how a person's browser behaves without their having chosen it, and reports on what happened |
| Ask where each download goes | `PromptForDownloadLocation` | This is what makes *the folder a person chose* true: a browser that does not ask writes to a folder it picked |

`NeverSet` is the ten that are not set, each with what it would take: the home
page, the new tab, the start page, the search engines, the proxy, both spellings of
the download folder, bookmarks, which sites a person may read — and `Preferences`,
which reaches any single setting the browser has by its internal name and is
therefore the door every other one of those could arrive through. It is refused
even where the setting behind it would have been harmless, for the same reason
ADR 0001 §2 refuses an escape hatch.

The reader would refuse all ten anyway, since it accepts only what is decided.
`NeverSet` earns its place by making the refusal *say what the key would have cost
the person*, at the spot somebody will be standing when they want to add one.

**Considered and not taken:** `DisableAppUpdate`. Applications are updated through
one road on this machine (task 1), so a browser updating itself would be a second.
It was left out because the sandboxed build of this browser ships without its own
updater already, and a policy claiming to turn off something that is not there is
a claim we cannot demonstrate.

### 7. The proxy reaches the browser because nothing here writes one

Task 4 builds `alo-proxy`: one proxy, machine-wide, published to applications
through the environment and the proxy interface they already read. The right thing
for this task to do about it is **nothing**, and to hold that it did nothing:
`Proxy` is on `NeverSet`, a configuration that writes one is refused naming what it
would take, and `Configuration::takes_the_machines_proxy` reads the document and
says so.

Writing `Mode: "system"` into the document was considered and not taken. It is
already this browser's behaviour on this platform, so setting it would state a
default as though it were a decision — and a machine with two copies of one setting
is a machine with two answers the day they differ. The handshake with task 4 is
therefore: alo-proxy publishes; the browser, unconfigured, follows. That is one
line for task 4's per-road test to cover, and it is named in *Proposed changes*.

### 8. The browser is a `Role`, not a special case

Nothing in this change asks whether an application is *the browser* in order to
treat it differently. `Shipped::the(Role::WebBrowser)` is the only place the
identifier comes from, a test holds that the browser is **not** on
`A_PERSONS_OWN` — an agent may be granted it, a person may remove it — and
`the_browser_is_an_application_like_any_other_and_arrives_granted_nothing` walks
the same road any application walks: arrives holding nothing, reaches nothing,
reaches exactly one folder once a person gives it one, and loses it when it is
removed.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| The browser from task 2 opens web addresses from any application, through the open-with portal's decision and its order | `. alo-software the_web_browser the_browser_opens_web_addresses_from_any_application` |
| …and an application holding nothing is refused before this machine is read, learning neither whether there is a browser nor which one | `. alo-software the_web_browser an_application_granted_nothing_opens_no_web_address_and_learns_nothing` |
| …and only the open web is opened: a path, code, a document the asker wrote, or a password in the address is refused | `. alo-software the_web_browser nothing_but_the_open_web_is_opened` |
| …and a machine whose browser was removed opens web addresses in nothing — not in the terminal it also ships | `. alo-software the_web_browser a_machine_whose_browser_was_removed_opens_web_addresses_in_nothing` |
| It is a sandboxed application like any other, with no grants beyond what a person gives it | `. alo-software the_web_browser the_browser_is_an_application_like_any_other_and_arrives_granted_nothing` |
| Its downloads go to the folder a person chose | `. alo-software the_web_browser a_download_goes_to_the_folder_the_person_chose` |
| The machine-wide proxy (task 4) reaches it | `. alo-software the_web_browser the_machines_one_proxy_reaches_the_browser` |
| Nothing alo OS ships sets its home page, search engine or telemetry, with telemetry off where a policy allows it | `. alo-software the_web_browser nothing_alo_os_ships_sets_its_home_page_search_engine_or_telemetry` |

Beside those, in the crate's own tests: `web_address` (four), `browsing` (five),
`opening_the_web` (six), `browser_settings` (two) and `browser_configuration`
(five), each with its refusal path.

## Verification

Run in WSL against this checkout (`/mnt/c/dev/alo-os-2`) with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-2-72aa7fda7f7de151`. Windows itself
cannot build this workspace here: a dependency's build script needs a C compiler
that is not installed, which is why every gate below is the Linux one.

- `cargo fmt --all`, then `cargo fmt --all --check`: clean.
- `cargo clippy --all-targets -- -D warnings`, whole workspace: clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-software --no-deps`: clean.
- `cargo test -p alo-software`: all passing — the crate's own tests, both
  integration suites and the doctest.
- `cargo test -p alo-saying -p alo-collected -p alo-citing`: pass — the ten new
  words are collected into the one vocabulary through this crate's existing
  entry, and every decision this change points at is one somebody wrote.
- The eight evidence tests, each on its own with `--exact`: each `1 passed`.

The full workspace suite was not run here, as instructed; the supervisor runs it.

## What was not run

- **The browser on a machine, reading the shipped configuration.** The check on a
  certified machine: put `browser.json` where the browser reads a machine-wide
  policy document (`alo_software::browser_configuration::WHERE_THE_BROWSER_READS_IT`,
  `/etc/firefox/policies/policies.json`), start the browser, and confirm from its
  own *about* page for enterprise policies that it read exactly three and no
  others; download a file and confirm it asks where; set a machine-wide proxy and
  confirm the browser uses it without being told.
- **Whether the sandboxed build sees that path is the one genuinely open question
  here, and it is not glossed.** The path above is the browser's own documented
  location for a machine-wide policy document. This browser is installed
  sandboxed, and `/etc` inside a sandbox is not this machine's `/etc`. Which of
  the rented tool's own mechanisms carries the file in — and whether the answer is
  a different path entirely — is a fact about a real machine and a real tool, and
  writing a guess into `WHERE_THE_BROWSER_READS_IT` would have been a claim
  without a measurement. `docs/quirks.md` is where the answer goes once a machine
  has been asked; nothing was written there now, because reality has not yet
  disagreed with anything — it has not yet been consulted.
- **The bus.** `org.freedesktop.portal.OpenURI`'s `OpenURI` method still answers
  `2`. Wiring it to `alo_software::opened` is the applications plan's, for the
  reason in *Decisions*, 1.
- **No surface shows any of this.** Choosing which application opens web addresses
  is a setting in `alo-shell`, which this plan does not touch; `WhatOpensWebAddresses`
  takes the choice as an argument rather than reading a file, so whichever crate
  comes to keep it can keep it without this one changing.

## What could not be turned off, named honestly

`docs/features.md` promises **no telemetry**, and that is alo OS's promise about
alo OS. A rented browser is somebody else's application, and a policy turns off
what its maker made a policy for and no more. With the two settings above, the
browser sends its maker no usage data and enrols the person in no experiments.
These remain, and none of them has a policy that removes it:

- **The crash reporter.** It is still installed. Sending a report is the person's
  choice, made after a crash, and nothing is sent unless they make it.
- **Its own block lists**, kept up to date against a service so that the browser
  can warn a person about a site. Turning that off is a change to how safe the
  browser is, which is not ours to make on a person's behalf.
- **Remotely-delivered settings**, which the browser fetches to configure itself.
- **Captive-portal detection** — the request that finds out whether the network
  wants a sign-in first.

None of these is usage telemetry, and none of them is an agent. The four laws'
*zero inference egress, measured at the network boundary* is measured **of an
agent**; a browser a person is driving is that person reaching the web, which is
what a browser is for. The egress indicator is about what an agent causes, and
nothing here causes any of it.

## Limitations

- `opened` answers *which application, and may it* — it opens nothing. Asking the
  application to open the address needs a running session, as it does for a file.
- The check on a web address is ours and deliberately strict: it refuses an
  address whose host is not written in ASCII. An address in a person's own
  alphabet reaches this machine already converted by whatever produced it, which
  is how the web carries them, but an application that hands over an unconverted
  one is refused rather than guessed at. Converting is a decision with a
  dependency behind it, and it belongs to whoever adds it with its own test.
- `NeverSet` is a list of the keys worth naming, not a proof that the browser has
  no eleventh. Completeness comes from the reader accepting only what is decided,
  which it does.
- The person's choice of browser is an argument, not a kept setting. Nothing keeps
  it yet.

## Proposed changes to shared documents (for the integration owner)

- **CHANGELOG.md:** *A web link from any application now opens in the web
  browser — the one the machine came with, or another one you chose. Only real
  web addresses are opened: a link that is really a file, a piece of code, or one
  with a password written into it is refused and explained. The browser is an
  ordinary application that arrives allowed nothing, so downloads go to the folder
  you pick when it asks. Nothing alo OS ships changes its home page, its search
  engine or which road out of the machine it takes; its reporting back to its own
  maker is turned off.*
- **ROADMAP.md (v0.5, *A web browser for the open web*):** decided and held in
  tests — what opens a web address, what may ask, and the configuration alo OS
  ships with it. Outstanding on a machine: the browser reading that configuration,
  and the `OpenURI` method on the bus.
- **Applications plan, for its lane:** `org.freedesktop.portal.OpenURI`'s
  `OpenURI` can stop answering `2`. Call `alo_software::opened(application,
  address, &WhatOpensWebAddresses::on(…), &grants, now)` where `open_file` calls
  `alo_portals::open_with::answered`, record the outcome the same way, and open it
  in the application the answer names. The dependency runs `alo-portals` →
  `alo-software`, never the other way.
- **Software plan, task 4 (`alo-proxy`), for whoever takes it:** the browser is one
  of the roads out to hold a test against, and the test is that **nothing is
  written into the browser**: it follows the machine's published proxy with no
  configuration of its own, and a shipped configuration that names a proxy is
  already refused (`NeverSet::Proxy`).
- **QUEUE.md / STATE.md:** task 3 of the software plan is done; reference this
  report. The two on-machine items above join task 1's and task 2's outstanding
  rented-tool acceptance rather than forming a new one.
