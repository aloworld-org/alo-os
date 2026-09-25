#!/bin/bash
# Per-task count over the v0.5 plans on the latest main.
# done     = Status says done, or a "**Done, DATE.**" paragraph in the block
# declined = Status says "not pursued" (an owner decision, not work left)
# blocked  = Status says blocked (as a word, so "unblocked" is not blocked)
# open     = everything else
PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
export PATH
set -u
cd /root/alo-os-lane-b || exit 1
git fetch origin --quiet && git reset --hard origin/main --quiet
cd docs/autonomy || exit 1
out=/root/gatelogs/count4.out
: > "$out"

for f in v0-5-*.md; do
  awk -v file="$f" '
    function emit() {
      if (!inblock) return
      if (done) d++
      else if (declined) x++
      else if (blocked) { b++; bl = bl "\n      blocked: " title }
      else { o++; ol = ol "\n      open:    " title }
    }
    /^### [0-9]+\./ {
      emit()
      inblock = 1; done = 0; blocked = 0; declined = 0; title = substr($0, 5, 70)
      next
    }
    inblock && /^\*\*Done, / { done = 1 }
    inblock && /^\*\*Status:\*\*/ {
      s = tolower($0)
      # Only the first word of the status counts: "Tasks 8, 9 and 12 are done"
      # inside a blocked line is about other tasks, not this one.
      bare = s; gsub(/\*/, "", bare)
      if (bare ~ /^status: *done/) done = 1
      if (s ~ /not pursued/) declined = 1
      if (s ~ /(^|[^n])blocked/ && s !~ /unblocked/) blocked = 1
      if (s ~ /^\*\*status:\*\* *blocked/) blocked = 1
    }
    END {
      emit()
      printf "%-52s done=%2d open=%d blocked=%d declined=%d%s%s\n", file, d, o, b, x, ol, bl
      printf "TOTALS %d %d %d %d\n", d, o, b, x
    }
  ' "$f" >> "$out"
done

awk '/^TOTALS/ {d+=$2; o+=$3; b+=$4; x+=$5}
     END {printf "\n==== ALL PLANS: tasks=%d  done=%d  open=%d  blocked=%d  declined=%d  -> left to do=%d\n", d+o+b+x, d, o, b, x, o+b}' "$out" >> "$out"
sed -i '/^TOTALS/d' "$out"
echo "main at $(git log --oneline -1)" >> "$out"
echo "probe_should_be_7=$( (exit 7); echo $? )" >> "$out"
cat "$out"
