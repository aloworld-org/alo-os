# ADR 0143 — alo works inside applications, and may post in the person's name when they switch it on

**Status:** **accepted in part, 2026-10-31, by the owner**: the four decisions
quoted under *What the owner decided* are theirs and are not reopened here.
**Everything else is proposed.** The section *Questions only the owner can
answer* is open, and no code may be written against an answer it has not got.
**Nothing here is built.**

*Number: `origin/main` ends at 0141, and the next number is already taken by a
decision about small vector artwork on a branch not yet landed, so this is the
one after it, chosen after a fetch at push time.*

## What it changes for the person

Today alo can rename, move and find files, and propose installing an
application. It is offered opening and closing applications too, and on a real
machine **those are refused**, because nothing on the screen lets a person grant
alo an application — only writing the grant into the grants file by hand
would (fact 11). **It cannot do anything inside an application**: it
cannot read what VS Code shows, cannot type a line into it, and cannot post
anything anywhere. After the work this record starts, a person can ask alo to
build something in VS Code and watch it happen in VS Code, approving what it
types; and, if they have switched it on in Settings, ask it to post to LinkedIn
in their name, approving each post with its full text in front of them.

**What it costs them is said plainly below**, because two parts of it are
risks no other part of this product carries: alo publishing under a person's
name to people who are not on this machine, and alo driving an editor that can
itself run programs.

## What the owner decided, 2026-10-31

1. **The public demo for developers is Demo 2 — alo working inside
   applications** — and not the first walk (`docs/walks/a-day-on-alo-os.md`).
2. A person installs **VS Code**: alo proposes it with `install_application` and
   the person approves.
3. alo uses **VS Code itself** to develop something simple. It types and saves
   in VS Code's editor, as a person would. **It does not write files behind its
   back.**
4. It is deployed to **GitHub Pages**.
5. alo **writes a LinkedIn post about it and posts it**. The owner's words:
   *"alo opens the browser or the desktop application for LinkedIn and posts the
   post. User allows this in the settings that allow alo to use my social media
   profile. Openclaw became popular with its WhatsApp and telegram and deleting
   feature so no problem if the users approves it."*
6. **Any installed application, not VS Code alone.** The owner's words, the
   same day: *"for alo driving applications it should be able to drive any
   application installed."*

These are law 5 and [ADR 0112](0112-with-the-persons-permission-the-agent-may-do-anything-on-the-machine.md)
applied: *if the person grants it, the agent may do it*, and the question is
what they are told and how they switch it on and off.

## What exists, and how far each part has run

| Piece | Built | Tested | Offered to a model | Run on a booted machine |
|---|---|---|---|---|
| `install_application` (propose, approve, install from Flathub) | yes | yes | **yes** | an agent-proposed install is in no walk |
| `open_application`, `focus_application`, `close_application` | yes | yes | **yes** | **never by an agent**: nothing on the machine lets a person grant alo an application (below), so each is refused at its grant |
| a grant over one application (`Reach::Application`) | the type, the grants file's `application` key, and the list's row (`alo-granted`) | yes | — | **no act makes one**: `alo-picking`'s header calls the folder picker *the one act on this machine that makes a grant* |
| `accessible.read_window` — read what a granted application shows | yes, `crates/alo-adapters` | yes, against real GTK 3 windows over at-spi2's own bus, **in CI on every build** (`the_accessibility_fallback_on_a_real_application.rs`) | **no** | **never** |
| `accessible.activate_control` — press one control by kind and name | yes | yes, same test | **no** | **never** |
| `picture_of_the_screen` | declared | yes | **no** | never |
| a verb that puts text into an application | **no** | — | — | — |
| a verb that writes a file | **no** — the six file verbs list, read, find, rename, move and archive | — | — | — |
| anything about a social account | **no** | — | — | — |

The fallback has **never been measured against a browser, an Electron
application, or an application inside a Flatpak sandbox**; its own report says
so (`docs/autonomy/updates/the-accessibility-fallback.md`, *What is not shown*).
`alo_turn::Machine` offers twelve verbs and its test
`every_verb_this_machine_offers_is_named_here_and_no_other` names them; neither
fallback verb is among them.

## What the decisions run into

Each of these is a fact read from this repository or measured on 2026-10-30,
and each one is a reason the demo is **weeks of work and not days**. None of
them makes the demo impossible; three of them make the demo **unsafe if built
the obvious way**, and those are marked.

### 1. No argument an agent sends can carry text

`docs/contracts/agent-verbs.md`, *What an argument can be*: *"None of them is
free text, and that is the point"* — and a newline is refused inside any value.
Typing a page into VS Code and publishing a post both need **many lines of
text** that the model wrote. The contract says how that changes: *"Adding a kind
is a change to what an agent can express and belongs in ADR 0001 before it
belongs in an implementation."* So [ADR 0001](0001-the-capability-model.md) is
amended first, with a new kind — proposed: **text**, bounded in length, shown to
the person **in full** in the approval, never handed to an interpreter by
anything on this machine, and refused if it holds characters a person cannot
see (the rule the sentence already keeps).

### 2. Nothing can put text into an application, and the fallback refuses typing by design

The fallback has two verbs, read and press. `app-adapters.md` refuses an
adapter argument *declared as a script, a command or free text*, and refuses the
`synthetic` mechanism *whatever else is true*. **The owner's words are "types …
as a person would"**, and there are two ways to do that, which are different
products — question 2 below.

### 3. VS Code is a terminal by another road — **unsafe if built the obvious way**

VS Code has a built-in terminal, *Run* and *Debug*, tasks, and extensions that
run code when a folder opens. An agent that can press any control in VS Code
and type into any text in it **can run any command**, which is exactly what
[ADR 0111](0111-the-person-chooses-whether-their-agent-may-use-the-terminal.md)
makes a separate thing the person switches on, and what
[ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
bounds with the three levels. VS Code is **not** on
`alo_capability::A_PERSONS_OWN`, so nothing today would stop it.

**Proposed, and following from existing law rather than a new choice:** while
alo is driving VS Code, text goes only into an editor — never into a terminal
inside VS Code — and a control that runs something (*Run*, *Debug*, *Run Task*,
a terminal) is refused unless the person's ADR 0064 level allows running code,
in which case it is that level's rules, recorded and undoable, that apply. The
folder alo opens is opened in VS Code's *Restricted Mode* so that extensions and
tasks in it do not start. How reliably a control *that runs something* can be
told apart from one that does not, through the accessibility tree, is
**unmeasured** and is measured in stage 1 of the plan and held by a test in stage 7.

### 4. VS Code is proprietary, packaged by volunteers, and its sandbox is nominal — **the install sentence must say so**

Measured from Flathub's own API on 2026-10-30, with Firefox as the known-present
control and `com.example.DoesNotExist` as the known-absent one (both answered as
expected; the script's exit-code probe read 7):

| | `com.visualstudio.code` | `com.vscodium.codium` |
|---|---|---|
| on Flathub | yes, 1.139.1 | yes, 1.140.06880 |
| licence | `LicenseRef-proprietary` (Microsoft's) | MIT |
| verified publisher | **no** — not published by Microsoft | yes (vscodium.com) |

Read from both packages' Flathub manifests (not measured on a machine): each
asks for `--filesystem=host` (every file the person can reach),
`--device=all`, and `--talk-name=org.freedesktop.Flatpak` — **which lets it
start any program outside its sandbox.** So *"installed sandboxed"*
([ADR 0005](0005-applications-are-sandboxed-and-ask.md)) is true of the
mechanism and **not true of what this application can do**, and today's install
sentence — *install {application} from {source}* — tells the person none of it.
Law 5 says a risk is offered *in plain words*. **The install of an application
that asks for the whole home and for leaving its sandbox must say so before it
is approved**; that is a promise made true, not a new one.

`install_application` is not limited to the catalogue, so either can be
installed today. The catalogue's own test refuses a proprietary licence
(`everything_offered_is_licensed_in_a_way_a_person_keeps`), so VS Code would not
be **listed** in the market without a change to that rule — it only matters if
the demo shows the market, and question 3 asks which editor.

### 5. What an application sends is not on the indicator — **law 1 would be broken by the obvious build**

The indicator draws what `alo-agentd` and alo OS themselves send
(`docs/contracts/native-egress-indicator.md`;
[ADR 0134](0134-the-desktop-is-told-what-alo-agentd-sends.md)). When alo
presses *Publish* in VS Code or *Post* in Firefox, **the bytes are sent by VS
Code or Firefox**, from their own processes, and nothing on this machine would
show them. *"No agent-caused egress escapes the indicator"* is one of the gate's
public guarantees; this would be the first escape. Question 6 asks how it is
shown, and nothing that publishes is built before it is answered.

### 6. Posting to linkedin.com through a browser is against LinkedIn's own rules

Read from LinkedIn's User Agreement (effective 3 November 2025, fetched
2026-10-30), §8.2: a member agrees not to *"use bots or other unauthorized
automated methods to access the Services … create, comment on, like, share, or
re-share posts"*. This is not the law; it is the person's contract with
LinkedIn, and the cost of breaking it falls on **their** account, which LinkedIn
may restrict. Law 5 lets the person choose this **knowing it** — which is why
the switch's risk sentence names it. LinkedIn's own publishing interface (the
*Posts* API, `w_member_social`) is the road it permits, and is question 4.

**There is no LinkedIn application for this system.** Measured on 2026-10-30:
Flathub's search answers **0** results for `linkedin` and does answer for the
control `firefox`. LinkedIn's own desktop application is for Windows only (read
from documentation). So *"the browser or the desktop application"* is the
browser.

### 7. A browser grant would reach every tab

A grant over an application is over all of its windows
([ADR 0040](0040-what-an-applications-grant-is-over.md)), and
`accessible.read_window` reads *what {application} shows in its windows*. A
grant over Firefox so that alo can post to LinkedIn would let it read the
person's bank and mail in the next tab during that turn. Context is offered and
never watched, and the read is still one turn — but it is **far more than the
person meant to offer**. Question 8.

### 8. Signing in to GitHub from VS Code needs a web link to open, and this machine refuses one

`docs/contracts/portals.md`: `OpenURI` *(a web link) … is answered `2`* — this
machine's portal does not yet open a web link for an application. VS Code's
*Sign in with GitHub* opens one. VS Code also offers signing in **with a code**
the person types at `github.com/login/device` themselves (read from VS Code's
documentation, not measured here), which needs no link opened for it. **Either
way the person signs in, never alo.** `git` is in the image
(`image/Containerfile`, started with `--version` at build); the VS Code and
VSCodium packages run on `org.freedesktop.Sdk` 25.08, which carries its own
`git` (read from the manifests, unmeasured). `gh` is in neither.

### 9. A push and a post cannot be undone, and the person must be told before, not after

[ADR 0110](0110-the-agent-writes-the-command-and-only-the-person-runs-it.md) §2:
*"A snapshot rewinds this machine's files. It does not recall a push, un-send an
email, or undo anything that already left — and the surface says so at the
moment of offering."* So the sentence that publishes to GitHub and the sentence
that posts to LinkedIn each say *this cannot be undone from this machine*.
Deleting the post is the nearest thing, and it is its own approval.

### 10. The accessibility bus may not be on a booted machine at all

The image installs no accessibility package by name (`image/Containerfile`
names neither `at-spi2-core` nor a screen reader, and installs with weak
dependencies off), and the fallback has never run on a booted machine. Whether
the bus starts in a signed-in session is **unmeasured**. If it does not, nothing
in this record can work, and nor can a screen reader.

### 11. "Any installed application": what can be driven, by what, and where it honestly stops

**The mechanisms, and which of them exist in this repository:**

| Mechanism | What it reaches | In the repository |
|---|---|---|
| The application's own interface on the bus (an adapter, `dbus`) | one application, through verbs someone reviewed | **built**: `crates/alo-adapters`, with GNOME Text Editor's `open_document` as the worked example; not offered by a turn |
| The application's scripting interface (an adapter, `api`) | Blender, LibreOffice, GIMP — through typed verbs, never a script | **refused when loaded** until the machine carries it out (`app-adapters.md`) |
| The accessibility tree (AT-SPI) — the fallback | any application that describes its window: GTK, Qt, LibreOffice, browsers, Electron when started with accessibility on | **built: read and press**; no typing (fact 2); not offered by a turn; never on a booted machine (fact 10) |
| A picture of the screen | anything that is drawn | **declared** (`picture_of_the_screen`, one approval per picture); not offered by a turn |
| Keystrokes and pointer movements the compositor delivers | anything that takes a keyboard and a pointer | **none**: no code, and `synthetic` is refused by the adapter contract; features.md promises *screenshot-and-click, switched on by the person* and *disabled by policy by default* |

**Application by application** — the seven a fresh machine ships, plus VS Code
and LibreOffice. *Documented* means read from the toolkit's or the project's
own documentation; **nothing in this table has been measured on an alo
machine**, and stage 1 of the plan measures it.

| Application | Toolkit | Can be driven through the tree? | Typing into it |
|---|---|---|---|
| Firefox (`org.mozilla.firefox`) | Gecko | documented yes — Gecko publishes web pages to AT-SPI; a page is as readable as its site made it | into a page's fields, documented; whether a site's own editor (LinkedIn's) takes text put in this way is unmeasured |
| Dolphin (`org.kde.dolphin`) | Qt 6 | documented yes, once Qt sees an assistive technology on the bus | file names only |
| Ark (`org.kde.ark`) | Qt 6 | documented yes | — |
| Text Editor (`org.gnome.TextEditor`) | GTK 4 | documented yes on Wayland; **GTK 4 publishes no tree on Broadway** (`docs/quirks.md`), so CI's real-window test is GTK 3 | the document is a text view; unmeasured |
| Image Viewer (`org.gnome.Loupe`) | GTK 4 | controls yes; the picture is a picture | — |
| Papers (`org.gnome.Papers`) | GTK 4 | controls yes; a page's text partly | — |
| Terminal (`app.devsuite.Ptyxis`) | GTK 4 | yes — and **typing into it is running commands**, which is ADR 0111's choice and the person's alone; `A_PERSONS_OWN` refuses it until that choice exists | only under ADR 0111 |
| VS Code (`com.visualstudio.code`) | Electron | documented yes with `--force-renderer-accessibility`; **its editor is drawn by the application and takes input through a hidden field**, so how much of a file the tree shows is unmeasured | unmeasured (Q2) |
| LibreOffice (`org.libreoffice.LibreOffice`, Flathub, MPL-2.0, 26.8.2.2 — measured 2026-10-30; on the image only as the converter) | its own, through GTK 3 | documented yes — Writer publishes its paragraphs as editable text | documented yes; unmeasured |

**Every one of them is a Flatpak**, and Flatpak gives an application the
accessibility bus through a proxy unless its package says `--no-a11y-bus`
(documented; neither VS Code's nor VSCodium's manifest says it). The fallback
already identifies an application by its sandbox, through the process behind its
connection. **That road through Flatpak's proxy has never been measured** — the
fallback's own report names it as not shown.

**What no mechanism drives reliably, said as a limit and not hidden:**
applications that **draw everything themselves and describe none of it** — games,
3D and painting canvases (Blender's whole interface, Krita's canvas), a web page
that is one big canvas, a remote desktop or a virtual machine's window, video,
and Windows programs run through a compatibility layer. The tree has nothing in
it to read; a picture plus keystrokes can be pointed at them, but what it does
is a guess about pixels, cannot be checked afterwards, and fails when the
picture changes. **alo says *I cannot work inside this application* for these**,
rather than guessing. Where such an application has a scripting interface
(Blender does), an adapter with typed verbs is the honest road, and it is built
per application.

**How the grant scales to "any".** The shape already exists and is the right
one: a grant over **one application, by its identifier**
(`Reach::Application`), in the same list as the folders, with the same four
lengths, revoked in one action, and an adapter's verbs offered only while that
grant is held (`app-adapters.md`). *Any installed application* is therefore
**any application the person has let alo use, one at a time** — never a switch
for all of them, which would be a grant nobody can enumerate, and never one
that appears because alo used an application once (*never inferred, never
widened by use*, `agent-verbs.md`). An application arrives from an install
**granted nothing** ([ADR 0042](0042-installing-an-application-is-an-errand-and-an-agent-only-proposes-it.md)),
so after alo installs VS Code the person still lets alo use it, as a second
act. **What is missing is the act**: nothing on the machine lets a person make
an application grant, so the three application verbs alo is offered today are
refused on every booted machine. Building that act is making an existing
promise true (*Grants: pick a folder, see what is granted, revoke it*, and
*application verbs … over granted applications only*), not a new one. Q10 asks
where it is made.

## What follows from existing rules — proposed, and not a new choice

These are what the repository already promises, applied. The owner may correct
any of them; none of them is a question about *whether*.

- **What alo may see while it drives an application.** Only what the
  application shows, read through its accessibility tree **during a turn the
  person started**, looked for again at the moment of each action, and never
  kept: no subscription, no event, the connection closed when the turn ends —
  the fallback's existing guarantees (`app-adapters.md`). No picture of the
  screen unless the person approves `picture_of_the_screen` for that one
  picture. A password field's contents are never read. **While alo is working
  in a window, that window wears the agent's presence** — the deep teal edge
  `docs/features.md` promises and nobody has built; it is part of this work,
  because a person watching their editor type must be able to tell whose hands
  are on it.
- **How the social switch is stored, revoked and ended.** It is a grant, under
  every term [ADR 0001](0001-the-capability-model.md) §3 and ADR 0112 set:
  enumerated, visible in Settings beside the folders, revocable in one action
  that takes effect on the next action, and expiring — an hour, a day, a month,
  or until removed, the same four lengths a folder has. It lives in the person's
  grants file ([ADR 0088](0088-a-machines-grants-belong-to-a-person.md);
  `docs/contracts/grants-file.md`), held to that file's rules about who may
  write it. **It is a new kind of grant** — over *posting as this person on this
  site*, not a path or an application — so `alo-capability` learns it the way
  ADR 0040 taught it applications. Off until switched on; on a managed machine
  an organisation may forbid it ([ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md)).
  **Revoking it is honest**: it stops the next post and does not recall the
  last one.
- **One approval is one post; one approval is one deletion.** The sentence
  carries the post's full text, the site, and the account it goes out under.
  The switch never stands in for an approval.
- **What the record keeps.** What alo did: each text it put into an
  application, with the application and the file or field; each control it
  pressed; each post, with its full text and where it went; each deletion; each
  refusal; and the grant and approval each happened under. **Never what it
  read** — what was in a window is not recorded, exactly as a context is not
  (`agent-verbs.md`, *Context on invocation*).
- **Egress.** LinkedIn and GitHub traffic caused by alo is agent-caused egress
  and is on the indicator while it happens and in the record afterwards. *How*
  is question 6; *whether* is law 1 and not open.

## Questions only the owner can answer

Each has options and a recommendation. **The recommendation is not the
decision.**

### Q1. What does one approval cover when alo types?

- **(a) Each block of text** — every insertion is its own sentence with its full
  text. Safest; a page of fifty lines in five blocks is five approvals.
- **(b) Each file** — alo proposes the whole content of one file, shown in full
  (as a before and after when the file already had something), then types it
  and saves; one approval, one file. One execution of one argument, so
  *one approval causes exactly one execution* still holds.
- **(c) Each turn** — one approval for everything alo types in the request.
  **This is a session**, which `agent-verbs.md` forbids (*"An approval is never a
  session"*), so it needs that rule amended, and the person approves text they
  have not yet seen.

**Recommendation: (b), with saving inside the same sentence** — *write this into
`index.html` in VS Code and save it*. It is what a person can actually read, and
it keeps every existing guarantee.

### Q2. How does alo type?

- **(a) Through the accessibility tree** — the text is put into the editor's
  text through at-spi's *EditableText*, the way a screen reader's dictation
  does. Auditable: the record says exactly what went where. **Whether VS Code's
  editor accepts text this way is unmeasured** and doubtful, because VS Code's
  editor is drawn by the application and takes input through a hidden field;
  Electron needs `--force-renderer-accessibility` (accepted by VS Code since
  2019, read from its issue tracker) or an assistive technology asking before it
  publishes a tree at all.
- **(b) As keystrokes the compositor delivers to one window** — alo's own
  compositor sends the characters to that window and no other, exactly as a
  keyboard would, so it looks like a person typing. It works in any application.
  It is the `synthetic` mechanism `app-adapters.md` refuses today, and the
  reason given there — *after the fact nobody can say what it actually did* — is
  about screenshot-and-click guessing positions; keystrokes sent to a named
  window, with their text recorded, can say exactly what was sent. It needs that
  refusal amended, and the window identified by its sandbox the way the
  fallback already identifies it.

**Recommendation: measure (a) first (stage 1), and decide on the measurement.**
If VS Code's editor does not accept it, (b) restricted to text, to one window
alo is presenting as its own, and to the editor never its terminal.

### Q3. VS Code or VSCodium?

- **VS Code** — what the owner named and what developers recognise; proprietary,
  not published on Flathub by Microsoft.
- **VSCodium** — the same editor built from the open source, MIT, verified on
  Flathub, and acceptable to the catalogue's licence rule.

**Recommendation: VS Code for the demo, because it is what the owner decided and
what the audience uses — with its install sentence saying it is proprietary and
what it can reach** (fact 4). VSCodium is listed in the catalogue for people who
want it. *The owner may reasonably answer the other way; this is the one
question where the demo's audience and the product's principles pull apart.*

### Q4. How does alo post to LinkedIn?

- **(a) In Firefox, as a person would** — what the owner described; against
  LinkedIn's §8.2, and the person's account carries the risk; driven through a
  page LinkedIn redesigns whenever it likes; and its traffic is Firefox's
  (fact 5).
- **(b) Through LinkedIn's own publishing interface** — the road LinkedIn
  permits; alo-agentd sends the post itself, so the indicator, the boundary and
  the record apply exactly as they do to a question sent to a provider. It needs
  **alo registered with LinkedIn as an application**, a sign-in the person
  completes in their browser, and a key kept in the keyring. It is not *inside
  an application*.

**Recommendation: (b) for the product, and (a) only with the risk sentence**,
because the product's whole argument is that nothing it does puts the person at
risk without telling them, and (b) is the one where nothing is at risk. If the
owner wants the demo to show alo in the browser, (a) with the sentence is lawful
under law 5 — **the owner should know it may get the demo account restricted.**

### Q5. Who pushes to GitHub?

- **(a) alo presses VS Code's *Publish to GitHub*** after one approval whose
  sentence says it leaves the machine and cannot be undone. Inside the
  application, as decided.
- **(b) alo writes `git push` and the person presses Enter** — ADR 0110's
  default for a command.

ADR 0110 is about **commands typed into a terminal**; pressing an application's
own button is not a command, so ADR 0110 does not decide this. VS Code runs
`git` itself either way. **Recommendation: (a)**, because it is the decision
(*inside the application*), and naming the repository `<their-name>.github.io`
makes GitHub Pages serve it with nothing further to switch on (read from
GitHub's documentation).

### Q6. How is an application's traffic shown?

- **(a) A line the turn raises** — *alo is using Firefox, which is sending to
  linkedin.com* — up from the press to the end of the turn. Declared, not
  measured: it shows that it is happening, not what was sent.
- **(b) Counted at the boundary** — the kernel programme that already watches a
  turn's sockets is attached to the application's own sandbox for the turn. It
  measures, and **it also measures everything else that application does** —
  the person's own tabs — which is watching the person.
- **(c) An instance of its own** — alo starts its own window of the application
  (its own Firefox profile, its own VS Code window) for the turn, and only that
  instance is counted. Measured, and nothing of the person's is watched.

**Recommendation: (c), with (a) as the line it draws.** It also answers Q8.

### Q7. May alo run what it built, to show it working?

The demo's *something simple* can be shown without running anything on the
machine — GitHub Pages serves it and Firefox opens it. **Recommendation: no
running, for the demo**; a preview or a server is ADR 0064's running code, at
the person's level, and is not needed here.

### Q8. What does a grant over Firefox reach?

- **(a) All of it**, as ADR 0040 says today.
- **(b) Only the window alo opened for the turn** (Q6 (c)).

**Recommendation: (b).** A person asking alo to post to LinkedIn has not offered
it their bank.

### Q9. One switch for *social media*, or one per site?

**Recommendation: one per site, and one account on it** — *post to LinkedIn as
Disan* — because *social media* is a category a person cannot check, and the
grants are enumerated. WhatsApp or Telegram, which the owner cited, are each
their own line when somebody asks for them.

### Q10. Where does a person let alo use an application?

- **(a) In Settings only** — a list of installed applications beside *Give alo
  a folder*, one row each, with the four lengths.
- **(b) Also when alo asks** — alo says *I would need to use VS Code for this*,
  and a surface **the compositor draws, not alo**, offers the grant with its
  length; the person's press there is the deliberate act. Nothing is granted by
  approving the request that needed it.
- **(c) Also at install** — the install sentence offers *and let alo use it*,
  unticked.

**Recommendation: (a) and (b).** (a) is the promise; (b) is what makes a demo
flow without the person leaving the conversation, and stays deliberate because
it is its own act on a surface alo cannot press. (c) merges two approvals into
one sentence, which is how a person ends up granting something they did not
read.

### Q11. For the applications nothing describes, is a picture plus keystrokes built at all?

- **(a) Not now** — alo says it cannot work inside them.
- **(b) Built, off by default, switched on by the person per application** —
  the screenshot-and-click line `docs/features.md` already promises, with the
  risk said: *alo will act on what it sees in pictures of this window, and may
  press the wrong thing.*

**Recommendation: (a) for Demo 2**, which needs none of them, and (b) as its
own piece of work afterwards. It is promised, so it is not a question of
whether; it is a question of when, which is the owner's.

## What this record does not do

It writes no code and declares no verb. It amends no contract: the argument kind
(fact 1), the typing mechanism (Q2), the new grant kind, and the narrower
Firefox grant (Q8) are each a contract change made in the stage that builds it,
after the answer. The stages are in
`docs/autonomy/demo-two-alo-inside-applications-plan.md`.
