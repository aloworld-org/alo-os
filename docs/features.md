# alo OS — features.md

Feature inventory: **what alo OS is**, in one list. **★** marks
differentiators — things no other operating system offers.

**There are no version tiers**, as of 2026-10-12. Every line here read
`[v0.01]`, `[v0.5]`, `[v1]` or `[v1.1]`, mapped to releases in `ROADMAP.md`,
and the owner removed all three hundred of them: the markers had stopped
ordering the work and started blocking it. [ADR
0105](decisions/0105-there-are-no-versions-until-there-is-a-launch.md) has the
measurement — three capabilities refused in one afternoon on the authority of a
marker nobody had revisited, one of them a thing the owner had decided a month
earlier.

**What this file no longer answers is *when*.** `ROADMAP.md` answers that, with
one list: what stands between today and a machine somebody outside this team can
use. Versions return when there is a launch to number.

Rule of the file, unchanged in the half that was working: **nothing gets built
that isn't listed here**, and nothing in Non-goals gets built at all. A feature
nobody wrote down is still not built — the cost of asking is a message, and the
cost of guessing wrong is a release nobody decided.

---

## The shell — what a person signs into (ADR 0002)

- **The colours come from a source this repository can read, and it is not CSS.** There is no CSS in alo OS and there will not be — the shell is native Rust (ADR 0002). But alo's palette currently *lives* in `alo-workplace`'s `tokens.css`, which a Rust compositor cannot read, so the source moves somewhere language-neutral and **generates** both: constants for this shell, and the custom properties the workspace's web client still needs. This ends CSS's authority over the palette rather than importing it, and nothing below can be drawn in alo's own colours until it exists
- Compositor: Wayland via Smithay, one display, keyboard and pointer
- Sign-in with an alo identity, and a local account that needs no tenant
- ★ The agent overlay: one key, from anywhere, with the current context offered and never harvested
- ★ **Asking alo opens a task window** — a request through the composer opens a window of its own where the person and the agent work together. It is a chat a person can follow, not a log they have to read: **the logs support the conversation, they are not the interface.** Six areas, each named because each is a promise. **Conversation** — the request, alo's replies and questions, and the person's answers. **Activity** — actions, progress, results and errors, with **Details** for the technical log, collapsed by default: a person who wants the machine's own words can have them and nobody has to read them to use this. **Model and location** — which model answered, and whether the processing was on this machine, on this network, or with a provider; law 1 is why this is a permanent area rather than a disclosure somebody goes looking for. **Permissions** — what this task may reach or change, and controls to grant or take back, visible for the life of the task so a person can see what they allowed an hour ago. **Stop** — asks for cancellation at once and **says when the work actually stopped, naming the changes already made**; a Stop that reported success while something was still running, or stayed silent about what had already changed, would be worse than no Stop. **Reply box** — answer, attach context, or send the task somewhere else
- ★ **A model changes for the next request, never midway through an action** — a change of model applies to the next request or a deliberate restart. Switching to an external provider makes any transfer of the conversation or of files **explicit**; it is never a consequence of changing a setting
- ★ **A task asks for each action by name** — when alo needs permission it proposes the specific action in the conversation, with **Allow once** and **Deny**. Broader access is a separate and deliberate choice, never a widening of this one — the capability model's *what a person approves is that sentence, and an approval is never a session*
- ★ **One window per task, and minimising keeps it running** — follow-ups stay in that window. Minimising puts its preview, its activity and its Stop control in the right-hand panel, so a person can set it going and watch it from the corner of their eye. *The four promises above were specified by the owner on 2026-10-12, all six areas named. This is the surface the composer opens, and the composer is the agent's only way in*
- ★ **A follow-up is sent with the conversation so far** — words typed in a task window after alo answered reach alo together with what was said before them in that window: the person's requests, alo's answers, and each proposed change with whether it was allowed, denied or not answered. Never what a verb read in an earlier turn, unless those words quoted it. Sent whole: alo leaves nothing out, and a conversation too long for the chosen model is said to be, in plain words, with where to continue. A reply sent while alo is still answering, or while a change waits for Allow once or Deny, is held under the conversation marked as waiting to be sent, goes by itself once it can, and can be taken back before then. Built only when the reply is sent, and with a provider the conversation leaves again with each follow-up, under the egress indicator. *Decided by the owner on 2026-10-28, ADR 0138*
- ★ **Closing a running task asks, and never orphans it** — closing the window of a task alo is still working on offers three named choices: **Stop and close**, **Keep running** — which leaves it in the right-hand panel, exactly where minimising puts it — and **Cancel**, which closes nothing. A running task is never silently killed by the close control and never left running with no window and no entry in the panel, because a person who cannot find a task cannot stop it. *Specified by the owner on 2026-10-12 with the task window's design. It is a separate promise from the one above rather than part of it: minimising and closing are two different acts, and only one of them was promised an answer* *Stop and close and Cancel were built on 2026-10-29 (ADR 0139); Keep running waits for the right-hand panel's place for a task, and until then is not offered rather than offered and broken.*
- Launcher and window management: open, focus, close, tile
- Lock screen, suspend and resume
- Multi-monitor, display scaling, hotplug
- Recovery and rollback screen — reachable when the workspace is not
- **Settings, as one place** — network, display, sound, printers, storage, keyboard, accounts, privacy, updates. Not a scattering of dialogues a person has to know the name of
- Accessibility: the AT-SPI tree the agent uses is the one a screen reader uses; EN 301 549 conformance is the same work, not extra work
- Multi-user on one machine, with per-person grants and no shared agent memory

**Making it yours**

The first thing anybody does with a new machine is make it look like theirs. It
is not a small feature: it is the moment a person decides whether the system is
theirs or the company's, and an operating system that cannot do it feels
unfinished however good the rest is.

**What they change is the surface they work on, not a picture behind it.** alo OS
ships no wallpaper: the canvas plane's own surface *is* the desktop, so making a
machine yours is done to that surface. [ADR
0075](decisions/0075-alo-os-has-no-wallpaper-the-canvass-own-surface-is-the-desktop.md),
accepted 2026-09-28, decides this and says why both places a picture could go are
worse than none.

- **Set the surface's material** — its colour and how it is finished; per display on a multi-monitor desk. *Read from a file or a rotating folder: **deleted rather than completed**, ADR 0075*
- The lock screen shows the same surface and no client's pixels. *An independent lock-screen image: **deleted rather than completed**, ADR 0075 — there is no longer a picture for it to be independent of*
- **Light and dark**, following the time of day if a person wants
- **Accent colour** — four designed hues, each with a value for a light ground and one for a dark, so it reads properly either way. The whole shell follows it, not one button (ADR 0010, as amended by [ADR 0067](decisions/0067-the-agents-colour-is-deep-teal-and-the-colour-it-vacates-is-given-back.md)). *This line said **five** until 2026-09-30. ADR 0010 designed five around a reserved terracotta; ADR 0067 reserved deep teal instead, released terracotta, and — amended 2026-09-27, after the owner was offered a deeper terracotta for light grounds and declined to invent one — settled that **four ship and no slot waits for a fifth**. An accent has to reach 4.5:1 on both grounds and terracotta on cream measures 2.87:1, so four hues that all read is the set, not four fifths of one*
- ★ **Deep teal is not one of them.** It means the agent and nothing else, so it is reserved rather than offered — an accent somebody could set to deep teal would take away the one signal that says the machine is acting on their behalf
- ★ **The agent is never signalled by colour alone** — deep teal always arrives with a mark and a word. A signal carried by hue fails for anybody who cannot distinguish that hue, and EN 301 549 does not allow colour to be the only means of conveying anything
- Text size and scaling, which is an accessibility setting as much as a taste one
- **A fresh machine already looks composed** — the plane ships with a surface of its own, so nobody meets a grey rectangle. *Wallpapers shipped in the image: **deleted rather than completed**, ADR 0075*
- Cursor size and colour; sounds, including silencing them
- ★ **Ask for it** — "make the surface warmer", "use dark after six" — the same propose-then-approve as any other change, because personalisation is exactly the low-stakes place people first learn to trust the agent
- Themes as a document, so a machine's look can be set once and applied across a fleet (ADR 0004)

## The ordinary things a desktop must do

Everything above is why alo OS is worth building. **This section is why it is
usable**, and it is where most of the engineering actually is. It is also the
whole product for anybody who declines the agent (ADR 0009) — so the bar is not
"good enough alongside an agent" but "worth choosing with the agent switched
off". An operating
system with a brilliant agent and no working Bluetooth is not a product, and a
feature list that skips copy and paste is not honest about the work.

Nothing here is a differentiator. All of it is required.

**Input and interaction**

- Copy, cut and paste — text, images and files, across applications
- Keyboard shortcuts, and a person can change them
- Window management: move, resize, snap, tile, minimise, maximise, close
- **Full screen** — one window covers the whole display, the Dock and the panel of put-aside windows with it, and gives the screen back when the person leaves it. An application may ask for it and is answered; a person may put a window into it without the application's help. *Added 2026-09-30 by the owner's direction, into the current release. It had never been listed at all: the line above promises **maximise**, which stops above the Dock and leaves it visible, and a client asking to fill the screen was met with silence rather than a refusal — measured the same day, which is what surfaced the omission. The tier is set here before any of it is built, because `CLAUDE.md` binds building to this file*
- **The alo Dock** — a band along an edge of the screen, above the canvas. It shows what you can open and brings what is already open into focus, and it does not move when the canvas does. Labels give way to icons where the short edge demands it ([ADR 0076](decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md))
- **Where the Dock goes** — **the bottom edge by default, and the person may choose bottom, left, right or top.** It works in both orientations rather than being a horizontal bar someone turned sideways. *Restored here on 2026-10-01 to match the owner's reversal of 2026-09-30, recorded in [ADR 0076](decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md): bottom is now the default rather than the only. It had been withdrawn below, and the withdrawal outlived the decision that made it — this file said the promise was gone while the record it cited said the promise was owed. **Built on 2026-10-10**: `alo_dock::Edge` has all four, `Layout::along` lays out each on its own terms, the draw places the bar and the corner on the chosen edge, `Changes` carries the person's choice and `dock.toml` keeps it, and `crate::words` declares the four names and their heading. What is not built is the panel a person clicks — the setting is reachable by a machine and has no row on a screen yet — and the overflow, which waits on the Dock being told what is open. The tier is set here before any of it is built, because `CLAUDE.md` binds building to this file*
- **Reaching the Dock over a full-screen window** — moving the pointer to the screen's edge reveals the covered surface over the content, and **the pointer can travel onto it and click without it disappearing on the way.** The keyboard reaches it too, and dismissing it from there puts focus back in the work rather than nowhere. **No reveal or hide delays:** a timer makes this depend on how fast somebody can move, which is the one thing a person with a tremor or a trackball cannot control. The same reveal serves the panel of put-aside windows at its own edge — one behaviour, one instance per surface — and it is the same behaviour at whichever edge the person put the surface on. *The clause that stood here until 2026-10-01 — `ADR 0076` is untouched by it: an edge is a fact about a surface, never a setting offered to a person — was true of that record as written and was reversed by the owner on 2026-09-30, the day after. The reveal never depended on it: it reveals a covered surface at that surface's own edge, and which edge that is was the only part the reversal changed.* *Added 2026-09-30 by the owner's direction, into the current release, together with the full-screen promise it stands on. It had lived only in [the Dock's design note](design/the-alo-dock.md) and a plan; the rules for it are built and tested in `alo-dock` with nothing yet calling them*
- **The top controls** — a band along the top of the screen holding the
  **active window's controls** and the way **back to the canvas**. Like the Dock
  it is above the canvas and does not move when the canvas does. **It gives way
  to a full-screen window** and is reached the same way the Dock is: by pointer
  or by keyboard, and **it stays while it is being used** rather than vanishing
  under the hand that reached for it. **A person who would rather it never
  hid may keep it visible.** *Added 2026-10-02 by the owner's decision. It had
  been drawn in [the regions note](design/the-regions-a-pointer-can-be-in.md)
  and listed here nowhere, which is the gap that mattered: `CLAUDE.md` binds
  building to this file, so three tasks arbitrated against a surface no lane was
  permitted to build and the pointer classifier could not land in any crate. The
  tier is set here before any of it is built, for the same reason. **The
  top-right corner is the panel's**, by the owner's ruling in that note — the
  top controls' region stops before it, and two surfaces claiming one corner was
  the fault that ruling settled.*
- **Which applications are on the Dock** — a pinned application is there
  whether or not it is open, and **stays there when every one of its windows is
  minimised, because it is pinned.** An unpinned application is there while it
  has at least one window that is not minimised, and **leaves the Dock when its
  last one is minimised** — its windows are still in the panel at the edge, each
  on its own. Each application appears **once**, however many windows it has.
  **"Not minimised" is not "on screen":** a window on another Place, outside the
  camera's view, or behind another one still counts. Pinned applications keep
  their chosen positions when they open; the rest follow in the order they were
  opened, and **focusing one never reorders the Dock.**
  *Added 2026-10-11 by the owner's ruling —
  [ADR 0103](decisions/0103-the-dock-more-apps-and-the-minimised-panel-are-three-connected-surfaces.md).
  It supersedes a rule that was never written down and was therefore never
  checkable: `alo_dock::Holding::showing` gives an icon to every application
  that has a window and asks nothing about that window's state, so an unpinned
  application whose windows were all put aside stayed on the bar. The tier is
  set here before any of it is built, because `CLAUDE.md` binds building to this
  file.*
- **What a Dock a person has never touched starts with** — **Files,
  Browser, Settings and the application market, in that order.** Files and
  Browser are resolved to applications that are **actually installed**;
  Settings and the market each open one of the compositor's own surfaces and
  **neither is an application** — neither has an identifier or a bus endpoint,
  and none is invented for either. So both are filled from the role itself and
  neither can be missing, which is what makes the market the one entry a
  machine with no network still has: it is the door to getting the rest.
  **An application that is not there is left out rather than drawn as a button
  that does nothing.** The
  defaults are written once, and only **after application discovery has
  succeeded** — an unfinished scan and a failed one are not evidence that an
  application is absent. After that the person owns the list: additions,
  removals, reordering and **a list they deliberately emptied** all survive a
  restart and an update, and **an empty list is never refilled with the
  defaults** on the grounds that it is empty.
  *Added 2026-10-11 by the owner's ruling, ADR 0103, into the current release.
  `alo_dock::pinning` answers this; `alo-desktop`'s `what_this_machine_offers`
  produces the discovery, and the Settings entry is a native action in the same
  saved list. **Nothing writes the list down yet**, so the defaults are worked
  out again on every boot — indistinguishable from correct until removal exists
  and wrong on the day it does. And **no machine has drawn it**: the producer
  answers `Failed` on a gate VM, which has no rented tool.* **Apps was a fourth default until
  2026-10-12 and the owner removed it**: it named an application store, nothing
  provides one, and *More apps* is not a substitute — that surface is the
  **pinned-and-open** grid its design specifies (`406-26320`), so an installed
  application that is neither pinned nor open does not belong in it. Removing
  the default neither creates nor cancels an application-store feature.
- ★ **The keyboard reaches every part of the screen, not only the
  windows** — **F6 and Shift+F6 move among the canvas, the Dock, the panel of
  put-aside windows, notifications and the status area**; Tab and Shift+Tab move
  among the controls inside whichever of those has the keyboard; Enter acts on
  the one that is focused; and Escape leaves.
  **The route is the design's, not a new invention** —
  `docs/design/the-shortcuts-and-the-edges.md` settles all four in its own
  table, and this line records that alo OS owes them rather than that somebody
  thought of them.
  **An area with nothing a person can act on is skipped**, so the cycle never
  parks the keyboard somewhere Tab does nothing; and **where focus was is given
  back** when the person leaves, rather than being dropped on the canvas.
  **This is what makes the rest of the Dock's keyboard promises true at all.**
  Its surfaces take the arrows, Enter and Escape already, and a person could
  not reach them without lifting a hand to a pointer: the opening move was
  missing. The same gap holds for the panel and the status area.
  *Added 2026-10-12 by the owner, into the current release, after it was
  measured that **F6 appears nowhere in this workspace but a key-name table**
  and that nothing anywhere models which area has the keyboard — so the design
  had specified the route and no part of it existed.*
- ★ **An icon's own menu, holding only what works** — the way a person
  reaches *pin*, *unpin* and *show windows* for one application, opened from
  its icon by the pointer, by the menu key and by a screen reader's own route.
  **It lists what this machine can actually do and nothing else.** An action
  with no performer is absent rather than greyed, which is the same rule as
  *omit unavailable applications; never show placeholders or non-working
  buttons* — and it is why this arrives with three items when
  `alo_dock::menu::What` names eleven: the other eight have nowhere to go yet,
  and each joins the menu in the change that makes it work.
  **The Dock never becomes a dashboard for the agent.** Nothing on it is
  permanent, `What::on_the_dock_permanently` is an empty list and a test says
  so, and *Let alo work in this window* is a window's menu rather than an
  icon's.
  *Added 2026-10-12 by the owner, into the current release, after it was
  measured that `docs/features.md` already promised the person owns their
  pinned list and **nothing in the product could pin or unpin anything** —
  every call outside a test was a fixture.*
- ★ **The Dock holds two kinds of thing, and the person owns both** —
  an **application entry**, which starts or focuses an application, and a
  **native-action entry**, which reaches one of alo's own surfaces. Settings is
  the first of the second kind. **The saved list says which a thing is** and
  each is carried out by its own handler, so a native action never has to wear
  an application's identifier to sit on the bar.
  **Native describes how it runs, not how long it stays.** A person may remove
  and reorder a native action exactly as they may an application, and
  additions, removals, order and a deliberately empty list all survive a
  restart and an update. **Defaults are applied at first setup only**, and an
  update never restores something a person removed.
  *Added 2026-10-12 by the owner's ruling, into the current release.
  `alo_dock::ADockEntry` is the distinction, the fresh defaults put Settings on
  the Dock as a native action, and a press or Enter on it opens Settings
  through the compositor rather than the launch path. **Surviving a restart is
  not built**: nothing writes the saved list, so none of removal, reordering or
  a deliberately empty list outlives a boot yet.*
- ★ **The Dock's own controls can be hidden** — and the way to bring
  them back is not on the Dock. The composer, *Show all*, *History* and *More
  apps* are visible by default and a person may hide any of them, with the
  choice persisted. Each may sit where its own layout needs rather than
  pretending to be an application pin.
  **Before any of them can be removed there is a discoverable way back that
  does not live on the Dock** — reachable by pointer, by keyboard and by a
  screen reader, a canvas action or its equal — because **a person who empties
  the Dock must still be able to restore it.** A control whose only route to
  itself is itself is a control a person can lose for good.
  And hiding a control never removes what it does: everything the Dock's
  controls reach stays reachable elsewhere. The design and the persistence
  contract are updated **together** before this is called complete.
  *Added 2026-10-12 by the owner's ruling, into the current release.*
- **Clicking an application** — the same wherever its icon is. **Nothing
  open:** it starts. **One window:** that window is focused, and restored first
  if it was minimised. **Several:** its own window picker opens, **minimised
  windows included and shown as minimised**, and choosing one focuses or
  restores that window. Focusing a window somewhere else **travels to it**, to
  the Place and position it already has — it is never silently moved. Clicking
  an application that is already focused **does not minimise it and does not
  open a second window**; *New window* stays a thing a person asks for.
  *Added 2026-10-11 by the owner's ruling, ADR 0103, into the current release.
  `alo_dock::clicking` decides this and **it is reached**:
  `Server::go_to_this_application` calls `what_a_press_on_an_icon_does` and acts
  on three of its five answers, from a pointer press and from the keyboard on
  the Dock's surfaces. **The note here said the opposite until 2026-10-14** —
  "no production caller reaches it: a press on an icon is taken and nothing is
  done with it" — which was true when it was written and had stopped being
  true. What is still owed is the window picker's travelling: `WhatAClickDoes`
  says when the canvas should travel to a window and the Dock ignores it
  deliberately, because the windows it is handed carry placeholder geometry.*
- **More apps** — an **overview of a person's applications**, not a
  spillover. Its control is **always on the bar**, so the route to it does not
  appear and vanish with the width of a display. It opens a **compact,
  non-modal popover anchored to that control** — above a bottom Dock, below a
  top one, inward from a side one — **inside the display and clear of the panel
  of minimised windows and the other fixed controls.** It never moves the panel
  to make room, and it never blurs or disables the canvas behind it. Inside: a
  **search field** and a **labelled icon grid** — pinned applications first in
  their saved order, then every other **currently open** application
  **including those whose windows are all minimised**, each application exactly
  once. Applications that are merely installed are **not** here; they are
  reached through Apps or the alo Bar. Search filters across both groups,
  keeps their order, and says plainly when nothing matches. The popover **grows
  to a bounded size and then scrolls its grid**, with the search field staying
  still — it never becomes a full-height sidebar. **Escape dismisses a window
  picker first and then the popover**, a press outside dismisses it, and the
  control itself closes it. A person using a keyboard can reach the search
  field, move through the grid and open an application, with the focused item
  scrolled into view; dismissing without choosing puts focus back on the
  control. Names and states reach assistive technology.
  *Added 2026-10-11 by the owner's ruling, ADR 0103, into the current release.
  It supersedes the line that stood here for five hours, which said this surface
  lists **the applications that did not fit** — written the same day, from
  [ADR 0102](decisions/0102-the-minimised-window-panel-and-the-docks-overflow-are-two-surfaces.md).
  That made the surface's existence depend on arithmetic. **Built**: the
  control, a list-shaped popover, its surface, its placement clear of the panel,
  and dismissal by an outside press. **Not built**: the grid, the search field,
  the window picker, every keyboard route, and Escape. The owner's note that
  **the list-style reference is historical input rather than the final design**
  is why the visual layout is not settled and the Figma file is owed an update.*

  *Corrected 2026-10-14: the "**Not built**" list above named five things —
  the grid, the search field, the window picker, every keyboard route, and
  Escape — and **all five are built and wired.** Measured by following each to
  a production caller rather than to a definition, which is the distinction
  that makes this checkable: `alo_dock::more_apps::TheOverview` is drawn by
  `dock_raster`; the search field is typed into by
  `a_key_types_into_the_search_field`; `alo_dock::picking::the_pickers_panel`
  is drawn by the same raster; `when_escape_is_pressed` and the arrow routes
  are reached from `a_key_reaches_the_docks_surfaces`; and that router's own
  caller is `<Desk as LoopInput>::dispatch` in `direct_desktop`, which is the
  frame loop a running machine is in.*

  *Left in place because a stale "not built" is how a thing gets built twice,
  and this one nearly managed it: a lane reading that list would have rebuilt
  the grid, the field and the keyboard. **The first attempt to measure it said
  the router had zero callers**, from a `grep -v` that excluded every line
  containing the function's own name — a check that could only ever answer
  zero. The method here names a caller for each item, so an item that loses one
  reads as absent rather than as a number nobody can check.*
- **Minimising is about one window** — it takes that window off the
  canvas, **keeps its Place, its position, its size and what is needed to bring
  it back**, puts an item in the panel at the edge, and **works out again
  whether its application still belongs on the Dock.** It never closes a window
  or throws away its work, and it is **not** *Compact*: a compacted window is
  still on the canvas. So an unpinned application with two windows keeps its
  icon when one is minimised and loses it when the second is, both windows stay
  separately in the panel, restoring either brings the icon back, and a pinned
  application keeps its icon the whole way through.
  *Added 2026-10-11 by the owner's ruling, ADR 0103, into the current release.
  The compositor half is built and contracted in
  [native-window-minimize](contracts/native-window-minimize.md); what is new
  here is the link to the Dock, which nothing computes.*
- The dock's size, and whether it hides when a window needs the room

**Restored, not withdrawn.** *The person decides where it goes — bottom, left, right or top, chosen in Settings* was withdrawn by [ADR 0076](decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md) on 2026-09-29 and **the owner reversed that within a day**. It is a promise again, at ****, in the list above. Its **** descendant — *per display, so the dock can sit along the bottom of the laptop and down the side of the external screen* — keeps the tier it had; the reversal moved the edge choice and left per-display placement where it was.

*This paragraph said `withdrawn` for two days after the decision it cited had been reversed. A withdrawal is the one kind of entry that has to be re-read when a record changes, because every other kind describes work that is owed and this kind describes work nobody is allowed to do — so it is the entry a reader trusts to mean stop, and the only one whose staleness silently forbids something the owner has asked for.*
- ★ **Divide the screen** — drag a window to an edge to take half, a corner to take a quarter, or split what is already open with the keyboard. The split holds while you work: resizing one side resizes its neighbour rather than overlapping it
- Remember a split, so returning to a pair of windows restores the arrangement rather than the last position of each
- Splitting works on an external display independently of the laptop's own

**Screens and desks**

*Added 2026-10-08 by the owner's direction, into the current release:* **"I want
all the discussed features of the monitors to be in the next release after this
one so you tasks today is to do all the features we have agreed on"**, following
**"users can name the monitors by any name like 'cat'"**. *The tier is set here
before one line of any of it is written, because `CLAUDE.md` binds building to
this file — the same way **Full screen** was added on 2026-09-30 and
**Notifications** was moved into this release on 2026-10-02. The design is
settled in [ADR 0099](decisions/0099-a-screen-carries-the-name-its-person-gave-it.md)
and the work is ordered in
[`docs/autonomy/more-than-one-display-plan.md`](autonomy/more-than-one-display-plan.md).*

*What makes these **scope** rather than sub-work, in one sentence: they add
promises to a person. Somebody who can call a monitor `cat` has been offered
something nobody was offered before, so not one line of it is written before this
file carries it with a tier.*

- ★ **A desk comes back when you arrive at it** — the screens in front of you, how they are arranged, how large things are on each, and where your windows were. Plug in at one desk and that desk returns; move to another and that one does. Nothing to choose, and nothing to set up twice. *The engine is built and tested in `alo-displays` — a separate arrangement per set of screens, with `three_sets_of_screens_remembered_apart` proving it — and has never had a caller. This line is what lets it reach a machine*
- ★ **You name your screens, and the name is yours** — any name you like, typed on the screen it belongs to. `cat` is a valid name for a monitor. Two screens may carry the same name, because forbidding that would take a choice away for the machine's convenience. **A name is a label, never a key:** the machine tells screens apart by what they report and which socket they are in, and shows the person's word. A person's own name is never translated, as a screen's make and model are not (ADR 0099)
- **Screens are named, not numbered, by default** — *the laptop*, *the big one on the right*. A number is the one thing nobody can match to the screen in front of them, which is the step people actually fail at
- **alo OS says which desk it thinks you are at, once** — and says nothing on the ordinary morning. *The sentences are written and translated already in `alo-displays`: your screens are arranged the way you last left them; this screen has not been used with this machine before; your screens have changed since you arranged them and your arrangement is still here; the arrangement you made at your other desk is still here for when you are back at it. `notes.rs` holds the rule that there is no note for the ordinary morning, because a system that announces every screen every time is one whose announcements nobody reads*
- ★ **Show me where** — when a screen is new, alo puts it beside the others and says so. If that guess is wrong, a card on the new screen is pushed off the edge towards the screen it actually sits beside, and **that gesture is the arrangement**. No map of grey rectangles to drag. Doing nothing keeps the guess and leaves the card offered where the notes are, so somebody who only wants to work is never stopped and never asked twice
- **Unplug and nothing is lost; plug back in and nothing moved** — the windows on a screen that goes come home visibly, with how many and from where, and an undo. Plugging it back puts them where they were
- **How large things are, per screen, in words** — *smaller*, *just right*, *bigger*, previewed on the screen being changed, with no percentages. **It cannot be offered until a display's size reaches an application's window and not only alo's own dock and panel** — measured 2026-10-08, and offering it before then would be a control that lies
- **Two windows onto one desk** — point both screens at the same part of your desk, or at different parts, and either is ordinary. *The architecture is already this: `more-than-one-display-plan.md` task 7 gives each display its own camera onto one canvas. The gesture is the new promise, and it stays at `` so it cannot delay the lines above*
- **Ask alo to use a screen by its name** — *put the chat on cat*. Kept at `` because an agent verb is a public contract surface, typed and enumerated under ADR 0001, and a contract is not changed to make a feature land sooner
- Switching between windows, and between applications
- Drag and drop between applications
- Right-click context menus, wherever a person expects one
- Touchpad gestures: scroll, zoom, swipe between workspaces
- **Keyboard layouts, switched easily** — and dead keys and a compose key that work. "Müller" and "Liège" are test cases in a European product, not edge cases
- Input methods for non-Latin scripts
- Virtual desktops
- Clipboard history, on the machine and never synced to any server. Sharing one copied item directly between a person's own paired devices is an opt-in: end to end encrypted, gone after about two minutes, never readable by the agent (ADR 0064)

**Capture**

- Screenshots: whole screen, one window, a selected region — to a file or the clipboard
- Annotate a screenshot without opening anything else
- **Screen recording**, with audio, to a file
- Screen sharing for calls
- ★ **A visible indicator whenever the screen, camera or microphone is in use** — by any application, including ours. Law 1 is about egress; this is the same instinct applied to the room you are sitting in

**Desktop**

- Notifications, with do-not-disturb *This was `` until 2026-10-02, when the owner moved it into the current release so a promise already at `` could be kept. The canvas asks that a recovery which moves a frame **shows** the move — *a frame that relocated itself silently is a person's arrangement edited without them* — and nothing else on this machine can tell a person anything, so the clause was unkeepable while this line sat higher. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed. The do-not-disturb half is already built — `alo-notifying`'s `quiet.rs` holds four reasons, two the person's and two the machine's, with the screen winning first so nothing is shown into a recording.*
- Status area: clock, battery, network, volume, brightness. **It is owed a location.** [ADR 0076](decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md) took *at the far end of the dock, wherever the dock is* off this promise — a clock is not something a person opens or brings into focus, so it is not the Dock's — and handed *where does it go* to [the shell's plan](autonomy/the-smallest-canvas-worth-showing.md). The promise stands; what it is owed first is a place
- ★ The egress indicator sits at the far end of the dock, above it, so "nothing has left this machine" sits where a person already glances rather than somewhere they must learn to look. **It did not move with the rest of the status area:** it has a corner of its own, and drawing it is the one part of this that is built
- **A person's home folder is not empty when they first sign in** — alo OS makes **Documents, Downloads and Pictures**, in the language the machine was set up in, and writes the record other software reads to find them. *Added 2026-10-24 by the owner's decision, after the absence was found from the other end: the screen for granting the agent a folder was being designed, and a fresh machine's home folder held nothing to grant. Windows, macOS and every Linux desktop make these when an account is created, and this file's own rule is to be ordinary everywhere it is not the point. **Desktop is deliberately not among them** — a Desktop folder exists elsewhere to hold the icons those systems put on the desktop, and this one has a canvas, so it would be a folder whose only purpose is a feature alo OS does not have. Music, Videos, Templates and Public wait until something on this machine plays, uses or shares from them. The names are translated, which makes them the one place a translator's mistake becomes a directory rather than a sentence — so `alo-their-folders` refuses a name that would be hidden, that is a path, or that would break the record, and makes the folders it can rather than costing a person all three*
- A file manager, with trash, and archives that open
- USB drives and external storage that appear when plugged in
- File associations — what opens what, changeable by a person
- A text editor and an image viewer, so a fresh machine is not helpless
- **A terminal.** Law 2 forbids the *agent* running arbitrary commands; it says nothing about a person, and an operating system that does not trust its owner with a shell is a toy
- ★ **Ask alo from the command line, and it acts.** One command, typed at your
  own prompt like any other: it answers a question, and when you ask for
  something done it finds what you meant, **proposes the command**, says **what
  grant the work needs**, and waits for your approval before anything happens.
  *Ask alo means the same thing here as it does in the dock and the composer —
  it acts. A terminal where alo could only chat would be the one place in the
  product where asking did nothing.* **There is one command, not two**: an
  answer-only road beside an acting one makes a person learn which is the real
  one. **And you never say which you meant** — alo works out whether a sentence
  wants an answer or a change. Nothing acts without the approval, so a question
  read as a request costs you reading one sentence rather than an action
  **Decided by the owner 2026-10-18**
  ([ADR 0124](decisions/0124-one-command-asks-alo-from-the-terminal-and-asking-means-acting.md)).
  **Built 2026-10-21, and this line said *Not built* until 2026-10-22.** All
  three of the owner's rulings hold: there is **one** command; it **works out
  which you meant**, by asking the model on this machine to answer with one word
  and treating anything else as *could not tell*; and an acting sentence goes to
  **the agent's own door** — `alo-asking.socket`, which systemd opens as the
  agent (ADR 0109) — rather than being answered.
  *The road and not the mention count: `main.rs` matches `WhatItWants::Acting`
  and returns `the_agent_was_asked`, which connects to the agent's socket and
  reads what came back. The 2026-10-18 measurement counted **mentions**, and
  mentions and roads are different populations — a crate can name a thing
  hundreds of times with nothing calling it, which is this repository's own
  commonest fault. This says which line dispatches.*
  **Nothing here approves anything, deliberately.** The command writes a
  sentence and reads what came back; the proposal is the daemon's to put and the
  approval is the person's to give. ADR 0124's own words: *the classifier is not
  what protects the person — the approval is.*
  **Not built**: nothing of this promise. What is left is that **nobody has seen
  it on a machine**, which is true of six other starred terminal lines and is
  waiting on an image.
  *The measurement that said* zero *was correct on 2026-10-18 and false from the
  moment the road landed. Nothing re-read it, which is why a measurement written
  into a promise has to carry its date — and this one did, which is how it was
  caught.*
- ★ **And you decide whether alo may use it.** **alo can open a terminal for
  you** — that is an ordinary thing to ask an assistant for, and it hands alo
  nothing: a terminal window opens onto a prompt waiting for a keystroke, and
  nothing lets alo type into it, so what it has opened is a window **only you
  can use**. Whether it may **run commands** in one is the part you decide. Off
  when a machine arrives, and yours to turn on. The
  sentence you turn it on with says what it costs, in the words the decision
  that used to forbid it wrote: *a shell is the whole machine by another road* —
  which is true of whoever can **drive** a shell, and is why opening the window
  is a different question from running things in it.
  Revocable in one action, and it expires. Whatever you choose, every command
  and its output are written to the record, your files are snapshotted before
  each run so it can be undone, and anything that leaves the machine is shown —
  **none of those three is a setting**, because none of them takes a choice away.
  *Added 2026-10-14 by the owner's ruling —* "we are not dictators so the users
  should be able to choose if the agent can use the terminal or not" *— and
  recorded as
  [ADR 0111](decisions/0111-the-person-chooses-whether-their-agent-may-use-the-terminal.md),
  which supersedes
  [ADR 0043](decisions/0043-the-terminal-is-a-persons-and-never-an-agents.md).
  That record walled the terminal off entirely, and three things had made it
  untenable: law 5 reached `CLAUDE.md` after it; ADR 0064 replaced its stated
  foundation six days later and permits an agent to run **any** command at full
  trust, so the wall forbade the lesser act while the greater was allowed; and
  ADR 0079 already permits handing an agent a whole remote machine. **Not
  built**: the capability, its surface and its refusal path are all owed, and
  `alo-capability` refuses an agent a terminal three ways today — including the
  opening the owner has now permitted.
  **The split between the two halves is the owner's second ruling of the same
  day**, in these words:* "agent can open terminal window application".
- ★ **You can always tell who is speaking.** A narrow gutter beside the
  transcript belongs to the terminal, and **no program can write into it** — it
  says `PROGRAM · cargo test`, `ALO · response`, `SYSTEM · observation` or
  `YOU · submitted request`, once per region rather than per line, in words
  rather than in colour alone. *Any program can print "Listening verified" —
  those are the sentences carrying the trust here — so who said something has to
  be answered somewhere a program cannot reach.* **Rich formatting never
  changes it**: a table drawn from `cargo test`'s own bytes is still the
  program's. And the promise is the narrow one, because no picture authenticates
  itself: **programs cannot impersonate the terminal's own provenance inside the
  live terminal**, and a screenshot is a picture like any other.
  [ADR 0113](decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)
  **Built 2026-10-14**: the column — 96 wide, measured off the design at two
  nodes and taken out of the cells' room before they are laid out rather than
  drawn over them — the four origins, and the word drawn in alo's own face
  where the transcript is mono. **The words are drawn as the design draws
  them** — `you`, `alo`, `system`, `program`, lowercase — rather than in the
  capitals this entry and ADR 0113 write them in: the decision says which
  origins exist and that the gutter names them in words, and the design file is
  the visual truth for how a word is set. `docs/design/the-terminal-window.md`
  carries that reading and the measurement behind it. The guarantee is tested on the **pixels**: a
  program printing `system`, `alo`, or *Listening verified* leaves the gutter
  byte-for-byte unchanged, and no road from text to an origin exists for one to
  be added later without a test refusing it. **The live terminal shows one
  origin, `program`**, because a pseudoterminal is the only thing that writes to
  a transcript today — including the person's own typing, which the shell echoes
  back and which is therefore the program's bytes. The other three become
  reachable when alo itself writes into a transcript.
  **The source qualifier is built, and this said *Not built* until 2026-10-22.**
  It is in the design's own form rather than ADR 0113's: the design puts the
  detail **inside the region as a caption** instead of in the 96-wide column, and
  that caption is drawn — `crates/alo-shell/src/the_terminals_cells_are_drawn.rs`
  fills what printed a result into *Source: {what} · Not independently verified*
  and draws it under the summary, in the **program's** region, which the design's
  third rule asks for and a test holds.
  *Both this line and `docs/design/the-terminal-window.md` said it was owed, and
  both gave the same reason — that it **belongs with rich results, which are not
  built**. Rich results were built on 2026-10-18: the test result #761, the
  benchmark #803 and the file diff #804. Two documents went stale together
  because they shared one premise, which is why a note that defers to another
  promise has to name it: when that promise lands, the deferral is what somebody
  has to come back and re-read.*
  **What was not checked**: whether the caption sits at the design's measured
  inset — 17 in, 862×22, 38 below the summary's row. The content and the region
  are right; the geometry is the next thing to measure off the frame, and it is
  not claimed here.
- ★ **And alo says what it measured apart from what it worked out.** One rule
  over everything below, because it is the same rule four of them would
  otherwise repeat: *blocked reading from the keyboard* is a measurement and
  *waiting for your password* is a reading of it, and they never look alike. A
  tool that blends them teaches you to distrust both
- ★ **Why is it waiting?** A command looks stuck; ask, and get **waiting for the
  keyboard**, **for the network**, **for another process or a lock**, **still
  computing**, or **unknown with the evidence available** — then *show the
  prompt*, *inspect the connection*, *reveal what is blocking it*, or *stop*.
  **Instead of killing something that was nearly done.** *Measured 2026-10-14:
  five states from four unprivileged reads — `/proc/PID/stat` field 3 for the
  state letter, `/proc/PID/wchan` for the kernel function, `/proc/PID/fd/0` for
  whether stdin is your terminal or a pipe. Observed with a control:
  `sleep` → `S`/`hrtimer_nanosleep`, a pipe reader → `S`/`wait_for_partner`, a
  busy loop → `R`/`wchan=0`, so **still computing is distinguishable rather
  than being the absence of evidence**. The fifth state falls out of the
  mechanism — a `wchan` is a kernel symbol and any table we write is finite —
  rather than being added for honesty's sake. **This one needs no kernel
  boundary**, which makes it the first buildable*
  **Built 2026-10-14**: the five answers, from the three unprivileged reads,
  about **the command the terminal has in front** rather than about the shell —
  asking about the shell answers *waiting for the keyboard* every time,
  including while a command runs, which is the one case anybody asks in.
  **Two things the measurement overturned**, both now in `docs/quirks.md`:
  `wchan` **cannot** tell a terminal read from a socket read — both are
  `wait_woken` on 7.0.0 — so what the command reads from is what separates the
  keyboard from the network, and the symbol names move between kernels (the
  `wait_for_partner` recorded above is this kernel's `anon_pipe_read`); and a
  first version of the table answered *waiting for another program* about
  `sleep`, because it reached for the file when the kernel had already said the
  command was waiting on a clock. It now consults the file only where the
  kernel has said nothing or said only *parked*. Checked against real processes
  on a real kernel, each with its control.
  **The asking is wired, 2026-10-17.**
  `AShellOnAPseudoterminal::why_the_command_in_front_is_waiting` is the
  question, and what was missing was only the joining: which process the
  terminal has in front, what the machine says about it, and what that reads as
  were three public pieces built and tested weeks apart that **nothing called
  together** — so the promise's *ask* had no ask. Measured on a real
  pseudoterminal: `cat` answers *waiting for the keyboard*, and when it finishes
  there is nothing waiting again. Mutation-proved — made to ask about the shell
  instead, which is the fault its own documentation warns about, and both tests
  fail. It carries which process the answer is about, because all four actions
  below act on something.
  **Still not built**: *show the prompt*, *inspect the connection*, *reveal what
  is blocking it* and *stop*. Each is two halves — a mechanism answerable from
  `/proc` that needs no decision, and a presentation that is a surface and needs
  one. **No surface asks yet**, which is why the answer is public API rather than
  anything a screen depends on.
  **Three of the four mechanisms are built, 2026-10-17**, and the fourth
  already existed. *Stop* sends `SIGINT` to the foreground process group —
  what this terminal's own interrupt key does, and deliberately not a kill,
  because the promise above is *instead of killing something that was nearly
  done*; to the group so a pipeline stops rather than half of one; and refusing
  the shell, since a person at a prompt has nothing to stop. *Reveal what is
  blocking it* names the process holding a lock, feeding an idle pipe, or being
  waited for as a child. *Inspect the connection* says which socket the command
  reads and where it goes. *Show the prompt* is `back_to_the_newest`, which
  scrolling already gave.
  **Two encodings were measured before they were written**, both wrong in ways
  that look right: `/proc/locks` pairs a holder with its blocked waiter and the
  `->` shifts every field by one, so counting positions reads a lock mode as a
  pid; and `/proc/net/tcp` writes the address little-endian and the port
  big-endian, so reading either the other way gives a real address or a real
  port that is somewhere else. The inode is token ten although the header lists
  eleven columns, because two pairs of columns share a token — counting them
  reads the uid as an inode. Both wrong orders are controls in the tests.
  **A control test found a real fault**: the pipe reader named a process about a
  busy loop whose input happened to be an inherited pipe. *Who holds this thing*
  is not *what is holding this process up*, so a process the kernel has on a
  processor is now blocked by nothing, asked before any reader runs.
  **Still owed: the surface**, and the mapping — five answers, four actions, and
  this list pairs none of them, which is the owner's to settle rather than a
  lane's to infer.
- ★ **Try a command without letting it touch anything.** Paste the install
  script from a README and run it in a box that sees the project and nothing
  else, with no network except what you approved, thrown away after. **You ask
  for it by putting one word first — `try ./install.sh` — and alo offers it
  when it sees a command worth boxing**, because the person who did not think to
  ask is the one who gets hurt. *There is no setting that boxes things for you:
  a protection with a cost is offered, never imposed (law 5).* **Decided by the
  owner 2026-10-18**; the word `try` landed the same day (#817) and the kernel
  bounding it uses on 2026-09-04. **alo offering is not built.** See what it
  **tried** to do, then keep it or discard it. *This is the sealed box of
  [ADR 0064](decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md),
  which was a decision and not yet a thing: the three level names appeared in
  **zero** Rust files, measured with a control, though cgroups, namespaces and
  sandbox machinery are all present*
  **The choice itself is built, 2026-10-15**: `alo_capability::HowCodeRuns`
  carries the three levels with the sealed box as the default, the plain-words
  cost of full trust as ADR 0064 wrote it, and the three protections that stay
  on at every level — the record, the snapshot before each run, and the egress
  indicator — as a list each level must answer the same, so *at every level,
  always on* is a test rather than a sentence. Law 5's real exposure is a later
  change adding a wall, so a guard reads the module's own source and refuses a
  road that could decline a level; it was checked by planting one.
  **The box is built too, and this line said otherwise until 2026-10-17.** It
  read *still not built: the box … that boundary does not exist*, and both halves
  were false when written: the boundary landed on 2026-09-04
  (`alo-bounding`, `alo-bounding-kernel`, `alo-boundaryd`) and the box landed in
  #729 on **2026-10-16**, the day before. `alo_running::a_run_inside_a_box`
  makes a control group, puts its id and the places it may reach into the
  boundary's map, starts `alo-enter-the-box`, waits, and removes both **on every
  road out including the ones where the command never started**. Measured on
  this lane's VM on 2026-10-17: 20 tests pass as the person, and the two that
  need root pass as root — a command inside the box reads the file it was given,
  and `cat` is refused the file it was not. The tests **fail rather than skip**
  when `/sys/fs/bpf` is not writable, so a green run on a machine that could not
  test it is not available.
  **The caller and the surface this entry said were owed both exist**, and the
  paragraph that said otherwise stood until 2026-10-21. It was measured on
  2026-10-17 and was true then; #817 landed `try` on 2026-10-18 after the owner
  chose the word, and #818 added `--only-here`. Re-measured 2026-10-21:

  | what the entry said | what is there now |
  |---|---|
  | no crate depends on `alo-running` | `crates/alo-try` does, and is the only one |
  | `HowCodeRuns` is read nowhere outside it | `crates/alo-try/src/main.rs` reads it |
  | nothing anywhere asks for a run | `alo-try` asks, through `WhereCommandsRun::ran` |
  | the surface is the owner's — no prefix, no command, no chord | the owner chose the prefix: a person types `try ./install.sh`, and `try` is a shell **function** in `crates/alo-terminal/shell-integration/alo.bash`, guarded on `command -v alo-try` so a `try` that could only fail is never defined |

  **One claim in it is still true, and it is the one worth keeping.** The level a
  person picks still decides nothing: `HowCodeRuns` is named in 11 files across
  exactly three crates — `alo-running`, `alo-capability` and `alo-try` — and
  **none of them stores or reads a person's choice**, which is the same gap the
  first terminal promise in this list carries. `try` does not close it and is not
  meant to: it passes `HowCodeRuns::InASealedBox` as a literal, deliberately,
  because *a person who typed `try` asked for a sealed box for this one command,
  whatever their setting says about how code runs by default*.

  **What is owed now is neither a caller nor a prefix.** The shell integration
  that defines `try` is `include_str!`'d into the binary and **a person installs
  it by hand**;
  [ADR 0123](decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
  term 6, accepted by the owner on 2026-10-18, promises *the snippet, the
  sentence saying plainly what it reveals, **and a one-press way to add it***.
  The one-press way is not built, and `crates/alo-terminal/src/the_snippet_a_person_installs.rs`
  says so in its own header rather than leaving it to be discovered: what surface
  offers it is a decision nobody has made, and a file that installed itself would
  break the term it was written for. **Searched 2026-10-21 under ten spellings —
  `one-press`, `one press`, `offers to add`, `install the snippet`, `bashrc`,
  `shell startup` and four more — it appears nowhere in this file, in
  `the-terminal-in-full-plan.md` or in the Mac lane's order**, so it is an
  accepted promise that no task owns. It is the owner's, for the reason the old
  paragraph gave about a different thing: `CLAUDE.md` sends a silent design to
  the owner rather than to a lane.
- ★ **Scope one command without changing a setting.** `--only-here` so it may
  not write outside this folder; `--offline` so it may not reach the network,
  and the indicator shows that it did not. **A process cannot credibly restrict
  itself** — the kernel has to, which is why no other terminal offers this as a
  prefix
- ★ **Find a command by what it did.** *The command that made the PDF. The last
  build that passed. What started this server.* Results link to the commands,
  outputs, files and processes themselves, and **where nothing connects them it
  says so** rather than inventing a link. *Shell history is a list of strings;
  this is a different thing to search. **Opt in, per session or per project** —
  because *it forgets everything that was not an agent*, further down this file,
  is not spent on this: nothing is watched, outcomes are kept because you asked
  for them*
- ★ **Scrollback you can move along, not only read.** In a session you asked to
  be able to undo, every command is a place you can return to: scroll up, choose
  one, and your files are as they were before it ran. *In every other terminal scrollback is a
  record of text. **A process cannot snapshot a filesystem it does not own**,
  and alo OS picks that filesystem at install so that it can*
  **Moving along it is built, 2026-10-15**: the transcript scrolls, by wheel —
  three lines a notch, pushing away goes back — and typing puts the newest
  output back on screen, which is the design's *current input stays reachable*.
  The grid was already asked to remember a thousand lines and **none of them
  could be reached**: a terminal that remembers and cannot show has forgotten,
  as far as a person is concerned. A notch that moves nothing is not consumed,
  so a terminal at the end of its scrollback does not swallow scrolls.
  **Narrowed on 2026-10-19, by the owner**
  ([ADR 0127](decisions/0127-a-persons-own-commands-are-bracketed-in-a-session-they-asked-for.md)):
  this said *every command in the session*, with no asking. A snapshot before
  every command means one before `ls` and one before `cd` — alo cannot tell a
  changing command from a reading one — and ADR 0045's window of *fifty changing
  turns* would then be spent before lunch, so the morning's real mistake would
  have expired by the afternoon. **It is opt in per session**, which is what the
  outcome search above it already is. The sentence is narrowed here rather than
  only in the ADR, because a promise narrowed in a decision and left standing in
  this file is the fault law 5's own paragraph was added for.
  **The snapshot half is built, 2026-10-22**
  ([ADR 0130](decisions/0130-what-takes-the-bracket-before-a-persons-own-command.md)):
  the person's own shell asks this machine before each command and the machine
  takes a read-only copy of their home.
  **And *your files are as they were* is built, 2026-10-22** — `alo-back`, in
  the terminal: the places kept are listed newest first, and naming one and a
  folder puts that folder back as it was, with what is in it now moved beside it
  rather than deleted. **Narrowed a second time here, by
  [ADR 0127](decisions/0127-a-persons-own-commands-are-bracketed-in-a-session-they-asked-for.md)
  term 2**, which is the narrowing that decision said would be carried into this
  file by the change that built it: *your files are as they were* is, honestly,
  **the folder you picked, now — or all of them, at your next sign-in.** Nothing
  can swap a directory somebody is living in, so the whole home waits for a
  sign-in; a privileged helper that did it anyway was refused outright, as a new
  privileged path whose job would be deleting a person's home.
  **Not built**: *scroll up, choose one*. The choosing is by naming a place, not
  yet by picking a row of scrollback, and the reason is in ADR 0130: **alo never
  learns which command a place belongs to**, deliberately, so a place carries a
  moment and no command. Pairing a row to a place is the piece of this line that
  is left, and it is a surface rather than a mechanism — everything underneath
  it now works.
  [ADR 0132](decisions/0132-how-a-person-picks-a-place-to-return-to-from-the-scrollback.md)
  proposes it with three roads and what each costs, measured: it is a decision
  rather than a commit because every road adds to a published contract, and the
  cheapest-looking one is the one `docs/contracts/shell-integration-marks.md`
  makes a promise about.
- ★ **Bisect the machine, not the repository.** *It worked yesterday* becomes a
  command: walk the snapshots to find when something broke, including everything
  version control never held — installed packages, caches, configuration, a
  system library that moved under you
- ★ **Output that is not stuck in a character grid.** A benchmark draws a chart
  you can zoom; a long error becomes something you keep beside your editor and
  come back to tomorrow. *Other terminals can show an inline image. None can
  make it a thing on your desk that outlives the scroll, because the compositor
  is not theirs*
  **All three rich views the design specifies are built**: the test result on
  2026-10-17, the benchmark in #803 and the file diff in #804 on 2026-10-18.
  **None has been seen on a machine.** *This read* the first rich view is built
  *until 2026-10-21.* The first was a run of tests recognised in what
  a program printed, drawn as the design's own frame draws it — the summary from
  the numbers, then *Source: … · Not independently verified*. Three of that
  frame's rules are tests rather than sentences: every accessor is named
  `it_said_…` because *formatted output is not system evidence*; the program's
  own text is never replaced, because *text reveals original bytes*; and the two
  lines extend the **program's** gutter region rather than alo's, because
  *PROGRAM includes parsed rich results*. **Untrusted is the load-bearing word**,
  so a line carrying any control character is refused whole, a count that will
  not parse is not a count, a sum that would overflow refuses the result, and
  neither the summary nor the provenance can be forged by a program printing
  them — both are built from parsed numbers and this repository's own keys.
  **Recognising nothing is the ordinary case**, which the design says by drawing
  its text fallback first. **Not built**: the three *Ask alo* actions, which are
  the agent's road; and *a thing on your desk that outlives the scroll*, which
  is a window rather than a line. *Of those two, the three actions' **words**
  are known — `Text`, `Expand` and `Open on canvas`, read off a render on
  2026-10-18 because they are in no layer name; what is not built is the road
  from the terminal to the agent, which nothing constructs.*

  **This entry also carried a second claim until 2026-10-21** — *not specified
  anywhere: the benchmark and diff views are instances of components no snapshot
  in this repository contains, so their insides cannot be checked against
  anything.* **It stopped being true on 2026-10-18**, and both views were built
  the same day.
  The components are committed as
  `docs/design/figma-snapshot/the-terminal-view-components.xml` — 6,888 bytes,
  holding `479:28967` *Terminal / View / Benchmark comparison* and `479:28986`
  *Terminal / View / File diff*, both 990×320. Three figures in it that a
  screenshot would have lost: the chart's bars are **one scale**, 2 px per
  millisecond from a shared origin, so a proportion and never two constants; the
  diff's removal marker is **U+2212 MINUS SIGN** and not a hyphen; and the
  action labels are in no layer name. See `docs/design/the-terminal-window.md`
- ★ **Hand somebody a session they can replay.** Not a transcript — the
  commands **and what each one changed**, replayed on their machine. The
  reproducible bug report.
  **Two halves, two decisions**: the commands and their exit codes come from
  [ADR 0123](decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
  (**accepted 2026-10-18**), and *what each one changed* is
  [ADR 0045](decisions/0045-what-undoing-rewinds-to.md)'s bracket
- **Explain a command before it runs.** Plain words for what it does, with
  anything destructive, anything needing elevated rights and anything leaving
  the machine marked. Edit it, or try it in the box first. **Called an
  explanation and not a guarantee**, because a script, an alias or something it
  downloads can change what happens
  **The answer half is built, 2026-10-17**: `alo_terminal::WhatACommandWillDo`
  carries the plain words, the three marks and the disclaimer, and an `alo` line
  in the transcript can hold it beside the five waiting answers. Two rules are
  tests rather than sentences — the disclaimer is on **every** explanation
  because the method that answers it takes no argument, and **nothing marked
  means nothing marked**, enforced by there being no `is_safe` to ask.
  **What is still owed is the asking, and it is not a chord**: alo does not know
  what a person has typed. Measured the same day — nothing keeps the typed line,
  `the_command_in_front` gives a process already running, and the grid is
  `alacritty_terminal` with a `VoidListener`, so no escape sequence is seen.
  Reading it off the screen would decide what the *person* typed from what a
  *program* printed; keeping our own copy diverges the first time somebody
  presses Up. The real road is **shell integration**, which is a new mechanism,
  a published contract, and a decision about a sequence any program can forge —
  so no shortcut was bound to a surface that cannot work.
  **The road is decided, accepted by the owner 2026-10-18**:
  [ADR 0123](decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
  — the shell reports its own line editor when the person asks it to, and alo
  reads a mark rather than the screen. No hook runs while somebody types, so
  there is nothing for alo to watch between a prompt and an asking. **Accepted
  2026-10-18**, so this is buildable. Three later terms and the gesture are the
  lane's drafts and are marked in the ADR as awaiting the owner; nothing built
  may rest on those until they are accepted
- **Say up front whether it can be undone.** *This can be put back* and *this
  cannot* are different sentences, and the second is the one worth seeing before
  you press Enter rather than after
- **Turn something that worked into something you can run again.** *Start my
  development environment*, holding reviewable commands, the permissions it
  needs and the inputs you can change, and checking its assumptions still hold
  before it runs. **It stores which permissions it needs, never a grant** —
  a grant is yours to give and yours to take back ([ADR
  0001](decisions/0001-the-capability-model.md) §3), and a saved action must
  not become a way to hold one you did not give. *This read* — grants expire,
  and a saved action must not become a way to hold one that does not — until
  2026-10-25, when a length that does not expire was added. The rule is
  unchanged and the reason is stronger: an action that could carry a grant
  could now carry one with no end. *So that nothing
  asks a model to invent the same sequence every morning*
- **Tell me when it finishes, if it fails, or when the server is ready.** Asked
  for explicitly, shown in the panel of put-aside windows, and **reported from
  what the processes actually did** rather than from an estimate
- **Four kinds of stop, and they are not the same control.** Interrupt what alo
  is doing; stop a server alo started; leave the conversation; close the
  session. *Conflating them is how a server is left running after you thought
  you had stopped it*
- **Nothing is left running where you cannot see it.** Close a terminal and alo
  says what it started that is still going. *The shell's job table dies with the
  shell; a machine's process tree does not*
- **Local and remote are never ambiguous.** A restrained header — `Remote ·
  staging · developer account` — and before any action, where it will run and
  whose files it touches. **What is attached here never travels to a remote
  machine or its model without you saying so.** *For SSH the header says what
  the connection claims rather than what alo measured: alo's own trust between
  machines is pairing
  ([ADR 0031](decisions/0031-the-pairing-is-the-key.md)), and an SSH host is
  neither local nor paired*
- **Run it on another of your machines and watch it here.** Output in this
  terminal, files resolved on the right side, under alo's own grants rather than
  a shell on a box
- **A project you come back to is where you left it.** Its terminals, folders,
  launch commands and related windows — *your workspace is restored, two
  services are stopped* — with **Start services** as something you press.
  **Never pretends processes survived and never silently re-runs anything.**
  *Built on the canvas's Places, which already promise that a Place remembers
  time, rather than beside them — two things called a workspace would outlive
  everyone who made them*
- ★ **Ask the machine itself, from the terminal.** *What is using the memory?
  Why is this slow? What is filling the disk? What is on port 3000? What is that
  process doing?* Answers name the actual processes, files and sockets rather
  than describing them, and each one can be acted on from where it is named.
  **A shell can only run a program you already know the name of**; this answers
  the question a person actually has. The measured and the worked-out stay
  apart: *this process is holding 4.2 GB* is an observation, *that is why your
  machine is slow* is a reading of it, and a sensor nobody can read is said to
  be missing rather than guessed at
- ★ **Do it, not just read about it.** *Close Slack. Stop that server. Put that
  window on the other screen.* **What a shell cannot do is work out which
  window you meant** — a window is not a program and has no name a person
  knows, so a tool that can close one still needs you to identify it. *This
  line claimed a shell cannot close a window at all, which the owner corrected
  on 2026-10-14: tools that request window operations exist on some systems.
  The advantage is resolving the intended window and applying one set of
  permissions to it, not that the operation is impossible elsewhere.* And the
  three it covers are different acts, named apart rather than collapsed:
  **asking a window to close**, **quitting the application**, and **forcing a
  process to stop** — the first is what *close Slack* means, unsaved-work
  prompts are the application's and are respected, and a window that has not
  closed is reported rather than assumed. At the default setting a change is
  proposed in a sentence and waits; grant more and it simply happens
- ★ **Anything you can do on this machine, you can do by typing what you mean.**
  Not only what has a command. Running a program, closing a window, changing a
  setting, revoking a grant, moving a file, asking what happened yesterday —
  **the terminal stops being the place you run programs and becomes the place
  you tell the machine what you want.** *The owner, 2026-10-14: "in the terminal
  the user can command and run anything on the pc."* What bounds it is the
  person's own settings and nothing else; what never changes is that the run is
  recorded, the files are snapshotted before it, and what leaves is shown
- **Completion that has read your whole machine, because nothing leaves it.**
  Your real file tree, your installed packages, your history, on the device. *A
  completer that sends your tree somewhere can never be given it*
  *The twenty lines above were added on 2026-10-14 by the owner's ruling
  — "these are great features and we shall add them all" — after
  [ADR 0112](decisions/0112-with-the-persons-permission-the-agent-may-do-anything-on-the-machine.md)
  removed the question of whether a person may permit them. **None of them is
  built.** Three carry a resolution that was the lane's judgement rather than
  the owner's and may be corrected: history is opt-in so the forgetting promise
  is not spent, a saved action stores which permissions it needs rather than a
  grant, and a workspace is built on Places. What each one needs, and in what
  order, is
  [ADR 0110](decisions/0110-the-agent-writes-the-command-and-only-the-person-runs-it.md);
  the shared rule above is why four of them no longer each repeat it.*
- ★ **alo writes the command, and you decide who runs it.** You ask for
  something and get a **command and a sentence about it**, in a surface alo
  draws. **Out of the box you press Enter and alo does not** — the surface is
  the compositor's and the agent reaches it through a verb carrying text and
  nothing else, so at that setting there is nothing that submits on your behalf.
  **Grant more and alo runs it**, at the level you picked, and the record, the
  snapshot before each run and the egress line stay on at every level. **The
  headline of this line read "only you run it" until 2026-10-14**, which was a
  default written as a rule — the owner: *no limitations for the AI agent, only
  when the user chooses in the settings*. Since
  [ADR 0111](decisions/0111-the-person-chooses-whether-their-agent-may-use-the-terminal.md)
  it is a default rather than the only shape: a person who grants more gets an
  agent that runs the command instead of proposing it, and the line above
  describes what they chose rather than what they are allowed. What no setting
  changes is that **running code is not a verb on the broker's list** — ADR 0064
  — so no `exec` exists at any level. *Every other AI terminal is built around the agent executing.
  Law 2 forbids that here, and that is what makes the next two lines possible:
  an agent running three hundred commands cannot be undone, and one command you
  chose can. And undo is not a nice idea: law 2 already says every run at every
  level **can be undone**, and nothing here does it yet.*
- ★ **Undo a command.** A command you run that way is **snapshotted before it
  runs**, and one key puts your files back. `rm -rf` in any other AI terminal is
  gone for ever, because a process cannot snapshot a filesystem it does not own;
  alo OS chooses that filesystem at install
  ([ADR 0045](decisions/0045-what-undoing-rewinds-to.md)) so that it can.
  **What undo does not cover is said where it is offered, not in a settings
  page**: it rewinds this machine's files, and it does not recall a push,
  un-send an email, or undo anything that already left. A reversibility
  overstated once is one nobody trusts again
- ★ **Ask alo from the terminal, and asking means acting.** One command, and
  you never say which you meant: `alo-ask open the text editor` is a request and
  `alo-ask may the tenant sublet?` is a question, typed the same way at the same
  prompt. A request is **proposed, never performed** — alo says what it would do
  and what grant the work needs, and nothing happens until you approve it, which
  is *reads answer, changes wait* in the terminal rather than only in the dock.
  *A misread question costs you reading one sentence, because a proposal nobody
  approves does not run* ([ADR 0124](decisions/0124-one-command-asks-alo-from-the-terminal-and-asking-means-acting.md)).
  **Partly built, 2026-10-22**: the road to the agent and the deciding between
  the two are built and tested; what decides is a model, and on a machine whose
  only model is small the deciding is unreliable — measured at 1 correct in 8 on
  requests. The approval is what protects you, and it is unconditional either way
- ★ **Ask what a command actually did** — `alo watch cargo build` — and the
  answer comes **from the kernel rather than from what the command printed**. A
  command that lies in its output lies to every other tool; it cannot lie to the
  kernel's own hooks. **One command, because you asked for that one.** This does
  **not** weaken *it forgets everything that was not an agent* further down this
  file and is not permitted to: nothing is watched, something was measured on
  request, and the contrast is the point — the one system that could watch
  everything measures only what you point at
  *The three lines above were written on 2026-10-14 after the owner said the
  feature list and the decisions must not be what stops a terminal worth
  switching for, and that both may be edited.
  [ADR 0110](decisions/0110-the-agent-writes-the-command-and-only-the-person-runs-it.md)
  holds the design, what each part needs, and the reasoning a later reader
  should disagree with rather than only discover. **None of it is built**: a
  terminal opens (`Ctrl`+`Alt`+`T`) and is an ordinary terminal; the snapshot
  filesystem is chosen at install and no command has ever been snapshotted; and
  the kernel boundary is a prototype whose own header calls it `the smallest
  thing that can carry that idea or disprove it`. **Whether any of it joins the
  six-item launch list in `ROADMAP.md` is the owner's and is not yet
  decided** — the earlier word was that the terminal comes after launch, which
  is a scheduling call and not a design one.*
- **Search your own files, without asking anything** — by name, kind, date and contents, in the file manager, indexed on the machine. The agent's *“where is that file?”* is a nicer way to reach this; **it is not the only way**, and a machine whose only search is a conversation is a machine somebody locked out of their own documents
- **What is running, and what it is using** — processes, memory, disk and network in a window. The plain answer to *“why is it slow?”*, for the person who cannot or will not ask
- **What is filling the disk** — shown as sizes you can open up and click through, not a number in Settings

**Software, and what applications expect (ADR 0005)**

Applications install sandboxed and reach the system through the XDG Desktop
Portal interfaces — the contract every Linux application already speaks. Meeting
it is what makes existing software work on alo OS without knowing we exist, and
each portal request is a grant in the sense of ADR 0001.

- **Install applications**, sandboxed, from Flathub or a repository the organisation runs; update and remove them
- **A decided list of what a person can install** — a curated catalogue, pinned and licence-recorded the way the fresh machine's own seven applications are, with an entry for each kind of work a person does: pictures, painting, illustration, photographs, page layout, video, audio and documents. A fresh machine can browse, read and look at a picture and cannot make one, which is the gap this closes. The list is data (`crates/alo-software/catalogue.toml`) and names no application a machine already ships, so a market screen cannot offer to install the browser a person is reading it in. *Asked for by the owner on 2026-10-16. The shape is what it is because of the candidate that prompted it: a Rust image editor with 65,000 stars that turned out to be sixteen days old, in early alpha, and published to no source a machine can install from — so an entry must name a source, a release and a licence, or it is refused.*
- ★ **One list of what has been granted to what** — agents and applications in the same place, revoked the same way
- Portals: file chooser and documents, open-with and default applications, notifications, print, screenshot, screen capture, camera, microphone, clipboard, trash, wallpaper, settings, inhibit (no sleep mid-presentation), network and power-profile monitors
- **Secret storage** — one keyring behind the Secret portal, so applications stop inventing credential storage
- **Session management**: log out, switch user, lock, and reopen what was open
- **Corporate proxy support**, machine-wide and honoured by applications. A great many company networks have no other route out
- Portals: USB devices, global shortcuts an application registers, dynamic launchers, remote desktop
- **Location services**, off by default, per-application, with an indicator when in use
- Applications contribute to search — one place to look, not one per program
- Realtime scheduling for audio work, which is what a workstation is often bought for
- Unsandboxed installation as a deliberate, clearly-marked act — never the default, and forbiddable by policy on a managed machine. Brought forward from v1 by ADR 0064: developers need their own tools from the first day

**Devices and media**

- Audio in and out, with device switching that works mid-call
- Bluetooth: pairing, audio, keyboards, mice
- Camera and microphone
- Media playback, and the codecs people actually have files in
- Power management, battery, sleep on lid close
- Night light and display colour

**Language and access**

- **The shell in the user's language — all 24 official EU languages to begin with**, and any language somebody contributes after that. Bulgarian, Croatian, Czech, Danish, Dutch, English, Estonian, Finnish, French, German, Greek, Hungarian, Irish, Italian, Latvian, Lithuanian, Maltese, Polish, Portuguese, Romanian, Slovak, Slovenian, Spanish and Swedish. Not "English plus the big five": a sovereignty product that cannot speak Maltese or Irish is selling sovereignty to some Europeans and not others, and those are exactly the member states with the least software in their own language
- Regional formats and timezones per language, and a keyboard layout offered with it — choosing Greek and then hunting for a Greek keyboard is the same bug twice
- **A region for dates and times, chosen apart from the language** — in Settings, a person picks the country or region whose way of writing dates and times they want, from every region in the world named in their own language (a Portuguese speaker in Belgium reads Portuguese and writes dates the Belgian way). **Every date and time on the machine follows it**: the status area's clock, the lock screen's clock and the record, which puts the time at the head of each entry and heads each day *Today*, *Yesterday* or the date, as frame `82:887` draws it. Where nobody has chosen, the language they read decides, with CLDR's own region for it — never one country's way for everybody. **A region decides the order and the clock; the words are always the language the person reads now** — Germany chosen while reading English is *28 Oct 2026, 17:38*, and the month turns French the day they change to French. **The language picked on the first-start sheet is the person's reading language**, not only their keyboard, so the machine speaks it and writes dates in it from the first minute. **The first-start screens set language, region and keyboard apart** (frames `09`, `09a`–`09c`): English can be read on a German keyboard, and **the region picked sets the machine's time zone**, asking which where the country has several — the United States, Russia, Brazil — from CLDR's zones in the person's language, offline. *Decided by the owner on 2026-10-28.*
- **Right-to-left ready**, so adding a language later is translation rather than rework, even though no official EU language needs it today
- ★ **The agent answers in the language you asked in** — the shell being translated is table stakes; being able to say "wo ist die Rechnung von Northstar?" and get an answer is the thing a cloud assistant does badly for smaller languages
- EEA and candidate languages as translations arrive: Norwegian, Icelandic, and the accession languages
- Community translation, so a language nobody sold us on can still be complete
- Screen reader, magnifier, high contrast, larger text
- ★ **A published accessibility conformance report against EN 301 549** — the harmonised European standard, and the mandatory technical specification for public-sector ICT procurement across the EU. Procurement asks for the report, not the intention
- Sticky keys, slow keys, and keyboard-only operation of everything
- Voice control of the shell — which for us is the agent, arriving somewhere it was always going

## `alo-agentd` — the agent's reach into the machine (ADR 0001)

- ★ **File verbs**: list, read, find, rename, move, archive — over granted paths only
- ★ **Application verbs**: open, focus, arrange, close
- ★ **Context on invocation**: focused window, selection, open document — offered, never watched
- ★ Grants: pick a folder, see what is granted, revoke it, and it expires
- ★ **You choose how long: an hour, a day, a month, or until you remove it.**
  Grants are expiring *by default* and the shortest length is the row the ring
  lands on, but nobody has to come back every month to give the same folder
  again. The one that does not end is the last row rather than the first, and
  it is the one row on the list that says what it is — because it cannot
  protect you by running out, so being findable is all the protection it has
  ([ADR 0001](decisions/0001-the-capability-model.md) §3, amended 2026-10-25
  by the owner: *I can give an agent full access or remove it but there is no
  time limit*)
- Every execution recorded with its origin, approval and grant
- ★ **The grant is a boundary the kernel imposes, not a rule the daemon follows** (ADR 0013). For the length of one turn, everything outside the grant is *unreachable* — Landlock for the files, seccomp for the syscalls, an eBPF programme on the turn's own cgroup for the sockets. A verb that overreached would fail at the syscall, not be talked out of it
- ★ **And the kernel is taught what a turn is** (ADR 0015) — our own programs on the kernel's security hooks, the same ones SELinux and AppArmor use, carrying a grant the daemon wrote into shared memory when the turn began. **No kernel is patched and no fork is kept**: they are loaded at boot and verified by the kernel before they run
- **So the record stops being anybody's account of themselves.** What a turn touched is what the kernel watched it touch — the difference between an audit log and a guarantee
- ★ **And it forgets everything that was not an agent.** A syscall outside a turn is checked and leaves no trace: no log line, no counter, no timestamp. Your editor, your browser and your terminal pass through and are forgotten. **The mechanism that could watch everything is the one place this promise is proved by a test rather than stated**
- ★ **So the record stops being a claim and becomes an observation.** Today it is `alo-agentd`'s honest account of itself, which is an audit log; with the boundary in place, what a turn touched is what the kernel watched it touch. *What did the agent do* is answered by the machine rather than by the program being asked about
- A turn whose boundary cannot be applied **does not run** — a refusal, not a warning, the same rule `alo-egress` already follows when a policy cannot be evaluated
- ★ **System verbs** through the privileged broker: printers, network, updates, storage
- ★ **Application adapters** — installed applications become agents with typed verbs (`@blender`, `@resolve`, `@gimp`); see `docs/contracts/app-adapters.md`
- The accessibility fallback: any application with no adapter is still readable and operable through its AT-SPI tree
- ★ Adapter SDK published, with a conformance suite third parties can run
- Policy: which verbs and adapters are permitted, set per machine or per fleet
- Screenshot-and-click, marked in the record and disabled by policy by default — last resort only, never the default mechanism

## alo working inside applications, and in the person's name (owner, 2026-10-31)

*The owner's decisions and answers of 2026-10-31, written down here because
building is gated on this file. [ADR 0143](decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
records them and what they run into. **None of this is built.** The lines say
what is promised; the ADR says what stands between the promise and a screen.*

*The rule under every line here is the owner's: "we are not dictators so we
allow users to choose what he wants: if he wants alo to do all for him we do, if
the user does not want we do not." So each choice below starts at the safe
answer and can be turned up, as far as **alo does it all for me**, with its risk
said in plain words — the shape running code already has (ADR 0064). The
record, undo wherever it can reach, the indicator and plain warnings stay on at
every setting, and **nothing alo causes to leave the machine is ever unshown**:
that is law 1, and a law is not a setting. The top of two of these —
*everything alo types, approved once* and *every application* — is promised and
not yet buildable: it goes past `CLAUDE.md`'s *an approval is never a session*,
and ADR 0143 asks the owner to confirm it is read the way ADR 0064's full trust
already is.*

- ★ **alo works inside the applications a person uses, the way a person does.**
  It opens the application, reads what its window shows, types into its
  editor and saves, so that what it made is in front of the person in the
  application they would have used themselves, not written to disk behind their
  back. **How much one approval covers is the person's setting**: by default one
  file, shown in full and saved in the same approval; stricter, each block on
  its own; or, turned all the way up, everything alo types for one request,
  approved once — said as *alo will type text you have not read line by line*.
  alo reads an application's window only during a turn the person started, and
  keeps no copy of what it read. *Owner, 2026-10-31.*
  **Any installed application** — the owner, the same day: *"for alo driving
  applications it should be able to drive any application installed."* **Only
  the ones the person has let it use**: by default the person lets alo use
  **one application at a time**, by name, for as long as they pick, and takes it
  back in one action — and never because alo used it once. A person may instead
  let alo use **every application on the machine**, told that this includes the
  ones that hold their passwords; even then it is one row in the list of what
  alo may reach, with its length, removed in one action. **What "any" honestly covers** is written down
  in ADR 0143, application by application: an application that describes its
  own window to assistive technology (most GTK, Qt and LibreOffice windows,
  browsers, and Electron applications started the right way) can be driven; one
  that draws everything itself — a game, a 3D or painting canvas, a remote
  desktop — **cannot be driven reliably by any mechanism**, and alo says so
  rather than guessing — unless the person has switched on, for that
  application, alo acting on pictures of its window, told that *alo may press
  the wrong thing*. **Off by default.**
  **In the browser, by default alo reads only the window it opened** for the
  turn; a person may let it read every tab, told that this includes their mail
  and their bank. **What a driven application sends is on the indicator**: by
  default alo works in a window of its own, so what that window sends can be
  counted; a person may let it work in their own window instead, told that the
  indicator will then show *that* the application is sending and cannot count
  what. Counting everything an application does, the person's own use included,
  is not offered at any setting — that would be watching the person.
- ★ **alo OS ships an open-source code editor, installed by default** — the
  owner, 2026-10-31. A person may also install Microsoft's VS Code, and alo
  works in it the same way. Which editor is proposed in ADR 0143 —
  **VSCodium**, the open-source build of the editor developers already know,
  MIT-licensed and verified on Flathub, recommended over Zed and Lapce — and
  not yet named by the owner. **It is a new role on the fresh machine's list**,
  beside the browser and the terminal, not a second text editor. Its package can
  read every file in the person's home and start programs outside its sandbox,
  and the machine says so where a person looks at what is installed.
- ★ **alo can post to a social account in the person's name — only when the person has switched that on.**
  Settings holds the choice, **off until the person turns it on**, and it says
  the risk in plain words before it can be turned on (proposed wording): *alo will be able to
  publish under your name to everyone who can see your profile; a post can be
  seen and copied before you delete it; and the site may treat posting by an
  assistant as against its rules and restrict your account.* By default it is
  switched on for one site and one account; a person may switch it on for every
  social site they are signed in to. It lasts as long as the person picks — an hour, a day, a month, or until
  they remove it — and it is removed in one action, taking effect at once, like
  every grant. **Switching it on never approves a post.** Each post is still
  proposed as a sentence with its **full text** and where it goes, and waits for
  one approval; one approval publishes it once. **Deleting a post is the same**:
  its own sentence, its own approval. A post that was published cannot be
  recalled by undo, and the sentence says so. **alo posts in the person's own
  browser or application, as they would — no site's publishing interface is
  used** — and a site's own terms are the person's to weigh, never alo's to
  enforce. *Owner, 2026-10-31: "User allows this in the settings that allow alo
  to use my social media profile … no problem if the users approves it"; and
  later the same day, "the agent is using the user's PC so we do not need APIs
  for this. Our alo accesses and operates any application installed."*
- **The developer demo: build something simple in the code editor the machine ships, and put it on GitHub Pages.**
  alo builds a small page inside the shipped editor, approving as the person's
  setting says; the person signs in to GitHub themselves — alo never types a
  password, and its reading of a window never includes a password field's
  contents; and alo presses **Publish** in the editor after one approval whose
  sentence says plainly that *publishing to the internet cannot be undone*. What
  the editor sends is on the indicator while it happens. **The demo posts
  nowhere social**: *"We may not do a demo with LinkedIn but we can do a demo
  with other applications. LinkedIn was just an example."* This is **Demo 2**,
  the public demo for developers, chosen over the first walk by the owner on
  2026-10-31. The stages are in
  [the Demo 2 plan](autonomy/demo-two-alo-inside-applications-plan.md).
  *Owner, 2026-10-31.*

## The AI stack — model choice and deployment configurations

**Four choices at first start, in this order** (owner, 2026-10-06, superseding
the clarification of 2026-09-08 kept below):

1. **On this computer** — the person's selected model and runtime on their PC,
   including models such as Llama where the integration supports them.
   *alo works locally. Availability depends on this PC.*
2. **A machine on your network** — a machine this one has been paired with,
   ADR 0003's one box serving an office.
   *Selected content goes to a machine on your network.*
3. **My provider** — a compatible API selected by the person, such as OpenAI or
   Mistral, using the credentials that provider requires.
   *Use your provider. Selected content leaves this PC; provider charges may
   apply.*
4. **No AI** — no model, no provider and no agent (ADR 0009).
   *Open apps, find files and use the full canvas by hand.*

**All four carry equal size, typography and weight, and none is pre-selected**
(ADR 0014 §4, ADR 0025). **No AI is a complete answer** — not a skip, not a
fallback, and not an invitation to persuade the person later.

**alo's own service is not one of the four.** It appears only after *My
provider* is chosen, as an ordinary provider beside the others: no badge, no
special styling, no automatic selection, no commercially privileged ordering
(ADR 0014). Provider names and ordering shown in the design file are
illustrative and do not establish an integration-support matrix.

**Sending to a machine on your network is still data leaving this PC**, and uses
the same disclosure and recording as any other destination. A local destination
is not by itself a promise that nothing leaves the building; that claim needs an
enforced local-only processing policy, which is a separate thing from where the
machine sits.

These are the primary user-facing choices, not four different privacy policies.
Do not replace them with "prefer local processing" and "keep questions on this
PC", or present those as competing setup choices. Any advanced privacy controls
belong separately in settings. This establishes the model-selection direction,
not acceptance of proposed ADR 0021 or new `ThisMachineOnly` semantics.

Existing release tiers still apply: this is not a claim that alo's hosted
service, paired-machine processing or every listed integration is available in
the current release.

### How a first-start screen is built (owner, 2026-10-07)

Settled against the design file and verified against the refreshed snapshot;
`docs/design/the-first-start.md` carries the measurements and the frame ids.

- ★ **The strip above every setup screen reads `alo OS`.**
  It is identity, not state: no step count, no stage name, and no record of
  which step a person came from. **The duplicate wordmark elsewhere on those
  screens is removed.** 69 of the 77 frames carry it.
- **The running canvas has no strip and must not gain one.**
  A frame has the strip when it is a setup sheet; a frame showing the canvas,
  the Dock and a place name has none. Stamping a setup label on the canvas
  would say setup is still running after it has finished.
- ★ **Setup shows no overall progress counter.**
  No *N of M*, no row of dots, no decorative progress line. The screen's
  heading says what the current task is. Branches walk different numbers of
  screens, so there is no honest denominator — and a build must not derive one
  from the frame inventory.
- **Real installation progress stays beside its own operation.**
  Download, preparation and installation progress, and the error and recovery
  screens around them, are not the counter and are not removed with it.
  *Removing a decoration must not remove the one thing a person waiting on a
  disk write actually needs.*
- ★ **`Optional` sits immediately above the heading, on ten screens.**
  Written as `Optional · <subject>`: the three access choices, file import and
  its two follow-ons, sign-in options, PIN, fingerprint and account recovery.
- **Each optional screen keeps its own way to decline.**
  They are not interchangeable — `Not now`, `Cancel`, `Back`, `Use password
  only`, `Use password for now`, `Done`. A build must not normalise them into
  one.
- ★ **One selection border: 2 logical pixels of navy, plus the word.**
  `#102A43`, with `Selected` beside it, and the former 1.5px access-screen
  border was a leftover. **Keyboard focus stays a separate mark** — an outer
  ring with its own gap — and moving focus never commits a choice. **Teal
  never means selected or focused**; it stays reserved for alo acting
  (ADR 0067). Display scaling is applied once, at the existing conversion
  boundary.
- **`Not now` leaves an access screen without changing permissions.**
  It must not commit the alternative the screen happens to be displaying, must
  not raise or lower access, and is not the same action as `Back`. Tiered at v1
  because the three levels it has to preserve are the v1 item *running code, at
  the level the person picks*; the design exists now and the rule is recorded
  now so that whoever builds the policy inherits it.

**What this supersedes, kept because the change is the point.** The 2026-09-08
clarification named **three** main model-selection choices — *local models, your
own API provider, and Alo* — with alo's own service as one of the three, and
added that the deployment configurations were *not four competing model-source
buttons*, that *the existing no-agent path remains a separate opt-out*, and that
*paired-machine support is preserved without adding a fourth primary source
category; its detailed placement remains to be designed*.

Every one of those is now decided the other way: alo's service sits inside *My
provider*, the no-agent path **is** the fourth choice rather than an opt-out
beside them, and the paired machine **is** a primary choice and is placed,
second. `crates/alo-setting-up`'s `THE_FOUR` already held this shape and this
file had not caught up — which is how `docs/design/figma-brief.md` came to brief
a designer one choice short, and how the design file came to carry a fourth box
that was not ours.

**alo OS works well with a model on this machine, with a model on a machine on
your network, with a provider you added — and with no model at all.** Those are
four supported configurations, not one real one and three compromises, and
**“works well” is a measured bar in each of them rather than a hope in three.**

**Model choice belongs to the person** (owner clarification, 2026-09-08).
Alo-provided models or services, the person's own local models and runtimes,
and compatible third-party APIs are legitimate choices. Alo ownership is never
a condition of being a valid choice. Model ownership and processing location
are separate: an alo-provided service may be remote, and a third-party model
may run entirely on this machine. Existing organisation policy, capability
validation and the release tiers below still apply; this clarification does not
bring later-release features forward or claim unimplemented integrations work.

Freedom to choose a model is distinct from a verified privacy guarantee. A
loopback address establishes where a service is contacted, not where it performs
inference. Any stronger local-only guarantee must be based on established
protection, not the model's brand. The treatment of unverified services under
`ThisMachineOnly` and the enforcement options in proposed ADR 0021 remain
unaccepted; this clarification neither changes that policy nor narrows the
egress promise. Paired-machine operation and the no-agent choice remain intact.

Each has its own bar, and they are not interchangeable:

- **On this machine** — CPU or GPU, and the model must actually drive the verbs,
  not merely produce sentences (ADR 0007, as corrected).
- **On a machine on your network** — a deliberate pairing made on both machines,
  and it is egress: the indicator fires, because *it only went down the
  corridor* is how a guarantee stops meaning anything (ADR 0003).
- **With a provider you added** — including alo's own, which says who and where
  in the same words as anyone else's and gets no exemption (ADR 0008, ADR 0014).
- **With none at all** — the whole machine still works, the agent's surfaces are
  absent rather than greyed out, and every capability an agent has is reachable
  by hand (ADR 0009). A person who never turns it on has a complete computer.

**Nothing moves between them on its own.** A local model that fails does not
become an API call, a provider that runs out does not become a local model, and
a machine with the agent off is never talked into turning it on.

- ★ **It runs on the machine you already own.** No graphics card required: the catalogue carries models that answer comfortably on an ordinary business laptop's CPU, and the system picks one (ADR 0007). This is what puts alo OS on the Windows 10 fleet rather than on a few hundred workstations
- ★ **It works well on a CPU and it works well on a GPU** (ADR 0007, as corrected). Neither is the other's fallback and neither is a “default”: the machine runs a model sized for what it has, and **“works well” is a measured bar in both cases rather than a hope in one of them**. A card buys a larger model and practical fine-tuning — not entry, and not the real version of the product
- ★ **The catalogue says whether a model can drive the verbs, not just whether it will run.** An agent turn asks a model to emit a typed verb call with valid arguments several times over, which is exactly what small models are worst at — sentences they manage, structure they lose. A model that runs beautifully on a laptop and cannot emit a valid call is useless as an agent, and a catalogue that only knew about memory would recommend it
- **And it is measured by us, not claimed by the publisher** — the same honesty already applied to how a model behaves on a CPU
- **A machine is only offered agent work it can actually do.** Where nothing catalogued clears the bar on a given machine, the honest answers are the ones already on offer — a paired machine, or a provider — shown as a choice and never substituted silently
- ★ **The GPU works on first boot**, where there is one — no driver installation, no CUDA archaeology. Acceleration, not an entry price
- ★ **A model runs in one command**, from a curated catalogue of open-weight models with their licences stated
- ★ **The release carries no model, and a person brings their own** — weights they already have, a provider they have an account with, or no model at all, and alo OS chooses none of the three for them. **Nothing is chosen on their behalf, and until somebody chooses, nothing answers** — which is the half of the older promise that survives. *This read **the local model is what the machine arrives ready to run** until 2026-10-06, when the owner decided the release ships no weights: a model we picked was a model picked for them, it was sized for the one certified laptop, and everybody downloaded 4.87 GiB including the people who would never run it. **Out of the box now answers nothing**, and saying so plainly is the point of this rewording; it returns later through our own API rather than through the image, and answering that way means inference leaving the machine — visible, recorded and never silent (Law 1)* (ADR 0095, ADR 0025, ADR 0016, ADR 0014)
- ★ **Add your own provider in Settings** — a name, an address, and a key: Mistral, your own endpoint, or whatever you already pay for. The key goes to the keyring, never into a settings file, so it cannot leak through a backup or a support bundle. You say where the provider runs; nothing is guessed from its address
- An address that is not https is refused rather than warned about, unless it is a service on this machine — "it is only our internal network" is how a key ends up on the wire in clear. **Or another computer on the person's own network**, written as its number (`http://192.168.1.20:11434`), after a plain caution that the question and the key cross that network unencrypted and anyone else on it could read them; saving it is the person's own Enter. *Added 2026-10-30 by the owner's decision (ADR 0021): an AI service on another computer at home was being refused outright, which is the cost the person should be told and allowed to choose (law 5, ADR 0064 row 3). An address on the internet over plain http is still refused; ADR 0064 row 3 reaches that case too, and it is not built.*
- Test a provider before saving it, so a mistyped key is found now rather than in the middle of a question
- ★ **Or alo's own service, which needs nothing set up** (ADR 0106). Setup's fifth choice, beside the other four and changeable in Settings afterwards. **It starts free, on a trial of credits, and asks for a subscription when they run out** — so choosing it answers the first question immediately rather than sending somebody off to find an API key, which is the one thing every other assistant makes a person do before anything works. The question leaves this network and alo answers it, which the indicator shows and the record keeps **exactly as it does for any other provider**: ours is the one a person has most reason to suspect of an exception, so it gets none. Nothing is charged and no account exists until the credits run out and the person says yes
- **The two accounts are never the same account.** The one a person makes to sign in is this machine's own and is not sent anywhere, which is what the first screen promises them; an alo subscription is a separate, online thing, asked for later and never on the sign-in screen. No sentence a person reads may suggest that signing in to the machine signs them in to anything
- ★ **Or not at all** (ADR 0009). Setup's fourth choice, with the same weight as the other three: no model, no provider, no agent. Everything else in this document still works, the agent's surfaces are absent rather than greyed out, and turning it on later is a setting rather than a reinstall
- ★ **A person never learns the name of anything we rented.** They install an application — not a Flatpak. They run a model — not Ollama. The system updates — it does not *pull an image*. Every engine underneath is configured and never patched (`CLAUDE.md`), and the same rule runs all the way to the surface: **if somebody has to learn what a container is to use their own computer, the machinery has become their problem instead of ours**
- **And it is enforced rather than remembered.** No sentence in `alo-strings` — the one place every word a person reads is written — may contain the name of a rented component. A translator handed “the Flatpak could not be installed” has been handed our plumbing to render into twenty-four languages
- **Anything an agent verb can do, a person can do by hand** (ADR 0009). A standing rule rather than a feature, and the check on every verb anybody proposes: a machine without a working agent must lose *convenience* and never *capability*.
- ★ **And it holds however the agent became unavailable** — declined at setup, no model downloaded, offline, the provider down, the key expired, or **the money ran out**. Only the first of those is a choice. If a verb is the only way to do something, that thing is not finished: it has a feature that disappears when somebody's card is declined
- **Running out of credit is its own answer, not an error.** A provider saying *payment required* or *quota exceeded* is reported as what it is — not as a rejected key, which sends somebody to check a key that is correct, and not as a status code. The machine says it once, where it happened, and carries on. **It never spends money somewhere else instead**, which would be the worst reading of ADR 0008's *never a silent fallback*
- **And it never nags.** A machine that cannot reach a model does not follow somebody around asking them to buy credit — that is the greyed-out panel ADR 0009 already refused, in a different disguise
- ★ **Or use an API instead** (ADR 0008). A model may answer on this machine, on a machine on your network, or behind a provider's API — for a laptop too thin to run one, or an organisation that would rather buy inference than operate it
- ★ **Which model answers is the person's setting; the organisation's rule is a bound around it and never a choice inside it** (ADR 0016). Two files and two owners — the bound in the machine's description an administrator writes, the choice in the person's own settings under their home directory, where nothing else about the machine goes. **A choice the bound forbids is refused in words naming the rule, never quietly swapped for a permitted one**: a machine that answered from somewhere else while showing a person the choice they made would break the one promise ADR 0008 exists to keep
- **alo hosts models in the EU, and you can subscribe to them** (ADR 0014) — for the machine that cannot run one worth using, and for the organisation that would rather buy inference than operate it, from a company subject to European law
- **It starts as Mistral, resold, and says so.** No inference of our own at first — an alo account in front of Mistral's API, which is French, so the sovereign claim is true from day one rather than from the day we can afford graphics cards. The provenance line names the whole chain: *answered by alo, using Mistral, in France*. **An intermediary that will not say who it forwards to is the thing this product exists to replace**, and we do not get an exception for being the intermediary
- ★ **The same model, whichever place you run it.** Mistral's weights are open, so the model we host and the model on your laptop are the same model — start on a subscription because your machine is slow, buy a better machine later, and move to local inference with no change in the answers. **No other vendor can offer this**: leaving OpenAI or Anthropic always means accepting a different model, because theirs have no local equivalent. Where it runs becomes a question of hardware and money rather than of quality
- ★ **And our own service gets no exemption.** It is one more provider: the same egress indicator fires, the same *answered by alo, in Frankfurt* provenance line is shown, the same policy applies — **a machine set to keep questions in the building refuses ours too** — and the same words appear when the balance runs out. There is no variant for it in the code, no default, no pre-selection, and no quieter indicator because we know where it runs
- **Selling inference means we profit when questions leave the machine**, which runs against this product's own promise. It is written into ADR 0014 rather than left unsaid, and the defence is structural: our service is configured exactly as a third party is, so there is nowhere for a special case to live
- **Cancelling is not an expiry.** Signing in to alo OS does not sign anybody in to a paid model, and stopping paying leaves an operating system that works rather than a trial that ended
- ★ **Where the answer came from is said where the answer appears** — "on this machine", "on the studio workstation, on your network", "by alo, in the EU". Not in a settings page somebody would have to go looking for, because a person about to paste a contract into a question is entitled to know where it is going first. **And it never claims more than alo OS can check**: a service you run yourself at a loopback address is *by a service at this machine's address — alo cannot verify where it was processed*, because an address establishes what was contacted and never what did the work (ADR 0021)
- ★ **Never a silent fallback, in either direction.** A local model that fails does not quietly become an API call — failing to answer is recoverable, a person's records leaving the building because a download was corrupt is not. **And a provider that fails does not quietly become a local model**, which would answer somebody from a smaller model wearing the same face and remake a choice they had already made
- ★ **Neither is the other's fallback.** A model on this machine and a provider you added are both ordinary, complete ways to run alo OS — not a good option and a degraded one. Which is in use is a person's decision, never a substitution the system performs when the first disappoints
- ★ **Unless the person says otherwise, in Settings** — one choice, theirs: *when what I chose is not available, you may use another model or API*. **Off until they turn it on**, and it changes who decides rather than what alo does silently: with it off, a question that cannot reach the place they chose is refused and says so; with it on, another is used **and the answer still says where it came from**, because the sentence beside every answer is not a thing this choice can switch off. *Asked for by the owner on 2026-10-18, in these words: "it should be in the settings if the users allows the alo to use the other models or apis if one is not available." It is law 5 over the two promises above it — they forbid alo substituting on a person's behalf, and this is the person doing it on their own. Until it is set, alo answers where it can and says plainly that the chosen place was not used, which is what shipped the same day and takes nothing away*
- A provider that will not say where it runs is reported as **unknown**, never assumed to be nearby — and unknown never satisfies a policy naming a region
- Policy over where inference may happen: anywhere, in the building, inside a region **the organisation names**, or this machine alone. We ship the mechanism, never a region of our own (ADR 0004, ADR 0008)
- Model lifecycle: pull, list, serve, unload, remove; disk accounted honestly
- ★ **Run a model we never catalogued.** Point alo OS at weights you already have and it runs them. *This was `v0.5` until 2026-10-06, when the owner decided the release carries no weights. **The tier moved rather than the scope gate being crossed**: with nothing shipped, this is how a person gets a local model at all, so a promise of bringing your own that waited for a later release would have been a promise with nothing behind it. The store the runtime serves from moved with it, from `/usr/share/alo/models` to `/var/lib/alo-model/models`, because `/usr` is the read-only half of a bootc machine and weights a person brings have to be able to land somewhere (ADR 0095)* **The catalogue recommends; it does not gate** — its job is stating licences and honest costs so somebody can choose well, never deciding what they may run on hardware they own. A machine where the only models are the ones we approved is a walled garden with a sovereign label on it
- **The machine warns and then gets out of the way.** A model too large for the memory in this laptop is *said so plainly, once* — and then run anyway if that is what somebody asked for. The honest costs the catalogue states are for deciding with, not for refusing with
- **What you bring is yours, including its licence.** We state the licence of everything we offer and gate our own catalogue on it. Weights somebody brings themselves come with their own terms and their own responsibility, and alo OS does not pretend to have checked them
- ★ **Guided fine-tune**: LoRA/QLoRA over a granted folder or a tenant's records, as a flow rather than a toolchain
- ★ The dataset, the adapter and the resulting weights never leave the machine
- Model runtime versioned *with* the drivers it needs, so an upgrade cannot break a working stack
- Serving more than one person from one workstation
- Evaluation: compare a fine-tune against the base model on your own questions, before trusting it

## Everyday pain — what people actually complain about

Not a category an operating system usually has. Each of these is a thing
everybody has suffered this month, none of them is solvable without an agent
that can reach the machine, and every one of them demonstrates in fifteen
seconds.

- ★ **"Where is that file?"** Ask in words — *"the contract Anna sent before the summer"* — over granted paths, indexed on the machine and never uploaded. Every cloud assistant can do this if you send it your documents; this one never sends them
- ★ **"Why is it slow?" and "what is filling my disk?"** Every operating system is opaque about itself and everyone has typed these into a search engine. With system verbs and the record, the agent can actually answer
- ★ **Printers, solved.** The agent finds it, sets it up, and fixes it when it stops. The most hated object in computing, and a small feature people tell other people about
- ★ **"I can't open this file."** A `.pages`, a `.heic`, a `.dwg`: the system converts it where it can, and where it cannot says plainly what will open it, instead of shrugging
- ★ **Undo what the agent did.** Every execution is already recorded with its origin and the image already rolls back — together they make *"undo everything the agent did this afternoon"* real. An agent you can reverse is an agent people let do more, and no other system offers it
  **The bracket is taken, 2026-10-17, and measured on a real filesystem.**
  `alo_turn::the_bracket_a_turn_takes` makes the two read-only snapshots either
  side of a changing turn and writes down what it did, which ADR 0045 called
  *the missing middle* — both ends of this feature were built and nothing took a
  bracket. Measured as root on a loopback `btrfs`: the snapshot is made by the
  base's own program, `btrfs subvolume list` names it, `property get -ts` says
  `ro=true`, a write into it is refused, and the two sides really differ. **No
  capability is needed and none is held** — ADR 0045's seventh term, and the
  guard on it is a test in `alo-letting-go`.
  **A turn that could not be bracketed still runs**, which is ADR 0045's third
  term and is structural rather than remembered: the taking answers *why not*
  instead of an error, so no `?` can turn *there was no room* into *the turn
  failed*. The snapshot is read-only because #752 says the agent may **neither
  alter nor destroy** a bracket.
  **What is still owed: nothing calls it.** The daemon learns who the person is
  from `SO_PEERCRED` in `listening.rs` and the turn does not, so carrying the
  person and their home to the turn is the next change — and it is plumbing
  rather than a decision or a mechanism.
- **Updates that never interrupt.** Atomic images mean an update can be genuinely invisible and instantly reversible. On the system people are leaving, this is the single most hated behaviour there is
- ★ **"Make this machine like my old one."** Configuration as a document, pointed at a person rather than an administrator: a replacement machine that is actually yours, not a week of rebuilding
- **A new colleague working on day one** — a managed machine that arrives with the right applications, policy and grants already in place

## The local network — machines that find each other (ADR 0003)

Discovery is open; **use requires a deliberate pairing on both machines**. Being
on the same WiFi confers nothing.

- Machines find each other with zero configuration — no addresses typed, no accounts
- ★ **One GPU box serves the office**: a machine without a GPU discovers the one with it, and the agents just work. The inference never leaves the building; it moves down the corridor. **It is still egress, and the indicator still fires** (ADR 0003) — *"it only went to the machine down the corridor"* is exactly the kind of exception that quietly ends a guarantee, so shared inference is shown like any other departure and the pairing is what makes it wanted rather than what makes it silent
- Pairing: mutual, deliberate, enumerated, revocable in one action, and expiring — grants, across a machine boundary
- ★ **The whole of it works with no internet at all.** An office that cannot connect still has working AI
- A self-hosted workspace on the network is **discovered, not configured** — no DNS step
- Files and printers shared between paired alo machines, with no server in the middle
- Enrollment by discovery: a new machine appears to the fleet and asks; an administrator admits it
- ★ Cross-machine agent work — an agent may **ask** a paired machine, and acts only under a grant made **on that machine, by its person**

## Identity, fleet and compliance — what a large organisation requires (ADR 0004)

A machine is **personal** — nobody above the person — or **managed**, in which
case the organisation sets policy and holds a recovery key, and the person is
told so at first sign-in. There is no silent enrollment.

- **Sign in with the organisation's own identity provider** — SAML/OIDC against Entra ID, Okta or Keycloak. Nobody maintains a second set of identities for us
- **Smartcard and national eID sign-in** — eIDAS and government ID cards; in much of EU public sector this is required, not preferred
- **Disk-encryption key escrow**, so a machine survives the person leaving
- **Remote lock and wipe** for a machine that is lost — destructive by design, and recorded
- **Records export to their SIEM** — Splunk, Sentinel, Elastic, over syslog/OpenTelemetry. A security team needs agent actions in *their* console, not ours
- **Update rings**: canary then broad, haltable. No organisation updates a fleet at once
- **An update mirror they host**, for machines that never reach the internet
- ★ **A private model catalogue** — the organisation curates which models may run, served from inside. No one lets staff pull arbitrary weights
- ★ **An adapter allowlist**, signed and centrally permitted — an adapter is code that drives your applications
- ★ **Agent policy by role**: which verbs, adapters and models, per department. A finance team's agent may raise an invoice; an intern's may not
- ★ **Agent retention policy, centrally set** — what agents remember and for how long. A GDPR question with an actual answer
- ★ **Inference accounting** — which team used the GPU, and for what. Whoever paid for the workstation asks within a month
- ★ **Egress attestation**: a signed, printable statement of exactly what left this machine in a period. The artifact an auditor asks for and nobody can currently produce
- Configuration as a document — image, policy, adapters and settings declared in one file, so an identical machine can be rebuilt
- Helpdesk assistance as a **session a person starts and can end**, never a capability an administrator holds
- Certification groundwork: ISO 27001, BSI Grundschutz, ANSSI, Common Criteria. Years and money rather than code, which is why it starts early

## Sovereignty, as testable claims

- ★ **The egress indicator**: every network egress an agent causes, visible at the moment it happens — **except through a service the person themself put on this machine**, which is their own trust boundary and not one alo OS polices. A process they started can forward a question anywhere and no operating system can see past it; what alo OS does instead is refuse to pretend otherwise, so an answer from such a service says it cannot verify where the question was processed rather than claiming it stayed here (ADR 0021)
- ★ **No telemetry.** Not "anonymised telemetry". None — and the policy lives in a Rust service, not a checkbox
- ★ **A working day with the runtime alo OS ships produces zero inference egress**, measured at the network boundary — and we publish the test. **The runtime, specifically**: it opens no socket at all, so the claim is carried by the absence of one. A service somebody else runs opens a loopback socket that this measurement cannot see past, and is not what this line promises
- Full-disk encryption, enrolled at install
- ★ **A forgotten password never loses a person's data** (owner, 2026-10-13:
  *"No data should not be lost"*). Two secrets, and neither one of them being
  forgotten may destroy anything:
  - **The disk's** PIN or passphrase — already answered by ADR 0054. The
    person writes down a recovery key and **types it back before the install
    finishes**, and `alo_encrypting::WrittenDown` is the only proof that
    happened, held as a type so the compiler refuses to skip it.
  - **The account's** password — *not* answered yet, and the gap is measured
    rather than assumed: `alo_accounts::Accounts` has `created` and `signs_in`
    and **no way to change or reset a password**, and nothing in
    `alo-greeting` or `alo-shell` offers one. The root filesystem is what is
    encrypted (ADR 0054), so a person who forgets their account password
    reaches their own sign-in screen with every byte they own intact on a disk
    that opens, and cannot get in. Nothing is destroyed; it is a wall, and a
    wall is not what *no data lost* means to the person in front of it.
  - **The way back is the recovery key they already hold**, and it widens
    nothing: whoever has that key can already read the whole root filesystem
    by opening the disk, so letting them set a new password with it grants no
    access they did not have. That is what makes this safe rather than a
    trade-off — and it is the mechanism `account recovery`, one of the eleven
    first-start screens, exists for.
- Signed images, verified before a deployment becomes bootable; Secure Boot with our key
- Third-party security audit of `alo-agentd` and the broker, published

## The system and the image

- Boots on one certified machine, firmware to sign-in
- Image built as an OCI container image — a **bootable container** (`bootc`) on a rented, unmodified Linux base (ADR 0011), so the operating system *is* the image rather than being installed by one. No third language enters the repository to build it
- ★ Atomic updates with rollback — the previous deployment stays bootable, which a bootc image gives us rather than us building it (ADR 0011)
- ★ **Installed from the machine it replaces** — download one program on the Windows machine alo OS is replacing; it checks the machine, says exactly what will happen, stages a minimal boot environment, and pulls the operating system itself — signed and versioned — from the same registry updates come from. Download, click, reboot, sign in, with nothing to burn and no firmware ceremony, for the person who has that Windows machine in front of them. Secure Boot is respected through the signed shim, never something a person is told to switch off; anything destructive takes a typed consent naming what is destroyed, and Windows stays bootable until the one named point of no return (ADR 0023)
- **The environment that installs asks what it needs to know, on the screen, at the moment it needs it.** Which network, and which disk. **It had no way to ask at all until this line**: the only thing it was ever told was which disk, handed to it as one word on the kernel's command line by the Windows half — so it is wired-only, and started from a stick it boots, names itself and refuses, because nothing has chosen a disk for it. *Added 2026-10-20 by the owner's ruling. The question put to them was whether a Wi-Fi password may be carried across the restart and written to the staging area; their answer was to ask instead — in their words,* can we do it as the windows 11 — *and [ADR 0128](decisions/0128-whether-a-wi-fi-password-may-be-written-to-the-staging-partition.md) records it. **No pre-shared key is written to that partition.** Windows 11, Ubuntu, Fedora, macOS recovery and ChromeOS all ask during setup, and not one of them copies a saved password out of another operating system, because an installer with a screen never needs to. A console picker comes first and a drawn one follows it: what sits underneath — NetworkManager joining a network, and the disk a person named — is the same either way, so the second replaces a front end and throws nothing away*
- **It says what is on each disk, in words, before anybody chooses one.** Not `nvme0n1p3 · ntfs · 340 GiB` but *holds Windows and about 340 GB of files*, and *empty — nothing would be lost*. **The disk a person is about to change is the one thing every installer describes in the machine's vocabulary rather than theirs**, and two disks of the same size from the same maker are an ordinary thing to own. `alo_installer::disks` already names a disk by its maker and numbers it only when two would share one name; what this line adds is what is **on** it. *Added 2026-10-20 by the owner's direction to solve the pain rather than only the task. It finishes the promise above — `says exactly what will happen` — rather than adding one*
- ★ **The stick is for trying alo OS, and installing is something chosen afterwards.** Start it, use the whole system, see that your screen, your Wi-Fi and your trackpad work on **your** machine with nothing on that machine changed — and install from inside it once you have seen it working. *Added 2026-10-20 by the owner's direction. This is Ubuntu's* Try Ubuntu *and it is the most-liked part of their installer, for a reason worth stating: the question a person actually has is not* how do I install this *but* will it work on my laptop *, and every other road answers that only after they have committed their disk*
- **And Windows starts the stick, so nobody hunts for a boot key.** The program running on Windows offers to restart straight into it — no two-second window, no F2, F10, F12 or Esc that differs on every machine, and no Fast Startup to turn off first. *Added 2026-10-20 by the owner's direction. This is where people are lost before they begin, and it is not a technical obstacle — it is a trivia question about their own laptop. Windows can restart into a chosen device and this installer already writes boot entries, so the mechanism is one we hold already*
- ★ **A stick, written two ways: by a person with the tool they already have, or by alo OS itself.** The image is published as a file anybody can write with Rufus, Etcher, `dd` or whatever their own system gives them — which is how every other operating system is carried to a machine, and it costs a person nothing to learn. **And alo OS writes one, from the program already running on Windows**, because *making the stick* is the step where most people trying another operating system stop: a second tool, a file to find, and a list of devices named the way the machine names them rather than the way a person recognises them. *Added 2026-10-20 by the owner's ruling —* let's build 1 and 2 *— after the three options were put to them with the costs. The owner chose both roads rather than either.*
- **The one destructive thing in this product that reaches a device a person did not choose for alo OS**, and it is written down as such. Writing a stick **erases it completely**: there is no road that keeps what is on it, no partial write and no undo, and the device is not the disk they asked us to install onto. So it carries the heaviest of the promises: **the devices offered are removable ones and never the disk this computer runs on**, each is named the way a person recognises it with what is on it said in words, and **nothing is written until they type that device's own name** — the same consent the disk already takes, kept rather than a second ceremony invented for it. *A stick that is offered and refused teaches nothing; a stick that is written by mistake is somebody's photographs, which is why this line is longer than the one above it.*
- **And a road from removable media beside it. Both, not one instead of the other.** This line said *no USB stick* until 2026-10-19, and that was never a rule — it was a description of the only road we had, written when there was no stick to write one with, and it hardened into a promise nobody decided to make. Two kinds of machine cannot use the road above: one with no Windows to start from, and one whose only network is Wi-Fi. The second is the one that matters now: the staged environment is a few hundred megabytes with nowhere to choose a network, so it is **wired-only**, and the target audience is developers, most of whom have no cable. Worse, the Windows half shrinks the disk *before* the restart and never mentions a network, so a Wi-Fi-only person today ends with a smaller Windows, unallocated space, no alo OS, and no removal road. Carrying the network across the restart is how the road above reaches Wi-Fi; removable media is how a machine with no Windows is reached at all
- Printing. Unglamorous, and it decides public-sector deals. `alo-printing` is built and gated against a real CUPS — found, set up, printed to, and told what is wrong when it stops — so what is owed is the **On the machine.** half: no paper has come out of anything, and law 3 is what ticks that box (ADR 0078)
- The documents people are actually sent open: `.docx`, `.xlsx`, `.pptx`
- A web browser for the open web — a pinned upstream one, since our own engine is not scheduled
- Installer
- Fleet enrollment, policy and signed updates — **for alo OS machines only**
- Backup and restore
- A compatibility list, grown from the certified machine outward

---

## Designs worth learning from HarmonyOS

Huawei's HarmonyOS does several everyday things well. These are the ones that
fit a PC and fit alo, adopted on 2026-09-22 and not yet built. Each is alo's own
version, not a copy: it runs through the grants, the record and the person's
choice like everything else.

- ★ **Live capsules for work in progress** — a small live pill in the status area for anything under way, **the agent's work first of all**: *building your site, 3 of 5*, *waiting for your approval*. What the agent is doing, visible at a glance without opening anything
- ★ **A privacy centre with history** — one page answering *who used what, and when*: which applications **and the agent** used the camera, the microphone, files and the network. Built from the record alo OS already keeps, which most systems do not have to draw on
- **Cards from dock icons** — hover over or long-press an application's icon and a small card shows what matters in it and what can be done, without opening it
- **A collection shelf** — gather text, pictures and files from several applications into one shelf, then drop them where they are needed, including on another of the person's paired devices. The agent may fill it when asked, and only then
- **Snap devices together** — drag one paired device's icon onto another's to connect them. A drag is deliberate, and the other device still asks (ADR 0003)
- **One design language on every screen** — when alo's phone applications come, the same design and the same behaviour on the phone as on the PC, adapting to the size of the screen rather than being redrawn for it
- **An alo typeface** covering all 24 official EU languages, drawn once for every screen
- **Motion that feels physical** — animations that move like real objects and never stutter, within the frame budget the shell already holds itself to

## The interface (ADR 0065)

The v1 interface, decided on 2026-09-22 and not yet built. The v0.5 shell is
its foundation and stays as the familiar option a person can keep (Law 5).

- ★ **A recording is a document once its words are text** — a person asks
what is in a video or a call and alo turns its speech into text **on the
machine**, then answers from it exactly as it answers from any document. It says
which road the answer came from: **from the words, not the picture**, so an
answer about a slide nobody read aloud is never given confidently. Frames are
sampled and looked at only where the words are not enough, and fewer of them on
a machine with less to spare. What is not promised is understanding *motion* —
what happened after he stood up — because that is not a claim this hardware can
keep, and a promise nobody can keep is worse than a gap
- ★ **Ask about a recording you never watched, and nothing is read until you
ask** — the question worth answering is *which recording was the one about the
Belgian contract*, across files nobody has opened. Two roads, and the person
picks: **on demand**, where alo transcribes that file when asked, says how long
it will take rather than spinning, and offers to keep the transcript beside it so
the second question is instant; or **a folder the person chose**, worked through
with what it did in the record. **Neither is a background index of somebody's
disk** — context is captured on invocation and nothing reads a person's things
while they are not looking. Transcripts are tiny where video is enormous, which
is what makes the second question and the search across files nearly free
- **Open it at the moment it was said** — *take me to where she talks about
the deadline* finds the words in the transcript and moves the playhead there.
The **manual path is the same feature without an agent**: the transcript sits
beside the recording, and clicking a line moves the picture to it. Asking **why**
she said it is the model reasoning rather than searching, so it **quotes the
passage and the moment it reasoned from** and the person checks it in one click
- ★ **The words that came with the recording are read first** — a great
many videos already carry a subtitle or caption track, and a subtitle track is a
transcript with timestamps that somebody else already made. alo reads those
**before it offers to make its own**, which means *open it where she says it* and
reading along both work **with no model at all**: instantly, with nothing to
wait for, and **in No AI**, where there is no model to ask. A person who declined
AI entirely does not thereby lose the ability to search their own recordings.
Where a track exists and a transcription would be better, the person is told
both are possible rather than one being chosen for them; where the track is in
another language it is still a set of moments, and saying so is better than
ignoring it
- **What the pipeline is asked for before it is built** — three things this
plan needs from whatever decodes a recording, cheap to require now and expensive
to retrofit: seeking that lands on **the moment asked for** rather than the
nearest keyframe; a transcript carrying **timestamps** and not only text; and
**one position control**, which the person's scrubber and alo both drive; plus
**the subtitle and caption tracks a file already carries**, surfaced as text with
their timings rather than only burned into the picture.
[ADR 0009](decisions/0009-a-good-computer-without-the-agent.md)'s
rule is the reason for the third: a second, private way to move the playhead
drifts from the first, and then alo can do something the person cannot
- ★ **Every goal is a canvas** — a Place is an endless surface, and objects **and applications alike open as panels on it**, where the person put them. Arranging is placing, and **nothing is stacked** — no window is buried behind another where the person cannot find it. This line used to add *so nothing is minimised*; the owner's direction of 2026-09-26 withdrew that, and the promise three lines below has recorded the withdrawal ever since while this one went on asserting it. Putting a window aside deliberately is not stacking, and a person asked for both. **Zoom** and **pan** move through it — by pinch, by wheel with a modifier, and by key — and the canvas replaces window switching, tiling and virtual desktops *This was `v1` until 2026-09-30, when the owner put the full canvas experience into the current release. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- ★ **The World, and moving between Places** — zoom out past a Place and every Place the person has is seen at once; zoom into one and it fills the screen. Moving between Places is the same gesture as moving across one, so there is no second way to navigate to learn. **This promise is the World as a level — seeing every Place and navigating between them.** Carrying a frame across is the promise five lines below, which owns it whole; this line used to end by promising that too, and two lines promising one capability is the fault this file keeps finding. *This promise did not exist in this file before 2026-09-30. `docs/decisions/0065` has defined the World since it was written — `World → Place → Object`, and *every Place the person has, seen at once* — and no line here ever carried it, so a decision record described a level of the interface that the only list of what gets built had never heard of. Found while moving the canvas into v0.01 because the owner named World navigation as required for completion and there was nothing to move*
- ★ **Frames, dragged and resized like a design canvas** — a frame shows nothing but its content while the person works, and its name and few controls appear when they point at it, select it or zoom out — the name is also what it is dragged by, so a click inside always belongs to the application. Handles resize it and the application is told its size as it happens, dragging moves it, several can be taken at once, guides and snapping line them up, and each frame carries a name shown when the canvas is far out. Fit the Place to the screen, fill the screen with what is selected, double-click to work inside — each with a keyboard form *This was `v1` until 2026-09-30, when the owner put the full canvas experience into the current release. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- ★ **A frame arrives the shape its work is** — a messaging application opens as a narrow column of conversations, a spreadsheet wide, a video sixteen by nine — declared by the application and remembered per Place once the person changes it *This was `v1` until 2026-09-30, when the owner put the full canvas experience into the current release. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- **Alo working in a window you put aside, and Stop** — a window handed to alo and
  put away still says what is happening to it: **the task it was given**, how far it has got,
  the last thing it did that you approved, and **Stop, always available and never withheld**.
  **Stop cancels the work and not the application** — it acts immediately with no *are you
  sure*, withdraws that run's authority, cancels its pending approvals and queued actions,
  and **leaves the application open, the window put aside, and completed changes kept**,
  because stopping and undoing are different acts. What the row then says is the **actual
  state** and never a hope: *stopping*, *stopped and changes kept*, or *stop requested with
  no confirmation* when the machine cannot confirm. An operation that cannot safely be
  interrupted **says so while cancellation completes** rather than claiming it stopped
  instantly. Stopping **does not revoke alo's access** — revoking is its own act — and
  starting again takes an explicit act by the person. For work spanning several windows,
  Stop targets **the task**, shows its scope before you press it, and updates every window it
  touched. *Added 2026-10-03 by the owner's decision, into the current release, together with
  the Stop design in task 7 of
  [putting a window aside](autonomy/putting-a-window-aside.md). It had been listed here
  nowhere: the nearest lines are* ★ **Give it to alo** *and* **The agent's presence**, *both
  ``, and both promise much more than this — a whole goal handed over under one capsule, a
  teal edge, a named cursor inside applications. **Those keep their tier.** This line names
  only the surface the ruling designed, so that building it does not quietly put the rest of
  v1 in the current release. **What it carves out of* ★ **Give it to alo** *is one phrase —*
  the person may step in, take over a piece or stop it *— and only as it applies to a window
  put aside. Handing a whole goal over, the capsule, and the plan shown first stay at*
  ``*, and the teal edge and the named cursor inside applications stay there too. So
  stopping now appears at two tiers on purpose rather than by drift, and the narrower line
  governs the put-aside surface.* `CLAUDE.md` binds building to this file, which is why the line is
  added before the drawing rather than after it*
- **The canvas is where they left it** — arrange windows across several Places,
  move their cameras, end the session, and a fresh start puts them back. **Place identity,
  frame identity and geometry, each window's presentation state and each Place's camera**
  all survive, and a window that was minimised, compacted or full screen comes back with its
  **normal geometry** intact rather than the shape it was last seen in. Frames stay reachable
  when displays or scaling change. *Added 2026-10-03 by the owner's decision, into the
  current release. It had never been listed here at all — the promise lived only in task 9 of
  [the smallest canvas worth showing](autonomy/the-smallest-canvas-worth-showing.md), which
  read `Done` while nothing wrote a file or read one back, so a person's layout did not in
  fact survive a restart. `CLAUDE.md` binds building to this file, which is why the line is
  added before the work rather than after it. **This is canvas layout persistence and not
  monitor arrangement persistence** — `alo-displays` keeps the physical screen layout and is
  a different thing with a confusingly similar name.*
- ★ **Compact, and minimised — two things, and the person picks** — *compacting* a frame, by dragging it small enough or with one key, leaves a **live tile on the canvas** that still shows what matters (the last messages, the track playing, the build at four of six) and can be acted on without growing back. *Minimising* puts a **preview in the panel at the edge of the screen**, which stays where it is while the canvas moves, and restoring returns the window to its own place on the canvas at the size it had. This line used to say *nothing is swallowed into a bar*, refusing the second outright; the owner's direction of 2026-09-26 keeps both, because a tile that stays visible and a thing put away for later are two different wishes and a person has both in a day. What the older sentence was protecting still holds: **compacting never becomes minimising by accident**, and nothing is put away without the person saying so. *This was `` until 2026-09-30, when the owner moved it into the current release so the panel could be built now. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- **Tidy this canvas** — alignment and distribution for the person who wants them, and the same asked of alo as a proposal shown before anything moves *This was `v1` until 2026-09-30, and it moved because leaving it there had become incoherent rather than because anything new was decided. The frames promise four lines above moved to v0.01 the same day and already says **several can be taken at once, guides and snapping line them up**. So multi-select and snapping were put in the current release while alignment and distribution, the adjacent operation on the same selection, stayed at v1: **one operation at two tiers, made so by the lane that moved the other half.** The laptop lane separately reports the owner naming multi-select, drag preview and arrange-together as v0.01 — recorded as reaching this file through that lane rather than from the owner, so it can be corrected if arrange-together was narrower than either lane read. **The hard part is known and is not the alignment arithmetic:** `alo-put-aside`'s proposing module already shows a placement before anything moves; what does not exist is a proposal that carries a **set** rather than one frame*
- ★ **A frame can be dragged out of one Place and into another**, and the work goes with it — **through the World by pointer, or by keyboard with *Move to Place***, so neither road is the only road. **Restoring a minimised window is not this**: it returns to the Place it was already on, and the view travels there. Nothing is relocated by a restore. *This was `v1.1` until 2026-09-30, when the owner put the full canvas experience into the current release and ruled that cross-Place movement includes both navigation and window transfer. The two were briefly conflated while this was being written — cross-Place **restoration** and a frame **transferred** between Places read as one sentence at a glance and are two acts, and the owner settled that both are in v0.01 and that a restore relocates nothing*
- ★ **A frame simplifies as it shrinks** — application, then compact form, then its name and what it is doing, chosen by how large it is on screen. Zoomed out a person reads *three new from Anna* and *the build at four of six*, never a wall of unreadable miniatures
- ★ **Zones that mean something** — name a region *drafting*, *waiting on Anna*, *done*, and dragging a frame into it does what the name says; a zone can be handed to alo whole
- ★ **A Place remembers time** — drag the ribbon and the canvas is as it was on Tuesday, from the snapshots undo already takes *Moved into the current release on 2026-10-04, when the owner ruled that the complete canvas is one milestone — [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md). The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- **Every screen is a view onto the canvas** — two displays are two viewports at their own zoom, not two desktops; on a small screen, focus shows one frame at a time *Moved into the current release on 2026-10-04, when the owner ruled that the complete canvas is one milestone — [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md). The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- **The habits people arrive with still work** — the keys that cycle windows, close one and switch desktops become cycle frames, remove from canvas, and move between Places *This was `v1` until 2026-09-30, when the owner put the full canvas experience into the current release. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- **A panel out of view costs nothing** — it is a still picture until it is reached, so a Place holding forty things is not forty programs running *Moved into the current release on 2026-10-04, when the owner ruled that the complete canvas is one milestone — [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md). The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- **Every canvas also answers as a list** — its panels in order, by keyboard and to a screen reader, because a surface that needs a touchpad excludes people (EN 301 549) *Moved into the current release on 2026-10-04, when the owner ruled that the complete canvas is one milestone — [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md). The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- ★ **The alo Bar** — ask, find, open, create or hand over, from one place. **It works with no model at all**: applications open, files are found, settings change, arithmetic is exact, commands run
- ★ **Give it to alo** — anything selected can be done by hand or handed over, and **a whole goal can be handed over**: alo shows its plan, works under one capsule, and returns only the decisions that must be the person's. The person may step in, take over a piece or stop it *The stopping clause of this line is carved out at `` for one surface only, by the owner's decision of 2026-10-03: a window put aside shows what alo is doing in it and offers Stop, which is promised in its own line above. **Nothing else here moved.** A whole goal handed over, the capsule and the plan shown first remain ``, and a reader finding stopping at two tiers should take the narrower line for the put-aside surface and this one for everything else. The carve-out is recorded on both lines because the Tidy-this-canvas promise below shows what happens when it is recorded on neither — one operation at two tiers, and a lane left to work out which governs.* *Moved into the current release on 2026-10-04, when the owner ruled that the complete canvas is one milestone — [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md). The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*
- **When the machine moves a window, the person is told** — a frame the machine had to move, because a fixed control came to cover its name, says so: what moved, and **where it was**, so the person can put it back. *Added 2026-10-04 by [ADR 0086](decisions/0086-the-complete-canvas-is-one-current-milestone.md), which names recovery notices as part of the complete canvas. It is **not** the notifications portal and not calm notifications — both stay where they are, because they are the ordinary notification system and the canvas needs one sentence about one frame. The record already exists: `Recovery::BroughtBack` carries where the frame was, and nothing says it*
- ★ **History** — what happened, why, and undo. Agent actions come from the kernel-watched record; a person's own work is shown from file versions, never from watching them
- **Content is the interface** — an open object fills the screen and tools appear when something is selected
- **No dock by default** — the alo key, the bottom edge or a swipe reveals the alo Edge. Its replacement must be found by somebody who has never seen it within thirty seconds, tested with people; a person may pin a dock
- **Which system the machine starts by default** — changed from alo OS's settings or from Windows, kept in one place both can reach (the EFI system partition), so both sides always show the same answer (ADR 0066). alo OS changes it through a verb on the broker's list, never by a person writing under /boot
- ★ **The privacy symbol is always visible and can never be hidden** — *private*, *local activity*, *data leaving*, *camera or microphone on*
- **Notifications are decisions** — *requires you*, *working*, *finished*, and *finished* only for work the person walked away from
- **Every application lives in a Place**, including the ones with ordinary windows, so there is one world rather than a modern half and an old half *This was `v1` until 2026-09-30, when the owner put the full canvas experience into the current release. The tier moved rather than the scope gate being crossed: `CLAUDE.md` binds building to what this file says, so the file is what changed*

## alo's visual language

Adopted by the owner on 2026-09-22 and not yet built. One rule runs through all
of it, so a person can read at a glance what is real, what is proposed, and who
is acting: **a ghost is proposed, solid is real, and deep teal is the agent** —
deep teal always with its mark and its word, never by colour alone.

- ★ **Ghost previews** — before the agent changes anything, a translucent ghost of the result appears in place: the files with their new names, the draft, the edited design. Accepting makes it solid; declining fades it away
- **The agent's presence** — a thin deep teal edge on the window the agent is working in, and its named cursor inside applications: *alo, for Disan*. Where the agent is, always, without a pop-up
- **The trust dial** — ADR 0064's three levels for running code as one dial in the agent's capsule, set per project
- ★ **The time ribbon** — a ribbon at the bottom edge; dragging it back fades the whole desktop into the past, with what changed glowing, and letting go restores what the person picks
- **The approval stack** — what the agent wants to do arrives as cards to accept or decline, each showing a real before and after: marked-up text, two versions of a design side by side, a difference in code
- ★ **Where the data went, on a map** — a small map on which a line is drawn to wherever anything leaves the machine for: *Paris*, *Frankfurt*. On most days nothing moves, and that stillness is the point
- **Explain mode** — hold one key and point at anything, and a card says what it is, what it does and what it can reach. For everyone arriving from another system
- **One bar for everything** — search, settings, commands and the agent are one bar, typed or spoken, and every action says exactly what it will do before it runs
- **Project spaces** — moving between projects rather than applications; the dock becomes the project's tools, with its people, files and the agent's memory of it
- **Calm notifications** — delivered in batches at times the person chooses, summarised by the agent, with a capsule saying how many are waiting

## What makes a person want it

Adopted by the owner on 2026-09-22 and not yet built. Each builds on something
alo OS already owns — the record, undo, the egress indicator, the model on the
machine — and each is a choice the person makes (Law 5), never a thing done to
them.

- ★ **Projects instead of applications** — open *the client's website* and its mail, meetings, tasks, design, code and the agent's memory of it are one place, not six windows. The agent's one project memory made visible
- ★ **Windows applications, and Android applications**, run through a compatibility layer, sandboxed and asking for what they need like any application — the largest single reason a person cannot leave Windows, removed. macOS applications are not promised: Apple's licence forbids it
- ★ **A throwaway box for anything downloaded** — run it in a box that closing erases without trace, built on the same sealed box ADR 0064 gives the agent
- ★ **Scroll the machine back in time** — a timeline to drag back to *Tuesday, 10:32*, see files and settings exactly as they were, and bring back only what is wanted. Built on the snapshots undo already takes
- ★ **Replay what the agent did** — every file it touched and every connection it made, step by step like a recording, with any single step undoable
- ★ **The machine works while the person sleeps** — tasks queued at night run on the machine's own model while it is charging, and the morning brings *here is what I did — approve or undo*. No paid cloud; the person's own hardware
- ★ **Live translation of anything, on the machine** — any window, document, subtitle or call, between the 24 official EU languages, with nothing sent anywhere. It works on a plane
- **"Why did my computer do that?"** — *why is it slow*, *why did the network drop at three*, *what changed since yesterday*, answered with evidence from the record rather than a guess
- ★ **Leave Windows in one evening, and come back if you want** — files, bookmarks, network passwords and the wallpaper brought across; the agent shows where each thing now lives; thirty days in which going back to Windows is one click, on the road *remove alo OS* already provides
- ★ **A monthly privacy receipt** — *this month, nothing you wrote went to an AI company; this much went to system updates; here is every destination*, signed so it can be shown to someone else
- **Settings in one sentence** — *warmer screen after eight, silence during meetings* becomes a set of rules the person can read and change, never hidden behaviour
- **Remember everything, if the person wants it** — a searchable memory of what was on screen, off by default, kept only on the machine and encrypted, with applications that can be excluded, and never readable by the agent without a grant. It is what *context offered, never watched* allows a person to choose, and it records other people's words too, which the switch says plainly
- **The machine suggests a workspace**, it never rearranges one — *switch to your photo editing space?* when a camera is plugged in, and nothing moves unless the person says so
- **Self-healing by going back** — when something breaks after a change, the machine says what happened and offers yesterday's working system in one click, on the rollback the image already has. Never a silent fix

## The person chooses (ADR 0064)

Law 5: on their own machine a person decides, and every protection is a default
they can change with its cost said plainly.

**Being decided and not built is no longer a reason to leave them alone.** The
owner's ruling of 2026-10-14: *permission levels in the ADR should not stop us
to development.* So these are in scope, and the first of them is being built —
which is why the line below says what is done rather than only what is
promised.

*Recorded here rather than only in a commit, because `CLAUDE.md` says scope is
gated on this file: a reader deciding whether something may be built looks
here, and a ruling that lives in a pull request is a ruling they will not
find.*

- ★ **Running code, at the level the person picks** — a sealed box the kernel locks down (the default), asking each time, or full trust. At every level each run is recorded and a snapshot taken first, so it can be undone.
  **Started 2026-10-14.** `alo_capability::ATerminalForTheAgent` is the first
  half of ADR 0111's split — whether an agent may **open** a terminal — and the
  three places that refused one outright now ask the person's choice beside the
  list. **The default is that they have not chosen**, so nothing is loosened
  until something passes the choice. What is still owed: the setting a person
  makes it in, the surface they make it on, and the three levels for *running*
  commands, of which the sealed box needs the kernel boundary
  ([ADR 0013](decisions/0013-the-grant-is-enforced-by-the-kernel.md)).
  **The sealed box runs a command as of 2026-10-16.** `alo-running` is the box,
  in a crate of its own because `alo-bounding` guarantees — with a test over its
  own source — that it starts no program; ADR 0117 is that decision. A command
  runs confined to the folders it was given and the kernel refuses the rest,
  measured end to end on a real kernel in
  `the_kernel_refuses_what_a_run_was_not_given.rs`: the same command, the same
  box, one line different in the bound, reading the file in one case and
  `EACCES` in the other. **The kernel boundary is no longer what this waits
  on.** What is owed after it: the three protections ADR 0064 keeps on at every
  level are the caller's, and nothing calls this yet — the record, the snapshot
  taken first, and the egress indicator — as are the setting and the surface
  above.
  **The setting a person makes it in exists as of 2026-10-22.**
  `/var/lib/alo/how-code-runs.toml`, beside the grants and held to the same
  three rules about who may have written it — because a file somebody else
  could write into saying `at-full-trust` is an escalation to running code
  without asking. `docs/contracts/how-code-runs-file.md` is its contract.
  **It keeps three states apart and that is the point of it**: a level the
  person chose, *no file at all* for a person who has not chosen, and a
  refusal — so neither a machine nobody has opened the setting on nor a file
  that will not read can report the sealed box as somebody's decision. What is
  owed after it is the **surface**: nothing in `alo-shell` or `alo-desktop`
  offers the three levels yet, so the choice still cannot be made by anybody. **And where a box's control group lives on a booted machine is open**:
  it cannot go under `alo-bounding`'s turn root, which was measured rather than
  reasoned about and is in `docs/quirks.md`.
- Update checking can be turned off, saying that security fixes stop arriving while it is off
- *Trust devices on this network*, off by default, saying that it also trusts guests' phones and every device on the network
- A provider address that is not https can be allowed after a plain warning that the key crosses the network readable.
  **Built for the person's own network as of 2026-10-30** (ADR 0021): a private
  address written as numbers is saved after the caution. **Not built for an
  address on the internet**, which is still refused; the owner's decision named
  the home network, and the wider case waits for one of its own.
- A **details** view that shows each rented component's own name as data, for the person who wants to know; plain words stay the default everywhere
- Screenshot-and-click switched on by the person for their own machine
- At full trust, a turn whose kernel boundary cannot be applied may run if the person chooses, told that the kernel is not watching it, and the record says so
- **Fast Startup: the installer asks.** Turning it off is recommended when a disk is shared, and both answers are safe because alo OS never mounts the Windows partition read-write

## Non-goals

**No kernel.** Linux, unmodified — hardware support is where OS projects die and
we do not fight that battle. **No inference kernels** — we do not compete with
llama.cpp or vLLM. **No model training toolchain from us** — we serve and adapt open weights, and building a trainer is somebody else's product. **That is a decision about what we build, never a limit on what the machine allows**: a person with the hardware runs whatever training they like on it, with the tools they choose, because it is their machine and their electricity. **No general-purpose distribution** — no package manager for the world and
no attempt to be Ubuntu; software is installed as sandboxed Flatpaks from
repositories other people run (ADR 0005), which is how we have applications
without packaging them. **No third-party device management** — fleet features
exist for alo OS machines; an MDM product is a different company. **No phone or
tablet** — not in v1, possibly never. **No directory service** — we do not
rebuild Active Directory or LDAP, and we do not become the place a company's
identities live; alo identities and pairing are what we offer. **No trusted-network
setting by default** (ADR 0003) — pairing stays how machines trust each other; a
person may opt in on their own network, told that it also trusts every device on
it (ADR 0064). **No code runs unless the person chose it** (ADR 0064, replacing
ADR 0001 §1): the verb list stays typed and closed, and running code is the
person's grant — a sealed box by default, asking each time, or full trust.

**Every line above is about what we build, and none of them is a limit on what
a person may have.** That distinction is already how this section is written —
*a person with the hardware runs whatever training they like on it, because it
is their machine and their electricity* — and
[ADR 0112](decisions/0112-with-the-persons-permission-the-agent-may-do-anything-on-the-machine.md)
makes it the rule rather than a habit: with the person's permission the agent may
do anything on the machine, and the only things their permission does not reach
are the four law 5 names — protections that restrict nothing, protections that
guard them from others, the law, and an organisation's policy on machines it
owns. **A non-goal that reads as *the agent may never* is a statement about what
is not built**, and a lane that turns one into a wall has found a bug in what it
is writing. ADR 0043 is the worked example of how that happens with every reason
to get it right.

Every absence here is a sales argument.
