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
makes one, `Index::kept_at` writes it, `Index::read_from` reads it back, and
`Index::again` refreshes it reading only what changed.

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

## The shape

The shape is the record's (`docs/contracts/record-file.md`), for the record's
reasons. One index is one file. The first line says what the file is; every
line after it is one entry, in the order the walk found them: each folder
before the things inside it, and names in order within a folder. Every line is
JSON, compact, with no newline inside it.

```
{"format":1,"of":"/home/ada/Documents","covered":{"whole":true,"most":20000,"unread":[],"elsewhere":[],"not_entered":[],"unnamed":0}}
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
| `covered` | What the walk under the folder could not reach. Required. |

**`format` is what tells the first line from an entry**, which has no such
field. A file whose first line is not a head is not an index.

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
