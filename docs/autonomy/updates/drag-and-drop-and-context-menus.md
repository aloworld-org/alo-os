# Drag and drop, and context menus

**Date:** 2026-09-17
**Workstream:** v0.5 — hands on the desktop
(`docs/autonomy/v0-5-hands-on-the-desktop-plan.md`, task 4)
**Contributor:** Claude Code, in `C:\dev\alo-os-b`
**Status:** ready for integration

## What changed, in a sentence a person outside this repository can read

Dragging something out of one window and letting it go in another now has a
decided answer: a drop carries the same text, images and files that copy and
paste carry, a window tells you what letting go would do before you let go, and
a file dropped on a sandboxed application reaches it through the documents
portal rather than as a path that would mean nothing inside its sandbox. A file
dropped onto the agent's own panel is offered **for that one question and is not
a grant** — the agent never gains lasting access to the file or the folder it
was in. And a right-click now opens a closed list of actions alo OS wrote, every
one of which a person can also reach from a shortcut, an application's own
menus, the files window or Settings; exactly one row involves the agent, it says
so in the words a person reads before choosing it, and on a machine with no
agent that row does not exist at all.

Nothing draws yet. These crates decide; the compositor will honour them.

## Two crates, not one — and why

The plan names the crates it owns (`alo-dividing`, `alo-desktops`,
`alo-keyboards`) and task 4 fits none of them, so the crate was this task's to
choose. **A drop and a menu share no type**: a drop is a payload, a target and a
delivery; a menu is a subject and a closed list of sentences. One crate holding
both would have a `lib.rs` that said *two things*, which is the shape CLAUDE.md
law 4 exists to prevent. So:

- **`crates/alo-handing`** — what a person's hand hands over. `dragging.rs` (one
  drag, let go once), `target.rs` (a window, the agent's surface, or nothing),
  `over.rs` (what letting go would do, while it is still held), `delivery.rs`
  and `uri_list.rs` (bytes on the spot, or files to export), `handed.rs`,
  `for_one_turn.rs`, `refusing.rs`, `words.rs`.
- **`crates/alo-menus`** — what the thing under the pointer offers. `subject.rs`,
  `action.rs` (the closed list of fourteen), `also_by.rs` (where else each is
  reached), `menu.rs`, `choosing.rs`, `refusing.rs`, `words.rs`.

Both are registered in `crates/alo-saying` (dependency, `EVERY_LIST`, a
`declare` call, `ONE_STRING_EACH` and the count), which is what
`crates/alo-collected` holds every word-declaring crate to.

## The decisions this task made

**A payload is `alo-clipboard`'s, throughout.** A `Drag` carries an
`alo_clipboard::Offer` and a `Box<dyn Gives>` — the forms a window can produce
and the way back to it — so text, images and files are one mechanism here
exactly as they are on the clipboard, and nothing in either crate switches on
what sort of thing is being moved. Nothing is read from the source window until
somebody lets go: `Drag::over` answers *what would happen if I let go now* with
the application never woken up.

**A drag happens once, and the compiler says so.** `Drag::let_go_on` takes
`self`. A drag that could be let go twice would be a move that happened twice —
the failure `alo_clipboard::Clipboard` retires a cut offer to prevent — and here
it is not a rule that is checked but a program that does not compile. Abandoning
a drag is dropping it.

**Files to a sandboxed window go through the documents portal; everything else
goes straight over.** A path is meaningless inside a Flatpak, so a drop that
handed one over would arrive as a file that is not there, and a compositor that
made it be there would have turned a drag of the wrist into a grant over
somebody's home folder. `Delivery::ThroughTheDocuments` names the files to
export and **carries no bytes at all**, which is what makes *no path of the
person's crosses the boundary* a property of the type rather than a claim. What
the portal then calls the exported copies is the portal's to decide, and nothing
here pretends to know it. A `text/uri-list` with no `file:` URI in it — a link
dragged out of a browser — has nothing to export and goes straight over.

**`text/uri-list` is read here, and only here.** A paste never looks inside a
payload; a drop must, because which files are being dropped is what decides the
delivery. So `uri_list.rs` reads that one form and checks every line the way
anything from a client is checked: a `file:` URI, on this machine, absolute, no
`..` step, escapes that are escapes, under 64 KiB. `%C3%BC` is decoded, because
*Müller* is a test case in a European product. A list that cannot be read
delivers **none** of its files: three out of four would be a drop that lost one
without saying so.

**A drop on the agent is not a grant, structurally.** `alo-capability` is a
**dev-dependency** of `alo-handing` and of nothing that ships, so there is no
`Grant`, `Grants` or `Reach` linked into the crate to make one out of. What the
agent gets is a `ForOneTurn`: the forms and the way back to the source window,
read **once** (it takes `self` and is not `Clone`) and only **while the question
lasts** (the moment it ends is carried in the value, so a daemon that forgets to
throw one away still has an offer that answers nothing). No path is extracted
and nothing is exported. `tests/a_drop_on_the_agent_is_not_a_grant.rs` reads the
manifest, and then does the drop in front of a real `Grants` and finds it empty.

**A menu's list is closed, and every entry has another road.** Fourteen actions
live in `action.rs`; no application adds a row, rewords one, or puts its own
sentence in the system's chrome. `Action::also_reached_by` is a `match` the
compiler holds to every variant, answering from a closed list of roads that
exist — and where the road is a shortcut it is named as `alo_shortcuts::Action`
names it, so an entry pointing at a shortcut this machine does not bind is not a
thing anybody can write. ADR 0009 read in the direction it is usually forgotten:
a capability that lives in a right-click is one that somebody driving the
machine from the keyboard does not have.

**One entry reaches the agent, and only over a file or a selection.** A window
and the desktop offer no road to it at all: what a person had open or had
selected is something they chose, and a window they happened to be looking at is
not (`alo-context`'s distinction, ADR 0001 §4). With
`TheAgent::NotOnThisMachine` the row is **absent, not greyed out** — ADR 0009,
*a greyed-out feature is an advertisement* — and the rest of the menu is
byte-for-byte what it was, so declining an agent costs a person nothing else.

## Acceptance, and the test for each

| The plan's acceptance | Workspace · crate · target · test |
|---|---|
| a drop carries what copy and paste carries — text, images, files — through `alo-clipboard`'s payload types | `.` `alo-handing` `a_drop_carries_what_a_paste_carries` `a_drop_carries_the_same_text_images_and_files_a_paste_does` |
| the target application receives it through the portal a sandboxed application expects | `.` `alo-handing` `a_drop_carries_what_a_paste_carries` `a_file_dropped_on_a_sandboxed_window_arrives_through_the_documents_portal` |
| **refusal:** a window that takes none of the forms is refused rather than converted, and is not asked for a byte | `.` `alo-handing` `a_drop_carries_what_a_paste_carries` `a_window_that_takes_none_of_the_forms_is_refused_rather_than_converted` |
| dropping a file onto an agent's surface is not a grant — no grant exists afterwards | `.` `alo-handing` `a_drop_on_the_agent_is_not_a_grant` `no_grant_exists_after_a_file_is_dropped_on_the_agents_surface` |
| …offered as context **for that turn only** | `.` `alo-handing` `a_drop_on_the_agent_is_not_a_grant` `what_was_dropped_is_offered_for_that_question_and_no_longer` |
| a context menu is a closed list of actions the thing under the pointer offers | `.` `alo-menus` `a_menu_is_a_closed_list_of_actions` `every_menu_offers_only_actions_from_the_closed_list` |
| …each an action a person could reach another way (ADR 0009) | `.` `alo-menus` `a_menu_is_a_closed_list_of_actions` `every_action_a_menu_offers_is_reachable_another_way` |
| no menu entry sends anything to the agent without the person choosing the entry that says so | `.` `alo-menus` `no_entry_reaches_the_agent_unless_it_says_so` `choosing_any_other_entry_offers_the_agent_nothing` |
| **refusal:** with no agent on the machine, no menu mentions one | `.` `alo-menus` `no_entry_reaches_the_agent_unless_it_says_so` `with_no_agent_no_menu_mentions_one` |

The refusal paths are tested beside the legitimate ones throughout: letting go
over nothing, a source window that quit between the pick-up and the drop, a form
nobody offered (refused **without the application being woken up**), a list of
files that cannot be read in seven different ways, a question that has ended,
and a menu left open over a file that has since been deleted.

## What the second pass on this task fixed

The first pass handed this work over and the supervisor's gates refused it, at
`git rebase origin/main` rather than at a test. Nothing was reset: the tree came
to the second pass exactly as the first left it, with the rebase stopped on its
second commit and both sides intact. Two things were wrong, and both were the
kind of break that only appears once the change meets a `main` that moved under
it.

- **The two root manifests conflicted, and both sides were right.** `Cargo.toml`
  and `Cargo.lock` each had one hunk where this change's `alo-menus` landed on
  the same line as `main`'s newer `alo-media-server`. Resolved by keeping both —
  the workspace member list gains two names, and the lockfile gains two package
  entries rather than one replacing the other.
- **`alo-saying`'s vocabulary list had the wrong length, and it was arithmetic
  nobody could have got right from either side alone.** `EVERY_LIST` and
  `ONE_STRING_EACH` carry their own length in the type, on purpose: "a crate
  added to one and not the other is a count that no longer proves anything."
  This change wrote `55` because it was written against a base where the list
  held 53 names. The commit it now sits on top of — *A split: halves and
  quarters that hold*, this same plan's task 1 — had added `alo-dividing` as a
  fifty-fourth name while leaving its own constants at `53`, so it did not
  compile on its own either. Rebased together the list holds 56 names, and both
  constants now say `56`. The compiler names the exact number, which is what
  that design is for; no list was edited to make a number fit.

Nothing else changed. No test was altered, deleted or relaxed, no crate gained
or lost a dependency, and the two new crates are byte for byte what the first
pass wrote.

## Verification

Run on 2026-09-17 in WSL Ubuntu against `/mnt/c/dev/alo-os-b`, in the
foreground, exit codes read. The first list is the first pass's; the second pass
re-ran the gates below it after the rebase, and those are the ones that stand.

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -p alo-handing -p alo-menus -p alo-saying -p alo-collected -- -D warnings` — clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-handing -p alo-menus --no-deps` — clean.
- `cargo test -p alo-handing` — 27 unit, 8 + 3 integration, 1 doc; all pass.
- `cargo test -p alo-menus` — 14 unit, 5 + 3 integration, 1 doc; all pass.
- `cargo test -p alo-saying` — 63 unit, 4 integration, 1 doc; all pass (the two
  crates are collected, and the count of what the machine says still equals the
  sum of what its crates say).
- `cargo test -p alo-collected` — 8 unit, 11 integration; all pass (no crate
  declares words that nothing collects).
- Each of the nine tests in the table above run on its own with
  `cargo test -p <crate> --test <target> -- --exact <name>`; each passes alone.

The workspace suite was **not** run here: the supervisor runs it after this
task.

Re-run after the rebase was resolved, same machine, same day:

- `cargo fmt --all` — clean, no file rewritten.
- `cargo clippy --all-targets -- -D warnings`, the **whole workspace** — clean.
  Run whole rather than per-crate because the break this pass fixed was a crate
  the change touches indirectly, and a narrower clippy is how it was missed.
- `cargo test -p alo-handing`, `-p alo-menus`, `-p alo-saying` — all pass.
- Each of the nine tests in the table above run on its own with
  `cargo test -p <crate> --test <target> -- --exact <name>`, and four further
  refusal tests with them — `a_list_of_files_that_cannot_be_read_delivers_none_of_them`,
  `nothing_that_ships_in_this_crate_could_make_a_grant`,
  `choosing_something_this_menu_never_offered_does_nothing` and
  `a_window_and_the_desktop_offer_the_agent_nothing_to_choose`. Thirteen runs,
  thirteen exit codes of zero.

Nothing physical: neither crate touches hardware, and nothing in either draws.

## Limitations

- **Nothing draws, and nothing is wired to Wayland.** The data device, the
  drag-and-drop cursor, where a menu opens and how it is read aloud are the
  compositor's, and the shell plan's later tasks. What exists is the decision
  both will honour.
- **The documents portal export is decided, not performed.** `alo-portals` has
  no documents-portal implementation yet; `Delivery::ThroughTheDocuments` names
  the files and the application, and whatever carries it out is that crate's
  work. This task deliberately did not add a portal to it.
- **A menu's fourteen actions are the ones v0.5 can honour.** *Move to another
  desktop* waits on task 3's `alo-desktops`; *Share* waits on a v1 promise.
  Adding a row is a string this crate declares, which is a change somebody makes
  deliberately.
- `alo-handing` has no drag **source** side beyond `alo_clipboard::Gives`: which
  window started a drag, and whether a drop on its own window is a no-op, are
  the compositor's to know and were not invented here.

## Proposed updates for the integration owner

- **CHANGELOG.md:** "Drag and drop between applications: a drop carries the same
  text, images and files copy and paste carry, the pointer says what letting go
  would do before you let go, and a file dropped on a sandboxed application
  reaches it through the documents portal rather than as a path. A file dropped
  onto the agent's panel is offered for that one question and grants it nothing
  (`alo-handing`). Context menus: a closed list of actions, each reachable
  another way, with exactly one row that involves the agent — absent, not greyed
  out, on a machine that has none (`alo-menus`). Nothing drawn yet."
- **ROADMAP.md, v0.5 *Input*:** *drag and drop* and *context menus* have their
  decided half; gestures, virtual desktops and keyboards remain. Nothing is
  ticked *on the machine*.
- **QUEUE.md / STATE.md:** hands-on-the-desktop task 4 done, marked in the plan.
  Task 2 is still blocked on the session-and-displays plan's task 3; tasks 3, 5
  (which depends on 3) and 6 are ready; task 7 waits on the rest.
