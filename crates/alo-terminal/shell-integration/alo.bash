# alo OS shell integration, for bash.
#
# What this does, in plain words: it tells alo where your prompt starts and
# ends, when a command starts and finishes, and — only when you press
# Ctrl+X Ctrl+A — what you have typed so far.
#
# What it reveals: every command you run, to alo, on this machine. Nothing
# leaves the machine because of this file.
#
# The format is published in docs/contracts/shell-integration-marks.md. The
# first four marks are OSC 133, which kitty, iTerm2, WezTerm and VS Code already
# emit — if you already have shell integration for one of those, alo reads it
# and you need nothing here except the Ctrl+X Ctrl+A binding at the bottom.
#
# alo does not add this to your shell startup. You do.

# ---------------------------------------------------------------------------
# The cycle: where the prompt is, and when a command ran.
# ---------------------------------------------------------------------------

# A — a prompt is about to be drawn; B — the prompt has ended, and what follows
# is what you type. Both are wrapped in \[ \] so bash does not count them when
# working out how wide the prompt is; a prompt measured wrong wraps in the wrong
# place and that is a worse bug than no integration.
if [[ "$PS1" != *'133;A'* ]]; then
	PS1='\[\e]133;A\a\]'"$PS1"'\[\e]133;B\a\]'
fi

# C — the command is running, and what follows is its output. PS0 is printed
# after you press Enter and before the command runs, which is exactly this mark.
if [[ "$PS0" != *'133;C'* ]]; then
	PS0='\[\e]133;C\a\]'"$PS0"
fi

# D — the command has finished, with its status.
#
# The first line must be the status read, before anything else can overwrite it,
# and the function returns it unchanged so that $? is still yours afterwards.
__alo_finished() {
	local status=$?
	printf '\e]133;D;%s\a' "$status"
	return "$status"
}

if [[ "$PROMPT_COMMAND" != *__alo_finished* ]]; then
	PROMPT_COMMAND="__alo_finished${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi

# ---------------------------------------------------------------------------
# The line you have typed and not run, sent only when you ask for it.
# ---------------------------------------------------------------------------

# No shell hook runs while you are typing, and that is deliberate: alo is not
# watching the line editor. This fires when you press the key and at no other
# time.
#
# The length is in BYTES of the command, not characters. `${#line}` counts
# characters, and for anything outside ASCII the two differ — a command with an
# accent in it would declare a length alo then refuses. `wc -c` counts bytes.
__alo_typed() {
	local line="$READLINE_LINE"
	local encoded
	local bytes
	encoded=$(printf '%s' "$line" | base64 | tr -d '\n')
	bytes=$(printf '%s' "$line" | wc -c | tr -d ' ')
	printf '\e]133;alo-typed;%s;%s\a' "$encoded" "$bytes"
}

# Ctrl+X Ctrl+A. The Ctrl+X prefix is where readline already keeps line-editor
# commands — Ctrl+X Ctrl+E opens the current line in an editor — and a prefix
# sequence is not a chord any compositor can take before the shell sees it.
bind -x '"\C-x\C-a": __alo_typed' 2>/dev/null
