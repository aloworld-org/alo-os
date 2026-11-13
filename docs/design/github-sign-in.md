# Signing in to GitHub: Settings → Accounts and the sign-in sheet

**Approved by the owner on 2026-11-13**, with pictures, asked and answered:
*"Approve both as drawn"* — the GitHub sign-in sheet and Settings → Accounts,
exactly as drawn in the proposal (`sheet-sign-in`, `sheet-signed-in`,
`accounts-signed-out`, `accounts-signed-in`, at 1× and 2×, and
`accounts-icon.svg`). The proposal's script names every measure with where it
was read; they are recorded here so they can be checked without it.

## Two rulings where the pictures and the owner differ

1. **No dimming** — *"No dimming, like Open with"*. The proposal drew the
   desktop behind the sheet dimmed (`#07131F` at 24 %), which is a figure the
   proposal invented: no frame in the design dims behind a sheet. The sheet is
   drawn over the desktop as the *Open with* sheet is, and nothing behind it
   changes colour.
2. **The side column scrolls** — *"The side column scrolls"*. Accounts is the
   tenth entry, and ten do not fit a 1280 × 800 screen; the column scrolls
   there rather than hiding or shrinking anything
   (`the-settings-window.md`, *Where the side column does not fit*).

And where it goes: **Accounts is under *This computer*, after Default apps.**

## Settings → Accounts

Measured at 1440 × 960, inside the Settings window as
[`the-settings-window.md`](the-settings-window.md) measures it; every figure is
a logical measure, scaled per display.

| Item | Measure |
|---|---|
| Side column entry | *Accounts*, a person drawn in the side column icons' line style: 20 frame, stroke 1.45833, round ends (`crates/alo-shell/icons/settings/accounts.svg`) |
| Heading | *Accounts*, Heading/Large 28 / 36; 8; *Services this computer is signed in to, and what each sign-in lets programs do.* Body/Large 16 / 25 `text/secondary` |
| Section | *Services you sign in to*, the section heading; the card, radius 12, border `border/default` |
| Row | *GitHub*, name SemiBold 14 / 22; 4; *Not signed in*, or *Signed in as* the login, Body/Medium `text/muted` |
| Button | *Sign in* or *Sign out*: the Quiet button, 48 tall, radius 12, label + 26 each side, at the row's end |
| What it risks | the row's risk box, `bg/canvas`, radius 10, 12 across, 10 down, warning sign 18 — the owner's words below, said **whether or not anybody is signed in** |

**The owner's words for the risk, as drawn:** *This sign-in lets GitHub's own
command, gh, read and change every repository your GitHub account can, public
and private, and create new ones. It is kept on this computer until you sign
out. Anything you run can use it — and at full access, so can alo.*

Two things the pictures do not draw, decided by what they do draw:

- **Before the desktop has said who is signed in**, the line reads
  *Checking…* and there is no button. A *Sign in* that might be a *Sign out*
  would be a button that lies.
- The signed-in picture puts a check in `status/positive` before *Signed in
  as*. **Not drawn yet**: a row's line has no sign today, and adding one is the
  page renderer's change, not this one's. Recorded here so it is not lost.

## The sheet

Drawn by the desktop like the *Open with* sheet
(`crates/alo-shell/src/the_open_with_sheet_drawn.rs`): 480 wide, 25 inside, the
title Heading/Medium 22 / 30, 14 under it, 11 above the card, 14 above the
buttons, 8 between them; corner 18 and the FLOATING shadow.

- **Signing in**: *Sign in to GitHub*; *GitHub will ask for this code. Enter it
  on the page that opens in your browser.*; the code in a card (radius 12,
  `border/default`, 16 + 48 + 16 tall, 20 across) in the design's mono at 32
  with a Quiet *Copy* button; *What it risks*; then *Cancel* (Quiet) and *Open
  github.com/login/device* (Primary, navy), each label + 18 a side.
- **Signed in**: *Signed in to GitHub as* the login; a check in
  `status/positive` and *Signed in. You can sign out in Settings → Accounts.*;
  *What it risks*; *Done*.

## How it works, and what it never does

- **GitHub's own command does the signing in**: `gh auth login --web`. alo
  reads the one-time code from gh's own words and **fills nothing in**; the
  person types the code at GitHub. **There is never a password field.**
- gh is not sandboxed and the link portal refuses unsandboxed callers, so **the
  desktop opens the page itself**, in the browser Settings → Default apps says
  opens web links, the way a link a person followed is opened — and only a page
  on `https://github.com/`, whatever gh printed.
- **Sign out runs `gh auth logout`.** Who is signed in is read with `gh config
  get -h github.com user`, which does not reach GitHub.
- **Every sign-in, sign-out and failure is in the record** (the record's
  `at-github` entry, `docs/contracts/record-file.md`): gh missing, gh's own
  error, no code arriving, the person cancelling, a sign-out that failed.
  Reading who is signed in is not recorded; it changes nothing.

## What is built, and where

- `crates/alo-shell`: the Accounts page (`settings_accounts.rs`,
  `settings_page_accounts.rs`), the ask and the answer between the shell and the
  desktop (`asking_github.rs`).
- `crates/alo-shell`: **the sheet** — what it shows and what a key or press on
  it does (`the_github_sheet.rs`), drawn (`the_github_sheet_drawn.rs`), read to
  a screen reader (`the_github_sheet_as_read.rs`), reached by keys and the
  pointer (`a_key_reaches_the_github_sheet.rs`), what it asks carried out
  (`the_github_sheet_on_the_server.rs`), its words
  (`the_github_sheets_words.rs`), and *Copy* putting the code on the clipboard
  (`the_shell_copies.rs`). It opens when the person presses *Sign in* on
  Settings → Accounts, and by itself when a command alo ran failed because
  nobody is signed in (below). The code is no longer said on the Accounts page; the
  sheet shows it.
- `crates/alo-desktop`: running gh (`signing_in_to_github.rs`) and reading what
  it says (`what_gh_says.rs`), tested against a gh written down in the tests for
  every failure.
- **The sheet opening when a command alo ran fails with *not signed in to
  GitHub*** (built 2026-11-13): `alo-agentd` tells each desktop watching
  (`the_desktops_told_github_needs_a_sign_in.rs`, the protocol's
  `watch-what-needs-a-github-sign-in`), the desktop hears it on a thread of its
  own (`hearing_that_github_needs_a_sign_in.rs`), and the shell puts the sheet
  up by the road *Sign in* takes (`the_github_sheet_rises_by_itself.rs`). See *When
  the sheet opens by itself*, below.
- **Not built yet: the same for a publication.** `publish_to_github_pages`
  (`feat/publish-a-folder-to-github-pages`, unmerged on 2026-11-13) answers a
  publication nobody signed in for as a refusal, not as a command's answer, so
  it does not pass the road above. Its refusal needs one call to
  `TheDesktopsToldGitHubNeedsASignIn::a_sign_in_is_needed`, made where that
  branch refuses — the day it lands.

## When the sheet opens by itself

Decided 2026-11-13 from the promise (*appears when alo can't publish because
you're not signed in*) and from the *Open with* sheet's rule
(`the_open_with_sheet.rs`: one up, others wait behind it, never for ever):

- **It waits for the person's press.** Asked on 2026-11-13 whether the sheet
  that rises by itself should start `gh auth login --web` and open GitHub's
  page at once, or wait, the owner chose **"Wait for my press"**: *the sheet
  opens and explains, with a "Sign in" button; nothing starts and no browser
  page opens until the person presses it.* So it rises in a **first state of
  its own**, drawn in the sheet's own parts (480 wide, 25 inside, the title
  14 above the line, 14 above the risk and the buttons):
  - the title *Sign in to GitHub*;
  - *A command alo ran did not work because you are not signed in to GitHub.
    Sign in, and then ask alo again: alo won't try again by itself.*;
  - *What it risks*, the owner's words, as on every state;
  - *Cancel* (Quiet) and a navy *Sign in*, the keyboard on *Sign in*.

  ***Sign in* moves it to waiting for the code** and only then asks gh, and
  from there it is the sheet as drawn. *Cancel* and Escape close it and ask
  gh nothing — not even to stop — so nothing is written down as a sign-in
  that was cancelled, because none began. **Sign in on Settings → Accounts is
  a press already**, so that road still starts at once, waiting for the code.
- **Once per failure, and never stacked.** A failure while the sheet is up is
  let go — it is already asking. Many failures before the next frame are one
  sheet.
- **Never over another sheet.** While the password sheet or *Open with* is up
  it waits, and goes up the frame that sheet comes down — **at most five
  minutes** (*Open with*'s own bound), after which it is let go rather than
  rise over something the person has moved on from.
- **Only for a command this machine ran** for the person's own turns. A paired
  machine's turns run commands as the sealed box, which tells nobody.
- **The person may dismiss it**: *Cancel* and Escape stop the sign-in, as from
  Settings.
- **alo never runs the command again by itself.** What the person reads — on
  the card at *ask me each time*, in alo's reply at full access, in the
  terminal — is *It did not work: you are not signed in to GitHub. Sign in from
  Settings → Accounts, or type gh auth login --web …, and then ask alo again:
  it does not try again by itself.* A command run again after the sign-in would
  be a second run nobody approved.
- **The record keeps the failed run and the sign-in as two entries**, as it did
  before: *the agent ran a command, and it did not work*, and *you signed in to
  GitHub*. **Not built: a record line that says to ask again.** The record's
  entry for a run keeps its status and not what it wrote, so it cannot tell a
  GitHub failure from any other; saying it there needs a new field on that
  entry (`docs/contracts/record-file.md`), which is a record-format change of
  its own.

## How the sheet behaves, where the pictures are silent

- **Waiting for the code** — the moment between *Sign in* and gh printing a
  code, which the proposal does not draw. The sheet is up at once, its card
  says *Checking…* (the Accounts page's own word for waiting) and it offers
  *Cancel* alone. *Copy* and *Open* appear with the code: before it there is
  nothing to copy and no page to open, and a button that does nothing is a
  button that lies — this page's own ruling about *Sign in* before the answer.
- **The keyboard**: Tab and the arrows go through the buttons in the order
  drawn and round again; Enter or Space presses the one ringed; Escape is
  *Cancel*, or *Done* once signed in. The keyboard starts on the navy button.
  Every key and press is the sheet's while it is up, so nothing reaches
  Settings behind it — the *Open with* sheet's rule.
- **The ring** around the button the keyboard is on is `text/primary`, a
  hairline drawn 3 outside the button's edge, so it shows on the navy button
  as well as on a quiet one.
- ***Open github.com/login/device*** names the page gh printed, without
  `https://`; the page opens by the road the code's own page took, only on
  `https://github.com/`. A code that arrives after the person cancelled opens
  nothing.
- ***Copy*** makes alo the owner of the clipboard: any application can paste
  the code, and alo's own fields paste it too.
- **A sign-in that did not work takes the sheet down**, and why is said on
  Settings → Accounts, where every other answer about GitHub is said. The
  *Open with* sheet has no failure state either; the proposal draws none, so
  none was invented.
- **A display that cannot hold the sheet** stops the sign-in rather than hold
  every key with nothing on the screen — the password sheet's rule.
- **The quiet buttons have a `border/default` edge**, as the proposal draws
  them. The *Open with* sheet's *Cancel* is drawn without one; that is the
  older sheet's departure, not this one's.
