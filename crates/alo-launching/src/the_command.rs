//! An entry's `Exec` line, turned into the program and arguments it names.
//!
//! The Desktop Entry specification, *The Exec key*, applied as written:
//!
//! 1. the value is a **string**, so the key file's own escapes come first —
//!    `\s`, `\n`, `\t`, `\r` and `\\`;
//! 2. arguments are separated by spaces, and an argument holding a reserved
//!    character is **enclosed in double quotes**, inside which `\"`, `` \` ``,
//!    `\$` and `\\` stand for the character after the backslash;
//! 3. **field codes** are replaced. A press opens no file, so `%f`, `%F`, `%u`
//!    and `%U` become nothing — except that a link the person followed
//!    (`Filling::link`, since 2026-11-12) is what `%u` and `%U` become, as
//!    one argument; `%f` and `%F` stay empty for a link, because a link is
//!    not a file on this machine — and a file alo was approved to open
//!    (`Filling::file`, since 2026-11-14) is what `%f` and `%F` become, and
//!    `%u` and `%U` where no link is given; `%i` becomes `--icon` and the entry's icon,
//!    `%c` its name and `%k` where the entry is; the deprecated `%d`, `%D`,
//!    `%n`, `%N`, `%v` and `%m` are removed; `%%` is a percent sign.
//!
//! # What is refused, and why refusing is the right answer
//!
//! **A line the specification calls invalid is not run in whatever way it
//! happens to parse.** An unquoted reserved character, an unterminated quote,
//! a backslash before anything the quoting rule names, a field code inside
//! quotes (*undefined*, in the specification's word), `%F`, `%U` or `%i`
//! inside a longer argument, a field code the specification does not define,
//! a `%` with nothing after it — each is a [`WhyNotACommand`] and the
//! application is not started. Guessing would run a program its entry's own
//! author did not write down.
//!
//! **Nothing here is a shell.** The result is an argument vector handed to the
//! person's systemd manager as one; no line is ever composed for `sh` to read.
//! An entry whose author wanted a shell writes `sh -c "…"` itself, and that is
//! the author's program, installed on the machine, run as written.

/// What the field codes that are not files are replaced with.
#[derive(Debug, Clone, Copy, Default)]
pub struct Filling<'a> {
    /// The entry's `Icon`, for `%i`.
    pub icon: Option<&'a str>,
    /// The entry's `Name`, for `%c`.
    pub name: Option<&'a str>,
    /// Where the entry file is, for `%k`.
    pub location: Option<&'a str>,
    /// A link the person followed, for `%u` and `%U` — already checked to
    /// be an `http`, `https` or `mailto` address on one line, so it is one
    /// argument and can never be read as an option.
    pub link: Option<&'a str>,
    /// A file alo was approved to open in this application, for `%f` and `%F`
    /// — and for `%u` and `%U` where there is no link, because the
    /// specification lets a local file be passed to them *as a file path*.
    /// Already checked to be an absolute path on one line, so it is one
    /// argument and, beginning with `/`, can never be read as an option.
    pub file: Option<&'a str>,
}

/// Why an `Exec` line is not a command this machine will run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhyNotACommand {
    /// Nothing is left once the field codes are gone.
    Empty,
    /// A reserved character outside double quotes.
    ReservedOutsideQuotes(char),
    /// A double quote that is never closed.
    UnterminatedQuote,
    /// A backslash inside quotes before a character the rule does not name.
    BadEscapeInQuotes(char),
    /// A field code inside a quoted argument.
    FieldCodeInQuotes(char),
    /// `%F`, `%U` or `%i`, which stand for whole arguments, inside a longer one.
    ListCodeInsideAnArgument(char),
    /// A field code the specification does not define.
    UnknownFieldCode(char),
    /// A `%` at the very end of the line.
    LoneFieldCodeMark,
}

/// One piece of an argument, before the field codes are replaced.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Piece {
    /// Text that stands for itself.
    Literal(String),
    /// A field code, by its letter.
    Code(char),
}

/// One argument as written: its pieces, and whether any of it was quoted.
#[derive(Debug, Default)]
struct Word {
    /// What it is made of, in order.
    pieces: Vec<Piece>,
    /// Whether a quote took part in it — `""` is an empty argument, not none.
    quoted: bool,
}

impl Word {
    /// Add one character of literal text.
    fn push(&mut self, letter: char) {
        if let Some(Piece::Literal(text)) = self.pieces.last_mut() {
            text.push(letter);
        } else {
            self.pieces.push(Piece::Literal(letter.to_string()));
        }
    }
}

/// The characters the specification reserves: an argument holding one is
/// quoted.
const RESERVED: &[char] = &[
    '"', '\'', '\\', '>', '<', '~', '|', '&', ';', '$', '*', '?', '#', '(', ')', '`',
];

/// The program and its arguments, from an `Exec` value as the file wrote it.
///
/// # Errors
///
/// [`WhyNotACommand`] for a line the specification calls invalid, or one that
/// names nothing to run.
pub fn the_command_in(exec: &str, filling: &Filling<'_>) -> Result<Vec<String>, WhyNotACommand> {
    let words = the_words_in(&unescaped(exec))?;
    let mut command = Vec::new();
    for word in words {
        expand(word, filling, &mut command)?;
    }
    match command.first() {
        Some(program) if !program.is_empty() => Ok(command),
        _ => Err(WhyNotACommand::Empty),
    }
}

/// The key file's own string escapes, applied before the quoting rule.
///
/// An escape the key file does not define is kept as written, backslash and
/// all, so that the quoting rule below sees it — `\"` inside quotes is the
/// common case, and it is the quoting rule's to read.
fn unescaped(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut letters = value.chars();
    while let Some(letter) = letters.next() {
        if letter != '\\' {
            out.push(letter);
            continue;
        }
        match letters.next() {
            Some('s') => out.push(' '),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// The arguments as written, with their field codes still in them.
fn the_words_in(line: &str) -> Result<Vec<Word>, WhyNotACommand> {
    let mut words = Vec::new();
    let mut word: Option<Word> = None;
    let mut letters = line.chars();
    while let Some(letter) = letters.next() {
        match letter {
            ' ' | '\t' | '\n' | '\r' => {
                if let Some(done) = word.take() {
                    words.push(done);
                }
            }
            '"' => {
                let current = word.get_or_insert_with(Word::default);
                current.quoted = true;
                quoted(&mut letters, current)?;
            }
            '%' => {
                let current = word.get_or_insert_with(Word::default);
                match letters.next() {
                    Some('%') => current.push('%'),
                    Some(code) => current.pieces.push(Piece::Code(code)),
                    None => return Err(WhyNotACommand::LoneFieldCodeMark),
                }
            }
            reserved if RESERVED.contains(&reserved) => {
                return Err(WhyNotACommand::ReservedOutsideQuotes(reserved));
            }
            other => word.get_or_insert_with(Word::default).push(other),
        }
    }
    if let Some(done) = word {
        words.push(done);
    }
    Ok(words)
}

/// The inside of one pair of double quotes, up to and including the closing
/// one.
fn quoted(letters: &mut std::str::Chars<'_>, into: &mut Word) -> Result<(), WhyNotACommand> {
    while let Some(letter) = letters.next() {
        match letter {
            '"' => return Ok(()),
            '\\' => match letters.next() {
                Some(escaped @ ('"' | '`' | '$' | '\\')) => into.push(escaped),
                Some(other) => return Err(WhyNotACommand::BadEscapeInQuotes(other)),
                None => return Err(WhyNotACommand::UnterminatedQuote),
            },
            '%' => match letters.next() {
                Some('%') => into.push('%'),
                Some(code) => return Err(WhyNotACommand::FieldCodeInQuotes(code)),
                None => return Err(WhyNotACommand::UnterminatedQuote),
            },
            other => into.push(other),
        }
    }
    Err(WhyNotACommand::UnterminatedQuote)
}

/// One argument with its field codes replaced, added to the command.
fn expand(
    word: Word,
    filling: &Filling<'_>,
    command: &mut Vec<String>,
) -> Result<(), WhyNotACommand> {
    if let ([Piece::Code(code)], false) = (word.pieces.as_slice(), word.quoted) {
        return standing_alone(*code, filling, command);
    }
    let mut argument = String::new();
    for piece in word.pieces {
        match piece {
            Piece::Literal(text) => argument.push_str(&text),
            Piece::Code(code @ ('F' | 'U' | 'i')) => {
                return Err(WhyNotACommand::ListCodeInsideAnArgument(code));
            }
            Piece::Code(code) => argument.push_str(inside_an_argument(code, filling)?),
        }
    }
    command.push(argument);
    Ok(())
}

/// A field code that is a whole argument, which may become none, one or two.
fn standing_alone(
    code: char,
    filling: &Filling<'_>,
    command: &mut Vec<String>,
) -> Result<(), WhyNotACommand> {
    match code {
        'i' => {
            if let Some(icon) = filling.icon {
                command.push("--icon".to_owned());
                command.push(icon.to_owned());
            }
        }
        'U' => {
            if let Some(link) = filling.link.or(filling.file) {
                command.push(link.to_owned());
            }
        }
        'F' => {
            if let Some(file) = filling.file {
                command.push(file.to_owned());
            }
        }
        other => {
            let text = inside_an_argument(other, filling)?;
            if !text.is_empty() {
                command.push(text.to_owned());
            }
        }
    }
    Ok(())
}

/// What a field code that stands for at most one string becomes.
fn inside_an_argument<'a>(code: char, filling: &Filling<'a>) -> Result<&'a str, WhyNotACommand> {
    match code {
        'u' => Ok(filling.link.or(filling.file).unwrap_or("")),
        'f' => Ok(filling.file.unwrap_or("")),
        'd' | 'D' | 'n' | 'N' | 'v' | 'm' => Ok(""),
        'c' => Ok(filling.name.unwrap_or("")),
        'k' => Ok(filling.location.unwrap_or("")),
        other => Err(WhyNotACommand::UnknownFieldCode(other)),
    }
}

#[cfg(test)]
#[path = "the_command_tests.rs"]
mod tests;
