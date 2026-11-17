# Two verbs taking the same arguments, and gpt-4o-mini sends neither

- **Version:** `gpt-4o-mini` at `https://api.openai.com`, held to the strict
  schema `alo_turn::the_shape_a_provider_is_held_to` builds, through the
  every-try check (`.github/workflows/every-try.yml`), on 2026-11-17.
- **Whose:** the provider's model, or how it is held to a schema. Which of the
  two is **not known**.
- **Behaviour:** asked *In Dolphin, open the right-click menu of notes.txt.*,
  with `accessible.open_menu` taking `application` and `name`, the model sent
  `accessible.type_text` in **20 of 20** tries, over two runs and both levels:
  the file's name as `name`, and as `text` an empty string, or `open_menu`, or
  `open menu` (dispatched runs 44044024246 and the pull-request run before it).
  It named the verb it meant inside the text and did not send that verb.
  `accessible.scroll_to`, offered beside it, took exactly the same two arguments.
  The same request had taken the road 10 of 10 the day before, when `scroll_to`
  was not offered.
- **Our response:** `accessible.open_menu` takes `thing` instead of `name`. With
  nothing else changed, the next run took the road **19 of 20** (run
  44046603843; the one other try opened the menu and then opened it again).
  That renaming the argument is what made the difference is **measured once**;
  that two calls of the same shape are what the model cannot tell apart **fits
  and is not checked** — `type_text` and `type_without_saving` share their three
  arguments and both took their roads 10 of 10 in the same week. When a new verb
  would take exactly the arguments another change takes, the every-try request
  for it is the check that says whether it can be chosen.
- **Date:** 2026-11-17.
