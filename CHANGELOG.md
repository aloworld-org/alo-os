# Changelog

What changed, in words a person outside this repository can read. Written
while the knowledge is fresh, not reconstructed at release time — newest
first.

A line here describes what somebody can now do, or what stopped being
wrong. "Refactored the grant store" is not a changelog line; "a revoked
grant now takes effect immediately instead of at the next sign-in" is.

---

## Unreleased

- The six file verbs now actually do it. A folder is listed, a file is read, a
  search finds things by name in a folder and the folders inside it, a file is
  renamed or moved, and a folder becomes one archive — a zip, which every
  desktop opens without being told how. Four things are true of all of them.
  **Nothing you did not name is replaced**: a rename or a move onto a name that
  is already taken is refused and says so, rather than quietly writing over
  somebody's file. **A grant covers where a file goes, not only where it comes
  from**: if you granted one document, an agent cannot rename it to something
  you never granted. **Nothing follows a link out of what you granted** — a
  search steps over links, and an archive leaves them where they are and tells
  you how many it left. And **every answer says when it stopped**: a listing, a
  read or a search that reached its limit says there is more, because an answer
  that quietly stopped looking reads exactly like one that found nothing.

- The dock goes where you want it: bottom, left, right or top, chosen in
  Settings. It is built for both orientations rather than being a horizontal bar
  turned on its side — the status area reflows, and where the short edge is
  tight, labels give way to icons. Whichever edge you choose, the clock, the
  battery and the indicator saying what has left this machine sit at the far end
  of it.

- An agent can now be asked to do six things with your files — list a folder,
  read a file, find something by name, rename, move, and make an archive of a
  folder — and each one says in ordinary words what it is for and what it would
  do before it does it. Looking answers straight away; renaming, moving and
  archiving wait for you to approve one sentence, once. **And a folder you
  granted covers where your files really are, not where something points.** A
  file inside a folder you granted that is really a shortcut to somewhere else —
  your keys, another person's folder, a disk you did not grant — is refused, and
  refused as what it is: something your grants do not cover, kept in the record
  beside every other refusal. A path you never granted is refused without your
  machine being looked at on its behalf, so an agent cannot learn whether a file
  exists by being told it is missing. There is no way to ask for a file by
  writing a search expression, and there is no verb that deletes anything.

- You can run alo OS with no AI at all. Setup's fourth choice is simply "not at
  all", with the same weight as the other three and nothing trying to talk you
  out of it — and what is left is the whole system: files, windows, printing,
  settings, updates, applications. The agent's surfaces are absent rather than
  greyed out, because a disabled panel is an advertisement, and turning it on
  later is a setting rather than a reinstall. A system sold on control that
  required the agent would not be a coherent thing to sell.
- The screen divides: drag a window to an edge for half or a corner for a
  quarter, or split what is open from the keyboard. The split holds while you
  work — resizing one side resizes its neighbour instead of overlapping it — and
  a pair you return to comes back arranged as you left it.

- What left your machine is now kept alongside what your agents did, so "what
  left this machine today?" is answerable at the end of a week and not only in
  the second the indicator lit up. Every departure is one entry — who reached
  out, where to, whether it was asking, fetching or sending, and when — **and
  a departure can only be written down from one the indicator actually showed
  you**, so the record cannot claim less than you saw or more. A question
  answered somewhere else is that departure rather than a second note beside it,
  which means the count is the number of things that really left rather than a
  number inflated by writing the same event down twice. And an egress your
  organisation's policy refused is kept as a refusal, with where it was going
  and which setting stopped it: a record that only kept what left could not
  answer the question somebody asks having just set a policy, which is what it
  actually stopped. A question your own machine answers is still kept, and it
  still names nowhere, because it went nowhere.

- Anything an agent sends out of your machine now has to be decided about and
  shown while it happens, and not only when it is a question to a model. The
  indicator is a list you can read rather than a light that is on: who is
  reaching out, where to, and whether it is asking something, fetching something
  or sending something. **A machine can no longer be given permission to reach
  out without that appearing** — permission and the indicator line are the same
  act, so a connection that was allowed but not shown is not a state that
  exists. A question your own machine answers is not a departure at all, so a
  day's work on a local model leaves the indicator quiet, which is what the
  zero-egress claim looks like from the inside. If your organisation has said
  where this machine may reach — inside the building, inside a named region, or
  nowhere — it is now one rule covering everything rather than a rule about
  models and a gap around everything else, and being refused says which setting
  stopped it and where it was going. A machine on your own network counts as
  leaving: it went down the corridor, and "it is only our own network" is
  exactly the assumption this refuses to make. An address is checked before it
  can be displayed, so nothing can be named in a way that makes the indicator
  line read as something other than what is happening.

- What an agent did is now kept, and so is everything it was stopped from
  doing. Each entry says what ran, whose authority it ran under, which approval
  it came from and which grant permitted it — and the grant it names is the one
  you can find in your list and take away, so "what had this folder already been
  used for?" is a question you can ask after revoking it. **The refusals are the
  point**: a folder that was never granted, a grant that had expired, a change
  you declined, and a request that was so malformed it never became a request at
  all are all kept, because a record of successes alone cannot answer what
  anybody actually asks of one. Where an answer came from is recorded too — on
  this machine, on a machine on your network, or by a provider — so "where did
  that go?" is answerable at the end of the week and not only in the second it
  appeared. What you *asked* is never recorded, and there is nowhere in an entry
  for it to go: this keeps a note of what your machine did, not a transcript of
  what you said to it. Asking it something is a question you put to it — what
  did this agent do this afternoon, what was it refused, what left this machine
  — rather than a search through text. Nothing takes an entry back out.

- A change an agent wants to make now waits for you, and one approval buys
  exactly one action. What you approve is the sentence describing what will
  happen, and what runs is what that sentence named — the arguments travel
  inside the approval rather than arriving beside it, so nothing can be
  substituted between your answer and the action. An approval cannot be spent
  twice: the question leaves the list the moment you answer it, and answering it
  again finds nothing there. There is no "remember this", no "allow for ten
  minutes" and no "always allow for this application" — durable permission is a
  grant you made deliberately and can take away in one action. Questions expire
  rather than sitting there collecting an accidental click, and something you
  never granted is refused before you are interrupted by a question about it at
  all, because an approval that leads to "actually, no" teaches people to click
  through. Your grants are checked again at the moment something would happen,
  so revoking a folder after approving something still stops it. And a request
  that only answers a question — listing a folder you granted — still simply
  answers, because making a question wait for approval is how people learn to
  approve without reading.

- What an agent can do is now a list, and a closed one: if something is not on
  it, the agent does not have it, and asking for it comes back saying so rather
  than being attempted. Each entry says what it is for in ordinary words,
  whether it answers a question or changes something, what it takes, and what
  has to be granted before it can run. Arguments are checked before anything
  happens: a path that is not a full path, a name with a folder hidden in it, a
  number outside its range, an option nobody offered — all refused at the door,
  with a message saying what to send instead. There is no argument that can
  carry a command, because there is no kind of argument that accepts free text,
  which is what stops a model that has been talked into something from writing
  what runs. And the sentence you would be approving is written by whoever wrote
  the verb, filled in from the checked arguments and from nothing else — a verb
  whose sentence would leave one of its arguments out cannot be added at all,
  because an approval that does not describe what will happen is not an
  approval.

- What an agent may reach is now decided by the grants you made, and by nothing
  else. A grant covers one folder and what is in it, one file, or one
  application; it names the agent it is for; it can be taken away in one action
  and stops on the next question rather than at the next sign-in; and it ends —
  a grant that never expires is not something this machine can hold, because a
  grant that outlives the reason you made it is exactly how a machine stops
  being yours. There is no grant to the whole machine, however it is spelled. A
  folder you granted covers what is inside it and stops there, so granting
  `Invoices` does not quietly hand over `Invoices2` next to it. And asking about
  a file never grants it: an agent that asks a hundred times is refused a
  hundred times, and every refusal says which it was — the grant expired, or you
  never made one — because those need different things from you. This is the
  rule underneath the file and application verbs rather than a settings panel
  you can open yet; the panel comes with the shell.

- You can add your own model provider in Settings — a name, an address and a
  key. Use Mistral, your own endpoint, or whatever you already pay for. The key
  is stored in the keyring and never written into a settings file, so it cannot
  escape through a backup, a log or a support bundle you email somebody. You say
  where the provider runs; nothing is guessed from its address. And an address
  that is not https is refused rather than warned about, unless it is a service
  on this machine — otherwise your key and your questions would travel in clear.

- Which model provider you use is yours to choose — Mistral, alo, your own
  endpoint, or none. alo OS ships no default that decides for you, and where an
  organisation does have a rule, the region is theirs to name: "the EU",
  "Switzerland", "the United States". Built in Europe and not only for Europe; a
  product that hardcoded its own region would make everybody else a special case
  in their own operating system.

- A model can answer from one of three places, and you are always told which: on
  this machine, on a machine on your network, or behind a provider's API — so a
  laptop too thin to run a model, or an organisation that would rather buy
  inference than operate it, can still use alo OS. Where the answer came from is
  said where the answer appears, not in a settings page, because somebody about
  to paste a contract into a question should know where it is going before they
  paste it. A provider that will not say where it runs is reported as unknown
  rather than assumed to be nearby. And a local model that fails never quietly
  becomes an API call: failing to answer is recoverable, your records leaving
  the building because a download was corrupt is not.

- alo OS runs its agents on the CPU by default. A graphics card makes the same
  system faster and makes fine-tuning practical, but it is acceleration rather
  than an entry price — because the Windows 10 fleet this project exists to
  catch has almost no discrete GPUs in it, and a system those machines cannot
  run agents on is a system they cannot adopt. The catalogue now states what a
  model costs in system memory and how it behaves without a card, and carries
  models small enough to answer comfortably on an ordinary laptop: a
  three-billion-parameter model that answers in seconds is a better agent than a
  nine-billion one that answers in minutes, because a turn makes several calls
  and the waiting multiplies.

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
