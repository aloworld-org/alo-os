# The first start: what a person is asked, and in what words

**What this is.** The onboarding screens as the canonical design file holds them
today, read on 2026-10-06, with each screen's node id and its copy verbatim.
Written because neither of the two places a reader would look has it.

**Why it is not already in the repository, measured twice:**

- **`figma-snapshot/70-28.xml` carries no copy at all.** 5,897 `<text>` nodes,
  every one of them self-closing: `<text id="189:10709" name="Hero title"
  x="88" y="256" width="570" height="24" />`. Zero occurrences of a
  `characters`, `content` or `value` attribute in the whole file. That is not a
  fault in the export — `get_metadata` returns *ids, names, positions and
  sizes* and nothing else, by design. **An onboarding screen is mostly words,
  and the words were never in the snapshot.**
- **And the snapshot is now behind the file.** Node `183:10679` is
  `14 · Setup / alo on this computer` in the committed export and
  `12 · Choose how alo helps` in the file today, with entirely different
  children. The live screens carry ids in the `370:`/`371:` range; the highest
  prefix anywhere in the snapshot is `358`. None of `371:30239`, `371:30245`,
  `371:30247`, `371:30252`, `370:30053` or `370:30057` appears in it.

**So a lane drawing onboarding from the committed snapshot would build a design
that no longer exists, from a file that never held the words.** `MANIFEST.md`
says the snapshot exists *so that a design change shows up in this repository as
a diff instead of as nothing* — and the thing it was built to prevent has now
happened to it. Refreshing it is a separate and larger job; this document is the
onboarding part, read directly from the file.

**Two of three machines have no Figma access**, so whoever draws these screens
needs this document rather than the file.

## Settled by the owner, 2026-10-06

**All three questions below were put to the owner and answered, and none of the
answers changed a decision — each one keeps a decision the documents had drifted
from.**

1. **The four are the four**, as `THE_FOUR` has them and in that order: on this
   computer, on another machine in your office, from a provider you add, not at
   all. **alo's own service is not a box**; it appears inside *from a provider*,
   beside the others. That is ADR 0014 §4 word for word, kept rather than
   revisited.
2. **A machine on this network stays on the screen.** The owner added that a
   person may also be able to choose it later in Settings; that is a separate
   surface and is not what first start shows.
3. **Nothing is pre-selected on arrival**, per ADR 0025 and ADR 0014 §4's *no
   default and no pre-selection*. The design file's frames each show a state
   *after* a choice is made — `12` with the first selected, `12a · My provider
   selected` with the second — which is what one frame per state looks like, not
   a screen that arrives chosen. **The file has no frame of the arriving state
   with nothing selected, and it needs one.**

**Where the divergence came from, since it is not the designer's fault.**
`figma-brief.md` listed **three** choices for this screen — this machine, a
machine on your network, a provider you add — and left out *not at all*. ADR
0009 had added it before that brief was written. A brief one choice short
produces exactly what the file holds: four boxes where the missing one has been
filled, reasonably, with something that is not ours. The brief is corrected as
of 2026-10-06.

## The conflict as it stood, kept for the record

**The design offers `alo in Europe` as a choice of its own. ADR 0014 says it is
not one.**

`crates/alo-setting-up` is the setup flow as a tested value, and its four are
fixed in `THE_FOUR`:

| `Offered` | the design's choice | |
|---|---|---|
| `OnThisMachine` | **On this computer** | agrees |
| `OnAMachineOnThisNetwork` | — | **missing from the design** |
| `FromAProvider` | **My provider** | agrees |
| `NotAtAll` | **No AI** | agrees |
| *(no variant, deliberately)* | **alo in Europe** | **conflicts** |

`offered.rs`'s own header is explicit about the one that conflicts:

> **alo's own service is not a fifth.** ADR 0014 makes it **one more provider**,
> with no default, no pre-selection and no special case anywhere in the code. So
> it is inside `Offered::FromAProvider` and has no variant of its own, exactly as
> `alo_choosing::Picked` has none — and its absence here is that decision being
> kept rather than an oversight.

And the one that is missing is `OnAMachineOnThisNetwork` — ADR 0003's one box
serving an office — which the crate says *a compositor drawing it cannot leave
out without removing a line of this crate, where a reviewer would see it*.

**This is a decision, not a drawing detail, and it is the owner's.** Either the
design is right and ADR 0014 is revisited with new facts, or the design predates
it and the screen changes. Drawing it as designed would put alo's own service in
a position the ADR forbids, and would quietly drop a choice the crate requires.
Nothing here proposes an answer; `CLAUDE.md` says read the ADR before proposing
an alternative, and relitigating without new facts wastes the scarcest resource
we have.

## The AI choice, as corrected — the specification

**Owner, 2026-10-06.** Section `383:26066`, arriving at `383:26067`. Every node
id in this flow is in `figma-snapshot/ROOTS.md`.

### The four, in this order, equal in size, typography and weight

| | title | consequence |
|---|---|---|
| 1 | **On this computer** | alo works locally. Availability depends on this PC. |
| 2 | **A machine on your network** | Selected content goes to a machine on your network. |
| 3 | **My provider** | Use your provider. Selected content leaves this PC; provider charges may apply. |
| 4 | **No AI** | Open apps, find files and use the full canvas by hand. |

**No AI is a complete answer** — not a skip, not a fallback, not an invitation
to persuade later. **alo's own service is not among them**: it appears after *My
provider* is chosen, as an ordinary provider beside others, with no badge, no
special styling, no automatic selection and no privileged ordering (ADR 0014).
Provider names and ordering in the file are illustrative and establish no
integration-support matrix.

### Arrival

- **Nothing selected, and no processing preference written.**
- **Continue is visible and disabled**, explaining *"Choose one option to
  continue."*
- **Back and Accessibility stay usable.**
- **Enter does not advance** while there is no selection.
- **Unset and the explicit *No AI* are different in the data model.** They
  already are: `SettingUp::selected` is `Option<Offered>` and starts `None`,
  `Offered::NotAtAll` is its own variant, and answering with nothing selected is
  `NotSetUp::NothingSelected` rather than a quiet default — with a test,
  `setup_opens_with_nothing_selected_and_writes_nothing`. **The model was never
  the gap; nothing on a machine draws it.**

### Selection and focus are separate

Selecting enables Continue, and is shown by a navy border **and the word
`Selected`** — never colour alone. **Teal stays reserved for alo acting** and
must never mean *chosen*.

Keyboard focus is **a 2px navy outer ring with a 4px gap**, and is its own
thing: **moving focus must never silently commit a processing choice.** Back
navigation preserves an explicit choice; a genuinely new setup starts without
one.

Kept: *"You can change this later. External processing requires a visible
choice."*

### Measured geometry, from `383:26067` on 2026-10-06

Read from the file rather than taken from the brief:

| element | size | position |
|---|---|---|
| main column | 600 wide | x 160, y 112 |
| each choice card | **600×108**, all equal | 10 between cards |
| Continue · Disabled | **184×48** | x 0 of the action row |
| Back | **184×48** | x 196 — a 12 gap |
| Accessibility | 160×48 | x 40, y 876 |

- **Small-height English:** equal **600×84** cards and a **28px** heading, with
  all four visible at 1280×720 (`385:26174`).
- **Long translation:** equal **600×132** cards (`385:26244`).
- **These are examples, not maximums.** Measure translated and enlarged text,
  then grow **every** card to the tallest required height — they stay equal.
- **Never truncate** a title, a consequence or the `Selected` label, and **never
  reduce text the person chose to enlarge** to make content fit.
- **Wide screens:** keep the centred 600 column, do not stretch (`385:26208`).
  **Narrow:** use the width with 32 side margins. **Short, or enlarged text:**
  scroll the content while navigation and Accessibility stay reachable outside
  the scrolling region. Reflow action rows without shrinking hit targets.
- **Apply display scaling once**, at the existing logical-to-physical boundary.

### Network processing — what the software must not claim

The design shows the finished product: discovery (`384:26276`), identity review
and pairing (`384:26324`), connected (`384:26375`). **It is not to be replaced
in Figma with an unavailable or lesser option** — the design is the destination,
and a limitation drawn into it outlives the limitation.

In software, **report the real connection state**. Pairing is not implemented;
`alo_setting_up::NotSetUp::NoPairedMachine` refuses it in words today, and that
refusal disappears when pairing lands without the design changing. **Do not fake
discovery, pairing or success**, and do not let a temporary runtime explanation
become the permanent design.

- Before pairing, **show the destination identity and the processing policy that
  applies**. Pairing grants no blanket access to a person's work, and a message
  from another machine grants no authority.
- **Do not promise "nothing leaves the building" because the destination is
  local.** That needs an enforced local-only processing policy. Sending to a
  machine on the network **is** data leaving this PC and uses the same
  disclosure and recording as any other destination.
- **No silent fallback**, to another machine or to a cloud provider.

### Provider — choosing a route is not connecting an account

Kept: **Set up later**, **About this choice**, *"Until connected, you can use
the whole computer by hand."*, *"Local setup. No online account required."*

**Store the route and the connection as distinct states.** Postponing the
connection must reach a usable canvas **without** reporting that AI is
connected, that local AI was selected, or that the person chose *No AI*. Example
account, machine and connection details in the prototype are never real state.

### Accessibility, before an account exists

It must work from Welcome onward, independent of account creation
(`375:30435`, `384:26446`, `385:26281`, `387:26226`).

- Accessible names carry **both** the choice's title and its consequence.
- Expose selected/unselected **and position within the group**.
- On arrival, announce the heading **and that nothing is selected**.
- Say **why** Continue is unavailable.
- Visible focus on every interactive control; everything keyboard-operable.
- Accessibility stays reachable **before any setup decision**, and returning
  from it comes back to the originating step **with its state**.
- Accessibility preferences **survive setup and carry into the account**.

**The design proposes `Ctrl`+`Alt`+`S` for the screen reader. Check the shortcut
contract for a conflict before registering it** — `alo-shortcuts` prices every
chord, and silently overriding another action is not open to this surface.

**The file's keyboard reactions demonstrate the interaction; they do not prove
native screen-reader support.** Implement through the OS accessibility and
shortcut systems and test with assistive technology.

### This is native Rust, not a web page

Use the existing appearance and layout systems. The React and Tailwind in any
`get_design_context` output is a visual target, never an implementation.

## The screens, as read on 2026-10-06

Node ids are the file's own, per `ROOTS.md`: *anything written from this snapshot
carries node ids. A number may accompany an id; it may never replace one.*
Each screen is 1440×960 in the design.

### `183:10673` — 08 · Welcome to alo OS

| slot | words |
|---|---|
| brand, top left | alo OS |
| step, top right | WELCOME |
| eyebrow | YOUR COMPUTER. YOUR WAY. |
| title | alo. |
| body | Let's make this computer yours. A few choices, then your first canvas. |
| action | Let's begin |
| bottom left | Accessibility |
| bottom right | Local setup. No online account required. |

### `183:10679` — 12 · Choose how alo helps

| slot | words |
|---|---|
| step, top right | SET UP · 4 OF 5 |
| eyebrow | INTELLIGENCE WHEN YOU WANT IT |
| title | How would you like to work? |
| body | Choose where alo runs, or continue without AI. |
| choice 1 | **On this computer · Selected** — alo works locally. Availability depends on this PC. |
| choice 2 | **My provider** — Use your provider. Selected content leaves this PC; provider charges may apply. |
| choice 3 | **alo in Europe** — Use alo's hosted service. Selected content leaves this PC; service charges may apply. |
| choice 4 | **No AI** — Open apps, find files and use the full canvas by hand. |
| footnote | You can change this later. External processing requires a visible choice. |
| actions | Continue · Back |
| bottom left | Accessibility |
| bottom right | Local setup. No online account required. |

**The selected state is a word, not only a colour** — `On this computer  ·
Selected`. The component's own description says why: *Selection uses navy and a
word; teal is reserved for alo acting.* That matches what `alo-shell` already
holds about deep teal meaning the agent, and it is also what makes the selection
legible without colour.

**`SET UP · 4 OF 5` contradicts `THE_FOUR` being unselected.** The design shows
this screen arriving with `On this computer` already selected;
`alo-setting-up`'s `SettingUp` is explicit that `selected` is `None` until the
person selects one and that *there is no way to build this value with one
already in it*, which is ADR 0025's *nothing is pre-selected*. A screen that
arrives with one chosen is that rule being broken at the surface. **Whether the
design means pre-selection or only shows a state after a click is not something
this document can tell from one frame**, and it is worth asking before drawing.

## What is in the file and not yet read

Named here so the next reader does not have to discover the list. These are the
2026-10-04 snapshot's names, which the renumbering above has already changed for
at least two of them, so treat each as an id to visit rather than a title:

`183:10675`, `183:10683`, `183:10687`, `183:10691`, `183:10695`, `183:10699`,
`183:10703`, `183:10707`, `183:10711`, `183:10715`, `184:10697`, `184:10714`,
`185:10702`, `185:10736`, `185:10770`, `186:10705`, `186:10737`, `194:10709`,
`194:10735`, `194:10761`, `194:10787`, and the entry screens `83:890`
(11 · First start) and `176:10686` (13 · Sign in).

## The type and the geometry

Every screen uses **Manrope**. The styles the two read screens carry:

| style | family, weight, size / line height, tracking |
|---|---|
| Display/Medium | Manrope SemiBold 36 / 44, −0.8 |
| Heading/Medium | Manrope SemiBold 22 / 30, −0.2 |
| Body/Large | Manrope Regular 16 / 25, 0 |
| Body/Medium | Manrope Regular 14 / 22, 0 |
| Label/Medium | Manrope SemiBold 13 / 18, +0.1 |
| Label/Small | Manrope SemiBold 11 / 16, +0.3 |

`figma-brief.md` says *Inter throughout, EB Garamond for the few editorial
moments*, and `updates/the-palette-follows-the-design-file.md` already records
that **the brief and the design file disagree about the typeface**. These
screens are the file's side of that disagreement, and it is unresolved.

**The frames are 1440×960 and a compositor draws at the display's real mode.**
Nothing here says how that translation is made; the dock and canvas notes in
this directory set whatever precedent exists, and it should be followed rather
than invented.

**The action target is 48 logical pixels**, from the component description:
*Explicit setup action. Named labels; 48 logical pixel target. Secondary actions
retain equal reachability.* The last clause is a rule, not a note — `Back` is the
same size as `Continue`.
