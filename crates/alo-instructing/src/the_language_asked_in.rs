//! **Which of the 24 languages a request is written in, decided on this
//! machine.**
//!
//! Task 5 of `docs/autonomy/v0-5-access-and-language-plan.md`: *the agent
//! answers in the language it was asked in*, and the language is the
//! **question's**, not the shell's — a person whose machine is in English may
//! ask in German, and answering them in English would be the machine deciding
//! which language they think in.
//!
//! # Why this is written here rather than rented
//!
//! Nothing leaves the machine to decide it (law 1), and a request is one
//! sentence: too short for the statistical models a library would carry, and
//! too private to send anywhere. So this is deliberately small — the script a
//! request is written in, and the short words a language uses that its
//! neighbours do not. It answers [`None`] where it cannot tell, and the clause
//! is then left out rather than a guess being put in front of a model.
//!
//! **What it is not.** It is not a language identifier for text in general, it
//! knows only the 24 official languages `alo-strings` carries, and it is wrong
//! on short requests that share their words with a neighbour — *Kontakt* is
//! German, Danish, Polish and Slovak at once. The report that comes with this
//! names the cases it gets wrong rather than claiming a number it does not have.

use alo_strings::Language;

/// **The language a request is written in**, or [`None`] where the request is
/// too short, is in no language this machine carries, or could be two of them.
#[must_use]
pub fn the_language_of(request: &str) -> Option<Language> {
    let words = words_of(request);
    if words.is_empty() {
        return None;
    }
    if let Some(tag) = by_the_script(request) {
        return Language::written(tag).ok();
    }
    let mut best: Option<(&str, usize)> = None;
    let mut tied = false;
    for (tag, marks) in THE_SHORT_WORDS {
        // A letter only one of the 24 writes is worth more than a short word
        // several of them share: `ř` is Czech wherever it stands, while `je` is
        // Czech, Slovak, Croatian and Slovene at once. Capped, so one long text
        // full of them cannot outweigh the words entirely.
        let score = marks
            .iter()
            .filter(|mark| words.iter().any(|word| word == *mark))
            .count()
            + 2 * letters_only_it_uses(request, tag).min(3);
        match best {
            Some((_, most)) if score > most => {
                best = Some((tag, score));
                tied = false;
            }
            Some((_, most)) if score == most && score > 0 => tied = true,
            Some(_) => {}
            None => best = Some((tag, score)),
        }
    }
    match best {
        Some((tag, score)) if score > 0 && !tied => Language::written(tag).ok(),
        _ => None,
    }
}

/// The words of a request, lowercased, punctuation dropped.
fn words_of(request: &str) -> Vec<String> {
    request
        .split(|letter: char| !letter.is_alphabetic() && letter != '\'')
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// A language its script alone names: Bulgarian is the only one of the 24
/// written in Cyrillic and Greek the only one in Greek.
///
/// **Most of the letters, not one of them.** A text in Latin letters with a
/// stray Cyrillic letter inside a word is not Bulgarian, and a reader that said
/// it was would be wrong where it matters: on 2026-09-16 the small model
/// answered in Estonian and in Slovak with single Cyrillic letters inside Latin
/// words (`prieponе`), and the first version of this read both as Bulgarian.
fn by_the_script(request: &str) -> Option<&'static str> {
    let letters = request
        .chars()
        .filter(|letter| letter.is_alphabetic())
        .count();
    if letters == 0 {
        return None;
    }
    let share_of = |from: char, to: char| {
        request
            .chars()
            .filter(|letter| (from..=to).contains(letter))
            .count()
            * 100
            / letters
    };
    match (
        share_of('\u{0400}', '\u{04FF}'),
        share_of('\u{0370}', '\u{03FF}'),
    ) {
        (cyrillic, _) if cyrillic >= MOSTLY => Some("bg"),
        (_, greek) if greek >= MOSTLY => Some("el"),
        _ => None,
    }
}

/// What share of a text's letters a script must be for the text to be in it.
const MOSTLY: usize = 40;

/// How many letters the request uses that, among these 24, only this language
/// writes — one mark each, so a sentence with none is decided by its words.
fn letters_only_it_uses(request: &str, tag: &str) -> usize {
    let only_theirs: &[char] = match tag {
        "pl" => &['ł', 'ż', 'ę', 'ą', 'ś', 'ć', 'ń', 'ź'],
        "cs" => &['ř', 'ů'],
        "sk" => &['ĺ', 'ŕ', 'ľ'],
        "hu" => &['ő', 'ű'],
        "ro" => &['ț', 'ș', 'ă'],
        "lv" => &['ģ', 'ķ', 'ļ', 'ņ'],
        "lt" => &['ų', 'ė', 'į'],
        "mt" => &['ġ', 'ħ', 'ż', 'ċ'],
        "et" => &['õ'],
        "da" => &['ø', 'å'],
        "sv" => &['ö', 'ä'],
        "de" => &['ß'],
        "es" => &['ñ', '¿', '¡'],
        "pt" => &['ã', 'õ'],
        "fr" => &['œ'],
        "sl" | "hr" => &['č', 'š', 'ž'],
        "ga" => &[],
        _ => &[],
    };
    request
        .chars()
        .filter(|letter| only_theirs.contains(&letter.to_lowercase().next().unwrap_or(*letter)))
        .count()
}

/// The short words each language uses that a person cannot write a sentence
/// without — articles, prepositions, the verb *to be*, the question words.
///
/// Chosen so that a request of a handful of words lands on one language, and
/// left honestly incomplete: where two languages share all of them, this
/// answers [`None`] and says nothing to the model.
const THE_SHORT_WORDS: [(&str, &[&str]); 24] = [
    ("bg", &["къде", "какво", "е", "на", "от", "за"]),
    (
        "hr",
        &[
            "gdje", "što", "je", "od", "za", "koji", "moja", "datoteka", "ćete", "kako", "ili",
            "vaš", "biti",
        ],
    ),
    (
        "cs",
        &[
            "kde", "co", "je", "od", "pro", "který", "soubor", "prosím", "se", "na", "být", "může",
            "vašem",
        ],
    ),
    (
        "da",
        &["hvor", "hvad", "er", "fra", "til", "den", "filen", "mine"],
    ),
    (
        "nl",
        &[
            "waar", "wat", "is", "van", "voor", "het", "de", "bestand", "mijn",
        ],
    ),
    (
        "en",
        &[
            "where", "what", "is", "the", "from", "for", "my", "file", "please",
        ],
    ),
    ("et", &["kus", "mis", "on", "minu", "fail", "arve", "palun"]),
    (
        "fi",
        &[
            "missä", "mikä", "on", "minun", "tiedosto", "lasku", "kiitos", "ja", "että", "ei",
            "mutta", "sen", "voi", "riippuu",
        ],
    ),
    (
        "fr",
        &[
            "où", "quoi", "est", "le", "la", "de", "pour", "mon", "fichier",
        ],
    ),
    (
        "de",
        &[
            "wo", "was", "ist", "der", "die", "das", "von", "für", "meine", "datei",
        ],
    ),
    ("el", &["πού", "τι", "είναι", "από", "για"]),
    (
        "hu",
        &["hol", "mi", "van", "az", "egy", "fájl", "számla", "kérem"],
    ),
    (
        "ga",
        &["cá", "cad", "tá", "an", "mo", "comhad", "sonrasc", "le"],
    ),
    (
        "it",
        &["dove", "cosa", "è", "il", "la", "di", "per", "mio", "file"],
    ),
    (
        "lv",
        &["kur", "kas", "ir", "no", "mans", "fails", "rēķins", "lūdzu"],
    ),
    (
        "lt",
        &["kur", "kas", "yra", "mano", "failas", "sąskaita", "prašau"],
    ),
    (
        "mt",
        &["fejn", "hu", "hi", "tal", "għal", "tiegħi", "fajl", "jekk"],
    ),
    (
        "pl",
        &[
            "gdzie", "co", "jest", "od", "dla", "mój", "plik", "faktura", "proszę",
        ],
    ),
    (
        "pt",
        &[
            "onde", "que", "está", "é", "do", "da", "para", "meu", "ficheiro",
        ],
    ),
    (
        "ro",
        &[
            "unde", "ce", "este", "din", "pentru", "meu", "fișier", "factura",
        ],
    ),
    (
        "sk",
        &[
            "kde", "čo", "je", "od", "pre", "môj", "súbor", "faktúra", "prosím", "sa", "že", "ale",
            "môže", "aké", "ktorej",
        ],
    ),
    (
        "sl",
        &[
            "kje", "kaj", "je", "od", "za", "moja", "datoteka", "račun", "prosim", "ki", "lahko",
            "tudi", "vaša", "način", "ste",
        ],
    ),
    (
        "es",
        &[
            "dónde", "qué", "está", "es", "el", "la", "de", "para", "mi", "archivo",
        ],
    ),
    (
        "sv",
        &[
            "var", "vad", "är", "från", "till", "min", "filen", "faktura", "tack",
        ],
    ),
];

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// One request a person might really type, per language, in the language.
    pub(crate) const A_REQUEST_IN_EACH: [(&str, &str); 24] = [
        ("bg", "къде е фактурата от Northstar?"),
        ("hr", "gdje je moja faktura od Northstara?"),
        ("cs", "kde je soubor s fakturou od Northstar?"),
        ("da", "hvor er fakturaen fra Northstar?"),
        ("nl", "waar is het bestand met de factuur van Northstar?"),
        ("en", "where is the invoice from Northstar?"),
        ("et", "kus on Northstari arve fail?"),
        ("fi", "missä on Northstarin lasku?"),
        ("fr", "où est la facture de Northstar ?"),
        ("de", "wo ist die Rechnung von Northstar?"),
        ("el", "πού είναι το τιμολόγιο από τη Northstar;"),
        ("hu", "hol van a Northstar számla fájl?"),
        ("ga", "cá bhfuil mo shonrasc ó Northstar?"),
        ("it", "dove è il file della fattura di Northstar?"),
        ("lv", "kur ir mans Northstar rēķins?"),
        ("lt", "kur yra mano Northstar sąskaita?"),
        ("mt", "fejn hu l-fajl tal-fattura tiegħi?"),
        ("pl", "gdzie jest faktura od Northstar?"),
        ("pt", "onde está a fatura da Northstar?"),
        ("ro", "unde este factura de la Northstar?"),
        ("sk", "kde je súbor s faktúrou od Northstar?"),
        ("sl", "kje je moja datoteka z računom?"),
        ("es", "¿dónde está la factura de Northstar?"),
        ("sv", "var är fakturan från Northstar?"),
    ];

    /// **A request in each of the 24 is read as that language, or as none** —
    /// and which ones it cannot tell apart is stated here rather than rounded
    /// away. Every language this misses is a language a person is answered in
    /// by the model's own choice, which is what happens today.
    #[test]
    fn a_request_in_each_language_is_read_as_that_language_or_as_none() {
        let mut read = 0;
        let mut unsure: Vec<&str> = Vec::new();
        let mut wrong: Vec<(&str, String)> = Vec::new();
        for (tag, request) in A_REQUEST_IN_EACH {
            match the_language_of(request) {
                Some(found) if found.tag() == tag => read += 1,
                Some(found) => wrong.push((tag, found.tag().to_owned())),
                None => unsure.push(tag),
            }
        }
        assert!(
            wrong.is_empty(),
            "a request read as the wrong language: {wrong:?}"
        );
        assert!(
            read >= 20,
            "only {read} of 24 were read; unsure about {unsure:?}"
        );
    }

    /// **Nothing is guessed.** An empty request, one that is only a name, and
    /// one in a language this machine does not carry are all [`None`].
    #[test]
    fn a_request_it_cannot_read_is_no_language_rather_than_a_guess() {
        for nothing in ["", "   ", "Northstar", "2026-09-16", "……"] {
            assert_eq!(the_language_of(nothing), None, "{nothing:?}");
        }
    }

    /// The script decides where a script belongs to one language only.
    #[test]
    fn a_script_only_one_of_the_24_uses_names_it_on_its_own() {
        assert_eq!(the_language_of("Здравейте").unwrap().tag(), "bg");
        assert_eq!(the_language_of("Γεια σας").unwrap().tag(), "el");
    }
}
