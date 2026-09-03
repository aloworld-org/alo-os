# Changelog

What changed, in words a person outside this repository can read. Written
while the knowledge is fresh, not reconstructed at release time — newest
first.

A line here describes what somebody can now do, or what stopped being
wrong. "Refactored the grant store" is not a changelog line; "a revoked
grant now takes effect immediately instead of at the next sign-in" is.

---

## Unreleased

- **alo OS can now put a question to a model.** Until this change nothing in
  the system did: it knew where an answer may come from, what your
  organisation's rule permits, what to do when the place you chose cannot
  answer, and how to show you what is leaving — and nothing joined those up.
  A question can now go to a provider you added, over https, and come back.

  **You see it go.** The indicator says *@mail is asking a question of Mistral,
  in the EU* while it happens, in your own language, and the line is up before
  the connection opens rather than after. If your machine's rule does not permit
  it, **nothing is sent at all** — not a connection, not a name lookup — and you
  read why in the rule's own words. The rule is asked at the moment the question
  would leave, so one tightened this morning is in force this afternoon.

  **The answer knows where it came from.** *by Mistral, in the EU* travels with
  it, so the sentence beside an answer is one nothing can forget to show: there
  is no way to hold an answer without it.

  **And nothing is ever asked somewhere else on your behalf.** A question that
  fails is a question that failed: you are told what happened, told outright
  that nothing was sent anywhere, and asked — once, about one question — whether
  to try somewhere else. A provider that answers by pointing your machine at a
  different address is refused rather than followed, with your question and your
  key still on this side of it.

  What is not built yet is the same path to a model on your own machine, and
  nothing here has been run against a provider anybody pays for.

- **You can have alo OS with no agent at all, and turning it off takes the
  agent's reach with it that second.** Setup's question about where your AI
  should run has a fourth answer — *not at all* — and it is a setting rather
  than a different edition: you can change your mind either way, whenever you
  like, and nothing is reinstalled.

  Turning the agent off ends **every** grant on the machine in one act. Not
  suspends: ends. The folder you picked in March and the document an invocation
  handed over five minutes ago both stop being reachable on the next question
  asked, and if you turn the agent back on in June what comes back is an agent
  with nothing granted — not June's agent holding March's folders. While it is
  off, nothing can be granted at all, because there is no list for a grant to go
  onto.

  **The record and the "something is leaving this machine" indicator stay.**
  They are not AI features, and somebody who declined an agent may want more
  than average to know what their machine did. So a machine with no agent still
  writes down every errand it ran on its own — signing you in, fetching a model,
  checking for an update — and you can now ask your record the one question that
  claim rests on: *is there anything in here that an agent did at all?*

  And if something does ask on a machine with no agent, it is refused and
  written down, in a sentence that says this machine has no agent rather than
  telling you to go and grant a folder in a panel your machine deliberately does
  not have.

- **What your machine did on its own is now in the record too — and it is the
  one entry with nobody's name on it.** The indicator already showed alo OS
  reaching the network for the three things it does with nobody having asked:
  signing you in, fetching a model, checking for an update. That answered *is
  anything leaving right now*. It did not answer *what left last Tuesday*, which
  is the half of the promise you can actually check, and a promise you can only
  check while you are watching is not much of one.

  Now every one of those errands is written down as it happens, with what it was
  and where it reached, and you can ask the record two questions afterwards:
  what left this machine, and which of it was the machine's own doing. The
  first counts everything that left, because everything that left, left.

  **There is no name on an errand, and that is deliberate.** It would have been
  easy to write *alo OS* into the column that says whose authority something was
  under, and it would have been a lie: nobody granted this machine permission to
  sign you in. So that column is simply empty on those entries — no invented
  identity, nothing for a security tool to file next to your agents, and no
  spelling of any name that answers for them when you ask what one of your
  agents did today.

- **The indicator now shows what alo OS itself does on the network, not only
  what its agents do — and there are three things it does, none of them about
  you.** Until now the light that says *something is leaving this machine*
  answered for agents alone. That is the promise law 1 makes, and it left the
  quieter question unanswered: what does the operating system do when nobody has
  asked it anything? On most systems the answer is telemetry, and you find out
  by reading a settings page.

  Here the answer is a list of three, and it is a closed one: signing you in,
  fetching a model so this machine can answer questions on its own hardware, and
  checking whether there is a newer release. Each of them appears on the same
  indicator, in the same place, in your own language — *alo OS is fetching a
  model from …* — so "nothing has left this machine" stays one thing to look at
  rather than two. Beside the list is the promise itself, and it is a sentence
  you read rather than one we published: **alo OS reaches the network for these
  reasons and no others, and never to say anything about how you use this
  machine.**

  There is no measurement, no diagnostics, no crash reporting and no anonymised
  anything — not switched off, but absent: there is no fourth reason a future
  version could quietly turn on, because adding one means editing a list that
  three tests and a feature-scope rule stand in front of. And your
  organisation's egress policy is deliberately not asked about these: a machine
  set to answer every question on its own hardware has to be able to download
  the model it answers with.

- **A line that is half in a language you do not read now says so, wherever the
  half is.** alo OS builds a lot of its sentences out of smaller ones: *the
  grant over your Invoices folder and everything in it has expired*, *@mail is
  asking a question of alo, in the EU*, *Super+Bild ↑ is already doing
  something else*. Until now only the outer sentence counted, so a line whose
  outer half somebody had translated passed as finished while the clause in the
  middle of it was still English — invisible to the person reading it, and to
  the count of what is left to translate.

  Every one of those clauses now answers for itself, at any depth. A refusal
  naming a place is only as translated as the place; a refusal with another
  part of the system's refusal inside it is only as translated as that one; a
  shortcut is only as translated as the least translated key in it. What has
  **not** changed is what counts as a language: your own paths, hostnames, a
  window's identifier, a colour you typed and the keys that print `Q` or `7`
  are yours and this machine's, not anybody's to translate, and they never mark
  a line as unfinished.

  Nothing you read changes today, because there are still no translations. What
  changes is that the day the first one arrives, half-done work looks half done.

- **When the model you chose cannot answer, alo OS stops and tells you —
  it never quietly asks somebody else instead.** A local model that fails does
  not become an API call. You are told what went wrong and *where* it went
  wrong — "nothing answered on this machine", "nothing was answered by alo, in
  the EU" — and, in the same breath, that nothing was sent anywhere and nothing
  will be unless you say so.

  **Asking somewhere else is something you approve, once.** If your machine has
  another place set up and your organisation permits it, you are offered it in
  a sentence that says where the question would go and what leaving means
  there: it would not leave this machine, or it would leave this machine and
  stay on your network, or it would leave this machine and the building. Saying
  yes is worth exactly one question — never a setting, and never a session. A
  setting you ticked in March cannot be present at a failure in June, which is
  the whole reason there isn't one.

  **And if there is nowhere to offer, you are told which of the two it is.**
  Nobody has set up a second place, or your organisation's rule closed the ones
  you have — and if it is the rule, you read that rule in its own words rather
  than finding an empty dialogue. All of it in your own language, and a line
  that is only half translated says so rather than passing for the language it
  is half in.

- **You choose which edge of the screen your dock sits on — bottom, left, right
  or top — and it is a dock built for that edge rather than the bottom one
  turned sideways.** A dock down the side of the screen puts each application's
  name *beside* its icon and still reads the ordinary way round, because a name
  turned ninety degrees is a name nobody can read at a glance. A dock along the
  bottom or the top puts the name underneath. The clock and the battery sit at
  the far end of the dock as a column when it runs down the screen and as a row
  when it runs across — and the far end of a row is the end you reach last, so
  for somebody reading Arabic or Hebrew it is the left.

  **When there is no room for the names, they give way to icons — and the name
  is still there.** Turn your text up far enough and a dock that had names on it
  becomes a dock of icons, because otherwise it would take a share of your
  screen it has no claim to. Resting on an icon still gives you its name, and a
  screen reader still reads it out; alo OS says so, in your language, in the same
  sentence that tells you what happened. Nothing has been taken away — it has
  moved.

  **Where that line falls is measured rather than judged.** The dock may take
  one part in six of the side of the screen it sits on, a name needs a line of
  text under an icon or five times the text's own size beside one, and the names
  stay for as long as both fit. Those numbers are set by a requirement rather
  than by an eye: text has to reach 200% without losing anything (EN 301 549),
  so on the smallest screen alo OS lays out for, on all four edges, the names
  are still there at 200%. A bigger screen keeps them longer. And the icons
  themselves never shrink below the smallest thing the same standard lets you be
  asked to press.

  Your settings file holds only the edge you picked. A machine nobody has
  touched writes nothing at all, which is what lets a later release move the
  default for everybody who never changed it and nobody who did.

- **What is on your screen when you ask an agent something now reaches it at
  that moment, for that question, and never afterwards.** The window in front of
  you, the text you had selected and the document you had open go with the
  question you asked and nothing else does — there is no verb an agent can use
  to look at your screen, and there is no way for alo OS to build one of these
  without you having pressed the key. When the question is answered, what it was
  given is gone.

  **Only the document you had open lets an agent do anything.** Having a file
  open when you ask is you saying *this one*, so an agent may act on that file —
  that file, not the folder it is in and not the invoices beside it. The window
  you happened to be looking at is different: an agent is *told* it is there and
  can still do nothing to it until you grant it, so asking a question with
  Blender in front of you has never given anything permission to close Blender.
  And text you had selected is text, even when it reads like a filename.

  **You can see what you offered, and take it back.** Each part is shown as its
  own line — and *nothing from your screen was offered* is a line too, rather
  than a blank space you have to interpret. The permission your open document
  creates appears in the same list as a folder you picked yourself, expires when
  the question is finished, and can be revoked in one action like any other.

  **What you were looking at is never written down.** The record keeps what an
  agent *did* and which permission it did it under. It does not keep what was on
  your screen, because a system that wrote that down at every question would
  have built, line by line, exactly the log of your day that this rule exists to
  prevent.

  If you select more than 200,000 characters, only the first part goes with the
  question and alo OS tells you how much was left out — in your own language,
  counted the way your language counts.

- **An agent can now be allowed to move a window to the left half, the right
  half or the whole of the screen — and what you approve reads as a sentence in
  your own language.** *Put org.blender.Blender on the left half of the screen*
  is the whole of what you agree to, and the agent gets exactly that: it is a
  grant over one application, and where a window goes is not something anybody
  grants at all. Two windows put on opposite halves is how you tile a pair of
  them. Quarters are a later release and are not offered, so an agent cannot ask
  for one.

  **The reason it took until now is worth the sentence.** Everywhere an agent
  picks from a list of options — this is the first, and it will not be the last
  — the option had two jobs it could not do at once. Software has to send
  something fixed and unambiguous (`left_half`); you have to read something in
  your own language. Until this release those were one string, so a machine
  reading German would have shown you a German sentence with `left_half` sitting
  in the middle of it. Now they are two: the fixed name is what is sent and what
  the record keeps, and the phrase is what you read and what a translator
  translates.

  **And a half-translated sentence can no longer pass for a finished one.** If
  the sentence has been translated into your language but the arrangement inside
  it has not, alo OS knows the line is not really in your language — so it can
  be marked while the system is being built and counted in what is still owed,
  rather than reaching you as one English phrase in the middle of a sentence
  with nothing anywhere saying so.

- **An agent can now be allowed to open, focus and close an application — and
  closing one asks it rather than taking it away.** These are the first
  capabilities alo OS has that are about applications rather than files, and
  each of them waits for you to approve one sentence: *open
  org.blender.Blender*, *bring it to the front*, *ask it to close*. A grant is
  over one application and covers nothing else, so allowing the agent to close
  your editor is not allowing it to close your browser.

  **"Ask it to close" is exactly what happens.** The application is asked, the
  same as pressing its close button: if it has unsaved work it puts its own
  question up and you answer that. Nothing here takes a window away, because
  approving *close this* is not approving *and throw away the afternoon you
  have not saved* — and unsaved work is the one thing on this list that cannot
  be undone afterwards.

  **Bringing a window to the front waits for approval too.** That may read as
  fussy for something so small, and it is not: a window that can put itself in
  front of the one you are typing into gets your next keystrokes.

  Two things it deliberately cannot do. **There is no way for an agent to ask
  what you have installed or what you have open** — it is told what is in front
  of you at the moment you invoke it, and nothing else, because the list of
  applications on a machine says a great deal about the person using it. And a
  refusal never leaks that either: an application you have not granted is
  refused in the same words whether or not it is installed.

  What you approve names the application the way the system knows it
  (`org.blender.Blender`) with its ordinary name shown beside it, rather than
  the other way round. Two applications can call themselves *Mail*; no two
  share an identifier, and the thing you are agreeing to should not be a name
  chosen by the thing you are agreeing to.

  The fourth verb `docs/features.md` promises — arranging a window — arrived in
  the entry above.

- **The record of what the agent did now survives the machine being turned
  off, and a record that has been shortened says so.** Until now what an agent
  did was kept in memory, which answers *what did it do this afternoon* and
  nothing about last month. It is written to a file as it happens — one line
  per thing that happened, on the disk before the write finishes, so an entry
  is not lost to a machine that loses power a moment later.

  **How long it is kept is one setting, and a machine ships keeping
  everything.** You can say *keep the last 30 days*; you cannot say *keep
  nothing*, and there is no way at all to say *remove that particular
  afternoon*. What ages out is decided by the rule and by the date, so what you
  set is what happens.

  **And a record that has had anything removed from it says so, permanently.**
  This is the point of the whole thing: a machine whose evidence has aged out
  and a machine that did nothing would otherwise be the same short file, and
  somebody asking *what did it do in March* would be told nothing and believe
  it. The record says where it now begins, in the language you read, and no
  later shortening can remove that sentence.

  Two more things it will not do quietly. A line it cannot read is **reported**
  rather than skipped, and a record with one in it is **not shortened at all**
  until somebody has looked at it — tidying the file would tidy away the
  evidence that something was wrong. And a record that is missing is refused
  rather than answered as an empty one, because a deleted record must not read
  as an innocent machine.

  For anybody whose tooling will read these files: the format is written down
  in `docs/contracts/record-file.md` and is a public surface, so it changes
  additively. A record written by a newer alo OS is refused rather than
  appended to.

- **The sentence you approve, the record of it and the refusal that quotes it
  are now one sentence rather than three renderings of it.** *Move
  march.pdf into Archive* is what an agent puts in front of you before it
  touches anything, and until now it was written down in the language the
  capability was declared in — so a machine reading Estonian could show you an
  Estonian sentence, ask you to approve it, and keep an English one in the
  record a security review reads afterwards. Now there is one thing: what names
  the sentence, and the values that go in it. The words are asked for wherever
  somebody reads them, and the screen, the approval and the record all read the
  same thing.

  Two clauses that used to arrive in English inside an otherwise translated
  sentence have gone with it: *what this argument is for*, when an agent asks
  for something without it, and the question quoted back to you when it stood
  too long to answer. Both are looked up with the same vocabulary as the
  sentence around them.

  Nothing about **what may run** changed. A capability is still refused without
  a vocabulary having been loaded, a verb still cannot be declared with a
  sentence that leaves one of its arguments out, and one approval is still worth
  exactly one execution.

  For anybody writing an adapter: a verb is now declared from the same declared
  strings a translator is given, rather than from English that is separately
  translated somewhere else. It is a change to a public surface, and it is what
  makes the guarantee above structural instead of something a test has to hope
  for.

- **The line that says something is leaving your machine is now in your own
  language.** It is the one alo OS is sold on: *@mail is asking a question of
  someone, which has not said where it runs*, read while it is happening rather
  than in a log afterwards. Read in a language you do not speak it is a light
  that blinks, so it moved — whole sentences, one for each of the three things
  an agent can cause, rather than a stem with a place glued onto the end, because
  where the name of a place goes in a sentence is not something a program can
  work out.

  The place inside the line moved with it, and says the same uncomfortable thing
  as the sentence you read beside an answer: a provider that has not said where
  it runs says so in both. A host an agent named — `alo.example` — is shown
  exactly as it was written, because it is somebody's address and not a phrase.
  So are every refusal your organisation's egress policy makes, and every reason
  an address could not be shown at all.

  Nothing about **what may leave** changed, and that is the part worth saying
  plainly: the policy is asked before a connection opens, it is asked without
  words, and a machine that failed to load a translation still refuses exactly
  what it refused before. What was refused is now kept as what it was rather
  than as an English sentence, so the words on your screen and the words in the
  record are one rendering of one refusal.

- **Where your question is about to go is now said in your own language** —
  and so is every reason it will not go there. That is the sentence you read
  beside an answer before you decide whether to paste a contract into the next
  one: *on this machine*, *on the studio workstation, on your network*, *by
  Mistral, in the EU*, or the uncomfortable one, *by someone, which has not
  said where it runs*. Reading the last of those in a language you do not speak
  is the same as not reading it, which is why it moved: it is the only thing on
  the screen telling you the question is about to leave the building.

  When your organisation has set a rule — questions stay in the building, or in
  a named region, or on this machine and nowhere else — the refusal now names
  the rule and the place it stopped, in one language, whichever language you
  read. It is kept as what it was rather than as a sentence, so the words on
  your screen and the words in the record are one rendering. And a machine set
  to keep questions in the building still never reaches out to a provider to
  find out whether its key works: nothing about what is permitted depends on
  which language is loaded, and there is a test that says so.

  Everything else this part of the system says moved with it: the provider you
  were adding and could not, the key you pasted the line around, the model that
  is not installed, the download that stopped. Two of those are worth naming.
  A key never reaches a sentence in any language — neither of the two strings
  about one has a gap for anything to be put into, and a translation that
  invented one is refused. And *there is not enough room for that download* now
  says exactly that: the two numbers sit beside it, for whoever writes a size
  the way your region writes one, rather than inside a sentence that would have
  to count in English.

- **Every way alo OS can tell an agent no can now be read in your own
  language** — which is the half of the system you meet when something does
  *not* happen. A folder you tried to grant and could not, an agent that sent a
  path that could lead somewhere else, a change that was never put to you
  because nothing covered it, a question you answered too late, and the two
  sentences the grants themselves say: *you have not granted this*, and *you
  did grant it and it has run out*. Those two say different things because they
  need different things from you, and both now name the folder and the agent in
  the middle of a sentence that is yours.

  The refusal and what it names are one language. A German machine reads
  *@files wurde /home/anna/Archive nicht gewährt*, with the path and the agent
  left exactly as your machine spells them, rather than a German sentence with
  an English clause inside it. A message that counts something is counted the
  reader's way: *longer than one character* is a different sentence from
  *longer than 255 characters*, and a language with more forms than English has
  is not held to English's two.

  **What is written down is what you were told.** A refusal is now kept as what
  it was until somebody asks it for words, so the sentence on your screen and
  the sentence in the record are one rendering rather than two accounts of one
  moment that nothing keeps equal. Nothing about what is refused changed: a
  machine with no translations loaded refuses exactly what it refused before,
  shows the same English, and says that is what it did.

- The colours you make your machine out of can now be named in your own
  language, and so can everything alo OS says when a value you chose cannot be
  used — a colour that is not written the way a colour is written, a screen whose
  name has a space on the end of it, a background set to change faster than the
  eye can follow, a schedule that turns dark and light at the same minute, a text
  size past either end of what the screen can show. The colour names are the part
  worth saying twice: they are single words, they are what you pick from rather
  than read once, and several of them have no ordinary word in most languages —
  so every one carries a note describing the colour rather than assuming the
  English word travels. German reads *Grünspan* where English borrowed
  *verdigris*, and *Anthrazit* where English named a grey after burnt wood, and
  neither list could have been reached from the other word by word. A refusal and
  the colour inside it are now in one language, so a German machine does not put
  an English colour in the middle of a German sentence. Nothing you have set
  changes: the settings file is written exactly as it was, and a machine with no
  translations loaded refuses exactly what it refused before and shows the same
  English — while saying that is what it did.

- The keyboard shortcuts can now be read in your own language — every row in the
  list, every key named in it, and everything alo OS says when a combination
  cannot be a shortcut. That includes the keys themselves, which is the part
  that would have been wrong on a German machine: the panel now says *Entf*,
  *Pos1*, *Strg* and *Bild ↑* where the keyboard in front of you does, rather
  than Delete, Home, Ctrl and Page Up. The keys that print the same mark
  everywhere — Q, 7, the comma, F1 — are shown as that mark and are deliberately
  not offered for translation, because `Super+Q` means the key marked Q on your
  own keyboard and always did. Nothing you have set changes: the settings file
  is written exactly as it was, a machine with no translations loaded shows the
  same English it showed before, and it says so rather than passing it off as
  your language.

- Everything the file half of alo OS says to you can now be said in your own
  language — and none of it can reach you in English without the system knowing
  it did. That is every refusal a file verb can give you (*there is already
  something at that name*, *this file is too big to read*, *that path really
  leads somewhere you did not grant*), what each of the six file verbs is for,
  what each thing it asks you for means, and **the sentence you approve before a
  file is renamed, moved or archived**. The sentence is the one that matters: a
  translation of it that lost one of the things it names — *move march.pdf*,
  with no word about where to — is refused when the language is loaded, the same
  way a verb whose English sentence left an argument out is refused when it is
  declared. A translator gets a note wherever a sentence cannot be translated
  from its own words, and the message about a file being too big now counts
  bytes properly in every language rather than saying *1 bytes*. Nothing about
  what is allowed changed: a machine with no translations loaded refuses exactly
  what it refused before, and what a refusal told you is what the record of it
  keeps.

- A sentence that counts something is now counted in your own language, and the
  rules for that were read rather than remembered. *1 byte* and *2 bytes* is one
  sentence in English with two shapes; it is three in Polish, five in Irish, and
  Latvian has a word for none of them — so a message like *this file holds 4 000
  000 bytes* is written once, translated once per shape, and shown in the shape
  **your** language uses for **that** number. The shapes come from the Unicode
  common locale data, quoted line by line beside the code that implements it,
  because a plural table written from memory is wrong in a language nobody here
  reads and nothing would ever say so. Three things follow that a translator
  meets rather than a programmer: a file that offers a form your language never
  uses for a whole number is refused, and the refusal names the forms it does
  use; a sentence may spell the number out — *one file* — only where that form
  really is one number, which Croatian's *one* (1, 21, 31, 101) and French's
  (none as well as one) are not; and what a translator is handed is the forms
  **their** language needs, so a half-done one is not mistaken for a finished
  one. A language whose counting rules nobody has added yet can still translate
  every string that does not count, and is told plainly what is missing rather
  than being quietly given English's two shapes.

- A provider you add can be tested before you save it, so a key with a
  character missing is found while you are still looking at the field you typed
  it into — not days later, in the middle of a question, as an answer that
  failed for no stated reason. The answer says which thing went wrong, because
  they send people to different places: *that key was not accepted* is not *this
  provider wants a key and was given none*, and neither is *that address
  answered, but not like a provider this system can use*, which is what you get
  when the address is of the website rather than of the API. A provider that
  works comes back with the models it offers, ready to be saved with it.
  **Testing sends nothing of yours** — no question, no document, no sample
  prompt — and it asks this machine's policy first: on a machine set to keep
  questions in the building, testing a provider outside it does not quietly
  happen anyway, it is refused in the policy's own words before anything is
  sent. The address you typed is the address that is reached: an address that
  redirects somewhere else is refused rather than followed, because a key does
  not travel to a host nobody agreed to. And the key itself is held for the one
  request and nothing else: it is never written down, never rendered in a log or
  an error, and cannot be read back out of the code that sends it.

- Your accent colour is yours, and terracotta is not one of the choices. It
  means alo is present or acting, and an accent somebody could set to terracotta
  would take away the one signal that says the machine is doing something on
  their behalf. Five designed hues are offered instead — verdigris, indigo,
  violet, moss and rose — each with a value for a light background and one for a
  dark, because a colour that reads well on cream is illegible on charcoal. And
  wherever alo appears, its colour arrives with a mark and a word: a signal
  carried by hue alone is no signal at all for the one man in twelve who cannot
  distinguish it.

  **Choosing one is now a thing the system does**, and the accent follows the
  machine: pick rose in the morning and it is still rose at eight in the
  evening, in the value drawn for a dark screen rather than the one that stops
  reading against it. Your choice is stored as the colour's name and not as a
  number, so a release that corrects a value corrects it for everybody who chose
  that colour, and putting the setting back gives you what the machine ships
  with. **Every one of the ten values has been measured** against the grounds it
  is drawn on and clears what EN 301 549 requires of ordinary text — a hue that
  did not would fail the build rather than reach somebody who cannot read it.
  Asking for terracotta by any road — by name in a settings file, or by its hex
  — is refused in a sentence that says why and what to choose instead.

- alo OS can now be translated, and English has stopped being able to hide.
  Every sentence the system says is named, and the answer to *what does this
  say* always carries **whether anybody translated it** — so a screen that is
  still English in a Latvian machine is something the system knows about, can
  list, and marks outright while it is being built, instead of something a
  person in Latvia finds after the release. A translator's file is checked
  against what the system actually says before a word of it is shown: a
  sentence that lost the file name or the size out of it is refused, in words
  addressed to the translator rather than to a programmer, and everything wrong
  with a file comes back at once rather than one mistake at a time. A part-done
  language is welcome — a few hundred strings at a time is how translation
  really happens, and the untranslated rest simply stays English until it is
  not. **You name the languages you read, in your order**, so somebody who
  reads Latvian and Russian meets Russian before English; nothing infers a
  second language on your behalf. All 24 official EU languages are listed, each
  written in its own language rather than in ours, because a list that says
  *Greek* is a list the people it is for cannot read. There are no translations
  yet: what exists is the machinery, and the reason it exists now is that every
  screen written before it would have had to be written twice.

- The machine is yours to look at. Put a picture behind your windows, or a
  folder of them that changes every so often, or a plain colour — and give one
  screen something different if you want, without the other screens changing.
  **A screen you have never set anything on shows what you chose**, so plugging
  in a projector in front of a room puts your picture there rather than ours.
  Light and dark can follow the clock — dark after six, light again at seven —
  and a machine you have not asked will not change its own appearance on the
  first evening. Text goes up to 300%, which is well past the 200% the
  accessibility standard for European public-sector procurement requires. **And
  the lock screen does not show a rotating folder of your photographs to
  whoever walks past**: while your desktop rotates, following it means the
  wallpaper alo OS shipped, unless you say outright that you want your pictures
  there. As with the shortcuts, only what you changed is written down, so a
  later release can ship a better wallpaper or a better schedule without
  reaching over your choices.

- The keyboard shortcuts are yours to change, and changing one never quietly
  takes another away. Ask for a combination something else already uses and you
  are told what has it — `Super+Left is already Put the window on the left half`
  — rather than finding out days later that snapping stopped working. Only what
  you changed is stored, so a later release can improve a default without
  reaching over your choices, and if one of ours ever lands on a combination you
  had already moved something onto, **yours keeps working** and the collision is
  shown in Settings instead of being decided behind your back. Copy, cut and
  paste cannot be taken by a system shortcut at all: they belong to whatever
  you typed them into. A shortcut can also simply be cleared, and it stays
  cleared. What the machine starts with is what these machines have always had —
  `Alt+F4` closes, `Super+Up` maximises, `Super+Left` and `Super+Right` take the
  halves, `Alt+Tab` moves between windows — with `Super+A` for the agent and
  `Super+Space` for the launcher.

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
