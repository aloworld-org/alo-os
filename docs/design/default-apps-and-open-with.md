# Default apps, and *Open with*

**Approved by the owner on 2026-11-12**, in a question dialog, quoted as chosen:
*"Approve both as drawn (Recommended)"*. Two frames in `XbSmsBYix0TrcQ0Qzaup64`:

- `25:2` *08 / Settings · Default apps* —
  [`figma-snapshot/default-apps-XbSmsBYix0TrcQ0Qzaup64-25-2.xml`](figma-snapshot/default-apps-XbSmsBYix0TrcQ0Qzaup64-25-2.xml)
- `25:52` *09 / Open with…* —
  [`figma-snapshot/open-with-XbSmsBYix0TrcQ0Qzaup64-25-52.xml`](figma-snapshot/open-with-XbSmsBYix0TrcQ0Qzaup64-25-52.xml)

Both XML files were read with `get_metadata` on the frame's own id on
2026-11-12 and written unchanged. **They hold geometry and layer names only.**
The type, colour and control figures below come from two places, and each line
says which: the frame geometry, or the Settings measures already recorded in
[`the-settings-window.md`](the-settings-window.md), which both frames are drawn
with. Fills and type were **not** re-read off these two frames.

**What it is for.** Links open in the person's default application with no
permission asked (the owner, 2026-11-12: *"No — open in my default, but record
it (Recommended)"*). These two surfaces are where the person chooses that
default, and the one moment an application may ask them to choose for one
link.

## 08 — Settings · Default apps

**A new category, *Default apps*, under *This computer*, between *Appearance*
and *Region and time*.** In the frame's side column (`25:8`) the rows are 44
apart — *Appearance* at 329, *Default apps* at 373 (chosen, ground `25:17`,
236 × 40 at (16, 364)), *Region and time* at 417 — the same pitch the other
categories already have.

The page (`25:22`, 720 wide at (316, 36) inside the window body):

| | Frame | Figure |
|---|---|---|
| Heading | `25:23` | *Default apps*, at 0, 36 tall |
| Introduction | `25:24` | *Which app opens a link, an email address or a kind of file.*, at 44, 25 tall |
| First section label | `25:26` | *Web links*, at 109, 18 tall |
| Its card | `25:27` | 720 wide at 135; rows of 72 (`25:28`, `25:35`), 146 for two rows with a one-unit border |
| Second section label | `25:42` | *Email addresses*, at 313 |
| Its card | `25:43` | 720 at 339, one row, 74 |
| Footnote | `25:51` | *Every link an app opens is written in your record: which app asked, the address, and which app opened it.*, at 421 |

**A row** (`25:28`), 72 tall, measured off the frame:

| | Frame | Figure |
|---|---|---|
| Ring | `25:29` | 22 × 22 at (20, 25) — the Settings choice ring (`the-settings-window.md`, *A choice*, `550:170`): not chosen, 1.5 at radius 9.25 in `text/muted`; chosen, 2 at radius 9 in the accent |
| Chosen dot | `25:30` | 10 × 10 at (26, 31) — radius 5, as that ring's dot |
| Application's icon | `25:31` | 28 × 28 at (58, 22); corner radius 8 (the coordinator's reading of the approved frame) |
| Name | `25:33` | at (102, 14), 22 tall — 14 SemiBold |
| Line under it | `25:34` | at (102, 36), 22 tall — 14 Regular, muted |

What each row's line says, from the frame: the default **now** says so —
*Opens web links now*; another installed handler says *Installed*; the mail row
says *Opens a new message to the address*.

**Only installed handlers are listed** — an application whose desktop entry
declares `x-scheme-handler/http` or `…/https` for *Web links*, and
`x-scheme-handler/mailto` for *Email addresses*. **A section with none says so
in a plain line instead of an empty card**, and no row is ever greyed (the
coordinator's relay of the owner's approval, 2026-11-12). A terminal is never
listed: no link is handed to one (`alo_applications::link_opener`).

**Choosing writes `[links]` in `what-opens-what.toml`** (`docs/contracts/person-settings.md`)
— `web` or `mail` — and nothing else. The footnote is literal: every link
opened is in the record (`docs/contracts/portal-answers-file.md`,
`link-opened`).

**The frame's intro block above the window** (`25:3`) is the concept card's
caption — *Which app opens what you open.* — and is not drawn on the machine.

## 09 — Open with…

**Drawn by the desktop, never the application, and only when an application
sets `ask`** on `org.freedesktop.portal.OpenURI`. Without `ask`, a link opens
in the default and no sheet appears.

The sheet (`25:57`), 480 × 332 in the frame:

| | Frame | Figure |
|---|---|---|
| Title | `25:58` | *Open with*, at (25, 23), 30 tall |
| Line | `25:59` | *&lt;App&gt; asked to open &lt;address&gt;*, at (25, 67), 432 wide |
| Card | `25:60` | 432 wide at (25, 103), rows of 72, the same row as 08's |
| Default row's line | `25:67` | *Your default for web links* |
| Other row's line | `25:73` | *Installed* |
| Buttons | `25:74` | 430 × 48 at (25, 263): *Cancel* (`25:75`, quiet) at 312, *Open* (`25:77`, navy) at 378 |

**Behaviour, from the frame's own note (`25:79`):** arrows choose, Enter
opens, Escape cancels and nothing opens; focus starts on the person's default;
**nothing here changes the default** — that stays in Settings → Default apps;
chosen or cancelled, the link is written in the record either way.

## Built, and not

**08 is built** (`crates/alo-shell/src/settings_default_apps.rs`, its page
`settings_page_default_apps.rs`). Two things it does that the frame does not
say, each a judgement written down here rather than left in a diff:

- **The side column's icon is borrowed.** Frame `25:8` draws its side column
  with no icons at all, for any category, while the approved Settings window
  draws one beside each. *Default apps* borrows the window's own file sign
  (`icons/settings/file.svg`) until the designer draws one.
- **A section with no handler** says *No app on this computer opens web links.
  Install a browser and it will be here.* (and the same for email addresses)
  in a plain line under its heading. Where the email section has no card, the
  record's footnote stands beneath it on its own.

**09 is not built yet.** Until it is, `ask` uses the default and the record
says the application asked to choose (`docs/contracts/portals.md`, `OpenURI`).

**Figures every screen size has to derive, not copy** (`CLAUDE.md`, *nothing is
built to one screen size*): the 72 row, the 22 ring, the 28 icon and the 480
sheet are logical measures, taken through each display's scale; the 720 page
width is the Settings content column's existing rule, not a new constant.
