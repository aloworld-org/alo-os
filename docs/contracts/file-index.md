# Contract — the file index

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is the file `alo-finding` keeps for a folder a person asked to have
searchable: everything under it by name, kind, date and the words in it, so
that *search your own files, without asking anything* (`docs/features.md`,
v0.5) is answered from this file and never by walking the folder again. The
file manager reads it, the agent's *"where is that file?"* reads it, and a
person who wants to see what their machine knows about their documents can
open it in anything that reads JSON.

`crates/alo-finding` is the shape as working code: `alo_finding::Index::of`
makes one at a moment the caller names, `Index::kept_at` writes it,
`Index::read_from` reads it back, `Index::again` refreshes it reading only
what changed, and `Indexed::again` does the last three for a folder by its
name, in one call.

## Where it is

One file per indexed folder, under the person's own data directory:

```
$XDG_DATA_HOME/alo/finding/<name>.index
```

and, where `$XDG_DATA_HOME` says nothing or is not an absolute path:

```
$HOME/.local/share/alo/finding/<name>.index
```

Per person, because a machine may have several people on it and one person's
documents are not another's. Under the data directory rather than the config
directory because an index is not a choice, and rather than the cache
directory because a cache is something a machine may throw away unasked and
an index of ten thousand documents is an afternoon's reading. On a Unix host
the file is created readable by its owner alone (`0600`, in a `0700` folder):
it holds every word of every text file under the folder.

`<name>` is the 64-bit FNV-1a hash of the folder's path, in sixteen lowercase
hexadecimal digits. A path spelled into a file name would run past what a
filesystem allows long before a person's folders do; the folder itself is in
the file's first line, so a reader that lists the directory tells the files
apart by reading each head, and a file whose head names a different folder
from the one asked about is refused rather than trusted.

A session with no usable `$HOME` has nowhere for an index to be, and
`alo-finding` says so rather than inventing one.

## The list of indexed folders

Beside the indexes, in the same directory, one file says which folders a
person asked to have indexed:

```
$XDG_DATA_HOME/alo/finding/folders.list
```

(and under `$HOME/.local/share` by the same rule). Its name is one no hash
can produce, so a person listing the directory sees which file is the list.
`alo_finding::Indexed::read_from` reads it, `Indexed::index_of` hands back
the index of a folder on it — read from that folder's index file, never by
walking the folder — or says the folder was never indexed, `Indexed::keep`
writes an index and puts its folder on the list, `Indexed::forget` takes a
folder off the list and removes its index file with it, and `Indexed::again`
brings a folder's index up to date by its name at a moment the caller passes
in: the kept index read, the folder indexed again reading only a file whose
size or time changed, and the result written whole in the index's place. A
folder not on the list is refused and is not indexed for the first time by
it; a folder that is gone since it was indexed is refused and its index is
kept, because an unplugged disk is not a request to forget it.

**Nothing watches.** When an index is brought up to date is the caller's
decision and nobody else's: there is no `inotify`, no thread and no timer in
`alo-finding`, and a test reads its shipped source to say so. A crate that
woke up on its own to read the disk would be a background reader, whatever
it did with what it read.

The shape is the index file's: the first line says what the file is, and
every line after it is one folder, in the order they were asked for. Every
line is compact JSON with no newline inside it.

```
{"format":1}
{"folder":"/home/ada/Documents"}
{"folder":"/home/ada/Pictures"}
```

| Line | Field | Meaning |
|---|---|---|
| first | `format` | Which shape the file is in. Required. `1` today. |
| each after | `folder` | A folder a person asked to index, as an absolute path. Its index is the file named by that path's hash, above. |

A list that is not there yet is an empty list: nothing has been asked for. A
file that is there and does not parse whole — any line — is refused whole,
with the reason, rather than read as an empty list: an empty list would say
every folder was never indexed while its index sat on the disk. The file is
**replaced whole**, never appended to, the way an index is. A later version
may add a field to either line; a reader of this version ignores a field it
does not know, and refuses a `format` it does not read.

**The list is the authority.** A file in the directory the list does not
name is nobody's index: the list is what says a folder is indexed, and
nothing lists the directory to find out. When a folder is kept, its index is
written first and the list second, so the list never names a folder whose
index was not written. When a folder is forgotten, its index file is removed
first and the list second, so the words of a folder a person asked to have
forgotten are off the disk before anything else.

**The list is not a grant.** A folder being on it says nothing about whether
an agent may search it: that is a grant, made by the person and checked by
`alo-capability` at the door, and the list holds folders and nothing the
record does — no agent, no grant, no approval, nothing about who asked — so
it answers the same whether an agent or a person asked.

## The shape

The shape is the record's (`docs/contracts/record-file.md`), for the record's
reasons. One index is one file. The first line says what the file is; every
line after it is one entry, in the order the walk found them: each folder
before the things inside it, and names in order within a folder. Every line is
JSON, compact, with no newline inside it.

```
{"format":1,"of":"/home/ada/Documents","made":{"secs":1760000100,"nanos":0},"covered":{"whole":true,"most":20000,"unread":[],"elsewhere":[],"not_entered":[],"unnamed":0}}
{"below":"2026","kind":"folder","bytes":0,"modified":{"secs":1760000000,"nanos":0},"contents":{"were":"not-a-file"}}
{"below":"notes.txt","kind":"text","bytes":47,"modified":{"secs":1760000060,"nanos":0},"contents":{"were":"read","words":["anna","before","contract","dear","from","summer","the"]}}
{"below":"2026/march.pdf","kind":"pdf","bytes":18201,"modified":{"secs":1759000000,"nanos":0},"contents":{"were":"not-text"}}
```

The file is **replaced whole**, never appended to: a sibling is written,
synced and renamed over it, so a machine that loses power in the middle keeps
the index it had. A file that does not parse whole — any line — is refused
whole rather than searched in part, because an index missing its last
thousand entries would answer *nothing matched* about a thousand files; the
index is simply made again.

## The first line

| Field | Meaning |
|---|---|
| `format` | Which shape the file is in. Required. `1` today. |
| `of` | The folder this is the index of, as an absolute path. Required. |
| `made` | The moment the index was made, as `{"secs":…,"nanos":…}` since the Unix epoch — the same shape as an entry's `modified`. Optional: added after the first indexes were written, so a head without it is an index made before the moment was kept, and is read with no moment rather than an invented one. |
| `covered` | What the walk under the folder could not reach. Required. |

**`format` is what tells the first line from an entry**, which has no such
field. A file whose first line is not a head is not an index.

**`made` is the caller's moment, not the machine's.** Whoever makes the
index — the file manager, a daemon, the person — passes in the moment, and
`alo-finding` reads no clock of its own; what the field says is when that
caller said the folder was read, so that an answer from the index can say
*as of then* beside its results. It is written when the index is made or
made again, and an index made again at a moment the caller names earlier
than the last one says the earlier moment, because nothing in the crate
looks at a clock to disagree.

`covered` says what a search over this index did not look at, so that an
empty answer is *nothing matched* and never *nothing was looked at*:

| Field | Meaning |
|---|---|
| `whole` | Whether the walk finished, rather than stopping at its bound. |
| `most` | The bound: how many things one walk looks at. `20000` today, which is `alo_files::MOST_WALKED`. |
| `unread` | Folders the machine would not let the walk read, each as `{"below":…,"why":…}` with what the machine said. |
| `elsewhere` | Folders on another filesystem, not entered. |
| `not_entered` | Folders the walk had not finished when it stopped at its bound. Empty unless `whole` is `false`. |
| `unnamed` | How many things were left out because their names cannot be shown safely. |

## What an entry says

| Field | Meaning |
|---|---|
| `below` | Where it is under the folder, with `/` between the parts on every host. |
| `kind` | What it is, from its own bytes — never from its name. One of the kinds below. |
| `bytes` | Its size. For a link, the size of the link itself. |
| `modified` | When it was last written, as the filesystem says: `{"secs":…,"nanos":…}` since the Unix epoch. |
| `contents` | Its words, or why the index has none. Tagged with `were`, below. |

### Kinds

Decided from the first eight kibibytes of the file: a known signature first,
then whether what is there reads as UTF-8 text with no control characters
beyond tab, newline, carriage return, form feed and escape, and *bytes*
otherwise.

| `kind` | What it is |
|---|---|
| `text` | Text of any language. The only kind whose words are read. |
| `pdf` | A PDF document (`%PDF-`). |
| `png`, `jpeg`, `gif` | An image in that format. |
| `zip` | A zip archive (`PK\x03\x04`), which is also what most office documents are. |
| `program` | An ELF program. |
| `bytes` | A file of no kind this version knows. Not a fault. |
| `empty` | A file with nothing in it. |
| `unread` | A file the machine would not let the index open, so its bytes were never seen. |
| `folder` | A folder. |
| `link` | A link. An entry of its own; never followed. |
| `other` | A device, a socket, a pipe. |

A later version may add a kind. A reader that meets one it does not know
should treat it as `bytes`.

### Contents

| `were` | Other fields | Meaning |
|---|---|---|
| `read` | `words` | The file is text and these are its words: each once, in lower case, sorted. A word is a run of letters or digits in any script. |
| `not-text` | | A kind with no reader: no words here. A search by words does not find it, and says so. |
| `not-read` | `why` | The file could not be read; `why` is what the machine said. |
| `too-big` | `bytes` | Larger than an index reads — `alo_files::MOST_READ`, a megabyte, the same bound a file verb reads under — so its words were not read. |
| `not-a-file` | | A folder, a link or a device. |

**The words are the contents, and the text is not.** An index holds the set
of words in a file and never the file: an index that held every document
whole would be a second copy of every document, under a different name, in a
place nobody thought to grant.

## What is **not** in it

- **Nothing about who asked.** No agent, no grant, no approval, no moment
  anybody asked anything: a person searching their own files is not an agent
  and is not asking anybody. That is the record's (`record-file.md`), and the
  index never holds anything the record does.
- **Nothing a model said.** No summary, no embedding, no guess at what a file
  is about. *Contents* means the words in the file, and no model is asked.
- **Nothing from outside the folder.** A link is an entry that is a link; what
  it points at is not walked, not read and not named.

## What changes additively

A later version may add a field to the head or to an entry, a kind, or a
value of `were`; a reader of this version ignores a field it does not know
and treats a kind it does not know as `bytes`. A changed meaning of an
existing field is a new `format`, and this version refuses a `format` it
does not read rather than guessing.

`made` in the head is the first field added this way: `format` stayed `1`,
a head written before it still reads, and a head written without it — an
index of an earlier version, made again — gains it the next time the index
is made.
