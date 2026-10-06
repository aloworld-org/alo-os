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

### The arrival screen, word for word — `383:26067`

Read 2026-10-06. **This is the authoritative copy for this screen**, and it
differs from the older `183:*` frames quoted further down, which were read
before the correction.

| slot | words |
|---|---|
| brand | alo OS |
| stage, top right | SET UP · AI CHOICE |
| title | How would you like to work? |
| introduction | Choose where alo runs, or **choose No AI**. |
| choice 1 | **On this computer** — alo works locally. Availability depends on this PC. |
| choice 2 | **A machine on your network** — Selected content goes to a machine on your network. |
| choice 3 | **My provider** — Use your provider. Selected content leaves this PC; provider charges may apply. |
| choice 4 | **No AI** — Open apps, find files and use the full canvas by hand. |
| consequence | Choose one option to continue. You can change this later. External processing requires a visible choice. |
| actions | Continue *(disabled)* · Back |
| bottom left | Accessibility |
| bottom right | Local setup. No online account required. |

**Three things here are not what an earlier reading of this flow would tell
you**, and each matters to whoever builds it:

- **The introduction says *or choose No AI*, not *or continue without AI*.**
  The older frame said the latter, and the difference is ADR 0009's whole
  point: *continue without* is a skip, *choose No AI* is an answer. Use the
  newer wording.
- **The stage reads `SET UP · AI CHOICE`, not `SET UP · 4 OF 5`.** The step
  count is gone from this screen.
- **"Choose one option to continue." is the first sentence of the consequence
  paragraph**, not a separate label beside the disabled control. One paragraph,
  three sentences, in that order. A vocabulary that splits it into three keys
  has to put them back in that order; one that keeps it as a single key cannot
  reuse the middle sentence on the other states, where it also appears without
  the first.

### What the states change, and what they do not

Measured across `383:26067` (arrival) and `383:26127` (network selected):

| | arrival | something selected |
|---|---|---|
| selected card border | — | **2px** navy, and the title gains ` · Selected` |
| unselected card border | 1px `#e7ebef` | 1px `#e7ebef` |
| card | 600×**108**, padding 20 across and 14 down | the same |
| **Continue** | **disabled** — ground `#eef2f4`, text `#596b78`, and **not interactive at all** | navy, interactive |
| actions present | Continue, Back | Continue, Back, **About this choice** |
| consequence | the three-sentence paragraph above | *You can change this later. External processing requires a visible choice.* |

**`About this choice` appears only once something is selected**, which is the
design answering a question the specification did not ask: there is nothing to
explain until there is a choice to explain.

**The disabled control is a non-interactive element in the design, not a styled
link.** It is visible, it is 184×48 like every other action, and it cannot be
activated — which is what *visible but disabled* has to mean for a pointer, a
keyboard and a screen reader alike.

### The provider list — `183:10687`, *12g · Provider list / Arrival*

**This is where ADR 0014 is either kept or broken, and the design keeps it.**
Read 2026-10-06.

| slot | words |
|---|---|
| stage | SET UP · AI CHOICE |
| title | Choose your provider. |
| introduction | Connect when you are ready. **No provider is selected.** |
| provider 1 | **Mistral** — Use your account. Provider charges may apply. |
| provider 2 | **alo in Europe** — Use your account. Provider charges may apply. |
| provider 3 | **OpenAI** — Use your account. Provider charges may apply. |
| consequence | Until connected, you can use the whole computer by hand. |
| actions | Set up later · About this choice |

**Three things make this ADR 0014 kept rather than merely claimed:**

- **alo's service is second, between Mistral and OpenAI** — not first, and the
  order is not alphabetical, so nothing puts it at the top by accident either.
- **Its consequence line is identical to the other two**, word for word. No
  badge, no extra sentence, no styling of its own.
- **Nothing is pre-selected here either**, and the screen says so in words:
  *No provider is selected.*

**The names are illustrative and establish no integration-support matrix** —
the owner's instruction of 2026-10-06 says so, and a reader must not take
Mistral and OpenAI appearing here as a claim that either is supported in the
current release.

**Cards here are 600×92, not 600×108**, because a provider row carries a name
rather than a title and a consequence of the same weight. The four on the AI
choice stay 108 and equal to each other; these three stay 92 and equal to each
other. **Equality is within a list, not across lists.**

**`Set up later` is the primary action**, in navy, which is the screen agreeing
with the promise: a person may leave setup without connecting an account, and
the way out is the most prominent thing on it rather than a link in the corner.

### The *No AI* detail — `194:10787`, *12 · Detail · No AI*

Read 2026-10-06. **The screen that decides whether *No AI* reads as an answer or
as a loss**, and it reads as an answer.

| slot | words |
|---|---|
| stage | AI CHOICE · DETAILS |
| eyebrow | NO AI |
| title | **A complete computer.** |
| introduction | The canvas, applications and ordinary tools remain available. |
| row 1 | **The Bar still works** — Find local files, open apps, calculate and change settings. |
| row 2 | **No agent activity** — No alo tasks are started. |
| row 3 | **Your choice stays yours** — Enable alo later only if you want to. |
| actions | **Choose No AI** *(primary)* · Back to choices |

**It names what the person keeps, never what they give up**, and the third row
is the no-persuasion rule written as copy: *Enable alo later **only if you want
to***. ADR 0009's *no persuasion attached* is observed here rather than merely
promised.

**A name that disagrees with the code, and it is person-facing.** This screen
calls a surface **The Bar**. The code calls it the launcher — `Action::Launcher`
and `words::LAUNCHER` in `alo-shortcuts`. Every *"the bar"* in `docs/` is the
idiom *clears the bar*, not a product name, so this is the first place that
surface has been given that name anywhere in the repository. **One surface
cannot have two person-facing names**, least of all in a product that
externalises every string for translation. Which name wins is the owner's, and
it is not only a setup question — it is whatever the launcher is called
everywhere.

**Detail actions are 160 wide, not 184.** The AI-choice screen's Continue and
Back are 184×48; these are 160×48, and the rows here carry 16 of vertical
padding against the choice cards' 14, with no fixed height. **The
specification's 184 is the choice screen's number** and does not generalise to
the details.

### Confirming a pairing — `384:26324`, *12i · Confirm network pairing*

Read 2026-10-06. **This screen meets all three of the owner's network
requirements**, which is worth recording as observed rather than as owed.

| slot | words |
|---|---|
| stage | SET UP · AI CHOICE |
| title | Connect to Office AI? |
| introduction | Check that this is the computer you intended to use. |
| row 1 | **Machine identity** — Office AI · office-ai.local |
| row 2 | **Processing policy** — Local models on this machine · No provider forwarding |
| row 3 | **Your approval** — Pairing connects the machines. Each task still needs access to its work. |
| consequence | Confirm the matching pairing request on the other computer. |
| actions | Pair computer · Cancel |

- **Identity and processing policy are both shown before pairing**, which is the
  requirement, and they are two separate rows rather than one sentence.
- **Pairing grants no blanket access to work** — row 3 says so in the person's
  own words, and it is the clause that must survive translation.
- **Both ends confirm.** *Confirm the matching pairing request on the other
  computer* is ADR 0003's deliberate link made on both machines, and it is why a
  message arriving from another machine grants no authority by itself.

**`Office AI · office-ai.local` is example data and never real state.** So is
the policy line beneath it: a real screen reads the destination's actual policy,
and a machine that cannot read one says so rather than showing this.

Rows are 600×92 with 14 of vertical padding; both actions are 184×48.

### Accessibility during setup — `375:30435`, *Accessibility · Setup options*

Read 2026-10-06.

| slot | words |
|---|---|
| stage | SET UP · AI CHOICE |
| title | Set up your way. |
| introduction | **Available now, before you create an account.** |
| row 1 | **Screen reader** — Read controls, choices and status aloud. |
| row 2 | **Text size** — Make setup text larger without cutting off controls. |
| row 3 | **More options** — High contrast, reduced motion and on-screen keyboard. |
| consequence | Press **Ctrl + Alt + S** at any time to turn the screen reader on or off. |
| action | Return to setup |

**The stage still reads `SET UP · AI CHOICE`** — the step the person came from,
not a stage of its own. That is the design carrying the requirement that
returning lands back on the originating step with its state, rather than
dropping the person at the start of setup.

Rows 600×92 with 14 padding; the single action 184×48.

### The shortcut the design proposes — checked, and free, with three conditions

**`Ctrl`+`Alt`+`S` collides with no shipped binding.** Measured in
`crates/alo-shortcuts/src/defaults.rs` on 2026-10-06: every shipped chord uses
`Super`, `Super`+`Shift`, `Alt` or `Alt`+`Shift` — the agent on `Super`+`A`, the
launcher on `Super`+`Space`, Settings on `Super`+`I`, close on `Alt`+`F4`,
minimise and maximise on `Super`+`Down` and `Super`+`Up`, snapping on
`Super`+`Left`/`Right`, window and application switching on the `Alt`+`Tab` and
`Super`+`Tab` families, zoom on `Super`+`Plus`/`Minus`, *show all* on
`Super`+`0`. **No `Ctrl`+`Alt` chord is shipped at all.**

Three things follow, and none of them is a design question:

1. **"At any time" is a claim about the shipped set.** A person's binding beats
   a shipped one — `alo-shortcuts` says so in its own header — so somebody may
   already hold this chord.
2. **There is no screen-reader action to bind.** Adding one is a **new
   `Action`**, and the crate prices that: *every action here costs a chord that
   no application on the machine can ever see again.* The change that adds it
   owes the paragraph `docs/autonomy/the-shell-plan.md` task 18 requires — which
   chord, why no existing action serves, and what applications can no longer
   see. `Action::ALL` is iterated in `window_command.rs`, so a new variant is
   held by whatever asserts over that list.
3. **It must work before any account exists**, which the screen itself promises,
   and `alo-shortcuts` keeps **a person's** bindings. A machine-wide chord that
   works before there is a person may not belong to that crate at all. **That is
   an architecture question and this document does not answer it.**

### A long translation — `385:26244`, *12m · Network selected / Long translation*

Read 2026-10-06, and it answers more than the question it is named for.

**The copy is real German, not lorem**, so the growth is a measurement rather
than a guess:

| | |
|---|---|
| On this computer | **Auf diesem Computer** — alo arbeitet lokal auf diesem Computer. Die Verfügbarkeit hängt von seiner Ausstattung ab. |
| A machine on your network | **Ein Computer in deinem lokalen Netzwerk · Ausgewählt** |
| | Ausgewählte Inhalte werden an einen Computer in deinem lokalen Netzwerk übertragen und dort verarbeitet. |
| My provider | **Mein Anbieter** — Nutze deinen eigenen Anbieter. Ausgewählte Inhalte verlassen diesen PC; es können zusätzliche Anbietergebühren anfallen. |
| No AI | **Keine KI** — Öffne Anwendungen, suche Dateien und nutze die gesamte Arbeitsfläche selbst, ohne einen KI-Assistenten. |

**`Selected` translates too — `· Ausgewählt`.** The selection word is not a
decoration appended in the shell; it is part of the translated string, and a
build that concatenated an English *Selected* onto a translated title would be
wrong in every language.

**All four cards grow to 132 together.** 108 in English, 132 here, and the four
stay equal — which is the rule *measure the translated text, then grow every
card to the tallest required height* observed in the file rather than only
stated.

**And the layout changes shape when the content scrolls.** Measured on this
frame, which is **1440×960, the same size as the arrival** — so these
differences are the layout responding, not a different canvas:

| | arrival `383:26067` | long translation `385:26244` |
|---|---|---|
| column | left-aligned, x 160 | **centred** — a 640 scroll container at x 400, the 600 column 20 inside it |
| scrolling | none | `Setup content / vertical scroll`, 640×704 at y 88, scrolls vertically |
| actions | inside the column | **outside the scroll container**, at y 812 |
| Accessibility | y 876 | y 884 |
| footer | y 892 | y 908 |

**The actions and Accessibility sit outside the scrolling region.** That is the
requirement — *scroll the content while keeping navigation and Accessibility
reachable outside that scrolling region* — carried by the design rather than
left to the implementation. Three actions here, each 184×48 at 0, 196 and 392,
so a 12 gap.

**A builder would get the column wrong by assuming one answer.** It is
left-aligned at 160 when the content fits and centred in a scroll container when
it does not; 1440 − 640 = 800, halved, is the 400. Neither position is *the*
position.

### A short screen — `385:26174`, *12k · Arrival / 1280 × 720*

Read 2026-10-06. The frame is **1280×720**, and all four choices are visible
without scrolling.

**What gets smaller, and what does not.** This is the rule a build would
otherwise guess at:

| | 1440×960 arrival | 1280×720 |
|---|---|---|
| heading | 36 on 44 | **28 on 36** |
| card | 600×108, padding 14 down | **600×84, padding 12 down** |
| gap between cards | 10 | **8** |
| gap between blocks | 12 | **8** |
| **choice title** | **16 on 25** | **16 on 25 — unchanged** |
| **choice consequence** | **14 on 22** | **14 on 22 — unchanged** |

**The readable text never shrinks.** The heading comes down, the padding
tightens and the gaps close, and the 16 and 14 of the choice title and its
consequence are untouched. That is *do not reduce user-selected text size to
make content fit* kept by the design, and it tells a build **which knobs it may
turn**: heading, padding, gaps — never body text.

**The column is centred here too**, by the same arithmetic as the long
translation: 1280 − 640 = 640, halved, is the 320 the scroll container sits at.
So centring goes with the scroll container rather than with a particular screen
width. Actions at y 572 and Accessibility at y 644 are outside it, as before.

**The consequence paragraph is the same three sentences** — *Choose one option
to continue. You can change this later. External processing requires a visible
choice.* — at the same 14 on 22. It is not abbreviated for the smaller screen.

### Keyboard focus — `384:26478`, *12j0 · Keyboard focus / On this computer*

Read 2026-10-06. **The frame that shows focus and selection are different
things**, and it shows it by what it leaves alone.

The focused card is **still an unselected card**: 1px `#e7ebef` border, no
`· Selected` in its title, and **`Continue` is still the disabled grey
control**. Nothing about having focus has chosen anything. *Focus movement alone
does not commit a choice* is observable here rather than only required.

**The ring is its own element, drawn outside the card.** The node is named
`Keyboard focus / not selected` — the design names the state where focus sits on
something unchosen:

| | card | focus ring |
|---|---|---|
| size | 600×108 | **608×116** |
| position | x 160, y 205 | **x 156, y 201** |
| border | 1px `#e7ebef` | **2px navy `#102a43`** |
| corner radius | 12 | **16** |

So the ring is **2px navy, 4 outside the card on every side**, which is the
specified *2px navy outer ring with a 4px gap* measured rather than restated.

**Its radius is the card's plus the gap** — 12 + 4 = 16. That is how a ring
concentric with a rounded rectangle is drawn, and a build that reused the card's
12 would show a ring that pinches at the corners.

**Three borders are now in play and they must not be confused:**

| border | means |
|---|---|
| 1px `#e7ebef` | an ordinary, unselected choice |
| **2px navy, on the card** | **selected** — and the title also says `Selected` |
| **2px navy, 4px outside the card** | **focused** — and nothing is chosen by it |

Selected and focused are both 2px navy. They are told apart by **where the
border is** and by **the word in the title**, which is why the word is not
decoration: for a person who cannot distinguish the two by position alone, the
word is the only thing that separates *this is where I am* from *this is what I
chose*.

### All four focus states — `384:26478`, `384:26513`, `384:26548`, `384:26583`

Measured 2026-10-06, **all four rather than one and an assumption**. They are
the same frame with the ring moved, and nothing else differs:

| frame | focus on | ring y | card top |
|---|---|---|---|
| `384:26478` *12j0* | On this computer | 201 | 205 |
| `384:26513` *12j1* | A machine on your network | 319 | 323 |
| `384:26548` *12j2* | My provider | 437 | 441 |
| `384:26583` *12j3* | No AI | 555 | 559 |

- **The ring is 608×116 at x 156 in every one**, always 4 above its card.
- **The step is 118** — 108 of card plus the 10 gap — so the ring tracks the
  card pitch exactly rather than being placed by hand.
- **`Continue` is the disabled grey control in all four.** Focus reaches the
  last choice and the way on is still shut, because focus has chosen nothing.
- **The ring is named `Keyboard focus / not selected` in all four**, including
  the one on *No AI*.

### Focused **and** selected — not drawn, and settled by the rules that are

**No frame among the fourteen read draws a card that is both focused and
selected**, and the state is reached the moment a person picks a choice and tabs
back to it — or simply selects with the keyboard, where focus is *already* on
the card being chosen. **It is the common case, not an edge one.**

**It needs no new visual language, because the two marks never collide:**

| | where it is drawn |
|---|---|
| **selected** | a 2px navy border **inside** the card's own 600×108 box, plus `· Selected` in the title |
| **focused** | a 2px navy ring **outside** that box, 4 away on every side, 608×116, radius 16 |

**So a focused, selected card is both at once, unchanged.** The ring's geometry
does not move, and this is measured rather than assumed: a selected card is
`h-[108px]` with a 2px border and an unselected one is `h-[108px]` with a 1px
border — **the border is drawn inside the box and does not change its size**, so
the 4 gap and the 608×116 ring are identical either way.

**What a build must therefore not do**, each of which would be a reasonable
guess and wrong:

- **Drop the ring because the card is already navy.** Then a keyboard user
  cannot tell which card they are on once they have chosen one.
- **Drop the card's border because the ring is navy.** Then the selection
  disappears while focus rests on it.
- **Thicken or recolour either** to tell them apart. They are already told apart
  by position — inside against outside — and by the word in the title.
- **Reuse the card's 12 radius for the ring.** It is 16, for the reason the
  section above gives.

**The frame is still owed**, and `385:26281` shows the same ring around the
Accessibility action — 168×56 at (36, 872) against a 160×48 action at
(40, 876), the same 4 outside — so the rule is already general across cards and
actions. This section specifies the combination from the design's own rules; it
does not invent one, and a drawn frame should confirm it rather than discover
it.

### Connected — `384:26375`, *13d · Ready / Network machine*

Read 2026-10-06. The end of the network road, and **the screen that could most
easily have overclaimed.**

| slot | words |
|---|---|
| title | Your canvas is ready. |
| introduction | Office AI is connected. **You decide what work to send.** |
| row 1 | **Your account** — Disan · Protected on this PC |
| row 2 | **A machine on your network** — Office AI · **Review or disconnect in Settings** |
| consequence | **Selected content leaves this PC for the paired machine.** |
| actions | Open my canvas · Review choices |

**It does not say *nothing leaves the building*.** The destination is on the
person's own network and the screen still says, in its last line, that content
leaves this PC. That is the requirement kept at the one moment it would have
been easiest to drop — a success screen, where the temptation is to reassure.

**It repeats that pairing is not permission.** *You decide what work to send*
says again at the end what the pairing screen said at the start, and the two
together are what make *each task still needs access to its work* a rule rather
than a sentence on one screen.

**The way out is named on the screen that completes the setup** — *Review or
disconnect in Settings* — rather than left for a person to search for later.

**`Disan` is example data, and it is the owner's own name.** So is `Office AI`.
A build that carried either through would be showing one person's name to
everybody, which is the sharpest form of the rule that **nothing in the
prototype is real state**.

**This screen is a claim the software has to earn.** It says a machine is
connected. Until pairing is implemented, nothing may show it — not with a
placeholder name, not with a hopeful one. `alo_setting_up::NotSetUp::NoPairedMachine`
is what is honest today, and this screen is what replaces it when the pairing
road exists.

## Responsive: the rule for every size, not only the three drawn

**The file gives three widths and two heights. A machine has neither.** These
screens must hold at any size a display reports and at any text size a person
chooses, so what follows states the rule for all of them — **derived from the
three measured frames and the owner's instruction of 2026-10-06, with the
regions nobody has drawn named as such.**

### Width

| available width | the column |
|---|---|
| **≥ 664** | **600, centred.** The scroll container is 640 and sits at (width − 640) ÷ 2 — measured 320 at 1280, 400 at 1440, 960 at 2560. |
| **< 664** | **width − 64**, with 32 each side. No frame draws this. |

**664 is 600 plus the two 32 margins**, and it is where one rule has to hand
over to the other. **The column never stretches** — at 2560 the cards are still
600, not 2.5× wider — because a line of text that long stops being readable,
which is the reason the instruction gives for keeping it fixed.

**The action row wraps before a hit target shrinks.** Three actions at 184 with
12 between them is 576, which fits a 600 column. Below a 576-wide column it does
not, and the rule is **wrap to a second row, never shrink** — every action stays
at least 48 high and keeps its width, because *secondary actions retain equal
reachability* is the component's own contract and a 40-tall button is not a
smaller version of a 48-tall one, it is a worse one.

### Height

**Two mechanisms, in this order**, both measured at 1280×720:

1. **Tighten.** Heading 36/44 → 28/36, card padding 14 → 12, gaps 10 → 8 and
   12 → 8.
2. **Scroll.** The content goes in a 640-wide scrolling region, and **the
   actions and `Accessibility` move outside it** so they are reachable without
   scrolling. That is the design's own arrangement on `385:26244` and
   `385:26174`, not an invention here.

**The readable text is in neither mechanism.** The choice title stays 16/25 and
its consequence 14/22 at every size measured. **A build may tighten and it may
scroll; it may not shrink what a person reads.**

### Text size, which is the case the frames understate

`alo_appearance::TextScale` goes to **200%** — its own doc calls that *the
standard's floor*. **The card heights in the file are examples at one text
size**, and the instruction says so: *these examples are not hard maximum
heights; measure translated and enlarged text, then grow every card to the
tallest required height.*

| measured | card |
|---|---|
| English, 1440 | 600×108 |
| English, short screen | 600×84 |
| German, 1440 | **600×132** |
| **200% text** | **not drawn anywhere** |

**At 200% the consequence line alone is about 44 tall before wrapping**, so a
card carrying a translated two-line consequence at 200% will exceed every height
in the file. **All four still grow together and stay equal** — that is the rule
the German frame establishes, and enlarging text does not exempt it.

### What is not drawn, and must not be guessed quietly

- **Any width below 1280.** The narrow rule above comes from the instruction,
  not from a frame.
- **Any text size above 100%.** The tallest card measured is 132, at 100%.
- **The two together** — narrow *and* enlarged — which is where a phone-sized
  display with large text lands, and where the action row has to wrap at the
  same time as the cards grow.

These are the three places a build will meet a geometry the file never showed
it. **Each should be measured on a machine rather than reasoned about**, and
`docs/autonomy/the-shell-plan.md` task 18's acceptance already insists on that:
the walk is driven from a key or a pointer, with no test in the loop.

### One scaling, at the boundary that already exists

Everything above is **logical pixels**. Display scaling is applied **once**, at
the existing logical-to-physical boundary, and never a second time inside this
surface — two applications of a fractional scale is how a 48 target becomes 47
and stops being a 48 target. `alo-appearance` owns the text scale and
`alo-displays` the display's own; this surface reads them and multiplies
nothing.

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

## Re-read, and no longer stale — `183:10679` and `183:10691`

**They were updated in the file, and this section had them wrong twice over.**
An earlier reading on 2026-10-06 caught them before the correction, showing *alo
in Europe*, `SET UP · 4 OF 5` and *continue without AI*; this document then
marked them as probably-stale history. **Re-read later the same day, both carry
the corrected four**, with *A machine on your network* in second place, no *alo
in Europe*, the `SET UP · AI CHOICE` stage and *Choose where alo runs, or choose
No AI*. The older quotation below is kept only as the record of what moved.

| frame | name now | selected card | `Continue` |
|---|---|---|---|
| `183:10679` | *12 · Choose how alo helps* | On this computer · Selected | **enabled, navy** |
| `183:10691` | *12c · No AI selected* | No AI · Selected | **enabled, navy** |

**`No AI` enables `Continue` exactly like the other three.** It is not a skip
link, not a *continue without* at the side; it is chosen and confirmed on the
same road as every other answer, which is ADR 0009's *same weight* at the level
of what a person actually does.

### This settles whether the arrival sentence is one key or three

**It is at least two, and the first must stand alone.** The consequence
paragraph differs between arrival and a chosen state:

| state | the paragraph |
|---|---|
| arrival `383:26067` | **Choose one option to continue.** You can change this later. External processing requires a visible choice. |
| something selected | You can change this later. External processing requires a visible choice. |

The last two sentences appear **without** the first the moment anything is
chosen. So *Choose one option to continue.* is its own string, shown only while
nothing is selected — it cannot be baked into a single key with the other two,
and a build that did would either repeat it after a choice or lose the other two
before one.

This is what `crates/alo-setting-up/src/words.rs` would need as a new key, and
it is the one piece of that vocabulary work that does **not** wait on the two
questions below: it adds a string rather than changing the meaning of an
existing one, so ADR 0068's retirement rule does not apply to it.

## Read before the correction — kept as the record of what moved

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
moments*, and `docs/autonomy/updates/the-palette-follows-the-design-file.md`
already records
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
