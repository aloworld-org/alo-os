# Demo 2: alo working inside applications

**Status: approved by the owner on 2026-11-10.** Asked *"Should I mark the
Demo 2 plan approved and start its stage 1?"*, the owner answered *"Yes
please"*. It is owned by the coordinator lane on this PC (`alo-os-shell`
checkout), as the charter's row says, and stage 1 — a measurement on the booted
qemu machine — comes first.

**What it is for.** The public demo for developers, decided by the owner on
2026-10-31: alo builds something simple **inside the open-source code editor
alo OS ships**, and presses *Publish* to put it on GitHub Pages. alo must be able
to drive **any installed application the person lets it use**, Microsoft's VS
Code included if they install it. **The demo posts nowhere social** — the owner:
*"LinkedIn was just an example"* — but posting in the person's name stays a
feature, and stage 12 builds it off the demo's path.

**The promises** are in `docs/features.md`, *alo working inside applications,
and in the person's name*. **The decisions, the owner's answers of 2026-10-31,
and the facts that make this hard** are in
[ADR 0143](../decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md).
Read it first; this plan does not repeat it.

**Every choice is the person's, and each stage builds its own.** The owner:
*"we are not dictators so we allow users to choose what he wants."* So a stage
that adds a way for alo to act also adds its setting, in ADR 0143's table:
the safe answer as the default, more up to *alo does it all for me*, each step
with its risk in plain words — and the record, undo where it reaches, the
indicator and plain warnings on at every level. **Two of those settings' top
levels are allowed** by the owner's decision of 2026-10-31 — *an approval is
never a session unless the person chose otherwise in Settings* — and are built
once `CLAUDE.md` carries those words, which is the owner's edit (ADR 0143).

**Where it ends.** The demo walked on the qemu machine, by a person pressing the
keys, under the rule `docs/walks/a-day-on-alo-os.md` sets: *nothing is
simulated*, and a walk that stops is a finding.

## How it became a plan with tasks

Until 2026-11-10 this plan was **proposed and not yet approved**, and its stages
sat under a heading of their own on purpose: a plan with a `## Tasks` section
must be named in `ROADMAP.md` and in `a-new-machine-becomes-a-lane.md`, and
`ROADMAP.md` has one writer, the integration owner (`SHARED_MAIN.md`). The
owner's approval made it a plan with tasks in one change: the heading renamed
to `## Tasks`, its row in `ROADMAP.md`'s phase 6 written by the integration
owner, and the owning machine's row in the charter. The two reconciling tests
count it from that change on.

Each stage's **Status** was read again against `main` in the same change.
Stage 3 moved, and its line says why. Stage 9 was already marked in progress
(#1154, #1163). The rest are still not started.

## Sizes

**S** is about a day, **M** two to four days, **L** one to two weeks, each for
one lane with its gate. The owner answered ADR 0143's questions on 2026-10-31;
what a stage still waits on is named in its own lines.

## Tasks

### 1. Measure the ground on the qemu machine

**Status:** not started. **Size:** S. **Waits on:** nothing.

**What it answers, for the person:** whether the things this whole plan stands
on are true on a real alo machine, before anybody builds on them. On a signed-in
qemu session: is the accessibility bus running at all (ADR 0143 fact 10); do
VSCodium and VS Code install from Flathub and start on alo's compositor, which
has no X11 support, in 4 GB; with `--force-renderer-accessibility`, does each
editor's window appear on the bus through Flatpak's proxy, and how much of an
open file does its tree show; can text be put into either editor, Text
Editor's document, LibreOffice Writer and a web page's own editor in Firefox
through the tree; do Zed's and Lapce's windows publish anything at all (the
editor choice rests on it); does Firefox's page appear; is each window's
sandbox read as the fallback reads it.

**Reuses:** `docs/walks/` and the qemu notes; `busctl` by hand; the fallback's
own acceptance list in `docs/autonomy/updates/the-accessibility-fallback.md`.
**Test:** none — **a measurement done once by hand**, and the report says so
and names which later stage turns each answer into a check that runs again.
**Delivers:** a report in `docs/autonomy/updates/`, and decides how alo types
on evidence — the owner left that to engineering (ADR 0143).

**What the boot walk of 2026-11-11 found, and what it changed.** The bus runs in
the person's session. VSCodium showed alo **only its window frame** — *This
window shows no control that can be named* — after `ACCESSIBILITY_ENABLED=1`,
because Chromium describes the page inside its window only when started with
`--force-renderer-accessibility` (`docs/quirks/chromium-describes-only-its-window-frame-until-asked.md`).
The owner then decided that **every application describes itself from the
start, with a switch in Settings → Accessibility** (`docs/features.md`, 2026-11-11),
so this question is now answered by default rather than by hand: the image
asks every toolkit, and the desktop adds Chromium's switch when it starts
VSCodium, VS Code or Chromium. The rest of this stage's questions stand.

### 2. The accessibility bus is on the image, started and not stat-ed

**Status:** not started. **Size:** S. **Waits on:** stage 1, and only if the
bus was missing.

**For the person:** without it nothing in this plan works, and neither does a
screen reader. **Reuses:** `image/Containerfile`'s pattern of starting each
program at build time; `crates/alo-image` tests. **Test:** an image test that
the bus launcher is installed and **answers**, not that a file exists.

### 3. A person can let alo use an application

**Status:** its top level is in, by another road; its default is not started.
At **full access** (ADR 0149), alo opens, brings forward and closes any
application the person's account can, with no grant (#1134), and a new machine
offers full access already chosen at first sign-in (#1137). At *ask me each
time* and the sealed box every application is still refused, because nothing
yet lets a person grant one: the Settings list, the compositor's offer and the
install-time grant are unbuilt. **Size:** M. **Waits on:** nothing for the
default; for an *every application* setting below full access, if one is still
wanted, `CLAUDE.md` carrying the owner's change (ADR 0143).

**For the person:** today alo is offered opening and closing applications and,
below full access, every one is refused on a real machine, because nothing lets
anybody grant it an application. After this, Settings lists the installed applications, and the
person lets alo use one, for an hour, a day, a month or until removed, and takes
it back in one action; and when alo needs one it has not got, a surface the
compositor draws — not alo — offers it. Also at install, and *every application
on this machine*, as the person's setting turned up, each said with its risk. **This makes an existing promise true** — *application
verbs over granted applications only* — because the grant had a type and a row
and no act.
**Reuses:** `alo_capability::Reach::Application`; the grants file's
`application` key; `alo-granted`'s listing; `alo-picking`'s `Granting` as the
shape of a deliberate act; `alo-remembering`. **Test:** granted, then
`open_application` opens it; revoked, the next call is refused; expired, gone;
a terminal refused unless ADR 0111's choice is made; and every refusal recorded.

### 4. The two accessibility verbs are offered by a turn

**Status:** not started. **Size:** M. **Waits on:** stages 1 and 3.

**For the person:** alo can read what a granted application shows and press a
named button in it — the first time it works inside an application at all.
**Reuses:** `crates/alo-adapters`' fallback, unchanged, whose walk and press
the person's `alo-app` command already shares (#1118) — but no turn offers
them; `alo-turn`'s executor pattern (`acting_on_an_application.rs`).
**Test:** the twelve-verb test becomes fourteen and `agent-verbs.md` says *offered*; with no invocation the tree is
never asked; a revoked grant stops the next press; on qemu, Firefox's window
read and a button pressed.

**What it can now rest on (2026-11-11):** an application describes itself from
the moment it opens, because the person's *Apps describe themselves* switch is
on as a machine comes (`alo_launching::describing_themselves`). That makes the
tree **able to answer**; it does not make anything read it. The verbs this stage
offers still ask only inside a turn the person started. **And where the person
turned the switch off**, a verb that finds no tree should say so and name the
switch, rather than *this window shows no control that can be named* — which
is this stage's to build, because the refusal is said by the verb.

### 5. An argument can carry text, shown in full

**Status:** not started. **Size:** M. **Waits on:** the owner's acceptance of
ADR 0143 fact 1, which amends ADR 0001.

**For the person:** alo can propose a page of code or a post, and the person
sees **all of it** before approving — nothing cut, nothing hidden.
**Reuses:** `alo_capability::Takes`; the approval surface; the existing refusal
of characters a person cannot see. **Test:** over-long text refused; invisible
characters refused; the approval shows the whole text and says when it scrolls;
the record keeps it.

### 6. alo types into an application

**Status:** not started. **Size:** L. **Waits on:** stage 1's measurement, which
decides how alo types (engineering, not a setting); stage 5; and, for the top
level of the setting, `CLAUDE.md` carrying the owner's change (ADR 0143).

**For the person:** alo writes into the editor and saves, and they watch it
happen. How much one approval covers is their setting: one file by default,
each block if stricter, everything for one request at the top. **Never into
a password field, and never into a terminal** — including the one inside the
editor — unless ADR 0111's choice says so.
**Reuses:** the fallback's identification by sandbox and its rule that a target
is found again at the moment of acting; either at-spi's editable text or the
compositor's delivery to one window, as stage 1's measurement decides.
**Test:** the CI real-window test grows a text field; refusals for a password
field, a terminal, a window not granted, and text not approved; one approval
types once at the default level; on qemu, a file typed and saved in the shipped
editor.

### 7. The fresh machine ships a code editor, and an editor is opened honestly

**Status:** not started. **Size:** M. **Waits on:** stage 1's answer that alo
can see into it. The editor is **VSCodium** (`com.vscodium.codium`), the
owner's choice of 2026-10-31.

**For the person:** a developer's first boot has VSCodium on the Dock, as
it has a browser and a terminal. Where they look at what is installed, it says
that it can read every file in their home and start programs outside its
sandbox (fact 4); and if they install VS Code, the install sentence says that
and that it is proprietary. When alo opens a folder in either, the folder opens
in Restricted Mode, and a control that would run something is refused unless
the person's running-code level (ADR 0064) allows it.
**Reuses:** `crates/alo-software/shipped.toml` and `alo_software::Role`, which
gains an eighth role, `code-editor` — one application per role is refused
twice, so this is a role and not a second `text-editor`; the first-boot
install of the shipped applications; `alo-software`'s install sentence and its
rented tool. **Test:** the shipped list holds eight roles and the editor's
entry passes the licence and release checks; the image test that the fresh
machine's applications arrive grows by one; an application asking for the
whole home is said so; a sealed package is not; *Run* refused at the default
level.

### 8. The window alo is working in wears its presence

**Status:** partly built, 2026-11-10, with the window controls' stage 3. A
window the person offers with *Help with this window* wears the teal edge, and
across its foot alo's mark, what it is doing in the person's words and Stop,
for the turn, on no other window, gone when the turn ends
(`crates/alo-shell/src/alos_band_in_a_window.rs`). **Not yet:** a window alo
picks itself while answering a request typed in the launcher — which is the
demo's road into the editor — because the shell learns which window only when
the person offers it; that needs the agent to say which window it is acting in
as it goes, on the asking socket. **Size:** M. **Waits on:** nothing; it is already
promised (*the agent's presence*, a deep teal edge with its mark and word).

**For the person:** they can always tell when it is alo's hands on their editor.
**Reuses:** the compositor's frame; `alo-appearance`'s roles. **Test:** the
edge appears for the turn on the window acted on, and on no other, and goes
when the turn ends.

### 9. What a driven application sends is on the indicator

**Status:** in progress. **Built 2026-11-10, reached by nothing yet:** the
desktop starts alo's own copy of Firefox, VSCodium or VS Code with a profile of
its own, counts only that copy, and shows one of three lines; the person's own
window is counted at neither setting; any other application is refused at the
default (answer A); the door's words are in `application-door.md`. **Not yet:**
a turn that asks the door (stage 4), the record entry that turn writes, and any
of it run on a machine. **The Settings choice answer A names was built
2026-11-10**: *alo may work in your own window*, on Folders alo can use, read
by the door at each ask. The owner
answered both questions on 2026-11-10 (below). **Size:** L. **Nothing that
publishes is walked before this lands.**

*What was found.* Three things this stage needs do not exist, and none is in
another stage. (1) **Nothing starts a window of alo's own.** The door's `open`
starts an application if it is not running and shows it either way, so it is
always the person's window; and VSCodium, Firefox and LibreOffice, by their own
documentation, hand a second start to the copy already running unless started with a profile of their own,
which a person's agent may not pass as an argument (law 2), so each needs a
reviewed recipe. (2) **ADR 0148's count is of the whole application** — every
unit with its identifier, the person's own window included. Used for a turn
as it stands, it would count the person's own use, which ADR 0143 does not
offer at any setting. It needs a count of the one unit alo started; which
Flatpak scope belongs to that start is unmeasured, and stage 1's list does
not yet ask it. (3) **Nothing in a
turn drives an application yet** — the two accessibility verbs are stage 4 —
so a count built now would have no caller.

*The two questions, which change what a person can do.* **A.** At the default,
alo works only in a window of its own. An application that cannot open a
second, separate window — every one started over the session bus, most GNOME
applications among them — could then not be used by alo at all until the person
turns the setting up, and alo would say so and point to it. Is that the
intended reading of *any installed application* at the default? **B.** alo's own
window is a separate profile, so the person is signed in to nothing there:
they would sign in to GitHub inside alo's editor window (stage 10), and features.md
says alo posts *in the person's own browser*. Is signing in again in alo's
window what was meant, or do publishing and posting run at the turned-up
setting?

*The owner's answers, 2026-11-10*, given in the question dialog and relayed by
the coordinator. **A:** *"Refuse, point to the setting (Recommended)"* — at the
default, alo refuses to work in an application that cannot open a separate
window of its own, says plainly that it cannot work there safely at this
setting, and names the Settings choice that lets it use the person's own
window, where the indicator says *that* the application is sending and not how
much. **B:** *"You sign in once in alo's copy (Recommended)"* — the first time
alo needs a service such as GitHub, alo's own copy shows the sign-in page and the
person signs in themselves; alo never sees the password; and it stays signed in
until the person signs it out. Both are recorded in ADR 0143.

**For the person:** when alo makes the editor or the browser send something, the
indicator says so while it happens and the record says so afterwards — law 1,
which would otherwise be broken for the first time. By default alo works in a
window of its own, so what that window sends is counted; turned up, it works in
the person's own window and the indicator says *that* it is sending, not how
much. Never unshown, at any setting.
**Reuses:** `alo-egress`'s indicator lines; `alo-bounding`'s socket counting;
ADR 0134's road to the desktop. **Test:** a driven instance's departure is
shown and counted; at the turned-up setting the line is shown; the person's own
window of the same application is **not** counted at either.

### 10. The person signs in to GitHub

**Status:** not started. **Size:** M. **Waits on:** stage 1's answer on the
editor's sign-in road.

**For the person:** they sign in themselves — alo never types a password — by
the code the editor shows or by a link that now opens in Firefox. If links are
needed, answering `OpenURI` for a web link with the machine's browser **makes
an existing promise true** (open-with and default applications) and is built
here. **Reuses:** `alo-portald`'s `OpenURI`; the shipped browser role.
**Test:** a link opens in the browser; refused for an application with no grant
of it, as the portal's file road is.

### 11. Publishing to GitHub Pages

**Status:** not started. **Size:** S. **Waits on:** stages 4, 6 and 9.

**For the person:** alo presses *Publish* in the editor — the owner's answer,
2026-10-31 — after one approval whose sentence says plainly that publishing to
the internet cannot be undone. Naming the repository after the person's own
GitHub site serves it with nothing more to switch on. **Reuses:** stage 4's press;
ADR 0110 §2's wording on what undo cannot reach. **Test:** the sentence carries
the destination and the warning; one approval, one press, recorded.

### 12. The social switch, posting, and deleting

**Status:** not started. **Size:** L. **Waits on:** stages 5, 6 and 9. **Not on
the demo's path**: the owner dropped posting from the demo on 2026-10-31, and
the walk does not wait for this.

**For the person:** Settings has *Let alo post as you*, off, for one site and
one account by default, or every social site they are signed in to when turned
up, with its risk in plain words — including that the site may restrict their
account; it lasts as long as they choose and goes in one action. Each post and
each deletion is its own sentence with the full text and its own approval. alo
posts in the person's own browser or application, as they would; no site's
publishing interface is used (the owner: *"we do not need APIs for this"*).
**Reuses:** the grants file and list (a new kind of grant, as ADR 0040 added
applications); stage 5's text; stage 6's typing; stage 4's press. **Test:** off by default; no post without the switch **and** an
approval; revoked stops the next post; expired gone; a deletion its own
approval; every post and refusal recorded; the departure on the indicator.

### 13. The Demo 2 walk

**Status:** not started. **Size:** S to write, then the walk. **Waits on:**
every stage above except 12.

**For the person:** the demo — alo builds a page in the shipped editor and
publishes it to GitHub Pages, posting nowhere social — done once on qemu by a
person pressing the keys, written up step by step in `docs/walks/` with where it
stopped if it stopped.
**Test:** the walk itself — **performed once by hand**, and its report says so.

## Roughly how long

Thirteen stages: four S, six M, three L. **The demo's path is twelve of them —
about five to eight weeks for one lane**, less with two lanes once stages 3 to
5 have landed; stage 12, posting, is one to two weeks more and off that path. It is not days. The
measurement in stage 1 can change this, in either direction, and is why it
comes first.
