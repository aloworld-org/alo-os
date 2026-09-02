# Changelog

What changed, in words a person outside this repository can read. Written
while the knowledge is fresh, not reconstructed at release time — newest
first.

A line here describes what somebody can now do, or what stopped being
wrong. "Refactored the grant store" is not a changelog line; "a revoked
grant now takes effect immediately instead of at the next sign-in" is.

---

## Unreleased

- alo OS will speak all 24 official EU languages — Maltese and Irish included,
  not "English plus the big five", because a sovereignty product that skips a
  member state's language is selling sovereignty to some Europeans and not
  others. Each language brings its regional formats, its timezones and a
  keyboard layout offered alongside it, and the agent answers in whichever
  language you asked in.

- A machine can be made yours: set the background from a file, a folder that
  rotates or a plain colour, per display; a lock-screen image of its own; light
  and dark, following the time of day if you want; an accent colour the whole
  shell follows rather than one button; and text scaling, which is an
  accessibility setting as much as a matter of taste. Settings become one place
  rather than a scattering of dialogues you have to know the name of. Later you
  will be able to simply ask — "make the background this photo" — under the same
  propose-then-approve as any other change.

- Models can now be listed, downloaded, removed and taken out of video memory,
  over the runtime alo OS pins. Downloads report progress as they go, because
  these are gigabytes and twenty silent minutes is indistinguishable from a
  hang. Only models the catalogue offers can be fetched, so the licence promise
  holds at the point it would otherwise be bypassed. And what is on disk is kept
  distinct from what is loaded in video memory — different questions, different
  costs, and the pair a person needs to answer "why is nothing else fitting?"

- The first code: alo OS knows which models it offers, and what may legally be
  done with each. Every entry states its licence and answers the commercial
  question outright, and the catalogue refuses to load an entry that claims
  conditions apply without saying which — a model an organisation may not use
  commercially, offered under a tidy licence name, would be worse than not
  offering it at all. European models are listed first. The catalogue is data,
  so adding a model is an edit rather than a release.

- Software can be installed, which the feature list had never said. Applications
  arrive sandboxed from Flathub or a repository the organisation runs, and reach
  the system through the XDG Desktop Portal interfaces every Linux application
  already speaks — so existing software works here without knowing we exist. And
  it is not a second permission system: a portal request is a grant, so there is
  one list of what has been granted to what, agents and applications together,
  revoked the same way. With it comes the plumbing that list implies — a
  keyring, session management, corporate proxy support, and the portals for
  screenshots, capture, camera, printing, clipboard and the rest.
- The ordinary desktop is written down: copy and paste, keyboard shortcuts a
  person can change, screenshots and screen recording, notifications, a file
  manager, Bluetooth, audio that switches mid-call, keyboard layouts with dead
  keys, a screen reader, and a terminal — because law 2 restrains the agent, not
  the machine's owner. None of it is a differentiator and all of it is required;
  a system with a brilliant agent and no working Bluetooth is not a product.
  With one addition that is ours: a visible indicator whenever the screen,
  camera or microphone is in use, by any application including our own.
- The things everybody actually complains about become answerable, because an
  agent can reach the machine: where that file is, in your own words and without
  anything being uploaded; why the machine is slow and what is filling the disk;
  printers that get found, set up and fixed; a file you cannot open, converted
  or plainly explained; updates that never interrupt; a replacement machine made
  like your old one — and, the one nobody else offers, undo everything the agent
  did this afternoon.
- A large organisation can deploy alo OS without abandoning how it already
  works: sign-in against its own identity provider, smartcards and national
  eID, policy by role, a curated model catalogue and signed adapter allowlist,
  agent actions in its own SIEM, staged updates and an internal mirror, key
  escrow so a machine survives its owner leaving, and a signed egress
  attestation to hand an auditor. A machine is personal or it is managed, and a
  managed machine says so at first sign-in — no silent enrollment, no
  administrator watching a screen, no acting in somebody's name.
- Machines on a company network will find each other with no configuration, so
  one GPU workstation can serve every desk and the inference stays in the
  building — working with no internet at all. Discovery is open and trust is
  not: using another machine takes a deliberate pairing on both, and an agent
  reaching across only ever acts under a grant made on the machine it is acting
  upon. Being on the same WiFi confers nothing.
- alo OS has its constitution, its capability model and its contracts. No
  code yet: the decisions that have to hold before anything is reviewable,
  written down first. The load-bearing one is that an agent reaches the
  machine only through enumerated verbs over granted paths, and that no
  verb runs an arbitrary command.
