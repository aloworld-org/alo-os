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
  desktop (`asking_github.rs`). Until the sheet is drawn, a sign-in's code and
  page are said on the Accounts page.
- `crates/alo-desktop`: running gh (`signing_in_to_github.rs`) and reading what
  it says (`what_gh_says.rs`), tested against a gh written down in the tests for
  every failure.
- **The sheet itself is the next change**, and with it the sheet opening when a
  publication or a command alo ran fails with *not signed in to GitHub*.
