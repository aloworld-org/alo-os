//! **The words the scripted answers are said in**: the vocabularies of every
//! crate whose sentence a road's answer carries, in English.
//!
//! English because the set's requests are English, and the model is told the
//! machine's answers in the language the machine speaks to it.

use alo_strings::{Strings, Vocabulary};

/// A vocabulary that would not join, by the crate it came from.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the words of {from} could not be joined: {why}")]
pub struct NotTheWords {
    /// The crate whose vocabulary it was.
    from: &'static str,
    /// What went wrong.
    why: String,
}

/// Every word an answer in the set says: the capability's (the full-access
/// sentence), the applications', the adapters' (reading, typing), the
/// publishing's, the asking's and the models' (where an answer came from).
///
/// # Errors
/// [`NotTheWords`], naming the vocabulary that did not declare or join.
pub fn the_words() -> Result<Strings, NotTheWords> {
    let mut vocabulary = Vocabulary::empty();
    let joined = |vocabulary: &mut Vocabulary,
                  from: &'static str,
                  declared: Result<Vocabulary, String>|
     -> Result<(), NotTheWords> {
        let other = declared.map_err(|why| NotTheWords { from, why })?;
        vocabulary.join(other).map_err(|why| NotTheWords {
            from,
            why: format!("{why:?}"),
        })
    };
    let said = |why: &dyn std::fmt::Debug| format!("{why:?}");
    joined(
        &mut vocabulary,
        "alo-capability",
        alo_capability::words::capability_words().map_err(|why| said(&why)),
    )?;
    joined(
        &mut vocabulary,
        "alo-applications",
        alo_applications::words::application_words().map_err(|why| said(&why)),
    )?;
    joined(
        &mut vocabulary,
        "alo-adapters",
        alo_adapters::words::adapter_words().map_err(|why| said(&why)),
    )?;
    joined(
        &mut vocabulary,
        "alo-publishing",
        alo_publishing::words::publishing_words().map_err(|why| said(&why)),
    )?;
    joined(
        &mut vocabulary,
        "alo-asking",
        alo_asking::words::asking_words().map_err(|why| said(&why)),
    )?;
    joined(
        &mut vocabulary,
        "alo-models",
        alo_models::words::model_words().map_err(|why| said(&why)),
    )?;
    Ok(Strings::of(vocabulary))
}

#[cfg(test)]
mod tests {
    use super::the_words;

    #[test]
    fn every_vocabulary_an_answer_needs_joins() {
        assert!(the_words().is_ok(), "{:?}", the_words().err());
    }
}
