# Every sentence, and the walk from nothing to a working application

**Date:** 2026-09-16
**Workstream:** v0.5 — software and the web
**Task:** *Every sentence, and the walk from nothing to a working application*
(`docs/autonomy/v0-5-software-and-the-web-plan.md`, task 7)
**Contributor:** Claude Code (Opus 5), checkout `C:\dev\alo-os-2`, gates run in
WSL Ubuntu on the same machine
**Status:** ready for integration.

## What changed, for somebody outside this repository

Every sentence a person can read about installing applications, the machine's
proxy, and what the agent does in an application was read in the order a person
meets it. There is now one test that walks through it: set a proxy, install the
text editor, open a web link, have the agent open a document in the text editor,
remove the text editor. A second test walks the same path and stops at each step
with a refusal a person could meet there. Both tables are below. A test now fails
if any of those sentences changes and its table does not change with it.

Reading them turned up four things, and each is fixed:

- **Two proxy messages had a line break and a run of blanks inside them.** The
  message for an address that is not a proxy's name read *"without http:// in*
  followed by a new line and five blanks, and its translator's note had the same
  flaw. In the source, a string continued onto the next line was missing the
  backslash that joins the lines.
- **Two settings had no words at all.** A mistyped address for a network's
  automatic proxy configuration, or a bad entry in the list of places to reach
  directly, could only be described in English written into the code. Both now
  have their own sentences, each with a translator's note.
- **A refusal told somebody to do something they can't do.** On a machine an
  organisation manages, when the network's rule cannot be read, the message said
  *"Set the proxy here by hand"*. The line just before it had told the person that
  their organisation set the proxy and it cannot be changed. It now says only what
  is true on every machine: *"Ask whoever runs this network for the address of its
  proxy"*.
- **Removing an application didn't say everything that ended.** Permission for
  the agent to use the application ends when the application is removed, and that
  grant disappears from the list of grants, but the message said only that what
  the application itself had been allowed had ended. It now says both.

## The walk

A person's own machine. No proxy is set and nobody has been granted anything.
The only application installed is the web browser this machine ships
(`Shipped::the(Role::WebBrowser)`), so installing the text editor is the first
thing the person does with the machine's software. Every row is exactly what the
machine's code said, in the vocabulary `alo-saying` collects. The key beside each
row is measured (see *Decisions*), not written down by hand.

| Step | Word | What the person reads |
|---|---|---|
| Set a proxy | `proxy.address.carries-a-password` | That proxy address has a name and a password written into it. Type the address on its own, and the name and the password in their own fields — the password is then kept safely and never written into a file |
| Set a proxy | `proxy.the-proxy.manual` | This machine reaches the network through the proxy set here |
| Set a proxy | `proxy.set-by.this-person` | You set this |
| Install the text editor | `egress.itself.installing-an-application` | alo OS is installing an application from dl.flathub.org |
| Install the text editor | `software.installed` | org.gnome.TextEditor is installed. It has been given nothing: it asks when it needs a file, the camera or anything else, and you answer |
| Install the text editor | `granted.nothing-granted` | Nothing is granted right now. No agent and no application has been granted anything on this machine, and there is nothing here to revoke |
| Open a web link | `software.web.refused.nothing-granted` | org.gnome.TextEditor has not been allowed anything on this machine, so the web address was not opened. Allow it the web browser you want it to open addresses in, and it can ask again |
| Open a web link | `granted.one-grant` | org.gnome.TextEditor has been granted the application org.mozilla.firefox |
| Open a web link | `software.web.opens.shipped` | org.mozilla.firefox opens web addresses. It came with this machine, and you can choose another or remove it |
| Ask the agent to use the text editor | `granted.one-grant` | org.gnome.TextEditor has been granted the application org.mozilla.firefox |
| Ask the agent to use the text editor | `granted.one-grant` | @alo has been granted the application org.gnome.TextEditor |
| Ask the agent to use the text editor | `granted.one-grant` | @alo has been granted /home/anna/Notes and everything in it |
| Ask the agent to use the text editor | `adapters.text-editor.open-document.sentence` | open /home/anna/Notes/March.txt in GNOME Text Editor |
| Remove the text editor | `software.removed` | org.gnome.TextEditor is removed. Everything it was allowed has ended, and so has every permission to use it |
| Remove the text editor | `granted.one-grant` | @alo has been granted /home/anna/Notes and everything in it |

What the same test also holds, beside the table:

- **The indicator names where the installation goes, not the proxy it goes
  through.** The road out is decided as going through `proxy.example.com:8080`,
  and the line reads `dl.flathub.org`.
- **Removing puts nothing on the indicator.** The rented tool was asked to do
  exactly two things: install the text editor, then remove it.
- **An application that answered is told nothing beyond what was approved.**
  `Driven::said` returns `None`, and one approval sent exactly one message.
- **After removal, the agent is not offered `text_editor.open_document` at all**,
  and two grants ended: the text editor's grant to the browser, and the agent's
  grant over the text editor.

## Where the walk stops

The same steps, each stopped by a refusal a person can meet there. Each refusal
is shown to have reached nothing.

| Step | Word | What the person reads |
|---|---|---|
| Set a proxy | `proxy.the-proxy.automatic` | This machine asks the network where each connection should go |
| Set a proxy | `proxy.set-by.an-organisation` | Your organisation set this |
| Set a proxy | `proxy.not-changed.an-organisation-set-it` | Your organisation set the proxy for this machine, so it cannot be changed here. Ask whoever manages it |
| Install the text editor | `egress.itself.installing-an-application` | alo OS is installing an application from dl.flathub.org |
| Install the text editor | `proxy.refused.nothing-works-it-out` | This machine cannot read the rule this network publishes, so nothing was sent. Ask whoever runs this network for the address of its proxy |
| Open a web link | `software.removed` | org.mozilla.firefox is removed. Everything it was allowed has ended, and so has every permission to use it |
| Open a web link | `software.web.nothing-opens` | No application on this machine opens web addresses, so nothing was opened. Install a web browser, and it opens them |
| Ask the agent to use the text editor | `software.removed` | org.gnome.TextEditor is removed. Everything it was allowed has ended, and so has every permission to use it |
| Ask the agent to use the text editor | `capability.refused.never-granted` | @alo has not been granted the application org.gnome.TextEditor — grants are made by picking a folder, never by asking for one |
| Remove the text editor | `software.refused.not-installed` | org.gnome.TextEditor is not installed on this machine, so there is nothing to update or remove. Check the application's name |

In row 5, the tool is never asked to install anything. In row 9, the person had
approved opening the document and then removed the text editor before it was
sent. The approval is refused at the moment it would run, and nothing is sent to
the application.

## Sources

- `crates/alo-software/tests/from_nothing_to_a_working_application.rs`, which
  holds both tables:
  - `the_walk_from_nothing_to_a_working_application_is_the_table`
  - `where_the_walk_stops_is_the_table_and_nothing_past_it_happens`
- `crates/alo-software/tests/every_sentence_of_software_and_the_web.rs`, which
  asks one question of all three crates:
  - `every_sentence_is_in_the_machines_vocabulary_with_a_note`: every word
    `alo-software`, `alo-proxy` and `alo-adapters` declare, including the text
    editor's words and the accessibility fallback's, is in the vocabulary the
    machine collects. Each has the same English and the same note, and every note
    is at least five words. Nothing else is under those three areas, and no
    counted sentence is either.
  - `no_sentence_or_note_names_the_machinery`, which looks for Flatpak, Flathub,
    OSTree, AT-SPI, D-Bus, and the browser engines Gecko, SpiderMonkey, WebKit,
    Blink, Chromium, Servo, WebEngine and Trident.
  - `every_sentence_and_note_reads_as_one_line`
  - Two tests that show each check catching what it looks for:
    `the_machinery_check_finds_every_name_it_forbids` and
    `the_one_line_check_finds_a_continued_line_without_its_backslash`.
- `crates/alo-proxy/src/words.rs`: nine new words (five for the address of the
  network's rule, four for the places reached directly), the two line breaks
  fixed, `proxy.refused.nothing-works-it-out` reworded, and a crate-local test
  `every_sentence_and_note_is_one_line_with_single_blanks`.
- `crates/alo-proxy/src/automatic.rs`: `NotAConfiguration` loses its English
  `Display` and gains `EVERY`, `word` and `said`, the shape `NotAnAddress`
  already had. Test: `each_refusal_of_a_configuration_address_is_its_own_sentence`.
- `crates/alo-proxy/src/exceptions.rs`: the same change for `NotAnException`.
  Test: `each_refusal_of_the_list_is_its_own_sentence`.
- `crates/alo-proxy/src/setting.rs`: `TheProxy::said`, alongside its existing
  `word`.
- `crates/alo-software/src/words.rs`: `software.removed` reworded.
- `crates/alo-software/Cargo.toml` (and `Cargo.lock`): `alo-saying` added as a
  dev-dependency.

## Decisions

- **Where the tests live.** `alo-software` already depends on `alo-proxy` and
  dev-depends on `alo-adapters` and `alo-granted`. Putting both files there needed
  one new dev-dependency (`alo-saying`) and no change to either other crate's
  manifest.
- **What *from nothing* means.** A machine with the shipped browser and nothing
  else installed. The walk asks for the application installed to be the same one
  the agent then uses and the person removes, and the reference adapter is for
  the text editor. On a machine with the whole shipped list, the text editor would
  already be installed, and the install step would be `already-installed`. Having
  only the browser keeps the walk truthful without inventing a machine the image
  would never produce, and the test's module documentation says so.
- **The key beside each row is measured, not claimed.** The walk speaks through
  the machine's whole vocabulary, set to prefer a private-use language (`qaa`) in
  which every phrase is its own English with `⟦key⟧ ` in front. The row's key is
  the one at the very front of the sentence. The English is what's left once
  every key is removed: a place named inside the indicator's line carries its own
  key in the middle. A row naming the wrong key fails the same way a row with the
  wrong words does. A sentence that came from no phrase fails with a message
  saying so.
- **Grant sentences in the walk come from the grants list.** The person meets a
  grant on the list; the portal dialogue where a grant is made belongs to the
  applications plan. The walk shows the list at each point where a grant is made
  or ends, and does not construct a dialogue it does not own.
- **Opening the link shows what Settings says opens web addresses.** When a link
  opens successfully, these crates say nothing: the browser's window appears.
  The row is `TheBrowser::said`, the sentence Settings shows, taken from the same
  `Opened` answer, so the table records *which* application opened it.
- **A place's name is data.** The walk installs from the place named `flathub`,
  because the shipped list (`shipped.toml`) uses that name for it, and the
  indicator names its host. No sentence or note names Flathub; the name is filled
  in as the machine knows it, a decision `software/src/words.rs` already made and
  this task does not revisit. `docs/features.md` names Flathub to people itself.
- **The two reworded sentences keep their keys.** No translation exists yet, so
  there is no translated text whose meaning a new key would protect.
- **English left in these crates on purpose.** Every `#[error]` in the three
  crates was read. These remain, because no person using the machine reads them:
  - `alo-software`'s `NotShipped` and `NotConfigured`: the release's own data
    files contradicting themselves.
  - `NotShown` and `NotAnInstallation`: an executor handing over the wrong
    authority or indicator line, as each type's documentation argues.
  - `NotAWebAddress`: a person reaches it only through `NotOpened::said`.
  - `alo-proxy`'s `NotReachable` and `NotAName`: built by code from a road's
    destination, or from a name in a file an organisation wrote.
  - `alo-adapters`' `NotLoaded`: read by an adapter's author.

  The two proxy types a person *types* into the settings panel were the
  exception, and they are fixed.

## For other owners (not changed here)

- **`alo-capability`, `capability.refused.never-granted`.** After an application
  is removed, an approval to use it is refused with *"— grants are made by picking
  a folder, never by asking for one"*. That is wrong for a grant over an
  application: nobody picks a folder for one. It also says *has not been granted*
  about a grant that existed and ended. This plan reads `alo-capability` and does
  not edit it. The wording belongs to that crate's owner, and whether *ended* and
  *never granted* are different refusals is their decision. The walk's second
  table records the sentence as it is today and will change with it.

## Verification

Run in WSL Ubuntu on this machine, from the checkout, with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-2-72aa7fda7f7de151`:

| Command | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy -p alo-proxy -p alo-software --all-targets -- -D warnings` | exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-proxy -p alo-software --no-deps` | exit 0 |
| `cargo test -p alo-proxy` | 88 unit, 4 integration: all pass |
| `cargo test -p alo-software` | 75 unit; 5 + 2 new integration; 12, 5, 8, 4 existing integration; doctests: all pass |
| `cargo test -p alo-saying` | 63 + 4 + 1: all pass (it collects the changed vocabulary) |

The whole workspace suite was not run here, as instructed; the supervisor runs it.

## What this does not claim

- The tables are the English source. No translation exists yet, and the notes are
  what a translator will be given.
- The walk uses the same stand-ins as every other test of these crates: the
  rented tool, the text editor's bus, and the browser. What it would add on a
  machine is what tasks 1 to 6 already name as their on-machine acceptance.
- No *On the machine* box moves.

## Proposed shared-document updates

- **CHANGELOG.md:** "Installing applications, the proxy and the agent's use of an
  application were read as a person meets them, in one walk now held by a test.
  Two proxy messages no longer carry a stray line break. The settings for a
  network's automatic proxy configuration and for places reached directly now
  refuse bad input in translatable sentences. A machine managed by an
  organisation is no longer told to set the proxy by hand. Removing an
  application now says that permission to use it ended too."
- **ROADMAP.md:** none. No release gate moves.
- **QUEUE.md / STATE.md:** v0.5 software and the web, task 7 done; task 8 remains
  blocked on the owner's decision recorded in the plan.
