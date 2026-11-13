//! **What GitHub's own command, `gh`, said**, read from its own words: the
//! one-time code a sign-in prints, the page it is typed on, the login a
//! finished sign-in names, and the last line of a failure.
//!
//! Each reading is of a line `gh auth login --web` prints, as gh 2.x prints it
//! with colour switched off:
//!
//! ```text
//! ! First copy your one-time code: 4F2A-9C1D
//! Open this URL to continue in your web browser: https://github.com/login/device
//! ✓ Authentication complete.
//! ✓ Logged in as anna
//! ```
//!
//! **Nothing is guessed.** A line that does not carry what is looked for is
//! not read as carrying it, and a code or a login that is not shaped like one
//! is not one — so nothing gh prints can put words of its own on the sheet or
//! in the record.

/// The page a one-time code is typed on, where gh did not name one.
pub const THE_DEVICE_PAGE: &str = "https://github.com/login/device";

/// The one-time code on `line`, where it carries one: two groups of four
/// capital letters or digits, joined by a hyphen.
#[must_use]
pub fn the_code_on(line: &str) -> Option<String> {
    let line = plain(line);
    let (_, after) = line.split_once("one-time code:")?;
    let code = after.split_whitespace().next()?;
    let (left, right) = code.split_once('-')?;
    let shaped = |part: &str| {
        part.len() == 4
            && part
                .chars()
                .all(|it| it.is_ascii_uppercase() || it.is_ascii_digit())
    };
    (shaped(left) && shaped(right)).then(|| code.to_owned())
}

/// The page on `line`, where it names one on `https://github.com/`.
#[must_use]
pub fn the_page_on(line: &str) -> Option<String> {
    let line = plain(line);
    line.split_whitespace()
        .find(|word| word.starts_with("https://github.com/"))
        .map(str::to_owned)
}

/// The login on `line`, where it says *Logged in as*.
#[must_use]
pub fn the_login_on(line: &str) -> Option<String> {
    let line = plain(line);
    let (_, after) = line.split_once("Logged in as ")?;
    let login = after.split_whitespace().next()?;
    a_login(login)
}

/// `said`, where it is shaped like a GitHub login: letters, digits, hyphens
/// and underscores, not starting with a hyphen, at most 100 long — the
/// record's own rule (`alo_record::at_github::LONGEST_LOGIN`), which allows
/// the underscore an organisation's managed accounts carry.
#[must_use]
pub fn a_login(said: &str) -> Option<String> {
    let said = said.trim();
    let shaped = !said.is_empty()
        && said.len() <= 100
        && !said.starts_with('-')
        && said
            .chars()
            .all(|it| it.is_ascii_alphanumeric() || it == '-' || it == '_');
    shaped.then(|| said.to_owned())
}

/// The last line `gh` printed with words on it, colour taken out — what a
/// failure is said with.
#[must_use]
pub fn the_last_line(lines: &[String]) -> Option<String> {
    lines
        .iter()
        .rev()
        .map(|line| plain(line))
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_owned())
}

/// `line` with terminal colour sequences taken out, in case a gh ignores
/// `NO_COLOR`.
fn plain(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(it) = chars.next() {
        if it == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                let _ = chars.next();
                for after in chars.by_ref() {
                    if after.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(it);
    }
    out
}

#[cfg(test)]
#[path = "what_gh_says_tests.rs"]
mod tests;
