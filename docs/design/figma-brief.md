# alo OS — Figma brief

The screens of alo OS, for whoever designs them — a person or a model. It is a
brief rather than a specification: it says what each screen has to do and what
the system must never do, and leaves how it looks to the designer.

Thirty screens for a working system; the twelve in part 1 carry the identity and
are enough to test whether the idea reads. Where a screen and an ADR disagree,
the ADR is the one that has been argued.

Two parts. **Part 1 is what you paste into Claude Desktop**; part 2 is the
detail behind it, to paste as a follow-up if the first pass needs grounding.

In Claude Desktop, with the Figma connector enabled, open a new chat and say
something like:

> Use Figma to design these screens. Create a new file called "alo OS" with one
> frame per screen at 1440×900, plus a small component set for the shared
> elements (status area, the agent overlay, buttons, the source line).
>
> *(then paste part 1)*

---

## Part 1 — the prompt

Design the interface for alo OS — an operating system for ordinary business
laptops whose interface is an agent.

The desktop is deliberately familiar: wallpaper, windows, a launcher, a status
area. People adopting this are leaving Windows 10, so nothing about moving a
window or finding an app should need relearning. Make that part calm and
unsurprising.

The one new surface is the agent overlay. One key summons it over whatever the
person is doing. It is not a window and not an app: no launcher icon, cannot be
minimised, does not take over the screen, and leaves without changing where the
person was. It can answer questions from files they have granted it, and when it
wants to CHANGE something it proposes one sentence describing exactly what will
happen and waits for a single approval. That sentence is the most important text
in the product — give it room, weight and stillness. Never a dialogue people
learn to dismiss.

Every answer says where it was answered: "on this machine", "on the studio
workstation, on your network", or "by Mistral, in the EU" — shown next to the
answer, never hidden in settings.

Palette: navy #102A43 for structure and text, cream #F8F6F2 as the ground, warm
porcelain #F4F1EC for the workspace canvas, charcoal #1F2529 for the rail, warm
stone #7A6F62 for metadata. Terracotta #E76F51 is the accent and appears ONLY
where the agent is present or acting — about five percent of any screen — so a
person can tell at a glance whether the machine is doing something on their
behalf. Type: Inter throughout, EB Garamond for occasional editorial headings.

Design light and dark as equals. Support 24 languages, so no layout may depend
on a label fitting in English. Keyboard operation and visible focus states
everywhere. Approve and decline carry equal visual weight.

Screens, in this order:

1. **Sign in** — wallpaper, one account, one field, nothing else
2. **Lock** — time, date, who is signed in; no notification content
3. **Setup: "Where should your AI run?"** — three choices in plain words (this
   machine / a machine on your network / a provider you add), each stating what
   leaves the machine and what happens offline
4. **Desktop at rest**
5. **Launcher**
6. **Agent overlay, just invoked and empty** — showing what it can currently see
7. **Agent overlay answering** — with its sources and where it was answered
8. **Agent overlay proposing a change** — one sentence, approve or decline
9. **Agent overlay refused** — "that folder isn't granted", offering the grant
10. **Grants** — what is granted, to what, until when; revoke in one action
11. **What happened** — the record of actions and refusals
12. **Models and providers** — installed models with disk cost, add a provider

---

## Part 2 — the detail behind it

### The five principles

1. **Terracotta means the agent.** Navy structure on a warm cream ground.
   Terracotta appears where — and only where — the agent is present or acting.
   Spend the accent nowhere else.
2. **Conventional where it is conventional.** Windows, launcher, settings,
   notifications: excellent and unsurprising. Invention here costs adoption and
   buys nothing.
3. **The approval sentence is the interface.** What a person approves is one
   sentence describing exactly what will happen.
4. **Say where the answer came from, where the answer is.** Not in a settings
   page. Somebody about to paste a contract into a question is entitled to know
   where it is going before they paste it.
5. **Nothing important lives only in a menu.** What is granted, what happened,
   and what left the machine each need a real screen.

### The navigation model — four layers, only the third is new

1. **The session** — sign-in, lock, recovery. Native, sparse, reachable when
   everything else has failed. It must never look like it is waiting to load.
2. **The desktop** — wallpaper, windows, launcher, status area. Ordinary on
   purpose. This is where somebody spends the day.
3. **The agent overlay** — one key, over whatever is in front of you.
4. **Applications** — the alo workspace and anything else installed.

**The idea in one line:** you never navigate *to* the agent — it comes to you,
with the context of whatever you were already doing, and leaves without changing
where you were.

### The first screen

The first screen a person *sees* is **sign-in**, and it should be almost
nothing.

The first screen that *matters* is **"Where should your AI run?"** in setup. No
other operating system asks this, and asking it in the first five minutes is the
whole product thesis delivered before anybody has done any work. Design it as a
choice between three understandable things, not a settings form: each option
says what leaves the machine, what it needs, and what happens if the network is
down. A default is pre-selected — the machine itself, where it can run one — and
no option is presented as the clever one.

### The other eighteen screens (phase two)

13. First boot — language (all 24 EU languages listed in their own language)
14. First boot — region and keyboard (the keyboard offered *with* the language)
15. First boot — account (alo identity, or a local account needing no tenant)
16. Recovery / rollback — reachable when the desktop is not; plain language
17. Window management — snap, tile, switch (states rather than a screen)
18. Notifications, including do-not-disturb the agent respects
19. Status area expanded — network, battery, sound, egress indicator
20. Egress: what left this machine, when, and where to — the auditor's screen
21. Grant request — choosing a folder, expiry chosen at the same moment
22. Model download — progress in gigabytes and time, never a spinner
23. Add a provider — name, address, key; the key goes to the keyring, said so
24. Settings home — one place, not scattered dialogues
25. Making it yours — background per display, lock image, light/dark schedule,
    accent, text size
26. Language and region — switchable without a reboot
27. Privacy and security — policy in words: where inference may happen
28. Updates — atomic, reversible, never interrupting; say when it will apply
29. Printers — unglamorous, and it decides public-sector deals
30. "This machine is managed" — who manages it, what policy applies, that a
    recovery key is escrowed. Shown at first sign-in on a managed machine.

### The palette, verbatim

| Colour | Hex | Role |
|---|---|---|
| Navy | `#102A43` | structure, text |
| Terracotta | `#E76F51` | the agent — about 5% of any screen |
| Cream | `#F8F6F2` | reading ground |
| Porcelain | `#F4F1EC` | workspace canvas |
| Charcoal | `#1F2529` | the rail |
| Warm stone | `#7A6F62` | metadata |

Type: **Inter** for everything a person operates; **EB Garamond** for the few
editorial moments, used sparingly. Tabular figures wherever sizes, times or
progress line up.

### Constraints that will bite

- **Twenty-four languages.** German compounds run long, Finnish longer. Design
  the longest case, not the tidiest.
- **Right-to-left ready**, even though no official EU language needs it.
  Mirrored layouts should be a flip, not a redesign.
- **EN 301 549** — the mandatory accessibility standard for EU public
  procurement. Keyboard-only operation, real focus states, contrast holding at
  200% text.
- **Light and dark are equals.** The terracotta must hold on both grounds.
- **Wallpaper is per display**, and people choose bad ones. Every surface must
  stay legible over anything.
- **No dark patterns.** Approve and decline carry equal weight; a refusal is
  never styled as a mistake.

---

*Written against alo OS as decided so far: ADR 0001 (the capability model),
0002 (the shell is native), 0007 (the CPU is the default) and 0008 (where
inference happens), in `github.com/aloworld-org/alo-os`. Where a screen and an
ADR disagree, the ADR is the one that has been argued.*
