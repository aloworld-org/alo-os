//! The Ollama adapter — **the only file in this repository that knows Ollama
//! exists**.
//!
//! [ADR 0006](../../../docs/decisions/0006-the-pinned-model-runtime.md) pins
//! Ollama as the runtime alo OS ships and puts [`ModelRuntime`] in front of it.
//! That promise is only worth anything if it is kept literally: every endpoint
//! path, every field name and every naming convention Ollama has lives here, so
//! a reviewer asking "what would replacing the runtime cost?" can read one file
//! and know.
//!
//! Two things this deliberately does **not** do.
//!
//! It does not *offer* a model the catalogue does not list. Ollama's own
//! library is not curated and states no licences; `docs/features.md` promises a
//! curated catalogue with licences, and that promise would be hollow if any
//! name could be downloaded through here. So [`Ollama::fetch`] refuses an id the
//! catalogue does not carry, with [`RuntimeError::NotOffered`] — and
//! [`Ollama::answers`] deliberately does not, because offering is downloading:
//! a model already on somebody's disk is theirs, and refusing to ask it
//! anything would be alo OS overruling the owner of the machine about their own
//! hardware rather than keeping a promise about what we ship.
//!
//! And it exposes no way to send an arbitrary request. The trait forbids it
//! (law 2), and an adapter that quietly added one would put the escape hatch
//! back where the trait had removed it. [`Ollama::answers`] is not that hatch
//! and is worth saying so about: it carries a question to a model, which is
//! text for a model to read, never a command for this machine to run.
//!
//! # What a question does to this file, and what it does not
//!
//! A question is the only thing here that is somebody's own words, and it is
//! borrowed for the length of one request. It goes into one JSON body and
//! nowhere else: not into a `RuntimeError`, which has no field for it, and not
//! into anything this file keeps. ADR 0001 §7 is what that is for, and
//! `a_question_goes_into_the_body_and_nowhere_else` is the test.
//!
//! # Where the runtime is, is also this file's
//!
//! [ADR 0019](../../../docs/decisions/0019-a-runtime-is-found-not-configured.md)
//! settles the one thing ADR 0006's rule was silent about: an *address* is as
//! much a mention of Ollama as a field name is, so nothing outside this file may
//! carry one. [`found_on_this_machine`] is the door the rest of alo OS uses: an
//! operator who could point the agent elsewhere would leave the egress
//! indicator honest about a destination nobody chose. It hands back an
//! [`Ollama`] by name, because an agent turn asking for its next request needs
//! the pinned runtime typed as itself (ADR 0032, decision 4); what keeps a
//! caller from pointing one elsewhere is not the type's name, which
//! [`Ollama::at`] never hid, but that nothing shipped outside this crate makes
//! one — `alo-agentd` holds that of itself in a test.

use crate::handing_over;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::card::Vendor;
use crate::catalogue::Catalogue;
use crate::road::WhichRoad;
use crate::runtime::{Installed, Loaded, ModelRuntime, Progress, ProgressSink, RuntimeError};
use crate::weights::Weights;

/// **Which graphics cards the pinned runtime puts weights on.**
///
/// Ollama's own answer, which is why it lives in the file that knows Ollama
/// exists (ADR 0006): its releases ship CUDA and ROCm libraries and nothing
/// for anybody else's card, so an integrated Intel processor is a device on
/// the bus that this runtime will not load a model onto — which is a sentence
/// a person reads ([`crate::WhyTheProcessor::TheRuntimeCannotUseThatCard`])
/// rather than a slow afternoon.
///
/// It says nothing about whether a card was used. That is
/// [`ModelRuntime::loaded`], measured, and this list is only ever the reason a
/// machine gives for the road it took.
const CARDS_THE_PINNED_RUNTIME_USES: &[Vendor] = &[Vendor::Nvidia, Vendor::Amd];

/// Where Ollama listens by default. The same endpoint `alo-workplace`'s
/// `AiConfig` has documented since 2025, which is why pointing the agents at a
/// local model is configuration rather than new code.
pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:11434";

/// How long to wait on a call that should be quick. Listing what is installed
/// is a local read; if it has not answered in this long, the runtime is not
/// well, and saying so beats hanging a user interface.
///
/// A timeout here stays [`RuntimeError::Unreachable`] rather than becoming
/// [`RuntimeError::TookTooLong`], and the difference is real: a listing that
/// takes ten seconds means something is wrong, while a model that takes five
/// minutes means it is thinking. Only [`Ollama::answers`] can say the second.
const QUICK_TIMEOUT: Duration = Duration::from_secs(10);

/// How long this machine waits for a model on it to answer.
///
/// Longer than the two minutes `alo_asking::hosted` waits for a provider, and
/// deliberately: ADR 0007 makes the CPU the default, and a model thinking on a
/// CPU is slower than the same question put to somebody's service on a card
/// they own. **This machine waits longer for itself than for anybody else.**
/// Finite all the same — a wait with no end is indistinguishable from a hang,
/// and [`RuntimeError::TookTooLong`] is a sentence a person can act on.
const WHILE_A_MODEL_THINKS: Duration = Duration::from_secs(300);

/// The most of an answer that is read.
///
/// The reply is on this machine and is still not read without a bound: no model
/// writes a megabyte in one answer, so this costs nothing real, and an answer
/// past it is [`RuntimeError::Unusable`] rather than half of what the model
/// said. `alo_asking::hosted` holds a provider to the same number for the same
/// reason.
const MOST_OF_AN_ANSWER: u64 = 1_000_000;

/// How long a deliberately loaded model stays in video memory without use.
/// Long enough that a person who loaded it on purpose does not find it gone by
/// the time they have finished typing; short enough that a forgotten model
/// eventually gives the card back.
const DEFAULT_KEEP_ALIVE: &str = "30m";

/// The model runtime on this machine, where this machine has one.
///
/// **Found, never configured.**
/// [ADR 0019](../../../docs/decisions/0019-a-runtime-is-found-not-configured.md):
/// no contract, machine description or settings file names an address. A key
/// for one would be an Ollama endpoint sitting in a public surface, which is
/// ADR 0006's one-file rule broken, and — in the organisation's file — the
/// organisation choosing which runtime answers, which is ADR 0016's
/// bound-versus-choice line one indirection away. A local runtime is at a local
/// address, and that is a fact about the runtime rather than about a
/// deployment.
///
/// **Nothing found is an answer rather than a failure**, which is why this
/// answers with [`Option`] and not [`Result`]. A machine where nobody has
/// installed a runtime is an ordinary machine, and what a person is told about
/// it is the daemon's *nothing on this machine has been chosen to answer
/// questions*.
///
/// **A runtime holding no weights is nothing found too.** The image carries
/// the pinned runtime on every machine (ADR 0025), so a socket answering at
/// the known address stopped meaning a model is on the disk — and this door
/// opens onto what answers questions, which a runtime alone is not.
///
/// **Ask it each time rather than once.** It is one request to a socket on this
/// machine, refused immediately when nothing is listening, so a runtime started
/// after the service was is found the next time somebody asks — and one that has
/// stopped is not still being offered.
///
/// The type that comes back is named, for ADR 0032: the envelope can be asked
/// of the pinned runtime and of no trait object. There is still deliberately no
/// override for an operator to reach for — no address goes in here. A
/// genuinely remote runtime is a [`crate::Provider`], which alo OS already
/// models, shows on the indicator and bounds.
#[must_use]
pub fn found_on_this_machine(catalogue: Catalogue) -> Option<Ollama> {
    found_at(DEFAULT_ENDPOINT, catalogue)
}

/// The same question, asked at an address a test can serve on.
///
/// Private, and that privacy is the whole of the difference between this and
/// [`found_on_this_machine`]: nothing outside this file chooses where a runtime
/// is looked for.
fn found_at(endpoint: &str, catalogue: Catalogue) -> Option<Ollama> {
    let runtime = Ollama::at(endpoint, catalogue);
    // What is on disk is the runtime's own question, and only a runtime can
    // answer it. Something else listening at that address answers with
    // something this cannot read, which is nothing found rather than a runtime
    // that will fail on the first real question — ADR 0019 refuses discovery by
    // asking what happens to answer.
    //
    // And a runtime holding **no weights** is nothing found too. Since ADR
    // 0025 the image carries the runtime on every machine we build, so *the
    // runtime answered* stopped implying anybody put a model on the disk — and
    // what this door opens onto is a thing that answers questions, which an
    // empty runtime is not. A runtime alone must not read as a model: until
    // there are weights, discovery keeps giving ADR 0019's found-nothing
    // answer, and the machine keeps saying nothing here answers questions.
    runtime
        .installed()
        .ok()
        .filter(|weights| !weights.is_empty())
        .map(|_| runtime)
}

/// Ollama, reached over its HTTP API.
#[derive(Debug, Clone)]
pub struct Ollama {
    /// Where the runtime listens, without a trailing slash.
    endpoint: String,
    /// What alo OS offers. Held here because it is the gate on `fetch`.
    catalogue: Catalogue,
}

// Ollama's wire shapes. Every field is `#[serde(default)]` on purpose: a
// runtime that adds, renames or omits a field between versions should cost us
// a missing value, never a failed parse of an otherwise good response.

/// `/api/tags` — what is on disk.
#[derive(Deserialize)]
struct TagsResponse {
    /// One entry per set of weights the runtime holds.
    #[serde(default)]
    models: Vec<TagEntry>,
}

/// One installed model, as `/api/tags` describes it.
#[derive(Deserialize)]
struct TagEntry {
    /// Ollama's own `family:tag` name, which never escapes this file.
    #[serde(default)]
    name: String,
    /// Bytes on disk, as the runtime measures them.
    #[serde(default)]
    size: u64,
    /// Present on recent versions; absent on older ones.
    #[serde(default)]
    details: Option<TagDetails>,
}

/// The nested detail block of a `/api/tags` entry.
#[derive(Deserialize)]
struct TagDetails {
    /// The quantisation actually installed, spelled Ollama's way.
    #[serde(default)]
    quantization_level: Option<String>,
}

/// `/api/ps` — what is in video memory now.
#[derive(Deserialize)]
struct PsResponse {
    /// One entry per loaded model.
    #[serde(default)]
    models: Vec<PsEntry>,
}

/// One loaded model, as `/api/ps` describes it.
#[derive(Deserialize)]
struct PsEntry {
    /// Ollama's `family:tag` name.
    #[serde(default)]
    name: String,
    /// What the runtime loaded in all, wherever it put it. Not the disk size
    /// `/api/tags` reports for the same weights, which is a third number.
    #[serde(default)]
    size: u64,
    /// How much of that is on the graphics processor. Nought on a machine that
    /// loaded the weights onto its own processor, which is ADR 0007's ordinary
    /// case rather than a failure.
    #[serde(default)]
    size_vram: u64,
}

/// `/api/chat` — one question, put to a model on this machine.
///
/// Ollama's own chat call rather than its OpenAI-compatible one. Both work; this
/// is the runtime speaking its own language, which is what keeps ADR 0006's
/// promise literal — the file that knows Ollama exists uses Ollama's API, and
/// nothing about our shape is chosen to look like somebody else's.
#[derive(Serialize)]
struct ChatRequest<'a> {
    /// Ollama's `family:tag` name, applied here and nowhere else.
    model: String,
    /// The conversation, which is one message: alo OS composes no preamble of
    /// its own and sends no previous turn.
    messages: [ChatMessage<'a>; 1],
    /// Whole answers only. Nothing in this repository has decided what a
    /// half-arrived answer is, so none is asked for.
    stream: bool,
    /// The shape the answer is held to, for an agent turn's question and only
    /// for that ([ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md)).
    /// Absent on every question a person puts to a model.
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<serde_json::Value>,
    /// Which road to put the question on, where the caller named one. Absent
    /// on every question that did not, so the request a person's turn sends is
    /// byte for byte the request it sent before this field existed.
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<ChatOptions>,
}

/// The runtime's own options block, of which this crate sets one field.
///
/// Ollama's spelling, here for [`crate::ollama`]'s reason and nowhere else:
/// `num_gpu` is how many of a model's layers it puts on a graphics card, and
/// nought is the processor road asked for rather than waited for. That is the
/// only way to put one question to both roads on one machine — restarting the
/// runtime with its card hidden is an operation on the machine, not something
/// alo OS does to somebody's computer to answer a question about it.
#[derive(Serialize)]
struct ChatOptions {
    /// How many layers go on a graphics card. Nought is the processor.
    num_gpu: u32,
}

/// One message in that call.
#[derive(Serialize)]
struct ChatMessage<'a> {
    /// Who is speaking. Always the person.
    role: &'a str,
    /// What they asked. Borrowed, and this is the only place it goes.
    content: &'a str,
}

/// What `/api/chat` answers with.
#[derive(Deserialize)]
struct ChatResponse {
    /// What the model wrote, absent on a reply that is not an answer.
    #[serde(default)]
    message: Option<ChatSaid>,
}

/// The message a model wrote.
#[derive(Deserialize)]
struct ChatSaid {
    /// Its text.
    #[serde(default)]
    content: String,
}

/// What `/api/show` answers, of which this crate reads one field.
#[derive(Debug, Deserialize)]
struct Shown {
    /// The text `ollama show --modelfile` prints, whose `FROM` line names the
    /// blob on disk. See [`Ollama::which_file_it_holds`] for why this is where
    /// the digest has to be read from.
    #[serde(default)]
    modelfile: String,
}

/// The digest a generated modelfile's `FROM` line names, or [`None`] when it
/// names anything else.
///
/// Deliberately strict. A `FROM` line can also carry a bare model name or a
/// path to a file somebody handed over, and neither is a digest — so this
/// answers only for the one shape it can read, and its caller turns [`None`]
/// into a refusal rather than a shrug.
fn the_digest_in(modelfile: &str) -> Option<String> {
    for line in modelfile.lines() {
        let Some(rest) = line.trim_start().strip_prefix("FROM ") else {
            continue;
        };
        let named = rest.trim().rsplit('/').next().unwrap_or_default();
        let Some(digest) = named.strip_prefix("sha256-") else {
            continue;
        };
        // Sixty-four lowercase hexadecimal characters and nothing else, which
        // is the one spelling `Requantised::sha256` is held to, so the two are
        // compared as written rather than normalised into agreement.
        if digest.len() == 64
            && digest
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
        {
            return Some(digest.to_owned());
        }
    }
    None
}

/// One line of `/api/pull`'s streamed response.
#[derive(Deserialize)]
struct PullLine {
    /// Bytes fetched so far, absent on lines that only report status.
    #[serde(default)]
    completed: Option<u64>,
    /// Bytes expected, which the runtime does not always know at the start.
    #[serde(default)]
    total: Option<u64>,
    /// Set when the download failed. Its text is never repeated to a caller.
    #[serde(default)]
    error: Option<String>,
}

impl Ollama {
    /// An adapter pointed at the default endpoint.
    #[must_use]
    pub fn new(catalogue: Catalogue) -> Self {
        Self::at(DEFAULT_ENDPOINT, catalogue)
    }

    /// An adapter pointed somewhere else — a test server, or a runtime on a
    /// paired machine once shared inference arrives (ADR 0003).
    #[must_use]
    pub fn at(endpoint: &str, catalogue: Catalogue) -> Self {
        Self {
            endpoint: endpoint.trim_end_matches('/').to_owned(),
            catalogue,
        }
    }

    /// Ollama names a model `family:tag`; our catalogue ids are our own. The
    /// mapping is one line today and may not stay that way, but it stays *here*
    /// either way.
    fn runtime_name(id: &str) -> String {
        if id.contains(':') {
            id.to_owned()
        } else {
            format!("{id}:latest")
        }
    }

    /// …and back, so what the runtime reports can be matched to the catalogue.
    fn catalogue_id(runtime_name: &str) -> String {
        runtime_name
            .strip_suffix(":latest")
            .unwrap_or(runtime_name)
            .to_owned()
    }

    /// One GET against the runtime, read to a string. Only for the small
    /// replies — a download is streamed, not buffered.
    fn get(&self, path: &str) -> Result<String, RuntimeError> {
        ureq::get(format!("{}{path}", self.endpoint))
            .config()
            .timeout_global(Some(QUICK_TIMEOUT))
            .build()
            .call()
            .map_err(|_| RuntimeError::Unreachable)?
            .body_mut()
            .read_to_string()
            .map_err(|_| RuntimeError::Unusable)
    }
}

impl Ollama {
    /// The digest of the file the runtime holds for this artefact, as the
    /// runtime itself says.
    ///
    /// # Why this asks the runtime rather than the registry
    ///
    /// The registry's manifest is a **promise**; this is the **goods**. A
    /// manifest read before the download says what the registry intended to
    /// serve, which is one tag-move away from what arrived, and checking it
    /// would be checking a proxy for the thing — the same shape as a recipe
    /// that tests a file's executable bit and calls the program working. It
    /// also keeps every request this crate makes on this machine: the runtime
    /// does the reaching out, as it already did before this check existed.
    ///
    /// # What was measured, against the pinned release
    ///
    /// Ollama 0.34.0, 2026-09-20. `/api/show` has **no field that states a
    /// digest**. What it has is `modelfile`, the text `ollama show --modelfile`
    /// prints, whose `FROM` line names the blob on disk:
    ///
    /// ```text
    /// FROM /…/.ollama/models/blobs/sha256-2e8040ce…68c2d
    /// ```
    ///
    /// That the name of that blob is the `sha256` of the GGUF the catalogue
    /// pins was measured three ways on one real pull of
    /// `hf.co/bartowski/SmolLM2-135M-Instruct-GGUF:Q4_K_M`: the registry
    /// manifest's `application/vnd.ollama.image.model` layer, the local
    /// manifest the runtime wrote, and `sha256sum` of the file itself — all
    /// `2e8040ce…68c2d`. Both catalogue entries that state a pin were checked
    /// against their registry manifests and agree with it exactly.
    /// `docs/quirks.md` carries the measurement.
    ///
    /// # Errors
    /// [`RuntimeError::PinNotChecked`] when the answer holds no digest this can
    /// read — **a refusal, because a check that could not be made has not
    /// passed.** That is the variant that will fire if a later release stops
    /// printing the `FROM` line this reads, which is the point of it: the pin
    /// stops being checkable loudly rather than quietly.
    fn which_file_it_holds(&self, id: &str, artefact: &str) -> Result<String, RuntimeError> {
        let body = serde_json::json!({ "model": artefact });
        let response = ureq::post(format!("{}/api/show", self.endpoint))
            .send_json(&body)
            .map_err(|_| RuntimeError::Unreachable)?;
        let shown: Shown = response
            .into_body()
            .read_json()
            .map_err(|_| RuntimeError::Unusable)?;
        the_digest_in(&shown.modelfile).ok_or_else(|| RuntimeError::PinNotChecked(id.to_owned()))
    }
}

impl ModelRuntime for Ollama {
    fn installed(&self) -> Result<Vec<Installed>, RuntimeError> {
        let body = self.get("/api/tags")?;
        let tags: TagsResponse = serde_json::from_str(&body).map_err(|_| RuntimeError::Unusable)?;
        Ok(tags
            .models
            .into_iter()
            .map(|m| Installed {
                id: Self::catalogue_id(&m.name),
                bytes_on_disk: m.size,
                quantisation: m.details.and_then(|d| d.quantization_level),
            })
            .collect())
    }

    fn loaded(&self) -> Result<Vec<Loaded>, RuntimeError> {
        let body = self.get("/api/ps")?;
        let ps: PsResponse = serde_json::from_str(&body).map_err(|_| RuntimeError::Unusable)?;
        Ok(ps
            .models
            .into_iter()
            .map(|m| Loaded {
                id: Self::catalogue_id(&m.name),
                loaded_bytes: m.size,
                on_the_gpu_bytes: m.size_vram,
            })
            .collect())
    }

    fn fetch(&self, id: &str, progress: &mut dyn ProgressSink) -> Result<(), RuntimeError> {
        // The catalogue gate. Without it, any name could be pulled and the
        // licence promise in docs/features.md would mean nothing.
        let Some(entry) = self.catalogue.get(id) else {
            return Err(RuntimeError::NotOffered(id.to_owned()));
        };
        // **What is fetched is the file the entry names, not the entry's id.**
        // An id is this catalogue's name for a model; the registry knows the
        // artefact. Asked for `mistral-7b-instruct:latest`, the pinned runtime
        // answers `pull model manifest: file does not exist` (`docs/quirks.md`),
        // so an entry that names no file is one there is nothing to fetch for.
        let Some(artefact) = entry.artefact.clone() else {
            return Err(RuntimeError::NotOffered(id.to_owned()));
        };
        let template = entry.chat_template.as_ref().map(|t| t.text.clone());

        let body = serde_json::json!({ "model": artefact, "stream": true });
        let response = ureq::post(format!("{}/api/pull", self.endpoint))
            .send_json(&body)
            .map_err(|_| RuntimeError::Unreachable)?;

        // The response is a stream of JSON objects, one per line, for as long as
        // the download runs. Read it line by line rather than to a string: these
        // are gigabytes, and a progress report that arrives at the end is not a
        // progress report.
        let reader = std::io::BufReader::new(response.into_body().into_reader());
        let mut saw_error = None;
        {
            use std::io::BufRead as _;
            for line in reader.lines() {
                let Ok(line) = line else {
                    return Err(RuntimeError::Unreachable);
                };
                if line.trim().is_empty() {
                    continue;
                }
                let Ok(parsed) = serde_json::from_str::<PullLine>(&line) else {
                    // A line we cannot read is not a reason to abandon a
                    // multi-gigabyte download; the next one usually parses.
                    continue;
                };
                if parsed.error.is_some() {
                    saw_error = Some(());
                    continue;
                }
                if let Some(done) = parsed.completed {
                    progress.advanced(Progress {
                        done_bytes: done,
                        total_bytes: parsed.total,
                    });
                }
            }
        }
        if saw_error.is_some() {
            // The runtime's own words are not repeated: this crate's errors
            // never carry a backend response body, and since item 9f they
            // cannot — the reason is a variant with a string of its own rather
            // than a sentence this file wrote.
            return Err(RuntimeError::DownloadIncomplete);
        }
        // **The pin, checked against what the machine actually got.** Only for
        // an entry that states one: an entry without a `[model.requantised]`
        // block asks no question here and is neither slowed nor newly able to
        // fail, because nothing below runs for it.
        if let Some(pinned) = entry.requantised.as_ref() {
            let arrived = self.which_file_it_holds(id, &artefact)?;
            if arrived != pinned.sha256 {
                return Err(RuntimeError::NotThePinnedFile {
                    model: id.to_owned(),
                    expected: pinned.sha256.clone(),
                    arrived,
                });
            }
        }
        self.named_for_the_catalogue(id, &artefact, template.as_deref())
    }

    fn remove(&self, id: &str) -> Result<(), RuntimeError> {
        // Ollama deletes with a DELETE *carrying a body*, which is unusual
        // enough that ureq's `delete()` builder cannot express it — its DELETE
        // is body-less. Building the request directly is the way through, and
        // is exactly the kind of runtime-shaped awkwardness this file exists to
        // absorb.
        let body = serde_json::json!({ "model": Self::runtime_name(id) }).to_string();
        let request = ureq::http::Request::builder()
            .method("DELETE")
            .uri(format!("{}/api/delete", self.endpoint))
            .header("content-type", "application/json")
            .body(body)
            .map_err(|_| RuntimeError::Unusable)?;
        let response = ureq::Agent::new_with_defaults().run(request);
        match response {
            Ok(_) => Ok(()),
            // Ollama answers 404 for a model it does not have. That is not a
            // failure to report as one — the caller asked for the disk back and
            // the disk is already back.
            Err(ureq::Error::StatusCode(404)) => Err(RuntimeError::NotInstalled(id.to_owned())),
            Err(_) => Err(RuntimeError::Unreachable),
        }
    }

    fn load(&self, id: &str) -> Result<(), RuntimeError> {
        // Ollama has no load call either: an empty generate with a non-zero
        // keep-alive brings the weights into video memory and leaves them
        // there. Same endpoint as unload, opposite keep-alive — which is
        // precisely why both belong in this file and neither belongs in the
        // trait's vocabulary.
        self.keep_alive(id, DEFAULT_KEEP_ALIVE)
    }

    fn unload(&self, id: &str) -> Result<(), RuntimeError> {
        self.keep_alive(id, "0")
    }

    fn answers(&self, question: &str, of_model: &str) -> Result<String, RuntimeError> {
        self.chat(question, of_model, None, WhichRoad::AsTheMachineIs)
    }

    fn cards_it_can_use(&self) -> &'static [Vendor] {
        CARDS_THE_PINNED_RUNTIME_USES
    }

    fn bring(&self, weights: &Weights) -> Result<(), RuntimeError> {
        let Some(file) = weights.file.as_ref() else {
            return Err(RuntimeError::NothingToBring(weights.id.clone()));
        };

        // **A bare name is a publisher's to fetch.** `FROM mistral` in a
        // Modelfile is an instruction to download, so a path that is not
        // absolute and on this disk would turn *run the weights you already
        // have* into an egress nobody asked for. Refused before anything is
        // sent, which is the only place it can be refused for certain: once the
        // runtime holds the Modelfile, it is the runtime deciding.
        if !file.is_absolute() || !file.is_file() {
            return Err(RuntimeError::NotAPathOnThisDisk(file.clone()));
        }

        // The file into the runtime's store under its own digest, then the
        // model made from that blob by name — the two requests the pinned
        // runtime takes. `handing_over` has what 0.34.0 answered to each, and to
        // the one-line Modelfile this door used to send.
        let Some(file_name) = file.file_name().and_then(|name| name.to_str()) else {
            return Err(RuntimeError::NotAPathOnThisDisk(file.clone()));
        };
        let digest = handing_over::digest_of(file)?;
        handing_over::handed_over(&self.endpoint, file, &digest, WHILE_A_MODEL_THINKS)?;
        let body = handing_over::the_create(&Self::runtime_name(&weights.id), file_name, &digest);
        let response = ureq::post(format!("{}/api/create", self.endpoint))
            .config()
            .timeout_global(Some(WHILE_A_MODEL_THINKS))
            .build()
            .send_json(&body);
        match response {
            Ok(_) => Ok(()),
            // It is there and it is working: importing weights copies
            // gigabytes, and the reasoning `answers` gives applies here.
            Err(ureq::Error::Timeout(_)) => Err(RuntimeError::TookTooLong),
            // The runtime looked at the file and would not take it — an
            // architecture it does not know, a file that is not what its name
            // says. Its own refusal, carried as one rather than reworded,
            // because this crate never repeats a backend's words.
            Err(ureq::Error::StatusCode(_)) => Err(RuntimeError::Unusable),
            Err(_) => Err(RuntimeError::Unreachable),
        }
    }
}

impl Ollama {
    /// **A question from an agent turn, held to the protocol's envelope** —
    /// [ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md).
    ///
    /// The same request as [`ModelRuntime::answers`] with one field more: the
    /// runtime is asked to hold the answer to
    /// [`crate::in_the_envelope::the_envelope`] — the protocol's version and
    /// exactly one of the three doors, nothing inside. Never used for a question
    /// a person asks: that is answered in prose, through `answers`.
    ///
    /// # Errors
    /// The same as [`ModelRuntime::answers`].
    pub fn answers_in_the_envelope(
        &self,
        question: &str,
        of_model: &str,
    ) -> Result<String, RuntimeError> {
        self.answers_in_the_envelope_on(question, of_model, WhichRoad::AsTheMachineIs)
    }

    /// **The same question, put on a road the caller names.**
    ///
    /// [`WhichRoad::AsTheMachineIs`] is what every other door here asks and
    /// sends no option at all. [`WhichRoad::TheProcessor`] asks the runtime for
    /// the processor whatever is on the bus, and exists for one reason:
    /// [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md) says
    /// *a GPU changes speed, not capability*, and the only way to hold that
    /// claim rather than assert it is to put one question to each road **on the
    /// same machine, to the same weights** and compare what comes back.
    /// `alo_driving::BothRoads` is what compares them.
    ///
    /// # Errors
    /// The same as [`ModelRuntime::answers`].
    pub fn answers_in_the_envelope_on(
        &self,
        question: &str,
        of_model: &str,
        road: WhichRoad,
    ) -> Result<String, RuntimeError> {
        self.chat(
            question,
            of_model,
            Some(crate::in_the_envelope::the_envelope()),
            road,
        )
    }

    /// A question in a person's own words, put on a road the caller names.
    ///
    /// # Errors
    /// The same as [`ModelRuntime::answers`].
    pub fn answers_on(
        &self,
        question: &str,
        of_model: &str,
        road: WhichRoad,
    ) -> Result<String, RuntimeError> {
        self.chat(question, of_model, None, road)
    }

    /// One question put to the runtime, held to `format` where one is given,
    /// on the road the caller named.
    fn chat(
        &self,
        question: &str,
        of_model: &str,
        format: Option<serde_json::Value>,
        road: WhichRoad,
    ) -> Result<String, RuntimeError> {
        let body = ChatRequest {
            model: Self::runtime_name(of_model),
            messages: [ChatMessage {
                role: "user",
                content: question,
            }],
            stream: false,
            format,
            options: match road {
                WhichRoad::AsTheMachineIs => None,
                WhichRoad::TheProcessor => Some(ChatOptions { num_gpu: 0 }),
            },
        };
        let response = ureq::post(format!("{}/api/chat", self.endpoint))
            .config()
            .timeout_global(Some(WHILE_A_MODEL_THINKS))
            .build()
            .send_json(&body);
        let mut response = match response {
            Ok(response) => response,
            // Ollama answers 404 for a model it does not hold. The person asked
            // a model that is not on this machine, which is a different thing
            // from the runtime not being there.
            Err(ureq::Error::StatusCode(404)) => {
                return Err(RuntimeError::NotInstalled(of_model.to_owned()));
            }
            // It is there and it is thinking. ADR 0007's CPU default makes this
            // ordinary rather than broken, and saying "nothing was running"
            // would send somebody to look at a runtime that is busy.
            Err(ureq::Error::Timeout(_)) => return Err(RuntimeError::TookTooLong),
            // Something answered, and not with an answer. Its own words are not
            // repeated: this crate's errors never carry a backend response body.
            Err(ureq::Error::StatusCode(_)) => return Err(RuntimeError::Unusable),
            Err(_) => return Err(RuntimeError::Unreachable),
        };
        let body = response
            .body_mut()
            .with_config()
            .limit(MOST_OF_AN_ANSWER)
            .read_to_string()
            .map_err(|_| RuntimeError::Unusable)?;
        let said: ChatResponse = serde_json::from_str(&body).map_err(|_| RuntimeError::Unusable)?;
        let said = said.message.map(|m| m.content).unwrap_or_default();
        if said.trim().is_empty() {
            // A reply with no answer in it is not an empty answer to show
            // somebody: they asked something and nothing came back.
            return Err(RuntimeError::Unusable);
        }
        Ok(said)
    }
}

impl Ollama {
    /// **The fetched file, made answerable by the catalogue's id** — and, for an
    /// entry whose file carries no chat template, asked through its publisher's.
    ///
    /// `/api/create` from the artefact just pulled: `from` names a model the
    /// runtime now holds, never a path and never something to fetch, and
    /// `template` is carried only when the entry states one. Afterwards every
    /// other method here — `answers`, `load`, `remove` — reaches the model by the
    /// id a person saw in the catalogue.
    fn named_for_the_catalogue(
        &self,
        id: &str,
        artefact: &str,
        template: Option<&str>,
    ) -> Result<(), RuntimeError> {
        let mut body = serde_json::json!({
            "model": Self::runtime_name(id),
            "from": artefact,
            "stream": false,
        });
        if let (Some(template), Some(fields)) = (template, body.as_object_mut()) {
            fields.insert("template".to_owned(), serde_json::Value::from(template));
        }
        let response = ureq::post(format!("{}/api/create", self.endpoint))
            .config()
            .timeout_global(Some(WHILE_A_MODEL_THINKS))
            .build()
            .send_json(&body);
        match response {
            Ok(_) => Ok(()),
            Err(ureq::Error::Timeout(_)) => Err(RuntimeError::TookTooLong),
            Err(ureq::Error::StatusCode(_)) => Err(RuntimeError::Unusable),
            Err(_) => Err(RuntimeError::Unreachable),
        }
    }
}

impl Ollama {
    /// Ask the runtime to hold a model in video memory for `keep_alive`, or to
    /// let it go when that is `"0"`.
    fn keep_alive(&self, id: &str, keep_alive: &str) -> Result<(), RuntimeError> {
        let body = serde_json::json!({
            "model": Self::runtime_name(id),
            "keep_alive": keep_alive,
        });
        let response = ureq::post(format!("{}/api/generate", self.endpoint))
            .config()
            .timeout_global(Some(QUICK_TIMEOUT))
            .build()
            .send_json(&body);
        match response {
            Ok(_) => Ok(()),
            Err(ureq::Error::StatusCode(404)) => Err(RuntimeError::NotInstalled(id.to_owned())),
            Err(_) => Err(RuntimeError::Unreachable),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on a bad index or a None is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{serving, serving_each, serving_in_turn};

    fn catalogue() -> Catalogue {
        Catalogue::built_in().unwrap()
    }

    /// **A machine with no runtime finds none**, and that is an answer rather
    /// than a failure: ADR 0019's *discovery that finds nothing is an answer*,
    /// as the shape of the type rather than as a sentence.
    #[test]
    fn a_machine_with_nothing_listening_finds_no_runtime() {
        assert!(found_at("http://127.0.0.1:1", catalogue()).is_none());
    }

    /// **A runtime that is there is found, and is the thing that then answers.**
    /// The whole road ADR 0019 opens, in one test: nobody named an address,
    /// something was found at the one this file knows, and a question went to
    /// it.
    #[test]
    fn a_runtime_on_this_machine_is_found_and_is_what_answers() {
        let (url, server) = serving_in_turn(
            &[
                r#"{"models":[{"name":"a-model:latest","size":1}]}"#,
                r#"{"message":{"role":"assistant","content":"No, not without written consent."}}"#,
            ],
            200,
        );
        let found = found_at(&url, catalogue()).unwrap();
        let answer = found.answers("may the tenant sublet?", "a-model").unwrap();
        let asked = server.join().unwrap();

        assert_eq!(answer, "No, not without written consent.");
        // Finding one is the runtime's own question, and asking it something is
        // the caller's — two calls, in that order.
        assert!(asked[0].starts_with("GET /api/tags "), "{}", asked[0]);
        assert!(asked[1].starts_with("POST /api/chat "), "{}", asked[1]);
    }

    /// **A runtime with no weights on its disk is nothing found.** ADR 0025
    /// put the pinned runtime on every image this repository builds, without
    /// the weights — so on every shipped machine, something now answers at
    /// the one address this file knows, holding nothing that could answer a
    /// question. A runtime alone must not read as a model: discovery keeps
    /// giving ADR 0019's found-nothing answer until there are weights, and a
    /// machine in that state keeps saying nothing on it answers questions
    /// rather than routing somebody's question to an empty runtime.
    #[test]
    fn a_runtime_with_no_weights_is_nothing_found() {
        let (url, server) = serving(r#"{"models":[]}"#, 200);
        assert!(found_at(&url, catalogue()).is_none());
        server.join().unwrap();
    }

    /// **Something else listening is not a runtime found.** ADR 0019 refuses
    /// discovery by looking at whatever answers, and this is that rule at the
    /// one address there is: a reply this cannot read is nothing found, rather
    /// than a runtime that would fail on somebody's first question.
    #[test]
    fn something_that_is_not_a_runtime_answering_is_not_a_runtime_found() {
        let (url, server) = serving("<!doctype html><title>Welcome</title>", 200);
        assert!(found_at(&url, catalogue()).is_none());
        server.join().unwrap();
    }

    /// **The address a runtime is looked for at is on this machine**, so
    /// discovery itself can never reach off it — law 1's zero inference egress
    /// covers the finding as well as the asking, and it is the address rather
    /// than a caller's care that makes it true.
    #[test]
    fn a_runtime_is_only_ever_looked_for_on_this_machine() {
        assert!(
            DEFAULT_ENDPOINT.starts_with("http://127.0.0.1:"),
            "{DEFAULT_ENDPOINT}"
        );
    }

    /// Compare request bodies without depending on how the HTTP client chooses
    /// to lay JSON out — it pretty-prints today and may not tomorrow, and a
    /// test that fails over a space is a test that will be silenced.
    fn without_spaces(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    /// **The digest is read only from the shape it can be read from**, and
    /// every other shape is nothing rather than a guess.
    ///
    /// This is about the failure that would be silent. The runtime states no
    /// digest anywhere structured; what it states is a generated modelfile
    /// whose `FROM` line names a blob. If a later release stops printing that
    /// line, or prints a path with no digest in it, this must answer *nothing*
    /// so that its caller refuses — because the alternative is a pin that
    /// quietly stops being checked while the catalogue goes on claiming it
    /// vouches for the file.
    #[test]
    fn the_digest_is_read_only_from_the_shape_it_can_be_read_from() {
        /// A real digest, of a real 105 MB artefact, measured 2026-09-20.
        const REAL: &str = "2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d";

        // The shape Ollama 0.34.0 really prints, measured.
        let real = format!(
            "# Modelfile generated by \"ollama show\"\n\
             # To build a new Modelfile based on this, replace FROM with:\n\
             # FROM hf.co/bartowski/SmolLM2-135M-Instruct-GGUF:Q4_K_M\n\n\
             FROM /Users/someone/.ollama/models/blobs/sha256-{REAL}\n\
             TEMPLATE \"\"\"{{{{ .Prompt }}}}\"\"\"\n"
        );
        assert_eq!(the_digest_in(&real).as_deref(), Some(REAL));

        // The commented line above the real one names the model rather than a
        // blob, which is why `FROM ` is wanted at the start of a line.
        assert_eq!(
            the_digest_in("# FROM hf.co/bartowski/SmolLM2-135M-Instruct-GGUF:Q4_K_M\n"),
            None
        );

        for (why, modelfile) in [
            ("no FROM line at all", "TEMPLATE \"\"\"x\"\"\"\n".to_owned()),
            (
                "a bare model name, which is what a Modelfile a person wrote carries",
                "FROM mistral\n".to_owned(),
            ),
            (
                "a path to weights somebody handed over, with no digest in it",
                "FROM /home/anna/models/weights.gguf\n".to_owned(),
            ),
            (
                "a blob named something other than a sha256",
                "FROM /var/lib/blobs/sha512-abc\n".to_owned(),
            ),
            (
                "too few characters to be a digest",
                format!("FROM /b/sha256-{}\n", &REAL[..63]),
            ),
            ("too many", format!("FROM /b/sha256-{REAL}0\n")),
            (
                "the right length and not hexadecimal",
                format!("FROM /b/sha256-{}\n", "z".repeat(64)),
            ),
            (
                "upper case, which the catalogue's own spelling is not",
                format!("FROM /b/sha256-{}\n", REAL.to_uppercase()),
            ),
        ] {
            assert_eq!(
                the_digest_in(&modelfile),
                None,
                "a modelfile with {why} was read as naming a digest"
            );
        }
    }

    /// **The listing Ollama 0.34.0 really sends**, verbatim off the runtime on
    /// 2026-09-13, reads as the fixture above assumed: the name, the size and the
    /// quantisation, with every field this crate does not read ignored rather
    /// than refused. The runtime's version is in the test's name because a later
    /// one is a later fixture.
    #[test]
    fn the_listing_ollama_0_34_0_sends_is_read() {
        let (url, server) = serving(
            r#"{"models":[{"name":"mistral:7b-instruct-v0.3-q4_K_M","model":"mistral:7b-instruct-v0.3-q4_K_M","modified_at":"2026-09-13T20:55:04.463878831+02:00","size":4372824384,"digest":"6577803aa9a036369e481d648a2baebb381ebc6e897f2bb9a766a2aa7bfbc1cf","details":{"parent_model":"","format":"gguf","family":"llama","families":["llama"],"parameter_size":"7.2B","quantization_level":"Q4_K_M","context_length":32768,"embedding_length":4096},"capabilities":["completion","tools"]}]}"#,
            200,
        );
        let got = Ollama::at(&url, catalogue()).installed().unwrap();
        server.join().unwrap();
        assert_eq!(
            got,
            vec![Installed {
                id: "mistral:7b-instruct-v0.3-q4_K_M".to_owned(),
                bytes_on_disk: 4_372_824_384,
                quantisation: Some("Q4_K_M".to_owned()),
            }]
        );
    }

    /// **The answer Ollama 0.34.0 really sends to `/api/chat`**, verbatim off
    /// the runtime on 2026-09-13, is read for its text alone.
    #[test]
    fn the_answer_ollama_0_34_0_sends_is_read() {
        let (url, server) = serving(
            r#"{"model":"qwen2.5:7b-instruct-q4_K_M","created_at":"2026-09-13T21:07:11.188133Z","message":{"role":"assistant","content":"Ready."},"done":true,"done_reason":"stop","total_duration":6364345416,"load_duration":5865330791,"prompt_eval_count":37,"prompt_eval_cached_count":0,"prompt_eval_duration":374170000,"eval_count":3,"eval_duration":109666000}"#,
            200,
        );
        let said = Ollama::at(&url, catalogue())
            .answers(
                "Answer with the single word: ready.",
                "qwen2.5:7b-instruct-q4_K_M",
            )
            .unwrap();
        let sent = server.join().unwrap();
        assert_eq!(said, "Ready.");
        assert!(sent.starts_with("POST /api/chat "), "{sent}");
        let body = sent.split_once("\r\n\r\n").map(|(_, body)| body).unwrap();
        let body: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(body.get("stream").unwrap(), false);
        assert_eq!(body.pointer("/messages/0/role").unwrap(), "user");
    }

    /// **A question a person asks carries no shape, and an agent turn's carries
    /// exactly the envelope** — decisions 1 and 3 of ADR 0032, as the bytes that
    /// go to the runtime.
    #[test]
    fn only_an_agent_turns_question_is_held_to_the_envelope() {
        let reply = r#"{"message":{"role":"assistant","content":"No."}}"#;

        let (url, server) = serving(reply, 200);
        Ollama::at(&url, catalogue())
            .answers("may the tenant sublet?", "a-model")
            .unwrap();
        let asked = server.join().unwrap();
        let body = asked.split_once("\r\n\r\n").map(|(_, body)| body).unwrap();
        let body: serde_json::Value = serde_json::from_str(body).unwrap();
        assert!(
            body.get("format").is_none(),
            "a person's question was given a shape: {body}"
        );

        let (url, server) = serving(reply, 200);
        Ollama::at(&url, catalogue())
            .answers_in_the_envelope("the next request, please", "a-model")
            .unwrap();
        let asked = server.join().unwrap();
        let body = asked.split_once("\r\n\r\n").map(|(_, body)| body).unwrap();
        let body: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(
            body.get("format"),
            Some(&crate::in_the_envelope::the_envelope()),
            "{body}"
        );
        assert_eq!(body.get("stream").unwrap(), false);
    }

    #[test]
    fn installed_reads_what_is_on_disk_and_drops_the_latest_tag() {
        let (url, server) = serving(
            r#"{"models":[{"name":"mistral-7b-instruct:latest","size":4370000000,"details":{"quantization_level":"Q4_K_M"}}]}"#,
            200,
        );
        let got = Ollama::at(&url, catalogue()).installed().unwrap();
        server.join().unwrap();
        assert_eq!(
            got,
            vec![Installed {
                // ":latest" is Ollama's convention, not ours, and must not
                // escape this file into catalogue ids.
                id: "mistral-7b-instruct".to_owned(),
                bytes_on_disk: 4_370_000_000,
                quantisation: Some("Q4_K_M".to_owned()),
            }]
        );
    }

    /// **Both halves of a residency are read**, which is the pair a grade
    /// records and what says which road a machine took.
    #[test]
    fn loaded_reads_what_was_loaded_and_how_much_of_it_is_on_the_card() {
        let (url, server) = serving(
            r#"{"models":[{"name":"teuken-7b-instruct:latest","size":6100000000,"size_vram":4563287407}]}"#,
            200,
        );
        let got = Ollama::at(&url, catalogue()).loaded().unwrap();
        server.join().unwrap();
        assert_eq!(
            got,
            vec![Loaded {
                id: "teuken-7b-instruct".to_owned(),
                loaded_bytes: 6_100_000_000,
                on_the_gpu_bytes: 4_563_287_407,
            }]
        );
    }

    /// **A machine that loaded the weights onto its processor reports nought
    /// on the card**, and that is a reading rather than a missing one: it is
    /// what makes the processor road measured instead of assumed.
    #[test]
    fn a_model_loaded_onto_the_processor_reports_nothing_on_the_card() {
        let (url, server) = serving(
            r#"{"models":[{"name":"mistral-7b-instruct:latest","size":4600000000,"size_vram":0}]}"#,
            200,
        );
        let got = Ollama::at(&url, catalogue()).loaded().unwrap();
        server.join().unwrap();
        assert_eq!(
            got,
            vec![Loaded {
                id: "mistral-7b-instruct".to_owned(),
                loaded_bytes: 4_600_000_000,
                on_the_gpu_bytes: 0,
            }]
        );
    }

    /// The catalogue gate, which is what makes the licence promise real. No
    /// request should even be attempted for a model we do not offer.
    #[test]
    fn fetching_a_model_the_catalogue_does_not_offer_is_refused_without_asking_the_runtime() {
        // Port 1 is not listening; if the gate leaked we would get Unreachable
        // instead of NotOffered, and the test would say so.
        let ollama = Ollama::at("http://127.0.0.1:1", catalogue());
        let err = ollama
            .fetch("some-uncurated-model", &mut Progress::ignored())
            .unwrap_err();
        assert!(
            matches!(&err, RuntimeError::NotOffered(id) if id == "some-uncurated-model"),
            "{err:?}"
        );
    }

    #[test]
    fn fetching_reports_progress_as_the_download_advances() {
        let (url, server) = serving_each(&[
            (
                200,
                "{\"status\":\"pulling\",\"completed\":100,\"total\":400}\n\
                 {\"status\":\"pulling\",\"completed\":400,\"total\":400}\n\
                 {\"status\":\"success\"}\n",
            ),
            (200, r#"{"status":"success"}"#),
        ]);
        let mut seen: Vec<Progress> = Vec::new();
        let result = Ollama::at(&url, catalogue()).fetch("mistral-7b-instruct", &mut |p| {
            seen.push(p);
        });
        let sent = server.join().unwrap();
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(seen.len(), 2, "{seen:?}");
        assert_eq!(seen[0].fraction(), Some(0.25));
        assert_eq!(seen[1].fraction(), Some(1.0));
        // **What is pulled is the artefact the entry names**, which is what the
        // registry knows — never the catalogue's own id.
        let pulled = sent.first().unwrap();
        assert!(pulled.starts_with("POST /api/pull "), "{pulled}");
        assert!(
            pulled.contains("mistral:7b-instruct-v0.3-q4_K_M"),
            "{pulled}"
        );
        assert!(!pulled.contains("mistral-7b-instruct:latest"), "{pulled}");
    }

    /// **After the pull, the model is made answerable by the catalogue's id**,
    /// from the file just pulled — and an entry whose file carries its own
    /// template is not given one.
    #[test]
    fn a_fetched_file_is_named_for_the_catalogue_and_given_no_template_it_did_not_need() {
        let (url, server) = serving_each(&[
            (200, "{\"status\":\"success\"}\n"),
            (200, r#"{"status":"success"}"#),
        ]);
        Ollama::at(&url, catalogue())
            .fetch("mistral-7b-instruct", &mut Progress::ignored())
            .unwrap();
        let sent = server.join().unwrap();
        let made = sent.get(1).unwrap();
        assert!(made.starts_with("POST /api/create "), "{made}");
        let body = made.split_once("\r\n\r\n").map(|(_, body)| body).unwrap();
        let body: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(body.get("model").unwrap(), "mistral-7b-instruct:latest");
        assert_eq!(body.get("from").unwrap(), "mistral:7b-instruct-v0.3-q4_K_M");
        assert!(body.get("template").is_none(), "{body}");
    }

    /// The digest `data/catalogue.toml` pins for Teuken, which is also what the
    /// registry's own manifest reports for that artefact (`docs/quirks.md`).
    const THE_TEUKEN_PIN: &str = "03fd13daafb6f20c1c5f4b908d163841cdd96803a6f5a7c0c39dd236a2b1630b";

    /// A digest of the right shape that is not the one the catalogue pins.
    const ANOTHER_FILE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    /// What `/api/show` answers for the file the entry pins, in the shape
    /// Ollama 0.34.0 really sends — a generated modelfile whose `FROM` line
    /// names the blob on disk.
    const SHOWN_HOLDING_THE_PINNED_FILE: &str = r##"{"modelfile":"# Modelfile generated by \"ollama show\"\n\nFROM /root/.ollama/models/blobs/sha256-03fd13daafb6f20c1c5f4b908d163841cdd96803a6f5a7c0c39dd236a2b1630b\n"}"##;

    /// The same, for a different file.
    const SHOWN_HOLDING_ANOTHER_FILE: &str = r#"{"modelfile":"FROM /root/.ollama/models/blobs/sha256-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n"}"#;

    /// The same, saying nothing this can read a digest out of.
    const SHOWN_SAYING_NOTHING: &str = r#"{"modelfile":"TEMPLATE \"\"\"x\"\"\"\n"}"#;

    /// **A file that is not the one pinned is refused, and the refusal names
    /// both digests.**
    ///
    /// The unit half of `tests/the_pin_an_entry_states.rs`, which measures the
    /// same refusal against the real runtime. This one holds the shape of the
    /// error and that **nothing is named for the catalogue afterwards** — a
    /// third request would mean the fetch carried on past the mismatch.
    #[test]
    fn a_fetch_whose_file_is_not_the_one_pinned_is_refused() {
        // Two replies, not three: the pull and the question about which file
        // arrived. Queueing a third would be queueing the create this fetch
        // must never reach — and the stand-in waits for one connection per
        // reply, so an unused third reply is a test that hangs rather than one
        // that fails.
        let (url, server) = serving_each(&[
            (200, "{\"status\":\"success\"}\n"),
            (200, SHOWN_HOLDING_ANOTHER_FILE),
        ]);
        let refused = Ollama::at(&url, catalogue())
            .fetch("teuken-7b-instruct", &mut Progress::ignored())
            .unwrap_err();
        assert_eq!(
            refused,
            RuntimeError::NotThePinnedFile {
                model: "teuken-7b-instruct".to_owned(),
                expected: THE_TEUKEN_PIN.to_owned(),
                arrived: ANOTHER_FILE.to_owned(),
            }
        );
        let sent = server.join().unwrap();
        assert_eq!(
            sent.len(),
            2,
            "the fetch asked for something after refusing: {sent:?}"
        );
        assert!(sent.first().unwrap().starts_with("POST /api/pull "));
        assert!(sent.get(1).unwrap().starts_with("POST /api/show "));
    }

    /// **A runtime that will not say which file it holds is a refusal, not a
    /// pass.**
    ///
    /// The variant that exists for the day a later release stops printing the
    /// line this reads. Measured here as an answer with no `FROM` line in it.
    #[test]
    fn a_runtime_that_says_nothing_about_the_file_is_refused() {
        // Two replies, not three: the pull and the question about which file
        // arrived. Queueing a third would be queueing the create this fetch
        // must never reach — and the stand-in waits for one connection per
        // reply, so an unused third reply is a test that hangs rather than one
        // that fails.
        let (url, server) = serving_each(&[
            (200, "{\"status\":\"success\"}\n"),
            (200, SHOWN_SAYING_NOTHING),
        ]);
        let refused = Ollama::at(&url, catalogue())
            .fetch("teuken-7b-instruct", &mut Progress::ignored())
            .unwrap_err();
        assert_eq!(
            refused,
            RuntimeError::PinNotChecked("teuken-7b-instruct".to_owned())
        );
        let sent = server.join().unwrap();
        assert_eq!(sent.len(), 2, "{sent:?}");
    }

    /// **An entry whose file carries no chat template is fetched with its
    /// publisher's** — Teuken, whose GGUF has none (`docs/quirks.md`), and only
    /// Teuken: every entry is walked, and only one names a template.
    #[test]
    fn only_an_entry_that_names_a_template_is_given_one_when_fetched() {
        let shipped = catalogue();
        let with_a_template: Vec<&str> = shipped
            .models
            .iter()
            .filter(|m| m.chat_template.is_some())
            .map(|m| m.id.as_str())
            .collect();
        assert_eq!(with_a_template, vec!["teuken-7b-instruct"]);

        // Three requests, not two: this entry states a `[model.requantised]`
        // block, so between the pull and the create the runtime is asked which
        // file it ended up with. The answer names the digest the catalogue
        // pins, which is the case where the fetch carries on.
        let (url, server) = serving_each(&[
            (200, "{\"status\":\"success\"}\n"),
            (200, SHOWN_HOLDING_THE_PINNED_FILE),
            (200, r#"{"status":"success"}"#),
        ]);
        Ollama::at(&url, shipped.clone())
            .fetch("teuken-7b-instruct", &mut Progress::ignored())
            .unwrap();
        let sent = server.join().unwrap();
        let body = sent
            .get(2)
            .unwrap()
            .split_once("\r\n\r\n")
            .map(|(_, body)| body)
            .unwrap();
        let body: serde_json::Value = serde_json::from_str(body).unwrap();
        let template = shipped
            .get("teuken-7b-instruct")
            .and_then(|m| m.chat_template.as_ref())
            .unwrap();
        assert_eq!(
            body.get("template").unwrap(),
            &serde_json::Value::from(template.text.clone())
        );
        assert_eq!(
            body.get("from").unwrap(),
            "hf.co/mradermacher/Teuken-7B-instruct-commercial-v0.4-GGUF:Q4_K_M"
        );
    }

    /// A malformed line mid-download must not abandon several gigabytes of
    /// progress. The next line almost always parses.
    #[test]
    fn a_line_that_does_not_parse_does_not_end_the_download() {
        let (url, server) = serving_each(&[
            (
                200,
                "{\"completed\":100,\"total\":400}\n\
                 not json at all\n\
                 {\"completed\":400,\"total\":400}\n",
            ),
            (200, r#"{"status":"success"}"#),
        ]);
        let mut seen = 0;
        let result = Ollama::at(&url, catalogue()).fetch("mistral-7b-instruct", &mut |_p| {
            seen += 1;
        });
        server.join().unwrap();
        assert!(result.is_ok(), "{result:?}");
        assert_eq!(seen, 2);
    }

    /// An error line in the stream is a failed download, and our error says so
    /// without repeating whatever the runtime wrote.
    #[test]
    fn an_error_in_the_stream_fails_the_fetch_without_quoting_the_runtime() {
        let (url, server) = serving(
            r#"{"error":"pull model manifest: file does not exist"}"#,
            200,
        );
        let err = Ollama::at(&url, catalogue())
            .fetch("mistral-7b-instruct", &mut Progress::ignored())
            .unwrap_err();
        server.join().unwrap();
        assert_eq!(err, RuntimeError::DownloadIncomplete);
        // The runtime's own words must not travel, and there is now nowhere
        // for them to travel in: the variant carries nothing.
        let said = err.said(&crate::testing::in_english());
        assert!(!said.text().contains("manifest"), "{said}");
    }

    #[test]
    fn removing_something_that_is_not_there_says_so_rather_than_failing_vaguely() {
        let (url, server) = serving(r#"{"error":"model not found"}"#, 404);
        let err = Ollama::at(&url, catalogue())
            .remove("mistral-7b-instruct")
            .unwrap_err();
        server.join().unwrap();
        assert!(
            matches!(&err, RuntimeError::NotInstalled(id) if id == "mistral-7b-instruct"),
            "{err:?}"
        );
    }

    #[test]
    fn a_runtime_that_is_not_listening_is_unreachable_rather_than_a_panic() {
        let ollama = Ollama::at("http://127.0.0.1:1", catalogue());
        assert!(matches!(
            ollama.installed().unwrap_err(),
            RuntimeError::Unreachable
        ));
    }

    /// Loading and unloading are the same Ollama endpoint with opposite
    /// keep-alives — it has neither a load nor an unload call. What these two
    /// tests hold is that the difference is a keep-alive and that it never
    /// leaves this file.
    #[test]
    fn loading_asks_the_runtime_to_hold_the_model_in_video_memory() {
        let (url, server) = serving("{}", 200);
        let result = Ollama::at(&url, catalogue()).load("mistral-7b-instruct");
        let request = server.join().unwrap();
        assert!(result.is_ok(), "{result:?}");
        assert!(request.contains("mistral-7b-instruct:latest"), "{request}");
        assert!(
            without_spaces(&request).contains(&format!(r#""keep_alive":"{DEFAULT_KEEP_ALIVE}""#)),
            "loading must ask for a non-zero keep-alive: {request}"
        );
    }

    #[test]
    fn unloading_asks_the_runtime_to_stop_holding_video_memory() {
        let (url, server) = serving("{}", 200);
        let result = Ollama::at(&url, catalogue()).unload("mistral-7b-instruct");
        let request = server.join().unwrap();
        assert!(result.is_ok(), "{result:?}");
        assert!(request.contains("mistral-7b-instruct:latest"), "{request}");
        assert!(
            without_spaces(&request).contains(r#""keep_alive":"0""#),
            "unloading must ask for a zero keep-alive: {request}"
        );
    }

    /// **The path the zero-egress promise is about.** A question goes to a
    /// model on this machine, the answer comes back as the model wrote it, and
    /// the whole of what went out is the model, the one message and
    /// `stream: false` — no preamble alo OS composed, no previous turn, nothing
    /// identifying the machine or the person.
    #[test]
    fn a_question_goes_into_the_body_and_nowhere_else() {
        let (url, server) = serving(
            r#"{"model":"mistral-7b-instruct:latest","message":{"role":"assistant","content":"No, not without written consent."},"done":true}"#,
            200,
        );
        let answer = Ollama::at(&url, catalogue())
            .answers("may the tenant sublet?", "mistral-7b-instruct")
            .unwrap();
        let request = server.join().unwrap();

        assert_eq!(answer, "No, not without written consent.");
        assert!(request.starts_with("POST /api/chat "), "{request}");
        let body = request.split("\r\n\r\n").nth(1).unwrap_or_default();
        let sent: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(
            sent,
            serde_json::json!({
                // Ollama's naming convention, applied here and nowhere else.
                "model": "mistral-7b-instruct:latest",
                "messages": [{"role": "user", "content": "may the tenant sublet?"}],
                "stream": false,
            }),
            "{request}"
        );
    }

    /// **A question is somebody's own words and this crate keeps none of
    /// them.** It goes into one body; nothing that comes back out of this file
    /// carries it, because no error here has a field it could travel in.
    #[test]
    fn a_question_never_comes_back_out_in_anything_this_file_answers_with() {
        let (url, server) = serving(r#"{"error":"model not found"}"#, 404);
        let err = Ollama::at(&url, catalogue())
            .answers("what is in this contract?", "mistral-7b-instruct")
            .unwrap_err();
        server.join().unwrap();
        assert!(
            matches!(&err, RuntimeError::NotInstalled(id) if id == "mistral-7b-instruct"),
            "{err:?}"
        );
        let rendered = format!("{err:?}");
        assert!(!rendered.contains("contract"), "{rendered}");
        let said = err.said(&crate::testing::in_english()).text().to_owned();
        assert!(!said.contains("contract"), "{said}");
    }

    /// **Asking is not gated by the catalogue, and downloading is.** A model on
    /// somebody's own disk is theirs; the licence promise is about what alo OS
    /// offers to fetch, and refusing to use what is already installed would be
    /// this system overruling the owner of the machine.
    #[test]
    fn a_model_the_catalogue_does_not_list_can_still_be_asked_if_it_is_installed() {
        let (url, server) = serving(
            r#"{"message":{"role":"assistant","content":"it answered anyway"}}"#,
            200,
        );
        let ollama = Ollama::at(&url, catalogue());
        let answer = ollama.answers("anything?", "some-uncurated-model").unwrap();
        let request = server.join().unwrap();
        assert_eq!(answer, "it answered anyway");
        assert!(request.contains("some-uncurated-model:latest"), "{request}");

        // And the gate that is a gate is still shut, on a runtime nothing is
        // listening on — so a leak would read as Unreachable rather than pass.
        let err = Ollama::at("http://127.0.0.1:1", catalogue())
            .fetch("some-uncurated-model", &mut Progress::ignored())
            .unwrap_err();
        assert!(matches!(err, RuntimeError::NotOffered(_)), "{err:?}");
    }

    /// Something answered, and it was not an answer. Four shapes of the same
    /// sentence to the person who asked: they asked, and nothing came back.
    #[test]
    fn a_reply_that_is_not_an_answer_is_unusable_rather_than_shown() {
        for (body, status) in [
            ("<!doctype html><title>Welcome</title>", 200),
            (r#"{"done":true}"#, 200),
            (r#"{"message":{"role":"assistant","content":"   "}}"#, 200),
            (r#"{"error":"something at that end"}"#, 500),
        ] {
            let (url, server) = serving(body, status);
            let err = Ollama::at(&url, catalogue())
                .answers("may the tenant sublet?", "mistral-7b-instruct")
                .unwrap_err();
            server.join().unwrap();
            assert_eq!(err, RuntimeError::Unusable, "{body}");
            // Whatever it said about itself does not travel into what a person
            // reads: there is nowhere in the variant for it to travel.
            let said = err.said(&crate::testing::in_english()).text().to_owned();
            assert!(!said.contains("something at that end"), "{said}");
        }
    }

    /// A runtime that is not running is not a model that is slow, and the two
    /// sentences send a person to different places.
    #[test]
    fn nothing_listening_is_unreachable_and_not_a_model_thinking() {
        let err = Ollama::at("http://127.0.0.1:1", catalogue())
            .answers("may the tenant sublet?", "mistral-7b-instruct")
            .unwrap_err();
        assert_eq!(err, RuntimeError::Unreachable);
        assert_ne!(err, RuntimeError::TookTooLong);
        let said = RuntimeError::TookTooLong.said(&crate::testing::in_english());
        assert!(said.text().contains("did not answer in the time"), "{said}");
    }

    #[test]
    fn loading_something_not_installed_says_so_rather_than_failing_vaguely() {
        let (url, server) = serving(r#"{"error":"model not found"}"#, 404);
        let err = Ollama::at(&url, catalogue())
            .load("mistral-7b-instruct")
            .unwrap_err();
        server.join().unwrap();
        assert!(
            matches!(&err, RuntimeError::NotInstalled(id) if id == "mistral-7b-instruct"),
            "{err:?}"
        );
    }

    /// A file on this disk, for a door that refuses anything else.
    fn a_file_of_our_own(called: &str) -> std::path::PathBuf {
        let at = std::env::temp_dir().join(format!("alo-brought-{called}-{}", std::process::id()));
        std::fs::write(&at, b"GGUF").unwrap();
        at
    }

    /// Weights naming a path that is not this machine's, which `Weights::at`
    /// could not make and a settings file read back from disk can carry: the
    /// file it named may have been deleted, or the line edited by hand.
    fn weights_naming(path: &str) -> Weights {
        let mut weights = Weights::checked("borrowed", 4_000).unwrap();
        weights.file = Some(std::path::PathBuf::from(path));
        weights
    }

    /// **The file goes to the runtime by its digest, and the model is made from
    /// that blob — the two requests the pinned runtime takes.**
    ///
    /// Ollama 0.34.0 refuses the one-line Modelfile this door used to send
    /// (`handing_over` has its words), so what is asserted here is the road it
    /// accepts: a `HEAD` asking whether the bytes are held, the bytes themselves
    /// when they are not, and a create naming the file by digest. The bodies are
    /// parsed rather than searched, and the create is held to carrying nothing
    /// but the file — no template, no system prompt, no parameters — because
    /// anything further would be this machine putting words in a model somebody
    /// else brought.
    #[test]
    fn bringing_a_file_hands_it_over_by_digest_and_makes_the_model_from_it() {
        let file = a_file_of_our_own("plain");
        let weights = Weights::at(&file).unwrap();
        let digest = crate::handing_over::digest_of(&file).unwrap();
        let (url, server) = serving_each(&[(404, ""), (201, ""), (200, r#"{"status":"success"}"#)]);

        Ollama::at(&url, catalogue()).bring(&weights).unwrap();

        let sent = server.join().unwrap();
        assert_eq!(
            sent.len(),
            3,
            "the runtime was not asked three times: {sent:?}"
        );
        let (asked, handed, made) = (
            sent.first().unwrap(),
            sent.get(1).unwrap(),
            sent.get(2).unwrap(),
        );
        let blob = format!("/api/blobs/sha256:{digest}");
        assert!(asked.starts_with(&format!("HEAD {blob} ")), "{asked}");
        assert!(handed.starts_with(&format!("POST {blob} ")), "{handed}");
        assert!(
            handed.ends_with("GGUF"),
            "the bytes sent are not the file's own: {handed}"
        );
        assert!(made.starts_with("POST /api/create "), "{made}");

        let body = made.split_once("\r\n\r\n").map(|(_, body)| body).unwrap();
        let body: serde_json::Value = serde_json::from_str(body).unwrap();
        let file_name = file.file_name().unwrap().to_str().unwrap();
        assert_eq!(
            body.pointer(&format!("/files/{file_name}")).unwrap(),
            &format!("sha256:{digest}")
        );
        assert!(
            body.get("model")
                .unwrap()
                .as_str()
                .unwrap_or_default()
                .contains(&weights.id),
            "the runtime is not told which id to answer to: {body}"
        );
        for putting_words_in_it in ["modelfile", "template", "system", "parameters", "from"] {
            assert!(
                body.get(putting_words_in_it).is_none(),
                "the create carries `{putting_words_in_it}`: {body}"
            );
        }
        drop(std::fs::remove_file(&file));
    }

    /// **Bytes the runtime already holds are not sent again.** A second bring
    /// of the same file is a `HEAD` and a create, not another copy of gigabytes.
    #[test]
    fn a_file_the_runtime_already_holds_is_not_sent_again() {
        let file = a_file_of_our_own("held");
        let weights = Weights::at(&file).unwrap();
        let (url, server) = serving_each(&[(200, ""), (200, r#"{"status":"success"}"#)]);

        Ollama::at(&url, catalogue()).bring(&weights).unwrap();

        let sent = server.join().unwrap();
        assert_eq!(sent.len(), 2, "the runtime was not asked twice: {sent:?}");
        let (asked, made) = (sent.first().unwrap(), sent.get(1).unwrap());
        assert!(asked.starts_with("HEAD /api/blobs/sha256:"), "{sent:?}");
        assert!(made.starts_with("POST /api/create "), "{sent:?}");
        drop(std::fs::remove_file(&file));
    }

    /// **Weights that name no file are refused at the door**, because there is
    /// nothing to tell the runtime about — a refusal about which door was used
    /// rather than about anything the person did.
    #[test]
    fn weights_with_no_file_are_refused_without_asking_the_runtime() {
        // Port 1 is not listening: had this leaked past the check we would see
        // `Unreachable` instead of the refusal.
        let weights = Weights::checked("mistral-7b-instruct", 4_000).unwrap();
        let refused = Ollama::at("http://127.0.0.1:1", catalogue())
            .bring(&weights)
            .unwrap_err();
        assert_eq!(
            refused,
            RuntimeError::NothingToBring("mistral-7b-instruct".to_owned())
        );
    }

    /// **A name that is not a path on this disk is refused before anything is
    /// sent**, and this is the refusal that keeps *run the weights you already
    /// have* from becoming a download nobody asked for: to the runtime, a bare
    /// model name is an instruction to fetch from a publisher.
    ///
    /// The door cannot lean on `Weights::at` having checked: these weights come
    /// back out of a settings file, where the path may since have been removed
    /// or the line edited by hand.
    #[test]
    fn a_path_that_is_not_a_file_on_this_disk_is_refused_before_anything_is_sent() {
        for not_a_file in ["mistral", "./relative.gguf", "/does/not/exist.gguf"] {
            let refused = Ollama::at("http://127.0.0.1:1", catalogue())
                .bring(&weights_naming(not_a_file))
                .unwrap_err();
            assert!(
                matches!(refused, RuntimeError::NotAPathOnThisDisk(_)),
                "`{not_a_file}` reached the runtime, where a bare name is a publisher's to \
                 fetch: {refused:?}"
            );
        }
    }

    /// **A file the runtime will not take is the runtime's own refusal**,
    /// carried rather than reworded — an architecture it does not know, a file
    /// that is not what its name says.
    #[test]
    fn a_file_the_runtime_will_not_take_is_carried_as_a_refusal() {
        let file = a_file_of_our_own("refused");
        let weights = Weights::at(&file).unwrap();
        // What 0.34.0 answers a create from bytes that are not weights.
        let (url, server) = serving_each(&[(200, ""), (500, r#"{"error":"unexpected EOF"}"#)]);

        let refused = Ollama::at(&url, catalogue()).bring(&weights).unwrap_err();

        drop(server.join());
        assert_eq!(refused, RuntimeError::Unusable);
        drop(std::fs::remove_file(&file));
    }

    /// **Bytes the runtime will not store are refused too**, and nothing is
    /// created from them: the runtime checks the digest it was given against
    /// the bytes, and a file that changed while it was being sent fails that.
    #[test]
    fn bytes_the_runtime_will_not_store_are_refused_and_nothing_is_made() {
        let file = a_file_of_our_own("mismatch");
        let weights = Weights::at(&file).unwrap();
        let (url, server) = serving_each(&[(404, ""), (400, r#"{"error":"digest mismatch"}"#)]);

        let refused = Ollama::at(&url, catalogue()).bring(&weights).unwrap_err();

        let sent = server.join().unwrap();
        assert_eq!(refused, RuntimeError::Unusable);
        assert_eq!(
            sent.len(),
            2,
            "a model was made from bytes the runtime refused: {sent:?}"
        );
        drop(std::fs::remove_file(&file));
    }

    /// And a runtime that is not there at all is what it always was, so a
    /// person who brought a file while the runtime was down is told the true
    /// thing rather than something about their file.
    #[test]
    fn a_runtime_that_is_not_there_is_unreachable() {
        let file = a_file_of_our_own("nobody-home");
        let weights = Weights::at(&file).unwrap();
        let refused = Ollama::at("http://127.0.0.1:1", catalogue())
            .bring(&weights)
            .unwrap_err();
        assert_eq!(refused, RuntimeError::Unreachable);
        drop(std::fs::remove_file(&file));
    }
}
