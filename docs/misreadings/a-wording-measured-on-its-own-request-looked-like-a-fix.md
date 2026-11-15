# A wording measured on its own request looked like a fix

## What was concluded

*"Reading the window first, every time, even right after opening, fixes the
VSCodium request"*, and *"looking in the catalogue first fixes installing"*.
Each was measured on the request it was written for, and each moved that
request: VSCodium 0 → 2 and 3 of 5, installing 0 → 3 of 5 (every-try, set 4,
2026-11-15, gpt-4o-mini).

## What was true

Run over the **whole set**, each one broke a request it was never aimed at:

- Opening Firefox went from 5 of 5 to 0 or 1 of 5. Told to read a window
  first, the model read Firefox's window after it had only been asked to open
  it.
- Asking what could be installed went from 5 of 5 to 0, 1 or 2 of 5. Told how
  to install, the model installed something it had only been asked to list.

A third change was made on a guessed cause, and it stopped nothing: *"the
model cannot answer because it is forbidden to say something was done"*. Both
of its wordings measured 0 of 5 on opening Firefox. All three were closed with
their tables (#1277, #1279, #1280).

## The mechanism

Every request is shown **every** paragraph. A sentence written for one request
is read in all fifteen, and a small model follows a vivid instruction wherever
it can. A run limited to the target request (`ALO_EVERY_TRY_ONLY`) can show
only the gain. The loss lands somewhere the run did not look.

The guessed cause is the other half: it fitted the evidence, *"it never
answers"*, and was not checked before a wording was built on it.

## The cure

A change to what the model is told is measured on **the whole set**, at least
twice, before its counts are quoted. A run of one request is a quick look,
never the measurement. And a cause is tried as cheaply as possible, by one
sentence measured once, before a pull request is built on it.
